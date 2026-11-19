//! Client version constants follow their headers (temper/docs/design/protocol/wires.md, 8).

#[test]
fn version_constants_equal_the_family_headers() {
    let client = &skein_codegen::values(crate::CLIENT_SCHEMA).schema;
    assert_eq!(client.family.as_bytes(), b"temper-client-wire");
    assert_eq!(crate::CLIENT_VERSION, u32::from(client.version));
    let public = &skein_codegen::values(crate::PUBLIC_SCHEMA).schema;
    assert_eq!(public.family.as_bytes(), b"temper-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version);
}
