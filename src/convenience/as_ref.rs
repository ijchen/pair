use std::convert::AsRef;

use crate::{ref_owner::RefOwner, Pair};

pub struct AsRefPair<O: AsRef<D>, D: ?Sized>(Pair<RefOwner<O, D>>);

impl<O: AsRef<D>, D: ?Sized> AsRefPair<O, D> {
    pub fn new(owner: O) -> Self {
        Self(Pair::new(RefOwner::new(owner, |owner| owner.as_ref())))
    }

    pub fn get_owner(&self) -> &O {
        self.0.get_owner().owner()
    }
    pub fn with_dependent<'a, F: for<'b> FnOnce(&'b D) -> T, T>(&'a self, f: F) -> T {
        self.0.with_dependent(|dependent| f(dependent))
    }
    pub fn get_dependent(&self) -> &D {
        self.0.with_dependent(|dependent| dependent)
    }
    pub fn into_owner(self) -> O {
        self.0.into_owner().into_owner()
    }
}
