#[test]
fn trycmd_contracts() {
    trycmd::TestCases::new().case("tests/cmd/*.toml");
}
