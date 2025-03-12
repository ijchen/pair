#![allow(missing_docs, reason = "integration test")]

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

/// Ensures `pair`s build artifacts are available in the target directory, and
/// returns a path to that target directory (relative to the current working
/// directory).
fn ensure_pair_available() -> Option<PathBuf> {
    Command::new("cargo")
        .arg("build")
        .status()
        .ok()?
        .success()
        .then_some(())?;

    Some(
        std::env::var("CARGO_TARGET_DIR")
            .map_or_else(|_| PathBuf::from("target"), PathBuf::from)
            .join("debug"),
    )
}

/// Returns the stderr from rustc attempting to compile the given file.
///
/// Makes quite a few assumptions about the environment, namely that
/// `ensure_pair_available` has been called.
///
/// # Panics
/// In quite a few situations, read the code lol
fn get_compiler_err(target_dir: &Path, test_file_path: &Path) -> String {
    let mut pair_path_arg = OsString::from("pair=");
    pair_path_arg.push(target_dir.join("libpair.rlib"));
    let mut dependency_arg = OsString::from("dependency=");
    dependency_arg.push(target_dir.join("deps"));

    let output = Command::new("rustc")
        .arg(test_file_path)
        .arg("--extern")
        .arg(pair_path_arg)
        .arg("-L")
        .arg(dependency_arg)
        .output()
        .expect("failed to get output of rustc command");

    assert!(
        !output.status.success(),
        "test compiled, but was expected not to: {test_file_path:?}"
    );

    String::from_utf8(output.stderr).expect("rustc output was not UTF-8")
}

#[test]
fn compile_fail_tests_nomiri() {
    // I would have preferred to use trybuild or compiletest_rs, but both seem
    // to require an exact .stderr match, which is not desirable for me. I don't
    // care about the *exact* error message, which may change in small ways
    // between different versions of the compiler. In fact, I was using trybuild
    // until I discovered that my tests fail on beta (which was 1.86.0) due to a
    // tiny change to what part of the source code gets highlighted. All I
    // really care about is that the code doesn't compile, and is generally for
    // the reason I expect. I wasn't able to find a better way to do this than a
    // custom little test framework. If you have a better idea, I'd welcome an
    // issue with the suggestion :)

    // Get a list of all test files in tests/compile_fails/
    // Some majorly sauced up functional magic
    let test_file_paths: Vec<_> = std::fs::read_dir("tests/compile_fails")
        .and_then(|dir_iter| {
            dir_iter
                .filter_map(|entry| {
                    entry
                        .and_then(|entry| {
                            Ok((entry.file_type()?.is_file()
                                && entry
                                    .path()
                                    .extension()
                                    .is_some_and(|extension| extension == "rs"))
                            .then_some(entry.path()))
                        })
                        .transpose()
                })
                .collect()
        })
        .expect("failed to read `tests/compile_fails` directory");

    // Ensure `pair`'s build artifacts are available, and get the target dir
    let target_dir = ensure_pair_available().expect("failed to compile `pair`");

    // For each file, ensure it fails to compile with the expected error message
    for test_file_path in test_file_paths {
        let expected_path = test_file_path.with_extension("expected");
        let expected_substrings: Vec<_> = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| panic!("failed to read file: {expected_path:?}"))
            .lines()
            .map(str::to_owned)
            .collect();

        let compiler_output = get_compiler_err(&target_dir, &test_file_path);

        for expected_substring in expected_substrings {
            assert!(
                compiler_output.contains(&expected_substring),
                "compiler error did not contain expected substring: {expected_substring}"
            );
        }
    }
}
