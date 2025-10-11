#![allow(missing_docs, reason = "integration test")]

use pair::{Owner, Pair};

struct ContraOwner;

impl<'owner> Owner<'owner> for ContraOwner {
    type Dependent = fn(&'owner u32);
}

fn parse<'a>(_: &'a ContraOwner) -> fn(&'a u32) {
    |_| {}
}

fn contravariant_example<'self_borrow>(pair: &'self_borrow Pair<ContraOwner>) {
    // The with_dependent function requires that the provided closure works for
    // *any* dependent lifetime. Because of that, our closure cannot assume that
    // the function pointer it's given can be called with any specific short
    // lifetime. In fact, the *only* lifetime we can assume the function pointer
    // requires is 'static - this is okay since we can give a 'static reference
    // to a function pointer that expects any shorter lifetime (since &'a u32 is
    // covariant in 'a). But any lifetime potentially less than 'static might
    // not live as long as the body of the function pointer was allowed to
    // assume - therefore, this cannot compile:
    // let f: &'self_borrow fn(&u32) = pair.with_dependent(|f| f);
    // But this can:
    let f: &'self_borrow fn(&'static u32) = pair.with_dependent(|f| f);

    f(&42);
}

#[test]
fn examples() {
    let pair: Pair<ContraOwner> = Pair::new(ContraOwner, parse);
    contravariant_example(&pair);
}
