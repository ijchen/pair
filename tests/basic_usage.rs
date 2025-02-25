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

#[test]
fn get_owner_stress_test() {
    // Let's just do a bunch of `get_owner(..)`s interlaced with a bunch of
    // other random stuff and see what MIRI thinks
    let mut pair = Pair::new(Buff(String::from("This is a test of pair.")));
    let owner1 = pair.get_owner();
    let owner2 = pair.get_owner();
    let owner3 = pair.get_owner();
    let dep1 = pair.with_dependent(|dep| dep);
    let owner4 = pair.get_owner();
    let dep2 = pair.with_dependent(|dep| dep);
    println!("{owner1:?}{owner2:?}{owner3:?}{owner4:?}{dep1:?}{dep2:?}");
    pair.with_dependent_mut(|dep| dep.push("hi"));
    let owner1 = pair.get_owner();
    let owner2 = pair.get_owner();
    println!("{owner1:?}{owner2:?}");
    let pair2 = pair;
    let owner1 = pair2.get_owner();
    let owner2 = pair2.get_owner();
    let owner3 = pair2.get_owner();
    let dep1 = pair2.with_dependent(|dep| dep);
    let owner4 = pair2.get_owner();
    let dep2 = pair2.with_dependent(|dep| dep);
    println!("{owner1:?}{owner2:?}{owner3:?}{owner4:?}{dep1:?}{dep2:?}");
}
