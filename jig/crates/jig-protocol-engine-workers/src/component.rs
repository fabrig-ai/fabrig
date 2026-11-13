//! Routing (protocol/channel/README.md, sections 4 and 9.2); Debug (jig/docs/design/protocol/service.md, 11.3).
use crate::{
    Bound, Event, Limits, Loss, Request, Unsent, Worker,
    connection::{Connection, Ending, Phase},
    opening::Verdict,
};
use alloc::boxed::Box;
use jig_protocol_worker_wire::{self as wire, Reason};
use skein_channel::{Closed, Event as ChannelEvent, Request as ChannelRequest};
use skein_lib::{Decimal, List, Map, Queue, Reader, Time, Token, bytes};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    Limits,
    Workers,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Life {
    Running,
    Stopping,
    Closing,
    Aborting,
    Closed,
}
#[derive(Clone, Copy, Debug)]
enum Reply {
    Assignment { run: Token, attempt: Token },
    Job { run: Token, attempt: Token },
    ReadList { read: u64 },
    Drop,
}
#[derive(Clone, Copy, Debug)]
struct Sending {
    kind: wire::Kind,
    bytes: u32,
    reply: Reply,
}
pub struct Component {
    pub(crate) limits: Limits,
    pub(crate) workers: Box<[Worker]>,
    pub(crate) wires: List<wire::Spoken>,
    pub(crate) codec: wire::v11::Limits,
    pub(crate) connections: Map<Token, Connection>,
    schema: skein_channel::Schema,
    next_channel: u64,
    next_part: u64,
    life: Life,
    dropped_down: u64,
    sending: Option<Sending>,
}
impl core::fmt::Debug for Component {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Component").field(&self.life).field(&(self.connections.len(), self.dropped_down)).finish()
    }
}
impl Component {
    pub fn new(limits: &Limits, workers: Box<[Worker]>, application: List<wire::Spoken>) -> Result<Self, Unusable> {
        if !crate::valid(limits) {
            return Err(Unusable::Limits);
        }
        let codec = wire::sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
        let schema = crate::limits::schema(limits).ok_or(Unusable::Limits)?;
        if workers.len() > skein_lib::length::usize_of(limits.names) {
            return Err(Unusable::Workers);
        }
        for (index, worker) in workers.iter().enumerate() {
            if worker.name.len() > skein_lib::length::usize_of(limits.sizes.name) {
                return Err(Unusable::Workers);
            }
            for earlier in workers.iter().take(index) {
                if earlier.name == worker.name {
                    return Err(Unusable::Workers);
                }
            }
        }
        let mut wires = List::uncapped();
        for wire in &wire::spoken() {
            wires.push(wire.clone()).expect("uncapped wire list");
        }
        for wire in &application {
            wires.push(wire.clone()).expect("uncapped wire list");
        }
        let connections = Map::with_capacity(limits.workers.checked_add(limits.opening).expect("checked limits"));
        Ok(Self {
            limits: *limits,
            workers,
            wires,
            codec,
            connections,
            schema,
            next_channel: 1,
            next_part: 1,
            life: Life::Running,
            dropped_down: 0,
            sending: None,
        })
    }
    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Accepted { connection } => self.accepted(now, connection, out),
            Event::Lower { connection, event } => self.lower(now, connection, event, out),
            Event::Gone { connection } => {
                if let Some(mut state) = self.connections.remove(&connection) {
                    ended(connection, &mut state, Loss::Stream, out);
                    lost(&mut state, out);
                }
            }
            Event::Stop => {
                if self.life == Life::Running {
                    self.life = Life::Stopping;
                }
            }
            Event::Close => {
                if self.life != Life::Closed && self.life != Life::Aborting {
                    self.life = Life::Closing;
                }
                self.fire(now, out);
            }
            Event::Abort => {
                if self.life != Life::Closed {
                    self.life = Life::Aborting;
                }
                self.fire(now, out);
            }
            event @ Event::Down { .. } => self.deliver(now, event, out),
            Event::TurnBusy { channel, run, attempt, turn } => {
                if let Some(connection) = self.target(channel) {
                    let record = crate::translate::turn_busy(run, attempt, turn, &self.codec);
                    self.send(now, connection, Some(record), Reply::Drop, out);
                } else {
                    self.dropped_down = self.dropped_down.saturating_add(1);
                }
            }
            Event::NotAdmitted { channel, text } => {
                if let Some(connection) = self.target(channel) {
                    let mut state = self.connections.remove(&connection).expect("heard channel has its connection");
                    self.refuse(now, connection, &mut state, Reason::NotAdmitted.code(), text, out);
                    self.progress(now, connection, &mut state, out);
                    self.keep(connection, state);
                }
            }
        }
        self.completed(out);
    }
    fn lower(&mut self, now: Time, connection: Token, event: skein_channel::LowerEvent, out: &mut Queue<Request>) {
        if let Some(mut state) = self.connections.remove(&connection) {
            match &event {
                skein_channel::LowerEvent::Read(skein_lib::stream::Up::Bytes(_)) if state.phase == Phase::Open => {
                    state.silence = Some(now.saturating_add(self.limits.silence));
                }
                skein_channel::LowerEvent::Read(_)
                | skein_channel::LowerEvent::Write(_)
                | skein_channel::LowerEvent::WriteFailed(_) => {}
            }
            state.up(event, connection, out);
            self.progress(now, connection, &mut state, out);
            self.keep(connection, state);
        }
    }
    fn deliver(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        if let Event::Down { channel, down, parts, values } = event {
            if let Some(connection) = self.target(channel) {
                let reply = match &down {
                    jig_domain_host::LinkDown::Assign { assignment } => {
                        Reply::Assignment { run: assignment.assignment.run, attempt: assignment.assignment.attempt }
                    }
                    jig_domain_host::LinkDown::AssignJob { job } => Reply::Job { run: job.run, attempt: job.attempt },
                    jig_domain_host::LinkDown::ReadList { read, .. } => Reply::ReadList { read: *read },
                    jig_domain_host::LinkDown::Sources { .. }
                    | jig_domain_host::LinkDown::Check { .. }
                    | jig_domain_host::LinkDown::SignOut { .. }
                    | jig_domain_host::LinkDown::Message { .. }
                    | jig_domain_host::LinkDown::Relayed { .. }
                    | jig_domain_host::LinkDown::TurnKept { .. }
                    | jig_domain_host::LinkDown::Acknowledge { .. }
                    | jig_domain_host::LinkDown::Cancel { .. }
                    | jig_domain_host::LinkDown::Grant { .. } => Reply::Drop,
                };
                let record = crate::translate::down(down, &parts, values, &self.codec);
                if record.is_none() {
                    let words: &[u8] = match reply {
                        Reply::Job { .. } => b"job cannot encode",
                        Reply::Assignment { .. } | Reply::ReadList { .. } | Reply::Drop => b"assignment cannot encode",
                    };
                    self.refused_record(channel, reply, words, out);
                }
                self.send(now, connection, record, reply, out);
            } else {
                self.dropped_down = self.dropped_down.saturating_add(1);
            }
        }
    }
    fn completed(&mut self, out: &mut Queue<Request>) {
        if (self.life == Life::Closing || self.life == Life::Aborting) && self.connections.is_empty() {
            self.life = Life::Closed;
            out.push(Request::Closed);
        }
    }
    fn assignment_refused(&self, channel: Token, run: Token, attempt: Token, words: &[u8], out: &mut Queue<Request>) {
        use jig_domain_host as host;
        let why = bytes::utf8_prefix(words, skein_lib::length::usize_of(self.limits.sizes.detail));
        out.push(Request::Up {
            channel,
            up: host::LinkUp::Answer {
                run,
                attempt,
                answer: host::RunAnswer {
                    read: None,
                    refused: Box::new([]),
                    turns: 0,
                    spent: 0,
                    ending: host::Ending::Refused(host::Refusal::Invalid(host::Invalid::Start {
                        cause: host::StartCause::Other,
                        why: Box::from(why),
                    })),
                },
            },
            parts: Box::new([]),
        });
    }
    fn target(&self, channel: Token) -> Option<Token> {
        for (connection, state) in &self.connections {
            if state.channel == channel && state.heard && state.phase == Phase::Open {
                return Some(*connection);
            }
        }
        None
    }
    fn refused_record(&self, channel: Token, reply: Reply, words: &[u8], out: &mut Queue<Request>) {
        match reply {
            Reply::Assignment { run, attempt } => self.assignment_refused(channel, run, attempt, words, out),
            Reply::Job { run, attempt } => {
                let why = bytes::utf8_prefix(words, skein_lib::length::usize_of(self.limits.sizes.detail));
                out.push(Request::Up {
                    channel,
                    up: jig_domain_host::LinkUp::JobAnswer {
                        run,
                        attempt,
                        answer: jig_domain_host::JobEnding::Refused(jig_domain_host::JobRefusal::Invalid(
                            jig_domain_host::JobInvalid::Start { why: Box::from(why) },
                        )),
                    },
                    parts: Box::new([]),
                });
            }
            Reply::ReadList { read } => out.push(Request::Up {
                channel,
                up: jig_domain_host::LinkUp::ListRead {
                    read,
                    outcome: jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent),
                },
                parts: Box::new([]),
            }),
            Reply::Drop => {}
        }
    }
    fn unsent(&mut self, now: Time, state: &mut Connection, sending: Sending, bound: Bound, out: &mut Queue<Request>) {
        out.push(Request::Unsent {
            worker: state.worker.expect("a record sent on an authenticated channel"),
            channel: state.channel,
            unsent: Unsent { kind: sending.kind, bytes: sending.bytes, bound },
        });
        let name: &[u8] = match sending.reply {
            Reply::Job { .. } => b"job",
            Reply::Assignment { .. } | Reply::ReadList { .. } | Reply::Drop => b"assignment",
        };
        let words = unsent_words(name, sending.bytes, bound);
        self.refused_record(state.channel, sending.reply, &words, out);
        match sending.reply {
            Reply::Drop => self.dropped_down = self.dropped_down.saturating_add(1),
            Reply::Assignment { .. } | Reply::Job { .. } | Reply::ReadList { .. } => {}
        }
        state.ping = Some(now.saturating_add(self.limits.ping));
    }
    fn send(
        &mut self,
        now: Time,
        connection: Token,
        record: Option<crate::translate::Down>,
        reply: Reply,
        out: &mut Queue<Request>,
    ) {
        let Some(record) = record else {
            self.dropped_down = self.dropped_down.saturating_add(1);
            return;
        };
        // Fit the translated record before encoding the frame the room measures
        // (jig/docs/design/protocol/channel/README.md, 8 and 10).
        let (record, past) = match record {
            crate::translate::Down::Assign(record) => {
                let (record, fitted) = jig_protocol_worker_wire::fit::fit(record, &self.codec, self.limits.room);
                let past = match fitted {
                    jig_protocol_worker_wire::fit::Fitted::Whole
                    | jig_protocol_worker_wire::fit::Fitted::Cut { .. } => None,
                    jig_protocol_worker_wire::fit::Fitted::Past { bytes } => Some(bytes),
                };
                (crate::translate::Down::Assign(record), past)
            }
            crate::translate::Down::ReadList(_)
            | crate::translate::Down::Sources(_)
            | crate::translate::Down::Check(_)
            | crate::translate::Down::SignOut(_)
            | crate::translate::Down::AssignJob(_)
            | crate::translate::Down::Message(_)
            | crate::translate::Down::Relayed(_)
            | crate::translate::Down::TurnKept(_)
            | crate::translate::Down::TurnBusy(_)
            | crate::translate::Down::Acknowledge(_)
            | crate::translate::Down::Cancel(_)
            | crate::translate::Down::Grant(_) => (record, None),
        };
        let kind = record.kind();
        let mut state = self.connections.remove(&connection).expect("heard channel exists");
        let frame = if past.is_none() { Some(record.frame()) } else { None };
        let bytes = match past {
            Some(bytes) => bytes,
            None => frame.as_ref().expect("a fitting record encoded").wire_len(),
        };
        let sending = Sending { kind, bytes, reply };
        if sending.bytes > self.limits.room {
            self.unsent(now, &mut state, sending, Bound::Room(self.limits.room), out);
        } else {
            self.sending = Some(sending);
            let frame = frame.expect("a fitting record encoded");
            state.down(ChannelRequest::Send { token: state.channel, frame }, connection, out);
            self.progress(now, connection, &mut state, out);
            assert!(self.sending.is_none(), "send answered within its entrance");
        }
        self.keep(connection, state);
    }
    fn accepted(&mut self, now: Time, connection: Token, out: &mut Queue<Request>) {
        if self.life == Life::Closed
            || self.connections.len() == self.connections.capacity()
            || self.connections.contains_key(&connection)
        {
            if !self.connections.contains_key(&connection) {
                let why = if self.life == Life::Closed { Loss::Stopping } else { Loss::NoRoom };
                out.push(Request::Ended { connection, worker: None, channel: None, why });
            }
            out.push(Request::TurnAway { connection });
            return;
        }
        let mut state = Connection::new(self.schema.clone(), &self.limits, now);
        out.push(Request::Take { connection });
        self.progress(now, connection, &mut state, out);
        self.keep(connection, state);
    }
    fn keep(&mut self, connection: Token, state: Connection) {
        let replaced = self.connections.insert(connection, state).expect("an accepted connection has a slot");
        assert!(replaced.is_none(), "one state per connection");
    }
    fn progress(&mut self, now: Time, connection: Token, state: &mut Connection, out: &mut Queue<Request>) {
        // Up produces one owner event; a decision and poll can produce Ready or Closed.
        for _ in 0_u32..3 {
            for _ in 0..skein_channel::MAX_UP {
                let Some(event) = state.above.pop() else { break };
                self.heard(now, connection, state, event, out);
            }
            state.poll(connection, out);
            if state.above.is_empty() {
                break;
            }
        }
        assert!(state.above.is_empty(), "channel decisions settle within three polls");
        if state.terminal {
            return;
        }
        if state.queued() {
            if state.stall.is_none() {
                state.stall = Some(now.saturating_add(self.limits.stall));
            }
        } else {
            state.stall = None;
            if state.ending == Ending::Graceful {
                state.down(ChannelRequest::Close, connection, out);
                for _ in 0..skein_channel::MAX_UP {
                    let Some(event) = state.above.pop() else { break };
                    self.heard(now, connection, state, event, out);
                }
            }
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "opening, ready and terminal events keep their ordered transitions together"
    )]
    fn heard(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        event: ChannelEvent,
        out: &mut Queue<Request>,
    ) {
        match event {
            ChannelEvent::Opening { credential, highest, .. } => {
                let verdict = self.judge(&credential);
                if self.life != Life::Running {
                    self.refuse(now, connection, state, Reason::Stopping.code(), Box::new([]), out);
                    return;
                }
                let worker = match verdict {
                    Verdict::Refuse { worker, reason, text } => {
                        state.worker = worker;
                        self.refuse(now, connection, state, reason.code(), text, out);
                        return;
                    }
                    Verdict::OutputRoom { worker, output, text } => {
                        state.worker = Some(worker);
                        ended(connection, state, Loss::OutputRoom { engine: self.limits.room, worker: output }, out);
                        self.refuse(now, connection, state, 2, text, out);
                        return;
                    }
                    Verdict::Accept { worker, wires, statement } => {
                        state.wires = wires;
                        state.statement = Some(statement);
                        worker
                    }
                    Verdict::Replace { worker, older, wires, statement } => {
                        state.wires = wires;
                        state.statement = Some(statement);
                        if let Some(mut old) = self.connections.remove(&older) {
                            self.refuse(now, older, &mut old, Reason::Replaced.code(), Box::new([]), out);
                            old.poll(older, out);
                            for _ in 0..skein_channel::MAX_UP {
                                let Some(event) = old.above.pop() else { break };
                                match event {
                                    ChannelEvent::Closed { why, text } => {
                                        closed(older, &mut old, why, text, self.life, out);
                                    }
                                    ChannelEvent::Opening { .. }
                                    | ChannelEvent::Ready { .. }
                                    | ChannelEvent::Body { .. }
                                    | ChannelEvent::Ping
                                    | ChannelEvent::Unsupported { .. }
                                    | ChannelEvent::Refused { .. }
                                    | ChannelEvent::Ended
                                    | ChannelEvent::Sent { .. }
                                    | ChannelEvent::Unsent { .. }
                                    | ChannelEvent::Drained
                                    | ChannelEvent::OutputFailed => {}
                                }
                            }
                            self.keep(older, old);
                        }
                        worker
                    }
                };
                state.worker = Some(worker);
                state.channel = Token::new(self.next_channel);
                self.next_channel =
                    self.next_channel.checked_add(1).expect("channel tokens never wrap in a process life");
                state.phase = Phase::Terms;
                state.hello_deadline = Some(now.saturating_add(self.limits.hello));
                state.down(ChannelRequest::Accept { version: highest }, connection, out);
            }
            ChannelEvent::Ready { version, terms } => {
                if let Err(text) = crate::statement::terms(&self.schema, version, &terms) {
                    self.refuse(now, connection, state, 2, text, out);
                } else {
                    let mut statement = state.statement.take().expect("authenticated statement");
                    statement.terms = terms;
                    out.push(Request::Opened {
                        worker: state.worker.expect("authenticated worker"),
                        wires: state.wires.clone(),
                        statement,
                    });
                    state.phase = Phase::Hello;
                    state.deadline = state.hello_deadline;
                    state.down(ChannelRequest::Read, connection, out);
                }
            }
            ChannelEvent::Closed { why, text } => closed(connection, state, why, text, self.life, out),
            ChannelEvent::Body { kind, body } => self.body(now, connection, state, kind, &body, out),
            ChannelEvent::Ping | ChannelEvent::Unsupported { .. } => {
                if state.phase == Phase::Open {
                    state.silence = Some(now.saturating_add(self.limits.silence));
                    state.down(ChannelRequest::Read, connection, out);
                } else if state.phase == Phase::Hello {
                    state.down(ChannelRequest::Read, connection, out);
                }
            }
            ChannelEvent::Ended | ChannelEvent::OutputFailed => {
                self.loss(now, connection, state, Loss::Stream, out);
            }
            ChannelEvent::Refused { reason, .. } => {
                self.loss(now, connection, state, Loss::RefusedPeer { reason }, out);
            }
            ChannelEvent::Unsent { why: skein_channel::Unsent::Full, .. } => {
                self.sending = None;
                self.loss(now, connection, state, Loss::Full, out);
            }
            ChannelEvent::Drained => {
                if state.phase == Phase::Open {
                    state.ping = Some(now.saturating_add(self.limits.ping));
                }
                state.stall = Some(now.saturating_add(self.limits.stall));
            }
            ChannelEvent::Unsent { why, .. } => {
                let sending = self.sending.take().expect("unsent answers the current send");
                self.unsent(now, state, sending, Bound::Channel(why), out);
            }
            ChannelEvent::Sent { .. } => self.sending = None,
        }
    }
    fn loss(&self, now: Time, connection: Token, state: &mut Connection, why: Loss, out: &mut Queue<Request>) {
        ended(connection, state, why, out);
        lose(now, self.limits.stall, state, out);
        state.down(ChannelRequest::Close, connection, out);
    }
    fn body(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        kind: u16,
        body: &[u8],
        out: &mut Queue<Request>,
    ) {
        match state.phase {
            Phase::Hello => {
                if kind == 0x0100
                    && let Ok(record) = wire::Hello::decode(&self.codec, &mut Reader::new(body))
                {
                    let Some(worker) = state.worker else {
                        self.refuse(now, connection, state, 3, Box::new([]), out);
                        return;
                    };
                    let Some(hello) = crate::translate::hello(record) else {
                        self.refuse(now, connection, state, 3, Box::new([]), out);
                        return;
                    };
                    state.phase = Phase::Open;
                    state.heard = true;
                    state.deadline = None;
                    state.ping = Some(now.saturating_add(self.limits.ping));
                    state.silence = Some(now.saturating_add(self.limits.silence));
                    out.push(Request::Hello { channel: state.channel, worker, hello });
                    state.down(ChannelRequest::Read, connection, out);
                    return;
                }
                self.refuse(now, connection, state, 3, Box::new([]), out);
            }
            Phase::Open => {
                state.silence = Some(now.saturating_add(self.limits.silence));
                if let Some(record) = crate::translate::decode(kind, body, &self.codec) {
                    let mut parts = crate::translate::Parts::new(self.next_part);
                    let up = crate::translate::up(record, &mut parts);
                    self.next_part = parts.next();
                    out.push(Request::Up { channel: state.channel, up, parts: parts.finish() });
                    state.down(ChannelRequest::Read, connection, out);
                } else {
                    self.refuse(now, connection, state, 3, Box::new([]), out);
                }
            }
            Phase::Closing => {}
            Phase::Opening | Phase::Terms => self.refuse(now, connection, state, 3, Box::new([]), out),
        }
    }
    fn refuse(
        &self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        reason: u16,
        text: Box<[u8]>,
        out: &mut Queue<Request>,
    ) {
        if !state.heard && !state.refusal {
            out.push(Request::Refused { worker: state.worker, reason, text: text.clone() });
        }
        state.refusal = true;
        ended(connection, state, Loss::Refused { reason }, out);
        lose(now, self.limits.stall, state, out);
        state.worker = None;
        state.down(ChannelRequest::Refuse { reason, text }, connection, out);
    }
    pub fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        let mut due = None;
        for (token, state) in &self.connections {
            if self.shutdown_pending(state) || expired(state.next_deadline(), now) {
                due = Some(*token);
                break;
            }
        }
        if let Some(connection) = due {
            let mut state = self.connections.remove(&connection).expect("due connection exists");
            if self.shutdown_pending(&state) {
                ended(connection, &mut state, Loss::Stopping, out);
                state.phase = Phase::Closing;
                state.ping = None;
                state.silence = None;
                state.stall = None;
                state.deadline = Some(now.saturating_add(self.limits.stall));
                state.ending = if self.life == Life::Aborting { Ending::Abort } else { Ending::Graceful };
                if self.life == Life::Aborting {
                    terminate(connection, &mut state, out);
                } else {
                    state.down(ChannelRequest::Finish, connection, out);
                }
            } else if state.phase == Phase::Closing && expired(state.deadline, now) && !state.terminal {
                ended(connection, &mut state, Loss::Stream, out);
                terminate(connection, &mut state, out);
            } else if expired(state.deadline, now) || expired(state.silence, now) || expired(state.stall, now) {
                let why = if expired(state.deadline, now) {
                    Loss::Deadline
                } else if expired(state.silence, now) {
                    Loss::Silence
                } else {
                    Loss::Stall
                };
                ended(connection, &mut state, why, out);
                lose(now, self.limits.stall, &mut state, out);
                state.worker = None;
                state.down(ChannelRequest::Close, connection, out);
            } else {
                state.ping = Some(now.saturating_add(self.limits.ping));
                state.down(ChannelRequest::Ping, connection, out);
            }
            self.progress(now, connection, &mut state, out);
            self.keep(connection, state);
        }
        self.completed(out);
    }
    fn shutdown_pending(&self, state: &Connection) -> bool {
        (self.life == Life::Closing || self.life == Life::Aborting)
            && !state.terminal
            && ((state.ending == Ending::Active && (self.life == Life::Aborting || state.phase != Phase::Closing))
                || (self.life == Life::Aborting && state.ending == Ending::Graceful))
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for (_, state) in &self.connections {
            if self.shutdown_pending(state) {
                return Some(Time::ZERO);
            }
            if let Some(deadline) = state.next_deadline() {
                next = Some(match next {
                    Some(before) => deadline.min(before),
                    None => deadline,
                });
            }
        }
        next
    }
    /// Outputs dropped because their channel, parts or values are unavailable, or bounds do not fit.
    #[must_use]
    pub fn dropped_down(&self) -> u64 {
        self.dropped_down
    }
}

fn closed(
    connection: Token,
    state: &mut Connection,
    why: Closed,
    text: Box<[u8]>,
    life: Life,
    out: &mut Queue<Request>,
) {
    if state.terminal {
        return;
    }
    if let Closed::RefusedHere(reason) = why
        && !state.heard
        && !state.refusal
    {
        out.push(Request::Refused { worker: state.worker, reason, text });
        state.refusal = true;
    }
    let loss = match why {
        Closed::RefusedHere(reason) => Loss::Refused { reason },
        Closed::RefusedPeer(reason) => Loss::RefusedPeer { reason },
        Closed::Framing => Loss::Framing,
        Closed::Stream | Closed::Truncated => Loss::Stream,
        Closed::Owner => {
            if life == Life::Closing || life == Life::Aborting {
                Loss::Stopping
            } else {
                Loss::Stream
            }
        }
    };
    ended(connection, state, loss, out);
    lost(state, out);
    state.terminal = true;
    state.worker = None;
    state.deadline = None;
    state.ping = None;
    state.silence = None;
    state.stall = None;
    state.phase = Phase::Closing;
    match why {
        Closed::Owner if state.ending == Ending::Graceful => out.push(Request::Close { connection }),
        Closed::RefusedHere(_) => out.push(Request::Close { connection }),
        Closed::Owner | Closed::RefusedPeer(_) | Closed::Framing | Closed::Stream | Closed::Truncated => {
            out.push(Request::Abort { connection });
        }
    }
}

fn lost(state: &mut Connection, out: &mut Queue<Request>) {
    if state.heard {
        state.heard = false;
        out.push(Request::Lost { channel: state.channel });
    }
}

fn expired(deadline: Option<Time>, now: Time) -> bool {
    match deadline {
        Some(at) => at <= now,
        None => false,
    }
}

fn lose(now: Time, stall: skein_lib::Duration, state: &mut Connection, out: &mut Queue<Request>) {
    if state.phase == Phase::Closing || state.terminal {
        return;
    }
    lost(state, out);
    state.phase = Phase::Closing;
    state.ping = None;
    state.silence = None;
    state.stall = None;
    state.deadline = Some(now.saturating_add(stall));
}

fn ended(connection: Token, state: &mut Connection, why: Loss, out: &mut Queue<Request>) {
    if state.loss.is_some() {
        return;
    }
    state.loss = Some(why);
    assert!(!state.ended, "one end per accepted connection");
    state.ended = true;
    out.push(Request::Ended {
        connection,
        worker: state.worker,
        channel: if state.heard { Some(state.channel) } else { None },
        why,
    });
}

fn terminate(connection: Token, state: &mut Connection, out: &mut Queue<Request>) {
    lost(state, out);
    state.terminal = true;
    state.worker = None;
    state.deadline = None;
    state.ping = None;
    state.silence = None;
    state.stall = None;
    out.push(Request::Abort { connection });
}

/// ASCII diagnostics for an unsent start (jig/docs/design/protocol/channel/records.md, 2, and jig/docs/design/protocol/channel/README.md, 8).
fn unsent_words(name: &[u8], bytes: u32, bound: Bound) -> Box<[u8]> {
    let count = Decimal::of(u64::from(bytes));
    let room = match bound {
        Bound::Room(room) => room,
        Bound::Channel(_) => 0,
    };
    let room = Decimal::of(u64::from(room));
    let (middle, value, end): (&[u8], &[u8], &[u8]) = match bound {
        Bound::Room(_) => (b" bytes past the link's room of ", room.as_bytes(), b" bytes"),
        Bound::Channel(why) => {
            let reason: &[u8] = match why {
                skein_channel::Unsent::WrongDirection => b"wrong-direction",
                skein_channel::Unsent::PeerDoesNotTake => b"not-taken",
                skein_channel::Unsent::TooLarge => b"too-large",
                skein_channel::Unsent::Closed => b"closed",
                skein_channel::Unsent::Full => unreachable!("full loses the link"),
            };
            (b" bytes refused by the channel: ", reason, b"")
        }
    };
    let parts = [name, b" of ", count.as_bytes(), middle, value, end];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded words");
    }
    let mut words = skein_lib::Writer::new(length);
    for part in parts {
        words.put(part).expect("measured words");
    }
    words.finish()
}
