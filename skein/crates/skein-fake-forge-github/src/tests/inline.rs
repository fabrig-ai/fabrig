use super::{fixtures as f, wall};
use crate::inline::{self, Inline, Lent};
use crate::server::{self, Server};
use crate::{Service, fixture, installations::Scope};
use skein_fake_forge_domain::{self as domain, api as a};
use skein_forge::github;
use skein_lib::stream::{Down, Up};
use skein_lib::{Env, Queue, Secret, Time, Token, bytes};
fn forge(seed: u64) -> Inline {
    let config = fixture::domain_config();
    let mut domain = domain::Domain::new(&config, seed);
    let setup = a::Setup { hooked: true, ..fixture::setup(2, fixture::PATH) };
    domain::repository(&mut domain, &config, setup);
    domain::grant(&mut domain, fixture::PATH, fixture::BOT, a::Permission::Write);
    let limits = fixture::limits();
    let mut service = Service::new(fixture::config(fixture::widgets(2)), &limits, seed).unwrap();
    fixture::install_one(&mut service, wall(1000));
    Inline::new(service, domain, config, limits)
}
fn request(method: &[u8], target: &[u8], authorization: &[u8], body: &[u8]) -> Box<[u8]> {
    let line = f::parts(&[method, b" ", target, b" HTTP/1.1\r\nHost: github.test\r\nUser-Agent: test\r\n"]);
    let version = github::pin::API_VERSION.as_bytes();
    let length = skein_lib::Decimal::of(skein_lib::len_u64(body.len()));
    let auth = f::parts(&[b"\r\nAuthorization: ", authorization, b"\r\nContent-Length: ", length.as_bytes()]);
    f::parts(&[&line, b"X-GitHub-Api-Version: ", version, &auth, b"\r\n\r\n", body])
}
fn token(seed: u64) -> (Inline, Secret) {
    let mut forge = forge(seed);
    let (secret, _) = forge.service.installations_mut().mint(1, Scope::Full, wall(1200)).unwrap();
    (forge, f::bearer(secret.expose()))
}
fn header<'a>(wire: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    let start = bytes::find(wire, &f::parts(&[b"\r\n", name, b": "]))?.checked_add(name.len())?.checked_add(4)?;
    let tail = wire.get(start..)?;
    tail.get(..bytes::find(tail, b"\r\n")?)
}
fn get(forge: &mut Inline, target: &[u8], auth: &[u8]) -> Box<[u8]> {
    forge.serve(&request(b"GET", target, auth, b""), wall(1200))
}
struct Connection {
    server: Server,
    above: Queue<server::Event>,
    below: Queue<Down>,
}
impl Connection {
    fn head(forge: &mut Inline, target: &[u8], auth: &[u8]) -> Self {
        let server = Server::new(Token::new(77), &forge.limits).unwrap();
        let (above, below) = (Queue::uncapped(), Queue::uncapped());
        let mut c = Self { server, above, below };
        let env = Env { now: Time::from_nanos(wall(1200).as_nanos()), wall: wall(1200), limits: forge.limits };
        server::start(&mut c.server, &mut forge.service, &env, &mut c.above, &mut c.below);
        c.below.clear();
        c.run(forge, Some(Up::Room), false);
        for line in request(b"GET", target, auth, b"").split_inclusive(|byte| *byte == b'\n') {
            c.run(forge, Some(Up::Bytes(f::boxed(line))), false);
        }
        c.below.clear();
        c
    }
    fn run(&mut self, forge: &mut Inline, up: Option<Up>, hooks: bool) {
        let env = Env { now: Time::from_nanos(wall(1200).as_nanos()), wall: wall(1200), limits: forge.limits };
        if let Some(up) = up {
            server::up(&mut self.server, &mut forge.service, &env, up, &mut self.above, &mut self.below);
        }
        let domain_env = Env { now: env.now, wall: env.wall, limits: forge.config };
        let mut lent = Lent { domain: &mut forge.domain, env: &domain_env, hooks };
        inline::pass(&mut self.server, &mut forge.service, &mut lent, &env, &mut self.above, &mut self.below);
    }
}
#[test]
fn an_app_request_is_answered_whole() {
    let wire = get(&mut forge(1), f::INSTALLATIONS, f::authorization(fixture::key(), 1000, 1600).expose());
    assert!(wire.starts_with(b"HTTP/1.1 200 "), "whole App response");
    assert_eq!(header(&wire, b"Date"), Some(b"Thu, 01 Jan 1970 00:20:00 GMT".as_slice()));
    assert_eq!(header(&wire, b"Content-Type"), Some(b"application/json; charset=utf-8".as_slice()));
    let body = wire.get(bytes::find(&wire, b"\r\n\r\n").unwrap().checked_add(4).unwrap()..).unwrap();
    let length = skein_lib::Decimal::of(skein_lib::len_u64(body.len()));
    assert_eq!(header(&wire, b"Content-Length"), Some(length.as_bytes()));
}
#[test]
fn a_token_read_is_admitted_then_called() {
    let (mut forge, auth) = token(2);
    let wire = get(&mut forge, b"/repos/acme/widgets/branches/main", auth.expose());
    assert!(wire.starts_with(b"HTTP/1.1 200 "), "branch response");
    assert_eq!(header(&wire, b"X-RateLimit-Used"), Some(b"1".as_slice()));
    let commit = skein_forge::forgejo::id::write(&skein_fake_forge_protocol::routes::object(1));
    assert!(bytes::find(&wire, &commit).is_some(), "domain commit in body");
}
#[test]
fn a_conditional_read_admits_after_its_call() {
    let (mut forge, auth) = token(3);
    let wire = get(&mut forge, b"/repos/acme/widgets", auth.expose());
    let auth = f::parts(&[auth.expose(), b"\r\nIf-None-Match: ", header(&wire, b"ETag").unwrap()]);
    for (merge, status, used) in [(true, b"304".as_slice(), b"1".as_slice()), (false, b"200", b"2")] {
        let settings = a::Settings { default: f::boxed(b"main"), merge, squash: true, rebase: true };
        domain::settings(&mut forge.domain, fixture::PATH, settings);
        let wire = get(&mut forge, b"/repos/acme/widgets", &auth);
        assert!(wire.starts_with(&f::parts(&[b"HTTP/1.1 ", status])), "conditional status");
        assert_eq!(header(&wire, b"X-RateLimit-Used"), Some(used));
        assert_eq!(header(&wire, b"Content-Length").is_none(), status == b"304");
    }
}
#[test]
fn a_blob_request_records_its_headers() {
    let mut forge = forge(4);
    let wire = forge.blob(&request(b"GET", b"/logs/invalid", b"Bearer fake\r\nRange: bytes=0-0", b""), wall(1200));
    assert!(wire.starts_with(b"HTTP/1.1 403 ") && wire.ends_with(b"\r\n\r\n"), "empty tamper refusal");
    assert_eq!(forge.service.blob_records().len(), 1);
    assert!(forge.service.blob_records().iter().next().unwrap().range, "Range recorded");
}
#[test]
fn a_request_without_host_is_refused() {
    let mut forge = forge(5);
    let wire = forge.serve(b"GET /app HTTP/1.1\r\n\r\n", wall(1200));
    assert!(wire.starts_with(b"HTTP/1.1 400 "), "Host required");
    assert!(forge.service.blob_records().is_empty(), "HTTP refusal precedes service");
}
#[test]
fn a_pass_stops_at_a_head_and_lends_its_credential() {
    let (mut forge, auth) = token(6);
    let mut c = Connection::head(&mut forge, b"/repos/acme/widgets/branches/main", auth.expose());
    assert_eq!(forge.domain.tally().answered, 0);
    assert_eq!(c.server.take_head().unwrap().header(b"authorization"), Some(b"".as_slice()));
    assert_eq!(c.server.credential(), Some(auth.expose()));
    c.run(&mut forge, None, false);
    assert_eq!(c.server.credential(), None);
    assert_eq!(forge.domain.tally().answered, 1);
}
#[test]
fn the_peers_server_names_a_bare_media_type() {
    let mut forge = forge(7);
    let mut c = Connection::head(&mut forge, f::INSTALLATIONS, f::authorization(fixture::key(), 1000, 1600).expose());
    drop(c.server.take_head());
    c.run(&mut forge, None, false);
    let Some(Down::Send(wire)) = c.below.pop() else { panic!("response head") };
    assert_eq!(header(&wire, b"Content-Type"), Some(b"application/json".as_slice()));
}
#[test]
fn the_fixture_builds_a_valid_service() {
    for rows in [fixture::repositories(), fixture::widgets(1)] {
        let mut service = Service::new(fixture::config(rows), &fixture::limits(), 1).expect("valid fixture");
        fixture::install_three(&mut service);
        assert!(service.installations_mut().get(3).unwrap().suspended, "third suspended");
    }
}
