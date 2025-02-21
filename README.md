Safe API for generic self-referential pairs of owner and dependent.

TODO: description

The API looks generally like this (some details omitted for brevity):
```rust
pub trait HasDependent<'owner> {
    type Dependent;
}

pub trait Owner: for<'any> HasDependent<'any> {
    fn make_dependent(&self) -> <Self as HasDependent<'_>>::Dependent;
}

pub struct Pair<O: Owner> { ... }

impl<O: Owner> Pair<O> {
    pub fn new(owner: O) -> Self { ... }

    pub fn get_owner(&self) -> &O { ... }

    pub fn with_dependent<'self_borrow, F, T>(&'self_borrow self, f: F) -> T
    where
        F: for<'any> FnOnce(&'self_borrow <O as HasDependent<'any>>::Dependent) -> T
    { ... }

    pub fn with_dependent_mut<'self_borrow, F, T>(&'self_borrow mut self, f: F) -> T
    where
        F: for<'any> FnOnce(&'self_borrow mut <O as HasDependent<'any>>::Dependent) -> T
    { ... }

    pub fn into_owner(self) -> O { ... }
}
```

# DO NOT USE THIS LIBRARY

As of right now, I have absolutely no idea whether or not this API is sound. You
should *absolutely not* use this library for anything that matters at this point
in time.

# License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
