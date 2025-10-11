extern crate pair;

use std::cell::UnsafeCell;

use pair::{Dependent, Owner, Pair};

struct NotSync(UnsafeCell<()>);

impl<'owner> Owner<'owner> for NotSync {
    type Dependent = ();
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<(NotSync, Dependent<'_, NotSync>)>();
    check_sync::<Dependent<'_, NotSync>>();

    check_send::<Pair<NotSync>>();
    check_sync::<Pair<NotSync>>();
}
