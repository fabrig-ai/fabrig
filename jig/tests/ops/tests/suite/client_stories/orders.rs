//! The linked Ops root against scripted stream, transport and delayed storage.
use jig_client_protocol_wire::Record as _;
use jig_client::{Event as ClientEvent, Message, Streamed};
use jig_client_protocol_wire as w;
use jig_client_view::address::Address as JigAddress;
use jig_ops_client_domain::{self as root, Address, Event, Place, Request, infrastructure, observability};
use jig_ops_world::{PARTY, PROJECT};
use skein_lib::{Duration, Env, List, Queue, Reader, Time, Token, Wall, Writer};

const SEED: u64 = 9211;
const STREAM: [u8; 16] = [11; 16];
const HEARTBEAT: Duration = Duration::from_secs(1);
// The fixture serves jig plus both Ops connector owners.
const OWNERS: u32 = 3;
// One initial inbox batch and one read/watch batch for each connector.
const EXCHANGES: u32 = 3;
const INFRASTRUCTURE: u16 = 2;
const OBSERVABILITY: u16 = 1;
const ONE_PROJECT: u32 = 1;
const INCIDENT: u64 = 12;

struct World {
    root: root::Domain,
    env: Env<root::Limits>,
    // Storage deliberately remains outstanding while later entrances run.
    writes: Vec<root::store::Record>,
    steps: Vec<Vec<&'static str>>,
    watches: Vec<(Token, Option<u16>, u64)>,
    traffic: Vec<(usize, &'static str, u16)>,
}

impl World {
    fn ready() -> Self {
        let mut limits = jig_ops_world::client::limits();
        limits.client.exchanges = EXCHANGES;
        let mut world = Self {
            root: root::new(&limits, SEED),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            writes: vec![],
            steps: vec![],
            watches: vec![],
            traffic: vec![],
        };
        world.step(Event::Start {
            address: Address::Jig(JigAddress::Inbox { project: None }),
            kept: List::with_capacity(0),
        });
        world.client(ClientEvent::SignIn { with: jig_client::With::Transport });
        world.stream(&hello());
        world.admit_watches();
        world.steps.clear();
        world.traffic.clear();
        world
    }

    fn admit_watches(&mut self) {
        for (exchange, member, _) in std::mem::take(&mut self.watches) {
            let answer = w::OpenAnswer::Admitted;
            let mut bytes = Writer::new(skein_lib::length::usize_of(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
            answer.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("opening answer");
            if let Some(member) = member {
                let part = w::BatchReply { member, reply: Some(bytes.finish()) };
                let mut bytes = Writer::new(skein_lib::length::usize_of(part.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
                part.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("opening part bytes");
                self.client(ClientEvent::Part { exchange, message: bytes.finish() });
            } else {
                self.client(ClientEvent::Reply { exchange, message: bytes.finish() });
            }
        }
    }

    fn client(&mut self, event: ClientEvent) {
        self.step(Event::Client { event });
    }

    fn stream(&mut self, event: &w::StreamEvent) {
        let mut bytes = Writer::new(skein_lib::length::usize_of(event.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
        event.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("measured stream event");
        self.client(ClientEvent::Streamed { message: Streamed::Record(bytes.finish()) });
    }

    fn step(&mut self, event: Event) {
        let mut out = Queue::uncapped();
        root::step(&mut self.root, &self.env, event, &mut out);
        root::settle(&mut self.root, &self.env, &mut out);
        root::take(&mut self.root);
        self.drain(out);
    }

    fn fire(&mut self) {
        let mut out = Queue::uncapped();
        root::fire(&mut self.root, &self.env, &mut out);
        root::settle(&mut self.root, &self.env, &mut out);
        root::take(&mut self.root);
        self.drain(out);
    }

    fn message(&mut self, exchange: Token, member: Option<u16>, message: Message) {
        match message {
            Message::Watch(bytes) => {
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("watch bytes");
                self.watches.push((exchange, member, open.watch));
                self.traffic.push((self.steps.len(), "open", owner(&open.subject.owner)));
            }
            Message::Read(bytes) => {
                let read = w::Owned::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("read bytes");
                self.traffic.push((self.steps.len(), "read", owner(&read.owner)));
            }
            Message::Unwatch(_) => self.traffic.push((self.steps.len(), "close", 0)),
            Message::Batch(_)
            | Message::LinkAsk(_)
            | Message::Request(_)
            | Message::StreamGone(_)
            | Message::SignInLink(_)
            | Message::SignInService
            | Message::SignOut
            | Message::ApprovalAsk(_)
            | Message::ApprovalRead(_)
            | Message::ApprovalDecide(_)
            | Message::SignInApproval(_)
            | Message::Authorize(_) => {
                panic!("unplanned exchange");
            }
        }
    }
    fn drain(&mut self, mut out: Queue<Request>) {
        let mut order = vec![];
        while let Some(request) = out.pop() {
            match request {
                Request::Save { record } => {
                    self.writes.push(record);
                    order.push("save");
                }
                Request::Erase { .. } => order.push("erase"),
                Request::Client { request: jig_client::Request::Exchange { exchange, message, .. } } => {
                    if let Message::Batch(bytes) = message {
                        let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("batch");
                        for (place, record) in batch.members.into_boxed().into_iter().enumerate() {
                            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture") - 1));
                            let message = match record {
                                w::Member::Read(record) => {
                                    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("read");
                                    Message::Read(writer.finish())
                                }
                                w::Member::Open(record) => {
                                    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("open");
                                    Message::Watch(writer.finish())
                                }
                                w::Member::Close(record) => {
                                    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("close");
                                    Message::Unwatch(writer.finish())
                                }
                            };
                            self.message(exchange, Some(u16::try_from(place).expect("member")), message);
                        }
                    } else {
                        self.message(exchange, None, message);
                    }
                    order.push("exchange");
                }
                Request::Client { .. } => order.push("transport"),
                Request::Unlinked { .. } => {
                    unreachable!("a linked root asks no unlinked request (jig's client/domain/client/README.md, 4.4)")
                }
                Request::Show { .. } => order.push("show"),
                Request::SignIn { .. }
                | Request::SignOut
                | Request::Fact { .. }
                | Request::Infrastructure(_)
                | Request::Observability(_) => order.push("fact"),
            }
        }
        self.steps.push(order);
    }
}

fn hello() -> w::StreamEvent {
    let mut served = List::with_capacity(OWNERS);
    for (owner, version) in [
        (w::Owner::Jig, jig_client_protocol_wire::VERSION),
        (family(OBSERVABILITY), jig_ops_client_protocol_wire::VERSION),
        (family(INFRASTRUCTURE), jig_ops_client_protocol_wire::VERSION),
    ] {
        served
            .push(
                w::Served { owner, first: version, last: version },
            )
            .expect("three owners");
    }
    let hello = w::Hello { stream: STREAM, served, heartbeat: HEARTBEAT };
    let w::StreamEvent::Delivery(delivery) = frame(0) else { unreachable!("me fixture") };
    w::StreamEvent::Opened(
        w::Opened { hello, me: delivery.document },
    )
}

fn family(number: u16) -> w::Owner {
    w::Owner::Family(w::Family { number })
}

fn owner(owner: &w::Owner) -> u16 {
    match owner {
        w::Owner::Jig => 0,
        w::Owner::Family(family) => family.number,
    }
}

fn text(bytes: &[u8]) -> w::DocumentText {
    w::DocumentText { bytes: bytes.into(), cut: false }
}

fn frame(watch: u64) -> w::StreamEvent {
    let mut projects = List::with_capacity(ONE_PROJECT);
    projects
        .push(
            w::MeProject {
                    left_out: 0,
                    defaults: List::with_capacity(0),
                    public: None,
                    project: PROJECT,
                    name: text(b"First"),
                    role: w::HeldRole::Member,
                    pool: None,
                    templates: Box::new([]),
                    creating: None,
                    models: List::with_capacity(0),
                },
        )
        .expect("one project");
    let projects = jig_protocol_engine_clients::translate::owner::me_projects_pieces(
        &projects,
        w::CEILINGS.me_projects,
        w::paths::ME_PROJECTS, &jig_client_protocol_wire::CEILINGS)
    .expect("project pieces");
    let frame = w::Me {
            party: w::MeParty {
                    number: PARTY,
                    kind: w::PartyKind::Person,
                    login: text(b"fixture"),
                    name: text(b"Pat"),
                },
            projects,
            needs: w::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            needs_total: 0,
            updates: w::Count { value: 0, limited: false },
            results_seen: 0,
            cut: false,
            retention: HEARTBEAT,
            deployment_role: None,
            providers: Box::new([]),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 4096,
            words_bytes: 8192,
        };
    let value = w::DeliveryValue::Me(frame);
    let mut bytes = Writer::new(skein_lib::length::usize_of(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("measured frame");
    let document = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() };
    w::StreamEvent::Delivery(
        w::Delivery { watch, document },
    )
}

/// `child_records_dispatched`: every child write precedes transport within
/// its entrance, even while earlier storage completions remain delayed.
#[test]
fn ops_child_records_precede_transport_within_a_step() {
    let mut world = World::ready();
    world.step(Event::Went {
        place: Place::Address(Address::Infrastructure(infrastructure::Address::Environments {
            project: PROJECT,
            pool: None,
        })),
    });
    assert_eq!(world.steps, vec![vec!["save", "exchange"]]);
}

/// `child_requests_dispatched`: each connector's read/watch consequences
/// finish routing before the next entrance, preserving the client's read/open
/// order while storage is delayed and the two connectors fill transport credit.
#[test]
fn ops_client_routes_finish_before_their_caller_continues() {
    let mut world = World::ready();
    world.step(Event::Infrastructure(infrastructure::Event::Went {
        address: infrastructure::Address::Environments { project: PROJECT, pool: None },
    }));
    world.admit_watches();
    world.step(Event::Observability(observability::Event::Went {
        address: observability::Address::Incident { project: PROJECT, incident: INCIDENT },
    }));
    world.fire();
    assert_eq!(
        world.traffic,
        [
            (0, "read", INFRASTRUCTURE),
            (0, "open", INFRASTRUCTURE),
            (2, "read", OBSERVABILITY),
            (2, "open", OBSERVABILITY),
        ]
    );
    assert!(world.root.client().admission().attempts <= EXCHANGES);
    assert_eq!(
        world.writes.last(),
        Some(&root::store::Record::Observability(observability::Record::Place {
            version: 1,
            party: PARTY,
            address: observability::Address::Incident { project: PROJECT, incident: INCIDENT },
        },))
    );
}
