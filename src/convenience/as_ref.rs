use std::convert::AsRef;

use crate::Pair;

use super::RefOwner;

pub struct AsRefPair<T: AsRef<U>, U: ?Sized>(Pair<RefOwner<T, U>>);

impl<T: AsRef<U>, U: ?Sized> AsRefPair<T, U> {
    pub fn new(owner: T) -> Self {
        Self(Pair::new(RefOwner::new(owner, |owner| owner.as_ref())))
    }

    pub fn get_owner(&self) -> &T {
        self.0.get_owner().owner()
    }
    pub fn get_dependent(&self) -> &U {
        self.0.with_dependent(|dependent| dependent)
    }
    pub fn into_owner(self) -> T {
        self.0.into_owner().into_owner()
    }
}
