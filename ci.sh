cargo clippy && \
cargo test && \
(
    cp Cargo.toml Cargo.toml.backup && \
    trap 'mv Cargo.toml.backup Cargo.toml' EXIT && \
    sed -i 's/edition = "2024"/edition = "2021"/' Cargo.toml && \
    MIRIFLAGS="-Zmiri-retag-fields" cargo +nightly miri test -- --skip nomiri
)
