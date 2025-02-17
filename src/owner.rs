// See: https://sabrinajewson.org/blog/the-better-alternative-to-lifetime-gats
pub trait HasDependent<'a, ForImpliedBound: Sealed = Bounds<&'a Self>> {
    type Dependent;
}
mod sealed {
    pub trait Sealed {}
    pub struct Bounds<T>(std::marker::PhantomData<T>);
    impl<T> Sealed for Bounds<T> {}
}
use sealed::{Bounds, Sealed};

/// A type which can act as the "owner" of some data, and can produce some
/// dependent type which borrows from `Self`.
///
/// This trait defines the "owner"/"dependent" relationship for use by the
/// [`Pair`](crate::pair) struct, as well as the function used to create the
/// dependent from a reference to the owner.
pub trait Owner: for<'any> HasDependent<'any> {
    /// Constructs the [`Dependent`](Owner::Dependent) from a reference to the
    /// owner.
    fn make_dependent(&self) -> <Self as HasDependent<'_>>::Dependent;
}

// impl<'any> Owner<'any> for String {
//     type Dependent = &'any str;

//     fn make_dependent(&'any self) -> Self::Dependent {
//         self
//     }
// }
// impl<'any, T> Owner<'any> for Vec<T> {
//     type Dependent = &'any [T];

//     fn make_dependent(&'any self) -> Self::Dependent {
//         self
//     }
// }
// impl<'any, T: std::ops::Deref> Owner<'any> for T {
//     type Dependent = &'any T::Target;

//     fn make_dependent(&'any self) -> Self::Dependent {
//         self
//     }
// }
