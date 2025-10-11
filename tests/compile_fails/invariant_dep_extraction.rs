extern crate pair;

use std::{cell::Cell, marker::PhantomData};

use pair::{Owner, Pair};

struct InvarOwner;

impl<'owner> Owner<'owner> for InvarOwner {
    type Dependent = PhantomData<Cell<&'owner ()>>;
}

fn main() {
    let pair: Pair<InvarOwner> = Pair::new(InvarOwner, |_| PhantomData);

    // This should fail to compile
    pair.with_dependent(|dep| dep);
}
