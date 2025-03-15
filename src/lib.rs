// These markdown links are used to override the docs.rs web links present at
// the bottom of README.md. These two lists must be kept in sync, or the links
// included here will be hard-coded links to docs.rs, not relative doc links.
//! [`Pair`]: Pair
#![cfg_attr(any(doc, test), doc = include_str!("../README.md"))]
#![no_std]

extern crate alloc;

mod drop_guard;
mod owner;
mod pair;

pub use owner::{Dependent, HasDependent, Owner};
pub use pair::Pair;
