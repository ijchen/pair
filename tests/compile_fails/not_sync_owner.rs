extern crate pair;

use std::{cell::UnsafeCell, convert::Infallible};

use pair::{Dependent, HasDependent, Owner, Pair};

struct NotSync(UnsafeCell<()>);

impl<'owner> HasDependent<'owner> for NotSync {
    type Dependent = ();
}

impl Owner for NotSync {
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
    check_send::<(NotSync, Dependent<'_, NotSync>)>();
    check_sync::<Dependent<'_, NotSync>>();

    check_send::<Pair<NotSync>>();
    check_sync::<Pair<NotSync>>();
}
