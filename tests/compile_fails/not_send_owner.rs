extern crate pair;

use std::sync::MutexGuard;

use pair::{Dependent, Owner, Pair};

struct NotSend(MutexGuard<'static, ()>);

impl<'owner> Owner<'owner> for NotSend {
    type Dependent = ();
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<Dependent<'_, NotSend>>();
    check_sync::<(NotSend, Dependent<'_, NotSend>)>();

    check_send::<Pair<NotSend>>();
    check_sync::<Pair<NotSend>>();
}
