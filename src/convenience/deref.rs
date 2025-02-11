use std::ops::Deref;

use crate::{Owner, Pair};

struct DerefOwner<D: Deref>(D);

impl<D: Deref> Owner for DerefOwner<D> {
    type Dependent<'a>
        = &'a D::Target
    where
        Self: 'a;

    fn make_dependent(&self) -> Self::Dependent<'_> {
        &self.0
    }
}

pub struct DerefPair<D: Deref>(Pair<DerefOwner<D>>);

impl<D: Deref> DerefPair<D> {
    pub fn new(owner: D) -> Self {
        Self(Pair::new(DerefOwner(owner)))
    }

    pub fn get_owner(&self) -> &D {
        &self.0.get_owner().0
    }
    pub fn get_dependent(&self) -> &D::Target {
        self.0.get_dependent()
    }
    pub fn into_owner(self) -> D {
        self.0.into_owner().0
    }
}
