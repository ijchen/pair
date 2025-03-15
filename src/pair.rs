//! Defines [`Pair`], the primary abstraction provided by this crate.

use core::{convert::Infallible, fmt::Debug, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use alloc::boxed::Box;

use crate::{HasDependent, Owner, drop_guard::DropGuard};

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
///
/// # Constructors
///
/// There are many different constructors for `Pair`, each serving a different
/// use case. There are three relevant factors to consider when deciding which
/// constructor to use:
///
/// 1. Can [`make_dependent`](Owner::make_dependent) fail (return [`Err`])?
/// 2. Does `make_dependent` require additional context?
/// 3. Is your owner already stored in a [`Box`]?
///
/// The simplest constructor, which you should use if you answered "no" to all
/// of the above questions, is [`Pair::new`]. It takes an `O: Owner`, and gives
/// you a `Pair<O>` - doesn't get much easier than that!
///
/// If your `make_dependent` can fail (meaning [`Owner::Error`] is not
/// [`Infallible`]), you should use one of the `try_*` constructors.
///
/// If your `make_dependent` requires additional context (meaning
/// [`Owner::Context`] is not [`()`](prim@unit)), you should use one of the
/// `*_with_context` constructors.
///
/// If your owner is already stored in a `Box`, you should use one of the
/// `*_from_box` constructors.
///
/// Every combination of these is supported, up to the most powerful (and least
/// ergonomic) [`Pair::try_new_from_box_with_context`]. You should use the
/// simplest constructor you can for your implementation of `Owner`.
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

/// Creates a [`NonNull<T>`] from [`Box<T>`]. The returned `NonNull` is the same
/// pointer as the Box, and therefore comes with all of Box's representation
/// guarantees:
/// - The returned `NonNull` will be suitably aligned for T
/// - The returned `NonNull` will point to a valid T
/// - The returned `NonNull` was allocated with the
///   [`Global`](alloc::alloc::Global) allocator and a valid
///   [`Layout`](alloc::alloc::Layout) for `T`.
fn non_null_from_box<T: ?Sized>(value: Box<T>) -> NonNull<T> {
    // See: https://github.com/rust-lang/rust/issues/47336#issuecomment-586578713
    NonNull::from(Box::leak(value))
}

impl<O: Owner + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    ///
    /// # Errors
    /// If [`<O as Owner>::make_dependent`](Owner::make_dependent) returns an
    /// error.
    pub fn try_new_with_context(owner: O, context: O::Context<'_>) -> Result<Self, (O, O::Error)>
    where
        O: Sized,
    {
        Self::try_new_from_box_with_context(Box::new(owner), context)
            .map_err(|(owner, err)| (*owner, err))
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    ///
    /// # Errors
    /// If [`<O as Owner>::make_dependent`](Owner::make_dependent) returns an
    /// error.
    pub fn try_new_from_box_with_context(
        owner: Box<O>,
        context: O::Context<'_>,
    ) -> Result<Self, (Box<O>, O::Error)> {
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
        let panic_drop_guard = DropGuard(|| {
            // If this code is executed, it means make_dependent panicked and we
            // never `mem::forget(..)`'d this drop guard. Recover and drop the
            // boxed owner.

            // SAFETY: `owner` was just created from a Box earlier in
            // `try_new_from_box_with_context`, and not invalidated since then.
            // Because we haven't given away access to a `Self`, and the one
            // borrow we took of the owner to pass to `make_dependent` has
            // expired (since it panicked), we know there are no outstanding
            // borrows to owner. Therefore, reconstructing the original Box<O>
            // is okay.
            let owner: Box<O> = unsafe { Box::from_raw(owner.as_ptr()) };

            // If the owner's drop *also* panics, that will be a double-panic.
            // This will cause an abort, which is fine - drops generally
            // shouldn't panic, and if the user *really* wants to handle this,
            // they can check if the thread is panicking within owner's drop
            // before performing any operations which could panic.
            drop(owner);
        });

        let maybe_dependent = {
            // SAFETY: `owner` was just converted from a valid Box, and inherits
            // the alignment and validity guarantees of Box. Additionally, the
            // value behind the pointer is currently not borrowed at all - this
            // marks the beginning of a shared borrow which will last until the
            // returned `Pair` is dropped (or ends immediately if make_dependent
            // panics or returns an error).
            unsafe { owner.as_ref() }.make_dependent(context)
        };

        // The call to `make_dependent` didn't panic - disarm our drop guard
        core::mem::forget(panic_drop_guard);

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

        // We're about to call `Box::new(..)` - if it panics, we want to be able
        // to drop the boxed owner before unwinding the rest of the stack to
        // avoid unnecessarily leaking memory (and potentially other resources).
        let panic_drop_guard = DropGuard(|| {
            // If this code is executed, it means `Box::new(..)` panicked and we
            // never `mem::forget(..)`'d this drop guard. Recover and drop the
            // boxed owner.

            // SAFETY: `owner` was just created from a Box earlier in
            // `try_new_from_box_with_context`, and not invalidated since then.
            // Because we haven't given away access to a `Self`, and the one
            // borrow of the owner stored in the dependent has expired (since we
            // gave ownership of the dependent to the `Box::new(..)` call that
            // panicked), we know there are no outstanding borrows to owner.
            // Therefore, reconstructing the original Box<O> is okay.
            let owner: Box<O> = unsafe { Box::from_raw(owner.as_ptr()) };

            // If the owner's drop *also* panics, that will be a double-panic.
            // This will cause an abort, which is fine - drops generally
            // shouldn't panic, and if the user *really* wants to handle this,
            // they can check if the thread is panicking within owner's drop
            // before performing any operations which could panic.
            drop(owner);
        });

        // Move `dependent` to the heap, so we can store it as a type-erased
        // pointer.
        let dependent = Box::new(dependent);

        // The call to `Box::new(..)` didn't panic - disarm our drop guard
        core::mem::forget(panic_drop_guard);

        // Type-erase dependent so its inexpressible self-referential lifetime
        // goes away (we know that it's borrowing self.owner immutably from
        // construction (now) until drop)
        let dependent: NonNull<<O as HasDependent<'_>>::Dependent> = non_null_from_box(dependent);
        let dependent: NonNull<()> = dependent.cast();

        Ok(Self {
            owner,
            dependent,
            prevent_covariance: PhantomData,
        })
    }

    /// Returns a reference to the owner.
    pub fn owner(&self) -> &O {
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
    /// inexpressible. For dependent types covariant over their lifetime, the
    /// closure may simply return the reference to the dependent, which may then
    /// be used as if this function directly returned a reference.
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
    /// the construction of `self` until drop) is inexpressible.
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
        // currently not borrowed at all, and can't be until our exclusive
        // borrow of `self` expires.
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
        let panic_drop_guard = DropGuard(|| {
            // If this code is executed, it means the dependent's drop panicked
            // and we never `mem::forget(..)`'d this drop guard. Recover and
            // drop the boxed owner.

            // SAFETY: `this.owner` was originally created from a Box, and never
            // invalidated since then. Because we took ownership of `self`, and
            // we just dropped the dependent (well, the drop panicked - but its
            // borrow of the owner has certainly expired), we know there are no
            // outstanding borrows to owner. Therefore, reconstructing the
            // original Box<O> is okay.
            let owner: Box<O> = unsafe { Box::from_raw(this.owner.as_ptr()) };

            // If the owner's drop *also* panics, that will be a double-panic.
            // This will cause an abort, which is fine - drops generally
            // shouldn't panic, and if the user *really* wants to handle this,
            // they can check if the thread is panicking within owner's drop
            // before performing any operations which could panic.
            drop(owner);
        });

        // Drop the dependent
        drop(dependent);

        // The dependent's drop didn't panic - disarm our drop guard
        core::mem::forget(panic_drop_guard);

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

impl<O: for<'any> Owner<Context<'any> = (), Error = Infallible> + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    pub fn new(owner: O) -> Self
    where
        O: Sized,
    {
        Self::new_with_context(owner, ())
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    pub fn new_from_box(owner: Box<O>) -> Self {
        Self::new_from_box_with_context(owner, ())
    }
}

impl<O: for<'any> Owner<Context<'any> = ()> + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    ///
    /// # Errors
    /// If [`<O as Owner>::make_dependent`](Owner::make_dependent) returns an
    /// error.
    pub fn try_new(owner: O) -> Result<Self, (O, O::Error)>
    where
        O: Sized,
    {
        Self::try_new_with_context(owner, ())
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    ///
    /// # Errors
    /// If [`<O as Owner>::make_dependent`](Owner::make_dependent) returns an
    /// error.
    pub fn try_new_from_box(owner: Box<O>) -> Result<Self, (Box<O>, O::Error)> {
        Self::try_new_from_box_with_context(owner, ())
    }
}

impl<O: Owner<Error = Infallible> + ?Sized> Pair<O> {
    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
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
    /// See the "Constructors" section in the documentation of [`Pair`] for
    /// information on the differences between constructors.
    pub fn new_from_box_with_context(owner: Box<O>, context: O::Context<'_>) -> Self {
        let Ok(pair) = Self::try_new_from_box_with_context(owner, context);
        pair
    }
}

/// The [`Drop`] implementation for [`Pair`] will drop both the dependent and
/// the owner, in that order.
//
// NOTE(ichen): There are definitely some weird dropck things going on, but I do
// not believe they can lead to any unsoundness. Because of the signature of
// Pair, dropck thinks we access an O and do nothing with O::Dependent. It's
// right about O - we don't access it directly, but the dependent (which we do
// drop) might access an O in its drop. Unfortunately, the compiler is wrong
// about O::Dependent. It doesn't see any indication of O::Dependent in the
// signature for Pair (because we've type erased it), so dropck has no idea that
// we will drop an O::Dependent in our drop.
//
// This sounds like a problem, but I believe it is not. The signature of Owner
// and HasDependent enforce that the dependent only borrows the owner, or things
// which the owner also borrows. Additionally, the compiler will ensure that
// anything the owner borrows are valid until the pair's drop. Therefore, the
// dependent cannot contain any references which will be invalidated before the
// drop of the Pair<O>. As far as I know, this is the only concern surrounding
// dropck not understanding the semantics of Pair, and cannot cause unsoundness
// for the reasons described above.
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
        let panic_drop_guard = DropGuard(|| {
            // If this code is executed, it means the dependent's drop panicked
            // and we never `mem::forget(..)`'d this drop guard. Recover and
            // drop the boxed owner.

            // SAFETY: `self.owner` was originally created from a Box, and never
            // invalidated since then. Because we are in drop, and we just
            // dropped the dependent (well, the drop panicked - but its borrow
            // of the owner has certainly expired), we know there are no
            // outstanding borrows to owner. Therefore, reconstructing the
            // original Box<O> is okay.
            let owner: Box<O> = unsafe { Box::from_raw(self.owner.as_ptr()) };

            // If the owner's drop *also* panics, that will be a double-panic.
            // This will cause an abort, which is fine - drops generally
            // shouldn't panic, and if the user *really* wants to handle this,
            // they can check if the thread is panicking within owner's drop
            // before performing any operations which could panic.
            drop(owner);
        });

        // Drop the dependent
        drop(dependent);

        // The dependent's drop didn't panic - disarm our drop guard
        core::mem::forget(panic_drop_guard);

        // Drop the owner `Box<O>`

        // SAFETY: `self.owner` was originally created from a Box, and never
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
unsafe impl<O: Owner + ?Sized> Sync for Pair<O>
where
    O: Sync,
    for<'any> <O as HasDependent<'any>>::Dependent: Sync,
{
}

impl<O: Owner + Debug + ?Sized> Debug for Pair<O>
where
    for<'any> <O as HasDependent<'any>>::Dependent: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.with_dependent(|dependent| {
            f.debug_struct("Pair")
                .field("owner", &self.owner())
                .field("dependent", dependent)
                .finish()
        })
    }
}

impl<O: for<'any> Owner<Context<'any> = (), Error = Infallible> + Default> Default for Pair<O> {
    fn default() -> Self {
        Self::new(O::default())
    }
}
