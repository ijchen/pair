extern crate pair;

use pair::{Owner, Pair};

struct NotSendOrSync(*mut ());

impl<'owner> Owner<'owner> for NotSendOrSync {
    type Dependent = NotSendOrSync;
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<Pair<NotSendOrSync>>();
    check_sync::<Pair<NotSendOrSync>>();
}
