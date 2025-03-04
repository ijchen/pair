use std::{cell::UnsafeCell, convert::Infallible};

use pair::{HasDependent, Owner, Pair};

struct MyOwner;
struct NotSync(UnsafeCell<()>);

impl<'owner> HasDependent<'owner> for MyOwner {
    type Dependent = NotSync;
}

impl Owner for MyOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent<'owner>(
        &'owner self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'owner>>::Dependent, Self::Error> {
        unimplemented!()
    }
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<(MyOwner, <MyOwner as HasDependent>::Dependent)>();
    check_sync::<MyOwner>();

    check_send::<Pair<MyOwner>>();
    check_sync::<Pair<MyOwner>>();
}
