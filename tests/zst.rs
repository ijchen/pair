#![allow(missing_docs, reason = "integration test")]

use pair::{HasDependent, Owner, Pair};
use std::convert::Infallible;

// ZST owner with non-ZST dependent
#[derive(Debug)]
struct ZstOwner;

#[derive(Debug, PartialEq)]
struct NonZstDependent(i32);

impl HasDependent<'_> for ZstOwner {
    type Dependent = NonZstDependent;
}

impl Owner for ZstOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
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

impl HasDependent<'_> for NonZstOwner {
    type Dependent = ZstDependent;
}

impl Owner for NonZstOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
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

impl HasDependent<'_> for BothZstOwner {
    type Dependent = ZstDependent;
}

impl Owner for BothZstOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
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
