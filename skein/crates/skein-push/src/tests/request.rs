use super::*;
#[test]
fn token_claims_signature_and_headers_are_wire_exact() {
    let vapid = Vapid::from_pkcs8_pem(PEM).expect("OpenSSL PKCS#8");
    let sealed = seal(&vapid, b"mailto:push@example.test", &subscription(), &push(), WALL).expect("seal");
    assert_eq!(sealed.origin.as_ref(), b"https://fcm.googleapis.com");
    assert_eq!(sealed.target.as_ref(), b"/fcm/send/abc");
    let fields: Vec<(&[u8], &[u8])> = sealed.headers.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();
    assert_eq!(
        &fields[..5],
        &[
            (b"TTL".as_slice(), b"60".as_slice()),
            (b"Urgency", b"high"),
            (b"Topic", b"updates_1"),
            (b"Content-Encoding", b"aes128gcm"),
            (b"Content-Type", b"application/octet-stream")
        ]
    );
    assert_eq!(fields[5].0, b"Authorization");
    let authorization = core::str::from_utf8(fields[5].1).expect("ascii");
    let (token, key) = authorization.strip_prefix("vapid t=").expect("scheme").split_once(", k=").expect("key");
    assert_eq!(base64::decode(Alphabet::Url, key.as_bytes(), 65).expect("public").as_ref(), vapid.public());
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(
        base64::decode(Alphabet::Url, parts[0].as_bytes(), 128).expect("header").as_ref(),
        b"{\"typ\":\"JWT\",\"alg\":\"ES256\"}"
    );
    assert_eq!(
        base64::decode(Alphabet::Url, parts[1].as_bytes(), 256).expect("claims").as_ref(),
        b"{\"aud\":\"https://fcm.googleapis.com\",\"exp\":1700043200,\"sub\":\"mailto:push@example.test\"}"
    );
    let (input, _) = token.rsplit_once('.').expect("signed prefix");
    let signature = base64::decode(Alphabet::Url, parts[2].as_bytes(), 64).expect("signature");
    assert!(skein_tls::verify_es256(
        vapid.public(),
        input.as_bytes(),
        &signature.as_ref().try_into().expect("fixed signature")
    ));
    let without_topic =
        seal(&vapid, b"https://example.test/contact", &subscription(), &Push { topic: None, ..push() }, WALL)
            .expect("no topic");
    assert_eq!(without_topic.headers.len(), 5);
    assert_eq!(without_topic.headers[2].0.as_ref(), b"Content-Encoding");
    for (urgency, wire) in [
        (Urgency::VeryLow, b"very-low".as_slice()),
        (Urgency::Low, b"low"),
        (Urgency::Normal, b"normal"),
        (Urgency::High, b"high"),
    ] {
        assert_eq!(urgency.as_bytes(), wire);
    }
}
#[test]
fn every_seal_error_refuses_at_its_entrance() {
    let vapid = Vapid::from_pkcs8_pem(PEM).expect("test key");
    let mut sub = subscription();
    let big = [0; 3994];
    for (payload, topic, ttl, subject, error) in [
        (b"".as_slice(), None, 0, b"mailto:p@e.test".as_slice(), SealError::Payload),
        (big.as_slice(), None, 0, b"mailto:p@e.test", SealError::Payload),
        (b"x", Some(b"".as_slice()), 0, b"mailto:p@e.test", SealError::Topic),
        (b"x", Some(b"bad+topic"), 0, b"mailto:p@e.test", SealError::Topic),
        (b"x", Some([b'a'; 33].as_slice()), 0, b"mailto:p@e.test", SealError::Topic),
        (b"x", None, 2_419_201, b"mailto:p@e.test", SealError::Ttl),
        (b"x", None, 0, b"", SealError::Subject),
        (b"x", None, 0, b"http://example.test", SealError::Subject),
        (b"x", None, 0, b"mailto:space here", SealError::Subject),
        (b"x", None, 0, b"mailto:quote\"", SealError::Subject),
        (b"x", None, 0, b"https:back\\slash", SealError::Subject),
    ] {
        assert_eq!(
            seal(&vapid, subject, &sub, &Push { payload, topic, ttl, urgency: Urgency::Normal }, WALL)
                .expect_err("refused"),
            error
        );
    }
    sub.p256dh = [0; 65];
    assert_eq!(seal(&vapid, b"mailto:p@e.test", &sub, &push(), WALL).expect_err("keys"), SealError::Keys);
    sub = subscription();
    sub.endpoint = Box::from(b"ftp://push.test/p".as_slice());
    assert_eq!(seal(&vapid, b"mailto:p@e.test", &sub, &push(), WALL).expect_err("endpoint"), SealError::Endpoint);
    let mut subject = Vec::from(b"mailto:".as_slice());
    subject.resize(SUBJECT_BYTES, b'a');
    let payload = [0; PAYLOAD_BYTES];
    let topic = [b'_'; 32];
    let push = Push { payload: &payload, ttl: 2_419_200, urgency: Urgency::Normal, topic: Some(&topic) };
    seal(&vapid, &subject, &subscription(), &push, WALL).expect("inclusive bounds");
    subject.push(b'a');
    assert_eq!(seal(&vapid, &subject, &subscription(), &push, WALL).expect_err("subject bound"), SealError::Subject);
}
#[test]
fn vapid_accepts_only_bounded_pkcs8_p256_pem() {
    assert!(Vapid::from_pkcs8_pem(PEM).is_some());
    for bytes in [
        b"".as_slice(),
        b"-----BEGIN EC PRIVATE KEY-----\nAAAA\n-----END EC PRIVATE KEY-----",
        b"-----BEGIN PRIVATE KEY-----\n!!!!\n-----END PRIVATE KEY-----",
        b"-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----",
        &[b'a'; 1025],
    ] {
        assert!(Vapid::from_pkcs8_pem(bytes).is_none());
    }
}
#[test]
fn answers_classify_every_status_row_and_boundary() {
    for (status, expected) in [
        (199, Answer::Failed(199)),
        (200, Answer::Made),
        (299, Answer::Made),
        (300, Answer::Failed(300)),
        (400, Answer::Refused(400)),
        (401, Answer::Refused(401)),
        (402, Answer::Failed(402)),
        (403, Answer::Refused(403)),
        (404, Answer::Gone),
        (410, Answer::Gone),
        (413, Answer::TooLarge),
        (429, Answer::Later(Some(9))),
        (499, Answer::Failed(499)),
        (500, Answer::Later(Some(9))),
        (599, Answer::Later(Some(9))),
        (600, Answer::Failed(600)),
        (u16::MAX, Answer::Failed(u16::MAX)),
    ] {
        assert_eq!(answer(status, Some(9)), expected);
    }
    assert_eq!(answer(503, None), Answer::Later(None));
}
