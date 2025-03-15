#![allow(missing_docs, reason = "integration test")]

use std::{
    cell::RefCell,
    convert::Infallible,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    rc::Rc,
};

use pair::{Dependent, HasDependent, Owner, Pair};

#[derive(Debug, PartialEq, Eq)]
struct MyPayload(u8);

// make_dependent panics
struct PanicOnMakeDependent(Rc<RefCell<bool>>);
impl Drop for PanicOnMakeDependent {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
    }
}

impl HasDependent<'_> for PanicOnMakeDependent {
    type Dependent = ();
}

impl Owner for PanicOnMakeDependent {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<Dependent<'_, Self>, Self::Error> {
        panic_any(MyPayload(7));
    }
}

#[test]
fn make_dependent_panic_handled() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let owner_drop_called2 = Rc::clone(&owner_drop_called);
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| {
        Pair::new(PanicOnMakeDependent(owner_drop_called2));
    }))
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(payload, MyPayload(7));
    assert!(*owner_drop_called.borrow());
}

// dependent drop panics in into_owner
#[derive(Debug)]
struct PanicOnDepDropIntoOwner(Rc<RefCell<bool>>);
impl Drop for PanicOnDepDropIntoOwner {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
    }
}

struct PanicOnDepDropIntoOwnerDep;
impl Drop for PanicOnDepDropIntoOwnerDep {
    fn drop(&mut self) {
        panic_any(MyPayload(11));
    }
}
impl HasDependent<'_> for PanicOnDepDropIntoOwner {
    type Dependent = PanicOnDepDropIntoOwnerDep;
}

impl Owner for PanicOnDepDropIntoOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(PanicOnDepDropIntoOwnerDep)
    }
}

#[test]
fn dependent_drop_panic_handled_in_into_owner() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(PanicOnDepDropIntoOwner(Rc::clone(&owner_drop_called)));
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| pair.into_owner()))
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(payload, MyPayload(11));
    assert!(*owner_drop_called.borrow());
}

// dependent drop panics in pair drop
#[derive(Debug)]
struct PanicOnDepDropPairDrop(Rc<RefCell<bool>>);
impl Drop for PanicOnDepDropPairDrop {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
    }
}

struct PanicOnDepDropPairDropDep;
impl Drop for PanicOnDepDropPairDropDep {
    fn drop(&mut self) {
        panic_any(MyPayload(3));
    }
}
impl HasDependent<'_> for PanicOnDepDropPairDrop {
    type Dependent = PanicOnDepDropPairDropDep;
}

impl Owner for PanicOnDepDropPairDrop {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(PanicOnDepDropPairDropDep)
    }
}

#[test]
fn dependent_drop_panic_handled_in_pair_drop() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(PanicOnDepDropPairDrop(Rc::clone(&owner_drop_called)));
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| drop(pair)))
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(payload, MyPayload(3));
    assert!(*owner_drop_called.borrow());
}
