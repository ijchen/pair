extern crate pair;

use std::{cell::Cell, convert::Infallible, marker::PhantomData};

use pair::{HasDependent, Owner, Pair};

struct InvarOwner;

impl<'owner> HasDependent<'owner> for InvarOwner {
    type Dependent = PhantomData<Cell<&'owner ()>>;
}

impl Owner for InvarOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(PhantomData)
    }
}

fn main() {
    let pair: Pair<InvarOwner> = Pair::new(InvarOwner);

    // This should fail to compile
    pair.with_dependent(|dep| dep);
}
