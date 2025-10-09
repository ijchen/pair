//! Defines the [`Owner`] and [`HasDependent`] traits, the common interface for
//! types stored in a [`Pair`](crate::Pair).

/// Defines the dependent type for the [`Owner`] trait.
///
/// Semantically, you can think of this like a lifetime Generic Associated Type
/// (GAT) in the `Owner` trait - the two behave very similarly, and serve the
/// same role in defining a [`Dependent`](HasDependent::Dependent) type, generic
/// over some lifetime.
///
/// A real GAT is not used due to limitations in the Rust compiler. For the
/// technical details on this, I recommend Sabrina Jewson's blog post on
/// [The Better Alternative to Lifetime GATs].
///
/// [The Better Alternative to Lifetime GATs]: https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats
pub trait Owner<'any, ForImpliedBound: Sealed = Bounds<&'any Self>> {
    /// The dependent type, borrowing from an owner. This type is what is
    /// returned from [`Owner::make_dependent`].
    type Dependent;
}

/// A type alias for the [`Dependent`](HasDependent::Dependent) of some
/// [`Owner`] with a specific lifetime `'owner`.
pub type Dependent<'a, O> = <O as Owner<'a>>::Dependent;

/// Used to prevent implementors of [`HasDependent`] from overriding the
/// `ForImpliedBounds` generic type from its default.
mod sealed {
    #![expect(unnameable_types, reason = "...kinda the point")]
    /// The `ForImpliedBounds` generic type for
    /// [`HasDependent`](super::HasDependent) should not be overridden from its
    /// default.
    pub trait Sealed {}
    #[derive(Debug)]
    /// The `ForImpliedBounds` generic type for
    /// [`HasDependent`](super::HasDependent) should not be overridden from its
    /// default.
    pub struct Bounds<T>(core::marker::PhantomData<T>);
    impl<T> Sealed for Bounds<T> {}
}
use sealed::{Bounds, Sealed};
