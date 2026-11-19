//! Client version constants follow their headers (jig/docs/design/protocol/README.md, 7).

#[test]
fn version_constants_equal_the_family_headers() {
<<<<<<< HEAD
    let client = &skein_codegen::values(crate::CLIENT_SCHEMA).schema;
    assert_eq!(client.family.as_bytes(), b"jig-client-wire");
    assert_eq!(crate::VERSION, u32::from(client.version));
    assert_eq!(crate::VERSION, 37);
    let public = &skein_codegen::values(crate::PUBLIC_SCHEMA).schema;
    assert_eq!(public.family.as_bytes(), b"jig-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version);
=======
    let client =
        concat!(include_str!("../../schema/client-v38.schema"), include_str!("../../schema/client-v38-part1.schema"));
    let client = skein_codec::schema::parse(client.as_bytes()).expect("client family parses");
    assert_eq!(client.spelled(client.family()), b"jig-client-wire");
    assert_eq!(crate::VERSION, u32::from(client.version()));
    assert_eq!(crate::VERSION, 38);
    let public =
        skein_codec::schema::parse(include_bytes!("../../schema/public-v1.schema")).expect("public family parses");
    assert_eq!(public.spelled(public.family()), b"jig-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version());
>>>>>>> refs/hand5/g9120
}
