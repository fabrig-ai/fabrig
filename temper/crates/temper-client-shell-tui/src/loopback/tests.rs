use super::*;
use skein_lib::Writer;

#[test]
fn only_the_exact_get_host_and_base64url_grant_reads() {
    const PORT: u16 = 43210;
    const HOST: &[u8] = b"127.0.0.1:43210";
    const PREFIX: &[u8] = b"/approved?grant=";
    // Includes both URL-only alphabet characters.
    let bytes = [0xfb; 32];
    let mut encoded = Writer::new(43);
    base64::encode(Alphabet::Url, &bytes, &mut encoded).expect("a 43-byte grant");
    let encoded = encoded.finish();
    let good = Returned { get: true, target: [PREFIX, &encoded].concat().into(), host: Some(HOST.into()) };
    assert_eq!(grant(&good, PORT).expect("valid grant").expose(), bytes);
    assert!(grant(&Returned { get: false, ..good.clone() }, PORT).is_none(), "other method");
    for host in [
        None,
        Some(b"127.0.0.1:43211".as_slice()),
        Some(b"localhost:43210"),
        Some(b"127.0.0.1"),
        Some(b"127.0.0.1:043210"),
        Some(b"[::1]:43210"),
    ] {
        assert!(grant(&Returned { host: host.map(Into::into), ..good.clone() }, PORT).is_none(), "Host {host:?}");
    }
    let tail = encoded.get(1..).expect("after the first character");
    for target in [
        [b"/approved/?grant=", &encoded[..]].concat(),
        [b"/Approved?grant=", &encoded[..]].concat(),
        [PREFIX, &encoded, b"&x=1"].concat(),
        [b"/approved?x=1&grant=", &encoded[..]].concat(),
        [PREFIX, tail].concat(),
        [PREFIX, &encoded, b"A"].concat(),
        [PREFIX, &encoded, b"="].concat(),
        [PREFIX, b"+", tail].concat(),
        [PREFIX, b"/", tail].concat(),
        [PREFIX, b"=", tail].concat(),
        [PREFIX, b"%2D", tail].concat(),
    ] {
        assert!(
            grant(&Returned { target: target.clone().into(), ..good.clone() }, PORT).is_none(),
            "target {target:?}"
        );
    }
}
