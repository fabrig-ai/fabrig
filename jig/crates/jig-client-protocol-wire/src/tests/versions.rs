//! Client version constants follow their headers (jig/docs/design/protocol/README.md, 7).

#[test]
fn version_constants_equal_the_family_headers() {
    let client = &skein_codegen::values(crate::CLIENT_SCHEMA).schema;
    assert_eq!(client.family.as_bytes(), b"jig-client-wire");
    assert_eq!(crate::VERSION, u32::from(client.version));
    assert_eq!(crate::VERSION, 43);
    let public = &skein_codegen::values(crate::PUBLIC_SCHEMA).schema;
    assert_eq!(public.family.as_bytes(), b"jig-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version);
}
