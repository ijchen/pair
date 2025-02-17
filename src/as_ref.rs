use std::{convert::AsRef, marker::PhantomData};

use crate::{HasDependent, Owner, Pair};

struct AsRefOwner<O, D>(O, PhantomData<D>);

impl<'any, O: AsRef<D>, D> HasDependent<'any> for AsRefOwner<O, D> {
    type Dependent = &'any D;
}

impl<O: AsRef<D>, D> Owner for AsRefOwner<O, D> {
    fn make_dependent(&self) -> <Self as crate::HasDependent<'_>>::Dependent {
        self.0.as_ref()
    }
}

pub struct AsRefPair<O: AsRef<D>, D>(Pair<AsRefOwner<O, D>>);

impl<O: AsRef<D>, D> AsRefPair<O, D> {
    pub fn new(owner: O) -> Self {
        Self(Pair::new(AsRefOwner(owner, PhantomData)))
    }

    pub fn get_owner(&self) -> &O {
        &self.0.get_owner().0
    }
    pub fn with_dependent<'a, F: for<'b> FnOnce(&'b D) -> U, U>(&'a self, f: F) -> U {
        self.0.with_dependent(|dependent| f(dependent))
    }
    pub fn get_dependent(&self) -> &D {
        self.0.with_dependent(|dependent| dependent)
    }
    pub fn into_owner(self) -> O {
        self.0.into_owner().0
    }
}
