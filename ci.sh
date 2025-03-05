cargo fmt --check && \
cargo clippy -- -Dwarnings && \
cargo clippy --no-default-features -- -Dwarnings && \
RUSTFLAGS="-Dwarnings" cargo test && \
RUSTFLAGS="-Dwarnings" cargo test --no-default-features -- --skip std_only && \
# TODO: trybuild tests fail on nightly (and beta) - error messages changed
RUSTFLAGS="-Dwarnings" cargo +nightly test -- --skip try_builds && \
RUSTFLAGS="-Dwarnings" cargo +nightly test --no-default-features -- --skip try_builds --skip std_only && \
RUSTFLAGS="-Z sanitizer=leak" cargo +nightly test -- --skip try_builds --skip loom && \
RUSTFLAGS="-Z sanitizer=leak" cargo +nightly test --no-default-features -- --skip try_builds --skip loom --skip std_only && \
(
    cp Cargo.toml Cargo.toml.backup && \
    trap 'mv Cargo.toml.backup Cargo.toml' EXIT && \
    sed -i 's/edition = "2024"/edition = "2021"/' Cargo.toml && \
    MIRIFLAGS="-Zmiri-retag-fields" cargo +nightly miri test -- --skip nomiri --skip loom
    MIRIFLAGS="-Zmiri-retag-fields" cargo +nightly miri test --no-default-features -- --skip nomiri --skip loom --skip std_only
)
