extern crate pair;

use std::sync::MutexGuard;

use pair::{Dependent, Owner, Pair};

struct MyOwner;
struct NotSend(MutexGuard<'static, ()>);

impl<'owner> Owner<'owner> for MyOwner {
    type Dependent = NotSend;
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<MyOwner>();
    check_sync::<(MyOwner, Dependent<'_, MyOwner>)>();

    check_send::<Pair<MyOwner>>();
    check_sync::<Pair<MyOwner>>();
}
