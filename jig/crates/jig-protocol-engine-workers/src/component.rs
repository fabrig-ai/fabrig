//! Routing (protocol/channel/README.md, section 4, and protocol/channel/worker-link.md, section 1.2); Debug (jig/docs/design/protocol/service-parts.md, 1.3).
use crate::{
    Bound, Event, Limits, Loss, Request, Unsent, Worker,
    connection::{Connection, Ending, Phase},
    opening::Verdict,
    statement::{self, Fitted},
};
use alloc::boxed::Box;
use jig_protocol_worker_wire::Record;
use jig_protocol_worker_wire::{self as wire, Reason, convert};
use skein_channel::{Closed, Event as ChannelEvent, Frame, Request as ChannelRequest};
use skein_lib::{Decimal, Duration, List, Map, Queue, Reader, Time, Token, bytes};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unusable {
    Limits,
    Workers,
    Application,
}
/// Checked engine texts and application roots, reusable across components.
#[derive(Debug)]
pub struct PreparedWires {
    wires: List<wire::Spoken>,
    plans: wire::layouts::Plans,
}
impl PreparedWires {
    pub fn new(jig: List<wire::Spoken>, application: wire::Application) -> Result<Self, Unusable> {
        let mut wires = List::uncapped();
        for wire in jig.into_boxed() {
            wires.push(wire).expect("uncapped wire list");
        }
        for wire in application.spoken.into_boxed() {
            wires.push(wire).expect("uncapped wire list");
        }
        let plans = statement::plans(&wires, &application.roots).or(Err(Unusable::Application))?;
        Ok(Self { wires, plans })
    }

    /// Copies the preparation for a component with no remembered openings.
    #[must_use]
    pub fn fresh(&self) -> Self {
        Self { wires: self.wires.clone(), plans: self.plans.fresh() }
    }
}

enum WireSetup {
    Texts { jig: List<wire::Spoken>, application: wire::Application },
    Prepared(PreparedWires),
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
    workers_ready: bool,
    names: Map<u32, Box<[u8]>>,
    pub(crate) wires: List<wire::Spoken>,
    pub(crate) codec: wire::v22::Limits,
    pub(crate) connections: Map<Token, Connection>,
    pub(crate) figures: Option<wire::Figures>,
    schema: skein_channel::Schema,
    channel_limits: skein_channel::Limits,
    next_channel: u64,
    next_part: u64,
    life: Life,
    dropped_down: u64,
    sending: Option<Sending>,
    pub(crate) plans: wire::layouts::Plans,
}
impl core::fmt::Debug for Component {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Component").field(&self.life).field(&(self.connections.len(), self.dropped_down)).finish()
    }
}
impl Component {
    pub fn new(
        limits: &Limits,
        workers: Box<[Worker]>,
        jig: List<wire::Spoken>,
        application: wire::Application,
    ) -> Result<Self, Unusable> {
        let full_schema = crate::limits::schema(limits).ok_or(Unusable::Limits)?;
        Self::with_schema(limits, workers, jig, application, full_schema)
    }

    /// Constructs readers of the full schema already prepared for this service.
    pub fn with_schema(
        limits: &Limits,
        workers: Box<[Worker]>,
        jig: List<wire::Spoken>,
        application: wire::Application,
        full_schema: skein_channel::Schema,
    ) -> Result<Self, Unusable> {
        Self::with_setup(limits, workers, WireSetup::Texts { jig, application }, full_schema)
    }

    /// As `with_schema`, using engine wire texts and roots that were checked once.
    pub fn with_prepared_wires(
        limits: &Limits,
        workers: Box<[Worker]>,
        wires: PreparedWires,
        full_schema: skein_channel::Schema,
    ) -> Result<Self, Unusable> {
        Self::with_setup(limits, workers, WireSetup::Prepared(wires), full_schema)
    }

    fn with_setup(
        limits: &Limits,
        workers: Box<[Worker]>,
        setup: WireSetup,
        full_schema: skein_channel::Schema,
    ) -> Result<Self, Unusable> {
        let channel_limits = crate::limits::checked_channel(limits, &full_schema).ok_or(Unusable::Limits)?;
        let codec = wire::sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
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
        let mut names = Map::with_capacity(limits.names);
        for worker in &workers {
            if worker.number == 0 || worker.number > limits.names {
                return Err(Unusable::Workers);
            }
            names
                .insert(worker.number.checked_sub(1).expect("positive number"), worker.name.clone())
                .or(Err(Unusable::Workers))?;
        }
        let PreparedWires { wires, plans } = match setup {
            WireSetup::Texts { jig, application } => PreparedWires::new(jig, application)?,
            WireSetup::Prepared(prepared) => prepared,
        };
        let connections = Map::with_capacity(limits.workers.checked_add(limits.opening).expect("checked limits"));
        // The restore publishes even an empty initial table through Event::Workers.
        let workers_ready = !workers.is_empty();
        Ok(Self {
            limits: *limits,
            workers,
            figures: None,
            workers_ready,
            names,
            wires,
            codec,
            connections,
            schema: full_schema,
            channel_limits,
            next_channel: 1,
            next_part: 1,
            life: Life::Running,
            dropped_down: 0,
            sending: None,
            plans,
        })
    }
    #[must_use]
    pub fn workers(&self) -> &[Worker] {
        &self.workers
    }
    /// Stable names survive withdrawal while an opening's diagnostics finish.
    #[must_use]
    pub fn worker_name(&self, position: u32) -> Option<&[u8]> {
        Some(self.names.get(&position)?)
    }

    /// The engine's channel table, prepared for these limits.
    #[must_use]
    pub const fn full_schema(&self) -> &skein_channel::Schema {
        &self.schema
    }

    /// The engine's current wires, in opening order.
    #[must_use]
    pub const fn spoken(&self) -> &List<wire::Spoken> {
        &self.wires
    }

    /// The limits used to bound a worker's kinds table.
    #[must_use]
    pub const fn codec(&self) -> &wire::v22::Limits {
        &self.codec
    }

    /// Takes timers for the next operation without moving an armed deadline
    /// (protocol/channel/figures.md, 3).
    #[must_use]
    pub fn retime(&mut self, timers: crate::Timers) -> bool {
        let limits = Limits { handshake: timers.handshake, hello: timers.hello, stall: timers.stall, ..self.limits };
        if !crate::limits::valid_with_schema(&limits, &self.schema) {
            return false;
        }
        self.limits = limits;
        true
    }

    pub fn down(&mut self, now: Time, event: Event, out: &mut Queue<Request>) {
        match event {
            Event::Figures { figures } => self.give_figures(now, figures, out),
            Event::Workers(workers) => {
                for worker in &workers {
                    self.names
                        .insert(worker.number.checked_sub(1).expect("checked worker number"), worker.name.clone())
                        .expect("checked worker table");
                }
                self.workers = workers;
                self.workers_ready = true;
                let mut waiting = List::uncapped();
                for (connection, state) in &self.connections {
                    if state.pending_opening.is_some() {
                        waiting.push(*connection).expect("uncapped waiting openings");
                    }
                }
                for connection in waiting.into_boxed() {
                    let mut state = self.connections.remove(&connection).expect("waiting opening exists");
                    self.progress(now, connection, &mut state, out);
                    self.keep(connection, state);
                }
            }
            Event::Accepted { connection, in_process } => self.accepted(now, connection, in_process, out),
            Event::Lower { connection, event } => self.lower(now, connection, event, out),
            Event::Gone { connection } => {
                if let Some(mut state) = self.connections.remove(&connection) {
                    ended(connection, &mut state, Loss::Stream, out);
                    lost(&mut state, out);
                }
            }
            Event::Restarting { wait } => self.restarting(now, wait, out),
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
            Event::PartBusy { channel, run, attempt, part } => {
                if let Some(connection) = self.target(channel) {
                    let record = crate::translate::Down::PartBusy(wire::PartBusy {
                        task: run.raw(),
                        attempt: attempt.raw(),
                        number: part,
                    });
                    self.send(now, connection, Some(record), Reply::Drop, out);
                }
            }
            Event::TurnBusy { channel, run, attempt, turn } => {
                if let Some(connection) = self.target(channel) {
                    let record = crate::translate::turn_busy(run, attempt, turn);
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
    /// The word goes only where the worker's layout holds it (protocol/channel/restart.md, 3).
    fn restarting(&mut self, now: Time, wait: Duration, out: &mut Queue<Request>) {
        let mut channels = List::uncapped();
        for (connection, state) in &self.connections {
            if state.phase == Phase::Open && state.heard {
                channels
                    .push((
                        *connection,
                        !state.omits(wire::Kind::Restarting),
                        state.worker.expect("authenticated worker"),
                    ))
                    .expect("uncapped channels");
            }
        }
        for (connection, held, worker) in channels.into_boxed() {
            let sent = if held {
                let record = wire::Restarting { wait_ms: crate::translate::wait_ms_to_record(wait) };
                self.send(now, connection, Some(crate::translate::Down::Restarting(record)), Reply::Drop, out);
                Some(wait)
            } else {
                None
            };
            let worker = self.workers.get(skein_lib::usize_of(worker)).expect("authenticated worker").name.clone();
            out.push(Request::Told { worker, version: wire::VERSION, wait: sent });
        }
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
                if let jig_domain_host::LinkDown::ReadUsage { .. } = &down {
                    let state = self.connections.get(&connection).expect("target channel exists");
                    if state.omits(wire::Kind::ReadUsage) {
                        self.dropped_down = self.dropped_down.saturating_add(1);
                        return;
                    }
                }
                let reply = match &down {
                    jig_domain_host::LinkDown::Pause { mode } => {
                        // A pause is best effort at the worker's layout (protocol/channel/pause.md, 3).
                        let state = self.connections.get(&connection).expect("heard channel exists");
                        if state.omits(wire::Kind::Pause) {
                            let worker = self
                                .workers
                                .get(skein_lib::usize_of(state.worker.expect("authenticated worker")))
                                .expect("authenticated worker")
                                .name
                                .clone();
                            out.push(Request::Unpaused { worker, version: wire::VERSION, mode: *mode });
                            return;
                        }
                        Reply::Drop
                    }
                    jig_domain_host::LinkDown::Assign { assignment } => {
                        Reply::Assignment { run: assignment.assignment.run, attempt: assignment.assignment.attempt }
                    }
                    jig_domain_host::LinkDown::AssignJob { job } => Reply::Job { run: job.run, attempt: job.attempt },
                    jig_domain_host::LinkDown::ReadList { read, .. } => Reply::ReadList { read: *read },
                    jig_domain_host::LinkDown::ReadUsage { .. }
                    | jig_domain_host::LinkDown::Sources { .. }
                    | jig_domain_host::LinkDown::Check { .. }
                    | jig_domain_host::LinkDown::SignOut { .. }
                    | jig_domain_host::LinkDown::Message { .. }
                    | jig_domain_host::LinkDown::Asked { .. }
                    | jig_domain_host::LinkDown::Relayed { .. }
                    | jig_domain_host::LinkDown::PartKept { .. }
                    | jig_domain_host::LinkDown::TurnKept { .. }
                    | jig_domain_host::LinkDown::Acknowledge { .. }
                    | jig_domain_host::LinkDown::Cancel { .. }
                    | jig_domain_host::LinkDown::Grant { .. } => Reply::Drop,
                };
                let record = match crate::translate::down(down, &parts, values, &self.codec) {
                    Ok(record) => Some(record),
                    Err(reason) => {
                        self.encoding_refused(channel, reply, reason, out);
                        return;
                    }
                };
                self.send(now, connection, record, reply, out);
            } else {
                self.dropped_down = self.dropped_down.saturating_add(1);
            }
        }
    }
    fn encoding_refused(&mut self, channel: Token, reply: Reply, reason: crate::Unencoded, out: &mut Queue<Request>) {
        let what: &[u8] = match reply {
            Reply::Assignment { .. } => b"assignment",
            Reply::Job { .. } => b"job",
            Reply::ReadList { .. } | Reply::Drop => b"record",
        };
        let words = reason.words(what);
        self.refused_record(channel, reply, &words, false, out);
        self.dropped_down = self.dropped_down.saturating_add(1);
    }
    fn completed(&mut self, out: &mut Queue<Request>) {
        if (self.life == Life::Closing || self.life == Life::Aborting) && self.connections.is_empty() {
            self.life = Life::Closed;
            out.push(Request::Closed);
        }
    }
    fn target(&self, channel: Token) -> Option<Token> {
        for (connection, state) in &self.connections {
            if state.channel == channel && state.heard && state.phase == Phase::Open {
                return Some(*connection);
            }
        }
        None
    }
    fn refused_record(&self, channel: Token, reply: Reply, words: &[u8], untranslated: bool, out: &mut Queue<Request>) {
        use jig_domain_host as host;
        let up = match reply {
            Reply::Assignment { run, attempt } => {
                let reason = if untranslated {
                    host::Refusal::Untranslated(Box::from(words))
                } else {
                    host::Refusal::Invalid(host::Invalid::Start {
                        cause: host::StartCause::Other,
                        why: self.detail(words),
                    })
                };
                host::LinkUp::Answer {
                    run,
                    attempt,
                    answer: host::RunAnswer {
                        read: None,
                        refused: Box::new([]),
                        turns: 0,
                        spent: 0,
                        ending: host::Ending::Refused(reason),
                    },
                }
            }
            Reply::Job { run, attempt } => {
                let reason = if untranslated {
                    host::JobRefusal::Untranslated(Box::from(words))
                } else {
                    host::JobRefusal::Invalid(host::JobInvalid::Start { why: self.detail(words) })
                };
                host::LinkUp::JobAnswer { run, attempt, answer: host::JobEnding::Refused(reason) }
            }
            Reply::ReadList { read } => host::LinkUp::ListRead {
                read,
                outcome: jig_domain_link::ListOutcome::Failed(jig_domain_link::ListFailure::Agent),
            },
            Reply::Drop => return,
        };
        out.push(Request::Up { channel, up, parts: Box::new([]) });
    }
    fn detail(&self, words: &[u8]) -> Box<[u8]> {
        Box::from(bytes::utf8_prefix(words, skein_lib::length::usize_of(self.limits.sizes.detail)))
    }
    fn unsent(&mut self, now: Time, state: &mut Connection, sending: Sending, bound: Bound, out: &mut Queue<Request>) {
        let untranslated = match &bound {
            Bound::Translation(_) => true,
            Bound::Room(_) | Bound::Channel(_) => false,
        };
        let words = match sending.reply {
            Reply::Assignment { .. } => unsent_words(b"assignment", sending.bytes, &bound),
            Reply::Job { .. } => unsent_words(b"job", sending.bytes, &bound),
            Reply::ReadList { .. } | Reply::Drop => Box::new([]),
        };
        out.push(Request::Unsent {
            worker: state.worker.expect("a record sent on an authenticated channel"),
            channel: state.channel,
            unsent: Unsent { kind: sending.kind, bytes: sending.bytes, bound },
        });
        self.refused_record(state.channel, sending.reply, &words, untranslated, out);
        match sending.reply {
            Reply::Drop => self.dropped_down = self.dropped_down.saturating_add(1),
            Reply::Assignment { .. } | Reply::Job { .. } | Reply::ReadList { .. } => {}
        }
        state.ping = Some(now.saturating_add(self.limits.ping));
    }
    /// Fit the frame to the peer's layout before measuring what the channel will send.
    fn converted_frame(
        &mut self,
        now: Time,
        state: &mut Connection,
        frame: Frame,
        kind: wire::Kind,
        reply: Reply,
        out: &mut Queue<Request>,
    ) -> Option<Frame> {
        let Some(peer) = &mut state.peer else { return Some(frame) };
        let bytes = frame.wire_len();
        match convert::send(peer, frame, &self.codec) {
            Ok(frame) => Some(frame),
            Err(problem) => {
                self.unsent(now, state, Sending { kind, bytes, reply }, Bound::Translation(problem), out);
                None
            }
        }
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
        let kind = record.kind();
        if kind.optional_at_worker() {
            let state = self.connections.get(&connection).expect("target channel exists");
            if state.sends(kind).is_none() {
                return;
            }
        }
        // Fit the translated record before encoding the frame the room measures
        // (jig/docs/design/protocol/channel/liveness.md, 1, and jig/docs/design/protocol/channel/README.md, 10).
        let (record, past) = match record {
            crate::translate::Down::Assign(record) => {
                let (record, fitted) = match jig_protocol_worker_wire::fit::fit(record, &self.codec, self.limits.room) {
                    Ok(fitted) => fitted,
                    Err(why) => {
                        let channel = self.connections.get(&connection).expect("heard channel exists").channel;
                        self.encoding_refused(channel, reply, crate::translate::problem(why), out);
                        return;
                    }
                };
                let past = match fitted {
                    jig_protocol_worker_wire::fit::Fitted::Whole
                    | jig_protocol_worker_wire::fit::Fitted::Cut { .. } => None,
                    jig_protocol_worker_wire::fit::Fitted::Past { bytes } => Some(bytes),
                };
                (crate::translate::Down::Assign(record), past)
            }
            crate::translate::Down::Pause(_)
            | crate::translate::Down::Restarting(_)
            | crate::translate::Down::Asked(_)
            | crate::translate::Down::ReadUsage(_)
            | crate::translate::Down::ReadList(_)
            | crate::translate::Down::Sources(_)
            | crate::translate::Down::Check(_)
            | crate::translate::Down::SignOut(_)
            | crate::translate::Down::AssignJob(_)
            | crate::translate::Down::Message(_)
            | crate::translate::Down::Relayed(_)
            | crate::translate::Down::PartKept(_)
            | crate::translate::Down::PartBusy(_)
            | crate::translate::Down::TurnKept(_)
            | crate::translate::Down::TurnBusy(_)
            | crate::translate::Down::Acknowledge(_)
            | crate::translate::Down::Cancel(_)
            | crate::translate::Down::Grant(_) => (record, None),
        };
        let kind = record.kind();
        let mut state = self.connections.remove(&connection).expect("heard channel exists");
        let frame = if past.is_none() {
            let frame = match record.frame(&self.codec) {
                Ok(frame) => frame,
                Err(reason) => {
                    self.encoding_refused(state.channel, reply, reason, out);
                    self.keep(connection, state);
                    return;
                }
            };
            let Some(frame) = self.converted_frame(now, &mut state, frame, kind, reply, out) else {
                self.keep(connection, state);
                return;
            };
            Some(frame)
        } else {
            None
        };
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
    /// Whole owner input and each ready channel (protocol/channel/figures.md, 3).
    fn give_figures(&mut self, now: Time, figures: wire::Figures, out: &mut Queue<Request>) {
        self.figures = Some(figures);
        let mut channels = List::uncapped();
        for (token, state) in &self.connections {
            if state.phase == Phase::Hello || state.phase == Phase::Open {
                channels.push(*token).expect("uncapped channels");
            }
        }
        for connection in &channels {
            let mut state = self.connections.remove(connection).expect("ready channel");
            self.send_figures(now, *connection, &mut state, out);
            self.progress(now, *connection, &mut state, out);
            self.keep(*connection, state);
        }
    }
    fn send_figures(&mut self, now: Time, connection: Token, state: &mut Connection, out: &mut Queue<Request>) -> bool {
        if state.omits(wire::Kind::Figures) {
            return true;
        }
        let Some(figures) = &self.figures else { return true };
        let mut parts = figures.clone();
        let mut admitted = None;
        for worker in &self.workers {
            if worker.number.checked_sub(1) == state.worker {
                admitted = Some(worker);
                break;
            }
        }
        let Some(worker) = admitted else {
            self.refuse(now, connection, state, Reason::NotAdmitted.code(), Box::new([]), out);
            return false;
        };
        parts.places = worker.places;
        let figures = parts;
        let length = figures.measure(&wire::CEILINGS).expect("composed figures fit limits");
        let mut body = skein_lib::Writer::new(skein_lib::usize_of(length));
        figures.write(&wire::CEILINGS, &mut body).expect("measured figures");
        let mut frame = skein_channel::frame_writer(wire::Kind::Figures.number(), length).expect("bounded figures");
        frame.put(&body.finish()).expect("measured figures");
        let frame = frame.finish().expect("measured figures");
        let Some(frame) = self.converted_frame(now, state, frame, wire::Kind::Figures, Reply::Drop, out) else {
            if state.phase == Phase::Terms {
                self.refuse(now, connection, state, 3, Box::new([]), out);
                return false;
            }
            return true;
        };
        if frame.wire_len() > self.limits.room {
            let sending = Sending { kind: wire::Kind::Figures, bytes: frame.wire_len(), reply: Reply::Drop };
            self.unsent(now, state, sending, Bound::Room(self.limits.room), out);
            self.loss(now, connection, state, Loss::Full, out);
            return false;
        }
        self.send_opening(now, connection, state, wire::Kind::Figures, frame, out);
        state.phase != Phase::Closing
    }
    fn send_opening(
        &mut self,
        now: Time,
        connection: Token,
        state: &mut Connection,
        kind: wire::Kind,
        frame: Frame,
        out: &mut Queue<Request>,
    ) {
        self.sending = Some(Sending { kind, bytes: frame.wire_len(), reply: Reply::Drop });
        state.down(ChannelRequest::Send { token: state.channel, frame }, connection, out);
        for _ in 0..skein_channel::MAX_UP {
            let Some(event) = state.above.pop() else { break };
            self.heard(now, connection, state, event, out);
        }
        assert!(self.sending.is_none(), "send answered within its entrance");
    }
    fn accepted(&mut self, now: Time, connection: Token, in_process: bool, out: &mut Queue<Request>) {
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
        let mut state =
            Connection::new(self.schema.clone(), self.channel_limits, self.limits.handshake, now, in_process);
        out.push(Request::Take { connection });
        self.progress(now, connection, &mut state, out);
        self.keep(connection, state);
    }
    fn keep(&mut self, connection: Token, state: Connection) {
        let replaced = self.connections.insert(connection, state).expect("an accepted connection has a slot");
        assert!(replaced.is_none(), "one state per connection");
    }
    fn progress(&mut self, now: Time, connection: Token, state: &mut Connection, out: &mut Queue<Request>) {
        // The settings restore resumes an opening, never extending its handshake deadline
        // (domain/engine/workers.md, 4).
        if self.workers_ready
            && state.phase == Phase::Opening
            && !expired(state.deadline, now)
            && let Some(opening) = state.pending_opening.take()
        {
            self.heard(now, connection, state, opening, out);
        }
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
        reason = "one exhaustive event match keeps the opening and channel lifecycle in order"
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
            ChannelEvent::Opening { credential } => {
                if self.life != Life::Running {
                    self.refuse(now, connection, state, Reason::Stopping.code(), Box::new([]), out);
                    return;
                }
                if !self.workers_ready {
                    state.pending_opening = Some(ChannelEvent::Opening { credential });
                    return;
                }
                let verdict = self.judge(&credential, state.in_process);
                let (worker, opening, older, fitted, statement) = match verdict {
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
                    Verdict::Accept { worker, opening, older, fitted, statement } => {
                        (worker, opening, older, fitted, statement)
                    }
                };
                if let Some(older) = older
                    && let Some(mut old) = self.connections.remove(&older)
                {
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
                let Fitted { peer, table, converted } = *fitted;
                state.opening = opening;
                state.peer = Some(Box::new(peer));
                state.table = Some(table.clone());
                state.converted = converted;
                state.statement = Some(statement);
                state.worker = Some(worker);
                state.channel = Token::new(self.next_channel);
                self.next_channel =
                    self.next_channel.checked_add(1).expect("channel tokens never wrap in a process life");
                state.phase = Phase::Terms;
                state.hello_deadline = Some(now.saturating_add(self.limits.hello));
                state.down(ChannelRequest::AcceptWith { table }, connection, out);
            }
            ChannelEvent::Ready { terms } => {
                let table = state.table.take().expect("the opening installed a table");
                let check = if state.opening == wire::Opening::Figures {
                    figures_terms(&table, state.sends(wire::Kind::Figures), &terms)
                } else {
                    statement::terms(&table, &terms)
                };
                if let Err(text) = check {
                    self.refuse(now, connection, state, 2, text, out);
                } else {
                    if !self.send_figures(now, connection, state, out) {
                        return;
                    }
                    if state.opening == wire::Opening::Figures {
                        out.push(Request::FiguresTaken { worker: state.worker.expect("authenticated worker") });
                        state.phase = Phase::Closing;
                        state.ending = Ending::Graceful;
                        state.deadline = Some(now.saturating_add(self.limits.stall));
                        state.down(ChannelRequest::Finish, connection, out);
                        return;
                    }
                    state.statement.as_mut().expect("authenticated statement").terms = terms;
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
        if let Some(peer) = &mut state.peer
            && !peer.same()
            && (state.phase == Phase::Hello || state.phase == Phase::Open)
        {
            let frame = match convert::receive(peer, kind, body, &self.codec) {
                Ok(frame) => frame,
                Err(problem) => {
                    let worker = state.worker.expect("a body from an authenticated channel");
                    let kind = convert::received_kind(peer, kind).expect("the admitted table maps this worker kind");
                    out.push(Request::Unread { worker, kind, problem });
                    self.refuse(now, connection, state, 3, Box::new([]), out);
                    return;
                }
            };
            self.decoded_body(now, connection, state, frame.kind(), frame.bytes().get(8..).expect("frame header"), out);
        } else {
            self.decoded_body(now, connection, state, kind, body, out);
        }
    }
    fn decoded_body(
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
                    let Some(mut hello) = crate::translate::hello(record) else {
                        self.refuse(now, connection, state, 3, Box::new([]), out);
                        return;
                    };
                    hello.reads_usage = state.sends(wire::Kind::ReadUsage).is_some();
                    state.phase = Phase::Open;
                    state.heard = true;
                    state.deadline = None;
                    state.ping = Some(now.saturating_add(self.limits.ping));
                    state.silence = Some(now.saturating_add(self.limits.silence));
                    let mut statement = state.statement.take().expect("authenticated statement");
                    statement.jobs = List::from_boxed(hello.jobs.clone());
                    let converted = core::mem::replace(&mut state.converted, List::with_capacity(0));
                    out.push(Request::Opened { worker, converted, statement });
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

fn figures_terms(
    table: &List<skein_channel::Kind>,
    kind: Option<u16>,
    terms: &List<skein_channel::Term>,
) -> Result<(), Box<[u8]>> {
    let Some(kind) = kind else { return Err(statement::kind_words(wire::Kind::Figures.number())) };
    for rule in table {
        if rule.kind == kind {
            for term in terms {
                if term.kind == kind && term.largest >= rule.largest {
                    return Ok(());
                }
            }
            return Err(statement::kind_words(kind));
        }
    }
    unreachable!("the worker's figures kind is in its table")
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

fn lose(now: Time, stall: Duration, state: &mut Connection, out: &mut Queue<Request>) {
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

/// ASCII diagnostics for an unsent start (jig/docs/design/protocol/channel/records.md, 2, and jig/docs/design/protocol/channel/liveness.md, 1).
fn unsent_words(name: &[u8], bytes: u32, bound: &Bound) -> Box<[u8]> {
    let count = Decimal::of(u64::from(bytes));
    let room = match bound {
        Bound::Room(room) => *room,
        Bound::Channel(_) => 0,
        Bound::Translation(problem) => return problem.kept(),
    };
    let room = Decimal::of(u64::from(room));
    let (middle, value, end): (&[u8], &[u8], &[u8]) = match bound {
        Bound::Translation(_) => unreachable!("translation already has its refusal words"),
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
