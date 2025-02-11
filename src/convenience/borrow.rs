use std::{borrow::Borrow, marker::PhantomData};

use crate::{Owner, Pair};

struct BorrowOwner<D: Borrow<T>, T>(D, PhantomData<T>);

impl<D: Borrow<T>, T> Owner for BorrowOwner<D, T> {
    type Dependent<'a>
        = &'a T
    where
        Self: 'a;

    fn make_dependent(&self) -> Self::Dependent<'_> {
        self.0.borrow()
    }
}

pub struct BorrowPair<D: Borrow<T>, T>(Pair<BorrowOwner<D, T>>);

impl<D: Borrow<T>, T> BorrowPair<D, T> {
    pub fn new(owner: D) -> Self {
        Self(Pair::new(BorrowOwner(owner, PhantomData)))
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
