// These markdown links are used to override the docs.rs web links present at
// the bottom of README.md. These two lists must be kept in sync, or the links
// included here will be hard-coded links to docs.rs, not relative doc links.
// ON_RELEASE: the below link(s) should be verified to match the readme, and
// this "on release" comment removed (the above one should stay).
//! [`Pair`]: Pair
#![cfg_attr(any(doc, test), doc = include_str!("../README.md"))]
#![no_std]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// TODO: move this to-do list to a to-do.md or smth
// - re-organize Cargo.toml to be more readable (esp. package section order)
// - document feature flags in README
// - clean up ci.sh

mod owner;
mod pair;
mod panicking;

pub use owner::{HasDependent, Owner};
pub use pair::Pair;
