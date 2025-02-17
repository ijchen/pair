use std::ops::Deref;

use crate::{HasDependent, Owner, Pair};

pub struct DerefOwner<O: Deref>(pub O);

impl<'any, O: Deref> HasDependent<'any> for DerefOwner<O> {
    type Dependent = &'any O::Target;
}
impl<O: Deref> Owner for DerefOwner<O> {
    fn make_dependent(&self) -> <Self as HasDependent<'_>>::Dependent {
        &self.0
    }
}

impl<O: Deref> Pair<DerefOwner<O>> {
    pub fn new_deref(owner: O) -> Self {
        Self::new(DerefOwner(owner))
    }

    pub fn with_dependent_deref<F: FnOnce(&O::Target) -> T, T>(&self, f: F) -> T {
        self.with_dependent(|dependent| f(dependent))
    }
    pub fn get_dependent_deref(&self) -> &O::Target {
        self.with_dependent(|dependent| dependent)
    }
}
