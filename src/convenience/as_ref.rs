use std::{convert::AsRef, marker::PhantomData};

use crate::{Owner, Pair};

struct AsRefOwner<D: AsRef<T>, T>(D, PhantomData<T>);

impl<D: AsRef<T>, T> Owner for AsRefOwner<D, T> {
    type Dependent<'a>
        = &'a T
    where
        Self: 'a;

    fn make_dependent(&self) -> Self::Dependent<'_> {
        self.0.as_ref()
    }
}

pub struct AsRefPair<D: AsRef<T>, T>(Pair<AsRefOwner<D, T>>);

impl<D: AsRef<T>, T> AsRefPair<D, T> {
    pub fn new(owner: D) -> Self {
        Self(Pair::new(AsRefOwner(owner, PhantomData)))
    }

    pub fn get_owner(&self) -> &D {
        &self.0.get_owner().0
    }
    pub fn get_dependent(&self) -> &T {
        self.0.get_dependent()
    }
    pub fn into_owner(self) -> D {
        self.0.into_owner().0
    }
}
