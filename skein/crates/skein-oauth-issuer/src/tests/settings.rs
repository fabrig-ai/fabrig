use super::*;
#[test]
fn settings_refuse_each_invalid_entity() {
    for (label, field, value, error) in [
        ("http outside loopback", 0_u8, b"http://issuer.example:80".as_slice(), Error::Address),
        ("empty host", 0, b"https:///path", Error::Address),
        ("userinfo", 0, b"https://user@issuer.example", Error::Address),
        ("fragment", 0, b"https://issuer.example#part", Error::Address),
        ("identifier query", 0, b"https://issuer.example?a=b", Error::Address),
        ("identifier trailing slash", 0, b"https://issuer.example/", Error::Address),
        ("loopback missing port", 0, b"http://127.0.0.1", Error::Address),
        ("loopback zero port", 0, b"http://127.0.0.1:0", Error::Address),
        ("loopback suffix", 0, b"http://localhost.evil:80", Error::Address),
        ("bad port", 0, b"https://issuer.example:65536", Error::Address),
        ("space", 1, b"https://issuer.example/a b", Error::Address),
        ("endpoint fragment", 2, b"https://issuer.example/token#x", Error::Address),
        ("userinfo endpoint", 3, b"ftp://issuer.example", Error::Address),
        ("key set endpoint", 4, b"https://", Error::Address),
        ("client id empty", 5, b"", Error::Client),
        ("client id space", 5, b"a b", Error::Client),
        ("client id control", 5, b"a\n", Error::Client),
        ("client id non-ASCII", 5, b"\xff", Error::Client),
        ("client secret empty", 6, b"", Error::Client),
        ("redirect off URL rules", 7, b"http://app.example:80", Error::Address),
    ] {
        let mut settings = settings();
        match field {
            0 => settings.issuer = boxed(value),
            1 => settings.authorization_endpoint = boxed(value),
            2 => settings.token_endpoint = boxed(value),
            3 => settings.userinfo_endpoint = boxed(value),
            4 => settings.jwks_uri = boxed(value),
            5 => settings.clients.get_mut(0).expect("client").id = boxed(value),
            6 => settings.clients.get_mut(0).expect("client").secret = Some(secret(value)),
            7 => settings.clients.get_mut(0).expect("client").redirect_uri = boxed(value),
            _ => unreachable!("table field"),
        }
        assert_eq!(Issuer::new(settings, limits()).expect_err(label), error, "{label}");
    }
}
#[test]
fn clients_are_unique_and_bounded_and_the_key_is_checked() {
    let mut duplicate = settings();
    duplicate
        .clients
        .push(Client { id: boxed(b"client"), secret: Some(secret(b"other")), redirect_uri: boxed(REDIRECT) })
        .expect("slot");
    assert_eq!(Issuer::new(duplicate, limits()).expect_err("duplicate"), Error::Client);
    let mut cap = limits();
    cap.clients = 0;
    assert_eq!(Issuer::new(settings(), cap).expect_err("too many clients"), Error::Limits);
    let mut bad = settings();
    bad.key = secret(b"not a key");
    assert_eq!(Issuer::new(bad, limits()).expect_err("bad key"), Error::Key);
}
#[test]
fn runtime_clients_are_unique_bounded_and_removable() {
    let mut settings = settings();
    settings.clients = List::with_capacity(0);
    let mut issuer = Issuer::new(settings, limits()).expect("empty provider");
    assert_eq!(issuer.authorize(QUERY), Err(Refusal::UnknownClient));
    let client = || Client { id: boxed(b"client"), secret: Some(secret(b"secret")), redirect_uri: boxed(REDIRECT) };
    issuer.add_client(client()).expect("first client, despite zero input capacity");
    let auth = issuer.authorize(QUERY).expect("added client");
    assert_eq!(issuer.add_client(client()), Err(ClientError::Duplicate));
    issuer.add_client(Client { id: boxed(b"other"), secret: None, redirect_uri: boxed(REDIRECT) }).expect("at limit");
    assert_eq!(
        issuer.add_client(Client { id: boxed(b"past"), secret: None, redirect_uri: boxed(REDIRECT) }),
        Err(ClientError::Full)
    );
    issuer.remove_client(b"client").expect("held client");
    assert_eq!(issuer.authorize(QUERY), Err(Refusal::UnknownClient));
    assert_eq!(issuer.redirect(&auth, &secret(b"code")), Err(Error::Client));
    assert_eq!(issuer.remove_client(b"client"), Err(ClientError::Unknown));
    assert!(
        issuer.authorize(&replace(QUERY, "client_id=client", "client_id=other")).is_ok(),
        "other registration kept"
    );
    issuer.add_client(client()).expect("removed slot reusable");
    issuer.authorize(QUERY).expect("added again");
    issuer.remove_client(b"other").expect("remove public client");
    issuer.remove_client(b"client").expect("remove last client");
    assert_eq!(issuer.authorize(QUERY), Err(Refusal::UnknownClient));
}

#[test]
fn runtime_clients_use_configured_client_validation() {
    let cap = limits();
    let mut issuer = issuer();
    for (id, secret_bytes, redirect, error) in [
        (boxed(b""), None, boxed(REDIRECT), Error::Client),
        (boxed(b"a b"), None, boxed(REDIRECT), Error::Client),
        (boxed(b"\xff"), None, boxed(REDIRECT), Error::Client),
        (boxed(&[b'a'; 65]), None, boxed(REDIRECT), Error::Limits),
        (boxed(b"other"), Some(boxed(b"")), boxed(REDIRECT), Error::Client),
        (boxed(b"other"), Some(boxed(&[b's'; 65])), boxed(REDIRECT), Error::Limits),
        (boxed(b"other"), None, boxed(b"http://remote.example:80"), Error::Address),
        (boxed(b"other"), None, boxed(&[b'u'; 257]), Error::Limits),
    ] {
        let client = Client { id, secret: secret_bytes.map(Secret::new), redirect_uri: redirect };
        assert_eq!(issuer.add_client(client), Err(ClientError::Invalid(error)));
        assert_eq!(issuer.settings.clients.len(), 1, "refusal changes nothing");
    }
    issuer
        .add_client(Client {
            id: boxed(&vec![b'i'; skein_lib::usize_of(cap.client_bytes)]),
            secret: Some(secret(&vec![b's'; skein_lib::usize_of(cap.client_bytes)])),
            redirect_uri: boxed(&[b"https://r/".as_slice(), &[b'x'; 246]].concat()),
        })
        .expect("all client fields at bound");
}

#[test]
fn endpoint_rules_accept_https_and_each_loopback_spelling() {
    for url in [
        b"https://issuer.example".as_slice(),
        b"https://issuer.example/path",
        b"http://127.0.0.1:8123",
        b"http://[::1]:8123",
        b"http://localhost:8123",
    ] {
        let mut settings = settings();
        settings.issuer = boxed(url);
        assert!(Issuer::new(settings, limits()).is_ok(), "{url:?}");
    }
}

#[test]
fn valid_accepts_test_limits() {
    assert!(valid(&limits()));
}

#[test]
fn valid_refuses_an_answer_past_u32() {
    let huge = Limits { claim_bytes: u32::MAX, ..limits() };
    assert!(!valid(&huge));
    assert_eq!(Issuer::new(settings(), huge).expect_err("answer length"), Error::Limits);
}

#[test]
fn valid_refuses_discovery_past_u32() {
    let huge = Limits { uri_bytes: 150_000_000, ..limits() };
    assert!(!valid(&huge));
    assert_eq!(Issuer::new(settings(), huge).expect_err("discovery length"), Error::Limits);
}

#[test]
fn valid_refuses_form_slots_past_u32() {
    let huge = Limits { request_bytes: u32::MAX, ..limits() };
    assert!(!valid(&huge));
    assert_eq!(Issuer::new(settings(), huge).expect_err("form slots"), Error::Limits);
}
