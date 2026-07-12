#[test]
fn crate_metadata_is_available_to_integration_tests() {
    assert_eq!(env!("CARGO_PKG_NAME"), "cage");
}
