use std::{
    convert::Infallible, fmt::Debug, marker::PhantomData, mem::ManuallyDrop,
    panic::AssertUnwindSafe, ptr::NonNull,
};

use crate::{HasDependent, Owner};

/// A self-referential pair containing both some [`Owner`] and its
/// [`Dependent`](HasDependent::Dependent).
///
/// The owner must be provided to construct a [`Pair`], and the dependent is
/// immediately created (borrowing from the owner). Both are stored together in
/// the pair, which heap-allocates the owner so that the pair itself may be
/// moved freely without invalidating any references stored inside the
/// dependent.
///
/// Conceptually, the pair itself has ownership over the owner `O`, the owner is
/// immutably borrowed by the dependent for the lifetime of the pair, and the
/// dependent is owned by the pair and valid for the pair's lifetime.
pub struct Pair<O: Owner + ?Sized> {
    // Derived from a Box<O>
    // Immutably borrowed by `self.dependent` from construction until drop
    owner: NonNull<O>,

    // Type-erased Box<<O as HasDependent<'self.owner>>::Dependent>
    dependent: NonNull<()>,

    // Need invariance over O - if we were covariant or contravariant, two
    // different `O`s with two different `Owner` impls (and importantly, two
    // different associated types in HasDependent) which have a sub/supertype
    // relationship could be incorrectly treated as substitutable in a Pair.
    // That would lead to effectively an unchecked transmute of each field,
    // which would obviously be unsound.
    //
    // Coherence doesn't help us here, since there are types which are able to
    // have different impls of the same trait, but also have a subtype/supertype
    // relationship (namely, `fn(&'static T)` and `for<'a> fn(&'a T)` )
    prevent_covariance: PhantomData<*mut O>,
}

/// Creates a [`NonNull<T>`] from [`Box<T>`]. The returned NonNull is the same
/// pointer as the Box, and therefore comes with all of Box's representation
/// guarantees:
/// - The returned NonNull will be suitably aligned for T
/// - The returned NonNull will point to a valid T
/// - The returned NonNull was allocated with the [`Global`](std::alloc::Global)
///   allocator and a valid [`Layout`](std::alloc::Layout) for `T`.
fn non_null_from_box<T: ?Sized>(value: Box<T>) -> NonNull<T> {
    // See: https://github.com/rust-lang/rust/issues/47336#issuecomment-586578713
    NonNull::from(Box::leak(value))
}

impl<O: Owner + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you already have a [`Box`]ed owner, consider
    /// [`Pair::try_new_from_box_with_context`] to avoid redundant reallocation.
    ///
    /// If you don't need to provide any context, consider the convenience
    /// constructor [`Pair::try_new`], which doesn't require a context.
    ///
    /// If this construction can't fail, consider the convenience constructor
    /// [`Pair::new_with_context`], which returns `Self` directly.
    pub fn try_new_with_context(owner: O, context: O::Context<'_>) -> Result<Self, (O, O::Err)>
    where
        O: Sized,
    {
        Self::try_new_from_box_with_context(Box::new(owner), context)
            .map_err(|(owner, err)| (*owner, err))
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you have an unboxed `O` and only box it for this function, consider
    /// the convenience constructor [`Pair::try_new_with_context`], which boxes
    /// the owner for you.
    ///
    /// If you don't need to provide any context, consider the convenience
    /// constructor [`Pair::try_new_from_box`], which doesn't require a context.
    ///
    /// If this construction can't fail, consider the convenience constructor
    /// [`Pair::new_from_box_with_context`], which returns `Self` directly.
    pub fn try_new_from_box_with_context(
        owner: Box<O>,
        context: O::Context<'_>,
    ) -> Result<Self, (Box<O>, O::Err)> {
        // Convert owner into a NonNull, so we are no longer restricted by the
        // aliasing requirements of Box
        let owner = non_null_from_box(owner);

        // Borrow `owner` to construct `dependent`. This borrow conceptually
        // lasts from now until drop, where we will drop `dependent` and then
        // drop owner.

        // We're about to call `make_dependent(..)` - if it panics, we want to
        // be able to drop the boxed owner before unwinding the rest of the
        // stack to avoid unnecessarily leaking memory (and potentially other
        // resources).
        let maybe_dependent = match std::panic::catch_unwind(AssertUnwindSafe(|| {
            // SAFETY: `owner` was just converted from a valid Box, and inherits
            // the alignment and validity guarantees of Box. Additionally, the
            // value behind the pointer is currently not borrowed at all - this
            // marks the beginning of a shared borrow which will last until the
            // returned `Pair` is dropped (or immediately, if `make_dependent`
            // panics or returns an error).
            unsafe { owner.as_ref() }.make_dependent(context)
        })) {
            Ok(maybe_dependent) => maybe_dependent,
            Err(payload) => {
                // make_dependent panicked - drop the owner, then resume_unwind

                // SAFETY: `owner` was just created from a Box earlier in this
                // function, and not invalidated since then. Because we haven't
                // given away access to a `Self`, and the one borrow we took of
                // the dependent to pass to `make_dependent` has expired, we
                // know there are no outstanding borrows to owner. Therefore,
                // reconstructing the original Box<O> is okay.
                let owner: Box<O> = unsafe { Box::from_raw(owner.as_ptr()) };

                // If the owner's drop *also* panics, we're in a super weird
                // state. I think it makes more sense to resume_unwind with the
                // payload of the first panic (from `make_dependent`), so if the
                // owner's drop panics we just ignore it and continue on to
                // resume_unwind with `make_dependent`'s payload.
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| drop(owner)));

                // It's very important that we diverge here - carrying on to the
                // rest of this constructor would be unsound.
                std::panic::resume_unwind(payload);
            }
        };

        // If `make_dependent(..)` failed, early return out from this function.
        let dependent = match maybe_dependent {
            Ok(dependent) => dependent,
            Err(err) => {
                // SAFETY: `owner` was just created from a Box earlier in this
                // function, and not invalidated since then. Because we haven't
                // given away access to a `Self`, and the one borrow we took of
                // the dependent to pass to `make_dependent` has expired, we
                // know there are no outstanding borrows to owner. Therefore,
                // reconstructing the original Box<O> is okay.
                let owner: Box<O> = unsafe { Box::from_raw(owner.as_ptr()) };

                return Err((owner, err));
            }
        };

        // Type-erase dependent so it's inexpressible self-referential lifetime
        // goes away (we know that it's borrowing self.owner immutably from
        // construction (now) until drop)
        let dependent: NonNull<<O as HasDependent<'_>>::Dependent> =
            non_null_from_box(Box::new(dependent));
        let dependent: NonNull<()> = dependent.cast();

        Ok(Self {
            owner,
            dependent,
            prevent_covariance: PhantomData,
        })
    }

    /// Returns a reference to the owner.
    pub fn get_owner(&self) -> &O {
        // SAFETY: `self.owner` was originally converted from a valid Box, and
        // inherited the alignment and validity guarantees of Box - and neither
        // our code nor any of our exposed APIs could have invalidated those
        // since construction. Additionally, the value behind the pointer is
        // currently in a shared borrow state (no exclusive borrows, no other
        // code assuming unique ownership), and will be until the Pair is
        // dropped. Here, we only add another shared borrow.
        unsafe { self.owner.as_ref() }
    }

    /// Calls the given closure, providing shared access to the dependent, and
    /// returns the value computed by the closure.
    ///
    /// The closure must be able to work with a
    /// [`Dependent`](HasDependent::Dependent) with any arbitrary lifetime that
    /// lives at least as long as the borrow of `self`. This is important
    /// because the dependent may be invariant over its lifetime, and the
    /// correct lifetime (lasting from the construction of `self` until drop) is
    /// impossible to express. For dependent types covariant over their
    /// lifetime, the closure may simply return the reference to the dependent,
    /// which may then be used as if this function directly returned a
    /// reference.
    pub fn with_dependent<'self_borrow, F, T>(&'self_borrow self, f: F) -> T
    where
        F: for<'any> FnOnce(&'self_borrow <O as HasDependent<'any>>::Dependent) -> T,
    {
        // SAFETY: `self.dependent` was originally converted from a valid
        // Box<<O as HasDependent<'_>>::Dependent>, and type-erased to a
        // NonNull<()>. As such, it inherited the alignment and validity
        // guarantees of Box (for an <O as HasDependent<'_>>::Dependent) - and
        // neither our code nor any of our exposed APIs could have invalidated
        // those since construction. Additionally, because we have a shared
        // reference to self, we know that the value behind the pointer is
        // currently either not borrowed at all, or in a shared borrow state
        // (no exclusive borrows, no other code assuming unique ownership).
        // Here, we only either create the first shared borrow, or add another.
        let dependent: &<O as HasDependent<'_>>::Dependent = unsafe {
            self.dependent
                .cast::<<O as HasDependent<'_>>::Dependent>()
                .as_ref()
        };

        f(dependent)
    }

    /// Calls the given closure, providing exclusive access to the dependent,
    /// and returns the value computed by the closure.
    ///
    /// The closure must be able to work with a
    /// [`Dependent`](HasDependent::Dependent) with any arbitrary lifetime that
    /// lives at least as long as the borrow of `self`. This is important
    /// because mutable references are invariant over their type `T`, and the
    /// exact T here (a `Dependent` with a very specific lifetime lasting from
    /// the construction of `self` until drop) is impossible to express.
    pub fn with_dependent_mut<'self_borrow, F, T>(&'self_borrow mut self, f: F) -> T
    where
        F: for<'any> FnOnce(&'self_borrow mut <O as HasDependent<'any>>::Dependent) -> T,
    {
        // SAFETY: `self.dependent` was originally converted from a valid
        // Box<<O as HasDependent<'_>>::Dependent>, and type-erased to a
        // NonNull<()>. As such, it inherited the alignment and validity
        // guarantees of Box (for an <O as HasDependent<'_>>::Dependent) - and
        // neither our code nor any of our exposed APIs could have invalidated
        // those since construction. Additionally, because we have an exclusive
        // reference to self, we know that the value behind the pointer is
        // currently not borrowed at all, and can't be until the mutable borrow
        // of `self` expires.
        let dependent: &mut <O as HasDependent<'_>>::Dependent = unsafe {
            self.dependent
                .cast::<<O as HasDependent<'_>>::Dependent>()
                .as_mut()
        };

        f(dependent)
    }

    /// Consumes the [`Pair`], dropping the dependent and returning the owner.
    ///
    /// If you don't need the returned owner in a [`Box`], consider the
    /// convenience method [`Pair::into_owner`], which moves the owner out of
    /// the box for you.
    pub fn into_boxed_owner(self) -> Box<O> {
        // Prevent dropping `self` at the end of this scope - otherwise, the
        // Pair drop implementation would attempt to drop the owner and
        // dependent again, which would be... not good (unsound).
        //
        // It's important that we do this before calling the dependent's drop,
        // since a panic in that drop would otherwise cause a double free when
        // we attempt to drop the dependent again when dropping `self`.
        let this = ManuallyDrop::new(self);

        // SAFETY: `this.dependent` was originally created from a Box, and never
        // invalidated since then. Because we took ownership of `self`, we know
        // there are no outstanding borrows to the dependent. Therefore,
        // reconstructing the original Box<<O as HasDependent<'_>>::Dependent>
        // is okay.
        let dependent: Box<<O as HasDependent<'_>>::Dependent> = unsafe {
            Box::from_raw(
                this.dependent
                    .cast::<<O as HasDependent<'_>>::Dependent>()
                    .as_ptr(),
            )
        };

        // We're about to drop the dependent - if it panics, we want to be able
        // to drop the boxed owner before unwinding the rest of the stack to
        // avoid unnecessarily leaking memory (and potentially other resources).
        if let Err(payload) = std::panic::catch_unwind(AssertUnwindSafe(|| drop(dependent))) {
            // Dependent's drop panicked - drop the owner, then resume_unwind

            // SAFETY: `this.owner` was originally created from a Box, and never
            // invalidated since then. Because we took ownership of `self`, and
            // we just dropped the dependent (well, the drop panicked - but its
            // borrow of the owner has certainly expired), we know there are no
            // outstanding borrows to owner. Therefore, reconstructing the
            // original Box<O> is okay.
            let owner: Box<O> = unsafe { Box::from_raw(this.owner.as_ptr()) };

            // If the owner's drop *also* panics, we're in a super weird state.
            // I think it makes more sense to resume_unwind with the payload of
            // the first panic (from dependent's drop), so if the owner's drop
            // panics we just ignore it and continue on to resume_unwind with
            // the dependent's payload.
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| drop(owner)));

            // It's very important that we diverge here - carrying on to the
            // rest of this function would be unsound.
            std::panic::resume_unwind(payload);
        }

        // SAFETY: `this.owner` was originally created from a Box, and never
        // invalidated since then. Because we took ownership of `self`, and we
        // just dropped the dependent, we know there are no outstanding borrows
        // to owner. Therefore, reconstructing the original Box<O> is okay.
        unsafe { Box::from_raw(this.owner.as_ptr()) }
    }

    /// Consumes the [`Pair`], dropping the dependent and returning the owner.
    ///
    /// If you manually box the returned owner for your own purposes, consider
    /// [`Pair::into_boxed_owner`] to avoid redundant reallocation.
    pub fn into_owner(self) -> O
    where
        O: Sized,
    {
        *self.into_boxed_owner()
    }
}

impl<O: for<'any> Owner<Context<'any> = (), Err = Infallible> + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you already have a [`Box`]ed owner, consider [`Pair::new_from_box`]
    /// to avoid redundant reallocation.
    ///
    /// If you need to provide some additional arguments/context to this
    /// constructor, consider [`Pair::new_with_context`], which allows passing
    /// in additional data.
    ///
    /// If this construction can fail, consider [`Pair::try_new`], which returns
    /// a [`Result`].
    pub fn new(owner: O) -> Self
    where
        O: Sized,
    {
        Self::new_with_context(owner, ())
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you have an unboxed `O` and only box it for this function, consider
    /// the convenience constructor [`Pair::new`], which boxes the owner for
    /// you.
    ///
    /// If you need to provide some additional arguments/context to this
    /// constructor, consider [`Pair::new_from_box_with_context`], which allows
    /// passing in additional data.
    ///
    /// If this construction can fail, consider [`Pair::try_new_from_box`],
    /// which returns a [`Result`].
    pub fn new_from_box(owner: Box<O>) -> Self {
        Self::new_from_box_with_context(owner, ())
    }
}

impl<O: for<'any> Owner<Context<'any> = ()> + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you already have a [`Box`]ed owner, consider
    /// [`Pair::try_new_from_box`] to avoid redundant reallocation.
    ///
    /// If you need to provide some additional arguments/context to this
    /// constructor, consider [`Pair::try_new_with_context`], which allows
    /// passing in additional data.
    ///
    /// If this construction can't fail, consider the convenience constructor
    /// [`Pair::new`], which returns `Self` directly.
    pub fn try_new(owner: O) -> Result<Self, (O, O::Err)>
    where
        O: Sized,
    {
        Self::try_new_with_context(owner, ())
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you have an unboxed `O` and only box it for this function, consider
    /// the convenience constructor [`Pair::try_new`], which boxes the owner for
    /// you.
    ///
    /// If you need to provide some additional arguments/context to this
    /// constructor, consider [`Pair::try_new_from_box_with_context`], which
    /// allows passing in additional data.
    ///
    /// If this construction can't fail, consider the convenience constructor
    /// [`Pair::new_from_box`], which returns `Self` directly.
    pub fn try_new_from_box(owner: Box<O>) -> Result<Self, (Box<O>, O::Err)> {
        Self::try_new_from_box_with_context(owner, ())
    }
}

impl<O: Owner<Err = Infallible> + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you already have a [`Box`]ed owner, consider
    /// [`Pair::new_from_box_with_context`] to avoid redundant reallocation.
    ///
    /// If you don't need to provide any context, consider the convenience
    /// constructor [`Pair::new`], which doesn't require a context.
    ///
    /// If this construction can fail, consider [`Pair::try_new_with_context`],
    /// which returns a [`Result`].
    pub fn new_with_context(owner: O, context: O::Context<'_>) -> Self
    where
        O: Sized,
    {
        let Ok(pair) = Self::try_new_with_context(owner, context);
        pair
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you have an unboxed `O` and only box it for this function, consider
    /// the convenience constructor [`Pair::new_with_context`], which boxes the
    /// owner for you.
    ///
    /// If you don't need to provide any context, consider the convenience
    /// constructor [`Pair::new_from_box`], which doesn't require a context.
    ///
    /// If this construction can fail, consider
    /// [`Pair::try_new_from_box_with_context`], which returns a [`Result`].
    pub fn new_from_box_with_context(owner: Box<O>, context: O::Context<'_>) -> Self {
        let Ok(pair) = Self::try_new_from_box_with_context(owner, context);
        pair
    }
}

/// The [`Drop`] implementation for [`Pair`] will drop both the dependent and
/// the owner, in that order.
impl<O: Owner + ?Sized> Drop for Pair<O> {
    fn drop(&mut self) {
        // Drop the dependent `Box<<O as HasDependent<'_>>::Dependent>`

        // SAFETY: `self.dependent` was originally created from a Box, and never
        // invalidated since then. Because we are in drop, we know there are no
        // outstanding borrows to the dependent. Therefore, reconstructing the
        // original Box<<O as HasDependent<'_>>::Dependent> is okay.
        let dependent = unsafe {
            Box::from_raw(
                self.dependent
                    .cast::<<O as HasDependent<'_>>::Dependent>()
                    .as_ptr(),
            )
        };

        // We're about to drop the dependent - if it panics, we want to be able
        // to drop the boxed owner before unwinding the rest of the stack to
        // avoid unnecessarily leaking memory (and potentially other resources).
        if let Err(payload) = std::panic::catch_unwind(AssertUnwindSafe(|| drop(dependent))) {
            // Dependent's drop panicked - drop the owner, then resume_unwind

            // SAFETY: `this.owner` was originally created from a Box, and never
            // invalidated since then. Because we are in drop, and we just
            // dropped the dependent (well, the drop panicked - but its borrow
            // of the owner has certainly expired), we know there are no
            // outstanding borrows to owner. Therefore, reconstructing the
            // original Box<O> is okay.
            let owner: Box<O> = unsafe { Box::from_raw(self.owner.as_ptr()) };

            // If the owner's drop *also* panics, we're in a super weird state.
            // I think it makes more sense to resume_unwind with the payload of
            // the first panic (from dependent's drop), so if the owner's drop
            // panics we just ignore it and continue on to resume_unwind with
            // the dependent's payload.
            let _ = std::panic::catch_unwind(AssertUnwindSafe(|| drop(owner)));

            // It's very important that we diverge here - carrying on to the
            // rest of drop would be unsound.
            std::panic::resume_unwind(payload);
        }

        // Drop the owner `Box<O>`

        // SAFETY: `this.owner` was originally created from a Box, and never
        // invalidated since then. Because we are in drop, and we just dropped
        // the dependent, we know there are no outstanding borrows to owner.
        // Therefore, reconstructing the original Box<O> is okay.
        let owner = unsafe { Box::from_raw(self.owner.as_ptr()) };

        drop(owner);
    }
}

// SAFETY: `Pair` has no special thread-related invariants or requirements, so
// sending a `Pair` to another thread could only cause problems if sending
// either the owner or the dependent to another thread could cause problems
// (since both are semantically moved with and made accessible through the
// `Pair`).
unsafe impl<O: Owner + ?Sized> Send for Pair<O>
where
    O: Send,
    for<'any> <O as HasDependent<'any>>::Dependent: Send,
{
}

// SAFETY: `Pair` has no special thread-related invariants or requirements, so
// sharing a reference to a `Pair` across multiple threads could only cause
// problems if sharing a reference to either the owner or the dependent across
// multiple threads could cause problems (since references to both are made
// accessible through references to the `Pair`).
unsafe impl<O: Owner> Sync for Pair<O>
where
    O: Sync,
    for<'any> <O as HasDependent<'any>>::Dependent: Sync,
{
}

impl<O: Owner + Debug + ?Sized> Debug for Pair<O>
where
    for<'any> <O as HasDependent<'any>>::Dependent: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with_dependent(|dependent| {
            f.debug_struct("Pair")
                .field("owner", &self.get_owner())
                .field("dependent", dependent)
                .finish()
        })
    }
}

impl<O: for<'any> Owner<Context<'any> = (), Err = Infallible> + Default> Default for Pair<O> {
    fn default() -> Self {
        Self::new(O::default())
    }
}
