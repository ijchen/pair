#![allow(missing_docs, reason = "integration test")]

use pair::{Owner, Pair};
use std::cell::{Cell, RefCell};

// Test with interior-mutable owner
struct InteriorMutableOwner {
    value: RefCell<i32>,
}

impl<'owner> Owner<'owner> for InteriorMutableOwner {
    type Dependent = &'owner RefCell<i32>;
}

impl InteriorMutableOwner {
    fn make_dependent(&self) -> &RefCell<i32> {
        &self.value
    }
}

#[test]
fn test_interior_mutable_owner() {
    let pair = Pair::new(
        InteriorMutableOwner {
            value: RefCell::new(42),
        },
        InteriorMutableOwner::make_dependent,
    );

    // Mutate the owner's value through the RefCell
    *pair.owner().value.borrow_mut() = 100;

    // Verify the change is visible to the dependent
    assert_eq!(*pair.with_dependent(|dep| dep).borrow(), 100);

    // Mutate the owner's value through the dependent
    *pair.with_dependent(|dep| dep).borrow_mut() = 210;

    // Verify the change is visible to the owner
    assert_eq!(*pair.owner().value.borrow(), 210);
}

// Test with interior-mutable dependent
struct RegularOwner {
    value: i32,
}

struct InteriorMutableDependent<'a> {
    value_cell: Cell<i32>,
    original_ref: &'a i32,
}

impl<'owner> Owner<'owner> for RegularOwner {
    type Dependent = InteriorMutableDependent<'owner>;
}

impl RegularOwner {
    fn make_dependent(&self) -> InteriorMutableDependent<'_> {
        InteriorMutableDependent {
            value_cell: Cell::new(self.value),
            original_ref: &self.value,
        }
    }
}

#[test]
fn test_interior_mutable_dependent() {
    let pair = Pair::new(RegularOwner { value: 42 }, RegularOwner::make_dependent);

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

impl<'owner> Owner<'owner> for BothInteriorMutableOwner {
    type Dependent = BothInteriorMutableDependent<'owner>;
}

impl BothInteriorMutableOwner {
    fn make_dependent(&self) -> BothInteriorMutableDependent<'_> {
        BothInteriorMutableDependent {
            owner_ref: &self.value,
            local_value: Cell::new(*self.value.borrow()),
        }
    }
}

#[test]
fn test_both_interior_mutable() {
    let pair = Pair::new(
        BothInteriorMutableOwner {
            value: RefCell::new(42),
        },
        BothInteriorMutableOwner::make_dependent,
    );

    // Mutate the owner
    *pair.owner().value.borrow_mut() = 100;
    assert_eq!(*pair.with_dependent(|dep| dep).owner_ref.borrow(), 100);
    assert_eq!(pair.with_dependent(|dep| dep).local_value.get(), 42);

    // Mutate the dependent
    pair.with_dependent(|dep| dep).local_value.set(200);
    assert_eq!(pair.with_dependent(|dep| dep).local_value.get(), 200);
}
