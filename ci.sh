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
    print_header 'Building documentation (stable)...'
    RUSTDOCFLAGS='-D warnings' cargo +stable doc --document-private-items --no-deps

    print_header 'Building documentation (nightly)...'
    RUSTDOCFLAGS='-D warnings' cargo +nightly doc --document-private-items --no-deps
}

lint() {
    print_header 'Linting with cargo clippy...'
    cargo +stable clippy --no-deps --all-targets -- -D warnings
}

build() {
    print_header 'Running cargo build...'
    RUSTFLAGS='-D warnings' cargo +stable build --all-targets
}

build_nostd() {
    print_header 'Building on no_std target...'
    RUSTFLAGS='-D warnings' cargo +stable build --target thumbv6m-none-eabi
}

run_tests_stable() {
    print_header 'Running tests (stable compiler)...'
    RUSTFLAGS='-D warnings' cargo +stable test
}

run_tests_beta() {
    print_header 'Running tests (beta compiler)...'
    RUSTFLAGS='-D warnings' cargo +beta test
}

run_tests_msrv() {
    local msrv="1.85.0"

    print_header "Running tests (MSRV compiler ($msrv))..."
    RUSTFLAGS='-D warnings' cargo "+$msrv" test
}

run_tests_leak_sanitizer() {
    # NOTE: loom seems to make the leak sanitizer unhappy. I don't think that
    # combination of tests is important, so we just skip loom tests here.

    print_header 'Running tests with leak sanitizer...'
    RUSTFLAGS='-D warnings -Z sanitizer=leak' cargo +nightly test -- --skip loom
}

run_tests_miri() {
    # NOTE: some tests (containing `nomiri`) can't run under MIRI, and are
    # skipped here.
    print_header 'Running tests with MIRI...'
    RUSTFLAGS='-D warnings' MIRIFLAGS='-Zmiri-strict-provenance' cargo +nightly miri test -- --skip nomiri
}

# Run all checks
all_checks() {
    check_fmt
    check_docs
    build
    build_nostd
    lint
    run_tests_stable
    run_tests_beta
    run_tests_msrv
    run_tests_leak_sanitizer
    run_tests_miri

    print_header "All checks passed! 🎉"
}

# Main function to handle command line arguments
main() {
    local command="${1:-"all"}"

    case "$command" in
        "all")                      all_checks               ;;
        "check_fmt")                check_fmt                ;;
        "check_docs")               check_docs               ;;
        "lint")                     lint                     ;;
        "build")                    build                    ;;
        "build_nostd")              build_nostd              ;;
        "run_tests_stable")         run_tests_stable         ;;
        "run_tests_beta")           run_tests_beta           ;;
        "run_tests_msrv")           run_tests_msrv           ;;
        "run_tests_leak_sanitizer") run_tests_leak_sanitizer ;;
        "run_tests_miri")           run_tests_miri           ;;
        *)
            echo "Unknown command: $command"
            echo "Available commands: all (default), check_fmt, check_docs, lint, build, build_nostd, run_tests_stable, run_tests_beta, run_tests_msrv, run_tests_leak_sanitizer, run_tests_miri"
            exit 1
            ;;
    esac
}

main "$@"
