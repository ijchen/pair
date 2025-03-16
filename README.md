# Pair

<!-- Badges -->
[![GitHub](https://img.shields.io/badge/Source-ijchen/pair-FFD639?labelColor=555555&logo=github)](https://github.com/ijchen/pair)
[![crates.io](https://img.shields.io/crates/v/pair?logo=rust)](https://crates.io/crates/pair)
[![docs.rs](https://img.shields.io/docsrs/pair?logo=docs.rs)](https://docs.rs/pair)
[![License](https://img.shields.io/crates/l/pair)](#)

Safe API for generic self-referential pairs of owner and dependent.

You define how to construct a dependent type from a reference to an owning type,
and [`Pair`] will carefully bundle them together in a safe and freely movable
self-referential struct.

# Example Usage

A typical use case might look something like this:
```rust
use pair::{Dependent, HasDependent, Owner, Pair};

// Let's say you have some buffer type that contains a string
#[derive(Debug)]
pub struct MyBuffer {
    data: String,
}

// And you have some borrowing "parsed" representation, containing string slices
#[derive(Debug)]
pub struct Parsed<'a> {
    tokens: Vec<&'a str>,
}

// And you have some expensive parsing function you only want to run once
fn parse(buffer: &MyBuffer) -> Parsed<'_> {
    let tokens = buffer.data.split_whitespace().collect();
    Parsed { tokens }
}



// You would then implement HasDependent and Owner for MyBuffer:

// Defines the owner/dependent relationship between MyBuffer and Parsed<'_>
impl<'owner> HasDependent<'owner> for MyBuffer {
    type Dependent = Parsed<'owner>;
}

// Define how to make a Parsed<'_> from a &MyBuffer
impl Owner for MyBuffer {
    type Context<'a> = (); // We don't need any extra args to `make_dependent`
    type Error = std::convert::Infallible; // Our example parsing can't fail

    fn make_dependent(&self, _: ()) -> Result<Dependent<'_, Self>, Self::Error> {
        Ok(parse(self))
    }
}



// You can now use MyBuffer in a Pair:
fn main() {
    // A Pair can be constructed from an owner value (MyBuffer, in this example)
    let mut pair = Pair::new(MyBuffer {
        data: String::from("this is an example"),
    });

    // You can obtain a reference to the owner via a reference to the pair
    let owner: &MyBuffer = pair.owner();
    assert_eq!(owner.data, "this is an example");

    // You can access a reference to the dependent via a reference to the pair,
    // but only within a provided closure.
    // See the documentation of `Pair::with_dependent` for details.
    let kebab = pair.with_dependent(|parsed: &Parsed<'_>| parsed.tokens.join("-"));
    assert_eq!(kebab, "this-is-an-example");

    // However, if the dependent is covariant over its lifetime (as our example
    // Parsed<'_> is) you can trivially extract the dependent from the closure.
    // This will not compile if the dependent is not covariant.
    let parsed: &Parsed<'_> = pair.with_dependent(|parsed| parsed);
    assert_eq!(parsed.tokens, ["this", "is", "an", "example"]);

    // You can obtain a mutable reference to the dependent via a mutable
    // reference to the pair, but only within a provided closure.
    // See the documentation of `Pair::with_dependent_mut` for details.
    pair.with_dependent_mut(|parsed| parsed.tokens.pop());
    assert_eq!(pair.with_dependent(|parsed| parsed.tokens.len()), 3);

    // If you're done with the dependent, you can recover the owner.
    // This will drop the dependent.
    let my_buffer: MyBuffer = pair.into_owner();
}
```

# How it Works

*Note: the implementation details described in this section are not part of the
crate's public API, and are subject to change.*

Under the hood, [`Pair`] moves the owner onto the heap, giving it a stable
memory address. It is then borrowed and used to construct the dependent, which
is also moved onto the heap. The dependent is type-erased, so that its
inexpressible self-referential lifetime goes away. All exposed APIs are careful
to ensure type and aliasing rules are upheld, regardless of anything safe user
code could do. When the owner needs to be dropped or recovered, the dependent
will first be recovered and dropped, ending the borrow of the owner. At that
point, the owner can safely be recovered and the `Pair` deconstructed.

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
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

<!--
docs.rs documentation links for rendered markdown (ex, on GitHub)
These are overridden when include_str!(..)'d in lib.rs
-->
<!-- ON_RELEASE: the below link(s) should be updated, and this comment removed -->
[`Pair`]: https://docs.rs/pair/__CRATE_VERSION_HERE__/pair/struct.Pair.html
