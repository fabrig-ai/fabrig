//! Fixtures for the machine world's hand (skein/docs/design/forge/connection.md, 10).
use core::net::{Ipv4Addr, SocketAddr};
use skein_forge::api;
use skein_forge_connection::{
    Limits, Settings, Shared,
    settings::{Forge, Kind, Transport},
};
use skein_lib::{Duration, List, Secret};
use std::boxed::Box;

pub const WORD: &[u8] = b"kit";
pub const USER_AGENT: &[u8] = b"kit/0.0.0";
pub const OP_BYTES: u32 = 512;
// Twice the operation bound, the owner's word, and marker framing (forge/README.md 5).
pub const BODY_ROOM: u32 = 2 * OP_BYTES + 3 + 15;
pub const WRITER: u64 = 2;

#[must_use]
pub fn limits() -> Limits {
    Limits {
        forges: 2,
        token_bytes: 64,
        key_bytes: 1200,
        connections: 2,
        kept_bytes: u64::from(shared().calls)
            .checked_mul(u64::from(shared().answer_bytes))
            .expect("fixture answer room"),
        head_bytes: 4096,
        headers: 32,
        body_bytes_in: 1_048_576,
        depth: 16,
        chunk: 1024,
        string_bytes: 128,
        body_bytes: BODY_ROOM,
        inner: 8,
        target_bytes: 1024,
        repositories: 2,
        users: 8,
        installations: 3,
        log_origins: 2,
        deadline: Duration::from_secs(10),
        idle: Duration::from_secs(5),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(8),
        renew_before: Duration::from_secs(60),
        // Beyond token-renewal stories; puller stories advance this timer themselves.
        log_poll: Duration::from_secs(86_400),
        log_catch_up: Duration::from_secs(3600),
        log_pages: 10,
        delivery_bytes: 65_536,
        hooks: 4,
        hook_head_bytes: 8192,
        hook_body_bytes: 1_048_576,
        hook_deadline: Duration::from_secs(10),
        hook_secret_bytes: 128,
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
    }
}

pub const FORGE: u16 = 1;
pub const REPO: api::Repository = api::Repository { forge: 1, repository: 1 };
pub const TOKEN: &[u8] = b"0123456789abcdef0123456789abcdef01234567";

#[must_use]
pub fn shared() -> Shared {
    Shared {
        calls: 2,
        op_bytes: OP_BYTES,
        answer_bytes: 65_536,
        rows: 8,
        lifetime: Duration::from_secs(10),
        name_bytes: 128,
        repositories: 2,
        io: skein_io::Limits { sockets: 8, refusals: 2, receive: 4096, sends: 4, ..skein_io::Limits::TEST },
    }
}

#[must_use]
pub fn settings() -> Settings {
    let mut forges = List::with_capacity(2);
    forges
        .push(Forge {
            number: FORGE,
            kind: Kind::Forgejo,
            log_origins: List::with_capacity(0),
            address: skein_io::Addr::Ip(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
            transport: Transport::Plaintext { name: None },
            access: Some(skein_forge_connection::Access {
                credential: skein_forge::credential::Credential::Token(Secret::new(Box::from(TOKEN))),
                user: WRITER,
                webhook_secret: None,
            }),
        })
        .expect("one forge");
    Settings { forges, webhooks: None, word: Box::from(WORD), user_agent: Box::from(USER_AGENT) }
}

#[must_use]
pub fn ok(body: &[u8]) -> Box<[u8]> {
    response(200, body)
}

#[must_use]
pub fn response(code: u16, body: &[u8]) -> Box<[u8]> {
    response_with_fields(code, body, b"")
}

#[must_use]
pub fn response_with_fields(code: u16, body: &[u8], fields: &[u8]) -> Box<[u8]> {
    let decimal = skein_lib::Decimal::of(skein_lib::len_u64(body.len()));
    let mut out = skein_lib::Writer::new(
        body.len()
            .saturating_add(fields.len())
            .saturating_add(b"HTTP/1.1 200 OK\r\nContent-Length: ".len())
            .saturating_add(decimal.as_bytes().len())
            .saturating_add(b"\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n".len()),
    );
    out.put(b"HTTP/1.1 ").expect("head fits");
    out.put(skein_lib::Decimal::of(u64::from(code)).as_bytes()).expect("code fits");
    out.put(b" OK\r\n").expect("head fits");
    out.put(fields).expect("fields fit");
    out.put(b"Content-Length: ").expect("head fits");
    out.put(decimal.as_bytes()).expect("length fits");
    out.put(b"\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n").expect("date fits");
    out.put(body).expect("body fits");
    out.finish()
}

#[must_use]
pub fn user_ok() -> Box<[u8]> {
    ok(b"{\"id\":2,\"login\":\"writer\"}")
}

/// The literal value of the sole named header on the HTTP wire.
#[must_use]
pub fn header<'a>(sent: &'a [u8], name: &[u8]) -> &'a [u8] {
    let mut found = None;
    let mut rest = sent;
    while let Some(end) = skein_lib::bytes::find(rest, b"\r\n") {
        let line = rest.get(..end).expect("line end in request");
        if let Some(value) = line.strip_prefix(name) {
            assert!(found.is_none(), "header occurs once");
            found = Some(value);
        }
        rest = rest.get(end.checked_add(2).expect("line end fits")..).expect("next line");
    }
    found.expect("header sent")
}

pub const TOKEN_BOUND: usize = 64;

#[must_use]
pub fn filled(length: usize, byte: u8) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(length);
    for _ in 0..length {
        out.put(&[byte]).expect("measured fill");
    }
    out.finish()
}

pub const OTHER_FORGE: u16 = 2;
pub const BRANCH: &[u8] = b"main";
pub const BRANCH_HEAD: [u8; 32] = [0; 32];

#[must_use]
pub fn app_access(key: &[u8], id: u64, user: u64) -> skein_forge_connection::Access {
    skein_forge_connection::Access {
        credential: skein_forge::credential::Credential::App { id, key: Secret::new(Box::from(key)) },
        user,
        webhook_secret: None,
    }
}

#[must_use]
pub fn github_settings() -> Settings {
    let mut settings = settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.kind = Kind::GitHub;
    forge.access = Some(app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
    settings
}
