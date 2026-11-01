//! Runtime forge membership (forge-setup.md, 6.3; playground-reset.md, 3 and 4).
use super::{fixtures, github_fixtures, host};
use crate::{
    github::manifest::Target,
    host::{Answer, Event, Forge, Host, Kind, Limits, Need, Plan, Request, Settings, Transport, Unusable},
    item::Item,
    reconcile::{Failure, Outcome, Stage, Why},
};
use skein_lib::{
    Env, Intake, List, Queue, Time, Token, Wall,
    stream::{Down, Up},
};

const FORGEJO: u16 = 1; // fixtures::given's forge.
const GITHUB: u16 = 4; // github_fixtures::given's forge.
const OTHER: u16 = 5;
const RUN: Token = Token::new(1);
const SECOND_RUN: Token = Token::new(2);
const THIRD_RUN: Token = Token::new(3);
const SOCKET: Token = Token::new(100);
const ADDRESS: skein_io::Addr =
    skein_io::Addr::Ip(core::net::SocketAddr::new(core::net::IpAddr::V4(core::net::Ipv4Addr::LOCALHOST), 3000));
const LONG_NAME: usize = 32768; // host::limits().io.output.
const ADMIN_BODY: &[u8] = br#"{"id":1,"login":"kit-admin","is_admin":true,"full_name":""}"#;
const BOT_BODY: &[u8] = br#"{"id":42,"login":"kit","is_admin":false,"full_name":"kit:bot"}"#;

fn forgejo(number: u16) -> Forge {
    let mut given = fixtures::given();
    given.forge = number;
    plain(Kind::Forgejo(given))
}
fn github(number: u16) -> Forge {
    let mut given = github_fixtures::given();
    given.forge = number;
    plain(Kind::GitHub(given))
}
fn plain(kind: Kind) -> Forge {
    Forge { kind, address: ADDRESS, transport: Transport::Plaintext { name: None } }
}
fn reconcile(forge: u16) -> Plan {
    Plan::Reconcile { forge, adopted: Box::new([]) }
}
fn reachable(forge: u16) -> Plan {
    Plan::Reachable { forge, installation: None, page: core::num::NonZeroU32::new(1).expect("first page") }
}
fn manifest(forge: u16) -> Plan {
    Plan::Manifest { forge, target: Target::Personal }
}
fn convert(forge: u16, state: &[u8]) -> Plan {
    Plan::Convert { forge, code: fixtures::secret(b"callback-code"), state: state.into() }
}
fn unknown(run: Token, forge: u16, step: Stage, subject: &[u8]) -> Event {
    Event::Stop { run, failure: Failure { forge, step, subject: subject.into(), why: Why::Unknown } }
}
struct World {
    host: Host,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<skein_io::Request>,
}
impl World {
    fn new(settings: Settings) -> Self {
        Self::with(host::limits(), settings)
    }
    fn with(limits: Limits, settings: Settings) -> Self {
        Self {
            host: Host::new(&limits, settings, host::crypto()).expect("valid host fixture"),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            up: Queue::uncapped(),
            io: Queue::uncapped(),
        }
    }
    fn add(&mut self, forge: Forge) {
        self.host.add(&self.env.limits, forge).expect("valid runtime forge");
    }
    fn down(&mut self, request: Request) {
        self.host.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn run(&mut self, run: Token, plan: Plan) {
        self.down(Request::Run { run, plan });
    }
    fn answer(&mut self, run: Token, answer: Answer) {
        self.down(Request::Answer { run, answer });
    }
    fn event(&mut self, event: skein_io::Event) {
        self.host.up(&self.env, event, &mut self.up, &mut self.io);
    }
    fn fire(&mut self) {
        self.host.fire(&self.env, &mut self.up, &mut self.io);
    }
    fn read(&mut self, run: Token, item: Item, value: &[u8]) {
        assert_eq!(self.up.pop(), Some(Event::Need { run, need: Need::Read(item) }));
        self.answer(run, Answer::Read(Some(fixtures::secret(value))));
    }
    fn connection(&mut self) -> Token {
        let Some(skein_io::Request::Connect { owner, addr }) = self.io.pop() else {
            panic!("connection request");
        };
        assert_eq!(addr, ADDRESS);
        owner
    }
    fn state(&mut self, forge: u16) -> Box<[u8]> {
        self.run(RUN, manifest(forge));
        match self.up.pop().expect("manifest outcome") {
            Event::Done { run: RUN, outcome: Outcome::Manifest { state, .. } } => state,
            Event::Need { .. } | Event::Done { .. } | Event::Stop { .. } | Event::Closed => panic!("manifest Done"),
        }
    }
    fn reply(&mut self, body: &[u8]) {
        self.reply_status("200 OK", body);
    }
    #[expect(clippy::disallowed_macros, reason = "the HTTP fixture frames its body's length")]
    fn reply_status(&mut self, status: &str, body: &[u8]) {
        let owner = self.connection();
        self.event(skein_io::Event::Connecting { owner, socket: owner });
        self.event(skein_io::Event::Connected { owner });
        let mut intake = Intake::with_capacity(self.env.limits.body_bytes_in);
        let head = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n\r\n", body.len());
        intake.append(head.as_bytes()).expect("fixture head fits");
        intake.append(body).expect("fixture body fits");
        for _ in 0_u32..4096 {
            match self.io.pop() {
                Some(skein_io::Request::Stream { down: Down::Demand { read, room }, .. }) => {
                    if room > 0 {
                        self.event(skein_io::Event::Stream { owner, up: Up::Room });
                    } else if let Some(bytes) = intake.meet(read) {
                        self.event(skein_io::Event::Stream { owner, up: Up::Bytes(bytes) });
                    }
                }
                Some(skein_io::Request::Stream { down: Down::Send(_) | Down::Finish, .. }) => (),
                Some(skein_io::Request::Close { entity }) => {
                    assert_eq!(entity, owner);
                    self.event(skein_io::Event::Closed { owner });
                    return;
                }
                Some(other) => panic!("unexpected io: {other:?}"),
                None => self.fire(),
            }
        }
        panic!("HTTP reply did not settle");
    }
    fn finish_reconcile(&mut self, run: Token) {
        self.read(run, Item::AdminToken, fixtures::ADMIN);
        self.finish_after_admin(run);
    }
    fn finish_after_admin(&mut self, run: Token) {
        self.reply(ADMIN_BODY);
        self.reply(BOT_BODY);
        for item in [Item::ApiToken, Item::GitToken] {
            self.read(run, item, b"bot-token");
            self.reply(BOT_BODY);
        }
        self.read(run, Item::User, &42_u64.to_le_bytes());
    }
}

#[test]
fn an_added_forgejo_forge_runs_its_reconcile() {
    let mut w = World::new(host::settings(&[]));
    w.add(forgejo(FORGEJO));
    w.add(github(GITHUB));
    w.run(RUN, reconcile(FORGEJO));
    w.finish_reconcile(RUN);
    assert_eq!(w.up.pop(), Some(Event::Done { run: RUN, outcome: fixtures::done() }));
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}

#[test]
fn an_added_github_forge_issues_and_takes_its_state() {
    let mut w = World::new(host::settings(&[]));
    w.add(forgejo(FORGEJO));
    w.add(github(GITHUB));
    let state = w.state(GITHUB);
    w.run(RUN, convert(GITHUB, &state));
    w.reply_status("404 Not Found", b"");
    assert_eq!(
        w.up.pop(),
        Some(Event::Stop {
            run: RUN,
            failure: Failure {
                forge: GITHUB,
                step: Stage::App,
                subject: b"convert".as_slice().into(),
                why: Why::Refused(404)
            }
        })
    );
    w.run(RUN, convert(GITHUB, &state));
    assert_eq!(w.up.pop(), Some(unknown(RUN, GITHUB, Stage::State, b"convert")));
    assert!(w.io.is_empty());
}

struct Admission {
    kept: &'static [u16],
    limits: Limits,
    kind: Kind,
    address: skein_io::Addr,
    name: Option<Box<[u8]>>,
    expected: Unusable,
}
impl Admission {
    fn new(kept: &'static [u16], forge: Forge, expected: Unusable) -> Self {
        Self { kept, limits: host::limits(), kind: forge.kind, address: forge.address, name: None, expected }
    }
    fn forge(&self) -> Forge {
        Forge {
            kind: self.kind.clone(),
            address: self.address,
            transport: Transport::Plaintext { name: self.name.clone() },
        }
    }
    fn settings(&self) -> Settings {
        let settings = host::settings(self.kept);
        let mut forges = List::with_capacity(self.limits.forges.checked_add(1).expect("small fixture count"));
        for forge in settings.forges.into_boxed() {
            forges.push(forge).expect("fixture count");
        }
        forges.push(self.forge()).expect("one candidate");
        Settings { forges, user_agent: settings.user_agent }
    }
}
#[test]
fn an_add_is_refused_as_new_refuses() {
    use crate::given::{Place, Refused};
    let mut address = Admission::new(&[FORGEJO], github(GITHUB), Unusable::Address { forge: GITHUB });
    address.address = skein_io::Addr::Ip(([192, 0, 2, 1], 3000).into());
    let mut name = Admission::new(&[FORGEJO], github(GITHUB), Unusable::PlaintextName { forge: GITHUB });
    name.name = Some(b"bad/name".as_slice().into());
    let mut request = Admission::new(&[FORGEJO], github(GITHUB), Unusable::Request { forge: GITHUB });
    // A name as long as io's whole request cap cannot leave room for the headers.
    let mut long_name = skein_lib::bytes::zeroed(LONG_NAME);
    long_name.fill(b'a');
    request.name = Some(long_name);
    let mut in_process = Admission::new(&[FORGEJO], github(GITHUB), Unusable::Address { forge: GITHUB });
    in_process.address = skein_io::Addr::InProcess(skein_io::Name::new(b"setup").expect("fixture name"));
    let mut forgejo_given = fixtures::given();
    forgejo_given.forge = OTHER;
    forgejo_given.bot_login = Box::new([]);
    let mut github_given = github_fixtures::given();
    github_given.limits.reachable = 0;
    let mut reachable = github_fixtures::given();
    reachable.limits.reachable = host::limits().inner.checked_add(1).expect("small page limit");
    let mut documents = Admission::new(
        &[FORGEJO],
        github(GITHUB),
        Unusable::GitHubDocuments(skein_forge::github::limits::Problem::Depth),
    );
    documents.limits.depth = 9;
    let mut metadata =
        Admission::new(&[FORGEJO], github(GITHUB), Unusable::GitHubMetadata { inner: 14, string_bytes: 27 });
    metadata.limits.string_bytes = 26;
    for case in [
        Admission::new(&[FORGEJO], github(FORGEJO), Unusable::Duplicate { forge: FORGEJO }),
        Admission::new(&[FORGEJO, OTHER], github(GITHUB), Unusable::Forges { forges: 2 }),
        address,
        in_process,
        name,
        request,
        Admission::new(
            &[FORGEJO],
            plain(Kind::Forgejo(forgejo_given)),
            Unusable::Given { forge: OTHER, refused: Refused { place: Place::BotLogin, other: None } },
        ),
        Admission::new(
            &[FORGEJO],
            plain(Kind::GitHub(github_given)),
            Unusable::GitHubGiven {
                forge: GITHUB,
                refused: crate::github::given::Refused { place: crate::github::given::Place::Reachable, other: None },
            },
        ),
        Admission::new(
            &[FORGEJO],
            plain(Kind::GitHub(reachable)),
            Unusable::Reachable { forge: GITHUB, reachable: 33, inner: 32 },
        ),
        documents,
        metadata,
    ] {
        assert_eq!(Host::new(&case.limits, case.settings(), host::crypto()).err(), Some(case.expected));
        let mut w = World::with(case.limits, host::settings(case.kept));
        w.run(RUN, reconcile(FORGEJO));
        let deadline = w.host.next_deadline();
        assert_eq!(w.host.add(&case.limits, case.forge()), Err(case.expected));
        assert_eq!(w.host.next_deadline(), deadline);
        w.finish_reconcile(RUN);
        assert_eq!(w.up.pop(), Some(Event::Done { run: RUN, outcome: fixtures::done() }));
        for kept in case.kept {
            w.host.remove(*kept);
        }
        // Filling both slots again proves a refused candidate kept no entry.
        w.add(forgejo(FORGEJO));
        w.add(forgejo(OTHER));
        w.run(RUN, reconcile(OTHER));
        w.finish_reconcile(RUN);
        assert_eq!(w.up.pop(), Some(Event::Done { run: RUN, outcome: fixtures::done() }));
        assert!(w.up.is_empty());
        assert!(w.io.is_empty());
    }
}

#[test]
fn removing_a_forge_ends_its_runs_unheard_and_keeps_the_others() {
    for socket_known in [false, true] {
        let mut w = World::new(host::settings(&[FORGEJO, OTHER]));
        w.run(RUN, reconcile(FORGEJO));
        w.read(RUN, Item::AdminToken, fixtures::ADMIN);
        let owner = w.connection();
        if socket_known {
            w.event(skein_io::Event::Connecting { owner, socket: SOCKET });
        }
        w.run(SECOND_RUN, reconcile(OTHER));
        w.host.remove(FORGEJO);
        assert!(w.host.next_deadline().is_some(), "the abort is driven without another owner request");
        w.fire();
        if !socket_known {
            w.event(skein_io::Event::Connecting { owner, socket: SOCKET });
        }
        assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: SOCKET }));
        w.answer(RUN, Answer::Written);
        w.run(THIRD_RUN, reconcile(FORGEJO));
        assert_eq!(w.up.pop(), Some(Event::Need { run: SECOND_RUN, need: Need::Read(Item::AdminToken) }));
        assert_eq!(w.up.pop(), Some(unknown(THIRD_RUN, FORGEJO, Stage::Admin, b"forge")));
        assert!(w.up.is_empty(), "removed run ends unheard");
        w.run(THIRD_RUN, reachable(OTHER));
        assert_eq!(
            w.up.pop(),
            Some(Event::Stop {
                run: THIRD_RUN,
                failure: Failure {
                    forge: OTHER,
                    step: Stage::Admin,
                    subject: b"kit-admin".as_slice().into(),
                    why: Why::Busy
                },
            }),
            "the aborted connection keeps its slot until io settles"
        );
        w.event(skein_io::Event::Closed { owner });
        w.answer(SECOND_RUN, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
        w.finish_after_admin(SECOND_RUN);
        assert_eq!(w.up.pop(), Some(Event::Done { run: SECOND_RUN, outcome: fixtures::done() }));
        assert!(w.up.is_empty());
        assert!(w.io.is_empty());
        assert_eq!(w.host.next_deadline(), None);
        w.add(forgejo(FORGEJO));
        w.run(RUN, reconcile(FORGEJO));
        w.finish_reconcile(RUN);
        assert_eq!(w.up.pop(), Some(Event::Done { run: RUN, outcome: fixtures::done() }));
    }
}

#[test]
fn removal_ends_both_a_writer_and_a_picker() {
    let mut w = World::new(host::settings(&[FORGEJO, OTHER]));
    w.run(RUN, reconcile(FORGEJO));
    w.run(SECOND_RUN, reachable(FORGEJO));
    for run in [RUN, SECOND_RUN] {
        assert_eq!(w.up.pop(), Some(Event::Need { run, need: Need::Read(Item::AdminToken) }));
    }
    w.host.remove(FORGEJO);
    for run in [RUN, SECOND_RUN] {
        w.answer(run, Answer::Read(Some(fixtures::secret(fixtures::ADMIN))));
    }
    assert!(w.up.is_empty(), "neither removed run reports an event");
    assert!(w.io.is_empty(), "late answers start no connection");
    assert_eq!(w.host.next_deadline(), None);
    w.run(THIRD_RUN, reconcile(OTHER));
    w.finish_reconcile(THIRD_RUN);
    assert_eq!(w.up.pop(), Some(Event::Done { run: THIRD_RUN, outcome: fixtures::done() }));
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
}

#[test]
fn a_removed_forges_state_is_unknown() {
    let mut w = World::new(host::settings(&[]));
    w.add(github(GITHUB));
    w.add(github(OTHER));
    let removed = w.state(GITHUB);
    let kept = w.state(OTHER);
    w.host.remove(GITHUB);
    w.host.remove(GITHUB);
    w.run(RUN, convert(OTHER, &removed));
    assert_eq!(w.up.pop(), Some(unknown(RUN, OTHER, Stage::State, b"convert")));
    w.add(github(GITHUB));
    w.run(RUN, convert(GITHUB, &removed));
    assert_eq!(w.up.pop(), Some(unknown(RUN, GITHUB, Stage::State, b"convert")), "removal erased the binding");
    w.run(RUN, convert(OTHER, &kept));
    let owner = w.connection();
    assert!(w.up.is_empty(), "the other forge's state still admits a conversion");
    w.host.remove(OTHER);
    w.fire();
    w.event(skein_io::Event::Connecting { owner, socket: SOCKET });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: SOCKET }));
    w.event(skein_io::Event::Closed { owner });
    assert!(w.up.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}

#[test]
fn a_run_for_an_unknown_forge_stops_unknown() {
    let mut w = World::new(host::settings(&[]));
    w.host.remove(OTHER);
    for (plan, step) in [(reconcile(OTHER), Stage::Admin), (manifest(OTHER), Stage::App)] {
        w.run(RUN, plan);
        assert_eq!(w.up.pop(), Some(unknown(RUN, OTHER, step, b"forge")));
        assert!(w.up.is_empty());
        assert!(w.io.is_empty());
        assert_eq!(w.host.next_deadline(), None);
    }
}

#[test]
fn a_closing_host_ignores_forge_changes() {
    let mut w = World::new(host::settings(&[FORGEJO]));
    w.run(RUN, reconcile(FORGEJO));
    w.read(RUN, Item::AdminToken, fixtures::ADMIN);
    let owner = w.connection();
    w.down(Request::Close);
    let mut invalid = fixtures::given();
    invalid.bot_login = Box::new([]);
    assert_eq!(w.host.add(&w.env.limits, plain(Kind::Forgejo(invalid))), Ok(()));
    w.host.remove(FORGEJO);
    w.event(skein_io::Event::Connecting { owner, socket: SOCKET });
    assert_eq!(w.io.pop(), Some(skein_io::Request::Abort { entity: SOCKET }));
    w.event(skein_io::Event::Closed { owner });
    assert_eq!(w.up.pop(), Some(Event::Closed));
    w.run(RUN, reconcile(FORGEJO));
    w.fire();
    assert!(w.up.is_empty());
    assert!(w.io.is_empty());
    assert_eq!(w.host.next_deadline(), None);
}
