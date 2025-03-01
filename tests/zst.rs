use pair::{HasDependent, Owner, Pair};
use std::convert::Infallible;

// ZST owner with non-ZST dependent
#[derive(Debug)]
struct ZstOwner;

#[derive(Debug, PartialEq)]
struct NonZstDependent(i32);

impl<'owner> HasDependent<'owner> for ZstOwner {
    type Dependent = NonZstDependent;
}

impl Owner for ZstOwner {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<NonZstDependent, Self::Err> {
        Ok(NonZstDependent(42))
    }
}

#[test]
fn test_zst_owner() {
    let pair = Pair::new(ZstOwner);

    assert_eq!(size_of_val(pair.get_owner()), 0);
    pair.with_dependent(|dep| {
        assert_eq!(*dep, NonZstDependent(42));
    });
}

// Non-ZST owner with ZST dependent
#[derive(Debug, PartialEq)]
struct NonZstOwner(i32);

#[derive(Debug)]
struct ZstDependent;

impl<'owner> HasDependent<'owner> for NonZstOwner {
    type Dependent = ZstDependent;
}

impl Owner for NonZstOwner {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<ZstDependent, Self::Err> {
        Ok(ZstDependent)
    }
}

#[test]
fn test_zst_dependent() {
    let pair = Pair::new(NonZstOwner(123));

    assert_eq!(*pair.get_owner(), NonZstOwner(123));
    pair.with_dependent(|dep| {
        assert_eq!(size_of_val(dep), 0);
    });
}

// Both owner and dependent are ZSTs
struct BothZstOwner;

impl<'owner> HasDependent<'owner> for BothZstOwner {
    type Dependent = ZstDependent;
}

impl Owner for BothZstOwner {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(&self, _: Self::Context<'_>) -> Result<ZstDependent, Self::Err> {
        Ok(ZstDependent)
    }
}

#[test]
fn test_both_zst() {
    let pair = Pair::new(BothZstOwner);
    assert_eq!(size_of_val(pair.get_owner()), 0);
    pair.with_dependent(|dep| {
        assert_eq!(size_of_val(dep), 0);
    });
}
