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
    string_ref: &'a str,
    int_ref: &'a i32,
    bool_ref: &'a bool,
}

impl<'owner> HasDependent<'owner> for MultiPartOwner {
    type Dependent = MultiPartDependent<'owner>;
}

impl Owner for MultiPartOwner {
    type Context<'a> = ();
    type Err = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Err> {
        Ok(MultiPartDependent {
            string_ref: &self.field1,
            int_ref: &self.field2[0],
            bool_ref: &self.field3,
        })
    }
}

#[test]
fn test_multiple_borrows() {
    let owner = MultiPartOwner {
        field1: "Hello, world!".to_string(),
        field2: vec![3, 1, 4, 1, 5],
        field3: true,
        field4: Box::default(),
    };

    let pair = Pair::new(owner);

    pair.with_dependent(|dep| {
        assert_eq!(dep.string_ref, "Hello, world!");
        assert_eq!(*dep.int_ref, 3);
        assert_eq!(*dep.bool_ref, true);
    });

    let owner = pair.into_owner();
    assert_eq!(owner.field1, "Hello, world!");
    assert_eq!(owner.field2, [3, 1, 4, 1, 5]);
    assert_eq!(owner.field3, true);
}
