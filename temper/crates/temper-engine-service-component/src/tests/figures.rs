//! A running engine gives restored and changed figures (protocol/configuration/figures.md, 9).
use super::fixtures;
use crate::engine::Engine;
use jig_domain_core::settings;
use jig_domain_people::{self as people, Reply};
use jig_protocol_worker_link as link;
use jig_protocol_worker_wire::{self as wire, Opening};
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{Duration, Intake, List, Queue, ReplyTo, Secret, Time, Token, stream};
use temper_client_protocol_wire::v1 as s;
use temper_engine_domain as root;
use temper_engine_domain_forge::settings as own;

const SOCKET: Token = Token::new(901);
const CONNECTION: Token = Token::new(902);
const NOW: Time = Time::from_nanos(1);

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
}
impl Peer {
    fn open(engine: &mut Engine, opening: Opening) -> Self {
        let settings = fixtures::start_settings(fixtures::engine(), &[(1, 100)]);
        let limits = crate::workers::limits(&settings).unwrap();
        let worker_limits = link::Limits {
            sizes: limits.sizes,
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
                secret: Secret::new(b"worker-secret".as_slice().into()),
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
            figures: List::with_capacity(2),
            ended: false,
            hello: false,
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
        let mut workers = fixtures::opening_workers();
        workers[0].digest = Some(skein_lib::sha256::digest(b"worker-secret"));
        engine.workers.down(NOW, jig_protocol_engine_workers::Event::Workers(workers), &mut engine.events);
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
                        self.hello = true;
                        self.worker.down(
                            NOW,
                            link::Event::Hello {
                                hello: jig_domain_link::Hello {
                                    jobs: Box::new([]),
                                    slots: 1,
                                    stop_bound: Duration::from_millis(100),
                                    workstreams: Box::new([]),
                                    hosting: Box::new([]),
                                    bounds: fixtures::worker_agent().bounds,
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
    panic!("settings decision settles");
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
    let value = own::SizedValue { size: agents.measure(), value: own::Value::AgentFigures(agents.clone()) };
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
    let mut parts = figures.into_parts();
    parts.places = 1;
    wire::Figures::new(&wire::CEILINGS, parts).unwrap()
}
fn notes(engine: &mut Engine) -> List<(u32, u32, u32)> {
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
    let mut parts = crate::stock::agents().into_parts();
    parts.profile = List::from_boxed(Box::new([s::AgentFigure::new(
        &s::CEILINGS,
        s::AgentFigureParts { name: b"policy.max_turns".as_slice().into(), value: 901 },
    )
    .unwrap()]));
    let first = s::AgentFigures::new(&s::CEILINGS, parts).unwrap();
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
        &[(expected_first.limits().len(), expected_first.agents().endpoints().len(), 2)]
    );
    let mut linked = Peer::open(engine, Opening::Link);
    assert_eq!(linked.figures.as_slice(), &[expected_first]);
    assert!(linked.hello);
    settle(engine);
    let mut parts = first.into_parts();
    parts.profile.get_mut(0).unwrap().clone_from(
        &s::AgentFigure::new(
            &s::CEILINGS,
            s::AgentFigureParts { name: b"policy.max_turns".as_slice().into(), value: 902 },
        )
        .unwrap(),
    );
    let second = s::AgentFigures::new(&s::CEILINGS, parts).unwrap();
    changed(engine, &second, 2);
    linked.settle(engine);
    let expected_second = expected(&second);
    assert_eq!(linked.figures.len(), 2);
    assert_eq!(linked.figures.as_slice()[1], expected_second);
    assert_eq!(
        notes(engine).as_slice(),
        &[(expected_second.limits().len(), expected_second.agents().endpoints().len(), 3)]
    );
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
    use jig_client_protocol_wire::{CEILINGS, WorkerSetting, WorkerSettingParts};
    WorkerSetting::new(
        &CEILINGS,
        WorkerSettingParts { number: 1, name: b"local".as_slice().into(), places, digest: None, withdrawn: false },
    )
    .unwrap()
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
    let mut parts = stock.clone().into_parts();
    for row in parts.profile.iter_mut() {
        if row.name() == b"policy.max_turns" {
            *row = s::AgentFigure::new(
                &s::CEILINGS,
                s::AgentFigureParts { name: b"policy.max_turns".as_slice().into(), value: 900 },
            )
            .unwrap();
        }
    }
    let agents = s::AgentFigures::new(&s::CEILINGS, parts).unwrap();
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
