//! Client version constants follow their headers (jig/docs/design/protocol/README.md, 7).

#[test]
#[expect(clippy::disallowed_macros, reason = "the committed schemas are version test inputs")]
fn version_constants_equal_the_family_headers() {
    let client =
        concat!(include_str!("../../schema/client-v21.schema"), include_str!("../../schema/client-v21-part1.schema"));
    let client = skein_codec::schema::parse(client.as_bytes()).expect("client family parses");
    assert_eq!(client.spelled(client.family()), b"jig-client-wire");
    assert_eq!(crate::VERSION, u32::from(client.version()));
    let public =
        skein_codec::schema::parse(include_bytes!("../../schema/public-v1.schema")).expect("public family parses");
    assert_eq!(public.spelled(public.family()), b"jig-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version());
}
