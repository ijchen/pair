extern crate pair;

use std::convert::Infallible;

use pair::{HasDependent, Owner, Pair};

#[derive(Debug)]
struct Buff(String);

impl<'owner> HasDependent<'owner> for Buff {
    type Dependent = Vec<&'owner str>;
}

impl Owner for Buff {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(self.0.split_whitespace().collect())
    }
}

fn main() {
    let pair = Pair::new(Buff(String::from("This is a test of pair.")));
    let dep: &Vec<&str> = pair.with_dependent(|dep| dep);

    drop(pair);

    let _ = dep;
}
