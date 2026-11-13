//! The component's small limits and configured forge.
use crate::Limits;
pub use crate::fixtures::{
    BODY_ROOM, BRANCH, BRANCH_HEAD, FORGE, OP_BYTES, OTHER_FORGE, REPO, TOKEN, TOKEN_BOUND, USER_AGENT, WORD, WRITER,
    app_access, filled, github_settings, header, limits, ok, response, settings, shared, user_ok,
};
/// Check and construction return the same refusal before any io is begun.
pub fn admission(settings: Settings, limits: &Limits, expected: Result<(), crate::Unusable>, label: &[u8]) {
    let shared = shared();
    assert_eq!(crate::limits::check(limits, &shared, &settings), expected, "{label:?}: check");
    let constructed = match Component::new(settings, limits, &shared) {
        Ok(_) => Ok(()),
        Err(reason) => Err(reason),
    };
    assert_eq!(constructed, expected, "{label:?}: construction");
}

pub const BODY_ROOM_SHORT: u32 = BODY_ROOM - 1;
pub const MAX_WORD_BODY_ROOM: u32 = 2 * OP_BYTES + 16 + 15;

use crate::{
    Component, Event, Settings, Shared,
    settings::{Forge, Kind, Transport},
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Secret, Time, Wall};

/// One connection's scripted stream, shared by its step tests.
#[derive(Debug)]
pub struct ConnectionHarness {
    pub kept: crate::kept::Store,
    pub connection: crate::connection::Connection,
    pub forgejo: crate::connection::forgejo::Exchange,
    pub github: crate::connection::github::Exchange,
    pub env: Env<Limits>,
    pub forge: Forge,
    pub io: Queue<skein_io::Request>,
    pub sent: Box<[u8]>,
    pub user_agent: Box<[u8]>,
    pub body_demands: u32,
    pub local_pumps: u32,
}
impl ConnectionHarness {
    pub fn new() -> Self {
        let forge = settings().forges.into_boxed().into_iter().next().expect("one forge");
        Self::with_forge(forge, shared())
    }
    pub fn github() -> Self {
        Self::github_with(shared())
    }
    pub fn github_with(shared: Shared) -> Self {
        let mut forge = settings().forges.into_boxed().into_iter().next().expect("one forge");
        forge.kind = Kind::GitHub;
        forge.access = None;
        Self::with_forge(forge, shared)
    }
    pub fn with_chunk(chunk: u32) -> Self {
        let forge = settings().forges.into_boxed().into_iter().next().expect("one forge");
        let mut limits = limits();
        limits.chunk = chunk;
        Self::with_forge_limits(forge, shared(), limits)
    }
    fn with_forge(forge: Forge, shared: Shared) -> Self {
        Self::with_forge_limits(forge, shared, limits())
    }
    fn with_forge_limits(forge: Forge, shared: Shared, limits: Limits) -> Self {
        let mut connection =
            crate::connection::Connection::new(&forge, &limits, shared, Time::from_nanos(10_000_000_000));
        connection.link.connecting(skein_lib::Token::new(100), &mut Queue::uncapped());
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        connection.connect(&env);
        Self {
            kept: crate::kept::Store::new(limits.kept_bytes),
            connection,
            forgejo: crate::connection::forgejo::Exchange::new(),
            github: crate::connection::github::Exchange::new(),
            env,
            forge,
            io: Queue::with_capacity(32),
            sent: Box::from([]),
            user_agent: Box::from(USER_AGENT),
            body_demands: 0,
            local_pumps: 0,
        }
    }
    pub fn pump(&mut self) {
        match self.forge.kind {
            Kind::Forgejo => self.forgejo.pump(&mut self.connection, &self.env, &mut self.io, &mut self.kept),
            Kind::GitHub => self.github.pump(&mut self.connection, &self.env, &mut self.io, &mut self.kept),
        }
    }
    /// Forgejo tests supply the component's complete token header.
    pub fn start_forgejo(
        &mut self,
        request: skein_forge::forgejo::request::Request,
        range: Option<skein_forge::forgejo::plans::plan::Range>,
    ) -> Result<(), &'static str> {
        let credential = skein_forge::credential::Credential::Token(Secret::new(Box::from(TOKEN)));
        let authorization = skein_forge::forgejo::credential::authorization(&credential).expect("fixture token");
        match self.forgejo.start(
            &mut self.connection,
            &self.env,
            &self.forge,
            &self.user_agent,
            skein_forge::forgejo::plans::plan::Exchange { request, range },
            authorization,
            &mut self.kept,
        ) {
            Ok(()) => Ok(()),
            Err(_) => Err("Forgejo refused"),
        }
    }
    pub fn start_github(
        &mut self,
        exchange: skein_forge::github::plans::plan::Exchange,
        authorization: Option<Secret>,
    ) -> Result<(), &'static str> {
        match self.github.start(
            &mut self.connection,
            &self.env,
            &self.forge,
            USER_AGENT,
            exchange,
            authorization,
            &mut self.kept,
        ) {
            Ok(()) => Ok(()),
            Err(_) => Err("GitHub refused"),
        }
    }
    pub fn ready(&self) -> bool {
        match self.forge.kind {
            Kind::Forgejo => self.forgejo.ready(&self.connection),
            Kind::GitHub => self.github.ready(&self.connection),
        }
    }
    pub fn answer(&mut self, response: &[u8]) -> List<skein_forge::forgejo::plans::Heard> {
        let mut heard = List::with_capacity(16);
        let mut wire = skein_world::stream::Wire::new(1);
        let mut written = false;
        for _ in 0_u32..1024 {
            self.answer_wire(response, &mut wire, &mut written);
            if let Some(event) = self.forgejo.heard(&mut self.connection) {
                heard.push(event).expect("fixture events fit");
            }
        }
        self.sent = Box::from(wire.received.as_slice());
        heard
    }
    /// The existing connection fixture with a buffered or fragmented peer body.
    pub fn buffered_answer(
        &mut self,
        response: &[u8],
        fragment: usize,
        eof: bool,
    ) -> List<skein_forge::forgejo::plans::Heard> {
        use skein_lib::stream::{Down, Read, Up};
        let mut intake = skein_lib::Intake::with_capacity(shared().io.intake);
        let mut at = 0_usize;
        let mut pending = None;
        let mut ended = false;
        let mut heard = List::with_capacity(16);
        for _ in 0_u32..1024 {
            if self.ready() && self.io.is_empty() {
                self.local_pumps = self.local_pumps.saturating_add(1);
            }
            self.pump();
            for _ in 0..self.io.len() {
                match self.io.pop().expect("queued request") {
                    skein_io::Request::Stream { down, .. } => match down {
                        Down::Demand { read: Read::Nothing, room: 0 } => {
                            assert!(pending.take().is_some(), "withdraw an outstanding read");
                        }
                        Down::Demand { read, room } => {
                            assert!(pending.is_none(), "one read below at a time");
                            let body = match read {
                                Read::Nothing => false,
                                Read::Scan { until, .. } => until != skein_lib::stream::Delimiter::LF,
                                Read::Fill(_) | Read::Line { .. } | Read::Arrived { .. } => true,
                            };
                            self.body_demands = self.body_demands.saturating_add(u32::from(body));
                            pending = Some((read, room));
                        }
                        Down::Send(bytes) => self.sent = bytes,
                        Down::Finish => {}
                    },
                    skein_io::Request::Listen { .. }
                    | skein_io::Request::Connect { .. }
                    | skein_io::Request::Bind { .. }
                    | skein_io::Request::Reject { .. }
                    | skein_io::Request::Output { .. }
                    | skein_io::Request::Spawn { .. }
                    | skein_io::Request::SpawnAlone { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. }
                    | skein_io::Request::Close { .. }
                    | skein_io::Request::Abort { .. } => unreachable!("one connection emits stream requests"),
                }
            }
            if at < response.len() {
                let end = at.saturating_add(fragment).min(response.len());
                assert!(end > at, "positive fixture fragments");
                intake.append(response.get(at..end).expect("fixture fragment")).expect("response capacity");
                at = end;
            }
            if let Some((read, room)) = pending {
                if let Some(bytes) = intake.meet(read) {
                    pending = None;
                    self.connection.stream(&self.env, Up::Bytes(bytes));
                } else if room > 0 {
                    pending = None;
                    self.connection.stream(&self.env, Up::Room);
                }
            }
            // A terminal may accompany the final byte delivery, before the
            // connection pumps either of HTTP's resulting events.
            if eof && !ended && at == response.len() && intake.is_empty() {
                ended = true;
                self.connection.stream(&self.env, Up::End);
            }
            for _ in 0_u32..16 {
                let Some(event) = self.forgejo.heard(&mut self.connection) else { break };
                heard.push(event).expect("fixture events fit");
            }
            if !self.ready() && pending.is_none() && at == response.len() {
                break;
            }
        }
        heard
    }
    pub fn github_answer(&mut self, response: &[u8]) -> List<skein_forge::github::plans::Heard> {
        let mut heard = List::with_capacity(16);
        let mut wire = skein_world::stream::Wire::new(1);
        let mut written = false;
        for _ in 0_u32..1024 {
            self.answer_wire(response, &mut wire, &mut written);
            if let Some(event) = self.github.heard(&mut self.connection) {
                heard.push(event).expect("fixture events fit");
            }
        }
        self.sent = Box::from(wire.received.as_slice());
        heard
    }
    fn answer_wire(&mut self, response: &[u8], wire: &mut skein_world::stream::Wire, written: &mut bool) {
        self.pump();
        for _ in 0..self.io.len() {
            let request = self.io.pop().expect("queued io request");
            match request {
                skein_io::Request::Stream { down, .. } => wire.take(down),
                skein_io::Request::Listen { .. }
                | skein_io::Request::Connect { .. }
                | skein_io::Request::Bind { .. }
                | skein_io::Request::Reject { .. }
                | skein_io::Request::Output { .. }
                | skein_io::Request::Spawn { .. }
                | skein_io::Request::SpawnAlone { .. }
                | skein_io::Request::Signal { .. }
                | skein_io::Request::Usage { .. }
                | skein_io::Request::Window { .. }
                | skein_io::Request::Close { .. }
                | skein_io::Request::Abort { .. } => unreachable!("one connection emits stream requests"),
            }
        }
        if !*written && self.connection.phase() == crate::connection::Phase::Reading {
            wire.write(response);
            *written = true;
        }
        if let Some(event) = wire.answer() {
            self.connection.stream(&self.env, event);
        }
    }
}
pub const HOOK_SECRET: &[u8] = b"fixture-hook-secret";
pub fn hook_settings() -> Settings {
    let mut settings = settings();
    settings.forges.get_mut(0).expect("forge").access.as_mut().expect("fixture access").webhook_secret =
        Some(Secret::new(Box::from(HOOK_SECRET)));
    settings.webhooks = Some(crate::hooks::Webhooks { listener: skein_io::Addr::Ip(([127, 0, 0, 1], 7200).into()) });
    settings
}
pub const HOOK_REPOSITORY: u64 = 2;
pub const HOOK_NUMBER: u64 = 7;
pub const HOOK_HEAD: [u8; 32] = [0x77; 32];
pub fn hook_body() -> skein_forge::forgejo::document::Body {
    skein_forge::forgejo::document::Body::Kept(Box::from(b"<!-- kit:key 0102 -->\nbody".as_slice()))
}
pub fn hook_id() -> skein_forge::forgejo::id::ObjectId {
    skein_forge::forgejo::id::ObjectId { format: skein_forge::forgejo::id::ObjectFormat::Sha256, bytes: HOOK_HEAD }
}
pub fn hook_ref(reference: &[u8]) -> skein_forge::forgejo::delivery::Delivery {
    skein_forge::forgejo::delivery::Delivery::Ref {
        repository: HOOK_REPOSITORY,
        reference: Box::from(reference),
        created: false,
        ref_type: skein_forge::forgejo::delivery::RefType::Branch,
    }
}

/// Trust shared by the configured TLS names; the HTTP harness captures
/// requests before their TLS record transport (the TLS world owns encryption).
pub fn trust() -> skein_tls::Config {
    let root = std::fs::read("../../tests/tls/fixtures/root.der").expect("shared TLS root");
    skein_tls::Config::from_der(&[root.into_boxed_slice()], &[]).expect("valid root").0
}

/// Exercise request construction and HTTP serialization on a scripted carrier.
/// The supplied forge supplies its transport name to the exchange.
pub fn request_bytes(settings: Settings) -> Box<[u8]> {
    let mut h = ConnectionHarness::new();
    h.forge = settings.forges.into_boxed().into_iter().next().expect("one configured forge");
    h.user_agent = settings.user_agent;
    h.start_forgejo(skein_forge::forgejo::request::Request::User, None).expect("valid request");
    h.answer(&user_ok());
    h.sent
}

// The request caps are the fixture's TLS send and io output bounds.
pub const TLS_REQUEST_ROOM: usize = 16_384;
pub const PLAIN_REQUEST_ROOM: usize = 32_768;
// Fixed framing (192) plus both 128-byte validators and their field framing (300).
pub const HEAD_FRAMING: usize = 492;
pub const SECRET_BOUND: usize = 128;
pub const PULL_PAYLOAD: &[u8] = b"{\"repository\":{\"id\":2},\"number\":7}";

/// Lowercase HMAC text for the configured hook secret.
pub fn hook_signature(body: &[u8]) -> [u8; 64] {
    let mut mac = skein_lib::sha256::Hmac::new(HOOK_SECRET);
    mac.update(body);
    let mut signature = [0; 64];
    for ([high, low], byte) in signature.as_chunks_mut::<2>().0.iter_mut().zip(mac.finish()) {
        *high = skein_lib::bytes::hex_digit(byte / 16);
        *low = skein_lib::bytes::hex_digit(byte);
    }
    signature
}
pub fn signed_head(body: &[u8], event: Option<&[u8]>) -> Box<[u8]> {
    hook_head(body, event, &hook_signature(body))
}
pub fn hook_head(body: &[u8], event: Option<&[u8]>, signature: &[u8]) -> Box<[u8]> {
    use skein_forge::forgejo::delivery::{EVENT_HEADER, SIGNATURE_HEADER};
    let target = skein_forge::forgejo::delivery::hook_target(FORGE);
    let length = skein_lib::Decimal::of(skein_lib::len_u64(body.len()));
    let mut pieces = List::with_capacity(16);
    for piece in
        [b"POST ".as_slice(), &target, b" HTTP/1.1\r\nHost: localhost\r\nContent-Length: ", length.as_bytes(), b"\r\n"]
    {
        pieces.push(piece).expect("bounded head pieces");
    }
    if let Some(event) = event {
        for piece in [EVENT_HEADER, b": ", event, b"\r\n"] {
            pieces.push(piece).expect("bounded event pieces");
        }
    }
    for piece in [SIGNATURE_HEADER, b": ", signature, b"\r\n\r\n"] {
        pieces.push(piece).expect("bounded signature pieces");
    }
    let mut bytes = 0_usize;
    for piece in &pieces {
        bytes = bytes.checked_add(piece.len()).expect("head length");
    }
    let mut out = skein_lib::Writer::new(bytes);
    for piece in pieces.into_boxed() {
        out.put(piece).expect("measured head");
    }
    out.finish()
}

#[derive(Debug)]
pub struct HookHarness {
    connection: crate::hooks::connection::Hooked,
    forges: List<crate::component::entry::ForgeEntry>,
    word: Box<[u8]>,
    env: Env<Limits>,
    wire: skein_world::stream::Wire,
    closed: bool,
    hints: u32,
    body_demands: u32,
}
pub fn hook() -> HookHarness {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let settings = hook_settings();
    let mut forges = List::with_capacity(env.limits.forges);
    for forge in settings.forges.into_boxed() {
        forges.push(crate::component::entry::ForgeEntry::new(forge, &env.limits)).expect("fixture forges");
    }
    HookHarness {
        connection: crate::hooks::connection::Hooked::new(skein_lib::Token::new(100), &env),
        forges,
        word: settings.word,
        env,
        wire: skein_world::stream::Wire::new(1),
        closed: false,
        hints: 0,
        body_demands: 0,
    }
}
impl HookHarness {
    pub fn write(&mut self, bytes: &[u8]) {
        self.wire.write(bytes);
        self.pump();
    }
    pub fn body_demands(&self) -> u32 {
        self.body_demands
    }
    pub fn hints(&self) -> u32 {
        self.hints
    }
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub fn status(&self) -> Option<u16> {
        if self.wire.received.len() < b"HTTP/1.1 204".len() {
            return None;
        }
        let text = self.wire.received.get(9..12).expect("HTTP status");
        let value = skein_lib::bytes::decimal(text).expect("decimal status");
        Some(u16::try_from(value).expect("status fits"))
    }
    fn pump(&mut self) {
        use skein_lib::stream::{Delimiter, Down, Read};
        let mut up = Queue::with_capacity(16);
        let mut io = Queue::with_capacity(16);
        for _ in 0_u32..1024 {
            self.connection.pump(skein_lib::Token::new(1), &self.env, &self.forges, &self.word, &mut up, &mut io);
            for _ in 0..io.len() {
                match io.pop().expect("queued request") {
                    skein_io::Request::Stream { down, .. } => {
                        if let Down::Demand { read, .. } = &down {
                            let body = match read {
                                Read::Nothing => false,
                                Read::Scan { until, .. } => *until != Delimiter::LF,
                                Read::Fill(_) | Read::Line { .. } | Read::Arrived { .. } => true,
                            };
                            self.body_demands = self.body_demands.saturating_add(u32::from(body));
                        }
                        self.wire.take(down);
                    }
                    skein_io::Request::Close { .. } => self.closed = true,
                    skein_io::Request::Listen { .. }
                    | skein_io::Request::Connect { .. }
                    | skein_io::Request::Bind { .. }
                    | skein_io::Request::Reject { .. }
                    | skein_io::Request::Output { .. }
                    | skein_io::Request::Spawn { .. }
                    | skein_io::Request::SpawnAlone { .. }
                    | skein_io::Request::Signal { .. }
                    | skein_io::Request::Usage { .. }
                    | skein_io::Request::Window { .. }
                    | skein_io::Request::Abort { .. } => unreachable!("hook emits stream/close"),
                }
            }
            for _ in 0..up.len() {
                match up.pop().expect("queued event") {
                    Event::Hint(_) => self.hints = self.hints.saturating_add(1),
                    Event::Installation { .. }
                    | Event::Fact { .. }
                    | Event::GitToken { .. }
                    | Event::Connected { .. }
                    | Event::Answered(_)
                    | Event::Stopped { .. }
                    | Event::Listening { .. }
                    | Event::ListenerLost
                    | Event::Closed => {
                        unreachable!("hook emits hints")
                    }
                }
            }
            if self.closed {
                break;
            }
            if let Some(event) = self.wire.answer() {
                self.connection.stream(&self.env, event);
            }
        }
    }
}

pub fn head_settings(name: &str, tls: bool, agent_bytes: usize) -> Settings {
    let mut settings = settings();
    settings.forges.get_mut(0).expect("forge").transport = if tls {
        Transport::Tls { server_name: skein_tls::Name::new(name).expect("TLS name"), trust: trust() }
    } else {
        Transport::Plaintext { name: Some(Box::from(name.as_bytes())) }
    };
    settings.user_agent = filled(agent_bytes, b'a');
    settings
}

// forge/README.md 3.7 fixed widths, with the fixture's string/body/inner bounds.
pub const ROW_ITEMS: u32 = 2971;
pub const ROW_REVIEWS: u32 = 1691;
pub const ROW_STATUSES: u32 = 544;
pub const ROW_REMARKS: u32 = 1234;
pub const ROW_BRANCHES: u32 = 144;
pub const ROW_COLLABORATORS: u32 = 16;
pub const ROW_PULLFILES: u32 = 2276;
pub const ROW_COMPARE: u32 = ROW_BRANCHES;
pub const ROW_RUNS: u32 = 384;
pub const ROW_TREE: u32 = 192;
pub const MAX_ROW: u32 = ROW_ITEMS;
// The page premise contains two maximal rows, and also tests one row and no room.
pub const PAGE_ROWS: u32 = 2; // The premise budgets exactly two maximal rows.
pub const ONE_PAGE_ROW: u32 = 1; // One maximal row is the admission boundary.
pub const NO_PAGE_ROWS: u32 = 0; // No row fits below the first row byte bound.

pub fn installation() -> skein_forge::github::document::Installation {
    use skein_forge::github::document;
    document::Installation {
        id: 1,
        account: 22,
        login: Box::from(b"acme".as_slice()),
        kind: document::Account::Organization,
        created: Wall::from_nanos(1_000_000_000),
        suspended: false,
    }
}
