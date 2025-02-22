/// Used to prevent implementors of [`HasDependent`] from overriding the
/// `ForImpliedBounds` generic type from its default.
mod sealed {
    /// The `ForImpliedBounds` generic type for
    /// [`HasDependent`](super::HasDependent) should not be overridden from its
    /// default.
    pub trait Sealed {}
    /// The `ForImpliedBounds` generic type for
    /// [`HasDependent`](super::HasDependent) should not be overridden from its
    /// default.
    pub struct Bounds<T>(std::marker::PhantomData<T>);
    impl<T> Sealed for Bounds<T> {}
}
use sealed::{Bounds, Sealed};

/// Defines the dependent type for the [`Owner`] trait.
///
/// Semantically, you can think of this like a lifetime Generic Associated Type
/// (GAT) in the `Owner` trait - the two behave very similarly, and serve the
/// same role in defining a [`Dependent`](HasDependent::Dependent) type, generic
/// over some lifetime.
///
/// A real GAT is not used due to limitations in the current Rust compiler. For
/// the technical details on this, I recommend Sabrina Jewson's blog post on
/// [The Better Alternative to Lifetime GATs].
///
/// [The Better Alternative to Lifetime GATs]: https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats
pub trait HasDependent<'owner, ForImpliedBound: Sealed = Bounds<&'owner Self>> {
    /// The dependent type, borrowing from an owner. This type is what is
    /// returned from [`Owner::make_dependent`].
    type Dependent;
}

/// A type which can act as the "owner" of some data, and can produce some
/// dependent type which borrows from `Self`. Used for the [`Pair`](crate::Pair)
/// struct.
///
/// The supertrait [`HasDependent<'_>`] defines the dependent type, acting as a
/// sort of generic associated type - see its documentation for more
/// information. The [`make_dependent`](Owner::make_dependent) function defines
/// how to create a dependent from a reference to an owner.
pub trait Owner: for<'any> HasDependent<'any> {
    /// Additional context provided to [`make_dependent`](Owner::make_dependent)
    /// as an argument.
    ///
    /// If additional context is not necessary, this should be set to `()`.
    //
    // TODO(ichen): default this to () when associated type defaults are
    // stabilized (https://github.com/rust-lang/rust/issues/29661)
    type Context;

    /// The error type returned by [`make_dependent`](Owner::make_dependent) in
    /// the event of an error.
    ///
    /// If `make_dependent` can't fail, this should be set to
    /// [`Infallible`](std::convert::Infallible).
    //
    // TODO(ichen): default this to std::convert::Infallible (or preferably !)
    // when associated type defaults are stabilized
    // (https://github.com/rust-lang/rust/issues/29661)
    type Err;

    /// Attempts to construct a [`Dependent`](HasDependent::Dependent) from a
    /// reference to an owner and some context.
    fn make_dependent(
        &self,
        context: Self::Context,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err>;
}
