#![allow(missing_docs, reason = "integration test")]

use std::convert::Infallible;

use pair::{Dependent, HasDependent, Owner, Pair};

#[derive(Debug, PartialEq)]
struct Buff(String);

impl<'owner> HasDependent<'owner> for Buff {
    type Dependent = Vec<&'owner str>;
}

impl Owner for Buff {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(self.0.split_whitespace().collect())
    }
}

#[test]
fn basic_usage() {
    let mut pair = Pair::new(Buff(String::from("This is a test of pair.")));
    let owner: &Buff = pair.owner();
    let dep: &Vec<&str> = pair.with_dependent(|dep| dep);

    assert_eq!(owner.0, "This is a test of pair.");
    assert_eq!(dep, &["This", "is", "a", "test", "of", "pair."]);

    pair.with_dependent_mut(|dep| dep.push("hi"));
    pair.with_dependent_mut(|dep| dep.push("hey"));
    assert_eq!(
        pair.with_both(|owner, dep| (owner, dep)),
        (
            &Buff(String::from("This is a test of pair.")),
            &vec!["This", "is", "a", "test", "of", "pair.", "hi", "hey"]
        )
    );
    pair.with_dependent_mut(|dep| dep.sort_unstable());
    assert_eq!(
        pair.with_dependent(|d| d),
        &["This", "a", "hey", "hi", "is", "of", "pair.", "test"]
    );

    let last_word = pair.with_both_mut(|owner, dep| {
        assert_eq!(owner, &Buff(String::from("This is a test of pair.")));
        dep.pop()
    });
    assert_eq!(last_word, Some("test"));
    assert_eq!(
        pair.with_dependent(|d| d),
        &["This", "a", "hey", "hi", "is", "of", "pair."]
    );

    let owner: Buff = pair.into_owner();
    assert_eq!(owner.0, "This is a test of pair.");
}

#[test]
fn basic_api_stress_test() {
    // Let's just do a bunch of the basic API functions interlaced together and
    // see what MIRI thinks
    let mut pair = Pair::new(Buff(String::from("This is a test of pair.")));
    let owner1 = pair.owner();
    let owner2 = pair.owner();
    let owner3 = pair.owner();
    let dep1 = pair.with_dependent(|dep| dep);
    let owner4 = pair.owner();
    let (owner5, dep2) = pair.with_both(|owner, dep| (owner, dep));
    println!("{owner1:?}{owner2:?}{owner3:?}{owner4:?}{owner5:?}{dep1:?}{dep2:?}");
    pair.with_dependent_mut(|dep| dep.push("hey"));
    let owner1 = pair.owner();
    let dep1 = pair.with_dependent(|dep| dep);
    let owner2 = pair.owner();
    let dep2 = pair.with_dependent(|dep| dep);
    println!("{owner1:?}{owner2:?}{dep1:?}{dep2:?}");
    pair.with_both_mut(|owner, dep| {
        if owner.0.contains("hey") {
            dep.push("what's up");
        }
    });
    pair.with_dependent_mut(|dep| dep.push("hello"));
    let owner1 = pair.owner();
    let owner2 = pair.owner();
    println!("{owner1:?}{owner2:?}");
    #[expect(
        clippy::redundant_closure_call,
        reason = "I'm just doing weird stuff for funsies and testing"
    )]
    let new_pair = (|x| x)(std::convert::identity(pair));
    let owner1 = new_pair.owner();
    let owner2 = new_pair.owner();
    let owner3 = new_pair.owner();
    let dep1 = new_pair.with_dependent(|dep| dep);
    let owner4 = new_pair.owner();
    let (owner5, dep2) = new_pair.with_both(|owner, dep| (owner, dep));
    println!("{owner1:?}{owner2:?}{owner3:?}{owner4:?}{owner5:?}{dep1:?}{dep2:?}");
    let owner1 = new_pair.owner();
    let dep1 = new_pair.with_dependent(|dep| dep);
    let owner2 = new_pair.owner();
    let dep2 = new_pair.with_dependent(|dep| dep);
    println!("{owner1:?}{owner2:?}{dep1:?}{dep2:?}");
}
