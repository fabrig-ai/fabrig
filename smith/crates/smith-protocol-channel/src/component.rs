//! Agent-side opening machine (protocol/channel.md, sections 2 and 5).
use alloc::boxed::Box;
use skein_channel::{Closed, Event, Frame, Lower, LowerEvent, Machine, Request, Role, Schema, StreamMode};
use skein_lib::{Duration, Map, Queue, Time, Token};

use crate::calls;
use crate::credentials::Credentials;
use crate::facts;
use crate::limits::{Error, Limits};
use smith_domain::run;
use smith_domain_session::record;
use smith_protocol_translate::encode_turn;
use smith_protocol_translate::{DecodedStart, Endpoints, answer_record};

mod cancellation;
mod ending;
mod notices;
mod receive;
mod state;
mod turns;

use ending::Ending;
use state::State;

use smith_channel::reason::RULES;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CallKey {
    activation: u64,
    completion: u32,
    position: u32,
}

impl CallKey {
    fn from_domain(name: run::CallName) -> CallKey {
        CallKey { activation: name.activation, completion: name.completion, position: name.position }
    }

    fn from_wire(name: &smith_channel::CallName) -> CallKey {
        CallKey { activation: name.activation(), completion: name.completion(), position: name.position() }
    }

    fn domain(self) -> run::CallName {
        run::CallName { activation: self.activation, completion: self.completion, position: self.position }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CallRoute {
    Host { relay: run::RelayName, send: Token },
    Delivery { owner: Token, send: Token },
}

/// One channel event for the agent service.
#[derive(Debug)]
pub enum OpenEvent {
    /// Framing and terms have been agreed at this version.
    Opened { version: u16 },
    /// A structurally valid Start with its charter translated before domain admission.
    Start { start: Box<DecodedStart> },
    /// A host-labelled message for the admitted run.
    Message { finish_requested: bool, name: Token, label: Box<[u8]>, text: Box<[u8]> },
    /// A declared host tool's one settled terminal.
    HostReturned { relay: run::RelayName, reply: run::HostReply },
    /// A delivery's one settled terminal.
    Delivered { owner: Token, delivery: Box<run::Delivery> },
    /// A delivery frame never entered the stream; service settles its IO operation.
    DeliveryUnsent { owner: Token },
    /// The host durably kept this numbered turn and its preceding prefix.
    Acknowledged { turn: u32 },
    /// A refreshed credential name and validity, without its value.
    Grant { grant: smith_domain::Grant },
    /// The host asked the admitted run to stop politely once.
    Cancel,
    /// Independent write failure; input and terminal rights may still continue.
    WriteFailed,
    /// The channel ended before or after opening.
    Ended { why: Closed },
}

/// The responder's framed channel and checked sending terms.
#[derive(Debug)]
pub struct Component {
    machine: Machine,
    schema: Schema,
    events: Queue<Event>,
    state: State,
    ending: Ending,
    bodies: smith_channel::Limits,
    charter: smith_charter::v4::Limits,
    transcript: smith_transcript::v6::Limits,
    endpoints: Endpoints,
    calls: Map<CallKey, CallRoute>,
    turn_capacity: u32,
    resume_turns: u32,
    search_hits: u32,
    fact_reserve_frames: u32,
    fact_reserve_bytes: u32,
    lost_facts: u64,
    lost_live_bytes: u64,
    credentials: Credentials,
}

impl Component {
    /// Build one channel with configured endpoint names and either stream shape.
    pub fn new(limits: &Limits, mode: StreamMode, endpoints: Endpoints) -> Result<Component, Error> {
        if !endpoints.fits(limits.endpoints, limits.charter.llm_endpoint) {
            return Err(Error::Endpoints);
        }
        let schema = match crate::limits::schema(limits) {
            Ok(schema) => schema,
            Err(error) => return Err(Error::Codec(error)),
        };
        let machine = match Machine::with_mode(schema.clone(), Role::Responder, limits.channel, mode) {
            Ok(machine) => machine,
            Err(error) => return Err(Error::Channel(error)),
        };
        Ok(Component {
            machine,
            schema,
            events: Queue::uncapped(),
            state: State::Opening,
            ending: Ending::Listening,
            bodies: limits.bodies,
            charter: limits.charter,
            transcript: limits.transcript,
            endpoints,
            calls: Map::with_capacity(limits.calls),
            turn_capacity: limits.turns,
            resume_turns: limits.resume_turns,
            search_hits: limits.search_hits,
            fact_reserve_frames: limits.fact_reserve_frames,
            fact_reserve_bytes: limits.fact_reserve_bytes,
            lost_facts: 0,
            lost_live_bytes: 0,
            credentials: Credentials::new(limits.grants),
        })
    }

    /// The channel has reported its end to its service.
    #[must_use]
    pub fn ended(&self) -> bool {
        self.ending.reported()
    }

    /// Consume one stream event and progress the opening.
    pub fn from_below(&mut self, event: LowerEvent, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        self.machine.up(event, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
    }

    /// Request the next read or write operation, including the initial Open read.
    pub fn fire(&mut self, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        self.machine.poll(&mut self.events, below);
        self.drain(to_service, below);
    }

    /// Send the domain's admission notice once the Start has entered it.
    pub fn send_admitted(
        &mut self,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.started() {
            return Err(Error::Order);
        }
        let record = smith_channel::Admitted::new(&self.bodies, smith_channel::AdmittedParts {})?;
        let frame = smith_channel::frame(smith_channel::Body::Admitted(record))?;
        self.state.admit();
        self.follow(below);
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Tell the host the one terminal of a refused message.
    pub fn send_message_refused(
        &mut self,
        name: Token,
        reason: run::MessageRefusal,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let reason = match reason {
            run::MessageRefusal::TooLarge => smith_channel::MessageRefusal::TooLarge,
            run::MessageRefusal::Full => smith_channel::MessageRefusal::Full,
            run::MessageRefusal::NameInUse => smith_channel::MessageRefusal::NameInUse,
            run::MessageRefusal::Ending => smith_channel::MessageRefusal::Ending,
        };
        let record = smith_channel::MessageRefused::new(
            &self.bodies,
            smith_channel::MessageRefusedParts { name: name.raw(), reason },
        )?;
        let frame = smith_channel::frame(smith_channel::Body::MessageRefused(record))?;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Send the domain's final answer and finish the write stream after it.
    pub fn send_answer(
        &mut self,
        now: Time,
        answer: run::Answer,
        read: Option<Token>,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        let permitted = match &answer {
            run::Answer::Refused(_) => self.state.started(),
            run::Answer::Parked { .. } | run::Answer::Accepted { .. } | run::Answer::Failed { .. } => {
                self.state.admitted()
            }
        };
        if !permitted {
            return Err(Error::Order);
        }
        let record = answer_record(now, answer, read, &self.bodies, &self.charter)?;
        let frame = smith_channel::frame(smith_channel::Body::Answer(record))?;
        self.state.answer();
        self.follow(below);
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.machine.down(Request::Finish, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Tell the host that the run waits after the last message it read.
    #[expect(clippy::manual_map, reason = "production steps do not use closure methods")]
    pub fn send_waiting(
        &mut self,
        read: Option<Token>,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let last_read = match read {
            Some(name) => Some(name.raw()),
            None => None,
        };
        let record = smith_channel::Waiting::new(&self.bodies, smith_channel::WaitingParts { last_read })?;
        let frame = smith_channel::frame(smith_channel::Body::Waiting(record))?;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Announce the bounded span of a check or other long operation.
    pub fn send_long(
        &mut self,
        operation: smith_domain::Operation,
        span: Duration,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let next = self.state.active()?.long_open.checked_add(1).ok_or(Error::Order)?;
        let operation = match operation {
            smith_domain::Operation::Checks => smith_channel::Operation::Check,
            smith_domain::Operation::Completion => smith_channel::Operation::Completion,
            smith_domain::Operation::Command => smith_channel::Operation::Command,
        };
        let record = smith_channel::Long::new(&self.bodies, smith_channel::LongParts { operation, span })?;
        let frame = smith_channel::frame(smith_channel::Body::Long(record))?;
        self.state.active_mut()?.long_open = next;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// End one announced operation after its observed terminal.
    pub fn send_long_done(
        &mut self,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() || self.state.active()?.long_open == 0 {
            return Err(Error::Order);
        }
        let record = smith_channel::LongDone::new(&self.bodies, smith_channel::LongDoneParts {})?;
        let frame = smith_channel::frame(smith_channel::Body::LongDone(record))?;
        let active = self.state.active_mut()?;
        active.long_open = active.long_open.checked_sub(1).expect("a begun operation owns this terminal");
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Tell the host one concrete transcript turn with activation-wide spend.
    #[expect(clippy::manual_map, reason = "production steps do not use closure methods")]
    #[expect(clippy::too_many_arguments, reason = "turn and its host envelope have independent names")]
    pub fn send_turn(
        &mut self,
        number: u32,
        read: Option<Token>,
        spent: run::Spend,
        opens_window: bool,
        turn: &record::Turn,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() || number == 0 {
            return Err(Error::Order);
        }
        let body = encode_turn(turn, &self.transcript, &self.endpoints)?;
        let body_bytes = turn.owned_bytes().ok_or(Error::Window)?;
        self.state.active()?.turns.check(number, body_bytes)?;
        let last_read = match read {
            Some(name) => Some(name.raw()),
            None => None,
        };
        let record = smith_channel::Turn::new(
            &self.bodies,
            smith_channel::TurnParts { number, spent: spent.units, last_read, opens_window, body },
        )?;
        let frame = smith_channel::frame(smith_channel::Body::Turn(record))?;
        self.state.turns_mut()?.keep(number, body_bytes)?;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Whether one largest authoritative frame can enter the bounded channel.
    /// The service checks before taking a request; facts keep this same byte reserve.
    #[must_use]
    pub fn takes(&self) -> bool {
        let room = self.machine.room();
        room.frames > 0 && room.bytes >= self.fact_reserve_bytes
    }

    /// Offer a content-free domain observation without spending reserved output room.
    pub fn send_fact(
        &mut self,
        fact: &smith_domain::Fact,
        started: Time,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<bool, Error> {
        let elapsed = fact.emitted().saturating_since(started);
        match facts::project(fact) {
            Some((kind, count)) => self.offer_fact(kind, elapsed, count, token, to_service, below),
            None => Ok(false),
        }
    }

    /// Count text observed from the LLM without putting its bytes on the host channel.
    pub fn send_text_arrived(
        &mut self,
        elapsed: Duration,
        count: u64,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<bool, Error> {
        self.offer_fact(smith_channel::FactKind::TextArrived, elapsed, count, token, to_service, below)
    }

    /// Projectable observations omitted outside admission or for lack of output room.
    #[must_use]
    pub fn lost_facts(&self) -> u64 {
        self.lost_facts
    }

    /// Whether one text piece fits without spending authoritative or fact room.
    #[must_use]
    pub fn takes_live(&self, kind: crate::PieceKind<'_>, bytes: u32) -> bool {
        if !self.state.admitted() || self.ended() {
            return false;
        }
        let Ok(active) = self.state.active() else { return false };
        let Some(ask) = &active.live else {
            return false;
        };
        let asked = match kind {
            crate::PieceKind::Text => ask.text(),
            crate::PieceKind::Input { .. } => ask.input(),
        };
        if !asked {
            return false;
        }
        let Some(frame) = crate::live::frame_bytes(&self.bodies, bytes) else {
            return false;
        };
        let Some(fact) = smith_channel::Fact::encoded_bound(&self.bodies) else {
            return false;
        };
        let needed = u64::from(self.fact_reserve_bytes).checked_add(fact);
        let needed = match needed {
            Some(n) => n.checked_add(8),
            None => None,
        };
        let needed = match needed {
            Some(n) => n.checked_add(u64::from(frame)),
            None => None,
        };
        let Some(needed) = needed else {
            return false;
        };
        let room = self.machine.room();
        u64::from(room.frames) > u64::from(self.fact_reserve_frames).saturating_add(1)
            && u64::from(room.bytes) >= needed
    }

    /// Send a piece, counting its bytes when admission, order or room prevents it.
    pub fn send_live(
        &mut self,
        piece: crate::Piece<'_>,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<bool, Error> {
        let length = u64::try_from(piece.bytes.len()).expect("piece length fits u64");
        if self.state.next_turn() != Some(piece.turn)
            || !self.takes_live(piece.kind, u32::try_from(length).unwrap_or(u32::MAX))
            || to_service.room() < crate::limits::EDGE_ROOM.to_domain
            || below.room() < crate::limits::EDGE_ROOM.below
        {
            self.lose_live(length);
            return Ok(false);
        }
        let frame = crate::live::frame(piece, &self.bodies)?;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(true)
    }

    /// Count bytes dropped above the channel boundary.
    pub fn lose_live(&mut self, bytes: u64) {
        self.lost_live_bytes = self.lost_live_bytes.saturating_add(bytes);
    }

    /// Live bytes omitted from the host channel.
    #[must_use]
    pub fn lost_live_bytes(&self) -> u64 {
        self.lost_live_bytes
    }

    /// Read a retained credential only below the domain when preparing its LLM call.
    #[must_use]
    pub fn grant_value(&self, name: smith_domain::GrantName) -> Option<&crate::GrantValue> {
        self.credentials.value(name.account, name.generation)
    }

    fn offer_fact(
        &mut self,
        kind: smith_channel::FactKind,
        elapsed: Duration,
        count: u64,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<bool, Error> {
        if !self.state.admitted()
            || self.ended()
            || to_service.room() < crate::limits::EDGE_ROOM.to_domain
            || below.room() < crate::limits::EDGE_ROOM.below
        {
            self.lost_facts = self.lost_facts.saturating_add(1);
            return Ok(false);
        }
        let record = smith_channel::Fact::new(&self.bodies, smith_channel::FactParts { kind, elapsed, count })?;
        let frame = smith_channel::frame(smith_channel::Body::Fact(record))?;
        let room = self.machine.room();
        let reserve = match crate::live::frame_bytes(&self.bodies, self.bodies.live_bytes) {
            Some(live) => match self.fact_reserve_bytes.checked_add(live) {
                Some(n) => n.checked_add(frame.wire_len()),
                None => None,
            },
            None => None,
        };
        let bytes_fit = match reserve {
            Some(needed) => room.bytes >= needed,
            None => false,
        };
        if u64::from(room.frames) <= u64::from(self.fact_reserve_frames).saturating_add(1) || !bytes_fit {
            self.lost_facts = self.lost_facts.saturating_add(1);
            return Ok(false);
        }
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(true)
    }

    /// Relay one declared host tool under its durable name and live attempt.
    #[expect(clippy::too_many_arguments, reason = "the typed channel boundary carries each domain value")]
    pub fn send_host_call(
        &mut self,
        now: Time,
        name: run::CallName,
        relay: run::RelayName,
        tool: Box<[u8]>,
        effect: run::HostEffect,
        input: run::HostInput,
        deadline: Time,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let key = CallKey::from_domain(name);
        if self.calls.contains_key(&key) || self.calls.len() >= self.calls.capacity() {
            return Err(Error::Calls);
        }
        let frame = calls::host_call(name, tool, effect, input, deadline.saturating_since(now), &self.bodies)?;
        if self.calls.insert(key, CallRoute::Host { relay, send: token }).is_err() {
            return Err(Error::Calls);
        }
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Relay one checked delivery after its run and checkout effects settled.
    #[expect(clippy::too_many_arguments, reason = "the typed channel boundary carries each domain value")]
    pub fn send_delivery(
        &mut self,
        now: Time,
        name: run::CallName,
        owner: Token,
        change: run::outcome::Change,
        deadline: Time,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let key = CallKey::from_domain(name);
        if self.calls.contains_key(&key) || self.calls.len() >= self.calls.capacity() {
            return Err(Error::Calls);
        }
        let frame = calls::delivery_call(name, change, deadline.saturating_since(now), &self.bodies)?;
        if self.calls.insert(key, CallRoute::Delivery { owner, send: token }).is_err() {
            return Err(Error::Calls);
        }
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Ask the host to settle a live tool relay; its eventual answer remains owed.
    pub fn send_withdraw(
        &mut self,
        relay: run::RelayName,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if !self.state.admitted() {
            return Err(Error::Order);
        }
        let mut name = None;
        for (key, route) in &self.calls {
            match route {
                CallRoute::Host { relay: held, send: _ } if *held == relay => name = Some(key.domain()),
                CallRoute::Host { .. } | CallRoute::Delivery { .. } => {}
            }
        }
        let name = name.ok_or(Error::Calls)?;
        let frame = calls::withdraw(name, &self.bodies)?;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Whether the channel can take application records.
    #[must_use]
    pub fn opened(&self) -> bool {
        self.state.opened()
    }

    fn refuse_rules(&mut self, below: &mut Queue<Lower>) {
        self.machine.down(
            Request::Refuse { reason: RULES, text: Box::from(*b"invalid Start") },
            &mut self.events,
            below,
        );
    }

    fn stash_initial_grants(&mut self, start: &smith_channel::Start) -> Result<(), Error> {
        for grant in start.grants() {
            self.credentials.keep(grant.account(), grant.generation(), crate::GrantValue::try_from(grant.value())?)?;
        }
        Ok(())
    }
}

fn invalid_answer(limits: &smith_channel::Limits, invalid: smith_channel::InvalidStart) -> Option<Frame> {
    let reason =
        smith_channel::InvalidStartValue::new(limits, smith_channel::InvalidStartValueParts { value: invalid }).ok()?;
    let refused = smith_channel::Refused::new(
        limits,
        smith_channel::RefusedParts { reason: smith_channel::StartRefusal::Invalid(reason) },
    )
    .ok()?;
    let record = smith_channel::Answer::new(
        limits,
        smith_channel::AnswerParts {
            turns: 0,
            spent: 0,
            last_read: None,
            result: smith_channel::RunResult::Refused(refused),
        },
    )
    .ok()?;
    let frame = smith_channel::frame(smith_channel::Body::Answer(record)).ok()?;
    Some(frame)
}

fn kind_text(kind: u16) -> Box<[u8]> {
    let bytes = kind.to_be_bytes();
    Box::from([
        b'k',
        b'i',
        b'n',
        b'd',
        b' ',
        b'0',
        b'x',
        skein_lib::bytes::hex_digit(bytes[0] >> 4),
        skein_lib::bytes::hex_digit(bytes[0] & 15),
        skein_lib::bytes::hex_digit(bytes[1] >> 4),
        skein_lib::bytes::hex_digit(bytes[1] & 15),
    ])
}
