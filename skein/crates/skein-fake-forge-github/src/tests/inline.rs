use super::{fixtures as f, wall};
use crate::{
    fixture,
    inline::{self, Inline, Lent},
    server::{self, Server},
    service::Service,
};
use skein_fake_forge_domain::{self as domain, api as a};
use skein_forge::github::{self, request::Method};
use skein_lib::{
    Env, Queue, ReplyTo, Secret, Time, Token, bytes,
    stream::{Down, Up},
};

fn forge(seed: u64) -> Inline {
    let config = fixture::domain_config();
    let mut domain = domain::Domain::new(&config, seed);
    domain::repository(&mut domain, &config, fixture::setup(2, fixture::PATH));
    domain::grant(&mut domain, fixture::PATH, fixture::BOT, a::Permission::Write);
    let limits = fixture::limits();
    let mut service = Service::new(fixture::config(fixture::widgets(2)), &limits, seed).unwrap();
    fixture::install_one(&mut service, wall(1000));
    Inline::new(service, domain, config, limits)
}
fn request(method: &[u8], target: &[u8], authorization: &[u8], body: &[u8]) -> Box<[u8]> {
    f::parts(&[
        method,
        b" ",
        target,
        b" HTTP/1.1\r\nHost: github.test\r\nUser-Agent: test\r\nX-GitHub-Api-Version: ",
        github::pin::API_VERSION.as_bytes(),
        b"\r\nAuthorization: ",
        authorization,
        b"\r\nContent-Length: ",
        skein_lib::Decimal::of(skein_lib::len_u64(body.len())).as_bytes(),
        b"\r\n\r\n",
        body,
    ])
}
fn token(forge: &mut Inline) -> Secret {
    f::bearer(
        forge.service.installations_mut().mint(1, crate::installations::Scope::Full, wall(1200)).unwrap().0.expose(),
    )
}
fn header<'a>(wire: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    let start = bytes::find(wire, &f::parts(&[b"\r\n", name, b": "]))?.checked_add(name.len())?.checked_add(4)?;
    let tail = wire.get(start..)?;
    tail.get(..bytes::find(tail, b"\r\n")?)
}
struct Connection {
    server: Server,
    env: Env<crate::Limits>,
    above: Queue<server::Event>,
    below: Queue<Down>,
}
impl Connection {
    fn head(forge: &mut Inline, wire: &[u8]) -> Self {
        let env = Env { now: Time::from_nanos(wall(1200).as_nanos()), wall: wall(1200), limits: fixture::limits() };
        let mut c = Self {
            server: Server::new(Token::new(77), &env.limits).unwrap(),
            env,
            above: Queue::uncapped(),
            below: Queue::uncapped(),
        };
        server::start(&mut c.server, &mut forge.service, &c.env, &mut c.above, &mut c.below);
        let mut input = skein_lib::Intake::with_capacity(c.env.limits.http.head);
        input.append(wire).unwrap();
        for _ in 0..c.env.limits.http.headers.checked_add(2).unwrap() {
            let Some(Down::Demand { read, .. }) = c.below.pop() else { break };
            c.up(forge, Up::Bytes(input.meet(read).unwrap()));
        }
        c
    }
    fn up(&mut self, forge: &mut Inline, up: Up) {
        server::up(&mut self.server, &mut forge.service, &self.env, up, &mut self.above, &mut self.below);
    }
    fn pass(&mut self, forge: &mut Inline, hooks: bool) {
        let env = Env { now: self.env.now, wall: self.env.wall, limits: forge.config };
        inline::pass(
            &mut self.server,
            &mut forge.service,
            &mut Lent { domain: &mut forge.domain, env: &env, hooks },
            &self.env,
            &mut self.above,
            &mut self.below,
        );
    }
}
#[test]
fn an_app_request_is_answered_whole() {
    let wire = forge(1).serve(
        &request(b"GET", b"/app/installations", f::authorization(fixture::key(), 1000, 1600).expose(), b""),
        wall(1200),
    );
    assert!(wire.starts_with(b"HTTP/1.1 200 "));
    assert_eq!(header(&wire, b"Date"), Some(b"Thu, 01 Jan 1970 00:20:00 GMT".as_slice()));
    assert_eq!(header(&wire, b"Content-Type"), Some(b"application/json; charset=utf-8".as_slice()));
    let body = wire.get(bytes::find(&wire, b"\r\n\r\n").unwrap().checked_add(4).unwrap()..).unwrap();
    assert_eq!(
        header(&wire, b"Content-Length"),
        Some(skein_lib::Decimal::of(skein_lib::len_u64(body.len())).as_bytes())
    );
}
#[test]
fn a_token_read_is_admitted_then_called() {
    let mut forge = forge(2);
    let auth = token(&mut forge);
    let wire = forge.serve(&request(b"GET", b"/repos/acme/widgets/branches/main", auth.expose(), b""), wall(1200));
    assert!(wire.starts_with(b"HTTP/1.1 200 "));
    assert_eq!(header(&wire, b"X-RateLimit-Used"), Some(b"1".as_slice()));
    assert!(bytes::find(&wire, b"0000000000000000000000000000000000000001").is_some());
}
#[test]
fn a_conditional_read_admits_after_its_call() {
    let mut forge = forge(3);
    let auth = token(&mut forge);
    let wire = forge.serve(&request(b"GET", b"/repos/acme/widgets", auth.expose(), b""), wall(1200));
    let auth = f::parts(&[auth.expose(), b"\r\nIf-None-Match: ", header(&wire, b"ETag").unwrap()]);
    for (merge, status, used) in [(true, b"304".as_slice(), b"1".as_slice()), (false, b"200", b"2")] {
        domain::settings(
            &mut forge.domain,
            fixture::PATH,
            a::Settings { default: f::boxed(b"main"), merge, squash: true, rebase: true },
        );
        let wire = forge.serve(&request(b"GET", b"/repos/acme/widgets", &auth, b""), wall(1200));
        assert!(wire.starts_with(&f::parts(&[b"HTTP/1.1 ", status])));
        assert_eq!(header(&wire, b"X-RateLimit-Used"), Some(used));
        if status == b"304" {
            assert_eq!(header(&wire, b"Content-Length"), None);
        }
    }
}
#[test]
fn a_blob_request_records_its_headers() {
    let mut forge = forge(4);
    let wire = forge.blob(&request(b"GET", b"/tampered", b"Bearer fake\r\nRange: bytes=0-0", b""), wall(1200));
    assert!(wire.starts_with(b"HTTP/1.1 403 "));
    assert!(wire.ends_with(b"\r\n\r\n"));
    let records = forge.service.blob_records();
    assert_eq!(records.len(), 1);
    assert!(records.iter().next().unwrap().range);
}
#[test]
fn a_request_without_host_is_refused() {
    let mut forge = forge(5);
    assert!(forge.serve(b"GET /app HTTP/1.1\r\n\r\n", wall(1200)).starts_with(b"HTTP/1.1 400 "));
    assert_eq!(forge.domain.calls(), 0);
    assert!(forge.service.blob_records().is_empty());
}
#[test]
fn a_pass_stops_at_a_head_and_lends_its_credential() {
    let mut forge = forge(6);
    let auth = token(&mut forge);
    let mut c =
        Connection::head(&mut forge, &request(b"GET", b"/repos/acme/widgets/branches/main", auth.expose(), b""));
    c.pass(&mut forge, false);
    assert_eq!(forge.domain.calls(), 0);
    assert!(c.server.take_head().unwrap().header(b"authorization").is_none());
    assert_eq!(c.server.credential(), Some(auth.expose()));
    c.pass(&mut forge, false);
    assert_eq!(c.server.credential(), None);
    assert_eq!(forge.domain.calls(), 1);
}
#[test]
fn the_peers_server_names_a_bare_media_type() {
    let mut forge = forge(7);
    let mut c = Connection::head(
        &mut forge,
        &request(b"GET", b"/app/installations", f::authorization(fixture::key(), 1000, 1600).expose(), b""),
    );
    drop(c.server.take_head());
    c.pass(&mut forge, false);
    c.up(&mut forge, Up::Room);
    c.pass(&mut forge, false);
    let mut media = None;
    for _ in 0..c.below.len() {
        if let Down::Send(wire) = c.below.pop().unwrap() {
            media = header(&wire, b"Content-Type").map(f::boxed);
        }
    }
    assert_eq!(media.as_deref(), Some(b"application/json".as_slice()));
}

#[test]
fn the_fixture_builds_a_valid_service() {
    for rows in [fixture::repositories(), fixture::widgets(1)] {
        let mut service = Service::new(fixture::config(rows), &fixture::limits(), 1).expect("valid fixture");
        fixture::install_three(&mut service);
        let store = service.installations_mut();
        let mut ids = skein_lib::List::with_capacity(3);
        for row in store.iter() {
            ids.push(row.id).expect("three ids");
        }
        assert_eq!(ids.as_slice(), &[1, 2, 3]);
        assert!(store.get(3).expect("third installation").suspended);
    }
}
