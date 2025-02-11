mod as_ref;
mod borrow;
mod deref;

// TODO: all of these should be well-documented to match Pair's docs, and should
// expose the same (well, except swapping out the Owner trait bound) API.
// TODO: also the structure of the generics is less than ideal (currently
// thinking something like <O, D> where O: AsRef<D> or something)

pub use as_ref::AsRefPair;
pub use borrow::BorrowPair;
pub use deref::DerefPair;
