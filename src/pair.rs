use std::{marker::PhantomData, ptr::NonNull};

use crate::Owner;

// SAFETY: `Pair` has no special thread-related invariants or requirements, so
// sending a `Pair` to another thread could only cause problems if sending
// either the owner or the dependent to another thread could cause problems
// (since both are semantically moved with and made accessible through the
// `Pair`).
unsafe impl<O: Owner + ?Sized> Send for Pair<O>
where
    O: Send,
    for<'a> O::Dependent<'a>: Send,
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
    for<'a> O::Dependent<'a>: Sync,
{
}

/// A self-referential pair containing both some [`Owner`] and it's
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
pub struct Pair<O: Owner + ?Sized> {
    // Derived from a Box<O>
    // Immutably borrowed by `self.dependent` from construction until drop
    owner: NonNull<O>,

    // Type-erased Box<O::Dependent<'self.owner>>
    dependent: NonNull<()>,

    // Need invariance over O.
    // TODO: explain why (coherence check allows different impls for fn ptrs)
    prevent_covariance: PhantomData<*mut O>,
}

/// Creates a [`NonNull<T>`] from [`Box<T>`]. The returned NonNull TODO:
/// describe it's properties you can assume (like dereferencability and valid
/// ways to recover the Box)
fn non_null_from_box<T: ?Sized>(value: Box<T>) -> NonNull<T> {
    // See: https://github.com/rust-lang/rust/issues/47336#issuecomment-586578713
    NonNull::from(Box::leak(value))
}

impl<O: Owner + ?Sized> Pair<O> {
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
        // invariants of Box (like noalias)
        let owner = non_null_from_box(owner);

        // Borrow `owner` to construct `dependent`. This borrow conceptually
        // lasts from now until drop, where we will drop `dependent` and then
        // drop owner.
        // SAFETY: TODO
        let dependent = unsafe { owner.as_ref() }.make_dependent();

        // Type-erase dependent so it's inexpressible self-referential lifetime
        // goes away (we know that it's borrowing self.owner immutably from
        // construction (now) until drop)
        let dependent: NonNull<O::Dependent<'_>> = non_null_from_box(Box::new(dependent));
        let dependent: NonNull<()> = dependent.cast();

        Self {
            owner,
            dependent,
            prevent_covariance: PhantomData,
        }
    }

    /// Returns a reference to the owner.
    pub fn get_owner(&self) -> &O {
        // SAFETY: TODO
        unsafe { self.owner.as_ref() }
    }

    /// Returns a reference to the dependent.
    pub fn get_dependent(&self) -> &O::Dependent<'_> {
        // SAFETY: TODO
        unsafe { self.dependent.cast().as_ref() }
    }

    /// Consumes the [`Pair`], dropping the dependent and returning the owner.
    ///
    /// If you don't need the returned owner in a [`Box`], consider the
    /// convenience method [`Pair::into_owner`], which moves the owner out of
    /// the box for you to reduce clutter in your code.
    pub fn into_boxed_owner(self) -> Box<O> {
        // SAFETY: TODO
        drop(unsafe { Box::from_raw(self.dependent.cast::<O::Dependent<'_>>().as_ptr()) });

        // SAFETY: TODO
        let boxed_owner = unsafe { Box::from_raw(self.owner.as_ptr()) };

        std::mem::forget(self);

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
impl<O: Owner + ?Sized> Drop for Pair<O> {
    fn drop(&mut self) {
        // Call `Drop::drop` on the dependent `O::Dependent<'_>`
        // SAFETY: TODO
        drop(unsafe { Box::from_raw(self.dependent.cast::<O::Dependent<'_>>().as_ptr()) });

        // Call `Drop::drop` on the owner `Box<O>`
        // SAFETY: TODO
        drop(unsafe { Box::from_raw(self.owner.as_ptr()) });
    }
}
