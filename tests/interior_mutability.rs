use pair::{HasDependent, Owner, Pair};
use std::cell::{Cell, RefCell};
use std::convert::Infallible;

// Test with interior-mutable owner
struct InteriorMutableOwner {
    value: RefCell<i32>,
}

impl<'owner> HasDependent<'owner> for InteriorMutableOwner {
    type Dependent = &'owner RefCell<i32>;
}

impl Owner for InteriorMutableOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(&self.value)
    }
}

#[test]
fn test_interior_mutable_owner() {
    let pair = Pair::new(InteriorMutableOwner {
        value: RefCell::new(42),
    });

    // Mutate the owner's value through the RefCell
    *pair.get_owner().value.borrow_mut() = 100;

    // Verify the change is visible to the dependent
    assert_eq!(*pair.with_dependent(|dep| dep).borrow(), 100);

    // Mutate the owner's value through the dependent
    *pair.with_dependent(|dep| dep).borrow_mut() = 210;

    // Verify the change is visible to the owner
    assert_eq!(*pair.get_owner().value.borrow(), 210);
}

// Test with interior-mutable dependent
struct RegularOwner {
    value: i32,
}

struct InteriorMutableDependent<'a> {
    value_cell: Cell<i32>,
    original_ref: &'a i32,
}

impl<'owner> HasDependent<'owner> for RegularOwner {
    type Dependent = InteriorMutableDependent<'owner>;
}

impl Owner for RegularOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(InteriorMutableDependent {
            value_cell: Cell::new(self.value),
            original_ref: &self.value,
        })
    }
}

#[test]
fn test_interior_mutable_dependent() {
    let pair = Pair::new(RegularOwner { value: 42 });

    // Mutate the dependent's Cell value
    pair.with_dependent(|dep| dep.value_cell.set(100));
    assert_eq!(pair.with_dependent(|dep| dep).value_cell.get(), 100);
    // The RegularOwner's original value should be unchanged
    assert_eq!(pair.with_dependent(|dep| dep).original_ref, &42);
}

// Test with interior-mutable owner and dependent
struct BothInteriorMutableOwner {
    value: RefCell<i32>,
}

struct BothInteriorMutableDependent<'a> {
    owner_ref: &'a RefCell<i32>,
    local_value: Cell<i32>,
}

impl<'owner> HasDependent<'owner> for BothInteriorMutableOwner {
    type Dependent = BothInteriorMutableDependent<'owner>;
}

impl Owner for BothInteriorMutableOwner {
    type Context<'a> = ();
    type Error = Infallible;

    fn make_dependent(
        &self,
        (): Self::Context<'_>,
    ) -> Result<<Self as HasDependent<'_>>::Dependent, Self::Error> {
        Ok(BothInteriorMutableDependent {
            owner_ref: &self.value,
            local_value: Cell::new(*self.value.borrow()),
        })
    }
}

#[test]
fn test_both_interior_mutable() {
    let pair = Pair::new(BothInteriorMutableOwner {
        value: RefCell::new(42),
    });

    // Mutate the owner
    *pair.get_owner().value.borrow_mut() = 100;
    assert_eq!(*pair.with_dependent(|dep| dep).owner_ref.borrow(), 100);
    assert_eq!(pair.with_dependent(|dep| dep).local_value.get(), 42);

    // Mutate the dependent
    pair.with_dependent(|dep| dep).local_value.set(200);
    assert_eq!(pair.with_dependent(|dep| dep).local_value.get(), 200);
}
