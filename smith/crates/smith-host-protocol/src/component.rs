//! Host-side channel (protocol/hosts.md, section 2; domain/host.md, section 6).
//! Keeps the opening, one application demand and at most one held closure reason,
//! never host policy or read-ahead records. `open` runs the handshake; `read`
//! demands one record, live piece or end; `send_*`, `from_below` and `fire`
//! translate output and stream events without spending that demand.
//! A withdrawal that crosses the host's answer settles the call: a decoded
//! withdrawal naming no call in flight is ignored, leaving the read owed
//! (smith/docs/design/protocol/channel.md, sections 3–4). Answered names are not
//! retained, since answers need no withdrawal and those names grow with a run.
use alloc::boxed::Box;
use skein_channel::{Closed, Event, Lower, LowerEvent, Machine, Request, Role, Schema, StreamMode, frame_writer};
use skein_lib::{Duration, Map, Queue, Time, Token, Writer};
use smith_host_domain::channel;

use crate::limits::{Error, Limits};
use crate::translate::{Values, encode_reply, encode_start};

mod notices;
mod receive;

const RULES: u16 = 256;
const LIMITS: u16 = 2;
const CHILD_TRANSITIONS: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Opening,
    Opened,
    Started,
    Admitted,
    Answered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CallKey {
    activation: u64,
    completion: u32,
    position: u32,
}

impl CallKey {
    fn from_wire(name: &smith_channel::CallName) -> CallKey {
        CallKey { activation: name.activation(), completion: name.completion(), position: name.position() }
    }

    fn domain(self) -> channel::CallName {
        channel::CallName { activation: self.activation, completion: self.completion, position: self.position }
    }
}

/// An opening result for the host service.
#[derive(Debug, PartialEq, Eq)]
pub enum OpenEvent {
    /// Framing and terms have been agreed at this version.
    Opened { version: u16 },
    /// The channel ended before or after opening.
    Hangup { why: Closed },
    /// The framed Start entered the channel's bounded output queue.
    Sent { token: Token },
    /// The Start did not enter the channel's output queue.
    Unsent { token: Token, why: skein_channel::Unsent },
    /// The agent's last word in host-domain vocabulary, with its wire record for audit.
    Answer { answer: channel::Answer, record: smith_channel::Answer },
    /// The agent admitted the Start before its later final answer.
    Admitted,
    /// The run refused one named host message at ingress.
    MessageRefused { name: Token, reason: smith_host_domain::MessageRefusal },
    /// The run is waiting after reading the named message, if any.
    Waiting { read: Option<Token> },
    /// The run announced a long operation with a bounded progress extension.
    Long { operation: smith_domain::Operation, span: Duration },
    /// The run ended its announced long operation.
    LongDone,
    /// One concrete turn and its checked host envelope.
    Turn { turn: channel::Turn },
    /// One content-free, best-effort fact kept in its bounded wire body.
    Fact { body: Box<[u8]> },
    /// One live piece checked against the start and stream order, for the owner.
    Live {
        turn: u32,
        retry: u32,
        kind: smith_channel::LiveKind,
        offset: u64,
        bytes: Box<[u8]>,
        call: Option<smith_channel::LiveCall>,
    },
    /// A provider rejected this known account and credential generation.
    Unauthorized { account: u32, generation: u64 },
    /// A provider exhausted this known account for a relative cooldown.
    Limited { account: u32, reset: Option<Duration> },
    /// A provider refused an account request, with bounded code and reason.
    Refused { account: u32, code: Box<[u8]>, reason: Box<[u8]> },
    /// The independent write stream failed while reads may still settle.
    WriteFailed,
    /// A named operation whose one host terminal remains owed.
    Call { call: Token, name: channel::CallName, deadline: Time, ask: channel::Ask },
    /// The agent withdrew a call; its terminal still remains owed.
    Withdraw { call: Token },
}

/// The initiator's framed channel and checked sending terms.
#[derive(Debug)]
pub struct Component {
    machine: Machine,
    schema: Schema,
    bodies: smith_channel::Limits,
    events: Queue<Event>,
    state: State,
    ended: bool,
    reading: bool,
    held: Option<Closed>,
    calls_by_name: Map<CallKey, Token>,
    calls_by_token: Map<Token, CallKey>,
    next_call: u64,
    now: Time,
    cancelled: bool,
    endpoints: smith_protocol_channel::Endpoints,
    charter: smith_charter::v4::Limits,
    transcript: smith_transcript::v6::Limits,
    sequence: u32,
    long_open: u32,
    live: crate::live::Streams,
}

impl Component {
    /// Build one channel with a pipe pair or one socket-like stream.
    pub fn new(
        limits: &Limits,
        mode: StreamMode,
        endpoints: smith_protocol_channel::Endpoints,
    ) -> Result<Component, Error> {
        if !endpoints.fits(limits.endpoints, limits.charter.llm_endpoint) {
            return Err(Error::Value);
        }
        let schema = match smith_protocol_channel::schema_for_start(&limits.bodies, limits.start_bytes) {
            Ok(schema) => schema,
            Err(error) => return Err(Error::Codec(error)),
        };
        let machine = match Machine::with_mode(schema.clone(), Role::Initiator, limits.channel, mode) {
            Ok(machine) => machine,
            Err(error) => return Err(Error::Channel(error)),
        };
        Ok(Component {
            machine,
            schema,
            bodies: limits.bodies,
            events: Queue::uncapped(),
            state: State::Opening,
            ended: false,
            reading: false,
            held: None,
            calls_by_name: Map::with_capacity(limits.calls),
            calls_by_token: Map::with_capacity(limits.calls),
            next_call: 1,
            now: Time::ZERO,
            cancelled: false,
            endpoints,
            charter: limits.charter,
            transcript: limits.transcript,
            sequence: 0,
            long_open: 0,
            live: crate::live::Streams::new(None, 0),
        })
    }

    /// Start the pipe opening with its empty credential.
    pub fn open(&mut self, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        self.machine.down(Request::Open { credential: Box::default() }, &mut self.events, below);
        self.fire(to_service, below);
    }

    /// Demand one record, one live piece or the channel's end, answered once.
    pub fn read(&mut self, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        assert!(self.state != State::Opening, "application reads follow opening");
        assert!(!self.reading, "one application read at a time");
        assert!(!self.ended, "no read after the delivered end");
        self.reading = true;
        match self.held.take() {
            Some(why) => self.end(why, to_service),
            None => {
                self.machine.down(Request::Read, &mut self.events, below);
                self.drain(to_service, below);
                self.fire(to_service, below);
            }
        }
    }

    /// Set the monotonic clock for converting agent-relative deadlines into host deadlines.
    pub fn set_now(&mut self, now: Time) {
        self.now = now;
    }

    /// Submit the first application record after opening, with host service values.
    pub fn send_start(
        &mut self,
        start: channel::Start,
        window: channel::Window,
        values: Values,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        let sequence = match &start.transcript {
            Some(history) => match history.value().turns.last() {
                Some(turn) => turn.sequence,
                None => 0,
            },
            None => 0,
        };
        let live = start.live;
        let frame =
            encode_start(start, window, values, &self.bodies, &self.charter, &self.transcript, &self.endpoints)?;
        if self.state != State::Opened {
            return Err(Error::MissingValue);
        }
        self.live = crate::live::Streams::new(live, 0);
        self.sequence = sequence;
        self.state = State::Started;
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Relay one named sender label and text to the admitted agent.
    pub fn send_message(
        &mut self,
        message: channel::Message,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if self.state != State::Admitted {
            return Err(Error::Order);
        }
        let record = crate::translate::encode_message(message, &self.bodies)?;
        let Ok(length) = usize::try_from(record.measure()) else {
            return Err(Error::MissingValue);
        };
        let mut body = Writer::new(length);
        record.encode(&mut body)?;
        let mut frame = frame_writer(0x0101, record.measure())?;
        frame.put(&body.finish())?;
        self.machine.down(Request::Send { token, frame: frame.finish()? }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Answer one admitted agent call, keeping its durable wire name.
    pub fn send_reply(
        &mut self,
        call: Token,
        reply: channel::Reply,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if self.state != State::Admitted {
            return Err(Error::Order);
        }
        let key = *self.calls_by_token.get(&call).ok_or(Error::Calls)?;
        let frame = encode_reply(key.domain(), reply, &self.bodies)?;
        self.calls_by_token.remove(&call);
        self.calls_by_name.remove(&key);
        self.machine.down(Request::Send { token, frame }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Acknowledge exact durable commitment of one forwarded turn.
    pub fn send_acknowledge(
        &mut self,
        turn: u32,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if self.state != State::Admitted {
            return Err(Error::Order);
        }
        let record = smith_channel::Acknowledge::new(&self.bodies, smith_channel::AcknowledgeParts { number: turn })?;
        let Ok(length) = usize::try_from(record.measure()) else {
            return Err(Error::MissingValue);
        };
        let mut body = Writer::new(length);
        record.encode(&mut body)?;
        let mut frame = frame_writer(0x0103, record.measure())?;
        frame.put(&body.finish())?;
        self.machine.down(Request::Send { token, frame: frame.finish()? }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Refresh one named grant with the service-owned credential value.
    pub fn send_grant(
        &mut self,
        grant: channel::Grant,
        credential: smith_protocol_channel::GrantValue,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if self.state != State::Admitted {
            return Err(Error::Order);
        }
        if crate::grant_form(&credential)? != grant.form {
            return Err(Error::Grants);
        }
        let value = crate::translate::grant_value(credential, &self.bodies)?;
        let grant = smith_channel::Grant::new(
            &self.bodies,
            smith_channel::GrantParts {
                account: grant.account,
                generation: grant.generation,
                valid: grant.valid,
                value,
            },
        )?;
        let record = smith_channel::GrantRefresh::new(&self.bodies, smith_channel::GrantRefreshParts { grant })?;
        let Ok(length) = usize::try_from(record.measure()) else {
            return Err(Error::MissingValue);
        };
        let mut body = Writer::new(length);
        record.encode(&mut body)?;
        let mut frame = frame_writer(0x0104, record.measure())?;
        frame.put(&body.finish())?;
        self.machine.down(Request::Send { token, frame: frame.finish()? }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Ask the agent to stop politely once, preserving answers owed to calls.
    pub fn send_cancel(
        &mut self,
        token: Token,
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<(), Error> {
        if self.state != State::Started && self.state != State::Admitted || self.cancelled {
            return Err(Error::Order);
        }
        let record = smith_channel::Cancel::new(&self.bodies, smith_channel::CancelParts {})?;
        let Ok(length) = usize::try_from(record.measure()) else {
            return Err(Error::MissingValue);
        };
        let mut body = Writer::new(length);
        record.encode(&mut body)?;
        let mut frame = frame_writer(0x0105, record.measure())?;
        frame.put(&body.finish())?;
        self.cancelled = true;
        self.machine.down(Request::Send { token, frame: frame.finish()? }, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
        Ok(())
    }

    /// Consume one stream event and progress the opening.
    pub fn from_below(&mut self, event: LowerEvent, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        self.machine.up(event, &mut self.events, below);
        self.drain(to_service, below);
        self.fire(to_service, below);
    }

    /// Request the next read or write operation.
    pub fn fire(&mut self, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        self.machine.poll(&mut self.events, below);
        self.drain(to_service, below);
    }

    /// Whether the channel can take application records.
    #[must_use]
    pub fn opened(&self) -> bool {
        self.state != State::Opening
    }

    fn drain(&mut self, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        // Eight child transitions bound an entrance: opening/terms, one demanded
        // record or refusal, and its output and close notifications.
        for _ in 0..CHILD_TRANSITIONS {
            let event = self.events.pop();
            match event {
                Some(Event::Opening { .. }) => {
                    self.machine.down(
                        Request::Refuse { reason: RULES, text: Box::from(*b"unexpected opening") },
                        &mut self.events,
                        below,
                    );
                }
                Some(Event::Ready { version, terms }) => {
                    match smith_channel::peer_terms_gap(&self.schema, Role::Initiator, version, &terms) {
                        Some(kind) => {
                            self.machine.down(
                                Request::Refuse { reason: LIMITS, text: kind_text(kind) },
                                &mut self.events,
                                below,
                            );
                        }
                        None => {
                            self.state = State::Opened;
                            to_service.push(OpenEvent::Opened { version });
                        }
                    }
                }
                Some(Event::Body { kind, body }) => match self.receive(kind, body) {
                    Ok(Some(event)) => self.deliver(event, to_service),
                    Ok(None) => self.machine.down(Request::Read, &mut self.events, below),
                    Err(error) => self.refuse_rules(receive::refusal_text(kind, error), below),
                },
                Some(Event::Closed { why }) => self.end(why, to_service),
                Some(Event::Ended) => {
                    // Clean EOF between demands stays in the machine's ReadState::Ended.
                    if self.reading {
                        self.end(Closed::Stream, to_service);
                    }
                }
                Some(Event::Sent { token }) => {
                    to_service.push(OpenEvent::Sent { token });
                }
                Some(Event::Unsent { token, why }) => {
                    to_service.push(OpenEvent::Unsent { token, why });
                }
                Some(Event::Ping | Event::Unsupported { .. }) => {
                    if self.reading {
                        self.machine.down(Request::Read, &mut self.events, below);
                    }
                }
                Some(Event::Refused { .. } | Event::Drained) | None => {}
                Some(Event::OutputFailed) => to_service.push(OpenEvent::WriteFailed),
            }
        }
    }

    fn deliver(&mut self, event: OpenEvent, to_service: &mut Queue<OpenEvent>) {
        assert!(self.reading, "one record answers one application read");
        self.reading = false;
        to_service.push(event);
    }

    fn end(&mut self, why: Closed, to_service: &mut Queue<OpenEvent>) {
        if self.ended || self.held.is_some() {
            return;
        }
        if self.state == State::Opening || self.reading {
            self.reading = false;
            self.ended = true;
            to_service.push(OpenEvent::Hangup { why });
        } else {
            self.held = Some(why);
        }
    }

    fn refuse_rules(&mut self, reason: &[u8], below: &mut Queue<Lower>) {
        self.machine.down(Request::Refuse { reason: RULES, text: Box::from(reason) }, &mut self.events, below);
    }
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

#[cfg(test)]
mod tests {
    use super::{Component, Event, OpenEvent};
    use alloc::boxed::Box;
    use skein_channel::{Lower, LowerEvent};
    use skein_lib::{Queue, Writer, stream};

    fn component() -> Component {
        let (component, _above, _below) = super::grant_tests::admitted_channel();
        component
    }

    fn turn_body(payload: Box<[u8]>) -> Box<[u8]> {
        let record = smith_channel::Turn::new(
            &smith_channel::CEILINGS,
            smith_channel::TurnParts { number: 1, spent: 2, last_read: None, opens_window: true, body: payload },
        )
        .expect("envelope");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measure"));
        record.encode(&mut writer).expect("encode");
        writer.finish()
    }

    fn accepted_body(payload: Box<[u8]>) -> Box<[u8]> {
        let result =
            smith_channel::Accepted::new(&smith_channel::CEILINGS, smith_channel::AcceptedParts { result: payload })
                .expect("accepted");
        let result =
            smith_channel::RunResult::new(&smith_channel::CEILINGS, smith_channel::RunResult::Accepted(result))
                .expect("result");
        let record = smith_channel::Answer::new(
            &smith_channel::CEILINGS,
            smith_channel::AnswerParts { turns: 0, spent: 0, last_read: None, result },
        )
        .expect("answer");
        let mut writer = Writer::new(usize::try_from(record.measure()).expect("measure"));
        record.encode(&mut writer).expect("encode");
        writer.finish()
    }

    #[test]
    fn malformed_typed_bodies_send_rules_before_the_domain_hears_payloads() {
        for (kind, body) in [
            (0x0109, turn_body(Box::from(&b"malformed turn"[..]))),
            (0x0110, accepted_body(Box::from(&b"malformed result"[..]))),
        ] {
            let mut component = component();
            let mut above = Queue::with_capacity(8);
            let mut below = Queue::with_capacity(16);
            component.read(&mut above, &mut below);
            while below.pop().is_some() {}
            component.events.push(Event::Body { kind, body });
            component.drain(&mut above, &mut below);
            component.fire(&mut above, &mut below);
            assert!(above.is_empty(), "malformed value never reaches the host domain");
            assert!(component.reading, "refusal preserves demand until Hangup");
            let Some(Lower::Write(stream::OutputDown::Room { right, .. })) = below.pop() else {
                panic!("rules refusal requests write room")
            };
            component.from_below(
                LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
                &mut above,
                &mut below,
            );
            let Some(Lower::Write(stream::OutputDown::Send { bytes, .. })) = below.pop() else {
                panic!("rules refusal is sent")
            };
            assert_eq!(skein_channel::parse_header(&bytes[..8]).expect("header").kind, 3);
            assert_eq!(u16::from_be_bytes([bytes[8], bytes[9]]), super::RULES);
        }
    }

    #[test]
    fn a_closed_reason_waits_for_one_read_without_being_overwritten() {
        let mut component = component();
        let mut above = Queue::with_capacity(8);
        let mut below = Queue::with_capacity(16);
        component.events.push(Event::Closed { why: skein_channel::Closed::Truncated });
        component.events.push(Event::Closed { why: skein_channel::Closed::Stream });
        component.events.push(Event::Ended);
        component.drain(&mut above, &mut below);
        assert!(above.is_empty(), "closure between reads is held");
        component.read(&mut above, &mut below);
        assert_eq!(above.pop(), Some(OpenEvent::Hangup { why: skein_channel::Closed::Truncated }));
        assert!(!component.reading);
        assert!(below.is_empty(), "held closure answers synchronously");
        component.events.push(Event::Closed { why: skein_channel::Closed::Stream });
        component.events.push(Event::Ended);
        component.drain(&mut above, &mut below);
        assert!(above.is_empty(), "duplicate closure cannot repeat the end");
    }

    #[test]
    fn a_concrete_turn_and_result_reach_the_host_as_equal_values() {
        let mut component = component();
        let source = smith_host_world::turn_value(512).into_value();
        let bytes = smith_protocol_channel::encode_turn(&source, &component.transcript, &component.endpoints)
            .expect("turn codec");
        let mut above = Queue::with_capacity(8);
        let mut below = Queue::with_capacity(16);
        component.read(&mut above, &mut below);
        component.events.push(Event::Body { kind: 0x0109, body: turn_body(bytes) });
        component.drain(&mut above, &mut below);
        let Some(OpenEvent::Turn { turn }) = above.pop() else { panic!("typed turn") };
        assert_eq!(turn.body.into_value(), source);
        let source = smith_host_world::declared_value(32).into_value();
        let bytes = smith_protocol_channel::encode_result(&source, &component.charter).expect("result codec");
        component.read(&mut above, &mut below);
        component.events.push(Event::Body { kind: 0x0110, body: accepted_body(bytes) });
        component.drain(&mut above, &mut below);
        let Some(OpenEvent::Answer { answer, .. }) = above.pop() else { panic!("typed result") };
        let smith_host_domain::RunResult::Accepted { outcome } = answer.result else { panic!("accepted result") };
        assert_eq!(outcome.into_value(), source);
    }
}

#[cfg(test)]
#[path = "component/tests.rs"]
mod grant_tests;
