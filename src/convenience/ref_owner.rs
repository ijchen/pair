use crate::{HasDependent, Owner};

pub struct RefOwner<O, D: ?Sized> {
    owner: O,
    f: fn(&O) -> &D,
}

impl<O, D: ?Sized> RefOwner<O, D> {
    pub fn new(owner: O, f: fn(&O) -> &D) -> Self {
        Self { owner, f }
    }

    pub fn owner(&self) -> &O {
        &self.owner
    }

    pub fn into_owner(self) -> O {
        self.owner
    }
}

impl<'any, O, D: ?Sized> HasDependent<'any> for RefOwner<O, D> {
    type Dependent = &'any D;
}
impl<O, D: ?Sized> Owner for RefOwner<O, D> {
    fn make_dependent(&self) -> <Self as HasDependent<'_>>::Dependent {
        (self.f)(&self.owner)
    }
}
