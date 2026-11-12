#[test]
fn grant_sizes_take_credentials_source_names_and_generated_identities() {
    let name = u32::try_from(crate::values::SOURCE_NAME_BYTES).unwrap();
    for (credential, identity, expected) in [
        (name - 1, name - 1, name),
        (name, name, name),
        (name + 1, name, name + 1),
        (name, name + 1, name + 1),
        (4096, name, 4096),
    ] {
        assert_eq!(crate::values::grant_bytes(credential, identity), Some(expected));
    }
}
