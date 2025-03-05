cargo fmt --check && \
cargo clippy -- -Dwarnings && \
RUSTFLAGS="-Dwarnings" cargo test && \
# TODO: trybuild tests fail on nightly (and beta) - error messages changed
RUSTFLAGS="-Dwarnings" cargo +nightly test -- --skip try_builds && \
RUSTFLAGS="-Z sanitizer=leak" cargo +nightly test -- --skip try_builds --skip loom && \
(
    cp Cargo.toml Cargo.toml.backup && \
    trap 'mv Cargo.toml.backup Cargo.toml' EXIT && \
    sed -i 's/edition = "2024"/edition = "2021"/' Cargo.toml && \
    MIRIFLAGS="-Zmiri-retag-fields" cargo +nightly miri test -- --skip nomiri --skip loom
)
