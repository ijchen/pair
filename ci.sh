#!/usr/bin/env bash

set -euo pipefail

# Prints text formatted as a header (colors and arrows and stuff, very cool)
print_header() {
    echo -e "\e[1;34m==>\e[0m \e[1m$1\e[0m"
}

check_fmt() {
    print_header 'Checking code formatting...'
    RUSTFLAGS='-D warnings' cargo +stable fmt --check
}

check_docs() {
    print_header 'Building documentation...'
    RUSTDOCFLAGS='-D warnings' cargo +stable doc --document-private-items --no-deps
}

lint() {
    print_header 'Linting with cargo clippy (default features)...'
    cargo +stable clippy --no-deps --all-targets -- -D warnings

    print_header 'Linting with cargo clippy (no features)...'
    cargo +stable clippy --no-deps --all-targets --no-default-features -- -D warnings

    print_header 'Linting with cargo clippy (all features)...'
    cargo +stable clippy --no-deps --all-targets --all-features -- -D warnings
}

build() {
    print_header 'Running cargo build (default features)...'
    RUSTFLAGS='-D warnings' cargo +stable build --all-targets

    print_header 'Running cargo build (no features)...'
    RUSTFLAGS='-D warnings' cargo +stable build --all-targets --no-default-features

    print_header 'Running cargo build (all features)...'
    RUSTFLAGS='-D warnings' cargo +stable build --all-targets --all-features
}

run_tests_stable() {
    print_header 'Running tests (stable compiler, default features)...'
    RUSTFLAGS='-D warnings' cargo +stable test

    # NOTE: some tests (containing `std_only`) require the `std` feature to run.
    print_header 'Running tests (stable compiler, no features)...'
    RUSTFLAGS='-D warnings' cargo +stable test --no-default-features -- --skip std_only

    print_header 'Running tests (stable compiler, all features)...'
    RUSTFLAGS='-D warnings' cargo +stable test --all-features
}

run_tests_beta() {
    # TODO: trybuild tests fail on nightly (and beta) - error messages changed

    print_header 'Running tests (beta compiler, default features)...'
    RUSTFLAGS='-D warnings' cargo +beta test -- --skip try_builds

    # NOTE: some tests (containing `std_only`) require the `std` feature to run.
    print_header 'Running tests (beta compiler, no features)...'
    RUSTFLAGS='-D warnings' cargo +beta test --no-default-features -- --skip std_only --skip try_builds

    print_header 'Running tests (beta compiler, all features)...'
    RUSTFLAGS='-D warnings' cargo +beta test --all-features -- --skip try_builds
}

run_tests_msrv() {
    local msrv="1.85.0"

    print_header "Running tests (MSRV compiler ($msrv), default features)..."
    RUSTFLAGS='-D warnings' cargo "+$msrv" test

    # NOTE: some tests (containing `std_only`) require the `std` feature to run.
    print_header "Running tests (MSRV compiler ($msrv), no features)..."
    RUSTFLAGS='-D warnings' cargo "+$msrv" test --no-default-features -- --skip std_only

    print_header "Running tests (MSRV compiler ($msrv), all features)..."
    RUSTFLAGS='-D warnings' cargo "+$msrv" test --all-features
}

run_tests_leak_sanitizer() {
    # NOTE: loom seems to make the leak sanitizer unhappy. I don't think that
    # combination of tests is important, so we just skip loom tests here.
    # TODO: trybuild tests fail on nightly (and beta) - error messages changed

    print_header 'Running tests with leak sanitizer (default features)...'
    RUSTFLAGS='-D warnings -Z sanitizer=leak' cargo +nightly test -- --skip loom --skip try_builds

    # NOTE: some tests (containing `std_only`) require the `std` feature to run.
    print_header 'Running tests with leak sanitizer (no features)...'
    RUSTFLAGS='-D warnings -Z sanitizer=leak' cargo +nightly test --no-default-features -- --skip std_only --skip loom --skip try_builds

    print_header 'Running tests with leak sanitizer (all features)...'
    RUSTFLAGS='-D warnings -Z sanitizer=leak' cargo +nightly test --all-features -- --skip loom --skip try_builds
}

# NOTE: MIRI runs pretty slowly, so splitting up the MIRI tests in CI actually
# gives a pretty meaningful speedup.
run_tests_miri_default_features() {
    # NOTE: some tests (containing `nomiri`) can't run under MIRI, and are
    # skipped here.
    print_header 'Running tests with MIRI (default features)...'
    RUSTFLAGS='-D warnings' MIRIFLAGS='-Zmiri-strict-provenance' cargo +nightly miri test -- --skip nomiri
}

run_tests_miri_no_features() {
    # NOTE: some tests (containing `nomiri`) can't run under MIRI, and are
    # skipped here.
    # NOTE: some tests (containing `std_only`) require the `std` feature to run.
    print_header 'Running tests with MIRI (no features)...'
    RUSTFLAGS='-D warnings' MIRIFLAGS='-Zmiri-strict-provenance' cargo +nightly miri test --no-default-features -- --skip nomiri --skip std_only
}

run_tests_miri_all_features() {
    # NOTE: some tests (containing `nomiri`) can't run under MIRI, and are
    # skipped here.
    print_header 'Running tests with MIRI (all features)...'
    RUSTFLAGS='-D warnings' MIRIFLAGS='-Zmiri-strict-provenance' cargo +nightly miri test --all-features -- --skip nomiri
}

# Run all checks
all_checks() {
    check_fmt
    check_docs
    build
    lint
    run_tests_stable
    run_tests_beta
    run_tests_msrv
    run_tests_leak_sanitizer
    run_tests_miri_default_features
    run_tests_miri_no_features
    run_tests_miri_all_features

    print_header "All checks passed! 🎉"
}

# Main function to handle command line arguments
main() {
    local command="${1:-"all"}"

    case "$command" in
        "all")                             all_checks                      ;;
        "check_fmt")                       check_fmt                       ;;
        "check_docs")                      check_docs                      ;;
        "build")                           build                           ;;
        "lint")                            lint                            ;;
        "run_tests_stable")                run_tests_stable                ;;
        "run_tests_beta")                  run_tests_beta                  ;;
        "run_tests_msrv")                  run_tests_msrv                  ;;
        "run_tests_leak_sanitizer")        run_tests_leak_sanitizer        ;;
        "run_tests_miri_default_features") run_tests_miri_default_features ;;
        "run_tests_miri_no_features")      run_tests_miri_no_features      ;;
        "run_tests_miri_all_features")     run_tests_miri_all_features     ;;
        *)
            echo "Unknown command: $command"
            echo "Available commands: all (default), check_fmt, check_docs, build, lint, run_tests_stable, run_tests_beta, run_tests_msrv, run_tests_leak_sanitizer, run_tests_miri_default_features, run_tests_miri_no_features, run_tests_miri_all_features"
            exit 1
            ;;
    esac
}

main "$@"
