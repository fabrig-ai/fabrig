//! Generic channel's opening machine (net/channel.md, sections 5, 6 and 9).
//! It keeps the checked schema, opening phase, one partly filled frame, peer
//! terms, a bounded output queue and one independent output right. It never
//! knows credential policy, body meaning, deadlines or transport type.
//! `new`, `down`, `up` and `poll` are its entry points; the owner reserves
//! `MAX_UP` and `MAX_DOWN` slots first.
//!
//! | State | Event | Next state, emits |
//! |---|---|---|
//! | WaitOpen | Open frame | WaitOwner, Opening; invalid offer: Refusing |
//! | WaitOwner | Accept/Refuse | WaitTerms/Refusing, Accept and Terms/Refuse |
//! | Idle | Open request | WaitAccept, Open |
//! | WaitAccept | Accept/Refuse | WaitTerms/Refusing, Terms/Refused |
//! | WaitTerms | Terms | Ready when own Terms handed down; Ready |
//! | any active | Ping | same, Ping |
//! | any active | malformed frame | Refusing, Refuse |
//! | Refusing | output drained | Closing, Finish |
//! | any active | Close/failure | Closing, cancel or drain output |
//! | Closing | output settled | Closed, Closed |
use alloc::boxed::Box;
use core::mem;
use skein_lib::{List, Queue, Token, bytes, stream, usize_of};

use super::boundary::{Closed, Event, Lower, LowerEvent, ReadWait, Request, Room, Unsent, Waiting, WriteWait};
use super::frame::{Control, Frame, Header, Term, control_frame, decode_control, parse_header};
use super::opening::{accept_open, check_terms, local_terms, offers_version};
use super::read::{ReadKind, classify};
use super::schema::{Limits, Role, Schema, SchemaError};
use super::write::admit;
use Phase::{Finished, Ready, Refusing, WaitAccept, WaitOpen, WaitOwner, WaitTerms};

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    WaitOpen,
    WaitOwner { lowest: u16, highest: u16 },
    WaitAccept,
    WaitTerms { version: u16, own: OwnTerms, peer: Option<List<Term>> },
    Ready { version: u16, peer: List<Term> },
    Refusing { why: Closed, finish: Finish },
    Finished { version: u16, finish: Finish },
    Closing { why: Closed },
    Closed,
}

impl Phase {
    fn stopped(&self) -> bool {
        match self {
            Self::Closing { .. } | Self::Closed => true,
            Self::Idle
            | Self::WaitOpen
            | Self::WaitOwner { .. }
            | Self::WaitAccept
            | Self::WaitTerms { .. }
            | Self::Ready { .. }
            | Self::Finished { .. }
            | Self::Refusing { .. } => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum OwnTerms {
    Due,
    Sent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Finish {
    Due,
    Sent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demand {
    None,
    Out,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Output {
    Free,
    Asked { right: Token },
    Granted { right: Token },
    CancelDue { right: Token },
    Cancelling { right: Token },
    ReleaseDue { right: Token },
    Failed { owed: Option<Token> },
    FailedGranted { right: Token },
}

impl Output {
    fn token(self) -> Option<Token> {
        match self {
            Self::Free | Self::Failed { owed: None } => None,
            Self::Asked { right }
            | Self::Granted { right }
            | Self::CancelDue { right }
            | Self::Cancelling { right }
            | Self::ReleaseDue { right }
            | Self::Failed { owed: Some(right) }
            | Self::FailedGranted { right } => Some(right),
        }
    }

    fn failed(self) -> bool {
        match self {
            Self::Failed { .. } | Self::FailedGranted { .. } => true,
            Self::Free
            | Self::Asked { .. }
            | Self::Granted { .. }
            | Self::CancelDue { .. }
            | Self::Cancelling { .. }
            | Self::ReleaseDue { .. } => false,
        }
    }

    fn cancel(self) -> Self {
        match self {
            Self::Asked { right } | Self::Failed { owed: Some(right) } => Self::CancelDue { right },
            Self::Granted { right } | Self::FailedGranted { right } => Self::ReleaseDue { right },
            Self::Free
            | Self::CancelDue { .. }
            | Self::Cancelling { .. }
            | Self::ReleaseDue { .. }
            | Self::Failed { owed: None } => self,
        }
    }
}

#[derive(Clone, Copy)]
enum OutputFate {
    Drop,
    Keep,
}

#[derive(PartialEq, Eq)]
enum ReadState {
    Header { first: Option<u8> },
    Body { header: Header, body: Box<[u8]>, filled: u32 },
    Skip { kind: u16, remaining: u32 },
    Skipped { kind: u16 },
    Ended,
    Idle,
}

// skein/docs/design/net/channel.md, 3 and 6: a body being read prints its length and progress.
impl core::fmt::Debug for ReadState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Header { first } => f.debug_struct("Header").field("first", first).finish(),
            Self::Body { header, body, filled } => f
                .debug_struct("Body")
                .field("header", header)
                .field("body_len", &body.len())
                .field("filled", filled)
                .finish(),
            Self::Skip { kind, remaining } => {
                f.debug_struct("Skip").field("kind", kind).field("remaining", remaining).finish()
            }
            Self::Skipped { kind } => f.debug_struct("Skipped").field("kind", kind).finish(),
            Self::Ended => f.write_str("Ended"),
            Self::Idle => f.write_str("Idle"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Queued {
    frame: Frame,
    tag: QueueTag,
}

#[derive(Debug, PartialEq, Eq)]
enum QueueTag {
    Control,
    Terms,
    Ping,
    Unsupported,
    Application,
}

/// Whether read and write share one stream or use a pipe in each direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamMode {
    /// Both directions use one socket-like stream.
    One,
    /// The machine reads one pipe and writes the other.
    Two,
}

/// One channel machine over a read stream and independent output stream.
#[derive(Debug)]
pub struct Machine {
    schema: Schema,
    role: Role,
    mode: StreamMode,
    limits: Limits,
    phase: Phase,
    read: ReadState,
    demand: Demand,
    output: Queue<Queued>,
    right: Output,
    next_right: u64,
    used_bytes: u32,
    used_frames: u32,
    terminal_text: Box<[u8]>,
}

impl Machine {
    /// Checks the schema before keeping its bounded tables.
    pub fn new(schema: Schema, role: Role, limits: Limits) -> Result<Machine, SchemaError> {
        Self::with_mode(schema, role, limits, StreamMode::One)
    }

    /// Checks a channel with an explicit one-stream or two-pipe topology.
    pub fn with_mode(schema: Schema, role: Role, limits: Limits, mode: StreamMode) -> Result<Machine, SchemaError> {
        schema.check(role, &limits)?;
        let output_capacity = limits.output_frames.checked_add(3).ok_or(SchemaError::InvalidLimit)?;
        let (phase, read) = match role {
            Role::Initiator => (Phase::Idle, ReadState::Idle),
            Role::Responder => (WaitOpen, ReadState::Header { first: None }),
        };
        Ok(Machine {
            schema,
            role,
            mode,
            limits,
            phase,
            read,
            demand: Demand::None,
            output: Queue::with_capacity(output_capacity),
            right: Output::Free,
            next_right: 1,
            used_bytes: 0,
            used_frames: 0,
            terminal_text: Box::new([]),
        })
    }

    /// Handles one owner request; `poll` afterwards emits the next demands.
    pub fn down(&mut self, request: Request, up: &mut Queue<Event>, below: &mut Queue<Lower>) {
        match request {
            Request::Open { credential } => self.open(credential),
            Request::Accept { version } => self.accept(version),
            Request::Refuse { reason, text } => self.refuse(reason, text, Closed::RefusedHere(reason)),
            Request::Read => self.read(up),
            Request::Send { token, frame } => self.send(token, frame, up),
            Request::Ping => self.ping(),
            Request::Finish => self.finish(),
            Request::Close => self.close(up, below),
        }
        self.maybe_terminal(up);
    }

    /// Handles exactly one lower event; `poll` afterwards progresses both sides.
    pub fn up(&mut self, event: LowerEvent, above: &mut Queue<Event>, _below: &mut Queue<Lower>) {
        match event {
            LowerEvent::Read(event) => self.read_event(event, above),
            LowerEvent::Write(event) => self.write_event(event, above),
            LowerEvent::WriteFailed(_) => self.output_failure(above),
        }
        self.maybe_terminal(above);
    }

    /// Emits the current read demand and a separate output reservation or send.
    pub fn poll(&mut self, above: &mut Queue<Event>, below: &mut Queue<Lower>) {
        self.flush_unsupported();
        self.cancel_output(below);
        if self.demand == Demand::Out && self.read == ReadState::Ended {
            below.push(Lower::Read(stream::Down::Demand { read: stream::Read::Nothing, room: 0 }));
            self.demand = Demand::None;
        }
        self.emit_read(below);
        self.emit_output(above, below);
        self.maybe_ready(above);
        self.maybe_finish(below);
        self.maybe_terminal(above);
    }

    /// The owner-visible waits for deadlines.
    #[must_use]
    pub fn waiting(&self) -> Waiting {
        let read = match self.phase {
            WaitOpen | WaitAccept | WaitTerms { .. } => ReadWait::Opening,
            Ready { .. } | Finished { .. } if self.read_credit() => ReadWait::Frame,
            Phase::Idle
            | WaitOwner { .. }
            | Finished { .. }
            | Phase::Closing { .. }
            | Phase::Closed
            | Ready { .. }
            | Refusing { .. } => ReadWait::Nothing,
        };
        let write =
            if self.output.is_empty() && self.right.token().is_none() { WriteWait::Nothing } else { WriteWait::Frames };
        Waiting { read, write }
    }

    /// The selected version after the opening, if ready.
    #[must_use]
    pub fn version(&self) -> Option<u16> {
        match self.phase {
            Ready { version, .. } | Finished { version, .. } => Some(version),
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Refusing { .. }
            | Phase::Closing { .. }
            | Phase::Closed => None,
        }
    }

    /// Application output room after all admitted frames are counted.
    #[must_use]
    pub fn room(&self) -> Room {
        Room {
            bytes: self.limits.output_bytes.checked_sub(self.used_bytes).expect("queued bytes fit the cap"),
            frames: self.limits.output_frames.checked_sub(self.used_frames).expect("queued frames fit the cap"),
        }
    }

    fn open(&mut self, credential: Box<[u8]>) {
        if self.role != Role::Initiator || self.phase != Phase::Idle {
            return;
        }
        let (lowest, highest) = self.schema.range().expect("validated versions");
        let control = Control::Open { magic: self.schema.magic, lowest, highest, features: 0, credential };
        match control_frame(&control, &self.limits) {
            Ok(frame) => {
                self.queue(frame, QueueTag::Control);
                self.phase = WaitAccept;
                self.read = ReadState::Header { first: None };
            }
            Err(_) => self.phase = Phase::Closed,
        }
    }

    fn accept(&mut self, version: u16) {
        let allowed = match self.phase {
            WaitOwner { lowest, highest } => version >= lowest && version <= highest,
            Phase::Idle
            | WaitOpen
            | WaitAccept
            | WaitTerms { .. }
            | Ready { .. }
            | Refusing { .. }
            | Finished { .. }
            | Phase::Closing { .. }
            | Phase::Closed => false,
        };
        if !allowed {
            return;
        }
        let answer = Control::Accept { version, features: 0 };
        let frame = control_frame(&answer, &self.limits).expect("fixed Accept fits");
        self.queue(frame, QueueTag::Control);
        let terms = local_terms(&self.schema, self.role, version, &self.limits);
        let frame = control_frame(&Control::Terms { entries: terms }, &self.limits).expect("checked Terms fit");
        self.queue(frame, QueueTag::Terms);
        self.phase = WaitTerms { version, own: OwnTerms::Due, peer: None };
        self.read = ReadState::Header { first: None };
    }

    fn refuse(&mut self, reason: u16, text: Box<[u8]>, why: Closed) {
        let finish = match self.phase {
            Finished { finish, .. } => finish,
            Phase::Closing { .. } | Phase::Closed | Refusing { .. } => return,
            Phase::Idle | WaitOpen | WaitOwner { .. } | WaitAccept | WaitTerms { .. } | Ready { .. } => Finish::Due,
        };
        if self.right.failed() {
            self.closing(Closed::Stream, OutputFate::Keep);
            if why == Closed::Framing {
                self.phase = Refusing { why: Closed::Stream, finish };
            }
            return;
        }
        assert!(text.len() <= 256, "refusal text is at most 256 bytes");
        self.clear_output();
        self.terminal_text.clone_from(&text);
        let control = Control::Refuse { reason, text };
        let frame = control_frame(&control, &self.limits).expect("bounded Refuse fits");
        self.queue(frame, QueueTag::Control);
        self.phase = Refusing { why, finish };
        self.read = ReadState::Idle;
        self.right = self.right.cancel();
    }

    fn read(&mut self, above: &mut Queue<Event>) {
        let may_read = match self.phase {
            Ready { .. } => true,
            Finished { .. } => self.mode == StreamMode::Two,
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Refusing { .. }
            | Phase::Closing { .. }
            | Phase::Closed => false,
        };
        if !may_read || self.read_credit() {
            return;
        }
        match self.read {
            ReadState::Ended => above.push(Event::Ended),
            ReadState::Idle => {
                self.read = ReadState::Header { first: None };
            }
            ReadState::Header { .. } | ReadState::Body { .. } | ReadState::Skip { .. } | ReadState::Skipped { .. } => {
                unreachable!("one owner Read installs only one frame demand")
            }
        }
    }

    fn ping(&mut self) {
        let may_send = match self.phase {
            WaitOwner { .. } | WaitAccept | WaitTerms { .. } | Ready { .. } => true,
            Phase::Idle | WaitOpen | Refusing { .. } | Finished { .. } | Phase::Closing { .. } | Phase::Closed => false,
        };
        if may_send && !self.queued(QueueTag::Ping) && !self.right.failed() {
            let frame = control_frame(&Control::Ping, &self.limits).expect("fixed Ping fits");
            self.queue(frame, QueueTag::Ping);
        }
    }

    fn send(&mut self, token: Token, frame: Frame, above: &mut Queue<Event>) {
        let (version, terms) = match self.phase {
            Ready { version, ref peer } if !self.right.failed() => (version, peer),
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Refusing { .. }
            | Ready { .. }
            | Finished { .. }
            | Phase::Closing { .. }
            | Phase::Closed => {
                above.push(Event::Unsent { token, why: Unsent::Closed });
                return;
            }
        };
        match admit(&self.schema, self.role, version, terms, self.room(), &frame) {
            Ok(()) => {
                self.used_bytes = self.used_bytes.checked_add(frame.wire_len()).expect("admission checked byte room");
                self.used_frames = self.used_frames.checked_add(1).expect("admission checked frame room");
                self.queue(frame, QueueTag::Application);
                above.push(Event::Sent { token });
            }
            Err(why) => above.push(Event::Unsent { token, why }),
        }
    }

    fn finish(&mut self) {
        match self.phase {
            Ready { version, .. } => self.phase = Finished { version, finish: Finish::Due },
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Refusing { .. }
            | Finished { .. }
            | Phase::Closing { .. }
            | Phase::Closed => {}
        }
    }

    fn close(&mut self, above: &mut Queue<Event>, below: &mut Queue<Lower>) {
        if self.phase.stopped() {
            return;
        }
        self.closing(Closed::Owner, OutputFate::Drop);
        if self.demand == Demand::Out {
            below.push(Lower::Read(stream::Down::Demand { read: stream::Read::Nothing, room: 0 }));
            self.demand = Demand::None;
        }
        self.cancel_output(below);
        self.maybe_terminal(above);
    }

    /// Active to closing (net/channel.md, 8.3), preserving an input end already seen.
    fn closing(&mut self, why: Closed, output: OutputFate) {
        self.phase = Phase::Closing { why };
        if self.read != ReadState::Ended {
            self.read = ReadState::Idle;
        }
        match output {
            OutputFate::Drop => {
                self.clear_output();
                self.right = self.right.cancel();
            }
            OutputFate::Keep => {}
        }
    }

    fn clear_output(&mut self) {
        self.output.clear();
        self.used_bytes = 0;
        self.used_frames = 0;
        match self.read {
            ReadState::Skipped { .. } => self.read = ReadState::Idle,
            ReadState::Header { .. }
            | ReadState::Body { .. }
            | ReadState::Skip { .. }
            | ReadState::Ended
            | ReadState::Idle => {}
        }
    }

    fn cancel_output(&mut self, below: &mut Queue<Lower>) {
        match self.right {
            Output::ReleaseDue { right } => {
                below.push(Lower::Write(stream::OutputDown::Release { right }));
                self.right = Output::Free;
            }
            Output::CancelDue { right } => {
                below.push(Lower::Write(stream::OutputDown::Cancel { right }));
                self.right = Output::Cancelling { right };
            }
            Output::Free
            | Output::Asked { .. }
            | Output::Granted { .. }
            | Output::Cancelling { .. }
            | Output::Failed { .. }
            | Output::FailedGranted { .. } => {}
        }
    }

    fn maybe_terminal(&mut self, above: &mut Queue<Event>) {
        if !self.output.is_empty() || self.right.token().is_some() {
            return;
        }
        match self.phase {
            Phase::Closing { why } | Refusing { why: why @ Closed::Stream, .. } => {
                self.phase = Phase::Closed;
                above.push(Event::Closed { why, text: mem::replace(&mut self.terminal_text, Box::new([])) });
            }
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Ready { .. }
            | Finished { .. }
            | Refusing { .. }
            | Phase::Closed => {}
        }
    }

    fn read_credit(&self) -> bool {
        self.version().is_some()
            && match self.read {
                ReadState::Header { .. }
                | ReadState::Body { .. }
                | ReadState::Skip { .. }
                | ReadState::Skipped { .. } => true,
                ReadState::Ended | ReadState::Idle => false,
            }
    }

    fn queued(&self, tag: QueueTag) -> bool {
        for item in &self.output {
            if item.tag == tag {
                return true;
            }
        }
        false
    }

    fn queue(&mut self, frame: Frame, tag: QueueTag) {
        self.output.try_push(Queued { frame, tag }).expect("control reserve has room");
    }

    fn emit_read(&mut self, below: &mut Queue<Lower>) {
        if self.demand == Demand::Out {
            return;
        }
        let demand = match &self.read {
            ReadState::Header { first: Some(_) } => Some(7),
            ReadState::Header { first: None } => Some(1),
            ReadState::Body { header, filled, .. } => {
                let left = header.body_len.checked_sub(*filled).expect("filled body does not pass length");
                Some(left.min(self.limits.chunk))
            }
            ReadState::Skip { remaining, .. } => Some((*remaining).min(self.limits.chunk)),
            ReadState::Skipped { .. } | ReadState::Ended | ReadState::Idle => None,
        };
        match demand {
            Some(bytes) if bytes > 0 => {
                below.push(Lower::Read(stream::Down::Demand { read: stream::Read::Fill(bytes), room: 0 }));
                self.demand = Demand::Out;
            }
            Some(_) | None => {}
        }
    }

    fn emit_output(&mut self, above: &mut Queue<Event>, below: &mut Queue<Lower>) {
        match self.right {
            Output::Granted { right } => {
                let item = self.output.pop().expect("grant has a queued frame");
                match item.tag {
                    QueueTag::Control | QueueTag::Ping | QueueTag::Unsupported => {}
                    QueueTag::Terms => match &mut self.phase {
                        WaitTerms { own, .. } => *own = OwnTerms::Sent,
                        Phase::Idle
                        | WaitOpen
                        | WaitOwner { .. }
                        | WaitAccept
                        | Ready { .. }
                        | Finished { .. }
                        | Refusing { .. }
                        | Phase::Closing { .. }
                        | Phase::Closed => {}
                    },
                    QueueTag::Application => {
                        self.used_bytes =
                            self.used_bytes.checked_sub(item.frame.wire_len()).expect("queued bytes were counted");
                        self.used_frames = self.used_frames.checked_sub(1).expect("queued frame was counted");
                    }
                }
                below.push(Lower::Write(stream::OutputDown::Send { right, bytes: item.frame.into_bytes() }));
                self.right = Output::Free;
                above.push(Event::Drained);
            }
            Output::Free => {
                if self.output.is_empty() {
                    return;
                }
                let right = Token::new(self.next_right);
                self.next_right = self.next_right.checked_add(1).expect("bounded output rights do not exhaust");
                let item = self.output.iter().next().expect("nonempty output");
                below.push(Lower::Write(stream::OutputDown::Room { right, bytes: item.frame.wire_len() }));
                self.right = Output::Asked { right };
            }
            Output::Asked { .. }
            | Output::CancelDue { .. }
            | Output::Cancelling { .. }
            | Output::ReleaseDue { .. }
            | Output::Failed { .. }
            | Output::FailedGranted { .. } => {}
        }
    }

    fn write_event(&mut self, event: stream::OutputUp, above: &mut Queue<Event>) {
        let stream::OutputUp::Settled { right, outcome } = event;
        if self.right.token() != Some(right) {
            return;
        }
        match outcome {
            stream::OutputOutcome::Granted => {
                self.right = match self.right {
                    Output::Asked { .. } | Output::Granted { .. } => Output::Granted { right },
                    Output::CancelDue { .. } | Output::Cancelling { .. } | Output::ReleaseDue { .. } => {
                        Output::ReleaseDue { right }
                    }
                    Output::Failed { .. } | Output::FailedGranted { .. } => Output::FailedGranted { right },
                    Output::Free => unreachable!("settlement names the held right"),
                }
            }
            stream::OutputOutcome::Cancelled => {
                let prior = mem::replace(&mut self.right, Output::Free);
                match prior {
                    Output::CancelDue { .. } | Output::Cancelling { .. } | Output::ReleaseDue { .. } => {}
                    Output::Failed { .. } | Output::FailedGranted { .. } => self.right = Output::Failed { owed: None },
                    Output::Asked { .. } | Output::Granted { .. } => self.output_failure(above),
                    Output::Free => unreachable!("settlement names the held right"),
                }
            }
            stream::OutputOutcome::Failed(_) => {
                self.right = if self.right.failed() { Output::Failed { owed: None } } else { Output::Free };
                self.output_failure(above);
            }
        }
    }

    fn output_failure(&mut self, above: &mut Queue<Event>) {
        if self.right.failed() || self.phase == Phase::Closed {
            return;
        }
        self.clear_output();
        self.right = match self.right {
            Output::Free => Output::Failed { owed: None },
            Output::Asked { right } => Output::Failed { owed: Some(right) },
            Output::Granted { right } => Output::FailedGranted { right },
            Output::CancelDue { .. }
            | Output::Cancelling { .. }
            | Output::ReleaseDue { .. }
            | Output::Failed { .. }
            | Output::FailedGranted { .. } => self.right,
        };
        match self.phase {
            Phase::Closing { .. } => return,
            Refusing { .. } => self.closing(Closed::Stream, OutputFate::Keep),
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Ready { .. }
            | Finished { .. }
            | Phase::Closed => {}
        }
        match self.mode {
            StreamMode::One => self.closing(Closed::Stream, OutputFate::Keep),
            StreamMode::Two => above.push(Event::OutputFailed),
        }
    }

    fn read_event(&mut self, event: stream::Up, above: &mut Queue<Event>) {
        match event {
            stream::Up::Bytes(bytes) => {
                self.demand = Demand::None;
                if !self.phase.stopped() {
                    self.read_bytes(bytes, above);
                }
            }
            stream::Up::End => {
                if self.phase.stopped() {
                    return;
                }
                let between_frames = match &self.read {
                    ReadState::Header { first: None }
                    | ReadState::Idle
                    | ReadState::Ended
                    | ReadState::Skipped { .. } => true,
                    ReadState::Header { first: Some(_) } | ReadState::Body { .. } | ReadState::Skip { .. } => false,
                };
                if self.version().is_some() && between_frames {
                    if self.read_credit() {
                        above.push(Event::Ended);
                    }
                    self.read = ReadState::Ended;
                    if self.right.failed() {
                        self.closing(Closed::Stream, OutputFate::Keep);
                    }
                } else {
                    self.closing(Closed::Truncated, self.failure_fate());
                }
            }
            stream::Up::Failed(_) => {
                if self.phase.stopped() {
                    return;
                }
                self.closing(Closed::Stream, self.failure_fate());
                self.demand = Demand::None;
            }
            stream::Up::Room => unreachable!("the channel only sends read-only demands"),
        }
    }

    fn failure_fate(&self) -> OutputFate {
        match self.mode {
            StreamMode::One => OutputFate::Drop,
            StreamMode::Two => OutputFate::Keep,
        }
    }

    #[expect(clippy::manual_let_else, reason = "step code uses exhaustive matches instead of let-else")]
    #[expect(clippy::boxed_local, reason = "application bodies move from this handler into Event::Body")]
    fn read_bytes(&mut self, bytes: Box<[u8]>, above: &mut Queue<Event>) {
        let state = mem::replace(&mut self.read, ReadState::Idle);
        match state {
            ReadState::Header { first: None } => {
                if bytes.len() == 1 {
                    let first = *bytes.first().expect("one delivered byte");
                    self.read = ReadState::Header { first: Some(first) };
                } else {
                    self.framing_error();
                }
            }
            ReadState::Header { first: Some(first) } => {
                if bytes.len() != 7 {
                    self.framing_error();
                    return;
                }
                let mut head = [0_u8; 8];
                *head.get_mut(0).expect("first header cell") = first;
                let rest = head.get_mut(1..).expect("seven header cells");
                for (destination, source) in rest.iter_mut().zip(bytes.iter()) {
                    *destination = *source;
                }
                let header = match parse_header(&head) {
                    Ok(header) => header,
                    Err(_) => {
                        self.framing_error();
                        return;
                    }
                };
                self.begin_body(header, above);
            }
            ReadState::Body { header, mut body, filled } => {
                let count = u32::try_from(bytes.len()).expect("bounded read chunk");
                let end = filled.checked_add(count).expect("body length bounded");
                let start = usize_of(filled);
                let end_index = usize_of(end);
                let target = body.get_mut(start..end_index).expect("chunk fits body");
                for (destination, source) in target.iter_mut().zip(bytes.iter()) {
                    *destination = *source;
                }
                if end == header.body_len {
                    self.received(header.kind, body, above);
                } else {
                    self.read = ReadState::Body { header, body, filled: end };
                }
            }
            ReadState::Skip { kind, remaining } => {
                let delivered = u32::try_from(bytes.len()).expect("bounded skip chunk");
                let left = remaining.checked_sub(delivered).expect("skip demand fits remaining body");
                if left == 0 {
                    self.read = ReadState::Skipped { kind };
                } else {
                    self.read = ReadState::Skip { kind, remaining: left };
                }
            }
            ReadState::Skipped { .. } | ReadState::Ended | ReadState::Idle => {}
        }
    }

    fn begin_body(&mut self, header: Header, above: &mut Queue<Event>) {
        let kind = match self.version() {
            Some(version) => classify(&self.schema, self.role, &self.limits, version, header),
            None => {
                if self.header_allowed(header) {
                    Some(ReadKind::Control)
                } else {
                    None
                }
            }
        };
        match kind {
            Some(ReadKind::Control | ReadKind::Body) => {
                if header.body_len == 0 {
                    self.received(header.kind, Box::from([]), above);
                } else {
                    let body = bytes::zeroed(usize_of(header.body_len));
                    self.read = ReadState::Body { header, body, filled: 0 };
                }
            }
            Some(ReadKind::Skip) => {
                if header.body_len == 0 {
                    self.read = ReadState::Skipped { kind: header.kind };
                } else {
                    self.read = ReadState::Skip { kind: header.kind, remaining: header.body_len };
                }
            }
            None => self.framing_error(),
        }
    }

    fn flush_unsupported(&mut self) {
        let kind = match self.read {
            ReadState::Skipped { kind } => kind,
            ReadState::Header { .. }
            | ReadState::Body { .. }
            | ReadState::Skip { .. }
            | ReadState::Ended
            | ReadState::Idle => return,
        };
        if self.output.room() == 0 || self.queued(QueueTag::Unsupported) {
            return;
        }
        let frame = control_frame(&Control::Unsupported { kind }, &self.limits).expect("fixed Unsupported fits");
        self.queue(frame, QueueTag::Unsupported);
        self.read = ReadState::Header { first: None };
    }

    fn header_allowed(&self, header: Header) -> bool {
        let maximum = match header.kind {
            1 => 14_u32.checked_add(self.limits.credential),
            2 => Some(4),
            3 => Some(262),
            4 => match self.limits.kinds.checked_mul(6) {
                Some(count) => count.checked_add(4),
                None => None,
            },
            5 => Some(0),
            6 => Some(2),
            _ => None,
        };
        match maximum {
            Some(maximum) => header.body_len <= maximum,
            None => false,
        }
    }

    #[expect(clippy::manual_let_else, reason = "step code uses exhaustive matches instead of let-else")]
    #[expect(
        clippy::too_many_lines,
        reason = "the control variants keep their opening and terminal transitions together"
    )]
    fn received(&mut self, kind: u16, body: Box<[u8]>, above: &mut Queue<Event>) {
        if kind >= 0x0100 {
            above.push(Event::Body { kind, body });
            return;
        }
        let control = match decode_control(kind, &body, &self.limits) {
            Ok(control) => control,
            Err(_) => {
                self.framing_error();
                return;
            }
        };
        match control {
            Control::Open { magic, lowest, highest, features, credential } => match self.phase {
                WaitOpen => match accept_open(&self.schema, magic, lowest, highest, features) {
                    Ok((lowest, highest)) => {
                        self.phase = WaitOwner { lowest, highest };
                        above.push(Event::Opening { credential, lowest, highest });
                    }
                    Err(reason) => {
                        if reason == 1 {
                            let ours = self.schema.range().expect("validated versions");
                            let text = super::opening::version_words(ours, (lowest, highest));
                            self.refuse(reason, text, Closed::RefusedHere(reason));
                        } else {
                            self.auto_refuse(reason);
                        }
                    }
                },
                Phase::Idle
                | WaitOwner { .. }
                | WaitAccept
                | WaitTerms { .. }
                | Ready { .. }
                | Refusing { .. }
                | Finished { .. }
                | Phase::Closing { .. }
                | Phase::Closed => {
                    self.framing_error();
                }
            },
            Control::Accept { version, features } => {
                if self.phase == WaitAccept && features == 0 && offers_version(&self.schema, version) {
                    self.phase = WaitTerms { version, own: OwnTerms::Due, peer: None };
                    self.read = ReadState::Header { first: None };
                } else {
                    self.framing_error();
                }
            }
            Control::Refuse { reason, text } => {
                self.terminal_text.clone_from(&text);
                above.push(Event::Refused { reason, text });
                self.closing(Closed::RefusedPeer(reason), OutputFate::Drop);
            }
            Control::Terms { entries } => {
                let (version, peer) = match &mut self.phase {
                    WaitTerms { version, peer, .. } => (*version, peer),
                    Phase::Idle
                    | WaitOpen
                    | WaitOwner { .. }
                    | WaitAccept
                    | Ready { .. }
                    | Refusing { .. }
                    | Finished { .. }
                    | Phase::Closing { .. }
                    | Phase::Closed => {
                        self.framing_error();
                        return;
                    }
                };
                if !check_terms(&self.schema, self.role, version, &entries) {
                    self.framing_error();
                    return;
                }
                *peer = Some(entries);
                if self.role == Role::Initiator {
                    let terms = local_terms(&self.schema, self.role, version, &self.limits);
                    let frame =
                        control_frame(&Control::Terms { entries: terms }, &self.limits).expect("checked Terms fit");
                    self.queue(frame, QueueTag::Terms);
                }
            }
            Control::Ping => match self.phase {
                Ready { .. } | Finished { .. } => above.push(Event::Ping),
                WaitOwner { .. } | WaitAccept | WaitTerms { .. } => {
                    above.push(Event::Ping);
                    self.read = ReadState::Header { first: None };
                }
                Phase::Idle | WaitOpen | Refusing { .. } | Phase::Closing { .. } | Phase::Closed => {
                    self.framing_error();
                }
            },
            Control::Unsupported { kind } => match self.phase {
                Ready { .. } | Finished { .. } => {
                    above.push(Event::Unsupported { kind });
                }
                Phase::Idle
                | WaitOpen
                | WaitOwner { .. }
                | WaitAccept
                | WaitTerms { .. }
                | Refusing { .. }
                | Phase::Closing { .. }
                | Phase::Closed => self.framing_error(),
            },
        }
    }

    fn auto_refuse(&mut self, reason: u16) {
        let why = if reason == 3 { Closed::Framing } else { Closed::RefusedHere(reason) };
        self.refuse(reason, Box::from([]), why);
    }

    fn framing_error(&mut self) {
        self.auto_refuse(3);
    }

    fn maybe_ready(&mut self, above: &mut Queue<Event>) {
        let phase = mem::replace(&mut self.phase, Phase::Idle);
        self.phase = match phase {
            WaitTerms { version, own: OwnTerms::Sent, peer: Some(peer) } => {
                above.push(Event::Ready { version, terms: peer.clone() });
                self.read = ReadState::Idle;
                Ready { version, peer }
            }
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Ready { .. }
            | Finished { .. }
            | Refusing { .. }
            | Phase::Closing { .. }
            | Phase::Closed => phase,
        };
    }

    fn maybe_finish(&mut self, below: &mut Queue<Lower>) {
        if !self.output.is_empty() || self.right.token().is_some() {
            return;
        }
        match self.phase {
            Refusing { why, finish: Finish::Due } => {
                if self.demand == Demand::Out {
                    below.push(Lower::Read(stream::Down::Demand { read: stream::Read::Nothing, room: 0 }));
                    self.demand = Demand::None;
                }
                self.closing(why, OutputFate::Keep);
            }
            Finished { version, finish: Finish::Due } => {
                if self.mode == StreamMode::One && (self.demand == Demand::Out || self.read_credit()) {
                    return;
                }
                self.phase = Finished { version, finish: Finish::Sent };
            }
            Phase::Idle
            | WaitOpen
            | WaitOwner { .. }
            | WaitAccept
            | WaitTerms { .. }
            | Ready { .. }
            | Phase::Closing { .. }
            | Phase::Closed
            | Refusing { finish: Finish::Sent, .. }
            | Finished { finish: Finish::Sent, .. } => return,
        }
        match self.mode {
            StreamMode::One => below.push(Lower::Read(stream::Down::Finish)),
            StreamMode::Two => below.push(Lower::FinishWrite),
        }
    }
}
