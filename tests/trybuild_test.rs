#[test]
fn try_builds() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild_fails/*.rs");
}
