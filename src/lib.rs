// These markdown links are used to override the docs.rs web links present at
// the bottom of README.md. These two lists must be kept in sync, or the links
// included here will be hard-coded links to docs.rs, not relative doc links.
// ON_RELEASE: the below links should be verified to match the readme, and this
// "on release" comment removed (the above one should stay).
//! [`Pair`]: Pair
//! [`HasDependent`]: HasDependent
//! [`Owner`]: Owner
#![doc = include_str!("../README.md")]

mod owner;
mod pair;

pub use owner::{HasDependent, Owner};
pub use pair::Pair;

// TODO:
// - Extensive testing, including:
//   - Property-based testing
//   - Some kind of "type level fuzzing"??
//   - Test against known weird cases, like:
//     - Types with all different kinds of variances
//     - Weird drop impls (including "Oisann" types (jonhoo reference))
//     - Impure / weird Deref impls
//     - Interior mutable types
//   - https://docs.rs/trybuild test cases demonstrating that misuses of the
//     API don't compile
//   - All under MIRI
// - Soundness audits by experienced dark arts rustaceans
