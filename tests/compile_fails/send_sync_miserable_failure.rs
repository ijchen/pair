extern crate pair;

use std::convert::Infallible;

use pair::{HasDependent, Owner, Pair};

struct NotSendOrSync(*mut ());

impl<'owner> HasDependent<'owner> for NotSendOrSync {
    type Dependent = NotSendOrSync;
}

impl Owner for NotSendOrSync {
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
    check_send::<Pair<NotSendOrSync>>();
    check_sync::<Pair<NotSendOrSync>>();
}
