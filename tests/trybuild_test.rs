#![allow(missing_docs, reason = "integration test")]

#[test]
fn try_builds_nomiri() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild_fails/*.rs");
}
