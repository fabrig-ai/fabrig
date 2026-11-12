use super::*;
#[test]
fn each_form_and_body_fault_is_refused_without_recording() {
    let (mut service, good) = fixture();
    let mut wrong_target = good.clone();
    wrong_target.target = Box::from(b"/elsewhere".as_slice());
    let mut method = good.clone();
    method.method = Box::from(b"GET".as_slice());
    for (request, status) in [(wrong_target, 404), (method, 405)] {
        assert_eq!(service.take(request, WALL).status, status);
    }
    for (name, value) in [
        (b"TTL".as_slice(), b"".as_slice()),
        (b"TTL", b"-1"),
        (b"TTL", b"2419201"),
        (b"Urgency", b"urgent"),
        (b"Topic", b""),
        (b"Topic", b"bad+topic"),
        (b"Topic", &[b'a'; 33]),
        (b"Content-Encoding", b"gzip"),
    ] {
        let mut request = good.clone();
        field(&mut request, name, value);
        assert_eq!(service.take(request, WALL).status, 400, "{name:?}: {value:?}");
    }
    let mut missing = good.clone();
    missing.headers[0].name = Box::from(b"Other".as_slice());
    assert_eq!(service.take(missing, WALL).status, 400, "missing TTL");
    for (length, rs, idlen, point, status) in [
        (102, 4096_u32, 65, 4, 400),
        (103, 17, 65, 4, 400),
        (103, 4096, 64, 4, 400),
        (103, 4096, 65, 3, 400),
        (4097, 4096, 65, 4, 413),
        (4183, 4096, 65, 4, 413),
    ] {
        let mut request = good.clone();
        let mut body = Vec::from(request.body.as_ref());
        body.resize(length, 0);
        body[16..20].copy_from_slice(&rs.to_be_bytes());
        body[20] = idlen;
        body[21] = point;
        request.body = body.into_boxed_slice();
        assert_eq!(service.take(request, WALL).status, status, "body {length}/{rs}/{idlen}/{point}");
    }
    assert_eq!(service.received(), []);
    for urgency in [b"very-low".as_slice(), b"low", b"normal", b"high"] {
        let mut request = good.clone();
        field(&mut request, b"Urgency", urgency);
        assert_eq!(service.take(request, WALL).status, 201);
    }
    let mut no_urgency = good.clone();
    no_urgency.headers[1].name = Box::from(b"Other".as_slice());
    let taken = service.take(no_urgency, WALL);
    assert_eq!(taken.status, 201);
    assert_eq!(taken.received.expect("record").urgency.as_ref(), b"normal");
    for (length, rs) in [(103, 18_u32), (4096, 4096)] {
        let mut request = good.clone();
        let mut body = Vec::from(request.body.as_ref());
        body.resize(length, 0);
        body[16..20].copy_from_slice(&rs.to_be_bytes());
        request.body = body.into_boxed_slice();
        assert_eq!(service.take(request, WALL).status, 201);
    }
}
#[test]
fn token_faults_are_unauthorized_or_forbidden() {
    let (mut service, good) = fixture();
    let mut missing = good.clone();
    missing.headers[5].name = Box::from(b"Other".as_slice());
    assert_eq!(service.take(missing, WALL).status, 401);
    for (auth, status) in [(b"bearer nope".as_slice(), 401), (b"vapid t=abc", 401), (b"vapid t=a.b.c, k=AAAA", 403)] {
        let mut request = good.clone();
        field(&mut request, b"Authorization", auth);
        assert_eq!(service.take(request, WALL).status, status);
    }
    let header = b"{\"typ\":\"JWT\",\"alg\":\"ES256\"}";
    for (label, claims) in [
        ("aud", b"{\"aud\":\"http://other.test\",\"exp\":1700043200,\"sub\":\"mailto:p@example.test\"}".as_slice()),
        ("expired", b"{\"aud\":\"http://push.test:4000\",\"exp\":1700000000,\"sub\":\"mailto:p@example.test\"}"),
        ("past 24h", b"{\"aud\":\"http://push.test:4000\",\"exp\":1700086401,\"sub\":\"mailto:p@example.test\"}"),
        ("subject", b"{\"aud\":\"http://push.test:4000\",\"exp\":1700043200,\"sub\":\"http://example.test\"}"),
        ("JSON", b"not JSON"),
    ] {
        let mut request = good.clone();
        field(&mut request, b"Authorization", &authorization(header, claims, false));
        assert_eq!(service.take(request, WALL).status, 403, "{label}");
    }
    for (head, corrupt) in [(b"{\"alg\":\"ES256\"}".as_slice(), false), (header.as_slice(), true)] {
        let mut request = good.clone();
        field(&mut request, b"Authorization", &authorization(head, GOOD, corrupt));
        assert_eq!(service.take(request, WALL).status, 403);
    }
    let mut malformed = good.clone();
    field(&mut malformed,b"Authorization",b"vapid t=not.a.token.extra, k=BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8");
    assert_eq!(service.take(malformed, WALL).status, 403);
    assert_eq!(service.received(), []);
    let last = b"{\"aud\":\"http://push.test:4000\",\"exp\":1700086400,\"sub\":\"https://example.test/contact\"}";
    let mut request = good;
    field(&mut request, b"Authorization", &authorization(header, last, false));
    assert_eq!(service.take(request, WALL).status, 201, "24h inclusive");
}
