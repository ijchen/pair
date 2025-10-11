#![allow(missing_docs, reason = "integration test")]

use pair::{Owner, Pair};

// 1-ZST owner with non-ZST dependent
#[derive(Debug)]
struct OneZstOwner;
const _: () = assert!(align_of::<OneZstOwner>() == 1);

#[derive(Debug, PartialEq)]
struct NonZstDependent(i32);

impl Owner<'_> for OneZstOwner {
    type Dependent = NonZstDependent;
}

impl OneZstOwner {
    #[expect(clippy::unused_self, reason = "convoluted example for test")]
    fn make_dependent(&self) -> NonZstDependent {
        NonZstDependent(42)
    }
}

#[test]
fn test_1zst_owner() {
    let pair = Pair::new(OneZstOwner, OneZstOwner::make_dependent);

    assert_eq!(size_of_val(pair.owner()), 0);
    assert_eq!(*pair.with_dependent(|dep| dep), NonZstDependent(42));
}

// Non-ZST owner with 1-ZST dependent
#[derive(Debug, PartialEq)]
struct Non1ZstOwner(i32);

#[derive(Debug)]
struct OneZstDependent;
const _: () = assert!(align_of::<OneZstDependent>() == 1);

impl Owner<'_> for Non1ZstOwner {
    type Dependent = OneZstDependent;
}

impl Non1ZstOwner {
    #[expect(clippy::unused_self, reason = "convoluted example for test")]
    fn make_dependent(&self) -> OneZstDependent {
        OneZstDependent
    }
}

#[test]
fn test_1zst_dependent() {
    let pair = Pair::new(Non1ZstOwner(123), Non1ZstOwner::make_dependent);

    assert_eq!(*pair.owner(), Non1ZstOwner(123));
    assert_eq!(size_of_val(pair.with_dependent(|dep| dep)), 0);
}

// Both owner and dependent are 1-ZSTs
struct Both1ZstOwner;
const _: () = assert!(align_of::<Both1ZstOwner>() == 1);

impl Owner<'_> for Both1ZstOwner {
    type Dependent = OneZstDependent;
}

impl Both1ZstOwner {
    #[expect(clippy::unused_self, reason = "convoluted example for test")]
    fn make_dependent(&self) -> OneZstDependent {
        OneZstDependent
    }
}

#[test]
fn test_both_1zst() {
    let pair = Pair::new(Both1ZstOwner, Both1ZstOwner::make_dependent);
    assert_eq!(size_of_val(pair.owner()), 0);
    assert_eq!(size_of_val(pair.with_dependent(|dep| dep)), 0);
}

// /////////////////////////////////////////////////////////////////////////////

// Non-1 alignment ZST owner with non-ZST dependent
#[derive(Debug)]
struct BigZstOwner([u64; 0]);
const _: () = assert!(align_of::<BigZstOwner>() > 1);

impl Owner<'_> for BigZstOwner {
    type Dependent = NonZstDependent;
}

impl BigZstOwner {
    #[expect(clippy::unused_self, reason = "convoluted example for test")]
    fn make_dependent(&self) -> NonZstDependent {
        NonZstDependent(23)
    }
}

#[test]
fn test_bigzst_owner() {
    let pair = Pair::new(BigZstOwner([]), BigZstOwner::make_dependent);

    assert_eq!(size_of_val(pair.owner()), 0);
    assert_eq!(*pair.with_dependent(|dep| dep), NonZstDependent(23));
}

// Non-ZST owner with Non-1 alignment ZST dependent
#[derive(Debug, PartialEq)]
struct NonBigZstOwner(i32);

#[derive(Debug)]
struct BigZstDependent([u64; 0]);
const _: () = assert!(align_of::<BigZstDependent>() > 1);

impl Owner<'_> for NonBigZstOwner {
    type Dependent = BigZstDependent;
}

impl NonBigZstOwner {
    #[expect(clippy::unused_self, reason = "convoluted example for test")]
    fn make_dependent(&self) -> BigZstDependent {
        BigZstDependent([])
    }
}

#[test]
fn test_bigzst_dependent() {
    let pair = Pair::new(NonBigZstOwner(789), NonBigZstOwner::make_dependent);

    assert_eq!(*pair.owner(), NonBigZstOwner(789));
    assert_eq!(size_of_val(pair.with_dependent(|dep| dep)), 0);
}

// Both owner and dependent are Non-1 alignment ZSTs
struct BothBigZstOwner([u64; 0]);
const _: () = assert!(align_of::<BothBigZstOwner>() > 1);

impl Owner<'_> for BothBigZstOwner {
    type Dependent = BigZstDependent;
}

impl BothBigZstOwner {
    #[expect(clippy::unused_self, reason = "convoluted example for test")]
    fn make_dependent(&self) -> BigZstDependent {
        BigZstDependent([])
    }
}

#[test]
fn test_both_bigzst() {
    let pair = Pair::new(BothBigZstOwner([]), BothBigZstOwner::make_dependent);
    assert_eq!(size_of_val(pair.owner()), 0);
    assert_eq!(size_of_val(pair.with_dependent(|dep| dep)), 0);
}
