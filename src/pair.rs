// # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # #
// #                                                             #
// #                       !!! WARNING !!!                       #
// #                                                             #
// #   SAFETY COMMENTS WERE WRITTEN BEFORE SUBSTANTIAL CHANGES   #
// #    TO THE `Owner` TRAIT WERE MADE, AND MAY BE INCORRECT!    #
// #                                                             #
// # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # # #
// p.s. I hope u like my cool ASCII box <3

// TODO: all comments are potentially out of date due to Owner trait updates.

use std::{marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::Owner;

// SAFETY: `Pair` has no special thread-related invariants or requirements, so
// sending a `Pair` to another thread could only cause problems if sending
// either the owner or the dependent to another thread could cause problems
// (since both are semantically moved with and made accessible through the
// `Pair`).
unsafe impl<O: for<'any> Owner<'any> + ?Sized> Send for Pair<O>
where
    O: Send,
    for<'any> <O as Owner<'any>>::Dependent: Send,
{
}

// SAFETY: `Pair` has no special thread-related invariants or requirements, so
// sharing a reference to a `Pair` across multiple threads could only cause
// problems if sharing a reference to either the owner or the dependent across
// multiple threads could cause problems (since references to both are made
// accessible through references to the `Pair`).
unsafe impl<O: for<'any> Owner<'any>> Sync for Pair<O>
where
    O: Sync,
    for<'any> <O as Owner<'any>>::Dependent: Sync,
{
}

/// A self-referential pair containing both some [`Owner`] and its
/// [`Dependent`](Owner::Dependent).
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
pub struct Pair<O: for<'any> Owner<'any> + ?Sized> {
    // Derived from a Box<O>
    // Immutably borrowed by `self.dependent` from construction until drop
    owner: NonNull<O>,

    // Type-erased Box<O::Dependent<'self.owner>>
    dependent: NonNull<()>,

    // Need invariance over O - if we were covariant or contravariant, two
    // different `O`s with two different `Owner` impls (and importantly, two
    // different Dependent associated types) which have a subtype/supertype
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
/// allocator and a valid [`Layout`](std::alloc::Layout) for `T`.
fn non_null_from_box<T: ?Sized>(value: Box<T>) -> NonNull<T> {
    // See: https://github.com/rust-lang/rust/issues/47336#issuecomment-586578713
    NonNull::from(Box::leak(value))
}

impl<O: for<'any> Owner<'any> + ?Sized> Pair<O> {
    // TODO: expose a `try_new` and `try_new_from_box` API (will need updates to
    // the Owner trait)

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you already have a [`Box`]ed owner, consider [`Pair::new_from_box`]
    /// to avoid redundant reallocation.
    pub fn new(owner: O) -> Self
    where
        O: Sized,
    {
        Self::new_from_box(Box::new(owner))
    }

    /// Constructs a new [`Pair`] with the given [`Owner`]. The dependent will
    /// be computed through [`Owner::make_dependent`] during this construction.
    ///
    /// If you have an unboxed `O` and only box it for this function, consider
    /// the convenience constructor [`Pair::new`], which boxes the owner for
    /// you to reduce clutter in your code.
    pub fn new_from_box(owner: Box<O>) -> Self {
        // Convert owner into a NonNull, so we are no longer restricted by the
        // aliasing requirements of Box
        let owner = non_null_from_box(owner);

        // Borrow `owner` to construct `dependent`. This borrow conceptually
        // lasts from now until drop, where we will drop `dependent` and then
        // drop owner.

        // SAFETY: `owner` was just converted from a valid Box, and inherits the
        // alignment and validity guarantees of Box. Additionally, the value
        // behind the pointer is currently not borrowed at all - this marks the
        // beginning of a shared borrow which will last until the returned
        // `Pair` is dropped.
        let dependent = unsafe { owner.as_ref() }.make_dependent();

        // Type-erase dependent so it's inexpressible self-referential lifetime
        // goes away (we know that it's borrowing self.owner immutably from
        // construction (now) until drop)
        let dependent: NonNull<O::Dependent> = non_null_from_box(Box::new(dependent));
        let dependent: NonNull<()> = dependent.cast();

        Self {
            owner,
            dependent,
            prevent_covariance: PhantomData,
        }
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

    /// TODO
    // TODO: The compiler will allow us to elide this generic  vvvv lifetime
    // what does it default to? This feels suspicious to me.   vvvv
    pub fn with_dependent<'a, F: for<'b> FnOnce(&'a <O as Owner<'b>>::Dependent) -> T, T>(
        &'a self,
        f: F,
    ) -> T {
        // SAFETY: TODO
        let dependent = unsafe { self.dependent.cast::<O::Dependent>().as_ref() };

        f(dependent)
    }

    /// TODO
    pub fn with_dependent_mut<
        'a,
        F: for<'b> FnOnce(&'a mut <O as Owner<'b>>::Dependent) -> T,
        T,
    >(
        &'a mut self,
        f: F,
    ) -> T {
        // SAFETY: TODO
        let dependent = unsafe { self.dependent.cast::<O::Dependent>().as_mut() };

        f(dependent)
    }

    /// Consumes the [`Pair`], dropping the dependent and returning the owner.
    ///
    /// If you don't need the returned owner in a [`Box`], consider the
    /// convenience method [`Pair::into_owner`], which moves the owner out of
    /// the box for you to reduce clutter in your code.
    pub fn into_boxed_owner(self) -> Box<O> {
        // Prevent dropping `self` at the end of this scope - otherwise, the
        // Pair drop implementation would attempt to drop the owner and
        // dependent, which would be... not good (unsound).
        //
        // It's important that we do this before calling the dependent's drop,
        // since a panic in that drop would otherwise cause at least a double
        // panic, and potentially even unsoundness (although that part I'm less
        // sure of)
        let this = ManuallyDrop::new(self);

        // SAFETY: `this.dependent` was originally created from a Box, and never
        // invalidated since then. Because we took ownership of `self`, we know
        // there are no outstanding borrows to the dependent. Therefore,
        // reconstructing the original Box<O::Dependent<'_>> is okay.
        drop(unsafe { Box::from_raw(this.dependent.cast::<O::Dependent>().as_ptr()) });

        // SAFETY: `this.owner` was originally created from a Box, and never
        // invalidated since then. Because we took ownership of `self`, and we
        // just dropped the dependent, we know there are no outstanding borrows
        // to owner. Therefore, reconstructing the original Box<O> is okay.
        let boxed_owner = unsafe { Box::from_raw(this.owner.as_ptr()) };

        boxed_owner
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

/// The [`Drop`] implementation for [`Pair`] will drop both the dependent and
/// the owner, in that order.
impl<O: for<'any> Owner<'any> + ?Sized> Drop for Pair<O> {
    fn drop(&mut self) {
        // Call `Drop::drop` on the dependent `O::Dependent<'_>`

        // SAFETY: `self.dependent` was originally created from a Box, and never
        // invalidated since then. Because we are in drop, we know there are no
        // outstanding borrows to the dependent. Therefore, reconstructing the
        // original Box<O::Dependent<'_>> is okay.
        drop(unsafe { Box::from_raw(self.dependent.cast::<O::Dependent>().as_ptr()) });

        // Call `Drop::drop` on the owner `Box<O>`

        // SAFETY: `this.owner` was originally created from a Box, and never
        // invalidated since then. Because we are in drop, and we just dropped
        // the dependent, we know there are no outstanding borrows to owner.
        // Therefore, reconstructing the original Box<O> is okay.
        drop(unsafe { Box::from_raw(self.owner.as_ptr()) });
    }
}
