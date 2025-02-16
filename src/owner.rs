/// A type which can act as the "owner" of some data, and can produce some
/// dependent type which borrows from `Self`.
///
/// This trait defines the "owner"/"dependent" relationship for use by the
/// [`Pair`](crate::pair) struct, as well as the function used to create the
/// dependent from a reference to the owner.
pub trait Owner<'owner> {
    /// The dependent type, which borrows from the owner.
    type Dependent;

    /// Constructs the [`Dependent`](Owner::Dependent) from a reference to the
    /// owner.
    fn make_dependent(&'owner self) -> Self::Dependent;
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
