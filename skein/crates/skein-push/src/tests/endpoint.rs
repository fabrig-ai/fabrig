use super::*;
#[test]
fn endpoints_preserve_origin_target_and_refuse_invalid_forms() {
    for (input, origin, target) in [
        (
            b"https://fcm.googleapis.com/fcm/send/abc".as_slice(),
            b"https://fcm.googleapis.com".as_slice(),
            b"/fcm/send/abc".as_slice(),
        ),
        (b"http://push.test:4000/push/alice", b"http://push.test:4000", b"/push/alice"),
        (b"https://127.0.0.1:65535/?x=y", b"https://127.0.0.1:65535", b"/?x=y"),
    ] {
        let parsed = Endpoint::parse(input).expect("endpoint");
        assert_eq!(parsed.origin(), origin);
        assert_eq!(parsed.target(), target);
    }
    let mut bound = Vec::from(b"https://push.test/".as_slice());
    bound.resize(ENDPOINT_BYTES, b'a');
    assert!(Endpoint::parse(&bound).is_some());
    bound.push(b'a');
    assert!(Endpoint::parse(&bound).is_none());
    for input in [
        b"".as_slice(),
        b"ftp://push.test/p",
        b"HTTPS://push.test/p",
        b"http://me@push.test/p",
        b"https://push.test/p#fragment",
        b"http://push.test/p q",
        b"http://push.test",
        b"http:///p",
        b"http://push.test:0/p",
        b"http://push.test:65536/p",
        b"http://push.test:/p",
        b"http://push.test:80:81/p",
        b"http://[::1]/p",
        b"http://push.test/p\x7f",
    ] {
        assert!(Endpoint::parse(input).is_none(), "{input:?}");
    }
    let mut host = Vec::from(b"https://".as_slice());
    host.extend_from_slice(&[b'a'; 253]);
    host.push(b'/');
    assert!(Endpoint::parse(&host).is_some());
    host.insert(8, b'a');
    assert!(Endpoint::parse(&host).is_none());
}
