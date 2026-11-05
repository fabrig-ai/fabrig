//! Hand-steered component exchanges (skein/docs/design/forge-connection.md, 10).
use skein_forge_connection::Limits;
use skein_lib::Duration;
pub const WORD: &[u8] = b"kit";
pub const USER_AGENT: &[u8] = b"kit/0.0.0";
pub const OP_BYTES: u32 = 512;
// Twice the operation bound, the owner's word, and marker framing (forge.md 6).
pub const BODY_ROOM: u32 = 2 * OP_BYTES + 3 + 15;
pub const WRITER: u64 = 2;
#[must_use]
pub fn limits() -> Limits {
    Limits {
        forges: 2,
        token_bytes: 64,
        key_bytes: 1200,
        connections: 2,
        head_bytes: 4096,
        headers: 32,
        body_bytes_in: 1_048_576,
        depth: 16,
        chunk: 1024,
        string_bytes: 128,
        body_bytes: BODY_ROOM,
        inner: 8,
        target_bytes: 1024,
        repositories: 2,
        users: 8,
        installations: 3,
        log_origins: 2,
        deadline: Duration::from_secs(10),
        idle: Duration::from_secs(5),
        start_backoff: Duration::from_secs(1),
        start_backoff_max: Duration::from_secs(8),
        renew_before: Duration::from_secs(60),
        // Beyond token-renewal stories; puller stories advance this timer themselves.
        log_poll: Duration::from_secs(86_400),
        log_catch_up: Duration::from_secs(3600),
        log_pages: 10,
        delivery_bytes: 65_536,
        hooks: 4,
        hook_head_bytes: 8192,
        hook_body_bytes: 1_048_576,
        hook_deadline: Duration::from_secs(10),
        hook_secret_bytes: 128,
        tls: skein_tls::client::Limits { read: 16384, send: 16384, records: skein_tls::client::MAX_RECORD },
    }
}

use core::net::{Ipv4Addr, SocketAddr};
use skein_forge::api;
use skein_forge_connection::{
    Answered, Component, Event, Settings, Shared,
    settings::{Forge, Kind, Transport},
    start::Start,
};
use skein_lib::{Env, List, Queue, Secret, Time, Wall};
use std::boxed::Box;
pub const FORGE: u16 = 1;
pub const REPO: api::Repository = api::Repository { forge: 1, repository: 1 };
pub const TOKEN: &[u8] = b"0123456789abcdef0123456789abcdef01234567";
#[must_use]
pub fn shared() -> Shared {
    Shared {
        calls: 2,
        op_bytes: OP_BYTES,
        answer_bytes: 65_536,
        rows: 8,
        lifetime: Duration::from_secs(10),
        name_bytes: 128,
        repositories: 2,
        io: skein_io::Limits {
            sockets: 8,
            names: 0,
            refusals: 2,
            intake: 32_768,
            receive: 4096,
            output: 32_768,
            sends: 4,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(1),
            retry: Duration::from_millis(10),
        },
    }
}
#[must_use]
pub fn settings() -> Settings {
    let mut forges = List::with_capacity(2);
    forges
        .push(Forge {
            number: FORGE,
            kind: Kind::Forgejo,
            log_origins: List::with_capacity(0),
            address: skein_io::Addr::Ip(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))),
            transport: Transport::Plaintext { name: None },
            access: Some(skein_forge_connection::Access {
                credential: skein_forge::credential::Credential::Token(Secret::new(Box::from(TOKEN))),
                user: WRITER,
                webhook_secret: None,
            }),
        })
        .expect("one forge");
    Settings { forges, webhooks: None, word: Box::from(WORD), user_agent: Box::from(USER_AGENT) }
}
#[derive(Debug)]
pub struct Harness {
    pub component: Component,
    pub env: Env<Limits>,
    pub up: Queue<Event>,
    pub io: Queue<skein_io::Request>,
    pub answers: skein_lib::Map<u64, Answered>,
    pub peers: skein_lib::Map<skein_lib::Token, Peer>,
    pub endpoints: List<(skein_io::Addr, u16)>,
    pub opened: u32,
    pub archive: List<(u16, Secret)>,
    pub stopped: Option<(u16, skein_forge_connection::Stopped)>,
    pub git_tokens: Option<List<Event>>,
    pub installation_events: Option<List<Event>>,
    pub facts: Option<List<Event>>,
    pub hints: List<skein_forge_connection::hooks::Hinted>,
    pub component_heap: Option<CallHeap>,
    pub connected_count: u32,
    pub closed: bool,
}
#[derive(Debug)]
pub struct Peer {
    forge: u16,
    socket: skein_lib::Token,
    wire: skein_world::stream::Wire,
    connected: bool,
}
impl Harness {
    #[must_use]
    pub fn new(settings: Settings) -> Self {
        Self::with(limits(), shared(), settings)
    }
    #[must_use]
    pub fn with(limits: Limits, shared: Shared, settings: Settings) -> Self {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut endpoints = List::with_capacity(settings.forges.len());
        for forge in &settings.forges {
            endpoints.push((forge.address, forge.number)).expect("configured endpoints");
        }
        let mut github = false;
        for forge in &settings.forges {
            if forge.kind == Kind::GitHub {
                github = true;
            }
        }
        let mut h = Self {
            component: Component::new(settings, &env.limits, &shared).expect("fixture limits agree"),
            env,
            up: Queue::with_capacity(32),
            io: Queue::with_capacity(32),
            answers: skein_lib::Map::with_capacity(32),
            peers: skein_lib::Map::with_capacity(32),
            endpoints,
            opened: 0,
            archive: List::with_capacity(32),
            stopped: None,
            git_tokens: if github { Some(List::with_capacity(32)) } else { None },
            installation_events: if github { Some(List::with_capacity(256)) } else { None },
            facts: if github { Some(List::with_capacity(32)) } else { None },
            hints: List::with_capacity(32),
            component_heap: None,
            connected_count: 0,
            closed: false,
        };
        h.component.begin(&h.env, &mut h.up, &mut h.io);
        h.drain();
        h
    }
    pub fn call(&mut self, call: u64, repository: api::Repository, op: api::Op) {
        self.send(skein_forge_connection::Request::Call { call: skein_lib::Token::new(call), repository, op });
    }
    pub fn known(&mut self, repository: api::Repository, row: skein_forge::forgejo::plans::tables::Repository) {
        self.component.forgejo_tables().keep_repository(repository, row);
    }
    pub fn send(&mut self, request: skein_forge_connection::Request) {
        let span = self.start_component();
        self.component.down(&self.env, request, &mut self.up, &mut self.io);
        self.count_component(span);
    }
    pub fn run(&mut self, span: Duration) {
        self.env.now = self.env.now.saturating_add(span);
        let span = self.start_component();
        self.component.fire(&self.env, &mut self.up, &mut self.io);
        self.count_component(span);
        self.drain();
        let span = self.start_component();
        self.component.reclaim();
        self.count_component(span);
        for _ in 0..self.up.len() {
            match self.up.pop().expect("queued event") {
                Event::Connected { .. } => {
                    self.connected_count = self.connected_count.checked_add(1).expect("fixture connects");
                }
                Event::Answered(answer) => {
                    assert!(self.answered(answer.call.raw()).is_none(), "one terminal per call");
                    drop(self.answers.insert(answer.call.raw(), answer).expect("fixture answers fit"));
                }
                Event::Stopped { forge, why } => {
                    assert!(self.stopped.is_none(), "start stopped once");
                    self.stopped = Some((forge, why));
                }
                event @ Event::Installation { .. } => {
                    self.installation_events
                        .as_mut()
                        .expect("GitHub fixture")
                        .push(event)
                        .expect("fixture changes fit");
                }
                event @ Event::Fact { .. } => {
                    self.facts.as_mut().expect("GitHub fixture").push(event).expect("fixture facts fit");
                }
                event @ Event::GitToken { .. } => {
                    self.git_tokens.as_mut().expect("GitHub fixture").push(event).expect("fixture tokens fit");
                }
                Event::Hint(hint) => self.hints.push(hint).expect("fixture hints fit"),
                Event::Listening { .. } | Event::ListenerLost => panic!("webhooks absent"),
                Event::Closed => {
                    assert!(!self.closed, "closed once");
                    self.closed = true;
                }
            }
        }
    }
    #[must_use]
    pub fn answered(&self, call: u64) -> Option<&Answered> {
        self.answers.get(&call)
    }
    #[must_use]
    pub fn start(&self, forge: u16) -> Start {
        self.component.start(forge).expect("fixture forge configured")
    }
    fn newest(&self, forge: u16) -> skein_lib::Token {
        let mut found = None;
        for (owner, peer) in &self.peers {
            if peer.forge == forge {
                found = Some(*owner);
            }
        }
        found.expect("forge has a connection")
    }
    fn owners(&self) -> List<skein_lib::Token> {
        let mut owners = List::with_capacity(32);
        for (owner, _) in &self.peers {
            owners.push(*owner).expect("fixture peers fit");
        }
        owners
    }
    #[expect(clippy::manual_map, reason = "the workspace uses exhaustive matches instead of closures")]
    fn start_component(&self) -> Option<skein_heap::Span> {
        match self.component_heap {
            Some(_) => Some(skein_heap::Span::start()),
            None => None,
        }
    }
    fn count_component(&mut self, span: Option<skein_heap::Span>) {
        if let Some(span) = span {
            self.component_heap.as_mut().expect("component metering").count(span.end());
        }
    }
    fn event(&mut self, event: skein_io::Event) {
        if let Some(heap) = &mut self.component_heap
            && let skein_io::Event::Stream { up: skein_lib::stream::Up::Bytes(bytes), .. } = &event
        {
            // Incoming bytes become the component's at this boundary.
            heap.held = heap.held.checked_add(i64::try_from(bytes.len()).expect("input bytes")).expect("held input");
        }
        let span = self.start_component();
        self.component.up(&self.env, event, &mut self.up, &mut self.io);
        self.count_component(span);
    }
    pub fn connected(&mut self, forge: u16) {
        let owner = self.newest(forge);
        let peer = self.peers.get_mut(&owner).expect("registered connect");
        // A connect completes once (skein/docs/design/io.md's stream table); a step over a reused socket hears nothing.
        if !core::mem::replace(&mut peer.connected, true) {
            self.event(skein_io::Event::Connected { owner });
        }
        self.drain();
    }
    pub fn refuse(&mut self, forge: u16) {
        let owner = self.newest(forge);
        self.event(skein_io::Event::Failed { owner, error: skein_io::Error::Refused });
        self.event(skein_io::Event::Closed { owner });
        drop(self.peers.remove(&owner));
        self.drain();
    }
    pub fn answer(&mut self, forge: u16, bytes: &[u8]) {
        let owner = self.newest(forge);
        self.peers.get_mut(&owner).expect("registered connect").wire.write(bytes);
        self.drain();
    }
    pub fn reset(&mut self, forge: u16) {
        let owner = self.newest(forge);
        self.event(skein_io::Event::Stream {
            owner,
            up: skein_lib::stream::Up::Failed(skein_lib::stream::Fault::Reset),
        });
        self.drain();
    }
    #[must_use]
    pub fn request_lines(&self, forge: u16) -> List<Box<[u8]>> {
        let mut lines = List::with_capacity(16);
        for (_, peer) in &self.peers {
            if peer.forge != forge {
                continue;
            }
            let mut start = 0;
            for (at, byte) in peer.wire.received.iter().enumerate() {
                if *byte == b'\n' {
                    let line = peer.wire.received.get(start..at).expect("line positions");
                    if line.starts_with(b"GET ") {
                        lines.push(Box::from(line.strip_suffix(b"\r").unwrap_or(line))).expect("fixture lines fit");
                    }
                    start = at.saturating_add(1);
                }
            }
        }
        lines
    }
    #[must_use]
    pub fn request_bytes(&self, forge: u16) -> Box<[u8]> {
        let mut length = 0_usize;
        for (_, peer) in &self.peers {
            if peer.forge == forge {
                length = length.checked_add(peer.wire.received.len()).expect("bounded requests");
            }
        }
        let mut out = skein_lib::Writer::new(length);
        for (_, peer) in &self.peers {
            if peer.forge == forge {
                out.put(&peer.wire.received).expect("requests measured");
            }
        }
        out.finish()
    }
    #[must_use]
    pub fn all_request_bytes(&self, forge: u16) -> Box<[u8]> {
        let current = self.request_bytes(forge);
        let mut length = current.len();
        for (number, bytes) in &self.archive {
            if *number == forge {
                length = length.checked_add(bytes.len()).expect("fixture history");
            }
        }
        let mut out = skein_lib::Writer::new(length);
        for (number, bytes) in &self.archive {
            if *number == forge {
                out.put(bytes.expose()).expect("measured history");
            }
        }
        out.put(&current).expect("measured current");
        out.finish()
    }
    #[must_use]
    pub fn connections_opened(&self) -> u32 {
        self.opened
    }
    #[must_use]
    pub fn stopped(&self) -> Option<(u16, skein_forge_connection::Stopped)> {
        self.stopped
    }
    #[must_use]
    pub fn ready() -> Self {
        let mut h = Self::new(settings());
        h.connected(FORGE);
        h.answer(FORGE, &user_ok());
        h.answer(FORGE, &settings_ok());
        h
    }
    fn drain(&mut self) {
        let mut quiet = 0_u32;
        for _ in 0..1024_u32 {
            for _ in 0..self.io.len() {
                match self.io.pop().expect("queued request") {
                    skein_io::Request::Connect { owner, addr } => {
                        let mut forge = None;
                        for (address, number) in &self.endpoints {
                            if *address == addr {
                                forge = Some(*number);
                            }
                        }
                        let forge = forge.expect("connect names a configured endpoint");
                        self.opened = self.opened.saturating_add(1);
                        let socket = skein_lib::Token::new(owner.raw().saturating_add(100));
                        drop(
                            self.peers
                                .insert(
                                    owner,
                                    Peer {
                                        forge,
                                        socket,
                                        wire: skein_world::stream::Wire::new(owner.raw()),
                                        connected: false,
                                    },
                                )
                                .expect("fixture peers fit"),
                        );
                        self.event(skein_io::Event::Connecting { owner, socket });
                    }
                    skein_io::Request::Stream { stream, down } => {
                        for owner in self.owners().into_boxed() {
                            let peer = self.peers.get_mut(&owner).expect("fixture peer");
                            if peer.socket == stream {
                                peer.wire.take(down);
                                break;
                            }
                        }
                    }
                    skein_io::Request::Close { entity } | skein_io::Request::Abort { entity } => {
                        let mut owner = None;
                        for (token, peer) in &self.peers {
                            if peer.socket == entity {
                                owner = Some(*token);
                                break;
                            }
                        }
                        if let Some(owner) = owner {
                            self.event(skein_io::Event::Closed { owner });
                            let peer = self.peers.remove(&owner).expect("closed peer");
                            self.archive
                                .push((peer.forge, Secret::new(Box::from(peer.wire.received.as_slice()))))
                                .expect("fixture history");
                        }
                    }
                    skein_io::Request::Listen { .. }
                    | skein_io::Request::Bind { .. }
                    | skein_io::Request::Reject { .. }
                    | skein_io::Request::Output { .. }
                    | skein_io::Request::Spawn { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. } => unreachable!("forge only uses sockets"),
                }
            }
            let mut events = List::with_capacity(32);
            let mut moved = false;
            for owner in self.owners().into_boxed() {
                let peer = self.peers.get_mut(&owner).expect("fixture peer");
                let trace = peer.wire.trace.len();
                if peer.connected
                    && let Some(up) = peer.wire.answer()
                {
                    events.push(skein_io::Event::Stream { owner, up }).expect("fixture events fit");
                }
                moved |= peer.wire.trace.len() != trace;
                // The trace signals progress; retaining it would charge the
                // scripted peer's transcript to component heap measurements.
                peer.wire.trace.clear();
            }
            moved |= !events.is_empty();
            for event in events.into_boxed() {
                self.event(event);
            }
            if !moved && self.io.is_empty() {
                quiet = quiet.saturating_add(1);
                if quiet == 32 {
                    break;
                }
            } else {
                quiet = 0;
            }
        }
    }
}
#[must_use]
pub fn ok(body: &[u8]) -> Box<[u8]> {
    response(200, body)
}
#[must_use]
pub fn response(code: u16, body: &[u8]) -> Box<[u8]> {
    response_with_fields(code, body, b"")
}
#[must_use]
pub fn last_page(body: &[u8]) -> Box<[u8]> {
    response_with_fields(200, body, b"X-HasMore: false\r\n")
}
#[must_use]
pub fn response_with_fields(code: u16, body: &[u8], fields: &[u8]) -> Box<[u8]> {
    let decimal = skein_lib::Decimal::of(skein_lib::len_u64(body.len()));
    let mut out = skein_lib::Writer::new(
        body.len()
            .saturating_add(fields.len())
            .saturating_add(b"HTTP/1.1 200 OK\r\nContent-Length: ".len())
            .saturating_add(decimal.as_bytes().len())
            .saturating_add(b"\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n".len()),
    );
    out.put(b"HTTP/1.1 ").expect("head fits");
    out.put(skein_lib::Decimal::of(u64::from(code)).as_bytes()).expect("code fits");
    out.put(b" OK\r\n").expect("head fits");
    out.put(fields).expect("fields fit");
    out.put(b"Content-Length: ").expect("head fits");
    out.put(decimal.as_bytes()).expect("length fits");
    out.put(b"\r\nDate: Wed, 01 Jan 2020 00:00:00 GMT\r\n\r\n").expect("date fits");
    out.put(body).expect("body fits");
    out.finish()
}
#[must_use]
pub fn ok_dated(body: &[u8]) -> Box<[u8]> {
    ok(body)
}
#[must_use]
pub fn status(code: u16) -> Box<[u8]> {
    let decimal = skein_lib::Decimal::of(u64::from(code));
    let mut out = skein_lib::Writer::new(
        b"HTTP/1.1 "
            .len()
            .saturating_add(decimal.as_bytes().len())
            .saturating_add(b" Refused\r\nContent-Length: 0\r\n\r\n".len()),
    );
    out.put(b"HTTP/1.1 ").expect("head fits");
    out.put(skein_lib::Decimal::of(u64::from(code)).as_bytes()).expect("code fits");
    out.put(b" Refused\r\nContent-Length: 0\r\n\r\n").expect("head fits");
    out.finish()
}
#[must_use]
pub fn user_ok() -> Box<[u8]> {
    ok(b"{\"id\":2,\"login\":\"writer\"}")
}
#[must_use]
pub fn settings_ok() -> Box<[u8]> {
    ok(b"{\"max_response_items\":2,\"default_paging_num\":30}")
}
#[must_use]
pub fn specimen() -> skein_forge::forgejo::plans::tables::Repository {
    skein_forge::forgejo::plans::tables::Repository {
        owner: Box::from(&b"fixture"[..]),
        name: Box::from(&b"specimen"[..]),
        object_format: skein_forge::forgejo::id::ObjectFormat::Sha1,
        merge_style: skein_forge::forgejo::document::MergeStyle::Squash,
        html_url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen"[..]),
    }
}

#[must_use]
pub fn known() -> Harness {
    let mut h = Harness::ready();
    h.known(REPO, specimen());
    h
}

/// The literal value of the sole named header on the HTTP wire.
#[must_use]
pub fn header<'a>(sent: &'a [u8], name: &[u8]) -> &'a [u8] {
    let mut found = None;
    let mut rest = sent;
    while let Some(end) = skein_lib::bytes::find(rest, b"\r\n") {
        let line = rest.get(..end).expect("line end in request");
        if let Some(value) = line.strip_prefix(name) {
            assert!(found.is_none(), "header occurs once");
            found = Some(value);
        }
        rest = rest.get(end.checked_add(2).expect("line end fits")..).expect("next line");
    }
    found.expect("header sent")
}

pub const TOKEN_BOUND: usize = 64;

#[must_use]
pub fn filled(length: usize, byte: u8) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(length);
    for _ in 0..length {
        out.put(&[byte]).expect("measured fill");
    }
    out.finish()
}

pub const OTHER_FORGE: u16 = 2;
pub const OTHER_REPO: api::Repository = api::Repository { forge: OTHER_FORGE, repository: 1 };
pub const BRANCH: &[u8] = b"main";
pub const BRANCH_HEAD: [u8; 32] = [0; 32];
pub const REPOSITORY_BODY: &[u8] = br#"{"id":1,"owner":{"id":2,"login":"fixture"},"name":"specimen","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":false,"allow_rebase":false,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#;
pub const BRANCH_BODY: &[u8] = br#"{"name":"main","commit":{"id":"0000000000000000000000000000000000000000"},"effective_branch_protection_name":""}"#;
#[must_use]
pub fn two_forges() -> Settings {
    let mut settings = settings();
    let mut other = self::settings().forges.into_boxed().into_iter().next().expect("forge");
    other.number = OTHER_FORGE;
    other.address = skein_io::Addr::Ip(([127, 0, 0, 1], 3001).into());
    settings.forges.push(other).expect("two forges fit");
    settings
}
#[must_use]
pub fn ready_two(limits: Limits, shared: Shared) -> Harness {
    let mut h = Harness::with(limits, shared, two_forges());
    for forge in [FORGE, OTHER_FORGE] {
        h.connected(forge);
        h.answer(forge, &user_ok());
        h.answer(forge, &settings_ok());
        h.run(limits.idle);
    }
    h
}
pub fn branch_answers(h: &mut Harness, forge: u16) {
    h.answer(forge, &ok(REPOSITORY_BODY));
    h.answer(forge, &ok(BRANCH_BODY));
    h.run(Duration::ZERO);
}

impl Harness {
    #[must_use]
    pub fn connections_for(&self, forge: u16) -> u32 {
        let mut count = 0_u32;
        for (_, peer) in &self.peers {
            if peer.forge == forge {
                count = count.checked_add(1).expect("bounded peers");
            }
        }
        count
    }
}

// The page premise contains two maximal rows, and also tests one row and no room.
pub const PAGE_ROWS: u32 = 2; // The premise budgets exactly two maximal rows.
// Eviction shares one connection and one call slot between two sequential requests.
pub const EVICTION_CONNECTIONS: u32 = 1; // A single connection forces reassignment.
pub const EVICTION_CALLS: u32 = EVICTION_CONNECTIONS;
pub const EVICTION_NO_CONNECTIONS: u32 = 0; // The evicted forge has no connection.
pub const EVICTION_ANSWERS: u32 = 2; // Each of the two sequential requests answers once.

#[must_use]
pub fn app_access(key: &[u8], id: u64, user: u64) -> skein_forge_connection::Access {
    skein_forge_connection::Access {
        credential: skein_forge::credential::Credential::App { id, key: Secret::new(Box::from(key)) },
        user,
        webhook_secret: None,
    }
}

#[must_use]
pub fn github_settings() -> Settings {
    let mut settings = settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.kind = Kind::GitHub;
    forge.access = Some(app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
    settings
}

pub const GITHUB_REPOSITORY: &[u8] = br#"{"id":1,"owner":{"id":22,"login":"acme","type":"Organization"},"name":"widgets","default_branch":"main","html_url":"https://github.com/acme/widgets","allow_merge_commit":true,"allow_squash_merge":true,"allow_rebase_merge":false}"#;
pub const GITHUB_BRANCH: &[u8] = br#"{"name":"main","commit":{"sha":"0000000000000000000000000000000000000000"}}"#;

// GitHub dialect fixtures: issue/comment, pull, combined status, check runs and GraphQL.
pub const GITHUB_ISSUE: &[u8] = br#"{"number":7,"state":"open","user":{"id":9},"updated_at":"2026-10-20T08:00:00Z","title":"widgets","labels":[],"body":"description"}"#;
pub const GITHUB_COMMENTS: &[u8] = br#"[{"id":8,"user":{"id":9},"created_at":"2026-10-20T08:00:00Z","updated_at":"2026-10-20T08:00:00Z","body":"hello"}]"#;
pub const GITHUB_PULL: &[u8] = br#"{"number":7,"state":"open","merged":false,"merge_commit_sha":null,"mergeable":true,"head":{"ref":"feature","sha":"0000000000000000000000000000000000000000"},"base":{"ref":"main","sha":"1111111111111111111111111111111111111111"},"requested_reviewers":[]}"#;
pub const GITHUB_COMBINED: &[u8] = br#"{"state":"pending","total_count":0,"statuses":[]}"#;
pub const GITHUB_RUNS: &[u8] = br#"{"check_runs":[]}"#;
pub const GITHUB_GRAPHQL: &[u8] = br#"{"data":{"repository":{"pullRequest":{"id":"opaque-node","mergeCommit":null,"mergeQueueEntry":null,"timelineItems":{"nodes":[]}}}}}"#;

// Sum only component/harness spans. Each next step starts with the retained
// growth of the preceding steps; fake allocations between them are excluded.
#[derive(Debug)]
pub struct CallHeap {
    pub peak: i64,
    pub held: i64,
}
impl CallHeap {
    pub fn count(&mut self, growth: skein_heap::Grown) {
        self.peak = self.peak.max(self.held.checked_add(growth.peak).expect("peak growth"));
        self.held = self.held.checked_add(growth.net).expect("retained growth");
    }
}

pub const LIVE: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":null}]"#;

pub const SUSPENDED: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":"1970-01-01T00:00:02Z"}]"#;

pub const FULL: &[u8] = b"artificial-full-api-secret";

pub const CONTENTS: &[u8] = b"artificial-contents-git-secret";

pub const EXPIRES: &[u8] = b"1970-01-01T01:00:00Z";

pub const EXPIRY: Wall = Wall::from_nanos(3_600_000_000_000);

pub const VIEWER: &[u8] = br#"{"data":{"viewer":{"databaseId":9,"login":"test-app[bot]"}}}"#;

pub const NEXT_PAGE: &[u8] = b"Link: <https://api.github.test/app/installations?per_page=100&page=2>; rel=\"next\"\r\n";

pub fn accepted(h: &mut Harness, ids: &[u64]) {
    h.send(skein_forge_connection::Request::Installations { forge: FORGE, accepted: Box::from(ids) });
}

#[must_use]
pub fn mint(token: &[u8], expires: &[u8]) -> Box<[u8]> {
    let mut body = skein_lib::Writer::new(
        token.len().checked_add(expires.len()).expect("fixture fields").checked_add(28).expect("JSON framing"),
    );
    body.put(b"{\"token\":\"").expect("measured token");
    body.put(token).expect("measured token");
    body.put(b"\",\"expires_at\":\"").expect("measured expiry");
    body.put(expires).expect("measured expiry");
    body.put(b"\"}").expect("measured mint");
    response(201, &body.finish())
}

pub fn full(h: &mut Harness) {
    h.answer(FORGE, &mint(FULL, EXPIRES));
}

pub fn contents(h: &mut Harness) {
    h.answer(FORGE, &mint(CONTENTS, EXPIRES));
}

#[must_use]
pub fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    for piece in bytes.windows(needle.len()) {
        if piece == needle {
            return true;
        }
    }
    false
}

#[must_use]
pub fn tokens(h: &Harness) -> &[Event] {
    h.git_tokens.as_ref().expect("GitHub harness").as_slice()
}

pub fn github_repository(h: &mut Harness, repository: api::Repository, account: u64) {
    h.component.github_tables().keep_repository(
        repository,
        skein_forge::github::plans::tables::Repository {
            owner: Box::from(b"acme".as_slice()),
            name: Box::from(b"widgets".as_slice()),
            owner_id: account,
            organization: true,
            merge: Some(skein_forge::github::plans::tables::Method::Merge),
            html_url: Box::from(b"https://github.com/acme/widgets".as_slice()),
        },
    );
}

use skein_forge::github::document;
use skein_forge_connection::seam::Read;
const SECOND: Duration = Duration::from_secs(1);
#[must_use]
pub fn row(id: u64, suspended: bool) -> document::Installation {
    document::Installation {
        id,
        account: 22,
        login: Box::from(b"acme".as_slice()),
        kind: document::Account::Organization,
        created: Wall::EPOCH,
        suspended,
    }
}

// Feed the committed reconciliation transition directly to cross a written job
// with loss; socket serialization itself is checked by the wire stories.
pub fn commit(h: &mut Harness, rows: Box<[document::Installation]>) {
    let gh = h.component.test_github_mut();
    let mut reading = skein_forge_connection::seam::Reading::new(gh.rows.capacity(), gh.generation, SECOND);
    reading.page(rows, &gh.rows, &gh.accepted, self::shared().name_bytes).expect("fixture read");
    gh.read = Read::Reading(reading);
    gh.commit();
}

// One-hour Full and seventy-minute Contents, with the fixture's sixty-second lead.
pub const FULL_DUE: u64 = 3540;

pub const CONTENTS_DUE: u64 = 4140;

pub const CONTENTS_EXPIRY: &[u8] = b"1970-01-01T01:10:00Z";

pub const LATER: &[u8] = b"1970-01-01T03:00:00Z";

pub const NEW_FULL: &[u8] = b"artificial-renewed-full";

pub const NEW_CONTENTS: &[u8] = b"artificial-renewed-contents";

#[must_use]
pub fn ready() -> Harness {
    initial(CONTENTS_EXPIRY)
}

#[must_use]
pub fn initial(contents_expiry: &[u8]) -> Harness {
    let mut h = minted_pair(EXPIRES, contents_expiry);
    h.answer(FORGE, &self::ok(VIEWER));
    h.run(Duration::ZERO);
    // Let the idle socket settle before crossing the renewal timer.
    h.run(h.env.limits.idle);
    h
}

#[must_use]
pub fn minted_pair(full_expiry: &[u8], contents_expiry: &[u8]) -> Harness {
    let mut h = Harness::new(self::github_settings());
    accepted(&mut h, &[1]);
    h.connected(FORGE);
    h.answer(FORGE, &self::ok(LIVE));
    h.answer(FORGE, &mint(FULL, full_expiry));
    h.answer(FORGE, &mint(CONTENTS, contents_expiry));
    h
}

pub fn advance(h: &mut Harness, seconds: u64) {
    h.run(Time::from_nanos(Duration::from_secs(seconds).as_nanos()).saturating_since(h.env.now));
}

#[must_use]
pub fn renewing() -> Harness {
    let mut h = ready();
    advance(&mut h, FULL_DUE);
    h.connected(FORGE);
    h
}

#[must_use]
pub fn contents_renewing() -> Harness {
    let mut h = renewing();
    renewed_full(&mut h);
    h.run(h.env.limits.idle);
    advance(&mut h, CONTENTS_DUE);
    h.connected(FORGE);
    h
}

pub fn renewed_full(h: &mut Harness) {
    h.answer(FORGE, &mint(NEW_FULL, LATER));
    h.run(Duration::ZERO);
}

pub fn renewed_contents(h: &mut Harness) {
    h.answer(FORGE, &mint(NEW_CONTENTS, LATER));
    h.run(Duration::ZERO);
}

pub fn github_account(h: &mut Harness, installation: u64, account: u64, login: &[u8], token: &[u8]) {
    use skein_forge_connection::seam::{Clock, Row, Tokens};
    let gh = h.component.test_github_mut();
    let _accepted = gh.accepted.insert(installation).expect("fixture accepted");
    drop(
        gh.rows
            .insert(
                installation,
                Row {
                    state: Tokens::Current,
                    revision: 0,
                    token_revision: 1,
                    refused_revision: None,
                    full: Clock { expiry: Time::from_nanos(3_600_000_000_000), ..Clock::empty() },
                    contents: Clock::empty(),
                    account: skein_forge_connection::Account { id: account, login: Box::from(login) },
                    suspended: false,
                    accepted: true,
                    api: Some(Secret::new(Box::from(token))),
                    permissions: skein_forge::github::document::Levels::NONE,
                    api_expires: Wall::from_nanos(3_600_000_000_000),
                    git_expires: Wall::from_nanos(3_600_000_000_000),
                },
            )
            .expect("fixture installation"),
    );
    let account = gh.rows.get(&installation).expect("fixture row").account.clone();
    drop(
        gh.known
            .insert(installation, skein_forge_connection::seam::Accepted { known: Some(account), checked: true })
            .expect("fixture accepted bookkeeping"),
    );
}

/// Seeds checked call state without opening a startup connection.
#[must_use]
pub fn github_calls() -> Harness {
    let mut settings = github_settings();
    settings.forges.get_mut(0).expect("forge").access = None;
    let mut h = Harness::new(settings);
    h.component.test_calls_ready(app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
    github_account(&mut h, 1, 22, b"acme", b"artificial-full-one");
    h
}
