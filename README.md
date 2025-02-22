Safe API for generic self-referential pairs of owner and dependent.

You define how to construct a dependent type from a reference to an owning type,
and `Pair` will carefully bundle them together in a safe and freely movable
self-referential struct.

# DO NOT USE THIS LIBRARY

As of right now, I have absolutely no idea whether or not this API is sound. You
should *absolutely not* use this library for anything that matters at this point
in time.

# API Overview

The core API looks *roughly* like this (some details omitted for brevity):
```rust
// You specify the dependent (borrowing) type - ex, &'owner str or Foo<'owner>
pub trait HasDependent<'owner> {
    type Dependent;
}

// You specify how to make the dependent from a reference to the owner type
pub trait Owner: for<'any> HasDependent<'any> {
    fn make_dependent(&self) -> <Self as HasDependent<'_>>::Dependent;
}

pub struct Pair<O: Owner> { ... }

impl<O: Owner> Pair<O> {
    // A Pair can be constructed from an owner
    pub fn new(owner: O) -> Self { ... }

    // The owner can be borrowed
    pub fn get_owner(&self) -> &O { ... }

    // The dependent can be borrowed, although normally only through a closure
    // (See the documentation of `with_dependent` for details)
    pub fn with_dependent<'self_borrow, F, T>(&'self_borrow self, f: F) -> T
    where
        F: for<'any> FnOnce(&'self_borrow <O as HasDependent<'any>>::Dependent) -> T
    { ... }

    // The dependent can be borrowed mutably, although only through a closure
    // (See the documentation of `with_dependent_mut` for details)
    pub fn with_dependent_mut<'self_borrow, F, T>(&'self_borrow mut self, f: F) -> T
    where
        F: for<'any> FnOnce(&'self_borrow mut <O as HasDependent<'any>>::Dependent) -> T
    { ... }

    // The owner can be recovered by consuming the Pair (dropping the dependent)
    pub fn into_owner(self) -> O { ... }
}
```

# How it Works

Under the hood, `Pair` moves the owner onto the heap, giving it a stable memory
address. It is then borrowed and used to construct the dependent, which is also
moved onto the heap. The dependent is type-erased, so that its inexpressible
self-referential lifetime goes away. All exposed APIs are careful to ensure type
and aliasing rules are upheld, regardless of anything safe user code could do.
When the owner needs to be dropped or recovered, the dependent will first be
recovered and dropped, ending the borrow of the owner. At that point, the owner
can safely be recovered and the `Pair` deconstructed.

# Related Projects

| Crate | Macro free | No `alloc` | Maintained | Soundness |
| ----- | :--------: | :-------------: | :--------: | :-------: |
| `pair` | ✅ | ❌ | ✅ | No known issues |
| [`ouroboros`](https://crates.io/crates/ouroboros) | ❌ | ❌ | ✅ | ⚠️ [Unsound](https://github.com/someguynamedjosh/ouroboros/issues/122) ⚠️ |
| [`self_cell`](https://crates.io/crates/self_cell) | ❌ | ❌ | ✅ | No known issues |
| [`yoke`](https://crates.io/crates/yoke) | ❌ | ✅ | ✅ | ⚠️ [Unsound](https://github.com/unicode-org/icu4x/issues/2095) ⚠️ |
| [`selfie`](https://crates.io/crates/selfie) | ✅ | ✅ | ❌ | ⚠️ [Unsound](https://github.com/prokopyl/selfie?tab=readme-ov-file#abandoned-this-crate-is-unsound-and-no-longer-maintained_) ⚠️ |
| [`nolife`](https://crates.io/crates/nolife) | ❌ | ❌ | ✅ | No known issues |
| [`owning_ref`](https://crates.io/crates/owning_ref) | ✅ | ✅ | ❌ | ⚠️ [Unsound](https://github.com/Kimundi/owning-ref-rs/issues/77) ⚠️ |
| [`rental`](https://crates.io/crates/rental) | ❌ | ✅ | ❌ | ⚠️ [Unsound](https://github.com/Voultapher/self_cell?tab=readme-ov-file#related-projects) ⚠️ |
| [`fortify`](https://crates.io/crates/fortify) | ❌ | ❌ | ✅ | No known issues |
| [`loaned`](https://crates.io/crates/loaned) | ❌ | ✅ | ✅ | No known issues |
| [`selfref`](https://crates.io/crates/selfref) | ❌ | ✅ | ✅ | No known issues |
| [`self-reference`](https://crates.io/crates/self-reference) | ❌ | ✅ | ❌ | ⚠️ [Unsound](https://github.com/ArtBlnd/self-reference/issues/1) ⚠️ |
| [`zc`](https://crates.io/crates/zc) | ❌ | ✅ | ✅ | No known issues |

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
