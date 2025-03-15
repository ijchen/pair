#![allow(missing_docs, reason = "integration test")]

use std::convert::Infallible;

use pair::{HasDependent, Owner, Pair};

trait MyTrait {
    fn get(&self) -> &i32;
}

struct MyConcrete(i32);

impl MyTrait for MyConcrete {
    fn get(&self) -> &i32 {
        &self.0
    }
}

impl<'owner> HasDependent<'owner> for dyn MyTrait {
    type Dependent = &'owner i32;
}
impl Owner for dyn MyTrait {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(self.get())
    }
}

#[test]
fn dyn_trait_owner() {
    let pair = Pair::new_from_box(Box::new(MyConcrete(69)) as Box<dyn MyTrait>);
    let owner: &dyn MyTrait = pair.owner();
    let dep: &i32 = pair.with_dependent(|dep| dep);

    assert_eq!(owner.get(), &69);
    assert_eq!(dep, &69);

    let owner: Box<dyn MyTrait> = pair.into_boxed_owner();
    assert_eq!(owner.get(), &69);
}
