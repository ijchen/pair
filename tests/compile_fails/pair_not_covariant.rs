#![allow(unused)]

extern crate pair;

use std::{convert::Infallible, marker::PhantomData};

use pair::{HasDependent, Owner, Pair};

fn main() {}

struct Foo<'a>(PhantomData<&'a ()>);

struct Covariant<T>(PhantomData<T>);

impl<'owner> HasDependent<'owner> for Foo<'_> {
    type Dependent = ();
}
impl Owner for Foo<'_> {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        unimplemented!()
    }
}

fn uh_oh<'shorter, 'longer>()
where
    'longer: 'shorter,
{
    // Foo<'a> should be covariant over 'a, so this should be okay
    let mut foo_shorter: Foo<'shorter> = Foo(PhantomData);
    let foo_longer: Foo<'longer> = Foo(PhantomData);
    foo_shorter = foo_longer;

    // Covariant<T> should be covariant over T, so this should be okay
    let mut cov_shorter: Covariant<Foo<'shorter>> = Covariant(PhantomData);
    let cov_longer: Covariant<Foo<'longer>> = Covariant(PhantomData);
    cov_shorter = cov_longer;

    // Pair<O> should *not* be covariant over O, so this should *not* be okay
    let mut pair_shorter: Pair<Foo<'shorter>> = Pair::new(Foo(PhantomData));
    let pair_longer: Pair<Foo<'longer>> = Pair::new(Foo(PhantomData));
    pair_shorter = pair_longer;
}
