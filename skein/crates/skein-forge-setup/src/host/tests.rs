use super::*;
use crate::tests::fixtures;
use skein_lib::Wall;

struct Running {
    host: Host,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<skein_io::Request>,
    owner: Token,
    socket: Token,
}
impl Running {
    fn new() -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
        let mut host = Host::new(&env.limits, crate::tests::host::settings(&[1]), crate::tests::host::crypto())
            .expect("host fixtures");
        let mut up = Queue::uncapped();
        let mut io = Queue::uncapped();
        host.down(
            &env,
            Request::Run { run: Token::new(1), plan: Plan::Reconcile { forge: 1, adopted: [].into() } },
            &mut up,
            &mut io,
        );
        assert_eq!(up.pop(), Some(Event::Need { run: Token::new(1), need: Need::Read(crate::item::Item::AdminToken) }));
        host.down(
            &env,
            Request::Answer { run: Token::new(1), answer: Answer::Read(Some(fixtures::secret(fixtures::ADMIN))) },
            &mut up,
            &mut io,
        );
        let Some(skein_io::Request::Connect { owner, .. }) = io.pop() else {
            panic!("a run opens its call");
        };
        let socket = Token::new(9);
        host.up(&env, skein_io::Event::Connecting { owner, socket }, &mut up, &mut io);
        assert_eq!(host.runs.len(), 1);
        assert_eq!(host.connections.len(), 1);
        assert!(up.is_empty());
        assert!(io.is_empty());
        Self { host, env, up, io, owner, socket }
    }
    fn down(&mut self, request: Request) {
        self.host.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn aborted(&mut self) {
        assert!(self.host.closing);
        assert!(self.host.aborting);
        assert!(self.host.runs.is_empty());
        assert!(self.up.is_empty());
        assert_eq!(self.io.pop(), Some(skein_io::Request::Abort { entity: self.socket }));
        assert!(self.io.is_empty());
        assert_eq!(self.host.connections.len(), 1);
        assert_eq!(self.host.next_deadline(), None);
    }
    fn closed(mut self) {
        self.host.up(&self.env, skein_io::Event::Connected { owner: self.owner }, &mut self.up, &mut self.io);
        self.host.up(
            &self.env,
            skein_io::Event::Stream { owner: self.owner, up: skein_lib::stream::Up::Bytes(bytes::copy_of(b"late")) },
            &mut self.up,
            &mut self.io,
        );
        assert!(self.up.is_empty());
        assert!(self.io.is_empty());
        self.host.up(&self.env, skein_io::Event::Closed { owner: self.owner }, &mut self.up, &mut self.io);
        assert_eq!(self.up.pop(), Some(Event::Closed));
        self.host.fire(&self.env, &mut self.up, &mut self.io);
        assert!(self.up.is_empty());
        assert!(self.io.is_empty());
        assert!(self.host.runs.is_empty());
        assert!(self.host.connections.is_empty());
        assert_eq!(self.host.next_deadline(), None);
    }
}

#[test]
fn abort_ends_running_and_waiting_runs_unheard_until_io_says_closed() {
    let mut running = Running::new();
    running.down(Request::Run {
        run: Token::new(2),
        plan: Plan::Reachable { forge: 1, installation: None, page: core::num::NonZeroU32::MIN },
    });
    assert_eq!(
        running.up.pop(),
        Some(Event::Need { run: Token::new(2), need: Need::Read(crate::item::Item::AdminToken) })
    );
    assert_eq!(running.host.runs.len(), 2);
    running.down(Request::Abort);
    running.aborted();
    running.closed();
}

#[test]
fn abort_after_close_marks_the_host_aborting_without_repeating_socket_aborts() {
    let mut running = Running::new();
    running.down(Request::Close);
    assert!(!running.host.aborting);
    running.down(Request::Abort);
    running.aborted();
    running.closed();
}

#[test]
fn close_after_abort_changes_nothing() {
    let mut running = Running::new();
    running.down(Request::Abort);
    running.aborted();
    running.down(Request::Close);
    assert!(running.host.aborting);
    assert!(running.up.is_empty());
    assert!(running.io.is_empty());
    running.closed();
}

#[test]
fn a_second_abort_changes_nothing() {
    let mut running = Running::new();
    running.down(Request::Abort);
    running.aborted();
    running.down(Request::Abort);
    assert!(running.host.aborting);
    assert!(running.up.is_empty());
    assert!(running.io.is_empty());
    running.closed();
}

#[test]
fn abort_upgrades_a_graceful_close_already_sent_to_io() {
    use crate::host::connection::Connection;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
    let forge = Forge {
        kind: Kind::Forgejo(fixtures::given()),
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()),
        transport: Transport::Plaintext { name: None },
    };
    let mut c = Connection::new(
        Token::new(1),
        &forge,
        b"kit",
        &skein_forge::forgejo::request::Request::User.into(),
        &fixtures::secret(fixtures::ADMIN),
        &env.limits,
    )
    .expect("connection");
    c.socket = Some(Token::new(9));
    c.fail();
    let mut io = Queue::with_capacity(4);
    assert!(c.pump(&env, &mut io));
    assert_eq!(io.pop(), Some(skein_io::Request::Close { entity: Token::new(9) }));
    c.abort();
    assert!(c.pump(&env, &mut io));
    assert_eq!(io.pop(), Some(skein_io::Request::Abort { entity: Token::new(9) }));
    assert!(!c.pump(&env, &mut io));
}
#[test]
fn a_listing_past_the_configured_page_is_refused_at_open() {
    use skein_forge::forgejo::request::{Page, Request};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::tests::host::limits() };
    let forge = Forge {
        kind: Kind::Forgejo(fixtures::given()),
        address: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()),
        transport: Transport::Plaintext { name: None },
    };
    let send = |limit| Send {
        call: Request::Tokens { login: b"kit".as_slice().into(), page: Page { page: Some(1), limit: Some(limit) } }
            .into(),
        auth: crate::call::Auth::Token(fixtures::secret(fixtures::ADMIN)),
    };
    let inner = env.limits.inner;
    open(Token::new(1), &forge, b"kit", &send(inner), &env).expect("a page at inner fits");
    assert_eq!(open(Token::new(1), &forge, b"kit", &send(inner + 1), &env).err(), Some(Refused::Unexpected));
}

#[test]
fn valid_accepts_test_limits() {
    let l = crate::tests::host::limits();
    let settings = crate::tests::host::settings(&[1]);
    assert_eq!(limits::check(&l, &settings), Ok(()));
}

fn refuses_limits(l: Limits, settings: Settings) {
    let error = limits::check(&l, &settings).expect_err("unrepresentable limits");
    assert_eq!(Host::new(&l, settings, crate::tests::host::crypto()).err(), Some(error));
}

#[test]
fn valid_refuses_a_json_wire_length_past_u32() {
    let l = Limits { body_bytes_in: u32::MAX, ..crate::tests::host::limits() };
    refuses_limits(l, crate::tests::host::settings(&[1]));
}

#[test]
fn valid_refuses_an_unrepresentable_http_head() {
    let l = Limits { head_bytes: 1, ..crate::tests::host::limits() };
    refuses_limits(l, crate::tests::host::settings(&[1]));
}

#[test]
fn valid_refuses_unrepresentable_tls_intake() {
    let mut l = crate::tests::host::limits();
    l.tls.read = u32::MAX;
    refuses_limits(l, crate::tests::host::settings(&[1]));
}

#[test]
fn valid_refuses_a_key_spelling_past_u32() {
    let mut settings = crate::tests::host::github_settings();
    for index in 0..settings.forges.len() {
        let forge = settings.forges.get_mut(index).unwrap();
        if let Kind::GitHub(given) = &mut forge.kind {
            given.limits.key_bytes = u32::MAX;
        }
    }
    refuses_limits(crate::tests::host::limits(), settings);
}

#[test]
fn valid_ignores_heap_run_plans() {
    let l = Limits { runs: u32::MAX, ..crate::tests::host::limits() };
    let mut settings = crate::tests::host::settings(&[1]);
    let Kind::Forgejo(given) = &mut settings.forges.get_mut(0).unwrap().kind else { unreachable!() };
    given.limits.files = u32::MAX;
    given.limits.file_bytes = u32::MAX;
    assert_eq!(limits::check(&l, &settings), Ok(()));
}

#[test]
fn valid_refuses_installation_slots_past_u32() {
    let mut settings = crate::tests::host::github_settings();
    for index in 0..settings.forges.len() {
        let forge = settings.forges.get_mut(index).unwrap();
        if let Kind::GitHub(given) = &mut forge.kind {
            given.limits.pages = u32::MAX;
        }
    }
    refuses_limits(crate::tests::host::limits(), settings);
}
