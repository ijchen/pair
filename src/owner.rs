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
pub trait HasDependent<'owner, ForImpliedBound: Sealed = Bounds<&'owner Self>> {
    /// The dependent type, borrowing from an owner. This type is what is
    /// returned from [`Owner::make_dependent`].
    type Dependent;
}

/// A type alias for the [`Dependent`](HasDependent::Dependent) of some
/// [`Owner`] with a specific lifetime `'owner`.
pub type Dependent<'owner, O> = <O as HasDependent<'owner>>::Dependent;

#[expect(
    clippy::missing_errors_doc,
    reason = "failure modes are specific to the trait's implementation"
)]
/// A type which can act as the "owner" of some data, and can produce some
/// dependent type which borrows from `Self`. Used for the [`Pair`](crate::Pair)
/// struct.
///
/// The supertrait [`HasDependent`] defines the dependent type, acting as a sort
/// of generic associated type - see its documentation for more information. The
/// [`make_dependent`](Owner::make_dependent) function defines how to create a
/// dependent from a reference to an owner.
pub trait Owner: for<'any> HasDependent<'any> {
    /// Additional context provided to [`make_dependent`](Owner::make_dependent)
    /// as an argument.
    ///
    /// If additional context is not necessary, this should be set to
    /// [`()`](prim@unit).
    //
    // TODO(ichen): default this to () when associated type defaults are
    // stabilized (https://github.com/rust-lang/rust/issues/29661)
    type Context<'a>;

    /// The error type returned by [`make_dependent`](Owner::make_dependent) in
    /// the event of an error.
    ///
    /// If `make_dependent` can't fail, this should be set to
    /// [`Infallible`](core::convert::Infallible).
    //
    // TODO(ichen): default this to core::convert::Infallible (or preferably !)
    // when associated type defaults are stabilized
    // (https://github.com/rust-lang/rust/issues/29661)
    type Error;

    /// Attempts to construct a [`Dependent`](HasDependent::Dependent) from a
    /// reference to an owner and some context.
    fn make_dependent<'owner>(
        &'owner self,
        context: Self::Context<'_>,
    ) -> Result<Dependent<'owner, Self>, Self::Error>;
}

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
