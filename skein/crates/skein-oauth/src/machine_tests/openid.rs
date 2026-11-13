use super::*;
use skein_json::writer::Encoder;
use skein_lib::{Secret, Writer, base64, usize_of};

const MIN_BITS: u32 = 2048; // oauth/README.md, section 2.2.
const MAX_BITS: u32 = 4096; // limits().modulus_bits.
const KEY_CAP: u32 = 4; // limits().keys.
const METADATA: &[u8] = br#"{"issuer":"https://issuer.example","jwks_uri":"https://issuer.example/keys"}"#;

pub(super) fn registration_for_keys(direct: bool) -> Registration {
    let mut registration = registration(true, WireFormat::Form);
    registration.scope = boxed(b"openid read");
    let mut additions = terms::terms(true, false);
    if let Some(openid) = &mut additions.openid {
        openid.key_set = if direct { Some(boxed(b"https://issuer.example/keys")) } else { None };
    }
    registration.terms = Some(additions);
    registration
}
pub(super) fn sign_in(registration: Registration) -> Event {
    Event::SignIn {
        registration,
        key: 7,
        generation: 0,
        state: state(),
        verifier: Some(verifier()),
        nonce: Some(secret(b"0123456789abcdef")),
        port: None,
        issued_client_id: None,
        id_token: None,
        subject: None,
        now: time(1),
    }
}
fn fetching(direct: bool) -> (Client, HttpRequest) {
    let mut client = Client::new(limits()).expect("client");
    let get = request(step(&mut client, sign_in(registration_for_keys(direct))));
    (client, get)
}
fn encoded(raw: &[u8]) -> Box<[u8]> {
    let length =
        base64::encoded_len(base64::Alphabet::Url, u32::try_from(raw.len()).expect("fixture")).expect("fixture");
    let mut out = Writer::new(usize_of(length));
    base64::encode(base64::Alphabet::Url, raw, &mut out).expect("fixture");
    out.finish()
}
fn modulus(bits: u32, leading_zero: bool) -> Box<[u8]> {
    const MAX_BYTES: usize = 514; // One byte beyond 4096 bits, plus a leading zero.
    let count = usize_of(bits.div_ceil(8));
    let mut raw = [0xff; MAX_BYTES];
    let offset = usize::from(leading_zero);
    if leading_zero {
        *raw.first_mut().expect("fixture") = 0;
    }
    *raw.get_mut(offset).expect("fixture") = 0xff_u8.checked_shr((8_u32.wrapping_sub(bits % 8)) % 8).expect("fixture");
    encoded(raw.get(..count.checked_add(offset).expect("fixture")).expect("fixture"))
}
fn write_keys(out: &mut Encoder, n: &[u8], kty: &[u8], use_: Option<&[u8]>, count: u32) {
    out.object_start();
    out.key(b"keys");
    out.array_start();
    for _ in 0..count {
        out.object_start();
        for (name, value) in [(b"kty".as_slice(), kty), (b"n", n), (b"e", b"AQAB"), (b"kid", b"test")] {
            out.key(name);
            out.string(value);
        }
        if let Some(value) = use_ {
            out.key(b"use");
            out.string(value);
        }
        out.object_end();
    }
    out.array_end();
    out.object_end();
}
fn key_set(bits: u32, leading_zero: bool, kty: &[u8], use_: Option<&[u8]>, count: u32) -> Box<[u8]> {
    let n = modulus(bits, leading_zero);
    let bounds = limits().document.writer_limits();
    let mut measure = Encoder::measure(&bounds);
    write_keys(&mut measure, &n, kty, use_, count);
    let mut out = Encoder::write(measure.measured().expect("fixture"), &bounds);
    write_keys(&mut out, &n, kty, use_, count);
    out.finish()
}
pub(super) fn one_key() -> Box<[u8]> {
    key_set(MIN_BITS, false, b"RSA", Some(b"sig"), 1)
}

#[test]
fn an_openid_sign_in_fetches_its_keys_before_the_visit() {
    let (mut client, get) = fetching(true);
    let endpoint = get.endpoint.clone();
    let visit = is_visit(step(&mut client, answered(get.id, 200, &one_key(), 2)));
    assert_eq!((endpoint.as_ref(), visit), (b"https://issuer.example/keys".as_slice(), true));
}
#[derive(Debug, PartialEq, Eq)]
struct MetadataVisit {
    metadata: Box<[u8]>,
    keys: Box<[u8]>,
    visited: bool,
}
#[test]
fn an_openid_sign_in_fetches_metadata_then_its_named_key_set_before_the_visit() {
    let (mut client, metadata) = fetching(false);
    let keys = request(step(&mut client, answered(metadata.id, 200, METADATA, 2)));
    let visited = is_visit(step(&mut client, answered(keys.id, 200, &one_key(), 3)));
    let got = MetadataVisit { metadata: metadata.endpoint, keys: keys.endpoint, visited };
    assert_eq!(
        got,
        MetadataVisit {
            metadata: boxed(b"https://issuer.example/.well-known/openid-configuration"),
            keys: boxed(b"https://issuer.example/keys"),
            visited: true,
        }
    );
}
#[test]
fn a_get_carries_no_body_or_content_type() {
    let (_, get) = fetching(true);
    assert_eq!(get_shape(&get), GetShape { method: Method::Get, content_type: 0, body: 0 });
}
#[test]
fn a_non_openid_sign_in_visits_at_once() {
    let mut client = Client::new(limits()).expect("client");
    let mut event = sign_in(registration(true, WireFormat::Form));
    if let Event::SignIn { nonce, .. } = &mut event {
        *nonce = None;
    }
    let visit = is_visit(step(&mut client, event));
    assert!(visit);
}
#[derive(Debug, PartialEq, Eq)]
struct KeyShape {
    count: usize,
    modulus_bytes: usize,
}
struct KeyRow {
    name: &'static str,
    bits: u32,
    zero: bool,
    kty: &'static [u8],
    use_: Option<&'static [u8]>,
    count: u32,
    expected: Option<KeyShape>,
}
fn key_shape(bits: u32, count: u32) -> KeyShape {
    KeyShape { count: usize_of(count), modulus_bytes: usize_of(bits / 8) }
}
fn key_row(name: &'static str) -> KeyRow {
    KeyRow {
        name,
        bits: MIN_BITS,
        zero: false,
        kty: b"RSA",
        use_: None,
        count: 1,
        expected: Some(key_shape(MIN_BITS, 1)),
    }
}
#[test]
fn the_key_set_keeps_only_keys_that_fit() {
    for row in [
        KeyRow { kty: b"EC", use_: Some(b"sig"), expected: None, ..key_row("EC") },
        KeyRow { use_: Some(b"enc"), expected: None, ..key_row("enc") },
        key_row("no use"),
        KeyRow { bits: MIN_BITS - 1, expected: None, ..key_row("short") },
        key_row("minimum"),
        KeyRow { bits: MAX_BITS, expected: Some(key_shape(MAX_BITS, 1)), ..key_row("maximum") },
        KeyRow { bits: MAX_BITS + 1, expected: None, ..key_row("long") },
        KeyRow { zero: true, ..key_row("leading zero") },
        KeyRow { count: KEY_CAP, expected: Some(key_shape(MIN_BITS, KEY_CAP)), ..key_row("key cap") },
        KeyRow { count: KEY_CAP + 1, expected: None, ..key_row("past key cap") },
    ] {
        let got = match keys::fitting_keys(&key_set(row.bits, row.zero, row.kty, row.use_, row.count), &limits()) {
            Ok(keys) => {
                Some(KeyShape { count: keys.len(), modulus_bytes: keys.first().expect("kept key").modulus.len() })
            }
            Err(_) => None,
        };
        assert_eq!(got, row.expected, "{}", row.name);
    }
}

struct FetchRow {
    name: &'static str,
    direct: bool,
    body: Box<[u8]>,
    status: u16,
    now: u64,
    attempts: u32,
    expected: Failure,
}
fn fetch_row(name: &'static str) -> FetchRow {
    FetchRow {
        name,
        direct: true,
        body: Box::default(),
        status: 200,
        now: 2,
        attempts: limits().max_attempts,
        expected: Failure::Unverified { check: Check::Keys },
    }
}
fn failed_fetch(row: FetchRow) -> Option<Request> {
    let mut bounds = limits();
    bounds.max_attempts = row.attempts;
    let mut client = Client::new(bounds).expect("client");
    let get = request(step(&mut client, sign_in(registration_for_keys(row.direct))));
    step(&mut client, answered(get.id, row.status, &row.body, row.now))
}
fn failed_fetch_rows() -> [FetchRow; 11] {
    let metadata_cap = usize_of(limits().metadata_bytes);
    let set_cap = usize_of(limits().key_set_bytes);
    let mut metadata_body = bytes::zeroed(metadata_cap.checked_add(1).expect("small fixture"));
    let mut set_body = bytes::zeroed(set_cap.checked_add(1).expect("small fixture"));
    for body in [&mut metadata_body, &mut set_body] {
        for byte in body.iter_mut() {
            *byte = b' ';
        }
    }
    [
        FetchRow { direct: false, body: boxed(b"bad"), ..fetch_row("unreadable metadata") },
        FetchRow { body: boxed(b"bad"), ..fetch_row("unreadable keys") },
        FetchRow { direct: false, body: metadata_body, ..fetch_row("metadata cap") },
        FetchRow { body: set_body, ..fetch_row("keys cap") },
        FetchRow {
            direct: false,
            body: boxed(br#"{"issuer":"https://other.example","jwks_uri":"https://issuer.example/keys"}"#),
            ..fetch_row("issuer")
        },
        FetchRow {
            direct: false,
            body: boxed(br#"{"issuer":"https://issuer.example","jwks_uri":"http://issuer.example/keys"}"#),
            ..fetch_row("http off loopback")
        },
        FetchRow {
            direct: false,
            body: boxed(br#"{"issuer":"https://issuer.example","jwks_uri":"https://a@b"}"#),
            ..fetch_row("userinfo in key set URL")
        },
        FetchRow { body: boxed(br#"{"keys":[]}"#), ..fetch_row("no fitting key") },
        FetchRow { status: 404, ..fetch_row("not found") },
        FetchRow { status: 503, attempts: 1, ..fetch_row("budget") },
        FetchRow { body: one_key(), now: 121, ..fetch_row("deadline") },
    ]
}
#[test]
fn each_keys_failure_is_unverified_before_any_visit() {
    for row in failed_fetch_rows() {
        let name = row.name;
        let expected = row.expected;
        assert_eq!(failure(failed_fetch(row)), expected, "{name}");
    }
}
#[test]
fn a_get_is_retried_after_an_ambiguous_send() {
    for (evidence, status) in [(HttpEvidence::Unknown, 0), (HttpEvidence::Response, 503)] {
        let (mut client, get) = fetching(true);
        let answer = HttpResponse {
            id: get.id,
            status,
            body: Box::default(),
            retry_after: Duration::ZERO,
            evidence,
            now: time(2),
            wall: wall(100),
        };
        let pending = step(&mut client, Event::Http(answer));
        let retry = request(step(&mut client, Event::Tick { now: time(4) }));
        let terminal = step(&mut client, answered(retry.id, 200, &one_key(), 5));
        let visited = is_visit(terminal);
        assert_eq!(
            RetryOutcome {
                pending: pending.is_none(),
                method: retry.method,
                same_endpoint: retry.endpoint == get.endpoint,
                visited
            },
            RetryOutcome { pending: true, method: Method::Get, same_endpoint: true, visited: true },
            "{evidence:?}"
        );
    }
}
#[test]
fn cancel_while_fetching_keys_is_cancelled() {
    let (mut client, _) = fetching(true);
    assert_eq!(failure(step(&mut client, Event::Cancel)), Failure::Cancelled);
}
#[test]
fn the_key_caps_are_admitted_where_the_design_allows() {
    for (bits, accepted) in [(2047, false), (2048, true), (8192, true), (8193, false)] {
        let mut bounds = limits();
        bounds.modulus_bits = bits;
        assert_eq!(Client::new(bounds).is_ok(), accepted, "bits {bits}");
    }
}
#[test]
fn the_metadata_cap_is_required_only_for_discovery() {
    for (direct, expected) in [(false, Err(Failure::Limit)), (true, Ok(()))] {
        let mut bounds = limits();
        bounds.metadata_bytes = 0;
        assert_eq!(validate_registration(&registration_for_keys(direct), &bounds, true), expected, "direct {direct}");
    }
}
#[test]
fn openid_key_routes_require_nonzero_key_storage_capacities() {
    skein_rows::check(
        [
            ("the_key_set_cap_is_required_for_each_keys_route", ClientLimits { key_set_bytes: 0, ..limits() }),
            ("the_key_count_cap_is_required_for_each_keys_route", ClientLimits { keys: 0, ..limits() }),
        ],
        |bounds| {
            for direct in [false, true] {
                assert_eq!(
                    validate_registration(&registration_for_keys(direct), &bounds, true),
                    Err(Failure::Limit),
                    "direct {direct}"
                );
            }
        },
    );
}

fn is_visit(result: Option<Request>) -> bool {
    match result {
        Some(Request::Visit { .. }) => true,
        Some(Request::Http(_) | Request::Tokens { .. } | Request::Revoked { .. } | Request::Failed { .. }) | None => {
            false
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
struct GetShape {
    method: Method,
    content_type: usize,
    body: usize,
}
fn get_shape(get: &HttpRequest) -> GetShape {
    GetShape { method: get.method, content_type: get.content_type.len(), body: get.body.len() }
}
#[derive(Debug, PartialEq, Eq)]
struct RetryOutcome {
    pending: bool,
    method: Method,
    same_endpoint: bool,
    visited: bool,
}

fn redeeming() -> (Client, HttpRequest) {
    redeeming_with(&one_key())
}
fn redeeming_with(keys: &[u8]) -> (Client, HttpRequest) {
    let (mut client, get) = fetching(true);
    let _visit = step(&mut client, answered(get.id, 200, keys, 2));
    let event = Event::Redirected {
        client_id: None,
        uri: boxed(b"http://127.0.0.1:2345/callback"),
        state: state(),
        code: Some(secret(b"code")),
        error: None,
        now: time(3),
    };
    let request = request(step(&mut client, event));
    (client, request)
}
#[test]
fn an_openid_exchange_without_an_id_token_is_missing() {
    let (mut client, post) = redeeming();
    let terminal = step(&mut client, answered(post.id, 200, &token_body(), 4));
    assert_eq!(failure(terminal), Failure::Unverified { check: Check::Missing });
}
#[test]
fn an_id_token_past_its_cap_ends_as_limit_before_any_check() {
    const ID_CAP: usize = 1024; // limits().document.id_token_bytes.
    let (mut client, post) = redeeming();
    let mut bounds = limits().document;
    bounds.id_token_bytes = u32::try_from(ID_CAP.checked_add(1).expect("fixture")).expect("fixture");
    bounds.string_bytes = bounds.id_token_bytes;
    let response = TokenResponse {
        access_token: secret(b"access"),
        refresh_token: Some(secret(b"refresh")),
        expires_in: 30,
        scope: None,
        id_token: Some(secret(&[b'a'; ID_CAP + 1])),
        earliest_refresh_at: None,
    };
    let body = encode_response(&response, &bounds).expect("answer just past cap");
    let terminal = step(&mut client, answered(post.id, 200, &body, 4));
    assert_eq!(failure(terminal), Failure::Limit);
}
fn refresh_id_token(payload: Option<&[u8]>) -> Option<Secret> {
    let payload = payload?;
    let payload = encoded(payload);
    let mut writer = Writer::new(payload.len().checked_add(7).expect("fixture"));
    for part in [b"e30.", payload.as_ref(), b".AA"] {
        writer.put(part).expect("measured");
    }
    Some(Secret::new(writer.finish()))
}

#[test]
fn a_refresh_keeps_the_sign_ins_id_token_unchecked() {
    const GOOD: &[u8] = br#"{"iss":"https://issuer.example","aud":"client","exp":1100,"sub":"user-42","nonce":"not-compared","email":"you@example.com","email_verified":true,"name":"Example Person"}"#;
    const PROFILE: Profile = Profile { email: true, name: true };
    for (name, payload, wanted, says) in [
        ("valid without signature", Some(GOOD), PROFILE, true),
        ("not requested", Some(GOOD), Profile::default(), false),
        ("issuer", Some(br#"{"iss":"https://other.example","aud":"client","exp":1100,"sub":"user-42","nonce":"n","name":"Example Person"}"#), PROFILE, false),
        ("audience", Some(br#"{"iss":"https://issuer.example","aud":"other","exp":1100,"sub":"user-42","nonce":"n","name":"Example Person"}"#), PROFILE, false),
        ("azp", Some(br#"{"iss":"https://issuer.example","aud":["client","other"],"azp":"other","exp":1100,"sub":"user-42","nonce":"n","name":"Example Person"}"#), PROFILE, false),
        ("expiry", Some(br#"{"iss":"https://issuer.example","aud":"client","exp":0,"sub":"user-42","nonce":"n","name":"Example Person"}"#), PROFILE, false),
        ("subject", Some(br#"{"iss":"https://issuer.example","aud":"client","exp":1100,"sub":"other","nonce":"n","name":"Example Person"}"#), PROFILE, false),
        ("no nonce", Some(br#"{"iss":"https://issuer.example","aud":"client","exp":1100,"sub":"user-42","name":"Example Person"}"#), PROFILE, false),
        ("no id token", None, PROFILE, false),
        ("malformed", Some(b"not JSON"), PROFILE, false),
    ] {
        let mut registration = registration_for_keys(true);
        registration.terms.as_mut().expect("terms").openid.as_mut().expect("openid").profile = wanted;
        let mut client = Client::new(limits()).expect("client");
        let mut event = refresh(registration, None);
        if let Event::Refresh { prior, .. } = &mut event {
            prior.subject = Some(boxed(b"user-42"));
            prior.id_token = Some(secret(b"prior-id"));
        }
        let post = request(step(&mut client, event));
        let id_token = refresh_id_token(payload);
        let response = TokenResponse {
            access_token: secret(b"access"), refresh_token: Some(secret(b"refresh")), expires_in: 30,
            scope: None, id_token, earliest_refresh_at: None,
        };
        let body = encode_response(&response, &limits().document).expect("answer");
        let Some(Request::Tokens { record, profile, .. }) = step(&mut client, answered(post.id, 200, &body, 2)) else {
            panic!("refresh must succeed: {name}")
        };
        assert!(record.subject.as_deref() == Some(b"user-42"), "{name}");
        assert!(secret_ref(record.id_token.as_ref()) == Some(b"prior-id"), "{name}");
        let expected = SavedToken {
            key: 7, generation: 2, access_token: secret(b"access"), refresh_token: Some(secret(b"refresh")),
            metadata: None, expires_at: wall(32), earliest_refresh_at: None, issued_client_id: None,
            granted_scopes: None, subject: Some(boxed(b"user-42")), id_token: Some(secret(b"prior-id")),
        };
        assert!(record == expected, "record changed for {name}");
        assert_eq!(profile, if says { ProfileClaims { email: Some(boxed(b"you@example.com")), name: Some(boxed(b"Example Person")) } } else { ProfileClaims::default() }, "{name}");
    }
}

#[test]
fn an_openid_sign_in_without_refresh_keeps_its_identity() {
    use super::access_only_openid::{ANSWER, KEY_SET};
    let (mut client, post) = redeeming_with(KEY_SET);
    let response = decode_response(&Json::from_bytes(ANSWER, &limits().document).expect("answer"), &limits().document)
        .expect("tokens");
    let Some(Request::Tokens { record, .. }) = step(&mut client, answered(post.id, 200, ANSWER, 4)) else {
        panic!("tokens expected")
    };
    assert_eq!((record.key, record.generation, record.expires_at), (7, 1, wall(34)));
    assert!(record.refresh_state().is_none());
    assert!(record.access_token.expose() == b"access");
    assert_eq!(record.subject.as_deref(), Some(b"user-42".as_slice()));
    assert!(record.id_token.is_some() && record.id_token == response.id_token);
    let encoded = encode_record(&record, &limits().document).expect("encode");
    assert_eq!(decode_record(&encoded, &limits().document).expect("decode"), record);
}
