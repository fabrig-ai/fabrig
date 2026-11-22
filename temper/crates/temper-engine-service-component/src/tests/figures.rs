//! A running engine gives restored and changed figures (temper/docs/design/protocol/configuration/figures.md, 9).
use super::{
    fixtures,
    limit_settings::{admission as limits, item},
};
use crate::{FiguresRevision, engine::Engine};
use jig_domain_core::settings;
use jig_domain_people::{self as people, Reply};
use jig_protocol_worker_link as link;
use jig_protocol_worker_wire::{self as wire, Opening};
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{Duration, Intake, List, Queue, ReplyTo, Secret, Time, Token, stream};
use temper_client_protocol_wire::v1 as s;
use temper_client_protocol_wire::v1::Record;
use temper_engine_domain as root;
use temper_engine_domain_forge::settings as own;

const SOCKET: Token = Token::new(901);
const CONNECTION: Token = Token::new(902);
const NOW: Time = Time::from_nanos(1);
const WORKER_SECRET: &[u8] = b"worker-secret";

struct Peer {
    worker: link::Component,
    out: Queue<link::Request>,
    to_engine: Intake,
    to_worker: Intake,
    engine_read: Option<stream::Read>,
    worker_read: Option<stream::Read>,
    owner: Option<Token>,
    figures: List<wire::Figures>,
    ended: bool,
    hello: bool,
    send_hello: bool,
}
impl Peer {
    fn open(engine: &mut Engine, opening: Opening) -> Self {
        Self::open_with_hello(engine, opening, true)
    }
    fn open_with_hello(engine: &mut Engine, opening: Opening, send_hello: bool) -> Self {
        let mut settings = fixtures::start_settings(fixtures::engine(), &[(1, 100)]);
        if let Some(state) = engine.limits_in_force() {
            settings.jig.limits = state.in_force.jig;
            settings.limits = state.in_force.temper;
        }
        let limits = crate::workers::limits(&settings).unwrap();
        let worker_limits = link::Limits {
            sizes: wire::sizes::Sizes { slots: 1, ..limits.sizes },
            chunk: limits.chunk,
            room: limits.room,
            handshake: limits.handshake,
            ping: limits.ping,
            silence: limits.silence,
            stall: limits.stall,
            backoff: Duration::from_millis(100),
            ceiling: Duration::from_secs(2),
            proved: Duration::from_secs(2),
            grace: Duration::from_secs(10),
            busy: Duration::from_millis(100),
        };
        let worker = link::Component::new(
            &worker_limits,
            wire::spoken(),
            link::Credential {
                opening,
                name: b"worker-a".as_slice().into(),
                secret: Secret::new(WORKER_SECRET.into()),
                application: temper_protocol_worker_wire::application(
                    &crate::translate::limits(&settings.limits).unwrap(),
                )
                .unwrap(),
            },
            7,
        )
        .unwrap();
        let mut peer = Self {
            worker,
            out: Queue::uncapped(),
            to_engine: Intake::with_capacity(limits.room),
            to_worker: Intake::with_capacity(limits.room),
            engine_read: None,
            worker_read: None,
            owner: None,
            figures: List::with_capacity(3),
            ended: false,
            hello: false,
            send_hello,
        };
        engine.workers.up(
            NOW,
            io::Event::Accepted {
                owner: Token::new(900),
                socket: SOCKET,
                peer: io::Addr::Ip(([127, 0, 0, 1], 8000).into()),
            },
            &mut engine.events,
        );
        if engine.limits_in_force().is_none() {
            let mut workers = fixtures::opening_workers();
            workers[0].digest = Some(skein_lib::sha256::digest(WORKER_SECRET));
            engine.workers.down(NOW, jig_protocol_engine_workers::Event::Workers(workers), &mut engine.events);
        }
        peer.worker.fire(NOW, &mut peer.out);
        peer.settle(engine);
        peer
    }
    fn worker_lower(&mut self, event: LowerEvent) {
        self.worker.down(NOW, link::Event::Lower { connection: CONNECTION, event }, &mut self.out);
    }
    fn engine_up(engine: &mut Engine, event: io::Event) {
        assert!(engine.workers.up(NOW, event, &mut engine.events));
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the transport fixture fails any request it does not drive")]
    fn settle(&mut self, engine: &mut Engine) {
        for _ in 0_u16..1000 {
            let mut progress = false;
            while let Some(request) = engine.workers.host.io.pop() {
                progress = true;
                match request {
                    io::Request::Bind { socket, owner } => {
                        assert_eq!(socket, SOCKET);
                        self.owner = Some(owner);
                    }
                    io::Request::Output { down: stream::OutputDown::Room { right, .. }, .. } => Self::engine_up(
                        engine,
                        io::Event::Output {
                            owner: self.owner.unwrap(),
                            up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
                        },
                    ),
                    io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. } => {
                        self.to_worker.append(&bytes).unwrap();
                    }
                    io::Request::Stream { down: stream::Down::Demand { read, .. }, .. } => {
                        self.engine_read = Some(read);
                    }
                    io::Request::Stream { down: stream::Down::Finish, .. } => self.ended = true,
                    io::Request::Close { .. } => {}
                    other => panic!("engine transport: {other:?}"),
                }
            }
            while let Some(request) = self.out.pop() {
                progress = true;
                match request {
                    link::Request::Dial => {
                        self.worker.down(NOW, link::Event::Dialled { connection: CONNECTION }, &mut self.out);
                    }
                    link::Request::Ready => {
                        if !self.send_hello {
                            continue;
                        }
                        self.hello = true;
                        self.worker.down(
                            NOW,
                            link::Event::Hello {
                                hello: jig_domain_link::Hello {
                                    fresh: false,
                                    reads_usage: true,
                                    jobs: Box::new([]),
                                    slots: 1,
                                    stop_bound: Duration::from_millis(100),
                                    workstreams: Box::new([]),
                                    hosting: Box::new([]),
                                    bounds: hello_bounds(engine),
                                },
                            },
                            &mut self.out,
                        );
                    }
                    link::Request::Figures { figures } => self.figures.push(figures).unwrap(),
                    link::Request::Down { down: jig_domain_host::LinkDown::Sources { sources }, parts } => {
                        assert!(sources.is_empty() && parts.is_empty());
                    }
                    link::Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => self
                        .worker_lower(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Granted,
                        })),
                    link::Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                        self.to_engine.append(&bytes).unwrap();
                    }
                    link::Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => {
                        self.worker_read = Some(read);
                    }
                    link::Request::Close { .. }
                    | link::Request::Closed
                    | link::Request::Lower { lower: Lower::FinishWrite | Lower::Read(stream::Down::Finish), .. } => {}
                    other => panic!("worker transport: {other:?}"),
                }
            }
            if let Some(read) = self.engine_read
                && let Some(bytes) = self.to_engine.meet(read)
            {
                self.engine_read = None;
                Self::engine_up(engine, io::Event::Stream { owner: self.owner.unwrap(), up: stream::Up::Bytes(bytes) });
                progress = true;
            }
            if let Some(read) = self.worker_read
                && let Some(bytes) = self.to_worker.meet(read)
            {
                self.worker_read = None;
                self.worker_lower(LowerEvent::Read(stream::Up::Bytes(bytes)));
                progress = true;
            }
            if !progress {
                return;
            }
        }
        panic!("figures channel settles");
    }
}

fn hello_bounds(engine: &Engine) -> jig_domain_link::Bounds {
    let mut bounds = fixtures::worker_agent().bounds;
    // The resolved deployment uses stock providers and carriers rather than the small typed fixture.
    if let Some(state) = engine.limits_in_force() {
        let agent = crate::stock::agents();
        let mut endpoints = List::with_capacity(temper_engine_service_vocabulary::figures::ENDPOINTS);
        let mut programs = List::with_capacity(endpoints.capacity());
        for provider in &engine.running_providers {
            for endpoint in &provider.endpoints {
                endpoints
                    .push(jig_domain_link::EndpointBound {
                        endpoint: endpoint.number,
                        window: agent.fallback.window,
                        output: agent.fallback.output,
                    })
                    .unwrap();
                programs.push(endpoint.number).unwrap();
            }
        }
        bounds.endpoints = endpoints.into_boxed();
        bounds.program_endpoints = programs.into_boxed();
        bounds.message_bytes = state.in_force.jig.core.tasks.message_bytes;
        bounds.inbox = state.in_force.jig.core.tasks.run_inbox;
        bounds.unacknowledged = state.in_force.jig.hub.turns;
        bounds.turns = 901;
        bounds.time = Duration::from_secs(86_400);
        bounds.waiting = Duration::from_secs(86_400);
        bounds.spend = u64::MAX;
    }
    bounds
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "only synthetic web replies are intercepted; production routes every other request"
)]
fn settle(engine: &mut Engine) -> List<Reply> {
    let mut replies = List::with_capacity(2);
    for _ in 0_u16..256 {
        if let Some(event) = engine.events.pop() {
            engine.step(event);
        }
        root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        if engine.requests.is_empty() {
            engine.step(root::Event::Resume);
            root::release(&mut engine.domain, &engine.env, &mut engine.requests);
        }
        while let Some(request) = engine.requests.pop() {
            match request {
                root::Request::Party(root::PartyRequest::Deliver(root::Delivery::WebReply { reply, .. })) => {
                    replies.push(reply).unwrap();
                }
                other => crate::route::request(engine, other),
            }
        }
        engine.store.iterate();
        crate::engine::answer(engine);
        engine.domain.reclaim();
        if engine.domain.quiescent() && engine.events.is_empty() && engine.store.idle() {
            return replies;
        }
    }
    panic!(
        "settings decision at revision {} settles (quiescent {}, event count {}, request count {}, store idle {}, commits {})",
        engine.domain.settings_revision(),
        engine.domain.quiescent(),
        engine.events.len(),
        engine.requests.len(),
        engine.store.idle(),
        engine.domain.commits_in_flight(),
    );
}

fn signed_in(engine: &mut Engine) {
    engine.step(root::Event::Party(root::PartyInput::SignedIn {
        until: None,
        held: None,
        reply_to: ReplyTo::new(Token::new(100)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    let replies = settle(engine);
    assert_eq!(replies.len(), 1);
}

fn changed(engine: &mut Engine, agents: &s::AgentFigures, against: u32) {
    let value = own::SizedValue {
        size: agents.measure(&temper_client_protocol_wire::v1::CEILINGS).expect("within ceilings"),
        value: own::Value::AgentFigures(agents.clone()),
    };
    engine.step(root::Event::Party(root::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [u8::try_from(against).unwrap(); 16],
        digest: [u8::try_from(against).unwrap(); 32],
        kinds: Box::new([root::KindValue { kind: 21, scope: 0, number: 1, value }]),
        change: Box::new(settings::Change {
            against,
            items: Box::new([settings::Item {
                setting: settings::Setting::Application { kind: 21, scope: 0, number: 1 },
                edit: settings::Edit::Set(settings::Value::Application(Box::new([]))),
            }]),
            running: settings::Running::Stays,
        }),
    }));
    assert_eq!(
        settle(engine).as_slice(),
        &[Reply::Outcome(people::Outcome::SettingsChanged { revision: against.checked_add(1).unwrap() })]
    );
}

fn expected(agents: &s::AgentFigures) -> wire::Figures {
    let settings = fixtures::start_settings(fixtures::engine(), &[(1, 100)]);
    let figures = temper_engine_service_vocabulary::figures::compose(&settings, agents);
    let mut parts = figures;
    parts.places = 1;
    parts
}
fn notes(engine: &mut Engine) -> List<(u32, u32, FiguresRevision)> {
    let mut values = List::with_capacity(2);
    for queue in [&mut engine.notes, &mut engine.workers.notes] {
        while let Some(note) = queue.pop() {
            if let crate::Note::WorkerFigures { limits, endpoints, revision } = note {
                values.push((limits, endpoints, revision)).unwrap();
            }
        }
    }
    values
}

#[test]
fn restored_agents_reach_a_figures_opening_and_owner_changes_reach_a_link() {
    let mut service = started(None);
    fixtures::restart(&mut service);
    let mut parts = crate::stock::agents();
    parts.profile =
        List::from_boxed(Box::new([s::AgentFigure { name: b"policy.max_turns".as_slice().into(), value: 901 }]));
    let first = parts;
    let engine = service.engine.as_mut().unwrap();
    signed_in(engine);
    changed(engine, &first, 1);
    let previous = service.engine.take().unwrap();
    let stored = fixtures::reopened_store(previous.store, &previous.env.limits);
    let mut service = started(Some(stored));
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let expected_first = expected(&first);
    let figures = Peer::open(engine, Opening::Figures);
    assert_eq!(figures.figures.as_slice(), core::slice::from_ref(&expected_first));
    assert!(!figures.hello);
    assert!(figures.ended);
    Peer::engine_up(engine, io::Event::Closed { owner: figures.owner.unwrap() });
    assert_eq!(
        notes(engine).as_slice(),
        &[(expected_first.limits.len(), expected_first.agents.endpoints.len(), FiguresRevision::Agents(2))]
    );
    let mut linked = Peer::open(engine, Opening::Link);
    assert_eq!(linked.figures.as_slice(), &[expected_first]);
    assert!(linked.hello);
    settle(engine);
    let mut parts = first;
    parts
        .profile
        .get_mut(0)
        .unwrap()
        .clone_from(&s::AgentFigure { name: b"policy.max_turns".as_slice().into(), value: 902 });
    let second = parts;
    changed(engine, &second, 2);
    linked.settle(engine);
    let expected_second = expected(&second);
    assert_eq!(linked.figures.len(), 2);
    assert_eq!(linked.figures.as_slice()[1], expected_second);
    assert_eq!(
        notes(engine).as_slice(),
        &[(expected_second.limits.len(), expected_second.agents.endpoints.len(), FiguresRevision::Agents(3))]
    );
}

const REMOTE: &[u8] = b"checkout.remote_timeout_ms";

fn heard(figures: &wire::Figures, name: &[u8]) -> u64 {
    for row in &figures.limits {
        if row.name.as_ref() == name {
            return row.value;
        }
    }
    unreachable!("the figures carry every named limit")
}

fn settings_changed(engine: &mut Engine, revision: u32) {
    assert_eq!(settle(engine).as_slice(), &[Reply::Outcome(people::Outcome::SettingsChanged { revision })]);
}

fn live_limits_world(held: bool) -> Box<crate::Service> {
    let mut service = limits::owned_with_held(held);
    let engine = service.engine.as_mut().unwrap();
    notes(engine);
    let mut worker = fixtures::opening_workers()[0].clone();
    worker.digest = Some(skein_lib::sha256::digest(WORKER_SECRET));
    engine.step(limits::party(
        Box::new([settings::Item {
            setting: settings::Setting::Worker(1),
            edit: settings::Edit::Set(settings::Value::Worker(Box::new(worker))),
        }]),
        false,
        1,
    ));
    settings_changed(engine, 2);
    notes(engine);
    service
}

#[test]
fn durable_live_limits_reach_the_link_and_later_agents_compositions() {
    let mut service = live_limits_world(false);
    let engine = service.engine.as_mut().unwrap();
    let mut agents = crate::stock::agents();
    let profile = b"policy.max_turns";
    agents.profile = List::from_boxed(Box::new([s::AgentFigure { name: profile.as_slice().into(), value: 901 }]));
    changed(engine, &agents, 2);
    let mut linked = Peer::open(engine, Opening::Link);
    assert!(linked.hello);
    assert_eq!(linked.figures.len(), 1);
    let before = linked.figures.get(0).unwrap().clone();
    assert_eq!(heard(&before, REMOTE), 60_000);
    assert_eq!(before.agents.profile.get(0).unwrap().value, 901);
    notes(engine);
    settle(engine);
    notes(engine);

    engine.step(limits::party(Box::new([limits::temper(Box::new([item(b"live_goals", 60)]))]), false, 3));
    settings_changed(engine, 4);
    linked.settle(engine);
    assert_eq!(linked.figures.len(), 1, "a next-start figure sends nothing");
    assert!(notes(engine).is_empty());

    engine.step(limits::party(
        Box::new([limits::temper(Box::new([item(REMOTE, 600_000), item(b"live_goals", 60)]))]),
        false,
        4,
    ));
    linked.settle(engine);
    assert_eq!(linked.figures.len(), 1, "the change is not yet durable");
    settings_changed(engine, 5);
    linked.settle(engine);
    assert_eq!(linked.figures.len(), 2);
    let taken = linked.figures.get(1).unwrap();
    assert_eq!(heard(taken, REMOTE), 600_000);
    assert_eq!(heard(taken, b"live_goals"), 40);
    assert_eq!(taken.agents, before.agents);
    assert_eq!(
        notes(engine).as_slice(),
        &[(taken.limits.len(), taken.agents.endpoints.len(), FiguresRevision::Limits(5))]
    );

    agents.profile.get_mut(0).unwrap().value = 902;
    changed(engine, &agents, 5);
    linked.settle(engine);
    assert_eq!(linked.figures.len(), 3);
    let changed = linked.figures.get(2).unwrap();
    assert_eq!(changed.limits, linked.figures.get(1).unwrap().limits);
    assert_eq!(changed.agents.profile.get(0).unwrap().value, 902);
    assert_eq!(
        notes(engine).as_slice(),
        &[(changed.limits.len(), changed.agents.endpoints.len(), FiguresRevision::Agents(6))]
    );
}

#[test]
fn held_limits_send_no_live_figures() {
    let mut service = live_limits_world(true);
    let engine = service.engine.as_mut().unwrap();
    let mut linked = Peer::open(engine, Opening::Link);
    assert!(linked.hello);
    assert_eq!(linked.figures.len(), 1);
    notes(engine);
    settle(engine);
    notes(engine);

    engine.step(limits::party(Box::new([limits::temper(Box::new([item(REMOTE, 600_000)]))]), false, 2));
    settings_changed(engine, 3);
    linked.settle(engine);
    assert_eq!(linked.figures.len(), 1, "limits-held sends no figures");
    assert!(notes(engine).is_empty());
    let state = engine.limits_in_force().unwrap();
    assert_eq!(state.in_force.temper.checkout.remote_timeout.as_millis(), 60_000);
}

#[test]
fn a_new_opening_uses_the_live_hello_timer() {
    let mut service = live_limits_world(false);
    let engine = service.engine.as_mut().unwrap();
    let hello = Duration::from_millis(1_234);
    assert_ne!(engine.limits_in_force().unwrap().in_force.jig.link.hello, hello);
    engine.step(limits::party(
        Box::new([limits::jig(Box::new([item(b"link.hello_ms", hello.as_millis())]))]),
        false,
        2,
    ));
    settings_changed(engine, 3);
    assert_eq!(engine.limits_in_force().unwrap().in_force.jig.link.hello, hello);

    let peer = Peer::open_with_hello(engine, Opening::Link, false);
    assert!(!peer.hello && !peer.ended);
    assert_eq!(peer.figures.len(), 1, "the link is open and waiting for a hello");
    assert_eq!(heard(peer.figures.get(0).unwrap(), b"link.hello_ms"), hello.as_millis());
    assert_eq!(engine.workers.host.next_deadline(), Some(NOW.saturating_add(hello)));
}

#[test]
fn provider_restore_and_owner_changes_reach_the_shell_without_changing_running_figures() {
    let mut service = started(None);
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let running = engine.running_providers.clone();
    let (path, restored) = engine.take_providers_write().expect("restore keeps the providers before needs");
    assert_eq!(path.as_ref(), b"providers.setting");
    assert_eq!(restored, running);
    assert!(engine.take_providers_write().is_none());
    signed_in(engine);
    let mut provider = restored[0].clone();
    provider.endpoints[0].address = jig_protocol_config::EndpointAddress::Socket(([127, 0, 0, 1], 8082).into());
    engine.step(root::Event::Party(root::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [1; 16],
        digest: [1; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Change {
            against: 1,
            running: settings::Running::Stays,
            items: Box::new([settings::Item {
                setting: settings::Setting::Provider(1),
                edit: settings::Edit::Set(settings::Value::Provider(Box::new(provider.clone()))),
            }]),
        }),
    }));
    assert_eq!(settle(engine).as_slice(), &[Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })]);
    let expected: Box<[_]> = Box::new([provider]);
    assert_eq!(engine.take_providers_write(), Some((path, expected)));
    assert_eq!(engine.running_providers, running);
    let mut changed = false;
    for note in &engine.notes {
        if let crate::Note::StartupWarning { line } = note {
            changed |= line.as_ref() == b"LLM providers changed; they apply at the engine's next start";
        }
    }
    assert!(changed, "the change applies at the next start");
    assert!(engine.take_providers_write().is_none());
}

fn started(stored: Option<crate::store::Store>) -> crate::Service {
    let mut settings = fixtures::start_settings(fixtures::engine(), &[(1, 100)]);
    let root = crate::limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    settings.kinds_stock = crate::stock::forge(&root);
    let store = match stored {
        Some(store) => store,
        None => fixtures::memory_store_for(&root, &settings.jig.limits.store, &settings.jig.llm.models()),
    };
    let io = crate::startup::checks::engine_io(&settings.jig.limits.io);
    let forges =
        crate::forge::settings(&[], List::with_capacity(0), None, &mut jig_protocol_config::Problems::empty()).unwrap();
    let mut service = crate::Service::prepared(settings, &fixtures::seeds(), None, forges, io, None, None).unwrap();
    service.engine.as_mut().unwrap().store = store;
    service
}

fn local_value(places: u32) -> jig_client_protocol_wire::WorkerSetting {
    use jig_client_protocol_wire::WorkerSetting;
    WorkerSetting { number: 1, name: b"local".as_slice().into(), places, digest: None, withdrawn: false }
}

#[test]
fn local_copy_follows_owner_changes_and_repairs_a_different_restore() {
    let path: Box<[u8]> = b"/data/local.setting".as_slice().into();
    let mut service = started(None);
    service.engine.as_mut().unwrap().keep_local(path.clone(), None);
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().unwrap();
    let (_, default) = engine.take_local_write().expect("missing copy is repaired");
    let mut reader = skein_lib::Reader::new(&default);
    assert_eq!(
        jig_client_protocol_wire::WorkerSetting::decode(&jig_client_protocol_wire::CEILINGS, &mut reader).unwrap(),
        local_value(4)
    );
    assert!(reader.is_empty());
    signed_in(engine);
    engine.step(root::Event::Party(root::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: 1,
        key: [1; 16],
        digest: [1; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Change {
            against: 1,
            items: Box::new([settings::Item {
                setting: settings::Setting::Worker(1),
                edit: settings::Edit::Set(settings::Value::Worker(Box::new(settings::WorkerSetting {
                    number: 1,
                    name: b"local".as_slice().into(),
                    places: 2,
                    digest: None,
                    withdrawn: false,
                }))),
            }]),
            running: settings::Running::Stays,
        }),
    }));
    assert_eq!(settle(engine).as_slice(), &[Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })]);
    let (written, bytes) = engine.take_local_write().expect("owner change keeps copy");
    assert_eq!(written, path);
    let mut reader = skein_lib::Reader::new(&bytes);
    assert_eq!(
        jig_client_protocol_wire::WorkerSetting::decode(&jig_client_protocol_wire::CEILINGS, &mut reader).unwrap(),
        local_value(2)
    );
    assert!(reader.is_empty());
    assert!(engine.take_local_write().is_none());
    let previous = service.engine.take().unwrap();
    let stored = fixtures::reopened_store(previous.store, &previous.env.limits);
    let mut service = started(Some(stored));
    service.engine.as_mut().unwrap().keep_local(path.clone(), Some(local_value(4)));
    fixtures::restart(&mut service);
    assert_eq!(service.engine.as_mut().unwrap().take_local_write(), Some((path.clone(), bytes)));
    let previous = service.engine.take().unwrap();
    let stored = fixtures::reopened_store(previous.store, &previous.env.limits);
    let mut service = started(Some(stored));
    service.engine.as_mut().unwrap().keep_local(path, Some(local_value(2)));
    fixtures::restart(&mut service);
    assert!(service.engine.as_mut().unwrap().take_local_write().is_none());
}

#[test]
fn box_copy_follows_owner_changes_and_repairs_a_different_restore() {
    let stock = crate::stock::agents();
    let mut service = started(None);
    service.engine.as_mut().unwrap().keep_agents(b"/data/agents.setting".as_slice().into(), Some(stock.clone()));
    fixtures::restart(&mut service);
    assert!(service.engine.as_mut().unwrap().take_agents_write().is_none());
    let mut parts = stock.clone();
    for row in parts.profile.iter_mut() {
        if row.name.as_ref() == b"policy.max_turns" {
            *row = s::AgentFigure { name: b"policy.max_turns".as_slice().into(), value: 900 };
        }
    }
    let agents = parts;
    let engine = service.engine.as_mut().unwrap();
    signed_in(engine);
    changed(engine, &agents, 1);
    let (path, bytes) = engine.take_agents_write().expect("accepted owner change keeps copy");
    assert_eq!(path.as_ref(), b"/data/agents.setting");
    let mut reader = skein_lib::Reader::new(&bytes);
    assert_eq!(s::AgentFigures::decode(&s::CEILINGS, &mut reader).unwrap(), agents);
    assert!(reader.is_empty());
    assert!(engine.take_agents_write().is_none());
    let previous = service.engine.take().unwrap();
    let stored = fixtures::reopened_store(previous.store, &previous.env.limits);
    let mut service = started(Some(stored));
    service.engine.as_mut().unwrap().keep_agents(path.clone(), Some(stock));
    fixtures::restart(&mut service);
    assert_eq!(service.engine.as_mut().unwrap().take_agents_write(), Some((path.clone(), bytes.clone())));
    let previous = service.engine.take().unwrap();
    let stored = fixtures::reopened_store(previous.store, &previous.env.limits);
    let mut service = started(Some(stored));
    service.engine.as_mut().unwrap().keep_agents(path, Some(agents));
    fixtures::restart(&mut service);
    assert!(service.engine.as_mut().unwrap().take_agents_write().is_none());
}
