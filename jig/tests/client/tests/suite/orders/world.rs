//! A surface world with a scripted caller, held transport and small-store
//! completions. The loop observes only requests; no client state is inspected
//! (jig/docs/design/client/domain/testing.md, 4).
use jig_client::{self as client, Down, Event, Message, Told, Up, contract as c};
use jig_client_protocol_wire::v19 as w;
use jig_client_test_domain::{self as root, first};
use jig_client_world::test;
use skein_lib::{Env, List, Queue, Reader, RequestEnvelope, RequestRecord, Secret, Time, Token, Wall, Writer};
use skein_world::domain::{Ledger, Schedule};
use std::collections::{BTreeMap, VecDeque};

pub const PARTY: u64 = 7;
const SEED: u64 = 8601;
const STREAM: [u8; 16] = [7; 16];
// The story holds three successive family pages, each with a read and watch,
// alongside the me watch, one inbox watch and its two reads: 6 + 1 + 3.
const ROOT_EXCHANGES: u32 = 10;
// Retention belongs to the supplied peer document, not the world's deadline.
const RETENTION: skein_lib::Duration = skein_lib::Duration::from_secs(60);
pub const PAGE_A: first::Address = first::Address { project: 1, item: 10 };
pub const PAGE_B: first::Address = first::Address { project: 1, item: 11 };
pub const PAGE_C: first::Address = first::Address { project: 1, item: 12 };

pub const OLD_DRAFT: u64 = 30;
pub const MIDDLE_DRAFT: u64 = 31;
const NEW_DRAFT: u64 = 32;
// Three restored drafts compete for the focused fixture's one retained draft.
const RESTORED_DRAFTS: u32 = 3;

pub enum Mode {
    Linked(client::Limits),
    Unlinked(client::Limits, List<RequestRecord<client::Held>>),
    Root(List<root::store::Record>),
}

impl Mode {
    pub fn root() -> Self {
        Self::Root(List::with_capacity(0))
    }

    pub fn with_drafts() -> Self {
        let mut records = Vec::new();
        for (task, touched) in [(OLD_DRAFT, 1), (MIDDLE_DRAFT, 2), (NEW_DRAFT, 3)] {
            records.push(root::store::Record::Jig(jig_client_view::store::Record::Draft {
                version: jig_client_view::store::STORE_VERSION,
                party: PARTY,
                task,
                words: c::Text::new(Box::from(*b"draft"), test::limits().jig.text).expect("fixture words"),
                touched,
            }));
        }
        Self::Root(test::list(records))
    }
}

enum Machine {
    Client(Box<client::Client>),
    Root(Box<root::Domain>, Box<root::Limits>),
}

pub enum Command {
    Client(Event),
    Go(first::Address),
    JigInbox,
    AdmitMe,
    ReplyRead(u32),
    ReleaseReads,
    Stored(bool),
    AckAll,
    Fire(Time),
    Visit,
    LoseRequest([u8; 16]),
    FinishRequest([u8; 16]),
    FinishRequests,
}

impl Command {
    pub fn credentials() -> Self {
        Self::Client(Event::SignIn { with: client::With::Held(Secret::new(Box::new([9; 40]))) })
    }

    pub fn watch(handle: client::Handle, task: u64) -> Self {
        Self::Client(Event::Watch { handle, subject: client::Watchable::Jig(c::Subject::Task { task }) })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    OpenStream,
    CloseStream,
    Told(Told),
    Save([u8; 16]),
    Erase([u8; 16]),
    Send([u8; 16]),
    Read(u32),
    ReadFamily(u64),
    InboxRead(w::InboxReadGroup),
    InboxWatch,
    WatchFamily(u64),
    WatchJig(w::Subject),
    ChildSave(root::store::Record),
    ChildErase(root::store::Key),
    StreamSaved,
    Unwatch,
}

struct Exchange {
    token: Token,
    kind: ExchangeKind,
}

enum ExchangeKind {
    Me(u64),
    Read(u32),
    OtherRead,
    Request([u8; 16]),
    Watch,
}

enum ChildWrite {
    Save(root::store::Record),
    Erase(root::store::Key),
}

struct Write {
    saves: Vec<client::Record>,
    erases: Vec<Box<[u8]>>,
}

/// Store and peer ledgers are external to the client; completions are held
/// until the script releases them, so later entrances remain ready under load.
pub struct World {
    machine: Machine,
    now: Time,
    trace: Vec<(&'static str, Edge)>,
    exchanges: Vec<Exchange>,
    completed_reads: Vec<Token>,
    writes: VecDeque<Write>,
    durable: BTreeMap<Box<[u8]>, client::Record>,
    recorded: BTreeMap<[u8; 16], Option<Wall>>,
    readers: Ledger<client::Handle, ()>,
    requests: Ledger<client::Key, ()>,
    child_records: Vec<root::store::Record>,
    child_pending: Vec<ChildWrite>,
    label: &'static str,
    unlinked: bool,
}

impl World {
    /// Yields at the scripted observation boundary; transport completions may
    /// remain held while the caller's ready entrances have all been delivered.
    pub fn run(mode: Mode, script: Vec<(&'static str, Command)>) -> Self {
        let (machine, unlinked, kept, root_kept) = match mode {
            Mode::Linked(limits) => (
                Machine::Client(Box::new(client::Client::new(&limits, SEED).expect("world bounds"))),
                false,
                List::with_capacity(0),
                List::with_capacity(0),
            ),
            Mode::Unlinked(limits, kept) => (
                Machine::Client(Box::new(client::Client::unlinked(&limits, SEED).expect("world bounds"))),
                true,
                kept,
                List::with_capacity(0),
            ),
            Mode::Root(root_kept) => {
                let mut limits = test::limits();
                limits.client.exchanges = ROOT_EXCHANGES;
                limits.jig.records = RESTORED_DRAFTS;
                (
                    Machine::Root(Box::new(root::new(&limits, SEED)), Box::new(limits)),
                    false,
                    List::with_capacity(0),
                    root_kept,
                )
            }
        };
        let mut world = Self {
            machine,
            now: Time::ZERO,
            trace: Vec::new(),
            exchanges: Vec::new(),
            completed_reads: Vec::new(),
            writes: VecDeque::new(),
            durable: BTreeMap::new(),
            recorded: BTreeMap::new(),
            readers: Ledger::new("current read"),
            requests: Ledger::new("keyed request"),
            child_records: Vec::new(),
            child_pending: Vec::new(),
            label: "start",
            unlinked,
        };
        for record in &kept {
            world.recorded.insert(record.key, record.first_sent);
            world.requests.open(record.request.key, ());
        }
        for record in &root_kept {
            world.child_records.push(record.clone());
        }
        world.start(kept, root_kept);
        let mut schedule = Schedule::new();
        for entrance in script {
            schedule.send(Time::ZERO, entrance);
        }
        while let Some((label, command)) = schedule.next(world.now) {
            world.label = label;
            world.execute(command);
        }
        world.readers.assert_settled();
        world.requests.assert_settled();
        assert!(world.writes.is_empty() && world.child_pending.is_empty(), "every store completion delivered");
        world
    }

    fn start(&mut self, kept: List<RequestRecord<client::Held>>, root_kept: List<root::store::Record>) {
        if matches!(self.machine, Machine::Root(..)) {
            self.root_event(root::Event::Start {
                address: root::Address::First(PAGE_A),
                kept: root_kept,
                offset: jig_client_view::boundary::Offset { minutes: 0 },
            });
        } else if self.unlinked {
            self.deliver(Event::StartUnlinked { kept });
        } else {
            self.deliver(Event::Start { kept: List::with_capacity(0) });
        }
        self.label = "sign_in";
        let with =
            if self.unlinked { client::With::Transport } else { client::With::Held(Secret::new(Box::new([7; 40]))) };
        self.deliver(Event::SignIn { with });
        self.label = "hello";
        if self.unlinked {
            self.deliver(Event::Up { up: Up::Served { served: true, versions: None } });
        } else {
            self.deliver(Event::Streamed { message: client::Streamed::Record(hello()) });
        }
        self.label = "me";
        let watch = self.me().1;
        if self.unlinked {
            let limits = test::limits().client.domain;
            let bounds = jig_client_protocol_link::Limits::new(limits, ROOT_EXCHANGES).expect("world bounds");
            let frame = jig_client_protocol_link::translate::documents::frame(frame(), &bounds).expect("peer frame");
            self.deliver(Event::Up {
                up: Up::Watched {
                    watch: Token::new(watch),
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))),
                },
            });
        } else {
            self.deliver(Event::Streamed { message: client::Streamed::Record(delivery(watch)) });
        }
        self.label = "initial_store";
        self.ack_all();
    }

    fn execute(&mut self, command: Command) {
        match command {
            Command::Client(event) => self.deliver(event),
            Command::Go(address) => {
                self.root_event(root::Event::Went { place: root::Place::Address(root::Address::First(address)) });
            }
            Command::JigInbox => self.root_event(root::Event::Went {
                place: root::Place::Address(root::Address::Jig(jig_client_view::address::Address::Inbox {
                    project: None,
                })),
            }),
            Command::AdmitMe => {
                let (exchange, _) = self.me();
                self.deliver(Event::Reply { exchange, message: open_answer() });
                self.remove(exchange);
            }
            Command::ReplyRead(project) => {
                let token = self.read_exchange(project);
                self.complete_read(token);
            }
            Command::ReleaseReads => {
                let mut tokens = self.completed_reads.clone();
                for exchange in &self.exchanges {
                    if matches!(exchange.kind, ExchangeKind::Read(_) | ExchangeKind::OtherRead) {
                        tokens.push(exchange.token);
                    }
                }
                for token in tokens {
                    self.complete_read(token);
                }
            }
            Command::Stored(stored) => self.stored(stored),
            Command::AckAll => self.ack_all(),
            Command::Fire(now) => {
                self.now = now;
                let mut out = Queue::uncapped();
                match &mut self.machine {
                    Machine::Client(machine) => machine.fire(now, Wall::EPOCH, &mut out),
                    Machine::Root(machine, limits) => {
                        let env = Env { now, wall: Wall::EPOCH, limits: **limits };
                        let mut root_out = Queue::uncapped();
                        root::fire(machine, &env, &mut root_out);
                        self.root_outputs(&mut root_out);
                    }
                }
                self.outputs(&mut out);
            }
            Command::Visit => self.deliver(Event::Up { up: Up::Served { served: true, versions: None } }),
            Command::LoseRequest(key) => self.request_answer(key, false),
            Command::FinishRequest(key) => self.request_answer(key, true),
            Command::FinishRequests => {
                let mut keys = Vec::new();
                for exchange in &self.exchanges {
                    if let ExchangeKind::Request(key) = exchange.kind {
                        keys.push(key);
                    }
                }
                for key in keys {
                    self.request_answer(key, true);
                }
            }
        }
    }

    fn deliver(&mut self, event: Event) {
        if let Event::Read { handle, .. } = &event {
            self.readers.take(*handle);
            self.readers.open(*handle, ());
        } else if let Event::Unwatch { handle } = &event {
            self.readers.take(*handle);
        } else if let Event::Send { key, .. } = &event {
            self.requests.open(*key, ());
        }
        match &mut self.machine {
            Machine::Client(machine) => {
                let mut out = Queue::uncapped();
                machine.step(self.now, Wall::EPOCH, event, &mut out);
                self.outputs(&mut out);
                let Machine::Client(machine) = &mut self.machine else { unreachable!("client mode") };
                machine.reclaim();
            }
            Machine::Root(..) => self.root_event(root::Event::Client { event }),
        }
    }

    fn root_event(&mut self, event: root::Event) {
        let Machine::Root(machine, limits) = &mut self.machine else { panic!("root entrance") };
        let env = Env { now: self.now, wall: Wall::EPOCH, limits: **limits };
        let mut out = Queue::uncapped();
        root::step(machine, &env, event, &mut out);
        root::take(machine, &env, &mut out);
        self.root_outputs(&mut out);
    }

    fn root_outputs(&mut self, out: &mut Queue<root::Request>) {
        let mut client_out = Queue::uncapped();
        let mut saves = Vec::new();
        let mut erases = Vec::new();
        while let Some(request) = out.pop() {
            match request {
                root::Request::Client { request } => {
                    client_out.push(request);
                    self.outputs(&mut client_out);
                }
                root::Request::Save { record: root::store::Record::Client(record) } => {
                    self.saved(&record);
                    saves.push(record);
                }
                root::Request::Erase { key: root::store::Key::Client(key) } => {
                    self.erased(&key);
                    erases.push(key);
                }
                root::Request::Save { record } => {
                    self.child_pending.push(ChildWrite::Save(record.clone()));
                    self.push(Edge::ChildSave(record));
                }
                root::Request::Erase { key } => {
                    self.child_pending.push(ChildWrite::Erase(key.clone()));
                    self.push(Edge::ChildErase(key));
                }
                root::Request::Show { .. } => {}
                other @ (root::Request::First(_)
                | root::Request::SignIn { .. }
                | root::Request::SignOut
                | root::Request::Fact { .. }) => panic!("unexpected root duty: {other:?}"),
            }
        }
        if !saves.is_empty() {
            self.writes.push_back(Write { saves, erases: Vec::new() });
        }
        if !erases.is_empty() {
            self.writes.push_back(Write { saves: Vec::new(), erases });
        }
    }

    fn outputs(&mut self, out: &mut Queue<client::Request>) {
        while let Some(request) = out.pop() {
            match request {
                client::Request::OpenStream { .. } => self.push(Edge::OpenStream),
                client::Request::CloseStream => self.push(Edge::CloseStream),
                client::Request::Told { told } => {
                    match &told {
                        Told::Page { handle, .. } => self.readers.end(*handle),
                        Told::Answered { key, .. } => self.requests.end(*key),
                        Told::Link { .. }
                        | Told::Pending { .. }
                        | Told::Kept { .. }
                        | Told::Dropped { .. }
                        | Told::Changed { .. }
                        | Told::Text { .. }
                        | Told::Ended { .. } => {}
                    }
                    self.push(Edge::Told(told));
                }
                client::Request::Save { records } => {
                    let mut saves = Vec::new();
                    for record in records.into_boxed() {
                        self.saved(&record);
                        saves.push(record);
                    }
                    self.writes.push_back(Write { saves, erases: Vec::new() });
                }
                client::Request::Erase { records } => {
                    let mut erases = Vec::new();
                    for key in records.into_boxed() {
                        self.erased(&key);
                        erases.push(key);
                    }
                    self.writes.push_back(Write { saves: Vec::new(), erases });
                }
                client::Request::Exchange { exchange, message, .. } => self.wire(exchange, message),
                client::Request::Down { down } => self.down(down),
                other @ (client::Request::SignOut
                | client::Request::Authorized(_)
                | client::Request::ServiceCredential { .. }) => panic!("unexpected client duty: {other:?}"),
            }
        }
    }

    fn wire(&mut self, token: Token, message: Message) {
        let kind = match message {
            Message::Watch(bytes) => {
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("open record");
                match open.subject().owner() {
                    w::Owner::Jig => {
                        let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record()))
                            .expect("jig subject");
                        let me = matches!(subject, w::Subject::Me);
                        if matches!(subject, w::Subject::Inbox(_)) {
                            self.push(Edge::InboxWatch);
                        } else {
                            self.push(Edge::WatchJig(subject));
                        }
                        if me { ExchangeKind::Me(open.watch()) } else { ExchangeKind::Watch }
                    }
                    w::Owner::Family(_) => {
                        self.push(Edge::WatchFamily(family_item(open.subject().record(), false)));
                        ExchangeKind::Watch
                    }
                }
            }
            Message::Read(bytes) => {
                let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("read envelope");
                match owned.owner() {
                    w::Owner::Jig => {
                        let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).expect("jig read");
                        if let w::Read::Project(read) = read {
                            self.push(Edge::Read(read.project()));
                            ExchangeKind::Read(read.project())
                        } else if let w::Read::Inbox(read) = read {
                            self.push(Edge::InboxRead(read.group().clone()));
                            ExchangeKind::OtherRead
                        } else {
                            panic!("scripted project or inbox read")
                        }
                    }
                    w::Owner::Family(_) => {
                        self.push(Edge::ReadFamily(family_item(owned.record(), true)));
                        ExchangeKind::OtherRead
                    }
                }
            }
            Message::Request(bytes) => {
                let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("keyed record");
                assert!(self.durable.contains_key(keyed.key().as_slice()), "record durable before its send");
                self.push(Edge::Send(*keyed.key()));
                ExchangeKind::Request(*keyed.key())
            }
            Message::Unwatch(_) => {
                self.push(Edge::Unwatch);
                return;
            }
            other @ (Message::StreamGone(_)
            | Message::SignInLink(_)
            | Message::SignInService
            | Message::SignOut
            | Message::Authorize(_)) => panic!("unexpected exchange: {other:?}"),
        };
        for exchange in &self.exchanges {
            assert_ne!(exchange.token, token, "one operation per token");
        }
        self.exchanges.push(Exchange { token, kind });
    }

    fn down(&mut self, down: Down) {
        match down {
            Down::Save { record } => {
                self.recorded.insert(record.key, record.first_sent);
                self.push(Edge::Save(record.key));
            }
            Down::Erase { key } => {
                self.recorded.remove(&key);
                self.push(Edge::Erase(key));
            }
            Down::Send { attempt, key, .. } => {
                assert!(matches!(self.recorded.get(&key), Some(Some(_))), "first-send record before its send");
                self.push(Edge::Send(key));
                self.exchanges.push(Exchange { token: attempt, kind: ExchangeKind::Request(key) });
            }
            Down::Open { watch, subject: client::Watchable::Jig(c::Subject::Me) } => {
                self.exchanges.push(Exchange { token: watch, kind: ExchangeKind::Me(watch.raw()) });
            }
            other @ (Down::Open { .. } | Down::Continue { .. } | Down::Close { .. } | Down::Read { .. }) => {
                panic!("unexpected unlinked duty: {other:?}")
            }
        }
    }

    fn saved(&mut self, record: &client::Record) {
        if record.key.as_ref() == jig_client_protocol_link::STREAM_KEY {
            self.push(Edge::StreamSaved);
        } else {
            self.push(Edge::Save(record.key.as_ref().try_into().expect("request record key")));
        }
    }

    fn erased(&mut self, key: &[u8]) {
        self.push(Edge::Erase(key.try_into().expect("request erase key")));
    }

    fn stored(&mut self, stored: bool) {
        let write = self.writes.pop_front().expect("one completion for a held write");
        if stored {
            for record in write.saves {
                self.durable.insert(record.key.clone(), record);
            }
            for key in write.erases {
                self.durable.remove(&key);
            }
        }
        self.deliver(if stored { Event::Stored } else { Event::NotStored });
    }

    fn ack_all(&mut self) {
        while !self.writes.is_empty() {
            self.stored(true);
        }
        for write in core::mem::take(&mut self.child_pending) {
            let (key, record) = match write {
                ChildWrite::Save(record) => (child_key(&record), Some(record)),
                ChildWrite::Erase(key) => (key, None),
            };
            let before = core::mem::take(&mut self.child_records);
            for saved in before {
                if child_key(&saved) != key {
                    self.child_records.push(saved);
                }
            }
            if let Some(record) = record {
                self.child_records.push(record);
            }
        }
    }

    fn me(&self) -> (Token, u64) {
        for exchange in &self.exchanges {
            if let ExchangeKind::Me(watch) = exchange.kind {
                return (exchange.token, watch);
            }
        }
        panic!("the peer received the own watch")
    }

    fn read_exchange(&self, project: u32) -> Token {
        for exchange in &self.exchanges {
            if matches!(exchange.kind, ExchangeKind::Read(number) if number == project) {
                return exchange.token;
            }
        }
        panic!("peer has project {project} demand")
    }

    fn complete_read(&mut self, exchange: Token) {
        self.deliver(Event::Reply { exchange, message: read_answer() });
        self.remove(exchange);
        if !self.completed_reads.contains(&exchange) {
            self.completed_reads.push(exchange);
        }
    }

    fn request_answer(&mut self, key: [u8; 16], final_answer: bool) {
        let mut token = None;
        for exchange in &self.exchanges {
            if matches!(exchange.kind, ExchangeKind::Request(name) if name == key) {
                token = Some(exchange.token);
            }
        }
        let exchange = token.expect("peer received the keyed request");
        if self.unlinked {
            self.deliver(Event::Up {
                up: Up::Answer {
                    attempt: exchange,
                    envelope: if final_answer { RequestEnvelope::Final } else { RequestEnvelope::Lost },
                    answer: if final_answer {
                        Some(client::Answered::Jig(c::Answer::Done(c::Outcome::Changed)))
                    } else {
                        None
                    },
                    why: None,
                },
            });
        } else if final_answer {
            self.deliver(Event::Reply { exchange, message: keyed_answer() });
        } else {
            self.deliver(Event::Lost { exchange });
        }
        self.remove(exchange);
    }

    fn remove(&mut self, token: Token) {
        let held = core::mem::take(&mut self.exchanges);
        for exchange in held {
            if exchange.token != token {
                self.exchanges.push(exchange);
            }
        }
    }
    fn push(&mut self, edge: Edge) {
        self.trace.push((self.label, edge));
    }

    pub fn at(&self, label: &str) -> Vec<Edge> {
        let mut rows = Vec::new();
        for (entrance, edge) in &self.trace {
            if *entrance == label {
                rows.push(edge.clone());
            }
        }
        rows
    }

    pub fn reads(&self) -> Vec<u32> {
        let mut reads = Vec::new();
        for (_, edge) in &self.trace {
            if let Edge::Read(project) = edge {
                reads.push(*project);
            }
        }
        reads
    }

    pub fn sends(&self) -> Vec<[u8; 16]> {
        let mut sends = Vec::new();
        for (_, edge) in &self.trace {
            if let Edge::Send(key) = edge {
                sends.push(*key);
            }
        }
        sends
    }

    pub fn sent_at(&self, name: [u8; 16]) -> &'static str {
        for (label, edge) in &self.trace {
            if *edge == Edge::Send(name) {
                return label;
            }
        }
        panic!("request was sent")
    }

    pub fn answers(&self) -> Vec<(client::Key, client::Outcome)> {
        let mut answers = Vec::new();
        for (_, edge) in &self.trace {
            if let Edge::Told(Told::Answered { key, outcome }) = edge {
                answers.push((*key, outcome.clone()));
            }
        }
        answers
    }

    pub fn records_and_sends(&self) -> Vec<Edge> {
        let mut rows = Vec::new();
        for (_, edge) in &self.trace {
            if matches!(edge, Edge::Save(_) | Edge::Erase(_) | Edge::Send(_)) {
                rows.push(edge.clone());
            }
        }
        rows
    }
}

pub fn record(key: client::Key, name: [u8; 16], first_sent: Option<Wall>) -> RequestRecord<client::Held> {
    RequestRecord { key: name, scope: PARTY, first_sent, request: client::Held { key, request: stop() } }
}

pub fn page_outputs(address: first::Address) -> Vec<Edge> {
    vec![
        Edge::ChildSave(root::store::Record::First(first::Record::Place { version: 1, party: PARTY, address })),
        Edge::ReadFamily(address.item),
        Edge::WatchFamily(address.item),
    ]
}

pub fn inbox_outputs() -> Vec<Edge> {
    vec![
        Edge::ChildSave(root::store::Record::Jig(jig_client_view::store::Record::Place {
            version: jig_client_view::store::STORE_VERSION,
            party: PARTY,
            address: jig_client_view::address::Address::Inbox { project: None },
        })),
        Edge::InboxWatch,
        Edge::InboxRead(w::InboxReadGroup::Needs),
        Edge::InboxRead(w::InboxReadGroup::Updates),
    ]
}

fn family_item(bytes: &[u8], read: bool) -> u64 {
    use jig_client_test_protocol_wire::v2 as f;
    let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("family envelope");
    let mut reader = Reader::new(envelope.record());
    let subject = if read {
        f::Read::decode(&f::CEILINGS, &mut reader).expect("family read").into_parts().value
    } else {
        f::Subject::decode(&f::CEILINGS, &mut reader).expect("family subject").into_parts().value
    };
    let f::FamilyKind::Item(item) = subject.kind() else { panic!("family item") };
    item.value()
}

fn frame() -> w::Me {
    let mut parts = jig_client_test_records::frame_record().into_parts();
    parts.retention = RETENTION;
    w::Me::new(&w::CEILINGS, parts).expect("peer frame")
}

fn hello() -> Box<[u8]> {
    let mut served = Vec::new();
    for (owner, version) in [
        (w::Owner::Jig, jig_client_protocol_wire::VERSION),
        (
            w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number: root::FIRST }).expect("owner")),
            jig_client_test_protocol_wire::VERSION,
        ),
        (
            w::Owner::Family(w::Family::new(&w::CEILINGS, w::FamilyParts { number: root::SECOND }).expect("owner")),
            jig_client_test_protocol_wire::VERSION,
        ),
    ] {
        served.push(
            w::Served::new(&w::CEILINGS, w::ServedParts { owner, first: version, last: version })
                .expect("served owner"),
        );
    }
    let event = w::StreamEvent::Hello(
        w::Hello::new(
            &w::CEILINGS,
            w::HelloParts {
                stream: STREAM,
                served: test::list(served),
                heartbeat: test::limits().client.domain.heartbeat,
            },
        )
        .expect("peer hello"),
    );
    let mut writer = Writer::new(usize::try_from(event.measure()).expect("measured event"));
    event.encode(&mut writer).expect("hello bytes");
    writer.finish()
}

fn delivery(watch: u64) -> Box<[u8]> {
    let frame = w::DeliveryValue::Me(frame());
    let mut writer = Writer::new(usize::try_from(frame.measure()).expect("measured frame"));
    frame.encode(&mut writer).expect("frame bytes");
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
    )
    .expect("owned frame");
    let event = w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch, document }).expect("delivery"),
    );
    let mut writer = Writer::new(usize::try_from(event.measure()).expect("measured delivery"));
    event.encode(&mut writer).expect("delivery bytes");
    writer.finish()
}

fn open_answer() -> Box<[u8]> {
    let answer = w::OpenAnswer::Admitted;
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("measured answer"));
    answer.encode(&mut writer).expect("open bytes");
    writer.finish()
}

fn read_answer() -> Box<[u8]> {
    let answer = w::ReadAnswer::None(
        w::ReadAnswerNone::new(&w::CEILINGS, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown })
            .expect("read none"),
    );
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("measured answer"));
    answer.encode(&mut writer).expect("read bytes");
    writer.finish()
}

fn keyed_answer() -> Box<[u8]> {
    let final_answer =
        w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome: w::Outcome::Changed }).expect("done"));
    let mut writer = Writer::new(usize::try_from(final_answer.measure()).expect("measured answer"));
    final_answer.encode(&mut writer).expect("final bytes");
    let owned = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
    )
    .expect("owned answer");
    let answer = w::KeyedAnswer::FinalAnswer(owned);
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("measured keyed answer"));
    answer.encode(&mut writer).expect("keyed bytes");
    writer.finish()
}

fn child_key(record: &root::store::Record) -> root::store::Key {
    match record {
        root::store::Record::Jig(record) => root::store::Key::Jig(record.key()),
        root::store::Record::First(first::Record::Place { .. }) => root::store::Key::First(first::Key::Place),
        root::store::Record::Client(record) => root::store::Key::Client(record.key.clone()),
        root::store::Record::Unlinked(record) => root::store::Key::Unlinked(record.key),
    }
}

pub fn stop() -> client::Ask {
    client::Ask::Jig(Box::new(c::Ask::Stop { project: super::PROJECT, task: super::TASK }))
}
