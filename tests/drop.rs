use std::{cell::RefCell, convert::Infallible, rc::Rc};

use pair::{HasDependent, Owner, Pair};

#[derive(Debug)]
struct OnDrop(Rc<RefCell<bool>>);
impl Drop for OnDrop {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
    }
}

struct OnDropDep(Rc<RefCell<bool>>);
impl Drop for OnDropDep {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
    }
}
impl<'owner> HasDependent<'owner> for OnDrop {
    type Dependent = OnDropDep;
}

impl Owner for OnDrop {
    type Context<'a> = Rc<RefCell<bool>>;
    type Err = Infallible;

    fn make_dependent(
        &self,
        context: Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err> {
        Ok(OnDropDep(context))
    }
}

#[test]
fn both_drops_called() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let dep_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new_with_context(
        OnDrop(Rc::clone(&owner_drop_called)),
        Rc::clone(&dep_drop_called),
    );

    assert!(!*owner_drop_called.borrow());
    assert!(!*dep_drop_called.borrow());

    drop(pair);

    assert!(*owner_drop_called.borrow());
    assert!(*dep_drop_called.borrow());
}

#[test]
fn dep_drop_called_on_into_owner() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let dep_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new_with_context(
        OnDrop(Rc::clone(&owner_drop_called)),
        Rc::clone(&dep_drop_called),
    );

    assert!(!*owner_drop_called.borrow());
    assert!(!*dep_drop_called.borrow());

    let owner = pair.into_owner();

    assert!(!*owner_drop_called.borrow());
    assert!(*dep_drop_called.borrow());

    drop(owner);

    assert!(*owner_drop_called.borrow());
    assert!(*dep_drop_called.borrow());
}
