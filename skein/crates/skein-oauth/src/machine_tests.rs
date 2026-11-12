use crate::*;
use alloc::boxed::Box;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use skein_lib::{Duration, Queue, Time, Wall, bytes};

fn boxed(input: &[u8]) -> Box<[u8]> {
    bytes::copy_of(input)
}
fn secret(input: &[u8]) -> skein_lib::Secret {
    skein_lib::Secret::new(boxed(input))
}
fn time(seconds: u64) -> Time {
    Time::from_nanos(seconds.saturating_mul(1_000_000_000))
}
fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.saturating_mul(1_000_000_000))
}
fn limits() -> ClientLimits {
    ClientLimits {
        document: Limits { record_bytes: 4096, tokens: 128, ..ClientLimits::TEST.document },
        code_bytes: 128,
        parameter_bytes: 256,
        resource_bytes: 128,
        nonce_bytes: 64,
        metadata_bytes: 1024,
        key_set_bytes: 4096,
        keys: 4,
        modulus_bits: 4096,
        ..ClientLimits::TEST
    }
}
fn registration(public: bool, wire: WireFormat) -> Registration {
    Registration {
        authorization_url: boxed(b"https://issuer.example/authorize"),
        token_endpoint: boxed(b"https://issuer.example/token"),
        client_id: boxed(b"client"),
        redirect_uri: if public {
            boxed(b"http://127.0.0.1:2345/callback")
        } else {
            boxed(b"https://web.example/callback")
        },
        scope: boxed(b"read profile"),
        wire,
        client_secret: if public { None } else { Some(secret(b"secret")) },
        pkce_for_confidential: false,
        metadata_claim: None,
        terms: None,
    }
}
fn state() -> skein_lib::Secret {
    secret(b"0123456789abcdef")
}
fn verifier() -> skein_lib::Secret {
    secret(b"dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")
}
fn step(client: &mut Client, event: Event) -> Option<Request> {
    let mut out = Queue::with_capacity(1);
    client.step(event, &mut out);
    out.pop()
}
fn request(result: Option<Request>) -> HttpRequest {
    match result {
        Some(Request::Http(request)) => request,
        _ => panic!("HTTP demand expected"),
    }
}
fn failure(result: Option<Request>) -> Failure {
    match result {
        Some(Request::Failed { failure }) => failure,
        _ => panic!("failure expected"),
    }
}
fn token_body() -> Box<[u8]> {
    token_body_with_scope(None)
}
fn token_body_with_scope(scope: Option<&[u8]>) -> Box<[u8]> {
    let mut granted = None;
    if let Some(scope) = scope {
        granted = Some(boxed(scope));
    }
    encode_response(
        &TokenResponse {
            access_token: secret(b"access"),
            refresh_token: Some(secret(b"refresh-new")),
            expires_in: 30,
            scope: granted,
            id_token: None,
            earliest_refresh_at: None,
        },
        &limits().document,
    )
    .expect("response")
}
fn answered(id: u64, status: u16, body: &[u8], now: u64) -> Event {
    Event::Http(HttpResponse {
        id,
        status,
        body: boxed(body),
        retry_after: Duration::ZERO,
        evidence: HttpEvidence::Response,
        now: time(now),
        wall: wall(now),
    })
}

fn refresh(registration: Registration, client_id: Option<Box<[u8]>>) -> Event {
    Event::Refresh {
        registration,
        prior: RefreshState {
            key: 7,
            generation: 1,
            refresh_token: secret(b"seed"),
            issued_client_id: client_id,
            granted_scopes: None,
            subject: None,
            id_token: None,
        },
        now: time(1),
    }
}

#[test]
fn public_sign_in_uses_s256_state_and_one_code_exchange() {
    let mut client = Client::new(limits()).expect("limits");
    let visit = step(
        &mut client,
        Event::SignIn {
            issued_client_id: None,
            id_token: None,
            subject: None,
            nonce: None,
            port: None,
            registration: registration(true, WireFormat::Form),
            key: 7,
            generation: 0,
            state: state(),
            verifier: Some(verifier()),
            now: time(1),
        },
    );
    let Some(Request::Visit { url }) = visit else { panic!("visit expected") };
    assert!(bytes::find(url.expose(), b"code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM").is_some());
    assert!(bytes::find(url.expose(), b"state=0123456789abcdef").is_some());
    let uri = boxed(b"http://127.0.0.1:2345/callback");
    let http = request(step(
        &mut client,
        Event::Redirected {
            client_id: None,
            uri,
            state: state(),
            code: Some(secret(b"one-use")),
            error: None,
            now: time(2),
        },
    ));
    assert!(bytes::find(&http.body, b"code_verifier=").is_some());
    assert!(bytes::find(&http.body, b"code=one-use").is_some());
    let Some(Request::Tokens { record, .. }) = step(&mut client, answered(http.id, 200, &token_body(), 3)) else {
        panic!("tokens expected")
    };
    assert_eq!(record.key, 7);
    assert_eq!(record.generation, 1);
    assert_eq!(record.refresh_token.as_ref().expect("refresh token").expose(), b"refresh-new");
    assert!(client.is_done());
    assert!(step(&mut client, answered(http.id, 200, &token_body(), 4)).is_none(), "late answer ignored");
}

#[test]
fn confidential_sign_in_sends_secret_in_json_only_to_token_endpoint() {
    let mut client = Client::new(limits()).expect("limits");
    let visit = step(
        &mut client,
        Event::SignIn {
            issued_client_id: None,
            id_token: None,
            subject: None,
            nonce: None,
            port: None,
            registration: registration(false, WireFormat::Json),
            key: 1,
            generation: 8,
            state: state(),
            verifier: None,
            now: time(1),
        },
    );
    let Some(Request::Visit { url }) = visit else { panic!("visit expected") };
    assert!(bytes::find(url.expose(), b"secret").is_none());
    let http = request(step(
        &mut client,
        Event::Redirected {
            client_id: None,
            uri: boxed(b"https://web.example/callback"),
            state: state(),
            code: Some(secret(b"grant")),
            error: None,
            now: time(2),
        },
    ));
    assert!(bytes::find(&http.body, b"\"client_secret\":\"secret\"").is_some());
    assert!(bytes::find(&http.body, b"\"grant_type\":\"authorization_code\"").is_some());
    let Some(Request::Tokens { record, .. }) = step(&mut client, answered(http.id, 200, &token_body(), 3)) else {
        panic!("tokens expected")
    };
    assert_eq!(record.generation, 9);
}

#[test]
fn confidential_registration_can_require_pkce_too() {
    let mut client = Client::new(limits()).expect("limits");
    let mut registration = registration(false, WireFormat::Form);
    registration.pkce_for_confidential = true;
    let Some(Request::Visit { url }) = step(
        &mut client,
        Event::SignIn {
            issued_client_id: None,
            id_token: None,
            subject: None,
            nonce: None,
            port: None,
            registration,
            key: 1,
            generation: 0,
            state: state(),
            verifier: Some(verifier()),
            now: time(1),
        },
    ) else {
        panic!("visit expected")
    };
    assert!(bytes::find(url.expose(), b"code_challenge_method=S256").is_some());
    let http = request(step(
        &mut client,
        Event::Redirected {
            client_id: None,
            uri: boxed(b"https://web.example/callback"),
            state: state(),
            code: Some(secret(b"grant")),
            error: None,
            now: time(2),
        },
    ));
    assert!(bytes::find(&http.body, b"code_verifier=").is_some());
    assert!(bytes::find(&http.body, b"client_secret=secret").is_some());
}

#[test]
fn public_redirect_spellings_supply_the_exact_loopback_bind_address() {
    for (uri, expected) in [
        (b"http://localhost:2345/callback".as_slice(), IpAddr::V4(Ipv4Addr::LOCALHOST)),
        (b"http://127.0.0.1:2345/callback".as_slice(), IpAddr::V4(Ipv4Addr::LOCALHOST)),
        (b"http://[::1]:2345/callback".as_slice(), IpAddr::V6(Ipv6Addr::LOCALHOST)),
    ] {
        assert_eq!(redirect_address(uri).expect("public redirect"), SocketAddr::new(expected, 2345));
        let mut client = Client::new(limits()).expect("limits");
        let mut registration = registration(true, WireFormat::Form);
        registration.redirect_uri = boxed(uri);
        let Some(Request::Visit { .. }) = step(
            &mut client,
            Event::SignIn {
                issued_client_id: None,
                id_token: None,
                subject: None,
                nonce: None,
                port: None,
                registration,
                key: 1,
                generation: 0,
                state: state(),
                verifier: Some(verifier()),
                now: time(1),
            },
        ) else {
            panic!("visit expected")
        };
    }
}

#[test]
fn localhost_redirect_admission_refuses_other_spellings_and_missing_parts() {
    for uri in [
        b"http://localhost.evil:2345/callback".as_slice(),
        b"http://localhost.:2345/callback",
        b"http://LOCALHOST:2345/callback",
        b"http://localhost:0/callback",
        b"http://localhost/callback",
        b"http://localhost:2345",
        b"http://localhost:2345?query=/callback",
        b"http://localhost:65536/callback",
        b"http://localhost:2345@evil/callback",
        b"https://localhost:2345/callback",
    ] {
        assert_eq!(redirect_address(uri), Err(Failure::InvalidRedirect));
        let mut client = Client::new(limits()).expect("limits");
        let mut registration = registration(true, WireFormat::Form);
        registration.redirect_uri = boxed(uri);
        assert_eq!(
            failure(step(
                &mut client,
                Event::SignIn {
                    issued_client_id: None,
                    id_token: None,
                    subject: None,
                    nonce: None,
                    port: None,
                    registration,
                    key: 1,
                    generation: 0,
                    state: state(),
                    verifier: Some(verifier()),
                    now: time(1)
                }
            )),
            Failure::InvalidRedirect
        );
    }
}

#[test]
fn localhost_registration_refuses_every_redirect_uri_change_before_exchange() {
    for uri in [
        b"http://127.0.0.1:2345/callback".as_slice(),
        b"http://localhost.:2345/callback",
        b"http://localhost.evil:2345/callback",
        b"http://LOCALHOST:2345/callback",
        b"http://localhost:2346/callback",
        b"http://localhost:2345/Callback",
        b"http://localhost:02345/callback",
    ] {
        let mut client = Client::new(limits()).expect("limits");
        let mut registration = registration(true, WireFormat::Form);
        registration.redirect_uri = boxed(b"http://localhost:2345/callback");
        let Some(Request::Visit { .. }) = step(
            &mut client,
            Event::SignIn {
                issued_client_id: None,
                id_token: None,
                subject: None,
                nonce: None,
                port: None,
                registration,
                key: 1,
                generation: 0,
                state: state(),
                verifier: Some(verifier()),
                now: time(1),
            },
        ) else {
            panic!("visit expected")
        };
        assert_eq!(
            failure(step(
                &mut client,
                Event::Redirected {
                    client_id: None,
                    uri: boxed(uri),
                    state: state(),
                    code: Some(secret(b"code")),
                    error: None,
                    now: time(2)
                }
            )),
            Failure::InvalidRedirect
        );
    }
}

#[test]
fn public_redirect_must_have_a_numeric_loopback_port() {
    for uri in [
        b"http://127.0.0.1:evil/callback".as_slice(),
        b"http://127.0.0.1:0/callback",
        b"http://127.0.0.1:65536/callback",
        b"http://127.0.0.1:3456.evil/callback",
    ] {
        let mut client = Client::new(limits()).expect("limits");
        let mut registration = registration(true, WireFormat::Form);
        registration.redirect_uri = boxed(uri);
        assert_eq!(
            failure(step(
                &mut client,
                Event::SignIn {
                    issued_client_id: None,
                    id_token: None,
                    subject: None,
                    nonce: None,
                    port: None,
                    registration,
                    key: 1,
                    generation: 0,
                    state: state(),
                    verifier: Some(verifier()),
                    now: time(1)
                }
            )),
            Failure::InvalidRedirect
        );
    }
}

#[test]
fn wrong_state_and_redirect_are_terminal_without_exchange() {
    for wrong_uri in [false, true] {
        let mut client = Client::new(limits()).expect("limits");
        let _visit = step(
            &mut client,
            Event::SignIn {
                issued_client_id: None,
                id_token: None,
                subject: None,
                nonce: None,
                port: None,
                registration: registration(true, WireFormat::Form),
                key: 1,
                generation: 0,
                state: state(),
                verifier: Some(verifier()),
                now: time(1),
            },
        );
        let uri =
            if wrong_uri { boxed(b"http://127.0.0.1:2345/other") } else { boxed(b"http://127.0.0.1:2345/callback") };
        let returned_state = if wrong_uri { state() } else { secret(b"unrelated-state!!") };
        assert_eq!(
            failure(step(
                &mut client,
                Event::Redirected {
                    client_id: None,
                    uri,
                    state: returned_state,
                    code: Some(secret(b"code")),
                    error: None,
                    now: time(2)
                }
            )),
            Failure::InvalidRedirect
        );
        assert!(
            step(
                &mut client,
                Event::Redirected {
                    client_id: None,
                    uri: boxed(b"http://127.0.0.1:2345/callback"),
                    state: state(),
                    code: Some(secret(b"code")),
                    error: None,
                    now: time(3)
                }
            )
            .is_none()
        );
    }
}

#[test]
fn refresh_rotation_preserves_old_token_when_answer_omits_one() {
    let mut client = Client::new(limits()).expect("limits");
    let http = request(step(
        &mut client,
        Event::Refresh {
            registration: registration(true, WireFormat::Json),
            prior: RefreshState {
                key: 3,
                generation: 4,
                refresh_token: secret(b"old"),
                issued_client_id: None,
                granted_scopes: None,
                subject: None,
                id_token: None,
            },
            now: time(1),
        },
    ));
    assert!(bytes::find(&http.body, b"\"refresh_token\":\"old\"").is_some());
    let body = encode_response(
        &TokenResponse {
            access_token: secret(b"access"),
            refresh_token: None,
            expires_in: 30,
            scope: None,
            id_token: None,
            earliest_refresh_at: None,
        },
        &limits().document,
    )
    .expect("response");
    let Some(Request::Tokens { record, .. }) = step(&mut client, answered(http.id, 200, &body, 2)) else {
        panic!("tokens expected")
    };
    assert_eq!(record.refresh_token.as_ref().expect("refresh token").expose(), b"old");
    assert_eq!(record.generation, 5);
}

#[test]
fn rejected_grant_malformed_answer_and_late_answer_are_distinct() {
    for (status, body, expected) in [
        (400, br#"{"error":"invalid_grant"}"#.as_slice(), Failure::Refused),
        (200, br#"{"access_token":1}"#, Failure::Malformed),
    ] {
        let mut client = Client::new(limits()).expect("limits");
        let http = request(step(
            &mut client,
            Event::Refresh {
                registration: registration(true, WireFormat::Form),
                prior: RefreshState {
                    key: 1,
                    generation: 0,
                    refresh_token: secret(b"old"),
                    issued_client_id: None,
                    granted_scopes: None,
                    subject: None,
                    id_token: None,
                },
                now: time(1),
            },
        ));
        assert_eq!(failure(step(&mut client, answered(http.id, status, body, 2))), expected);
    }
    let mut client = Client::new(limits()).expect("limits");
    let http = request(step(
        &mut client,
        Event::Refresh {
            registration: registration(true, WireFormat::Form),
            prior: RefreshState {
                key: 1,
                generation: 0,
                refresh_token: secret(b"old"),
                issued_client_id: None,
                granted_scopes: None,
                subject: None,
                id_token: None,
            },
            now: time(1),
        },
    ));
    assert_eq!(failure(step(&mut client, answered(http.id, 200, &token_body(), 12))), Failure::TimedOut);
}

#[test]
fn proved_unsent_attempt_retries_with_bounded_backoff() {
    let mut client = Client::new(limits()).expect("limits");
    let http = request(step(
        &mut client,
        Event::Refresh {
            registration: registration(true, WireFormat::Form),
            prior: RefreshState {
                key: 1,
                generation: 0,
                refresh_token: secret(b"old"),
                issued_client_id: None,
                granted_scopes: None,
                subject: None,
                id_token: None,
            },
            now: time(1),
        },
    ));
    assert!(
        step(
            &mut client,
            Event::Http(HttpResponse {
                id: http.id,
                status: 0,
                body: boxed(b""),
                retry_after: Duration::ZERO,
                evidence: HttpEvidence::Unsent,
                now: time(2),
                wall: wall(2)
            })
        )
        .is_none()
    );
    assert_eq!(client.next_deadline(), Some(time(4)));
    assert!(step(&mut client, Event::Tick { now: time(3) }).is_none());
    let retry = request(step(&mut client, Event::Tick { now: time(4) }));
    assert_ne!(retry.id, http.id);
    assert_eq!(retry.body, http.body);
    assert!(step(&mut client, answered(http.id, 200, &token_body(), 5)).is_none(), "old answer ignored");
    let Some(Request::Tokens { .. }) = step(&mut client, answered(retry.id, 200, &token_body(), 5)) else {
        panic!("tokens expected")
    };
}

#[test]
fn issuer_https_endpoints_name_a_host() {
    for (uri, admitted) in [
        (b"https://".as_slice(), false),
        (b"https://a@b", false),
        (b"https://@b/x", false),
        (b"https:///x", false),
        (b"https://?q", false),
        (b"https://:443/x", false),
        (b"https://issuer.example", true),
        (b"https://issuer.example:8443/t?x=1", true),
        (b"https://a/b@c", true),
    ] {
        for authorization in [false, true] {
            for public in [false, true] {
                let mut configured = registration(public, WireFormat::Form);
                if authorization {
                    configured.authorization_url = boxed(uri);
                } else {
                    configured.token_endpoint = boxed(uri);
                }
                let mut client = Client::new(limits()).expect("limits");
                let result = step(
                    &mut client,
                    Event::SignIn {
                        issued_client_id: None,
                        id_token: None,
                        subject: None,
                        nonce: None,
                        port: None,
                        registration: configured,
                        key: 1,
                        generation: 0,
                        state: state(),
                        verifier: if public { Some(verifier()) } else { None },
                        now: time(1),
                    },
                );
                if admitted {
                    let Some(Request::Visit { url }) = result else { panic!("expected Visit for {uri:?}") };
                    if authorization {
                        assert!(url.expose().starts_with(uri));
                    }
                } else {
                    assert_eq!(failure(result), Failure::Malformed, "{uri:?}");
                    assert!(client.is_done());
                    assert!(step(&mut client, Event::Tick { now: time(2) }).is_none());
                }
            }
        }
    }
}

#[test]
fn issuer_http_endpoints_are_admitted_only_on_numeric_loopback() {
    for (uri, admitted) in [
        (b"http://127.0.0.1/issuer".as_slice(), true),
        (b"http://127.0.0.0:1/issuer", true),
        (b"http://127.255.255.255:65535/issuer", true),
        (b"http://127.42.1.2/issuer?tenant=one", true),
        (b"http://127.0.0.1", true),
        (b"http://127.0.0.1?tenant=one", true),
        (b"http://[::1]/issuer", true),
        (b"http://[0:0:0:0:0:0:0:1]:80/issuer", true),
        (b"http://[::1]:65535/issuer", true),
        (b"http://126.255.255.255/issuer", false),
        (b"http://128.0.0.0/issuer", false),
        (b"http://0.0.0.0/issuer", false),
        (b"http://192.168.1.1/issuer", false),
        (b"http://[::]/issuer", false),
        (b"http://[::2]/issuer", false),
        (b"http://[::ffff:127.0.0.1]/issuer", false),
        (b"http://issuer.example/issuer", false),
        (b"http://localhost/issuer", false),
        (b"http://127.0.0.1.evil/issuer", false),
        (b"http://127.0.0.1@evil/issuer", false),
        (b"http://user@127.0.0.1/issuer", false),
        (b"http://[::1]@evil/issuer", false),
        (b"http://127.0.0.1:80@evil/issuer", false),
        (b"http://127.0.0.1:0/issuer", false),
        (b"http://127.0.0.1:65536/issuer", false),
        (b"http://127.0.0.1:/issuer", false),
        (b"http://127.0.0.1:evil/issuer", false),
        (b"http://[::1]:0/issuer", false),
        (b"http://[::1]:65536/issuer", false),
        (b"http://[::1]evil/issuer", false),
        (b"http://[::1/issuer", false),
        (b"http://127.1/issuer", false),
        (b"http://2130706433/issuer", false),
        (b"http://0x7f000001/issuer", false),
        (b"http://127.00.0.1/issuer", false),
        (b"ftp://127.0.0.1/issuer", false),
    ] {
        for authorization in [false, true] {
            for public in [false, true] {
                let mut configured = registration(public, WireFormat::Form);
                if authorization {
                    configured.authorization_url = boxed(uri);
                } else {
                    configured.token_endpoint = boxed(uri);
                }
                let expected_token = configured.token_endpoint.clone();
                let mut client = Client::new(limits()).expect("limits");
                let result = step(
                    &mut client,
                    Event::SignIn {
                        issued_client_id: None,
                        id_token: None,
                        subject: None,
                        nonce: None,
                        port: None,
                        registration: configured,
                        key: 1,
                        generation: 0,
                        state: state(),
                        verifier: if public { Some(verifier()) } else { None },
                        now: time(1),
                    },
                );
                if admitted {
                    match result {
                        Some(Request::Visit { url }) => {
                            if authorization {
                                assert!(url.expose().starts_with(uri));
                            }
                        }
                        other => panic!("expected Visit for {uri:?}, got {}", other.is_some()),
                    }
                    let http = request(step(
                        &mut client,
                        Event::Redirected {
                            client_id: None,
                            uri: if public {
                                boxed(b"http://127.0.0.1:2345/callback")
                            } else {
                                boxed(b"https://web.example/callback")
                            },
                            state: state(),
                            code: Some(secret(b"grant")),
                            error: None,
                            now: time(2),
                        },
                    ));
                    assert_eq!(http.endpoint, expected_token, "the endpoint binding survives the redirect");
                } else {
                    assert_eq!(failure(result), Failure::Malformed, "{uri:?}");
                    assert!(client.is_done());
                    assert!(step(&mut client, Event::Tick { now: time(2) }).is_none());
                }
            }
        }
    }
}

#[test]
fn refresh_keeps_its_admitted_http_loopback_endpoint() {
    for (uri, admitted) in [
        (b"http://127.1.2.3:9876/token".as_slice(), true),
        (b"http://[::1]:9876/token", true),
        (b"http://issuer.example/token", false),
    ] {
        let mut configured = registration(true, WireFormat::Form);
        configured.token_endpoint = boxed(uri);
        let prior = RefreshState {
            key: 7,
            generation: 1,
            refresh_token: secret(b"refresh-old"),
            issued_client_id: None,
            granted_scopes: None,
            subject: None,
            id_token: None,
        };
        let mut client = Client::new(limits()).expect("limits");
        let result = step(&mut client, Event::Refresh { registration: configured, prior, now: time(1) });
        if admitted {
            assert_eq!(request(result).endpoint.as_ref(), uri);
        } else {
            assert_eq!(failure(result), Failure::Malformed);
        }
    }
}

mod terms;

mod issued;

mod openid;
mod revoke;

#[test]
fn confidential_redirects_admit_https_and_fixed_loopback() {
    for uri in [
        b"http://127.0.0.1:8080/cb".as_slice(),
        b"http://[::1]:8080/cb",
        b"http://localhost:8080/cb",
        b"https://app.example.com/cb",
    ] {
        let mut registration = registration(false, WireFormat::Form);
        registration.redirect_uri = boxed(uri);
        assert_eq!(validate_registration(&registration, &limits(), true), Ok(()), "{uri:?}");
    }
}

#[test]
fn invalid_confidential_redirects_are_refused_during_registration_validation() {
    skein_rows::check(
        [
            ("confidential_redirects_refuse_loopback_neighbours_and_plain_http: 1", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://localhost.:8080/cb".as_slice());
                value
            }),
            ("confidential_redirects_refuse_loopback_neighbours_and_plain_http: 2", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://localhost.evil:8080/cb");
                value
            }),
            ("confidential_redirects_refuse_loopback_neighbours_and_plain_http: 3", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://localhost:0/cb");
                value
            }),
            ("confidential_redirects_refuse_loopback_neighbours_and_plain_http: 4", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://127.0.0.1/cb");
                value
            }),
            ("confidential_redirects_refuse_loopback_neighbours_and_plain_http: 5", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://127.0.0.1:8080");
                value
            }),
            ("confidential_redirects_refuse_loopback_neighbours_and_plain_http: 6", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://app.example.com/cb");
                value
            }),
            ("confidential_redirects_never_vary: 1", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"http://127.0.0.1:8080/cb".as_slice());
                value.terms = Some(terms::terms(false, true));
                value
            }),
            ("confidential_redirects_never_vary: 2", {
                let mut value = registration(false, WireFormat::Form);
                value.redirect_uri = boxed(b"https://app.example.com/cb");
                value.terms = Some(terms::terms(false, true));
                value
            }),
        ],
        |registration| assert_eq!(validate_registration(&registration, &limits(), true), Err(Failure::InvalidRedirect)),
    );
}

#[test]
fn a_sign_in_without_refresh_completes_and_round_trips() {
    let mut client = Client::new(limits()).expect("client");
    let mut sign_in = openid::sign_in(registration(true, WireFormat::Form));
    if let Event::SignIn { nonce, .. } = &mut sign_in {
        *nonce = None;
    }
    let _visit = step(&mut client, sign_in);
    let post = request(step(
        &mut client,
        Event::Redirected {
            client_id: None,
            uri: boxed(b"http://127.0.0.1:2345/callback"),
            state: state(),
            code: Some(secret(b"code")),
            error: None,
            now: time(2),
        },
    ));
    let body = br#"{"access_token":"access","expires_in":30,"scope":"read","earliest_refresh_at":20}"#;
    let Some(Request::Tokens { record, .. }) = step(&mut client, answered(post.id, 200, body, 3)) else {
        panic!("tokens expected")
    };
    assert_eq!((record.key, record.generation, record.expires_at), (7, 1, wall(33)));
    assert!(record.access_token.expose() == b"access");
    assert!(record.refresh_state().is_none());
    assert!(record.subject.is_none() && record.id_token.is_none());
    assert_eq!(record.granted_scopes.as_deref(), Some(b"read".as_slice()));
    assert_eq!(record.earliest_refresh_at, Some(wall(20)));
    let encoded = encode_record(&record, &limits().document).expect("encode");
    assert_eq!(decode_record(&encoded, &limits().document).expect("decode"), record);
}

mod access_only_openid;

mod admits;

#[test]
fn awaits_state_checks_the_waiting_phase_state_and_deadline() {
    skein_rows::check(
        [
            ("waiting", (0_u8, true)),
            ("another state", (1, false)),
            ("deadline", (2, false)),
            ("before visit", (3, false)),
            ("redirect taken", (4, false)),
        ],
        |(kind, expected)| {
            let mut client = Client::new(limits()).expect("client");
            let registration =
                if kind == 3 { openid::registration_for_keys(true) } else { registration(true, WireFormat::Form) };
            let mut event = openid::sign_in(registration);
            if kind != 3
                && let Event::SignIn { nonce, .. } = &mut event
            {
                *nonce = None;
            }
            drop(step(&mut client, event));
            if kind == 4 {
                drop(step(
                    &mut client,
                    Event::Redirected {
                        uri: boxed(b"http://127.0.0.1:2345/callback"),
                        state: state(),
                        code: Some(secret(b"code")),
                        error: None,
                        client_id: None,
                        now: time(2),
                    },
                ));
            }
            let candidate = if kind == 1 { b"1123456789abcdef" } else { b"0123456789abcdef" };
            let now = if kind == 2 { time(121) } else { time(2) };
            let deadline = client.next_deadline();
            assert_eq!(client.awaits_state(candidate, now), expected);
            assert_eq!(client.next_deadline(), deadline, "the question changes nothing");
        },
    );
}

#[test]
fn permitted_client_limits_pass_validation_including_large_bounds() {
    skein_rows::check(
        [
            ("valid_accepts_test_limits", limits()),
            (
                "valid_ignores_heap_key_strings",
                ClientLimits {
                    keys: u32::MAX,
                    document: Limits { string_bytes: u32::MAX, ..limits().document },
                    ..limits()
                },
            ),
        ],
        |bounds| assert!(client_valid(&bounds)),
    );
}

fn refuses_limits(bounds: ClientLimits) {
    assert!(!client_valid(&bounds));
    assert_eq!(Client::new(bounds).err(), Some(Failure::Limit));
}

#[test]
fn invalid_client_limits_refuse_validation_and_client_construction() {
    skein_rows::check(
        [
            ("valid_refuses_zero_attempts", ClientLimits { max_attempts: 0, ..limits() }),
            ("valid_refuses_zero_request_time", ClientLimits { request_time: Duration::ZERO, ..limits() }),
            ("valid_refuses_zero_sign_in_time", ClientLimits { sign_in_time: Duration::ZERO, ..limits() }),
            ("valid_refuses_modulus_outside_supported_range", ClientLimits { modulus_bits: 2047, ..limits() }),
            (
                "valid_refuses_document_length_past_tokenizer",
                ClientLimits { document: Limits { document_bytes: u32::MAX, ..limits().document }, ..limits() },
            ),
            ("valid_refuses_parameter_slots_past_u32", ClientLimits { parameter_bytes: u32::MAX, ..limits() }),
            (
                "valid_refuses_token_bytes_past_record_schema",
                ClientLimits { document: Limits { token_bytes: u32::MAX, ..limits().document }, ..limits() },
            ),
            (
                "valid_refuses_id_token_bytes_past_record_schema",
                ClientLimits { document: Limits { id_token_bytes: u32::MAX, ..limits().document }, ..limits() },
            ),
            (
                "valid_refuses_issued_client_bytes_past_record_schema",
                ClientLimits { document: Limits { issued_client_bytes: u32::MAX, ..limits().document }, ..limits() },
            ),
            (
                "valid_refuses_granted_bytes_past_record_schema",
                ClientLimits { document: Limits { granted_bytes: u32::MAX, ..limits().document }, ..limits() },
            ),
            (
                "valid_refuses_subject_bytes_past_record_schema",
                ClientLimits { document: Limits { subject_bytes: u32::MAX, ..limits().document }, ..limits() },
            ),
        ],
        refuses_limits,
    );
}

#[test]
fn refused_and_ignored_events_preserve_the_exchange() {
    for trace in [
        "keys:http sign-in:limit refresh:limit revoke:revoked wrong-id:silent redirect:silent key-set:visit",
        "refresh:http sign-in:limit refresh:limit revoke:revoked redirect:silent tokens:tokens",
        "refresh:http rate-limit:silent sign-in:limit refresh:limit revoke:revoked wrong-id:silent redirect:silent retry:http tokens:tokens",
        "redirect:silent cancel:silent refresh:http tokens:tokens",
        "refresh:http tokens:tokens redirect:silent cancel:silent refresh:limit",
    ] {
        let mut client = Client::new(limits()).expect("client");
        let mut id: u64 = 0;
        for cell in trace.split_whitespace() {
            let (input, expected) = cell.split_once(':').expect("input:output");
            let event = match input {
                "keys" => openid::sign_in(openid::registration_for_keys(true)),
                "sign-in" => openid::sign_in(registration(true, WireFormat::Form)),
                "refresh" => refresh(registration(true, WireFormat::Form), None),
                "revoke" => revoke::begin(registration(true, WireFormat::Form), false),
                "wrong-id" => answered(id.saturating_add(1), 200, b"", 3),
                "redirect" => issued::redirect(None),
                "cancel" => Event::Cancel,
                "retry" => Event::Tick { now: time(4) },
                "rate-limit" => answered(id, 429, b"", 2),
                "key-set" => answered(id, 200, &openid::one_key(), 5),
                "tokens" => answered(id, 200, &token_body(), 5),
                _ => unreachable!("trace input"),
            };
            let deadline = client.next_deadline();
            let result = match step(&mut client, event) {
                Some(Request::Http(request)) => {
                    id = request.id;
                    "http"
                }
                Some(Request::Visit { .. }) => "visit",
                Some(Request::Tokens { .. }) => "tokens",
                Some(Request::Failed { failure: Failure::Limit }) => "limit",
                Some(Request::Revoked { confirmed: false }) => "revoked",
                None => "silent",
                other => panic!("unexpected trace output: {other:?}"),
            };
            assert_eq!(result, expected, "{trace:?}: {input}");
            if ["limit", "revoked", "silent"].contains(&expected) && input != "rate-limit" {
                assert_eq!(client.next_deadline(), deadline, "{trace:?}: {input}");
            }
        }
    }
}
