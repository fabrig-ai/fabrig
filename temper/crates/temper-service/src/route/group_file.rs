//! Group files written before a box start (temper/docs/plan/playground-reset-r4/notes/contracts.md, 3).
use crate::engine::Engine;
use alloc::boxed::Box;
use skein_io::{
    file,
    file_layer::{self, FileIo},
    kernel::{Complete, Fd, Op, Submit},
};
use skein_json::writer::{Encoder, Limits};
use skein_lib::{Decimal, Duration, Queue, Time, Token};
use temper_engine_domain_forge::record;

pub(crate) fn render(number: u16, setup: &record::Setup, playground: bool) -> Box<[u8]> {
    let limits = Limits { depth: 8, length: u32::MAX };
    let mut encoder = Encoder::measure(&limits);
    encode(&mut encoder, number, setup, playground);
    let length = encoder.measured().expect("checked setup record is JSON text");
    let mut encoder = Encoder::write(length, &limits);
    encode(&mut encoder, number, setup, playground);
    encoder.finish()
}

fn encode(e: &mut Encoder, number: u16, setup: &record::Setup, playground: bool) {
    e.object_start();
    e.key(b"setup");
    e.object_start();
    e.key(b"forge");
    e.unsigned(u64::from(number));
    match &setup.mode {
        record::SetupMode::Owned(_) => {
            e.key(b"start");
            e.string(b"asked");
        }
        record::SetupMode::Fake(_) | record::SetupMode::Managed { .. } => {}
    }
    if !setup.directory.is_empty() {
        member(e, b"directory", &setup.directory);
    }
    e.key(b"bot");
    account(e, &setup.bot);
    e.key(b"mode");
    match &setup.mode {
        record::SetupMode::Managed { admin_token: None } => e.string(b"managed"),
        record::SetupMode::Managed { admin_token: Some(token) } => {
            e.object_start();
            e.key(b"managed");
            e.object_start();
            member(e, b"admin_token", token);
            e.object_end();
            e.object_end();
        }
        record::SetupMode::Owned(owned) => {
            e.object_start();
            e.key(b"owned");
            encode_owned(e, owned);
            e.object_end();
        }
        record::SetupMode::Fake(fake) => {
            e.object_start();
            e.key(b"fake");
            e.object_start();
            listen(e, fake.listen);
            member(e, b"git", &fake.git);
            member(e, b"owner", &fake.owner);
            e.key(b"repositories");
            e.array_start();
            for repository in &fake.repositories {
                e.object_start();
                member(e, b"name", &repository.name);
                member(e, b"landing", &repository.landing);
                e.object_end();
            }
            e.array_end();
            e.object_end();
            e.object_end();
        }
    }
    match &setup.mode {
        record::SetupMode::Managed { admin_token: Some(_) } => {
            e.key(b"orgs");
            strings(e, &setup.orgs);
        }
        record::SetupMode::Managed { admin_token: None } | record::SetupMode::Fake(_) | record::SetupMode::Owned(_) => {
        }
    }
    e.object_end();
    e.key(b"playground");
    e.boolean(playground);
    e.object_end();
}

fn encode_owned(e: &mut Encoder, owned: &record::OwnedEntry) {
    e.object_start();
    member(e, b"forgejo", if owned.fetch.forgejo { b"fetched" } else { &owned.forgejo });
    e.key(b"runner");
    if let Some(runner) = &owned.runner {
        e.object_start();
        member(e, b"binary", if owned.fetch.runner { b"fetched" } else { &runner.binary });
        e.key(b"labels");
        strings(e, &runner.labels);
        e.key(b"capacity");
        e.unsigned(u64::from(runner.capacity));
        member(e, b"path", &runner.path);
        e.object_end();
    } else {
        e.null();
    }
    listen(e, owned.listen);
    e.key(b"person");
    account(e, &owned.person);
    if let Some(mirror) = &owned.fetch.mirror {
        e.key(b"mirror");
        e.object_start();
        e.key(b"forgejo");
        origin(e, &mirror.forgejo);
        e.key(b"runner");
        origin(e, &mirror.runner);
        e.object_end();
    }
    if let Some(public) = &owned.public {
        e.key(b"public");
        origin(e, public);
    }
    e.object_end();
}

fn member(e: &mut Encoder, name: &[u8], value: &[u8]) {
    e.key(name);
    e.string(value);
}
fn strings(e: &mut Encoder, values: &[Box<[u8]>]) {
    e.array_start();
    for value in values {
        e.string(value);
    }
    e.array_end();
}
fn account(e: &mut Encoder, account: &record::Account) {
    e.object_start();
    member(e, b"login", &account.login);
    member(e, b"email", &account.email);
    e.object_end();
}
fn origin(e: &mut Encoder, origin: &record::Origin) {
    e.string_parts(&[b"https://", &origin.host, b":", Decimal::of(u64::from(origin.port)).as_bytes()]);
}
#[expect(clippy::disallowed_macros, reason = "socket address spelling is the setup file's protocol text")]
fn listen(e: &mut Encoder, address: core::net::SocketAddr) {
    e.key(b"listen");
    e.object_start();
    member(e, b"socket", alloc::format!("{address}").as_bytes());
    e.object_end();
}

const OWNER: Token = Token::new(1);
const MASK: u64 = (1_u64 << 60) - 1;
/// One file operation and its cancellation may be in flight together.
pub(crate) const OPERATIONS: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Opening,
    Ready,
    MakingBox,
    OpeningBox,
    MakingGroups,
    OpeningGroups,
    Storing,
    Failed,
}

/// The link owns its data root and a directory opened beneath it (skein/docs/design/io.md, 5).
pub(crate) struct Files {
    io: FileIo,
    above: Queue<file::Event>,
    below: Option<file::Request>,
    pub(crate) submitted: Queue<Submit>,
    data: Option<Token>,
    root: Option<Token>,
    box_directory: Option<Token>,
    groups: Option<Token>,
    state: State,
    pending: Option<(u16, Box<[u8]>)>,
    closing: bool,
}

impl Files {
    pub(crate) fn new(data: Fd, user: u32, deadline: Duration, seed: u64) -> Self {
        let mut io = FileIo::new(4, temper_forge_setup::file::BYTES, 1, deadline).with_effective_user(user);
        io.seed_randomness(seed);
        let token = io.adopt_root(data).expect("data directory slot");
        Self {
            io,
            above: Queue::uncapped(),
            below: Some(file::Request::OpenDirectory {
                owner: OWNER,
                root: data,
                name: b".".as_slice().into(),
                no_follow: true,
            }),
            submitted: Queue::with_capacity(4),
            data: Some(token),
            root: None,
            box_directory: None,
            groups: None,
            state: State::Opening,
            pending: None,
            closing: false,
        }
    }

    pub(crate) fn next_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }

    pub(crate) fn work_pending(&self, now: Time) -> bool {
        !self.above.is_empty()
            || !self.submitted.is_empty()
            || self.io.is_due(now)
            || self.io.takes()
                && (self.below.is_some()
                    || self.closing
                        && (self.groups.is_some()
                            || self.box_directory.is_some()
                            || self.root.is_some()
                            || self.data.is_some())
                    || self.pending.is_some() && self.state != State::Opening)
    }

    pub(crate) fn closed(&self) -> bool {
        self.closing
            && self.io.takes()
            && self.io.open_files() == 0
            && self.above.is_empty()
            && self.submitted.is_empty()
    }

    pub(crate) fn close(&mut self) {
        self.closing = true;
        self.below = None;
        self.pending = None;
    }

    pub(crate) fn complete(&mut self, mut complete: Complete) {
        complete.op = Token::new(complete.op.raw() & MASK);
        if let Op::Cancel { target } = &mut complete.kind {
            *target = Token::new(target.raw() & MASK);
        }
        let mut submitted = Queue::with_capacity(4);
        file_layer::up(&mut self.io, complete, &mut self.above, &mut submitted);
        self.publish(&mut submitted);
    }

    fn publish(&mut self, submitted: &mut Queue<Submit>) {
        for _ in 0..submitted.len() {
            let mut submit = submitted.pop().expect("group file submissions");
            submit.op = Token::new(submit.op.raw() | crate::tokens::BOX_FILES);
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::new(target.raw() | crate::tokens::BOX_FILES);
            }
            self.submitted.push(submit);
        }
    }

    fn iterate(&mut self, now: Time) -> Option<bool> {
        let mut submitted = Queue::with_capacity(4);
        file_layer::expire(&mut self.io, now, &mut submitted);
        let mut result = None;
        for _ in 0..self.above.len() {
            let event = self.above.pop().expect("group file events");
            if let Some(answer) = self.arrived(event) {
                result = Some(answer);
            }
        }
        if self.io.takes() {
            if self.closing {
                let file = if self.groups.is_some() {
                    self.groups.take()
                } else if self.box_directory.is_some() {
                    self.box_directory.take()
                } else if self.root.is_some() {
                    self.root.take()
                } else {
                    self.data.take()
                };
                if let Some(file) = file {
                    self.below = Some(file::Request::Close { owner: OWNER, file });
                }
            } else if self.pending.is_some() {
                match self.state {
                    State::Ready => {
                        self.state = State::MakingBox;
                        self.below = Some(self.directory(b"box"));
                    }
                    State::Failed => {
                        self.pending = None;
                        result = Some(false);
                    }
                    State::Opening
                    | State::MakingBox
                    | State::OpeningBox
                    | State::MakingGroups
                    | State::OpeningGroups
                    | State::Storing => {}
                }
            }
            if let Some(request) = self.below.take() {
                file_layer::down(&mut self.io, now, request, &mut self.above, &mut submitted);
            }
        } else if self.closing {
            file_layer::cancel(&mut self.io, OWNER, &mut submitted);
        }
        self.publish(&mut submitted);
        result
    }

    fn arrived(&mut self, event: file::Event) -> Option<bool> {
        match event {
            file::Event::Opened { file, .. } => {
                match self.state {
                    State::Opening => self.root = Some(file),
                    State::OpeningBox => self.box_directory = Some(file),
                    State::OpeningGroups => self.groups = Some(file),
                    State::Ready | State::MakingBox | State::MakingGroups | State::Storing | State::Failed => {
                        unreachable!("only directory opens return handles")
                    }
                }
                if self.closing {
                    return None;
                }
                match self.state {
                    State::Opening => self.state = State::Ready,
                    State::OpeningBox => {
                        self.state = State::MakingGroups;
                        self.below = Some(self.directory(b"groups"));
                    }
                    State::OpeningGroups => self.store(),
                    State::Ready | State::MakingBox | State::MakingGroups | State::Storing | State::Failed => {
                        unreachable!("only directory opens return handles")
                    }
                }
            }
            file::Event::Made { .. } => {
                if self.closing {
                    return None;
                }
                self.made();
            }
            file::Event::Stored { .. } => {
                if self.closing {
                    return None;
                }
                assert!(self.state == State::Storing, "Stored answers the group file's store");
                self.state = State::Ready;
                return Some(true);
            }
            file::Event::Failed { .. }
            | file::Event::Refused { .. }
            | file::Event::Cancelled { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. } => {
                if self.closing {
                    return None;
                }
                self.state = if self.state == State::Opening { State::Failed } else { State::Ready };
                self.pending = None;
                return Some(false);
            }
            file::Event::Closed { .. } => assert!(self.closing, "only shutdown closes a group directory"),
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Renamed { .. }
            | file::Event::Removed { .. }
            | file::Event::Listed { .. } => unreachable!("the group file's directory or store terminal"),
        }
        None
    }

    fn made(&mut self) {
        match self.state {
            State::MakingBox => {
                if self.box_directory.is_some() {
                    self.state = State::MakingGroups;
                    self.below = Some(self.directory(b"groups"));
                } else {
                    self.state = State::OpeningBox;
                    self.below = Some(file::Request::OpenPrivate {
                        owner: OWNER,
                        root: self.root.expect("opened data directory"),
                        path: b"box".as_slice().into(),
                    });
                }
            }
            State::MakingGroups => {
                if self.groups.is_some() {
                    self.store();
                } else {
                    self.state = State::OpeningGroups;
                    self.below = Some(file::Request::OpenPrivate {
                        owner: OWNER,
                        root: self.box_directory.expect("private box directory"),
                        path: b"groups".as_slice().into(),
                    });
                }
            }
            State::Opening
            | State::OpeningBox
            | State::OpeningGroups
            | State::Ready
            | State::Storing
            | State::Failed => {
                unreachable!("Made answers a directory request")
            }
        }
    }

    fn store(&mut self) {
        let (number, bytes) = self.pending.take().expect("one group file write");
        self.state = State::Storing;
        self.below = Some(file::Request::Store {
            owner: OWNER,
            root: self.groups.expect("private groups directory"),
            path: Decimal::of(u64::from(number)).as_bytes().into(),
            bytes,
            expected: file::Expect::Any,
            no_follow: true,
        });
    }

    fn directory(&self, path: &[u8]) -> file::Request {
        file::Request::MakeDirectory {
            owner: OWNER,
            root: if self.state == State::MakingGroups {
                self.box_directory.expect("private box directory")
            } else {
                self.root.expect("opened data directory")
            },
            path: path.into(),
            mode: 0o700,
        }
    }
}

pub(crate) fn write(engine: &mut Engine, number: u16, bytes: Box<[u8]>) {
    let files = engine.box_link.files.as_mut().expect("box file driver");
    assert!(files.pending.is_none(), "one group file write at a time");
    files.pending = Some((number, bytes));
}

pub(crate) fn drive(engine: &mut Engine, now: Time, closing: bool) {
    let Some(files) = &mut engine.box_link.files else { return };
    if closing {
        files.close();
    }
    if let Some(stored) = files.iterate(now) {
        let mut out = Queue::uncapped();
        engine.box_link.file_written(stored, &mut out);
        super::box_link::drain(engine, &mut out);
    }
}
