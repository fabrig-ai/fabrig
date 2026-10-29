//! The linked Ops root against scripted stream, transport and delayed storage.
use jig_client::{Event as ClientEvent, Message, Streamed};
use jig_client_protocol_wire::v19 as w;
use jig_client_view::address::Address as JigAddress;
use jig_client_view::boundary::Offset;
use jig_ops_client_domain::{self as root, Address, Event, Place, Request, infrastructure};
use skein_lib::{Duration, Env, List, Queue, Reader, Time, Token, Wall, Writer};

const SEED: u64 = 9211;
const PARTY: u64 = 7;
const PROJECT: u32 = 1;
const STREAM: [u8; 16] = [11; 16];
const HEARTBEAT: Duration = Duration::from_secs(1);
// The fixture serves jig plus both Ops connector owners.
const OWNERS: u32 = 3;

struct World {
    root: root::Domain,
    env: Env<root::Limits>,
    // Storage deliberately remains outstanding while later entrances run.
    writes: Vec<root::store::Record>,
    steps: Vec<Vec<&'static str>>,
    watches: Vec<(Token, u64)>,
}

impl World {
    fn ready() -> Self {
        let limits = jig_ops_world::client::limits();
        let mut world = Self {
            root: root::new(&limits, SEED),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            writes: vec![],
            steps: vec![],
            watches: vec![],
        };
        world.step(Event::Start {
            address: Address::Jig(JigAddress::Inbox { project: None }),
            kept: List::with_capacity(0),
            offset: Offset { minutes: 0 },
        });
        world.client(ClientEvent::SignIn { with: jig_client::With::Transport });
        world.stream(hello());
        // All three children share the Me watch. The peer admits its exchange
        // before delivering the frame, then storage is left delayed.
        let (exchange, watch) = world.watches.remove(0);
        let answer = w::OpenAnswer::Admitted;
        let mut bytes = Writer::new(usize::try_from(answer.measure()).expect("open answer size"));
        answer.encode(&mut bytes).expect("measured open answer");
        world.client(ClientEvent::Reply { exchange, message: bytes.finish() });
        world.stream(frame(watch));
        world.steps.clear();
        world
    }

    fn client(&mut self, event: ClientEvent) {
        self.step(Event::Client { event });
    }

    fn stream(&mut self, event: w::StreamEvent) {
        let mut bytes = Writer::new(usize::try_from(event.measure()).expect("stream event size"));
        event.encode(&mut bytes).expect("measured stream event");
        self.client(ClientEvent::Streamed { message: Streamed::Record(bytes.finish()) });
    }

    fn step(&mut self, event: Event) {
        let mut out = Queue::uncapped();
        root::step(&mut self.root, &self.env, event, &mut out);
        root::take(&mut self.root, &self.env, &mut out);
        let mut order = vec![];
        while let Some(request) = out.pop() {
            match request {
                Request::Save { record } => {
                    self.writes.push(record);
                    order.push("save");
                }
                Request::Erase { .. } => order.push("erase"),
                Request::Client { request: jig_client::Request::Exchange { exchange, message, .. } } => {
                    if let Message::Watch(bytes) = message {
                        let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("watch bytes");
                        self.watches.push((exchange, open.watch()));
                    }
                    order.push("exchange");
                }
                Request::Client { .. } => order.push("transport"),
                Request::Show { .. } => order.push("show"),
                Request::SignIn { .. } | Request::SignOut | Request::Fact { .. }
                | Request::Infrastructure(_) | Request::Observability(_) => order.push("fact"),
            }
        }
        self.steps.push(order);
    }
}

fn hello() -> w::StreamEvent {
    let mut served = List::with_capacity(OWNERS);
    for (owner, version) in [
        (w::Owner::Jig, jig_client_protocol_wire::VERSION),
        (family(1), jig_ops_client_protocol_wire::VERSION),
        (family(2), jig_ops_client_protocol_wire::VERSION),
    ] {
        served.push(w::Served::new(&w::CEILINGS, w::ServedParts { owner, first: version, last: version })
            .expect("served owner")).expect("three owners");
    }
    w::StreamEvent::Hello(w::Hello::new(&w::CEILINGS, w::HelloParts { stream: STREAM, served, heartbeat: HEARTBEAT })
        .expect("hello fixture"))
}

fn family(number: u16) -> w::Owner {
    w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number }).expect("owner number"))
}

fn text(bytes: &[u8]) -> w::DocumentText {
    w::DocumentText::new(&w::CEILINGS, w::DocumentTextParts { bytes: bytes.into(), cut: false })
        .expect("short fixture text")
}

fn frame(watch: u64) -> w::StreamEvent {
    let mut projects = List::with_capacity(1);
    projects.push(w::MeProject::new(&w::CEILINGS, w::MeProjectParts {
        project: PROJECT, name: text(b"First"), role: w::HeldRole::Member,
        pool: None, templates: Box::new([]), creating: None, models: List::with_capacity(0),
    }).expect("project fixture")).expect("one project");
    let projects = jig_protocol_engine_clients::translate::owner::me_projects_pieces(
        &projects, w::CEILINGS.me_projects, w::Path::MeProjects,
    ).expect("project pieces");
    let frame = w::Me::new(&w::CEILINGS, w::MeParts {
        party: w::MeParty::new(&w::CEILINGS, w::MePartyParts {
            number: PARTY, kind: w::PartyKind::Person, login: text(b"fixture"), name: text(b"Pat"),
        }).expect("party fixture"),
        projects,
        needs: w::Needs::new(&w::CEILINGS, w::NeedsParts {
            questions: 0, proposals: 0, escalations: 0, person_tasks: 0,
        }).expect("empty needs"),
        needs_total: 0,
        updates: w::Count::new(&w::CEILINGS, w::CountParts { value: 0, limited: false }).expect("empty updates"),
        results_seen: 0, cut: false, retention: HEARTBEAT, deployment_role: None,
        providers: Box::new([]), providers_cut: false, key_bytes: 256, redirect_bytes: 4096,
    }).expect("frame fixture");
    let value = w::DeliveryValue::Me(frame);
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("frame size"));
    value.encode(&mut bytes).expect("measured frame");
    let document = w::Owned::new(&w::CEILINGS, w::OwnedParts {
        owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish(),
    }).expect("owned frame");
    w::StreamEvent::Delivery(w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch, document })
        .expect("frame delivery"))
}

/// `child_records_dispatched`: every child write precedes transport within
/// its entrance, even while earlier storage completions remain delayed.
#[test]
fn ops_child_records_precede_transport_within_a_step() {
    let mut world = World::ready();
    world.step(Event::Went { place: Place::Address(Address::Infrastructure(
        infrastructure::Address::Environments { project: PROJECT, pool: None },
    )) });
    assert_eq!(world.steps, vec![vec!["save", "exchange", "exchange"]]);
}
