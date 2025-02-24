use std::convert::Infallible;

use pair::{HasDependent, Owner, Pair};

#[derive(Debug)]
struct Buff(String);

impl<'owner> HasDependent<'owner> for Buff {
    type Dependent = Vec<&'owner str>;
}

impl Owner for Buff {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err> {
        Ok(self.0.split_whitespace().collect())
    }
}

#[test]
fn basic_usage() {
    let mut pair = Pair::new(Buff(String::from("This is a test of pair.")));
    let owner: &Buff = pair.get_owner();
    let dep: &Vec<&str> = pair.with_dependent(|dep| dep);

    assert_eq!(owner.0, "This is a test of pair.");
    assert_eq!(dep, &["This", "is", "a", "test", "of", "pair."]);

    pair.with_dependent_mut(|dep| dep.push("hi"));
    pair.with_dependent_mut(|dep| dep.push("hey"));
    assert_eq!(
        pair.with_dependent(|d| d),
        &["This", "is", "a", "test", "of", "pair.", "hi", "hey"]
    );
    pair.with_dependent_mut(|dep| dep.sort());
    assert_eq!(
        pair.with_dependent(|d| d),
        &["This", "a", "hey", "hi", "is", "of", "pair.", "test"]
    );

    let last_word = pair.with_dependent_mut(|dep| dep.pop());
    assert_eq!(last_word, Some("test"));
    assert_eq!(
        pair.with_dependent(|d| d),
        &["This", "a", "hey", "hi", "is", "of", "pair."]
    );

    let owner: Buff = pair.into_owner();
    assert_eq!(owner.0, "This is a test of pair.");
}
