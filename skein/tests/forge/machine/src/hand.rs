//! Hand-steered component exchanges (skein/docs/design/forge/connection.md, 10).
use skein_forge::api;
use skein_forge_connection::{Answered, Component, Event, Limits, Settings, Shared, settings::Kind, start::Start};
pub use skein_forge_connection_fixtures::{
    BODY_ROOM, BRANCH, BRANCH_HEAD, FORGE, OP_BYTES, OTHER_FORGE, REPO, TOKEN, TOKEN_BOUND, USER_AGENT, WORD, WRITER,
    app_access, filled, github_settings, header, limits, ok, response, response_with_fields, settings, shared, user_ok,
};
use skein_lib::{Duration, Env, List, Queue, Secret, Time, Wall};
use std::boxed::Box;
use std::ops::Bound::{Excluded, Unbounded};

mod forgejo;
mod github;
pub use forgejo::*;
pub use github::*;
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
        // A connect completes once (skein/docs/design/runtime/io/README.md's stream table); a step over a reused socket hears nothing.
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
    fn drain(&mut self) {
        let mut quiet = 0_u32;
        for _ in 0..1024_u32 {
            if self.peers.is_empty() && self.io.is_empty() {
                return;
            }
            let mut moved = !self.io.is_empty();
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
                        let owner =
                            self.peers.iter().find_map(|(owner, peer)| (peer.socket == stream).then_some(*owner));
                        if let Some(owner) = owner {
                            self.peers.get_mut(&owner).expect("fixture peer").wire.take(down);
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
                    | skein_io::Request::SpawnAlone { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. } => unreachable!("forge only uses sockets"),
                }
            }
            let mut events = List::with_capacity(32);
            // Resume by key to borrow each peer in place without a list of keys.
            let mut next = self.peers.first().map(|(owner, _)| *owner);
            while let Some(owner) = next {
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
                next = self.peers.range((Excluded(owner), Unbounded)).next().map(|(owner, _)| *owner);
            }
            moved |= !events.is_empty();
            for event in events.into_boxed() {
                self.event(event);
            }
            if !moved && self.io.is_empty() {
                quiet = quiet.saturating_add(1);
                if quiet == skein_forge_kit::machine::QUIET {
                    break;
                }
            } else {
                quiet = 0;
            }
        }
    }
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
pub fn settings_ok() -> Box<[u8]> {
    ok(b"{\"max_response_items\":2,\"default_paging_num\":30}")
}
#[must_use]
pub fn specimen() -> skein_forge::forgejo::plans::tables::Repository {
    skein_forge::forgejo::plans::tables::Repository {
        owner: Box::from(&b"fixture"[..]),
        name: Box::from(&b"specimen"[..]),
        object_format: skein_forge::forgejo::id::ObjectFormat::Sha1,
        settings: api::Settings { default_branch: Box::from(*b"main"), merge: true, squash: false, rebase: false },
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

pub const OTHER_REPO: api::Repository = api::Repository { forge: OTHER_FORGE, repository: 1 };
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
// The page premise contains two maximal rows, and also tests one row and no room.
pub const PAGE_ROWS: u32 = 2; // The premise budgets exactly two maximal rows.
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
