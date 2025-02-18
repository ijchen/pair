use std::ops::Deref;

use crate::Pair;

use super::RefOwner;

pub struct DerefPair<O: Deref>(Pair<RefOwner<O, O::Target>>);

impl<O: Deref> DerefPair<O> {
    pub fn new(owner: O) -> Self {
        Self(Pair::new(RefOwner::new(owner, |owner| owner)))
    }

    pub fn get_owner(&self) -> &O {
        self.0.get_owner().owner()
    }
    pub fn with_dependent<'a, F: for<'b> FnOnce(&'b O::Target) -> T, T>(&'a self, f: F) -> T {
        self.0.with_dependent(|dependent| f(dependent))
    }
    pub fn get_dependent(&self) -> &O::Target {
        self.0.with_dependent(|dependent| dependent)
    }
    pub fn into_owner(self) -> O {
        self.0.into_owner().into_owner()
    }
}
