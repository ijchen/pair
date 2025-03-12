extern crate pair;

use std::{convert::Infallible, sync::MutexGuard};

use pair::{HasDependent, Owner, Pair};

struct NotSend(MutexGuard<'static, ()>);

impl<'owner> HasDependent<'owner> for NotSend {
    type Dependent = ();
}

impl Owner for NotSend {
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
    check_send::<<NotSend as HasDependent>::Dependent>();
    check_sync::<(NotSend, <NotSend as HasDependent>::Dependent)>();

    check_send::<Pair<NotSend>>();
    check_sync::<Pair<NotSend>>();
}
