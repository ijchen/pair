#![allow(missing_docs, reason = "integration test")]

use std::{cell::RefCell, convert::Infallible, rc::Rc};

use pair::{HasDependent, Owner, Pair};

#[derive(Debug)]
struct OnDrop<T> {
    value: Rc<RefCell<T>>,
    f: fn(&mut T),
}
impl<T> Drop for OnDrop<T> {
    fn drop(&mut self) {
        (self.f)(&mut self.value.borrow_mut());
    }
}

struct OnDropDep<T> {
    value: Rc<RefCell<T>>,
    f: fn(&mut T),
}
impl<T> Drop for OnDropDep<T> {
    fn drop(&mut self) {
        (self.f)(&mut self.value.borrow_mut());
    }
}
impl<T> HasDependent<'_> for OnDrop<T> {
    type Dependent = OnDropDep<T>;
}

impl<T> Owner for OnDrop<T> {
    type Context<'a> = (Rc<RefCell<T>>, fn(&mut T));
    type Error = Infallible;

    fn make_dependent(
        &self,
        context: Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(OnDropDep {
            value: context.0,
            f: context.1,
        })
    }
}

#[test]
fn both_drops_called() {
    // Using a common Vec instead of two bools, since we care about drop order
    let drops_called = Rc::new(RefCell::new(Vec::new()));
    let pair = Pair::new_with_context(
        OnDrop {
            value: Rc::clone(&drops_called),
            f: |v| v.push("owner"),
        },
        (Rc::clone(&drops_called), |v| v.push("dep")),
    );

    assert_eq!(*drops_called.borrow(), [] as [&str; 0]);

    drop(pair);

    assert_eq!(*drops_called.borrow(), ["dep", "owner"]);
}

#[test]
fn dep_drop_called_on_into_owner() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let dep_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new_with_context(
        OnDrop {
            value: Rc::clone(&owner_drop_called),
            f: |d| *d = true,
        },
        (Rc::clone(&dep_drop_called), |d| *d = true),
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
