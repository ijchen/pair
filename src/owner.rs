/// A type which can act as the "owner" of some data, and can produce some
/// dependent type which borrows from `Self`.
///
/// This trait defines the "owner"/"dependent" relationship for use by the
/// [`Pair`](crate::pair) struct, as well as the function used to create the
/// dependent from a reference to the owner.
pub trait Owner {
    /// The dependent type, which borrows from the owner.
    type Dependent<'a>
    where
        Self: 'a;

    /// Constructs the [`Dependent`](Owner::Dependent) from a reference to the
    /// owner.
    fn make_dependent(&self) -> Self::Dependent<'_>;
}
