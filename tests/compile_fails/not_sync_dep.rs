extern crate pair;

use std::cell::UnsafeCell;

use pair::{Dependent, Owner, Pair};

struct MyOwner;
struct NotSync(UnsafeCell<()>);

impl<'owner> Owner<'owner> for MyOwner {
    type Dependent = NotSync;
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<(MyOwner, Dependent<'_, MyOwner>)>();
    check_sync::<MyOwner>();

    check_send::<Pair<MyOwner>>();
    check_sync::<Pair<MyOwner>>();
}
