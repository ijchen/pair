extern crate pair;

use std::{convert::Infallible, sync::MutexGuard};

use pair::{Dependent, HasDependent, Owner, Pair};

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
    ) -> Result<Dependent<'owner, Self>, Self::Error> {
        unimplemented!()
    }
}

fn check_send<T: Send>() {}
fn check_sync<T: Sync>() {}

fn main() {
    check_send::<Dependent<'_, NotSend>>();
    check_sync::<(NotSend, Dependent<'_, NotSend>)>();

    check_send::<Pair<NotSend>>();
    check_sync::<Pair<NotSend>>();
}
