#![allow(missing_docs, reason = "integration test")]

use std::{
    cell::RefCell,
    convert::Infallible,
    panic::{AssertUnwindSafe, catch_unwind, panic_any},
    rc::Rc,
};

use pair::{HasDependent, Owner, Pair};

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

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        panic_any(MyPayload(7));
    }
}

#[test]
fn make_dependent_panic_handled_std_only() {
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

// make_dependent panics and owner drop panics
struct PanicOnMakeDependentAndOwnerDrop(Rc<RefCell<bool>>);
impl Drop for PanicOnMakeDependentAndOwnerDrop {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
        panic!("lol");
    }
}

impl HasDependent<'_> for PanicOnMakeDependentAndOwnerDrop {
    type Dependent = ();
}

impl Owner for PanicOnMakeDependentAndOwnerDrop {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        panic_any(MyPayload(42));
    }
}

#[test]
fn make_dependent_and_owner_drop_panic_handled_std_only() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let owner_drop_called2 = Rc::clone(&owner_drop_called);
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| {
        Pair::new(PanicOnMakeDependentAndOwnerDrop(owner_drop_called2));
    }))
    .unwrap_err()
    .downcast()
    .unwrap();

    assert_eq!(payload, MyPayload(42));
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

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(PanicOnDepDropIntoOwnerDep)
    }
}

#[test]
fn dependent_drop_panic_handled_in_into_owner_std_only() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(PanicOnDepDropIntoOwner(Rc::clone(&owner_drop_called)));
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| pair.into_owner()))
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(payload, MyPayload(11));
    assert!(*owner_drop_called.borrow());
}

// dependent drop panics and owner drop panics in into_owner
#[derive(Debug)]
struct PanicOnDepAndOwnerDropIntoOwner(Rc<RefCell<bool>>);
impl Drop for PanicOnDepAndOwnerDropIntoOwner {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
        panic!("ruh roh, raggy");
    }
}

struct PanicOnDepAndOwnerDropIntoOwnerDep;
impl Drop for PanicOnDepAndOwnerDropIntoOwnerDep {
    fn drop(&mut self) {
        panic_any(MyPayload(1));
    }
}
impl HasDependent<'_> for PanicOnDepAndOwnerDropIntoOwner {
    type Dependent = PanicOnDepAndOwnerDropIntoOwnerDep;
}

impl Owner for PanicOnDepAndOwnerDropIntoOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(PanicOnDepAndOwnerDropIntoOwnerDep)
    }
}

#[test]
fn dependent_and_owner_drop_panic_handled_in_into_owner_std_only() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(PanicOnDepAndOwnerDropIntoOwner(Rc::clone(
        &owner_drop_called,
    )));
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| pair.into_owner()))
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(payload, MyPayload(1));
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

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(PanicOnDepDropPairDropDep)
    }
}

#[test]
fn dependent_drop_panic_handled_in_pair_drop_std_only() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(PanicOnDepDropPairDrop(Rc::clone(&owner_drop_called)));
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| drop(pair)))
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(payload, MyPayload(3));
    assert!(*owner_drop_called.borrow());
}

// dependent drop panics and owner drop panics in into_owner
#[derive(Debug)]
struct PanicOnDepAndOwnerDropPairDrop(Rc<RefCell<bool>>);
impl Drop for PanicOnDepAndOwnerDropPairDrop {
    fn drop(&mut self) {
        *self.0.borrow_mut() = true;
        panic!("ruh roh, raggy");
    }
}

struct PanicOnDepAndOwnerDropPairDropDep;
impl Drop for PanicOnDepAndOwnerDropPairDropDep {
    fn drop(&mut self) {
        panic_any(MyPayload(5));
    }
}
impl HasDependent<'_> for PanicOnDepAndOwnerDropPairDrop {
    type Dependent = PanicOnDepAndOwnerDropPairDropDep;
}

impl Owner for PanicOnDepAndOwnerDropPairDrop {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(PanicOnDepAndOwnerDropPairDropDep)
    }
}

#[test]
fn dependent_and_owner_drop_panic_handled_in_pair_drop_std_only() {
    let owner_drop_called = Rc::new(RefCell::new(false));
    let pair = Pair::new(PanicOnDepAndOwnerDropPairDrop(Rc::clone(
        &owner_drop_called,
    )));
    let payload: MyPayload = *catch_unwind(AssertUnwindSafe(|| drop(pair)))
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(payload, MyPayload(5));
    assert!(*owner_drop_called.borrow());
}
