#![allow(missing_docs, reason = "integration test")]

use std::convert::Infallible;

use pair::{Dependent, HasDependent, Owner, Pair};

#[derive(Debug)]
struct Buff<T: ?Sized>(T);

impl Buff<[u8]> {
    pub fn new<const N: usize>(data: [u8; N]) -> Box<Self> {
        Box::new(Buff(data))
    }
}

impl<'owner> HasDependent<'owner> for Buff<[u8]> {
    type Dependent = (&'owner [u8], usize);
}

impl Owner for Buff<[u8]> {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(&self, (): Self::Context<'_>) -> Result<Dependent<'_, Self>, Self::Error> {
        let start = usize::min(1, self.0.len());
        let end = self.0.len().saturating_sub(1);
        Ok((&self.0[start..end], self.0.len()))
    }
}

#[test]
fn unsized_owner() {
    let mut pair = Pair::new_from_box(Buff::new([2, 69, 42, 5, 6, 7, 8]));
    let owner: &Buff<[u8]> = pair.owner();
    let dep: &(&[u8], usize) = pair.with_dependent(|dep| dep);

    assert_eq!(owner.0, [2, 69, 42, 5, 6, 7, 8]);
    assert_eq!(*dep, ([69, 42, 5, 6, 7].as_slice(), 7));

    pair.with_dependent_mut(|dep| dep.1 -= 2);
    pair.with_dependent_mut(|dep| dep.1 *= 10);
    assert_eq!(
        *pair.with_dependent(|d| d),
        ([69, 42, 5, 6, 7].as_slice(), 50)
    );
    pair.with_dependent_mut(|dep| std::ops::AddAssign::add_assign(&mut dep.1, 5));
    assert_eq!(
        *pair.with_dependent(|d| d),
        ([69, 42, 5, 6, 7].as_slice(), 55)
    );

    let n = pair.with_dependent_mut(|dep| {
        dep.1 = 0;
        dep.1
    });
    assert_eq!(n, 0);
    assert_eq!(
        *pair.with_dependent(|d| d),
        ([69, 42, 5, 6, 7].as_slice(), 0)
    );

    let owner: Box<Buff<[u8]>> = pair.into_boxed_owner();
    assert_eq!(owner.0, [2, 69, 42, 5, 6, 7, 8]);
}
