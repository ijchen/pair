#![allow(missing_docs, reason = "integration test")]

use pair::{HasDependent, Owner, Pair};
use std::convert::Infallible;

#[derive(PartialEq)]
struct MultiPartOwner {
    field1: String,
    field2: Vec<i32>,
    field3: bool,
    field4: Box<u32>,
}

struct MultiPartDependent<'a> {
    string: &'a str,
    int: &'a i32,
    boolean: &'a bool,
}

impl<'owner> HasDependent<'owner> for MultiPartOwner {
    type Dependent = MultiPartDependent<'owner>;
}

impl Owner for MultiPartOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(MultiPartDependent {
            string: &self.field1,
            int: &self.field2[0],
            boolean: &self.field3,
        })
    }
}

#[test]
fn test_multiple_borrows() {
    #![expect(clippy::bool_assert_comparison, reason = "for clarity and consistency")]

    let owner = MultiPartOwner {
        field1: "Hello, world!".to_string(),
        field2: vec![3, 1, 4, 1, 5],
        field3: true,
        field4: Box::default(),
    };

    let pair = Pair::new(owner);

    pair.with_dependent(|dep| {
        assert_eq!(dep.string, "Hello, world!");
        assert_eq!(*dep.int, 3);
        assert_eq!(*dep.boolean, true);
    });

    let owner = pair.into_owner();
    assert_eq!(owner.field1, "Hello, world!");
    assert_eq!(owner.field2, [3, 1, 4, 1, 5]);
    assert_eq!(owner.field3, true);
}
