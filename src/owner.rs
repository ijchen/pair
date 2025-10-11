//! Defines the [`Owner`] trait, which specifies the owner/dependent relationship for data used in a
//! [`Pair`](crate::Pair).

/// Enables using `Self` as the owner in a [`Pair`](crate::Pair) by specifying the dependent type.
///
/// # Implementing
///
/// In order to use `Self` in a `Pair`, [`Owner`] must be implemented for all possible lifetimes:
///
/// ```
/// # use pair::Owner;
/// # struct MyBuffer;
/// impl<'any> Owner<'any> for MyBuffer {
///     type Dependent = &'any [u8];
/// }
/// ```
///
/// Implementing `Owner` on only a subset of lifetimes is not useful, as `Pair` requires that
/// `Owner` is implemented for all possible lifetimes.
///
/// # Why not a Generic Associated Type?
///
/// Semantically, you can think of [`Dependent`](Owner::Dependent) like a lifetime Generic
/// Associated Type (GAT):
///
/// ```
/// # use pair::Owner;
/// // This (how the real Owner trait works):
///
/// # mod example_1 {
/// # struct Foo;
/// # struct Bar<'a>(&'a ());
/// trait Owner<'any> {
///     type Dependent;
/// }
///
/// impl<'any> Owner<'any> for Foo {
///     type Dependent = Bar<'any>;
/// }
/// # }
///
/// // Is spiritually equivalent to a lifetime GAT:
///
/// # mod example_2 {
/// # struct Foo;
/// # struct Bar<'a>(&'a ());
/// trait Owner {
///     type Dependent<'any>;
/// }
///
/// impl Owner for Foo {
///     type Dependent<'any> = Bar<'any>;
/// }
/// # }
/// ```
///
/// The two behave very similarly, and serve the same role in defining a dependent type generic over
/// some lifetime. A real GAT is not used due to limitations in the Rust compiler. For the technical
/// details on this, I recommend Sabrina Jewson's blog post on [The Better Alternative to Lifetime
/// GATs].
///
/// [The Better Alternative to Lifetime GATs]: https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats
pub trait Owner<'any, ForImpliedBound: Sealed = Bounds<&'any Self>> {
    /// The dependent type, borrowing from the owner.
    type Dependent;
}

/// A type alias for the [`Dependent`](Owner::Dependent) of an [`Owner`] with the lifetime `'a`.
pub type Dependent<'a, O> = <O as Owner<'a>>::Dependent;

/// Used to prevent implementors of [`Owner`] from overriding the `ForImpliedBounds` generic type
/// from its default.
mod sealed {
    #![expect(unnameable_types, reason = "...kinda the point")]
    /// The `ForImpliedBounds` generic type for [`Owner`](super::Owner) should not be overridden
    /// from its default.
    pub trait Sealed {}

    /// The `ForImpliedBounds` generic type for [`Owner`](super::Owner) should not be overridden
    /// from its default.
    #[derive(Debug)]
    pub struct Bounds<T>(core::marker::PhantomData<T>);

    impl<T> Sealed for Bounds<T> {}
}
use sealed::{Bounds, Sealed};
