//! Google's sign-in and a confidential `OpenID` client through the clients' HTTP world.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary protocol world fixtures use scratch files and textual HTTP"
)]
use super::{fixtures::http::Peer, sign_in};
use jig_client_protocol_wire as wire;
use jig_protocol_config::clients::{Provider, ProviderClient};
use skein_lib::{
    Duration, Reader, Secret, Writer,
    base64::{self, Alphabet},
    bytes,
};
use std::{
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
};
const CALLBACK: &str = "https://forge.example/user/oauth2/temper/callback";
const VERIFIER: &str = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFG";

struct Configuration {
    provider: Provider,
    files: [(Box<[u8]>, Secret); 2],
}
fn configuration() -> Configuration {
    let scratch =
        skein_scratch::paths::package().join("../../target").join(format!("provider-world-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o700)).unwrap();
    let key = scratch.join("provider.key");
    assert!(
        Command::new("openssl")
            .args(["genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:3072", "-out"])
            .arg(&key)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600)).unwrap();
    let secret = scratch.join("forgejo.secret");
    std::fs::write(&secret, b"throwaway-provider-secret").unwrap();
    std::fs::set_permissions(&secret, std::fs::Permissions::from_mode(0o600)).unwrap();
    let files = [
        (b"provider.key".as_slice().into(), Secret::new(std::fs::read(&key).unwrap().into())),
        (b"forgejo.secret".as_slice().into(), Secret::new(std::fs::read(&secret).unwrap().into())),
    ];
    std::fs::remove_dir_all(&scratch).unwrap();
    Configuration {
        provider: Provider {
            key: b"provider.key".as_slice().into(),
            addresses: Box::new([3]),
            clients: Box::new([ProviderClient {
                id: b"forgejo".as_slice().into(),
                secret: b"forgejo.secret".as_slice().into(),
                redirect_uri: CALLBACK.as_bytes().into(),
            }]),
        },
        files,
    }
}
fn request(service: &mut crate::Service, method: &str, path: &str, fields: &str, body: &[u8]) -> Peer {
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: temper.example.com\r\nConnection: close\r\nContent-Length: {}\r\n{fields}\r\n",
        body.len()
    );
    let mut request = head.into_bytes();
    request.extend_from_slice(body);
    let mut peer = Peer::raw(service.engine.as_mut().unwrap(), &request);
    for _ in 0_u32..16 {
        sign_in::pump(service, &mut peer);
        if peer.is_closed() {
            break;
        }
    }
    assert!(peer.is_closed(), "the HTTP exchange completes");
    peer
}
fn body(peer: &Peer) -> &[u8] {
    let at = bytes::find(peer.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    peer.received().get(at..).unwrap()
}
fn json(peer: &Peer) -> skein_json::Document {
    jig_protocol_config::document(
        body(peer),
        &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
    )
    .unwrap()
}
fn text(document: &skein_json::Document, name: &[u8]) -> Box<[u8]> {
    let mut object = jig_protocol_config::read::Object::new(document, 0).unwrap();
    jig_protocol_config::read::text(document, object.member(document, name).unwrap(), 16_384).unwrap()
}
fn encoded(value: &[u8], alphabet: Alphabet, length: usize) -> Box<[u8]> {
    let mut out = Writer::new(length);
    base64::encode(alphabet, value, &mut out).unwrap();
    out.finish()
}
fn authorize(service: &mut crate::Service, cookie: &[u8]) -> Box<[u8]> {
    let challenge = encoded(&skein_lib::sha256::digest(VERIFIER.as_bytes()), Alphabet::Url, 43);
    let query = format!(
        "response_type=code&client_id=forgejo&redirect_uri=https%3A%2F%2Fforge.example%2Fuser%2Foauth2%2Ftemper%2Fcallback&scope=openid%20email%20profile&state=world&nonce=world&code_challenge={}&code_challenge_method=S256",
        core::str::from_utf8(&challenge).unwrap()
    );
    let limits = service.engine.as_ref().unwrap().clients.env.limits.wire;
    let record = wire::Authorize::new(&limits, wire::AuthorizeParts { query: query.into_bytes().into() }).unwrap();
    let mut out = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut out).unwrap();
    let fields = format!(
        "Content-Type: application/octet-stream\r\nJig-Client: browser\r\nCookie: {}\r\n",
        core::str::from_utf8(cookie).unwrap()
    );
    let peer = request(service, "POST", "/jig/v1/authorize", &fields, &out.finish());
    assert_eq!(peer.received().get(..12), Some(b"HTTP/1.1 200".as_slice()));
    let wire::AuthorizeAnswer::Go(go) = wire::AuthorizeAnswer::decode(&limits, &mut Reader::new(body(&peer))).unwrap()
    else {
        panic!("the signed-in page receives go")
    };
    let location = go.location();
    assert!(location.starts_with(CALLBACK.as_bytes()));
    let text = core::str::from_utf8(location).unwrap();
    text.split_once("code=").unwrap().1.split('&').next().unwrap().as_bytes().into()
}
#[test]
#[expect(clippy::print_stderr, reason = "report each world story’s elapsed time without any secret bytes")]
fn a_google_member_redeems_by_basic_and_by_body_and_reads_the_same_claims() {
    let Configuration { provider, files } = configuration();
    let (mut service, mut google) = sign_in::world_settings(2, Duration::from_secs(600), true, Some(provider), &files);
    sign_in::account(&mut google, b"google-sub", Some(b"sam@example.com"), true);
    let signed = sign_in::signed_return(&mut service, &mut google, b"/sign-in/google");
    assert!(signed.received().starts_with(b"HTTP/1.1 303"));
    let cookie = sign_in::header(&signed, "set-cookie").split(|byte| *byte == b';').next().unwrap().to_vec();
    assert!(cookie.starts_with(b"jig-sign-in="));
    for basic in [true, false] {
        let started = std::time::Instant::now();
        let code = authorize(&mut service, &cookie);
        let mut form = format!(
            "grant_type=authorization_code&code={}&redirect_uri=https%3A%2F%2Fforge.example%2Fuser%2Foauth2%2Ftemper%2Fcallback&code_verifier={VERIFIER}",
            core::str::from_utf8(&code).unwrap()
        );
        let mut fields = "Content-Type: application/x-www-form-urlencoded\r\n".to_owned();
        if basic {
            let credentials = b"forgejo:throwaway-provider-secret";
            let length = credentials.len().div_ceil(3).checked_mul(4).unwrap();
            let encoded = encoded(credentials, Alphabet::Standard, length);
            fields.push_str("Authorization: Basic ");
            fields.push_str(core::str::from_utf8(&encoded).unwrap());
            fields.push_str("\r\n");
        } else {
            form.push_str("&client_id=forgejo&client_secret=throwaway-provider-secret");
        }
        let token = request(&mut service, "POST", "/oauth/token", &fields, form.as_bytes());
        assert!(token.received().starts_with(b"HTTP/1.1 200"));
        assert_eq!(sign_in::header(&token, "cache-control"), b"no-store");
        assert_eq!(sign_in::header(&token, "content-type"), b"application/json");
        let document = json(&token);
        let access = text(&document, b"access_token");
        let id = text(&document, b"id_token");
        let payload = id.split(|byte| *byte == b'.').nth(1).unwrap();
        let payload = base64::decode(Alphabet::Url, payload, 16_384).unwrap();
        let claims = jig_protocol_config::document(
            &payload,
            &jig_protocol_config::DocumentLimits { bytes: 16_384, tokens: 512, depth: 8 },
        )
        .unwrap();
        assert_eq!(text(&claims, b"sub").as_ref(), b"3:sam@example.com");
        let userinfo = request(
            &mut service,
            "GET",
            "/oauth/userinfo",
            &format!("Authorization: Bearer {}\r\n", core::str::from_utf8(&access).unwrap()),
            &[],
        );
        assert!(userinfo.received().starts_with(b"HTTP/1.1 200"));
        let user = json(&userinfo);
        assert_eq!(text(&user, b"sub"), text(&claims, b"sub"));
        assert_eq!(text(&user, b"preferred_username").as_ref(), b"sam");
        assert_eq!(text(&user, b"email").as_ref(), b"sam@example.com");
        let mut object = jig_protocol_config::read::Object::new(&user, 0).unwrap();
        assert!(jig_protocol_config::read::boolean(&user, object.member(&user, b"email_verified").unwrap()).unwrap());
        std::eprintln!(
            "provider {} story: {} s",
            if basic { "Basic" } else { "body" },
            started.elapsed().as_secs_f64()
        );
    }
    let discovery = request(&mut service, "GET", "/.well-known/openid-configuration", "", &[]);
    assert_eq!(text(&json(&discovery), b"issuer").as_ref(), b"https://temper.example.com");
    assert_eq!(text(&json(&discovery), b"token_endpoint").as_ref(), b"https://temper.example.com/oauth/token");
    let keys = request(&mut service, "GET", "/oauth/keys", "", &[]);
    assert!(core::str::from_utf8(body(&keys)).unwrap().contains("\"RS256\""));
    for path in ["/.well-known/openid-configuration", "/oauth/keys", "/oauth/token", "/oauth/userinfo"] {
        let peer = request(&mut service, "PUT", path, "", &[]);
        assert!(peer.received().starts_with(b"HTTP/1.1 405"));
    }
    let bound = jig_protocol_engine_clients::provider_limits(&service.engine.as_ref().unwrap().clients.env.limits)
        .request_bytes;
    let oversized = vec![b'x'; usize::try_from(bound).unwrap().checked_add(1).unwrap()];
    let refused = request(
        &mut service,
        "POST",
        "/oauth/token",
        "Content-Type: application/x-www-form-urlencoded\r\n",
        &oversized,
    );
    assert!(refused.received().starts_with(b"HTTP/1.1 413"));
}
#[test]
fn provider_routes_without_a_provider_are_not_found() {
    let (mut service, _) = sign_in::world(2);
    for (method, path) in [
        ("GET", "/.well-known/openid-configuration"),
        ("GET", "/oauth/keys"),
        ("POST", "/oauth/token"),
        ("GET", "/oauth/userinfo"),
    ] {
        let peer = request(&mut service, method, path, "", &[]);
        assert!(peer.received().starts_with(b"HTTP/1.1 404"));
    }
}
