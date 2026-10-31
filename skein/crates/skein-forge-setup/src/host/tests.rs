use super::*;
use crate::tests::fixtures;
use skein_lib::Wall;

// Two forges give each held run its own connection within the fixture's limits.
const ABORT_PARTIES: [(Token, u16, Token); 2] =
    [(Token::new(11), 1, Token::new(101)), (Token::new(12), 2, Token::new(102))];

struct AbortHost {
    host: Host,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<skein_io::Request>,
    bindings: List<(Token, Token)>,
}
impl AbortHost {
    fn empty() -> Self {
        let limits = crate::tests::host::limits();
        Self {
            host: Host::new(&limits, crate::tests::host::settings(&[1, 2]), crate::tests::host::crypto())
                .expect("host fixtures"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            up: Queue::uncapped(),
            io: Queue::uncapped(),
            bindings: List::with_capacity(limits.runs),
        }
    }
    fn new() -> Self {
        let mut world = Self::empty();
        for (run, forge, socket) in ABORT_PARTIES {
            world.down(Request::Run { run, plan: Plan::Reconcile { forge, adopted: Box::new([]) } });
            let _read = world.up.pop().expect("owner read");
            world.down(Request::Answer { run, answer: Answer::Read(Some(fixtures::secret(fixtures::ADMIN))) });
            let Some(skein_io::Request::Connect { owner, .. }) = world.io.pop() else {
                panic!("run opens its connection");
            };
            world.bindings.push((owner, socket)).expect("one connection per run");
        }
        world
    }
    fn down(&mut self, request: Request) {
        self.host.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn event(&mut self, event: skein_io::Event) {
        self.host.up(&self.env, event, &mut self.up, &mut self.io);
    }
    fn bind_all(&mut self) {
        for index in 0..self.bindings.len() {
            let (owner, socket) = *self.bindings.get(index).expect("fixture binding");
            self.event(skein_io::Event::Connecting { owner, socket });
        }
    }
    fn take_aborts(&mut self) {
        for (_, socket) in &self.bindings {
            assert_eq!(self.io.pop(), Some(skein_io::Request::Abort { entity: *socket }));
        }
        assert!(self.io.is_empty());
    }
    fn fire(&mut self) {
        self.host.fire(&self.env, &mut self.up, &mut self.io);
    }
}

#[test]
fn an_abort_aborts_each_connection_and_ends_each_run() {
    let mut world = AbortHost::new();
    world.bind_all();
    let (owner, _) = *world.bindings.get(0).expect("first connection");
    world.host.connections.get_mut(&owner).expect("held connection").fail();

    world.down(Request::Abort);

    world.take_aborts();
    assert!(world.host.runs.is_empty());
    // Failure has no cancelled reason: abort ends runs unheard, as Close does.
    assert!(world.up.is_empty());
    assert_eq!(world.host.next_deadline(), None);
    for (run, _, _) in ABORT_PARTIES {
        world.down(Request::Answer { run, answer: Answer::Read(None) });
    }
    world.fire();
    assert!(world.up.is_empty() && world.io.is_empty());
}

#[test]
fn an_abort_after_a_close_changes_only_what_still_closes() {
    let mut world = AbortHost::new();
    let (settled, socket) = *world.bindings.get(0).expect("first connection");
    let (pending, late_socket) = *world.bindings.get(1).expect("second connection");
    world.event(skein_io::Event::Connecting { owner: settled, socket });

    world.down(Request::Close);
    assert_eq!(world.io.pop(), Some(skein_io::Request::Abort { entity: socket }));
    world.event(skein_io::Event::Closed { owner: settled });
    world.down(Request::Abort);
    assert!(world.up.is_empty() && world.io.is_empty());
    // Close already aborts connections; only the late binding still needs io.
    world.event(skein_io::Event::Connecting { owner: pending, socket: late_socket });
    assert_eq!(world.io.pop(), Some(skein_io::Request::Abort { entity: late_socket }));
    assert!(world.io.is_empty());
    assert!(world.host.connections.get(&settled).is_none());
    world.event(skein_io::Event::Closed { owner: pending });
    assert_eq!(world.up.pop(), Some(Event::Closed));
    assert!(world.up.is_empty());
}

#[test]
fn a_close_after_an_abort_changes_nothing() {
    let mut world = AbortHost::new();
    world.down(Request::Abort);

    world.down(Request::Close);

    assert!(world.up.is_empty() && world.io.is_empty());
    world.bind_all();
    world.take_aborts();
    assert!(world.up.is_empty());
    assert!(world.host.runs.is_empty());
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn closed_follows_an_abort_once() {
    let mut world = AbortHost::new();
    world.bind_all();
    world.down(Request::Abort);
    world.take_aborts();

    world.down(Request::Abort);
    assert!(world.up.is_empty() && world.io.is_empty());
    let (first, _) = *world.bindings.get(0).expect("first connection");
    let (last, _) = *world.bindings.get(1).expect("last connection");
    world.event(skein_io::Event::Closed { owner: first });
    assert!(world.up.is_empty());
    world.event(skein_io::Event::Closed { owner: last });
    assert_eq!(world.up.pop(), Some(Event::Closed));
    world.event(skein_io::Event::Closed { owner: last });
    world.down(Request::Abort);
    world.down(Request::Close);
    world.fire();
    assert!(world.up.is_empty() && world.io.is_empty());
    assert!(world.host.connections.is_empty());
    assert_eq!(world.host.next_deadline(), None);
}

#[test]
fn an_empty_host_aborts_once_without_waiting() {
    let mut world = AbortHost::empty();

    world.down(Request::Abort);

    assert_eq!(world.up.pop(), Some(Event::Closed));
    world.down(Request::Abort);
    world.fire();
    assert!(world.up.is_empty() && world.io.is_empty());
    assert_eq!(world.host.next_deadline(), None);
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
