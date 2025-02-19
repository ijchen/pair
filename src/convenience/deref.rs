use std::ops::Deref;

use crate::Pair;

use super::RefOwner;

pub struct DerefPair<T: Deref>(Pair<RefOwner<T, T::Target>>);

impl<T: Deref> DerefPair<T> {
    pub fn new(owner: T) -> Self {
        Self(Pair::new(RefOwner::new(owner, |owner| owner)))
    }

    pub fn get_owner(&self) -> &T {
        self.0.get_owner().owner()
    }
    pub fn get_dependent(&self) -> &T::Target {
        self.0.with_dependent(|dependent| dependent)
    }
    pub fn into_owner(self) -> T {
        self.0.into_owner().into_owner()
    }
}
