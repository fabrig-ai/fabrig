//! Tests of the trusted OAuth output's consumer. Signature verification stays
//! in skein-oauth; these inputs stand immediately after its Tokens terminal.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests construct and inspect protocol fixtures"
)]
use super::*;
use crate::tests::{fixtures, sign_in as world};
fn token(payload: &[u8]) -> Box<[u8]> {
    let encoded = base64::encoded_len(Alphabet::Url, u32::try_from(payload.len()).unwrap()).unwrap();
    let mut out = Writer::new(usize::try_from(encoded).unwrap().checked_add(4).unwrap());
    out.put(b"e.").unwrap();
    base64::encode(Alphabet::Url, payload, &mut out).unwrap();
    out.put(b".s").unwrap();
    out.finish()
}
fn record(payload: &[u8], subject: &[u8]) -> oauth::SavedToken {
    oauth::SavedToken {
        key: 1,
        generation: 1,
        access_token: Secret::new(b"throwaway-access".as_slice().into()),
        refresh_token: None,
        metadata: None,
        expires_at: Wall::from_nanos(3_600_000_000_000),
        issued_client_id: None,
        granted_scopes: None,
        subject: Some(subject.into()),
        id_token: Some(Secret::new(token(payload))),
        earliest_refresh_at: None,
    }
}
fn finish(
    service: &mut crate::Service,
    issuer: &mut skein_fake_oauth::Issuer,
    payload: &[u8],
    subject: &[u8],
    then: &[u8],
) -> fixtures::http::Peer {
    let (url, cookie) = world::start(service, issuer, then);
    let state = query(split(&url).1, b"state").unwrap();
    let target = format!("/sign-in/google/return?state={}&code=c", core::str::from_utf8(&state).unwrap());
    let mut peer = world::peer(service, target.as_bytes(), Some(&cookie));
    world::pump(service, &mut peer);
    let engine = service.engine.as_mut().unwrap();
    let _request = engine.clients.google_http().unwrap();
    let google = engine.clients.google.as_mut().unwrap();
    let exchange = *google.pending.first().expect("pending sign-in").0;
    let pending = google.pending.get_mut(&exchange).unwrap();
    pending.out = Queue::uncapped();
    pending.out.push(oauth::Request::Tokens {
        record: record(payload, subject),
        missing: Box::new([]),
        profile: oauth::ProfileClaims::default(),
    });
    world::pump(service, &mut peer);
    peer
}
#[test]
fn the_claims_reader_reads_only_top_level_fields_and_both_true_spellings() {
    for verified in ["true", "\"true\""] {
        let claims = claims_from(&token(format!("{{\"email\":\"sam@example.com\",\"email_verified\":{verified},\"name\":\"Sam\",\"nested\":{{\"email\":\"other@example.com\"}}}}").as_bytes()), &limits::client(fixtures::engine().limits.sign_in).document).unwrap();
        assert_eq!(claims.email.as_deref(), Some(b"sam@example.com".as_slice()));
        assert!(claims.verified);
        assert_eq!(claims.name.as_deref(), Some(b"Sam".as_slice()));
    }
}
#[test]
fn then_is_a_single_origin_path_after_one_decode() {
    for input in [
        b"".as_slice(),
        b"then=https%3A%2F%2Fevil.example",
        b"then=%2F%2Fevil.example",
        b"then=%2Fback%5Cslash",
        b"then=%2Fline%0D%0A",
        b"then=%2Fbad%GG",
        b"then=%2Fa&then=%2Fb",
    ] {
        assert_eq!(then(input).as_ref(), b"/");
    }
    assert_eq!(then(b"then=%2Fprojects%3Ftab%3Dmine%26sort%3Dnew").as_ref(), b"/projects?tab=mine&sort=new");
    let at = format!("then=%2F{}", "a".repeat(1023));
    assert_eq!(then(at.as_bytes()).len(), 1024);
    let past = format!("{at}a");
    assert_eq!(then(past.as_bytes()).as_ref(), b"/");
}
#[test]
fn endings_escape_addresses_errors_and_return_attributes() {
    let address = b"a<&\"@example.com";
    let then = b"/a?x=\"&y=<";
    let sheet = Some(b"/bundle/a\"<&/files/pages.css".as_slice());
    for body in [
        pages::not_member(sheet, address, then),
        pages::another(sheet, address, then),
        pages::unverified(sheet, Some(address), then),
        pages::cancelled(sheet, b"<script>", then),
        pages::expired(sheet, then),
        pages::unreachable(sheet, then),
        pages::busy(sheet, then),
    ] {
        let text = core::str::from_utf8(&body).unwrap();
        assert!(!text.contains("<script>"));
        assert!(text.contains("<link rel=\"stylesheet\" href=\"/bundle/a&quot;&lt;&amp;/files/pages.css\">"));
        assert!(!text.contains("a<&\"@"));
        assert!(text.contains("then=%2Fa%3Fx%3D%22%26y%3D%3C"));
    }
}
#[test]
fn endings_draw_the_column_and_link_the_bundles_sheet() {
    for sheet in [Some(b"/bundle/d1/files/pages.css".as_slice()), None] {
        for (body, choose, back) in [
            (pages::not_member(sheet, b"member@example.com", b"/sign-in"), true, true),
            (pages::another(sheet, b"member@example.com", b"/sign-in"), true, true),
            (pages::unverified(sheet, Some(b"member@example.com"), b"/sign-in"), true, true),
            (pages::unverified(sheet, None, b"/sign-in"), true, true),
            (pages::cancelled(sheet, b"access_denied", b"/sign-in"), false, true),
            (pages::expired(sheet, b"/sign-in"), false, true),
            (pages::unreachable(sheet, b"/sign-in"), false, true),
            (pages::busy(sheet, b"/sign-in"), false, false),
        ] {
            let text = core::str::from_utf8(&body).unwrap();
            assert!(!text.contains("&#39;") && !text.contains('\''), "typographic apostrophes: {text}");
            assert!(text.starts_with("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\"><title>temper</title>"));
            if sheet.is_some() {
                assert!(text.contains("<link rel=\"stylesheet\" href=\"/bundle/d1/files/pages.css\">"));
            } else {
                assert!(!text.contains("<link"));
            }
            assert!(text.contains("<header><a href=\"/\">temper</a></header><main><h1>"));
            let label = if choose { "Use another Google account" } else { "Continue with Google" };
            let account = if choose { "&amp;account=choose" } else { "" };
            assert!(text.contains(&format!(
                "<a class=\"primary\" href=\"/sign-in/google?then=%2Fsign-in{account}\">{label}</a>"
            )));
            assert_eq!(text.contains("<p class=\"quiet\"><a href=\"/sign-in\">Back to sign in</a></p>"), back);
            assert!(text.ends_with("</main></body></html>"));
        }
    }
}
#[test]
fn a_members_checked_identity_binds_and_mints_twice_then_refuses_another_subject() {
    let (mut service, mut issuer) = world::world(2);
    for _ in 0_u32..2 {
        let peer = finish(
            &mut service,
            &mut issuer,
            br#"{"email":"SAM@example.com","email_verified":true,"name":"Sam"}"#,
            b"first-sub",
            b"/sign-in/google?then=%2Fprojects%2F1",
        );
        assert!(
            peer.received().starts_with(b"HTTP/1.1 303"),
            "status: {}; events: {}; pending: {}; ready: {}",
            core::str::from_utf8(peer.received()).unwrap().lines().next().unwrap_or("no response"),
            service.engine.as_ref().unwrap().events.len(),
            service.engine.as_ref().unwrap().clients.google.as_ref().unwrap().pending.len(),
            service.engine.as_ref().unwrap().domain.ready()
        );
        assert_eq!(world::header(&peer, "location"), b"/projects/1");
        let text = core::str::from_utf8(peer.received()).unwrap();
        assert!(text.contains("jig-sign-in="));
        assert!(text.contains("SameSite=Strict"));
        assert!(text.contains("; Secure"));
        assert!(text.contains("temper-sign-in=; Path=/sign-in/google; HttpOnly; SameSite=Lax; Max-Age=0; Secure"));
    }
    let peer = finish(
        &mut service,
        &mut issuer,
        br#"{"email":"sam@example.com","email_verified":true}"#,
        b"another-sub",
        b"/sign-in/google",
    );
    world::ending(&peer, b"HTTP/1.1 403", b"signs in here with another Google account");
    let rows = fixtures::load(&mut service.engine.as_mut().unwrap().store, temper_engine_domain::Range::Forge);
    assert!(rows.iter().any(|row| match row {
        temper_engine_domain::Record::Forge { row } => {
            if let forge::Stored::Binding { address, subject } = row.as_ref() {
                address.as_ref() == b"sam@example.com" && subject.as_ref() == b"first-sub"
            } else {
                false
            }
        }
        temper_engine_domain::Record::Core(_) => false,
    }));
}
#[test]
fn checked_nonmembers_and_unverified_addresses_mint_nothing() {
    let overlong = format!(r#"{{"email":"{}","email_verified":true}}"#, "a".repeat(255));
    for (payload, ending) in [
        (
            br#"{"email":"other@example.com","email_verified":true}"#.as_slice(),
            "other@example.com can’t sign in here".as_bytes(),
        ),
        (br#"{"email":"sam@example.com","email_verified":false}"#, "Google hasn’t verified sam@example.com".as_bytes()),
        (br#"{"name":"No address","email_verified":true}"#, "Google hasn’t verified this account’s address".as_bytes()),
        (overlong.as_bytes(), "Google hasn’t verified this account’s address".as_bytes()),
    ] {
        let (mut service, mut issuer) = world::world(2);
        service.adopt_bundle(fixtures::bundle::bundle());
        let peer = finish(&mut service, &mut issuer, payload, b"first-sub", b"/sign-in/google?then=%2F%2Fevil.example");
        world::ending(&peer, b"HTTP/1.1 403", ending);
        assert!(
            core::str::from_utf8(peer.received())
                .unwrap()
                .contains("<link rel=\"stylesheet\" href=\"/bundle/0123456789abcdef/files/pages.css\">")
        );
        assert!(!core::str::from_utf8(peer.received()).unwrap().contains("jig-sign-in="));
    }
}
#[test]
fn mint_refused_at_the_sign_in_capacity_answers_busy() {
    let (mut service, mut issuer) = world::world(2);
    for _ in 0_u32..2 {
        let peer = finish(
            &mut service,
            &mut issuer,
            br#"{"email":"sam@example.com","email_verified":true}"#,
            b"first-sub",
            b"/sign-in/google",
        );
        assert!(peer.received().starts_with(b"HTTP/1.1 303"));
    }
    let peer = finish(
        &mut service,
        &mut issuer,
        br#"{"email":"sam@example.com","email_verified":true}"#,
        b"first-sub",
        b"/sign-in/google",
    );
    world::ending(&peer, b"HTTP/1.1 503", "temper couldn’t sign you in just now".as_bytes());
    assert!(!core::str::from_utf8(peer.received()).unwrap().contains("jig-sign-in="));
}
#[test]
fn the_recovery_link_uses_the_first_google_owner_without_a_link_owner() {
    let mut config = fixtures::engine();
    let sign_in = world::configured();
    config.jig.owners[0].identity = people::IdentityKey { provider: 3, subject: b"sam@example.com".as_slice().into() };
    let clients = Clients::new(
        &jig_protocol_config::Directories::working(),
        &config.jig,
        &config.clients,
        &fixtures::seeds(),
        &config.limits.root.journal,
        Some(&sign_in),
        None,
        &[],
    )
    .unwrap();
    let identity = clients.initial.as_ref().unwrap().link.as_ref().unwrap();
    assert_eq!(identity.key.provider, 3);
    assert_eq!(identity.key.subject.as_ref(), b"sam@example.com");
    assert_eq!(identity.login.as_ref(), b"sam");
    assert_eq!(identity.name.as_ref(), b"sam");
}
#[test]
fn an_unconfigured_origin_fails_before_any_socket_is_opened() {
    let (mut service, _) = world::world(2);
    let google = service.engine.as_mut().unwrap().clients.google.as_mut().unwrap();
    google.scripted = false;
    google
        .endpoints
        .push(fetch::Endpoint {
            origin: b"http://127.0.0.1:31998".as_slice().into(),
            address: ([127, 0, 0, 1], 31998).into(),
            transport: fetch::Transport::Plaintext,
        })
        .unwrap();
    let mut peer = world::peer(&mut service, b"/sign-in/google", None);
    world::pump(&mut service, &mut peer);
    assert!(peer.received().starts_with(b"HTTP/1.1 502"));
}
#[test]
fn stopping_drops_pending_sign_ins_and_unsent_fetch_records() {
    let (mut service, _) = world::world(2);
    let mut peer = world::peer(&mut service, b"/sign-in/google", None);
    world::pump(&mut service, &mut peer);
    let clients = &mut service.engine.as_mut().unwrap().clients;
    assert!(!clients.google.as_ref().unwrap().http.is_empty());
    clients.down(kit::Event::Stop);
    clients.google_advance(&mut Queue::with_capacity(8));
    assert!(clients.google.as_ref().unwrap().closed());
}
#[test]
fn the_full_pending_table_stays_under_its_measured_heap_ceiling() {
    // Full pending sign-in table: a quarter above the measured 227,889-byte peak,
    // rounded up to two significant digits. Move by hand only in the commit of
    // a real change that passes this ceiling.
    const PENDING_HEAP_CEILING: u64 = 290_000;
    let settings = world::configured();
    let limits = fixtures::engine().limits.sign_in;
    let seeds = fixtures::seeds();
    let meter = skein_world::domain::heap::Meter::new();
    meter.start();
    let mut google = Google::new(&settings, limits, &seeds, 1024);
    for number in 0..limits.pending {
        let redirect: Box<[u8]> = b"https://temper.example.com/sign-in/google/return".as_slice().into();
        let registration = google.registration(redirect.clone(), false);
        let state = random(&mut google.rng);
        let nonce = random(&mut google.rng);
        let verifier = random(&mut google.rng);
        let mut pending = Pending {
            tag: Box::new([0; 16]),
            then: core::iter::repeat_n(b'/', 1024).collect(),
            deadline: Time::from_nanos(600_000_000_000),
            call: None,
            redirect,
            client: Some(oauth::Client::new(limits::client(limits)).unwrap()),
            out: Queue::uncapped(),
            phase: Phase::Starting,
        };
        pending.client.as_mut().unwrap().step(
            oauth::Event::SignIn {
                registration,
                key: 1,
                generation: 1,
                state: Secret::new(state),
                verifier: Some(Secret::new(verifier)),
                nonce: Some(Secret::new(nonce)),
                port: None,
                issued_client_id: None,
                id_token: None,
                subject: None,
                now: Time::ZERO,
            },
            &mut pending.out,
        );
        google.keep_pending(Token::new(u64::from(number)), pending);
    }
    let measured = meter.end();
    meter.check(measured, PENDING_HEAP_CEILING, &"Google pending sign-ins");
    assert_eq!(google.pending.len(), limits.pending);
}
