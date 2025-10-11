#![allow(missing_docs, reason = "integration test")]

use std::{cell::RefCell, rc::Rc};

use pair::{Owner, Pair};

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

// TODO: this is allowed??
impl<T> Owner<'_> for OnDrop<T> {
    type Dependent = OnDropDep<T>;
}

fn make_dependent<T>(context: (Rc<RefCell<T>>, fn(&mut T))) -> OnDropDep<T> {
    OnDropDep {
        value: context.0,
        f: context.1,
    }
}

#[test]
fn both_drops_called() {
    // Using a common Vec instead of two bools, since we care about drop order
    let drops_called = Rc::new(RefCell::new(Vec::new()));
    let pair = Pair::new(
        OnDrop {
            value: Rc::clone(&drops_called),
            f: |v| v.push("owner"),
        },
        |_| make_dependent((Rc::clone(&drops_called), |v| v.push("dep"))),
    );

    assert_eq!(*drops_called.borrow(), [] as [&str; 0]);

    drop(pair);

    assert_eq!(*drops_called.borrow(), ["dep", "owner"]);
}

#[test]
fn dep_drop_called_on_into_owner() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let dep_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(
        OnDrop {
            value: Rc::clone(&owner_drop_called),
            f: |d| *d = true,
        },
        |_| make_dependent((Rc::clone(&dep_drop_called), |d| *d = true)),
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
