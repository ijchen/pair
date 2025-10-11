<!-- ON_RELEASE: Add changelog entry for new version and remove this comment -->

# Major Version 0

## v0.3.0

Refactored `Owner`, `HasDependent`, and `Pair`:
- `HasDependent` trait renamed to `Owner`
- `Owner` trait removed
- `Pair` constructors now use a provided `FnOnce` to construct the dependent

## v0.2.0

Implemented the core functionality of this crate:
- Defined `Pair` struct
- Defined `Owner` and `HasDependent` traits
- Defined `Dependent` type alias

## v0.1.0

Initial (empty) release.
