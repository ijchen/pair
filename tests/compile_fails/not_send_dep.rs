extern crate pair;

use std::{convert::Infallible, sync::MutexGuard};

use pair::{HasDependent, Owner, Pair};

struct MyOwner;
struct NotSend(MutexGuard<'static, ()>);

impl<'owner> HasDependent<'owner> for MyOwner {
    type Dependent = NotSend;
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
    check_send::<MyOwner>();
    check_sync::<(MyOwner, <MyOwner as HasDependent>::Dependent)>();

    check_send::<Pair<MyOwner>>();
    check_sync::<Pair<MyOwner>>();
}
