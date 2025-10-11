#![allow(missing_docs, reason = "integration test")]

use pair::{Owner, Pair};

#[derive(Debug)]
struct SimpleOwner(i32);

impl<'owner> Owner<'owner> for SimpleOwner {
    type Dependent = &'owner i32;
}

impl SimpleOwner {
    fn make_dependent(&self) -> &i32 {
        &self.0
    }
}

// Pair that owns another Pair
#[derive(Debug)]
struct PairOwner {
    value: i32,
    inner_pair: Pair<SimpleOwner>,
}

#[derive(Debug)]
struct PairOwnerDependent<'owner> {
    value_ref: &'owner i32,
    inner_pair_ref: &'owner Pair<SimpleOwner>,
}

impl<'owner> Owner<'owner> for PairOwner {
    type Dependent = PairOwnerDependent<'owner>;
}

impl PairOwner {
    fn make_dependent(&self) -> PairOwnerDependent<'_> {
        PairOwnerDependent {
            value_ref: &self.value,
            inner_pair_ref: &self.inner_pair,
        }
    }
}

#[test]
fn test_pair_owning_pair() {
    let inner_pair = Pair::new(SimpleOwner(42), SimpleOwner::make_dependent);

    let pair_owner = PairOwner {
        value: 100,
        inner_pair,
    };

    let outer_pair = Pair::new(pair_owner, PairOwner::make_dependent);

    outer_pair.with_dependent(|outer_dep| {
        assert_eq!(*outer_dep.value_ref, 100);

        // Access the inner pair's owner and dependent
        assert_eq!(outer_dep.inner_pair_ref.owner().0, 42);
        assert_eq!(outer_dep.inner_pair_ref.with_dependent(|dep| dep), &&42);
    });

    let PairOwner { value, inner_pair } = outer_pair.into_owner();
    assert_eq!(value, 100);
    assert_eq!(inner_pair.owner().0, 42);
    assert_eq!(inner_pair.with_dependent(|dep| dep), &&42);
}
