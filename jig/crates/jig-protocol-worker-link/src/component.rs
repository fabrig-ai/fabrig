//! Opening, admission, loss and redial transitions
//! (jig/docs/design/protocol/channel/README.md, 4 and 9.1).
use crate::{
    Event, Limits, Loss, REFUSED_FRAMING, REFUSED_LIMITS, Request,
    backoff::Backoff,
    busy::Busy,
    connection::{Connection, Phase},
    grace::Grace,
    limits,
    relays::Relays,
    translate,
};
use alloc::boxed::Box;
use core::mem;
use jig_domain_host as host;
use jig_domain_link::Hello;
use jig_protocol_worker_wire::{self as wire, sizes};
use skein_channel::{self as channel, Closed, Machine, Role};
use skein_lib::{Decimal, Duration, List, Queue, Secret, Time, Token, Writer, bytes};

#[derive(Debug)]
pub struct Credential {
    pub name: Box<[u8]>,
    pub secret: Secret,
    pub application: wire::Application,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    Limits,
    Credential,
    Application,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Life {
    Active,
    Stopping,
    Aborting,
    Closed,
}
#[derive(Debug)]
pub struct Component {
    limits: Limits,
    credential: wire::Credential,
    schema: channel::Schema,
    wires: List<wire::Spoken>,
    translation: Box<wire::translation::Translation>,
    phase: Phase,
    backoff: Backoff,
    events: Queue<channel::Event>,
    below: Queue<channel::Lower>,
    next_part: u64,
    dropped: u64,
    relays: Relays,
    busy: Busy,
    grace: Grace,
    life: Life,
    facts: u64,
    pending_logins: Option<channel::Frame>,
}
impl Component {
    pub fn new(limits: &Limits, jig: List<wire::Spoken>, credential: Credential, seed: u64) -> Result<Self, Unusable> {
        if !limits::valid(limits) {
            return Err(Unusable::Limits);
        }
        let bodies = sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
        let schema = wire::schema_for(&bodies, &jig).ok_or(Unusable::Limits)?;
        let mut spoken_wires = List::uncapped();
        for spoken in &jig {
            spoken_wires.push(spoken.clone()).expect("uncapped wires");
        }
        for spoken in &credential.application.spoken {
            spoken_wires.push(spoken.clone()).or(Err(Unusable::Credential))?;
        }
        let mut wires = List::with_capacity(bodies.credential_wires);
        for spoken in &jig {
            let wire = wire::Wire::new(
                &bodies,
                wire::WireParts {
                    name: bytes::copy_of(spoken.name.as_bytes()),
                    lowest: spoken.lowest,
                    layouts: spoken.layouts.clone(),
                },
            )
            .or(Err(Unusable::Credential))?;
            wires.push(wire).or(Err(Unusable::Credential))?;
        }
        for spoken in &credential.application.spoken {
            let wire = wire::Wire::new(
                &bodies,
                wire::WireParts {
                    name: bytes::copy_of(spoken.name.as_bytes()),
                    lowest: spoken.lowest,
                    layouts: spoken.layouts.clone(),
                },
            )
            .or(Err(Unusable::Credential))?;
            wires.push(wire).or(Err(Unusable::Credential))?;
        }
        let translation =
            Box::new(wire::translation::Translation::new(credential.application).or(Err(Unusable::Application))?);
        let credential = wire::Credential::new(
            &bodies,
            wire::CredentialParts { name: credential.name, secret: credential.secret, wires, output: limits.room },
        );
        let Ok(credential) = credential else { return Err(Unusable::Credential) };
        let backoff = Backoff::new(limits, seed);
        Ok(Self {
            limits: *limits,
            credential,
            schema,
            wires: spoken_wires,
            translation,
            phase: Phase::Idle { at: Time::ZERO },
            backoff,
            events: Queue::uncapped(),
            below: Queue::uncapped(),
            next_part: 1,
            dropped: 0,
            relays: Relays::new(limits),
            busy: Busy::new(limits, seed),
            grace: Grace::new(),
            life: Life::Active,
            facts: 0,
            pending_logins: None,
        })
    }
    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        if self.life == Life::Closed {
            return;
        }
        match event {
            Event::SourceFault => self.lose(now, Loss::Fault, out),
            Event::Stop => self.end(now, false, out),
            Event::Abort => self.end(now, true, out),
            Event::CancelRelay { run, attempt, call } => {
                if self.relays.end(run, attempt, &call) {
                    out.push(Request::RelayCancelled { run, attempt, call });
                }
            }
            Event::Dialled { connection } => self.dialled(now, connection, out),
            Event::Unreachable => match self.phase {
                Phase::Connecting => {
                    if self.life != Life::Active {
                        self.completed(out);
                        return;
                    }
                    let span = self.redial_wait(now);
                    self.phase = Phase::Idle { at: now.saturating_add(span) };
                    out.push(Request::Fact { why: Loss::Unreachable });
                }
                Phase::Idle { .. } | Phase::Opening { .. } | Phase::Open { .. } | Phase::Closing { .. } => {}
            },
            Event::Lower { connection, event } => {
                if self.phase.token() == Some(connection) {
                    match &event {
                        channel::LowerEvent::Read(skein_lib::stream::Up::Bytes(_)) => match &mut self.phase {
                            Phase::Open { connection, .. } => {
                                connection.silence = Some(now.saturating_add(self.limits.silence));
                            }
                            Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
                        },
                        channel::LowerEvent::Read(_)
                        | channel::LowerEvent::Write(_)
                        | channel::LowerEvent::WriteFailed(_) => {}
                    }
                    if let Some(link) = self.phase.connection() {
                        link.machine.up(event, &mut self.events, &mut self.below);
                    }
                    self.pump(now, out);
                }
            }
            Event::Gone { connection } => self.gone(now, connection, out),
            Event::Hello { hello } => {
                if self.life != Life::Active {
                    return;
                }
                self.hello(now, hello, out);
                self.pump(now, out);
            }
            Event::Up { up, parts } => {
                if self.life != Life::Active {
                    return;
                }
                self.up(up, &parts, out);
                self.pump(now, out);
            }
        }
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if self.life == Life::Closed {
            return;
        }
        if self.grace.fire(now) {
            out.push(Request::GracePassed);
        }
        if let Some((run, attempt, turn)) = self.busy.expire(now) {
            out.push(Request::Resend { run, attempt, turn });
        }
        match self.phase {
            Phase::Idle { at } => {
                if now >= at {
                    self.phase = Phase::Connecting;
                    out.push(Request::Dial);
                }
            }
            Phase::Opening { until, .. } => {
                if now >= until {
                    self.lose(now, Loss::Handshake, out);
                    self.pump(now, out);
                }
            }
            Phase::Closing { until, .. } => {
                if let Some(until) = until
                    && now >= until
                {
                    if let Some(connection) = self.phase.token() {
                        out.push(Request::Abort { connection });
                    }
                    match &mut self.phase {
                        Phase::Closing { until, .. } => *until = None,
                        Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Open { .. } => {}
                    }
                }
            }
            Phase::Open { proved, ref mut connection, .. } => {
                let why = if expired(connection.silence, now) {
                    Some(Loss::Silence)
                } else if expired(connection.stall, now) {
                    Some(Loss::Stall)
                } else {
                    None
                };
                if let Some(why) = why {
                    self.lose(now, why, out);
                    self.pump(now, out);
                    return;
                }
                if !connection.queued() && expired(connection.ping, now) {
                    connection.ping = Some(now.saturating_add(self.limits.ping));
                    self.command(channel::Request::Ping);
                }
                if let Some(at) = proved
                    && now >= at
                {
                    self.backoff.proved();
                    match &mut self.phase {
                        Phase::Open { proved, .. } => *proved = None,
                        Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
                    }
                }
            }
            Phase::Connecting => {}
        }
        self.pump(now, out);
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.life == Life::Closed {
            return None;
        }
        let phase = match self.phase {
            Phase::Idle { at } => Some(at),
            Phase::Opening { until, .. } => Some(until),
            Phase::Closing { until, .. } => until,
            Phase::Open { proved, ref connection, .. } => crate::connection::earlier(proved, connection.next()),
            Phase::Connecting => None,
        };
        crate::connection::earlier(crate::connection::earlier(phase, self.busy.next()), self.grace.next())
    }
    #[must_use]
    pub fn dropped_facts(&self) -> u64 {
        self.facts
    }

    /// Records dropped because their bounds or named parts do not fit.
    #[must_use]
    pub fn dropped_records(&self) -> u64 {
        self.dropped
    }

    fn dialled(&mut self, now: Time, token: Token, out: &mut Queue<Request>) {
        match self.phase {
            Phase::Connecting => {}
            Phase::Idle { .. } | Phase::Opening { .. } | Phase::Open { .. } | Phase::Closing { .. } => {
                out.push(Request::Abort { connection: token });
                return;
            }
        }
        let schema = self.schema.clone();
        let channel = limits::channel_limits(&self.limits).expect("constructor checked channel limits");
        let machine = Machine::new(schema, Role::Initiator, channel).expect("constructor checked channel");
        self.phase = Phase::Opening {
            connection: Connection {
                token,
                machine,
                ping: None,
                silence: None,
                stall: None,
                wires: None,
                stated: false,
                version: wire::VERSION,
            },
            until: now.saturating_add(self.limits.handshake),
        };
        if self.life != Life::Active {
            self.close_ending(now);
            self.pump(now, out);
            return;
        }
        let mut writer = Writer::new(skein_lib::length::usize_of(self.credential.measure()));
        self.credential.encode(&mut writer).expect("measured credential fits");
        self.command(channel::Request::Open { credential: writer.finish() });
        self.pump(now, out);
    }
    fn gone(&mut self, now: Time, token: Token, out: &mut Queue<Request>) {
        if self.phase.token() != Some(token) {
            return;
        }
        self.pending_logins = None;
        if self.life != Life::Active {
            self.completed(out);
            return;
        }
        let phase = mem::replace(&mut self.phase, Phase::Connecting);
        let at = match phase {
            Phase::Closing { retry, .. } => retry,
            Phase::Open { .. } => {
                self.busy.clear();
                self.grace.lost(now, self.limits.grace);
                out.push(Request::Lost);
                out.push(Request::Fact { why: Loss::Closed(Closed::Stream) });
                let span = self.redial_wait(now);
                now.saturating_add(span)
            }
            Phase::Opening { .. } => {
                out.push(Request::Fact { why: Loss::Closed(Closed::Stream) });
                let span = self.redial_wait(now);
                now.saturating_add(span)
            }
            Phase::Idle { at } => at,
            Phase::Connecting => now,
        };
        self.phase = Phase::Idle { at };
    }
    fn hello(&mut self, now: Time, value: Hello, out: &mut Queue<Request>) {
        match &mut self.phase {
            Phase::Open { hello, .. } => {
                if *hello {
                    return;
                }
                let limits = sizes::limits(&self.limits.sizes).expect("checked sizes");
                let Some(frame) = translate::hello(value, &limits) else {
                    self.dropped = self.dropped.saturating_add(1);
                    return;
                };
                *hello = true;
                let Some(frame) = self.translated(frame, &limits, out) else { return };
                self.command(channel::Request::Send { token: Token::new(0), frame });
                // Queue replay without polling lower demands: one hello step stays bounded.
                let mut index = 0_u32;
                for _ in 0..self.limits.sizes.slots.saturating_mul(self.limits.sizes.calls) {
                    let Some(replay) = self.relays.replay(index) else { break };
                    match self.phase {
                        Phase::Open { .. } => {}
                        Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => break,
                    }
                    let Some(frame) = self.translated(replay.frame, &limits, out) else {
                        self.relays.end(replay.run, replay.attempt, &replay.call);
                        self.refuse_call(replay.run, replay.attempt, replay.call, b"call refused", out);
                        continue;
                    };
                    index = index.checked_add(1).expect("within relay capacity");
                    self.command(channel::Request::Send { token: Token::new(1), frame });
                    for _ in 0..channel::MAX_UP {
                        let Some(event) = self.events.pop() else { break };
                        self.received(now, event, out);
                    }
                }
            }
            Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
        }
    }
    fn up(&mut self, up: host::LinkUp, parts: &[wire::Part], out: &mut Queue<Request>) {
        let limits = sizes::limits(&self.limits.sizes).expect("checked sizes");
        let up = match up {
            host::LinkUp::Call { run, attempt, contract, call, tool, writes, input, deadline } => {
                if input.len() > skein_lib::length::usize_of(limits.call_input) {
                    self.refuse_call(run, attempt, call, b"call too large", out);
                    return;
                }
                let name = bytes::copy_of(&call);
                let up = host::LinkUp::Call { run, attempt, contract, call, tool, writes, input, deadline };
                let Some(frame) = translate::up(up, parts, &limits) else {
                    self.dropped = self.dropped.saturating_add(1);
                    self.refuse_call(run, attempt, name, b"call refused", out);
                    return;
                };
                if let Err(call) = self.relays.add(run, attempt, bytes::copy_of(&name), &frame) {
                    out.push(Request::Down {
                        down: host::LinkDown::Relayed { run, attempt, call, answer: host::Relayed::Busy },
                        parts: Box::new([]),
                    });
                    return;
                }
                if let Phase::Open { hello: true, .. } = self.phase {
                    let Some(frame) = self.translated(frame, &limits, out) else {
                        self.relays.end(run, attempt, &name);
                        self.refuse_call(run, attempt, name, b"call refused", out);
                        return;
                    };
                    self.command(channel::Request::Send { token: Token::new(1), frame });
                }
                return;
            }
            up @ (host::LinkUp::Ask { .. }
            | host::LinkUp::ListRead { .. }
            | host::LinkUp::Draining
            | host::LinkUp::Logins { .. }
            | host::LinkUp::SignedOut { .. }
            | host::LinkUp::Piece { .. }
            | host::LinkUp::Activity { .. }
            | host::LinkUp::Unauthorized { .. }
            | host::LinkUp::Limited { .. }
            | host::LinkUp::AccountRefused { .. }
            | host::LinkUp::Turn { .. }
            | host::LinkUp::JobAnswer { .. }
            | host::LinkUp::Answer { .. }) => up,
        };
        match self.phase {
            Phase::Open { hello: true, .. } => {
                let Some(frame) = translate::up(up, parts, &limits) else {
                    self.dropped = self.dropped.saturating_add(1);
                    return;
                };
                let Some(frame) = self.translated(frame, &limits, out) else { return };
                if frame.kind() == wire::Kind::Logins.number() {
                    self.pending_logins = Some(frame);
                    return;
                }
                if frame.kind() == 0x0104 && !self.fact_fits(&frame) {
                    self.facts = self.facts.saturating_add(1);
                    return;
                }
                self.command(channel::Request::Send { token: Token::new(1), frame });
            }
            Phase::Idle { .. }
            | Phase::Connecting
            | Phase::Opening { .. }
            | Phase::Open { hello: false, .. }
            | Phase::Closing { .. } => {}
        }
    }
    fn refuse_call(&self, run: Token, attempt: Token, call: Box<[u8]>, reason: &[u8], out: &mut Queue<Request>) {
        let bound = skein_lib::length::usize_of(self.limits.sizes.answer);
        let text = bytes::copy_of(reason.get(..reason.len().min(bound)).expect("bounded ASCII reason"));
        out.push(Request::Down {
            down: host::LinkDown::Relayed { run, attempt, call, answer: host::Relayed::Answer { error: true, text } },
            parts: Box::new([]),
        });
    }
    fn command(&mut self, request: channel::Request) {
        if let Some(link) = self.phase.connection() {
            link.machine.down(request, &mut self.events, &mut self.below);
        }
    }
    fn pump(&mut self, now: Time, out: &mut Queue<Request>) {
        self.send_logins();
        if let Some(link) = self.phase.connection() {
            link.machine.poll(&mut self.events, &mut self.below);
        }
        // Each channel operation emits at most MAX_UP, and handlers perform at most two.
        for _ in 0..channel::MAX_UP.saturating_mul(3) {
            self.drain_lower(now, out);
            let Some(event) = self.events.pop() else { break };
            self.received(now, event, out);
            if let Some(link) = self.phase.connection() {
                link.machine.poll(&mut self.events, &mut self.below);
            }
        }
        self.drain_lower(now, out);
        assert!(self.events.is_empty(), "one channel event cascade fits its bound");
        if let Some(connection) = self.phase.connection() {
            if connection.queued() {
                if connection.stall.is_none() {
                    connection.stall = Some(now.saturating_add(self.limits.stall));
                }
            } else {
                connection.stall = None;
            }
        }
        if self.life == Life::Stopping {
            match &mut self.phase {
                Phase::Open { connection, .. } if !connection.queued() => {
                    self.close_ending(now);
                    self.pump(now, out);
                }
                Phase::Idle { .. }
                | Phase::Connecting
                | Phase::Opening { .. }
                | Phase::Open { .. }
                | Phase::Closing { .. } => {}
            }
        }
    }
    fn drain_lower(&mut self, now: Time, out: &mut Queue<Request>) {
        for _ in 0..channel::MAX_DOWN.saturating_mul(3) {
            let Some(lower) = self.below.pop() else { break };
            match &lower {
                channel::Lower::Write(skein_lib::stream::OutputDown::Send { bytes, .. }) => {
                    let header = channel::parse_header(bytes.get(..8).expect("a whole output frame"))
                        .expect("checked frame header");
                    if header.kind == 0x0100 {
                        match &mut self.phase {
                            Phase::Open { proved, .. } => *proved = Some(now.saturating_add(self.limits.proved)),
                            Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
                        }
                    }
                }
                channel::Lower::Read(_)
                | channel::Lower::Write(
                    skein_lib::stream::OutputDown::Room { .. }
                    | skein_lib::stream::OutputDown::Cancel { .. }
                    | skein_lib::stream::OutputDown::Release { .. },
                )
                | channel::Lower::FinishWrite => {}
            }
            if let Some(connection) = self.phase.token() {
                out.push(Request::Lower { connection, lower });
            }
        }
    }
    fn received(&mut self, now: Time, event: channel::Event, out: &mut Queue<Request>) {
        match &event {
            channel::Event::Body { .. } | channel::Event::Ping | channel::Event::Unsupported { .. } => {
                if self.life == Life::Active
                    && let Some(connection) = self.phase.connection()
                {
                    connection.silence = Some(now.saturating_add(self.limits.silence));
                }
            }
            channel::Event::Ready { .. }
            | channel::Event::Refused { .. }
            | channel::Event::Closed { .. }
            | channel::Event::Ended
            | channel::Event::OutputFailed
            | channel::Event::Unsent { .. }
            | channel::Event::Opening { .. }
            | channel::Event::Drained
            | channel::Event::Sent { .. } => {}
        }
        match event {
            channel::Event::Ready { version, terms } => self.ready(now, version, &terms, out),
            channel::Event::Refused { reason, text } => {
                out.push(Request::Refused { reason, text });
                self.lose(now, Loss::Closed(Closed::RefusedPeer(reason)), out);
            }
            channel::Event::Closed { why, .. } => {
                if self.life != Life::Active {
                    if let Some(connection) = self.phase.token() {
                        if self.life == Life::Aborting {
                            out.push(Request::Abort { connection });
                        } else {
                            out.push(Request::Close { connection });
                        }
                    }
                    return;
                }
                let started = self.begin_loss(now, Loss::Closed(why), out);
                if started {
                    self.command(channel::Request::Close);
                }
                if let Some(connection) = self.phase.token() {
                    out.push(Request::Close { connection });
                }
            }
            channel::Event::Ended | channel::Event::OutputFailed => self.lose(now, Loss::Closed(Closed::Stream), out),
            channel::Event::Body { kind, body } => self.body(now, kind, &body, out),
            channel::Event::Ping | channel::Event::Unsupported { .. } => {
                self.command(channel::Request::Read);
            }
            channel::Event::Unsent { why, .. } => {
                let why = match why {
                    channel::Unsent::Full => Loss::Full,
                    channel::Unsent::WrongDirection
                    | channel::Unsent::PeerDoesNotTake
                    | channel::Unsent::TooLarge
                    | channel::Unsent::Closed => Loss::Closed(Closed::Owner),
                };
                self.lose(now, why, out);
            }
            channel::Event::Opening { .. } => self.lose(now, Loss::Closed(Closed::Framing), out),
            channel::Event::Drained => {
                self.send_logins();
                if let Some(connection) = self.phase.connection() {
                    if self.life == Life::Active {
                        connection.ping = Some(now.saturating_add(self.limits.ping));
                    }
                    connection.stall = Some(now.saturating_add(self.limits.stall));
                }
            }
            channel::Event::Sent { .. } => {}
        }
    }
    fn ready(&mut self, now: Time, version: u16, terms: &List<channel::Term>, out: &mut Queue<Request>) {
        if let Some(kind) = wire::terms_gap(&self.schema, version, Role::Initiator, terms) {
            self.command(channel::Request::Refuse {
                reason: REFUSED_LIMITS,
                text: bytes::copy_of(Decimal::of(u64::from(kind)).as_bytes()),
            });
            let started = self.begin_loss(now, Loss::Closed(Closed::RefusedHere(REFUSED_LIMITS)), out);
            assert!(started, "terms are checked during opening");
        } else {
            let singleton = crate::wires::singletons(&self.wires);
            let statement =
                self.schema.version(version).expect("agreed version").kind(wire::Kind::Wires.number()).is_some();
            if !statement && singleton.is_none() {
                self.lose(now, Loss::EngineStatesNoWires, out);
                return;
            }
            let ready = singleton.is_some();
            if let Some(connection) = self.phase.connection() {
                connection.wires = singleton;
                connection.version = version;
            }
            let phase = mem::replace(&mut self.phase, Phase::Connecting);
            self.phase = match phase {
                Phase::Opening { connection, .. } => Phase::Open { connection, proved: None, hello: false },
                other @ (Phase::Idle { .. } | Phase::Connecting | Phase::Open { .. } | Phase::Closing { .. }) => other,
            };
            if let Some(connection) = self.phase.connection() {
                connection.live(now, &self.limits);
            }
            if ready {
                out.push(Request::Ready);
            }
            self.command(channel::Request::Read);
        }
    }
    #[expect(clippy::too_many_lines, reason = "translate before dispatching statement and current records")]
    fn body(&mut self, now: Time, kind: u16, body: &[u8], out: &mut Queue<Request>) {
        let limits = sizes::limits(&self.limits.sizes).expect("checked sizes");
        let Some(connection) = self.phase.connection() else { return };
        let agreed = match &connection.wires {
            Some(wires) => wires.as_slice(),
            None => &[],
        };
        let Ok(frame) = translate::receive(&mut self.translation, kind, body, connection.version, agreed, &limits)
        else {
            if kind == wire::Kind::Wires.number() {
                self.lose(now, Loss::Fault, out);
            } else {
                self.command(channel::Request::Refuse { reason: REFUSED_FRAMING, text: Box::new([]) });
                self.begin_loss(now, Loss::Closed(Closed::RefusedHere(REFUSED_FRAMING)), out);
            }
            return;
        };
        let kind = frame.kind();
        let body = frame.bytes().get(8..).expect("validated frame header");
        if kind == wire::Kind::Wires.number() {
            let record = wire::v14::Agreed::decode(&limits, &mut skein_lib::Reader::new(body));
            let agreed = match record {
                Ok(record) => crate::wires::statement(&self.wires, &record),
                Err(_) => None,
            };
            let Some(agreed) = agreed else {
                self.lose(now, Loss::Fault, out);
                return;
            };
            if crate::wires::singletons(&self.wires).is_none() {
                let incomplete = self.translation.incomplete(agreed.as_slice());
                if !incomplete.is_empty() {
                    for wire in &incomplete {
                        out.push(Request::CannotSendWhole { wire: wire.name, version: wire.version });
                    }
                    self.lose(now, Loss::Fault, out);
                    return;
                }
            }
            let Some(connection) = self.phase.connection() else { return };
            if connection.stated {
                self.lose(now, Loss::Fault, out);
                return;
            }
            let ready = connection.wires.is_none();
            connection.wires = Some(agreed);
            connection.stated = true;
            if ready {
                out.push(Request::Ready);
            }
            self.command(channel::Request::Read);
            return;
        }
        if let Some(connection) = self.phase.connection()
            && connection.wires.is_none()
        {
            self.lose(now, Loss::Fault, out);
            return;
        }
        if kind == 0x0184 {
            let record = wire::TurnBusy::decode(&limits, &mut skein_lib::Reader::new(body));
            match record {
                Ok(record) => {
                    self.admitted();
                    if self.busy.add(now, Token::new(record.task()), Token::new(record.attempt()), record.number()) {
                        self.command(channel::Request::Read);
                    } else {
                        self.lose(now, Loss::Busy, out);
                    }
                }
                Err(_) => {
                    self.command(channel::Request::Refuse { reason: REFUSED_FRAMING, text: Box::new([]) });
                    self.begin_loss(now, Loss::Closed(Closed::RefusedHere(REFUSED_FRAMING)), out);
                }
            }
            return;
        }
        let mut parts = Queue::uncapped();
        match translate::down(kind, body, &limits, now, &mut parts, &mut self.next_part, out) {
            Some(down) => {
                self.admitted();
                let forward = match &down {
                    host::LinkDown::Relayed { run, attempt, call, .. } => self.relays.end(*run, *attempt, call),
                    host::LinkDown::TurnKept { run, attempt, turn } => {
                        self.busy.kept(*run, *attempt, *turn);
                        true
                    }
                    host::LinkDown::Acknowledge { run, attempt } => {
                        self.busy.acknowledged(*run, *attempt);
                        true
                    }
                    host::LinkDown::ReadList { .. }
                    | host::LinkDown::Sources { .. }
                    | host::LinkDown::Check { .. }
                    | host::LinkDown::SignOut { .. }
                    | host::LinkDown::AssignJob { .. }
                    | host::LinkDown::Assign { .. }
                    | host::LinkDown::Message { .. }
                    | host::LinkDown::Cancel { .. }
                    | host::LinkDown::Asked { .. }
                    | host::LinkDown::Grant { .. } => true,
                };
                if forward {
                    // One translated record carries at most its workspace payload.
                    let mut payloads = List::with_capacity(parts.len());
                    for _ in 0..parts.len() {
                        payloads.push(parts.pop().expect("one queued part per slot")).expect("one slot per part");
                    }
                    out.push(Request::Down { down, parts: payloads.into_boxed() });
                }
                self.command(channel::Request::Read);
            }
            None => {
                self.command(channel::Request::Refuse { reason: REFUSED_FRAMING, text: Box::new([]) });
                let started = self.begin_loss(now, Loss::Closed(Closed::RefusedHere(REFUSED_FRAMING)), out);
                assert!(started, "a body arrives on an open channel");
            }
        }
    }
    fn admitted(&mut self) {
        match self.phase {
            Phase::Open { hello: true, .. } => self.grace.clear(),
            Phase::Open { hello: false, .. }
            | Phase::Idle { .. }
            | Phase::Connecting
            | Phase::Opening { .. }
            | Phase::Closing { .. } => {}
        }
    }
    fn redial_wait(&mut self, now: Time) -> Duration {
        let span = self.backoff.next();
        let Some(end) = self.grace.next() else { return span };
        let quarter = self.limits.grace.as_nanos().checked_div(4).expect("nonzero divisor");
        let span = span.min(Duration::from_nanos(quarter));
        let last = Time::from_nanos(end.as_nanos().saturating_sub(self.limits.handshake.as_nanos()));
        if now < last { span.min(last.saturating_since(now)) } else { span }
    }
    fn send_logins(&mut self) {
        let Some(connection) = self.phase.connection() else { return };
        if connection.queued() {
            return;
        }
        let Some(frame) = self.pending_logins.take() else { return };
        self.command(channel::Request::Send { token: Token::new(1), frame });
    }
    fn fact_fits(&mut self, frame: &channel::Frame) -> bool {
        let cap = sizes::worker_cap(&self.limits.sizes).expect("checked cap");
        let rest = cap.cap.bytes.checked_sub(cap.facts).expect("facts included in cap");
        let frames = cap.cap.frames.checked_sub(cap.facts.div_euclid(25)).expect("facts included in cap");
        let Some(connection) = self.phase.connection() else { return false };
        let room = connection.machine.room();
        room.bytes.saturating_sub(rest) >= frame.wire_len() && room.frames > frames
    }
    fn end(&mut self, now: Time, abort: bool, out: &mut Queue<Request>) {
        self.pending_logins = None;
        if self.life == Life::Closed {
            return;
        }
        if self.life == Life::Aborting || (self.life == Life::Stopping && !abort) {
            return;
        }
        self.life = if abort { Life::Aborting } else { Life::Stopping };
        self.busy.clear();
        self.grace.clear();
        if let Some(connection) = self.phase.connection() {
            connection.quiet();
        }
        match &mut self.phase {
            Phase::Open { proved, .. } => *proved = None,
            Phase::Idle { .. } | Phase::Connecting | Phase::Opening { .. } | Phase::Closing { .. } => {}
        }
        match self.phase {
            Phase::Idle { .. } => self.completed(out),
            Phase::Connecting => {}
            Phase::Open { .. } if !abort => self.command(channel::Request::Finish),
            Phase::Opening { .. } | Phase::Open { .. } | Phase::Closing { .. } => self.close_ending(now),
        }
        self.pump(now, out);
    }
    fn close_ending(&mut self, now: Time) {
        let phase = mem::replace(&mut self.phase, Phase::Connecting);
        self.phase = match phase {
            Phase::Opening { connection, .. } | Phase::Open { connection, .. } | Phase::Closing { connection, .. } => {
                Phase::Closing { connection, retry: Time::ZERO, until: Some(now.saturating_add(self.limits.stall)) }
            }
            other @ (Phase::Idle { .. } | Phase::Connecting) => other,
        };
        self.command(channel::Request::Close);
    }
    fn completed(&mut self, out: &mut Queue<Request>) {
        self.phase = Phase::Idle { at: Time::ZERO };
        self.life = Life::Closed;
        self.grace.clear();
        self.busy.clear();
        out.push(Request::Closed);
    }
    fn lose(&mut self, now: Time, why: Loss, out: &mut Queue<Request>) {
        if self.life != Life::Active {
            self.life = Life::Aborting;
            self.close_ending(now);
            return;
        }
        if self.begin_loss(now, why, out) {
            self.command(channel::Request::Close);
        }
    }
    fn begin_loss(&mut self, now: Time, why: Loss, out: &mut Queue<Request>) -> bool {
        self.pending_logins = None;
        let phase = mem::replace(&mut self.phase, Phase::Connecting);
        let mut started = false;
        self.phase = match phase {
            Phase::Open { mut connection, .. } => {
                self.busy.clear();
                self.grace.lost(now, self.limits.grace);
                connection.quiet();
                connection.stall = None;
                out.push(Request::Lost);
                let span = self.redial_wait(now);
                out.push(Request::Fact { why });
                started = true;
                Phase::Closing {
                    connection,
                    retry: now.saturating_add(span),
                    until: Some(now.saturating_add(self.limits.handshake)),
                }
            }
            Phase::Opening { connection, .. } => {
                let span = self.redial_wait(now);
                out.push(Request::Fact { why });
                started = true;
                Phase::Closing {
                    connection,
                    retry: now.saturating_add(span),
                    until: Some(now.saturating_add(self.limits.handshake)),
                }
            }
            other @ (Phase::Idle { .. } | Phase::Connecting | Phase::Closing { .. }) => other,
        };
        started
    }
}

fn expired(at: Option<Time>, now: Time) -> bool {
    match at {
        Some(at) => now >= at,
        None => false,
    }
}

impl Component {
    fn translated(
        &mut self,
        frame: channel::Frame,
        limits: &wire::v14::Limits,
        out: &mut Queue<Request>,
    ) -> Option<channel::Frame> {
        let kind = frame.kind();
        let connection = self.phase.connection()?;
        let agreed = match &connection.wires {
            Some(wires) => wires.as_slice(),
            None => &[],
        };
        match translate::send(&mut self.translation, frame, connection.version, agreed, limits) {
            Ok(frame) => Some(frame),
            Err(error) => {
                self.dropped = self.dropped.saturating_add(1);
                out.push(Request::Untranslated { kind, halt: error.halt });
                None
            }
        }
    }
}
