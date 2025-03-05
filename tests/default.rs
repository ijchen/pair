#![allow(missing_docs, reason = "integration test")]

use pair::{HasDependent, Owner, Pair};
use std::{convert::Infallible, fmt::Debug};

#[derive(Default, Debug, PartialEq)]
struct DefaultOwner<T>(T);

impl<'owner, T> HasDependent<'owner> for DefaultOwner<T> {
    type Dependent = &'owner T;
}

impl<T> Owner for DefaultOwner<T> {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(&self.0)
    }
}

fn test_default<T: Default + PartialEq + Debug>() {
    let pair: Pair<DefaultOwner<T>> = Pair::default();

    assert_eq!(pair.get_owner().0, T::default());
    pair.with_dependent(|dep| {
        assert_eq!(dep, &&T::default());
    });

    assert_eq!(pair.into_owner().0, T::default());
}

#[test]
fn test_default_implementation() {
    test_default::<()>();
    test_default::<u8>();
    test_default::<u16>();
    test_default::<u32>();
    test_default::<u64>();
    test_default::<u128>();
    test_default::<usize>();
    test_default::<i8>();
    test_default::<i16>();
    test_default::<i32>();
    test_default::<i64>();
    test_default::<i128>();
    test_default::<f32>();
    test_default::<f64>();
    test_default::<bool>();
    test_default::<isize>();
    test_default::<char>();
    test_default::<&str>();
    test_default::<[u8; 10]>();
    test_default::<Option<()>>();
    test_default::<Vec<bool>>();
    test_default::<String>();
    test_default::<Box<f32>>();
}
