use crate::{PartyOutput, engine, route, tests::fixtures as f};
use jig_client_protocol_wire::Record as _;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_domain as root;
use skein_io as io;
use skein_lib::{ReplyTo, Time, Token, Wall, length::len_u64};

fn listening(engine: &mut crate::Engine) {
    let addr = address();
    let clients = engine.clients.as_mut().expect("clients configured");
    assert!(clients.up(io::Event::Listening { owner: Token::new(4_u64 << 60), listener: Token::new(99), addr }));
}
#[test]
fn the_listener_opens_before_the_restart_begins() {
    let mut service = started(true);
    let engine = f::engine(&mut service);
    waiting_workers(engine);
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    assert!(engine.begin);
    assert!(engine.store.host.above.is_empty());
    take_listener(engine);
    listening(engine);
    before_restart_is_not_ready(engine);
    let workers = engine.workers.as_mut().expect("workers");
    assert!(workers.up(
        Time::ZERO,
        io::Event::Listening { owner: Token::new(1_u64 << 60), listener: Token::new(98), addr: address() },
        &mut engine.events
    ));
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    assert!(!engine.begin);
    let Some(jig_protocol_store::Event::Page { items, next, .. }) = engine.store.host.above.pop() else {
        panic!("restart load completed by the real store")
    };
    assert_eq!(items.len(), 0);
    assert_eq!(next, None);
}
#[test]
fn the_printed_link_reaches_the_caller_once() {
    let mut service = started(true);
    listening(f::engine(&mut service));
    f::turn(&mut service);
    let link = service.links().pop().expect("printed link");
    assert!(link.starts_with(b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret="));
    assert!(service.links().is_empty());
    f::turn(&mut service);
    assert!(service.links().is_empty());
}
const HOST_UPTIME: Time = Time::from_nanos(600_000_000_000);
const UNUSABLE_QUEUE_BYTES: u32 = 0;
#[test]
fn a_printed_link_survives_the_first_host_clock() {
    let mut service = prepared(true);
    service.clock(HOST_UPTIME, Wall::EPOCH);
    let engine = ready_client(&mut service);
    let body = link_body(engine);
    engine.clients.as_mut().expect("clients").iterate(
        HOST_UPTIME,
        Wall::EPOCH,
        &mut engine.events,
        &engine.env.limits.connector,
        engine.env.limits.core.documents.rows,
        &mut engine.accounts,
    );
    let mut peer = Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine);
    let Some(root::Event::Core(core::Event::SignIn { .. })) = engine.events.pop() else {
        panic!("the fresh printed link must reach the root on a host already up for ten minutes")
    };
}
#[test]
fn a_refused_clients_constructor_ends_the_first_iterate() {
    let mut service = prepared(true);
    f::engine(&mut service).clients.as_mut().expect("clients").env.limits.queued_bytes = UNUSABLE_QUEUE_BYTES;
    crate::iterate(&mut service, HOST_UPTIME, Wall::EPOCH);
    let mut problems = jig_protocol_config::Problems::empty();
    problems
        .push(jig_protocol_config::Path::root().member(b"engine").member(b"clients"), jig_protocol_config::What::Bound);
    assert_eq!(service.end(), Some(&crate::End::Stopped(crate::Stop::Start(problems))));
}
#[test]
fn with_no_clients_member_party_outputs_go_to_the_port() {
    let mut service = started(false);
    let engine = f::engine(&mut service);
    route::request(
        engine,
        root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply {
            to: ReplyTo::new(Token::new(23)),
            reply: people::Reply::SignedOut,
        })),
    );
    let Some(PartyOutput::Held(root::Delivery::Core(core::Held::PeopleReply { to, reply }))) = engine.outputs.pop()
    else {
        panic!("party reply")
    };
    assert_eq!((to.into_token(), reply), (Token::new(23), people::Reply::SignedOut));
    assert!(engine.outputs.is_empty());
}

const PEER_SOCKET: Token = Token::new(100);

struct Peer {
    socket: Token,
    wire: skein_world::stream::Wire,
    owner: Option<Token>,
    closed: u32,
}
impl Peer {
    fn new(engine: &mut crate::Engine, method: &[u8], path: &[u8], body: &[u8], bearer: Option<&[u8]>) -> Self {
        Self::at(engine, PEER_SOCKET, method, path, body, bearer)
    }
    fn at(
        engine: &mut crate::Engine,
        socket: Token,
        method: &[u8],
        path: &[u8],
        body: &[u8],
        bearer: Option<&[u8]>,
    ) -> Self {
        let header = match bearer {
            Some(value) => f::join(&[b"Authorization: Bearer ", value, b"\r\n"]),
            None => Box::default(),
        };
        let length = f::decimal(len_u64(body.len()));
        let bytes = f::join(&[
            method,
            b" ",
            path,
            b" HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nConnection: close\r\nContent-Type: application/octet-stream\r\n",
            &header,
            b"Content-Length: ",
            &length,
            b"\r\n\r\n",
            body,
        ]);
        Self::accept(engine, socket, &bytes)
    }
    fn request(engine: &mut crate::Engine, bytes: &[u8]) -> Self {
        Self::accept(engine, PEER_SOCKET, bytes)
    }
    fn accept(engine: &mut crate::Engine, socket: Token, bytes: &[u8]) -> Self {
        let mut wire = skein_world::stream::Wire::new(7);
        wire.write(bytes);
        let clients = engine.clients.as_mut().expect("clients configured");
        assert!(clients.up(io::Event::Accepted { owner: Token::new(4_u64 << 60), socket, peer: address() }));
        Self { socket, wire, owner: None, closed: 0 }
    }
    fn pump(&mut self, engine: &mut crate::Engine) {
        let clients = engine.clients.as_mut().expect("clients configured");
        for _ in 0_u32..256 {
            advance_client(clients, &mut engine.events, &engine.env.limits);
            while let Some(request) = clients.io.pop() {
                match request {
                    io::Request::Listen { owner, addr } => {
                        assert!(clients.up(io::Event::Listening { owner, listener: Token::new(99), addr }));
                    }
                    io::Request::Bind { socket, owner } => {
                        assert_eq!(socket, self.socket);
                        self.owner = Some(owner);
                    }
                    io::Request::Stream { stream, down } => {
                        assert_eq!(stream, self.socket);
                        self.wire.take(down);
                    }
                    io::Request::Close { .. } | io::Request::CloseAfterFlush { .. } | io::Request::Abort { .. } => {
                        self.closed = self.closed.checked_add(1).expect("socket close count");
                        assert!(clients.up(io::Event::Closed { owner: self.owner.expect("bound socket") }));
                    }
                    value @ (io::Request::Connect { .. }
                    | io::Request::Reject { .. }
                    | io::Request::Output { .. }
                    | io::Request::Spawn { .. }
                    | io::Request::SpawnAlone { .. }
                    | io::Request::Signal { .. }
                    | io::Request::Usage { .. }
                    | io::Request::Window { .. }) => panic!("unexpected client io {value:?}"),
                }
            }
            if let Some(owner) = self.owner
                && let Some(up) = self.wire.answer()
            {
                assert!(clients.up(io::Event::Stream { owner, up }));
            }
        }
    }
}
#[test]
fn shutdown_closes_a_call_still_waiting_for_its_body() {
    let mut service = started(true);
    let engine = ready_decisions(&mut service);
    let _link = engine.clients.as_mut().expect("clients").links.pop().expect("printed link");
    for _ in 0_u32..64 {
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    let mut peer = Peer::request(
        engine,
        b"POST /jig/v1/read HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nContent-Type: application/octet-stream\r\nContent-Length: 1\r\n\r\n",
    );
    peer.pump(engine);
    assert_eq!(peer.closed, 0, "the peer keeps the admitted request open");
    let clients = engine.clients.as_mut().expect("clients");
    assert!(clients.next_deadline().is_some(), "the open request has a stall deadline");
    clients.down(jig_protocol_engine_clients::Event::Stop);
    clients.server_requests();
    let Some(io::Request::Close { entity }) = clients.io.pop() else { panic!("listener close") };
    assert_eq!(entity, Token::new(99));
    clients.down(jig_protocol_engine_clients::Event::Close);
    peer.pump(engine);
    assert_eq!(peer.closed, 1, "shutdown ends the call without waiting for its body or stall deadline");
    let clients = engine.clients.as_mut().expect("clients");
    assert!(!clients.closed, "the listener's close completion is still delayed");
    assert!(!clients.work_pending(Time::ZERO), "waiting for io leaves the loop idle");
    assert_eq!(clients.next_deadline(), None);
    assert!(clients.up(io::Event::Closed { owner: Token::new(4_u64 << 60) }));
    peer.pump(engine);
    assert!(engine.clients.as_ref().expect("clients").closed);
}

fn address() -> io::Addr {
    io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into())
}
fn ready_client(service: &mut crate::Service) -> &mut crate::Engine {
    let engine = f::engine(service);
    let clients = engine.clients.as_mut().expect("clients");
    clients.advance(
        &mut engine.events,
        &engine.env.limits.connector,
        engine.env.limits.core.documents.rows,
        &mut engine.accounts,
    );
    take_listener(engine);
    listening(engine);
    engine.clients.as_mut().expect("clients").starting = None;
    engine
}
fn link_body(engine: &mut crate::Engine) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    use skein_lib::{
        Writer,
        base64::{self, Alphabet},
    };
    let link = engine.clients.as_mut().expect("clients").links.pop().expect("printed link");
    let at = skein_lib::bytes::find(&link, b"secret=").expect("query").checked_add(7).expect("query start");
    let secret = base64::decode(Alphabet::Url, link.get(at..).expect("secret bytes"), 32).expect("secret");
    let value = w::LinkSignIn { secret: skein_lib::Secret::new(secret) };
    let mut writer = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured record");
    writer.finish()
}
#[test]
fn a_party_s_reply_reaches_the_clients_component() {
    use jig_protocol_engine_clients::{SecretOwner, SecretsRequest};
    let mut service = started(true);
    let engine = ready_client(&mut service);
    let body = link_body(engine);
    let mut peer = Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine);
    let Some(root::Event::Core(core::Event::SignIn { reply_to, .. })) = engine.events.pop() else {
        panic!("sign-in routed to root")
    };
    route::request(
        engine,
        root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply {
            to: reply_to,
            reply: people::Reply::SignedIn { sign_in: 41, person: 1, expires: Wall::from_nanos(100) },
        })),
    );
    peer.pump(engine);
    assert!(engine.outputs.is_empty());
    let Some(SecretsRequest::Keep { owner, .. }) = engine.clients.as_mut().expect("clients").secrets.pop() else {
        panic!("digest keep")
    };
    assert_eq!(owner, SecretOwner::SignIn(41));
}

fn signed_stream(engine: &mut crate::Engine) -> Peer {
    let (bearer, kept) = credential();
    let mut peer = Peer::new(engine, b"GET", b"/jig/v1/stream", &[], Some(&bearer));
    peer.pump(engine);
    authenticate(engine, &mut peer, kept);
    assert!(skein_lib::bytes::find(&peer.wire.received, b"text/event-stream").is_some());
    peer
}
fn ready_decisions(service: &mut crate::Service) -> &mut crate::Engine {
    let engine = ready_client(service);
    decisions_open(engine);
    engine
}
#[test]
fn an_erased_sign_in_record_ends_its_stream_once_durable() {
    use jig_protocol_engine_clients::{SecretOwner, SecretsRequest};
    let mut service = started(true);
    let engine = ready_client(&mut service);
    let mut peer = signed_stream(engine);
    let mut writes = skein_lib::Queue::with_capacity(1);
    writes.push(root::Write::Erase(root::Key::Core(core::Key::People(people::Key::SignIn(41)))));
    engine.clients.as_mut().expect("clients").commit(57, &writes);
    peer.pump(engine);
    assert_eq!(peer.closed, 0);
    assert!(engine.clients.as_mut().expect("clients").secrets.is_empty());
    engine.clients.as_mut().expect("clients").committed(57);
    peer.pump(engine);
    assert_eq!(peer.closed, 1);
    let clients = engine.clients.as_mut().expect("clients");
    let Some(SecretsRequest::Erase { owner }) = clients.secrets.pop() else { panic!("digest erase") };
    assert_eq!(owner, SecretOwner::SignIn(41));
    clients.committed(57);
    assert!(clients.secrets.is_empty());
}

/// `ending_commit_durable`: endings cross the one-ending output carrier in
/// commit order, while an unacknowledged root decision holds the watermark.
/// The clients' three-commit table is exercised across successive drains.
#[test]
fn sign_in_endings_follow_their_own_commits_under_load() {
    use jig_protocol_engine_clients::{SecretOwner, SecretsRequest};
    const WATERMARK: u32 = 1; // One writing period holds admission shut.
    const SETTLE: u32 = 8; // The service fixtures' startup settlement turns.
    const ENDINGS_PER_COMMIT: u32 = 1; // One erased sign-in in each scripted commit.
    const FIRST_COMMIT: u64 = 57;
    const SECOND_COMMIT: u64 = 58;
    const LAST_COMMIT: u64 = 59;
    const FIRST_SIGN_IN: u64 = 41;
    const SECOND_SIGN_IN: u64 = 42;
    const LAST_SIGN_IN: u64 = 43;
    let mut service = started(true);
    let engine = ready_decisions(&mut service);
    for _ in 0..SETTLE {
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    engine.env.limits.watermark = WATERMARK;
    for number in [1, 2] {
        engine.step(root::Event::Core(core::Event::Period { project: 1, period: number, budget: 100, next: None }));
    }
    assert!(engine.blocked, "the unacknowledged writing period holds the next period at the watermark");
    let clients = engine.clients.as_mut().expect("clients");
    let mut writes = skein_lib::Queue::with_capacity(ENDINGS_PER_COMMIT);
    for (commit, sign_in) in
        [(FIRST_COMMIT, FIRST_SIGN_IN), (SECOND_COMMIT, SECOND_SIGN_IN), (LAST_COMMIT, LAST_SIGN_IN)]
    {
        writes.push(root::Write::Erase(root::Key::Core(core::Key::People(people::Key::SignIn(sign_in)))));
        clients.commit(commit, &writes);
        let _write = writes.pop().expect("one ending per commit");
    }
    advance_client(clients, &mut engine.events, &engine.env.limits);
    assert!(clients.secrets.is_empty(), "no ending is durable yet");
    clients.committed(SECOND_COMMIT);
    advance_client(clients, &mut engine.events, &engine.env.limits);
    for sign_in in [FIRST_SIGN_IN, SECOND_SIGN_IN] {
        let Some(SecretsRequest::Erase { owner }) = clients.secrets.pop() else { panic!("durable ending") };
        assert_eq!(owner, SecretOwner::SignIn(sign_in), "endings leave in commit order");
    }
    advance_client(clients, &mut engine.events, &engine.env.limits);
    assert!(clients.secrets.is_empty(), "the last ending waits for its own commit");
    clients.committed(LAST_COMMIT);
    advance_client(clients, &mut engine.events, &engine.env.limits);
    let Some(SecretsRequest::Erase { owner }) = clients.secrets.pop() else { panic!("last durable ending") };
    assert_eq!(owner, SecretOwner::SignIn(LAST_SIGN_IN));
    for commit in [FIRST_COMMIT, SECOND_COMMIT, LAST_COMMIT] {
        clients.committed(commit);
        advance_client(clients, &mut engine.events, &engine.env.limits);
        assert!(clients.secrets.is_empty(), "each ending leaves once despite duplicate acknowledgements");
    }
}

fn take_listener(engine: &mut crate::Engine) {
    let limits = engine.env.limits;
    engine.clients.as_mut().expect("clients").iterate(
        engine.env.now,
        engine.env.wall,
        &mut engine.events,
        &limits.connector,
        limits.core.documents.rows,
        &mut engine.accounts,
    );
    let Some(io::Request::Listen { .. }) = engine.clients.as_mut().expect("clients").io.pop() else {
        panic!("clients listener")
    };
}

fn started(clients: bool) -> crate::Service {
    let mut service = prepared(clients);
    service.clock(Time::ZERO, Wall::EPOCH);
    service
}
fn prepared(clients: bool) -> crate::Service {
    prepared_over(clients, None)
}
fn prepared_over(clients: bool, stores: Option<jig_protocol_store::Stores>) -> crate::Service {
    let (mut config, [first, second]) = crate::startup::fixture_connectors(&f::engine_document());
    let core = &mut config.limits.core;
    core.tasks.tasks = 2;
    core.tasks.exemptions = 2;
    core.people.operations = 2;
    core.tasks.project_tasks = 2;
    core.tasks.tree_tasks = 2;
    core.tasks.batch = 1;
    core.tasks.holdings = 2;
    core.tasks.run_inbox = 2;
    core.people.people = 2;
    core.people.sign_ins = 2;
    core.people.pending = 2;
    core.people.waiters = 2;
    core.people.inbox_tasks = 4;
    core.fleet.workers = 2;
    core.fleet.attempts = 2;
    core.fleet.calls = 2;
    core.views.watchers = 4;
    core.views.subjects_per_change = 4;
    core.documents.rows = 2;
    config.limits.journal = jig_protocol_config::JournalSizes { commits: 3, writes: 2000, door: 32, release: 32 };
    config.limits.routes = 16;
    let mut limits = crate::limits::small(&config.limits);
    limits.root.journal = f::journal(&limits.root);
    limits.root.watermark = root::default_watermark(&limits.root.journal);
    config.limits.journal = jig_protocol_config::JournalSizes {
        commits: limits.root.journal.commits,
        writes: limits.root.journal.writes,
        door: limits.root.journal.door,
        release: limits.root.journal.release,
    };
    assert!(root::valid(&limits.root), "test root limits: {:?}", limits.root);
    let seeds = f::seeds();
    let root = root::Config { core: jig_protocol_config::core(&config, seeds.deployment, seeds.seed), first, second };
    let clients = if clients {
        Some(
            crate::clients::Clients::new(
                &config,
                address(),
                &seeds,
                &crate::limits::small(&jig_protocol_config_fixtures::small()).server,
            )
            .expect("clients fixture"),
        )
    } else {
        None
    };
    let values = crate::values::Values {
        application: skein_lib::Map::with_capacity(0),
        accounts: jig_protocol_accounts::table::Table::new(1, 256),
    };
    let connector = limits.root.connector;
    let systems = [
        crate::system::System::new(connector.resources, connector.made),
        crate::system::System::new(connector.resources, connector.made),
    ];
    let store = crate::store::Store::new(
        &limits.root,
        &crate::store::limits(&limits.root, &limits.store).expect("fixture store admission"),
        &root.core.settings.models,
        f::seeds().installation,
        f::seeds().seed,
    );
    let children = engine::Children { accounts: None, workers: None, clients };
    let mut engine = crate::Engine::new(root, &limits.root, systems, children, values, store);
    match stores {
        Some(stores) => f::open_memory_store(&mut engine, stores),
        None => f::memory_store(&mut engine),
    }
    crate::Service::new(Some(engine), skein_lib::List::with_capacity(0), limits.io, 512)
}

fn before_restart_is_not_ready(engine: &mut crate::Engine) {
    use jig_client_protocol_wire as w;
    engine.clients.as_mut().expect("clients").starting = Some(());
    let body = link_body(engine);
    let mut peer = Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    peer.pump(engine);
    assert_eq!(engine.clients.as_ref().expect("clients").early.len(), 1, "sign-in is staged before restart");
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    peer.pump(engine);
    assert!(engine.begin, "the unopened worker listener still prevents restart");
    assert!(engine.store.host.above.is_empty(), "the refused sign-in made no commit");
    let value = w::SignInAnswer::decode(
        &engine.clients.as_ref().expect("clients").env.limits.wire,
        &mut skein_lib::Reader::new(response_body(&peer)),
    )
    .expect("sign-in answer");
    assert_eq!(value, w::SignInAnswer::NotReady);
}

const RESTART_TURNS: u32 = 64; // The real-store restart fixture's settlement budget.

fn restore_people(engine: &mut crate::Engine) {
    for _ in 0..RESTART_TURNS {
        crate::store::answer(engine);
        if engine.domain.core_mut().start_phase() == Some(core::start::Phase::Sweep) {
            return;
        }
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    panic!("the real store restores people before decisions open");
}

fn withhold_store(engine: &mut crate::Engine, held: &mut skein_lib::Queue<jig_protocol_store::Event>) {
    while let Some(event) = engine.store.host.above.pop() {
        held.push(event);
    }
}

fn sign_in_answer(engine: &crate::Engine, peer: &Peer) -> jig_client_protocol_wire::SignInAnswer {
    assert!(peer.wire.received.starts_with(b"HTTP/1.1 200 "), "HTTP answer: {:?}", peer.wire.received);
    jig_client_protocol_wire::SignInAnswer::decode(
        &engine.clients.as_ref().expect("clients").env.limits.wire,
        &mut skein_lib::Reader::new(response_body(peer)),
    )
    .expect("sign-in answer")
}

#[test]
fn sign_in_is_not_ready_until_restart_decisions_open() {
    use jig_client_protocol_wire as w;
    let mut service = started(true);
    let engine = ready_client(&mut service);
    let body = link_body(engine);
    restore_people(engine);
    // One withheld commit is enough to hold ordinary admission at its watermark.
    engine.env.limits.watermark = 1;
    let mut held = skein_lib::Queue::uncapped();
    withhold_store(engine, &mut held);
    let mut peer = Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    for _ in 0..RESTART_TURNS {
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
        peer.pump(engine);
        withhold_store(engine, &mut held);
    }
    assert!(!engine.begin, "StartBegin was submitted");
    assert!(!engine.domain.ready(), "remaining store completions still hold decisions closed");
    assert!(!held.is_empty(), "store completions remain withheld");
    assert_eq!(sign_in_answer(engine, &peer), w::SignInAnswer::NotReady);
    while let Some(event) = held.pop() {
        engine.store.host.above.push(event);
    }
    decisions_open(engine);
    let mut peer = Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    for _ in 0..RESTART_TURNS {
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
        peer.pump(engine);
    }
    let w::SignInAnswer::SignedIn(_) = sign_in_answer(engine, &peer) else {
        panic!("the same printed-link sign-in succeeds after decisions open");
    };
}

fn before_wires_ack()
-> (crate::Service, jig_client_protocol_wire::SignedIn, skein_lib::Queue<jig_protocol_store::Event>) {
    use jig_client_protocol_wire as w;
    let mut first = started(true);
    let engine = ready_decisions(&mut first);
    let body = link_body(engine);
    let mut peer = Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    for _ in 0..RESTART_TURNS {
        tick_peer(engine, &mut peer);
    }
    let w::SignInAnswer::SignedIn(signed_in) = sign_in_answer(engine, &peer) else { panic!("durable sign-in") };
    let stores = engine.store.host.component.fork_memory().expect("copy of the durable stores");
    let mut restart = prepared_over(true, Some(stores));
    restart.clock(Time::ZERO, Wall::EPOCH);
    let engine = ready_client(&mut restart);
    let _printed = engine.clients.as_mut().expect("clients").links.pop().expect("printed link");
    for _ in 0..RESTART_TURNS {
        // The digest can be checked once Wires is durable. The root still
        // needs its commit terminal to finish the restart.
        engine.store.iterate();
        if engine.store.host.mark_durable() {
            assert!(!engine.domain.ready());
            let mut held = skein_lib::Queue::uncapped();
            withhold_store(engine, &mut held);
            assert!(!held.is_empty(), "the root still needs the Wires acknowledgement");
            return (restart, signed_in, held);
        }
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    }
    panic!("restart made its Wires commit durable");
}

fn resume_restart(engine: &mut crate::Engine, mut held: skein_lib::Queue<jig_protocol_store::Event>) {
    while let Some(event) = held.pop() {
        engine.store.host.above.push(event);
    }
    decisions_open(engine);
}

fn versioned_stream(engine: &mut crate::Engine, bearer: &[u8]) -> Peer {
    let version = f::decimal(u64::from(jig_client_protocol_wire::VERSION));
    let path = f::join(&[b"/jig/v1/stream?version=", &version]);
    let mut peer = Peer::new(engine, b"GET", &path, &[], Some(bearer));
    tick_peer(engine, &mut peer);
    peer
}

fn tick_peer(engine: &mut crate::Engine, peer: &mut Peer) {
    peer.pump(engine);
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    peer.pump(engine);
}

fn first_stream_event(engine: &crate::Engine, peer: &Peer) -> jig_client_protocol_wire::StreamEvent {
    use skein_lib::base64::{self, Alphabet};
    assert!(peer.wire.received.starts_with(b"HTTP/1.1 200 "), "HTTP answer: {:?}", peer.wire.received);
    let body = response_body(peer);
    let at = skein_lib::bytes::find(body, b"data: ").expect("first event");
    let data = body.get(at.saturating_add(b"data: ".len())..).expect("event data");
    let end = skein_lib::bytes::find(data, b"\n\n").expect("complete event");
    let limits = &engine.clients.as_ref().expect("clients").env.limits;
    let record = base64::decode(Alphabet::Standard, data.get(..end).expect("base64 line"), limits.message_bytes)
        .expect("event record");
    jig_client_protocol_wire::StreamEvent::decode(&limits.wire, &mut skein_lib::Reader::new(&record))
        .expect("decoded first event")
}

#[test]
fn a_stream_waits_for_decisions_then_opens_with_me() {
    use jig_client_protocol_wire as w;
    let (mut service, signed_in, held) = before_wires_ack();
    let engine = f::engine(&mut service);
    let bearer = bearer(signed_in.token.as_ref().expect("native token").expose());
    let mut peer = versioned_stream(engine, &bearer);
    for _ in 0..RESTART_TURNS {
        if !engine.clients.as_ref().expect("clients").work_pending(Time::ZERO) {
            break;
        }
        tick_peer(engine, &mut peer);
    }
    assert!(!engine.domain.ready());
    assert!(peer.wire.received.is_empty(), "no response before decisions open: {:?}", peer.wire.received);
    assert_eq!(peer.closed, 0, "the opening stream stays connected");
    assert!(!engine.clients.as_ref().expect("clients").work_pending(Time::ZERO), "a held watch alone is not runnable");

    resume_restart(engine, held);
    assert!(engine.work_pending(Time::ZERO), "decisions opening wakes the held watch");
    for _ in 0..RESTART_TURNS {
        tick_peer(engine, &mut peer);
    }
    let w::StreamEvent::Opened(opened) = first_stream_event(engine, &peer) else { panic!("hello with me") };
    let limits = &engine.clients.as_ref().expect("clients").env.limits.wire;
    let value = w::DeliveryValue::decode(limits, &mut skein_lib::Reader::new(&opened.me.record)).expect("me record");
    let w::DeliveryValue::Me(me) = value else { panic!("me document") };
    assert_eq!(me.party.number, signed_in.party);
    assert_eq!(me.sign_in_expires, signed_in.expires);
}

#[test]
fn a_stream_gone_at_restart_never_submits_its_watch() {
    let (mut service, signed_in, held) = before_wires_ack();
    let engine = f::engine(&mut service);
    let bearer = bearer(signed_in.token.as_ref().expect("native token").expose());
    let mut peer = versioned_stream(engine, &bearer);
    tick_peer(engine, &mut peer);
    let clients = engine.clients.as_mut().expect("clients");
    let Some(root::Event::Core(core::Event::Watch { .. })) = clients.startup_frames.iter().next() else {
        panic!("the opening watch is waiting at the gate");
    };
    assert!(clients.up(io::Event::Stream {
        owner: peer.owner.expect("bound stream"),
        up: skein_lib::stream::Up::Failed(skein_lib::stream::Fault::Other),
    }));
    peer.pump(engine);
    assert_eq!(peer.closed, 1, "the peer went away while decisions were closed");
    assert!(!engine.domain.ready());
    let clients = engine.clients.as_ref().expect("clients");
    assert!(clients.startup_frames.is_empty(), "the gone peer's watch is no longer held");
    assert!(clients.early.is_empty(), "the matching unwatch was consumed, not staged");
    assert!(engine.events.is_empty(), "neither input was queued for the root");

    resume_restart(engine, held);
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    assert_eq!(engine.domain.core().views.watchers(), 0, "no watch left in the core");
}

#[test]
fn a_non_frame_watch_is_refused_busy_while_starting() {
    use jig_client_protocol_wire as w;
    const WATCH_SOCKET: Token = Token::new(101);
    const WATCH_NAME: u64 = 1;
    let (mut service, signed_in, _held) = before_wires_ack();
    let engine = f::engine(&mut service);
    let bearer = bearer(signed_in.token.as_ref().expect("native token").expose());
    let mut stream = Peer::new(engine, b"GET", b"/jig/v1/stream", &[], Some(&bearer));
    tick_peer(engine, &mut stream);
    let w::StreamEvent::Hello(hello) = first_stream_event(engine, &stream) else {
        panic!("expected a hello event first")
    };
    let limits = engine.clients.as_ref().expect("clients").env.limits.wire;
    let value = w::Open {
        stream: hello.stream,
        watch: WATCH_NAME,
        subject: w::Owned {
            owner: w::Owner::Jig,
            version: w::VERSION,
            record: w::Subject::Deployment.encode(&limits).expect("deployment subject"),
        },
    };
    let body = value.encode(&limits).expect("watch request");
    let mut peer = Peer::at(engine, WATCH_SOCKET, b"POST", b"/jig/v1/watch", &body, Some(&bearer));
    tick_peer(engine, &mut peer);
    tick_peer(engine, &mut peer);
    assert!(!engine.domain.ready());
    let answer =
        w::OpenAnswer::decode(&limits, &mut skein_lib::Reader::new(response_body(&peer))).expect("watch answer");
    assert_eq!(answer, w::OpenAnswer::Refused(w::OpenAnswerRefused { why: w::WatchRefusal::Busy }));
}

fn waiting_workers(engine: &mut crate::Engine) {
    let config = f::config();
    assert!(engine.events.is_empty());
    engine.workers = Some(
        crate::workers::Workers::new(
            &config,
            f::seeds().deployment,
            &crate::limits::small(&config.limits).workers,
            None,
        )
        .expect("workers fixture"),
    );
}

fn credential() -> (Box<[u8]>, jig_protocol_engine_clients::Kept) {
    use skein_lib::{Secret, Writer};
    let mut token = Writer::new(40);
    token.put(&41_u64.to_be_bytes()).expect("number");
    token.put(&[7; 32]).expect("random bytes");
    let token = token.finish();
    let mut digest = Writer::new(32);
    for word in io::digest::digest(&token).0 {
        digest.put(&word.to_be_bytes()).expect("digest word");
    }
    (
        bearer(&token),
        jig_protocol_engine_clients::Kept { digest: Secret::new(digest.finish()), expires: Wall::from_nanos(100) },
    )
}

fn bearer(token: &[u8]) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let size = u32::try_from(token.len()).expect("token size");
    let encoded = base64::encoded_len(Alphabet::Url, size).expect("encoded bearer size");
    let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(encoded));
    base64::encode(Alphabet::Url, token, &mut out).expect("bearer bytes");
    out.finish()
}

fn authenticate(engine: &mut crate::Engine, peer: &mut Peer, kept: jig_protocol_engine_clients::Kept) {
    use jig_protocol_engine_clients::{Event, SecretOwner, SecretsAnswer, SecretsRequest};
    let clients = engine.clients.as_mut().expect("clients");
    let Some(SecretsRequest::Get { owner }) = clients.secrets.pop() else { panic!("digest get") };
    assert_eq!(owner, SecretOwner::SignIn(41));
    clients.down(Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::SignIn(41), value: Some(kept) }));
    peer.pump(engine);
}
fn response_body(peer: &Peer) -> &[u8] {
    let at = skein_lib::bytes::find(&peer.wire.received, b"\r\n\r\n")
        .expect("HTTP head")
        .checked_add(4)
        .expect("body start");
    peer.wire.received.get(at..).expect("response body")
}
fn application_read(_engine: &crate::Engine) -> Box<[u8]> {
    use jig_client_protocol_wire as w;
    let owner = w::Owner::Family(w::Family { number: 1 });
    let window = w::Window { rows: 2, after: None };
    let value = w::FamilyRecord {
        scope: w::Scope::Deployment,
        window,
        record: jig_client_test_protocol_wire::codec::read(&jig_test_connector::ClientSubject::Deployment),
    };
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured family record");
    let record = w::Owned { owner, version: jig_client_test_protocol_wire::VERSION, record: writer.finish() };
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(
        record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured read");
    writer.finish()
}
#[test]
fn a_core_refusal_finishes_an_application_read() {
    use jig_client_protocol_wire as w;
    let mut service = started(true);
    let engine = ready_client(&mut service);
    let body = application_read(engine);
    let (bearer, kept) = credential();
    for (label, why, expected) in [
        ("busy", core::reads::Why::Busy, w::ReadNone::Busy),
        ("signed out", core::reads::Why::SignedOut, w::ReadNone::SignedOut),
    ] {
        let mut peer = Peer::new(engine, b"POST", b"/jig/v1/read", &body, Some(&bearer));
        peer.pump(engine);
        authenticate(engine, &mut peer, kept.clone());
        let Some(root::Event::ReadApplication { reply, .. }) = engine.events.pop() else {
            panic!("{label}: application read")
        };
        route::request(
            engine,
            root::Request::Deliver(root::Delivery::Core(core::Held::Read {
                reply,
                answer: core::reads::Answer::None(why),
            })),
        );
        peer.pump(engine);
        let limits = &engine.clients.as_ref().expect("clients").env.limits.wire;
        let answer =
            w::ReadAnswer::decode(limits, &mut skein_lib::Reader::new(response_body(&peer))).expect("read answer");
        let w::ReadAnswer::None(answer) = answer else { panic!("{label}: refusal") };
        assert_eq!(answer.value, expected, "{label}");
        assert_eq!(peer.closed, 1, "{label}");
    }
}

#[test]
fn durable_sign_in_endings_drain_in_commit_order() {
    use jig_protocol_engine_clients::{SecretOwner, SecretsRequest};
    let mut service = started(true);
    let engine = ready_client(&mut service);
    let clients = engine.clients.as_mut().expect("clients");
    let mut writes = skein_lib::Queue::with_capacity(2);
    for number in [41, 42] {
        writes.push(root::Write::Erase(root::Key::Core(core::Key::People(people::Key::SignIn(number)))));
    }
    clients.commit(57, &writes);
    clients.committed(57);
    advance_client(clients, &mut engine.events, &engine.env.limits);
    for (label, number) in [("first ending", 41), ("second ending", 42)] {
        let Some(SecretsRequest::Erase { owner }) = clients.secrets.pop() else { panic!("{label}: erase") };
        assert_eq!(owner, SecretOwner::SignIn(number), "{label}");
    }
    clients.committed(57);
    advance_client(clients, &mut engine.events, &engine.env.limits);
    assert!(clients.secrets.is_empty(), "each ending was sent once");
}

fn advance_client(
    clients: &mut crate::clients::Clients,
    events: &mut skein_lib::Queue<root::Event>,
    limits: &root::Limits,
) {
    clients.iterate(Time::ZERO, Wall::EPOCH, events, &limits.connector, limits.core.documents.rows, &mut None);
}

#[test]
fn a_kept_digest_reaches_the_store_s_secrets() {
    use jig_protocol_engine_clients::{SecretOwner, SecretsRequest};
    use jig_protocol_store::{Event, Secrets};
    let mut service = started(true);
    let engine = ready_decisions(&mut service);
    let (bearer, value) = credential();
    let expected = jig_protocol_store::SecretValue::Digest {
        expiry: value.expires.as_nanos(),
        digest: value.digest.expose().try_into().expect("credential digest"),
    };
    engine
        .clients
        .as_mut()
        .expect("clients")
        .secrets
        .push(SecretsRequest::Keep { owner: SecretOwner::SignIn(41), value });
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    let answer = engine
        .store
        .host
        .secrets(Token::new(77), Secrets::Get { owner: jig_protocol_store::SecretOwner::SignIn, number: 41 });
    let Some(Event::Secret { owner, value: Some(value) }) = answer else {
        panic!("same-pass stored digest get: {answer:?}")
    };
    assert_eq!(owner, Token::new(77));
    assert_eq!(value, expected);
    let mut peer = Peer::new(engine, b"GET", b"/jig/v1/stream", &[], Some(&bearer));
    peer.pump(engine);
    assert_eq!(engine.clients.as_ref().expect("clients").secrets.len(), 1, "credential get staged");
    engine::iterate(engine, Time::ZERO, Wall::EPOCH);
    assert!(engine.clients.as_ref().expect("clients").secrets.is_empty(), "get answered in this iterate");
    peer.pump(engine);
    assert!(skein_lib::bytes::find(&peer.wire.received, b"text/event-stream").is_some());
    assert!(engine.outputs.is_empty(), "the digest reached no parties port");
}

fn decisions_open(engine: &mut crate::Engine) {
    for _ in 0_u32..64 {
        engine::iterate(engine, Time::ZERO, Wall::EPOCH);
        if engine.domain.ready() {
            break;
        }
    }
    assert!(engine.domain.ready(), "real store restore and root restart completed");
    assert_eq!(engine.end, None);
}

#[test]
fn the_printed_url_carries_the_secret_the_component_gave() {
    let listener = ::core::net::SocketAddr::new(::core::net::IpAddr::from([127, 0, 0, 1]), 7100);
    let secret = skein_lib::Secret::new(Box::new([0; 32]));
    assert_eq!(
        crate::clients::printed(listener, &secret).as_ref(),
        b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    );
}

#[test]
fn a_call_past_the_mounts_room_is_answered_busy() {
    use skein_lib::Queue;
    let mut clients = f::server::retained_clients();
    let capacity = crate::limits::small(&jig_protocol_config_fixtures::small()).clients_http.calls;
    let mut peers = skein_lib::List::with_capacity(capacity.checked_add(1).expect("first refused call"));
    // The existing Small server has eight connections for four mounted calls.
    // The first socket is the server fixture's existing socket.
    for number in 0..=capacity {
        if number == capacity {
            let (_, owner, _) = peers.as_slice().first().expect("retained call at the mount bound");
            clients.server.up(io::Event::Closed { owner: *owner }, &mut clients.server_out);
            clients.server_requests();
        }
        let socket = Token::new(f::server::SOCKET.raw() + u64::from(number));
        let (owner, call) = f::server::retained_call(&mut clients.server, &mut clients.server_out, socket);
        peers.push((socket, owner, call)).expect("fixture calls");
        clients.server_requests();
    }
    assert_eq!(clients.refused.len(), 1);
    let (response, closes) = f::server::retained_response(&mut clients, &peers);
    assert!(response.as_slice().starts_with(b"HTTP/1.1 503 "), "{response:?}");
    let head_end = skein_lib::bytes::find(response.as_slice(), b"\r\n\r\n").expect("response head");
    assert_eq!(response.as_slice().get(head_end + 4..), Some(b"".as_slice()));
    assert_eq!(closes, 1);
    assert!(clients.refused.is_empty());
    assert_eq!(clients.mount.room(), 0);
    // Retired connections leave their mounted calls until the mount handles the queued ends.
    for (_, owner, _) in peers.iter().take(skein_lib::length::usize_of(capacity)) {
        clients.server.up(io::Event::Closed { owner: *owner }, &mut clients.server_out);
    }
    clients.server_requests();
    assert_eq!(clients.mount.room(), 0);
    let env = skein_lib::Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: crate::limits::small(&jig_protocol_config_fixtures::small()).clients_http,
    };
    let mut out = Queue::with_capacity(capacity.checked_mul(2).expect("retirement output"));
    while clients.mount.up(&env, &mut out) {}
    assert_eq!(clients.mount.room(), capacity);
}

#[test]
fn a_moved_record_bound_past_the_wire_value_bound_refuses_start() {
    let (mut config, _) = crate::startup::fixture_connectors(&f::engine_document());
    config.limits.driver.client.document.record_bytes = config.limits.clients.wire.keyed_value_some;
    let Err(problems) = crate::clients::Clients::new(
        &config,
        address(),
        &f::seeds(),
        &crate::limits::small(&jig_protocol_config_fixtures::small()).server,
    ) else {
        panic!("moved file does not fit the wire")
    };
    assert_eq!(problems.problems.get(0).expect("bound refusal").what, jig_protocol_config::What::Bound);
}

#[test]
fn debug_of_the_clients_shows_the_links_count_never_a_link() {
    const CANARY: &str = "http://127.0.0.1:7100/jig/v1/sign-in/link?secret=canary-jig-links-84d2f6";
    let (config, _) = crate::startup::fixture_connectors(&f::engine_document());
    let mut clients = crate::clients::Clients::new(
        &config,
        address(),
        &f::seeds(),
        &crate::limits::small(&jig_protocol_config_fixtures::small()).server,
    )
    .expect("clients fixture");
    clients.links.push(Box::from(CANARY.as_bytes()));

    let debug = alloc::fmt::format(format_args!("{clients:?}"));
    let canary_bytes = alloc::fmt::format(format_args!("{:?}", CANARY.as_bytes()));
    assert!(!debug.contains(CANARY), "Debug hides the printed link's text");
    assert!(!debug.contains(canary_bytes.as_str()), "Debug hides the printed link's bytes");
    assert!(debug.contains("links_entries: 1"), "Debug shows the printed link count");
}
