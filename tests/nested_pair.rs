use pair::{HasDependent, Owner, Pair};
use std::convert::Infallible;

#[derive(Debug)]
struct SimpleOwner(i32);

impl<'owner> HasDependent<'owner> for SimpleOwner {
    type Dependent = &'owner i32;
}

impl Owner for SimpleOwner {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err> {
        Ok(&self.0)
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

impl<'owner> HasDependent<'owner> for PairOwner {
    type Dependent = PairOwnerDependent<'owner>;
}

impl Owner for PairOwner {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err> {
        Ok(PairOwnerDependent {
            value_ref: &self.value,
            inner_pair_ref: &self.inner_pair,
        })
    }
}

#[test]
fn test_pair_owning_pair() {
    let inner_pair = Pair::new(SimpleOwner(42));

    let pair_owner = PairOwner {
        value: 100,
        inner_pair,
    };

    let outer_pair = Pair::new(pair_owner);

    outer_pair.with_dependent(|outer_dep| {
        assert_eq!(*outer_dep.value_ref, 100);

        // Access the inner pair's owner and dependent
        assert_eq!(outer_dep.inner_pair_ref.get_owner().0, 42);
        assert_eq!(outer_dep.inner_pair_ref.with_dependent(|dep| dep), &&42);
    });

    let PairOwner { value, inner_pair } = outer_pair.into_owner();
    assert_eq!(value, 100);
    assert_eq!(inner_pair.get_owner().0, 42);
    assert_eq!(inner_pair.with_dependent(|dep| dep), &&42);
}
