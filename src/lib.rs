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
// - Crate/module level documentation
