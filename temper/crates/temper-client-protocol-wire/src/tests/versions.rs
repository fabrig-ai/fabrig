//! Client version constants follow their headers (temper/docs/design/protocol/wires.md, 8).

#[test]
#[expect(clippy::disallowed_macros, reason = "the committed schemas are version test inputs")]
fn version_constants_equal_the_family_headers() {
    let client = skein_codec::schema::parse(include_bytes!("../../schema/client-wire-v1.schema"))
        .expect("client family parses");
    assert_eq!(client.spelled(client.family()), b"temper-client-wire");
    assert_eq!(crate::CLIENT_VERSION, u32::from(client.version()));
    let public = skein_codec::schema::parse(include_bytes!("../../schema/public-v1.schema"))
        .expect("public family parses");
    assert_eq!(public.spelled(public.family()), b"temper-public-wire");
    assert_eq!(crate::PUBLIC_VERSION, public.version());
}
