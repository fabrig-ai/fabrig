//! Agent-side receive transitions (protocol/channel.md, sections 2–5 and 7).
use super::cancellation::Cancellation;
use super::state::{Active, State};
use super::turns::Turns;
use super::{CallKey, CallRoute, Component, invalid_answer, kind_text};
use crate::{Error, OpenEvent, calls};
use alloc::boxed::Box;
use core::mem;
use skein_channel::{Closed, Event, Lower, Request, Role};
use skein_lib::{Queue, Reader, Time, Token};
use smith_channel::kind;
use smith_domain::run;
use smith_domain_session::record;
use smith_protocol_translate::{decode_charter, decode_transcript, invalid_start, saved_delivery, start_context};

struct Broken;

impl Component {
    pub(super) fn follow(&mut self, below: &mut Queue<Lower>) {
        self.state.follow(&mut self.machine, &mut self.events, below);
    }

    pub(super) fn drain(&mut self, to_service: &mut Queue<OpenEvent>, below: &mut Queue<Lower>) {
        // Eight child transitions bound opening/terms, one demanded record or
        // refusal, and the resulting output and close notifications.
        for _ in 0_u8..8_u8 {
            let Some(event) = self.events.pop() else { break };
            match event {
                Event::Opening { credential, lowest: _, highest } => {
                    if credential.is_empty() {
                        self.machine.down(
                            Request::Accept { version: highest.min(kind::VERSION) },
                            &mut self.events,
                            below,
                        );
                    } else {
                        self.machine.down(
                            Request::Refuse {
                                reason: smith_channel::reason::UNAUTHORIZED,
                                text: Box::from(*b"credential on pipe"),
                            },
                            &mut self.events,
                            below,
                        );
                    }
                }
                Event::Ready { version, terms } => {
                    match smith_channel::peer_terms_gap(&self.schema, Role::Responder, version, &terms) {
                        Some(kind) => self.machine.down(
                            Request::Refuse { reason: smith_channel::reason::LIMITS, text: kind_text(kind) },
                            &mut self.events,
                            below,
                        ),
                        None => {
                            let state = mem::replace(&mut self.state, State::Refused);
                            self.state = match state {
                                State::Opening => State::Opened,
                                State::Opened
                                | State::Started(_)
                                | State::Admitted(_)
                                | State::Answered(_)
                                | State::Refused => unreachable!("the channel becomes ready once"),
                            };
                            to_service.push(OpenEvent::Opened { version });
                            self.follow(below);
                        }
                    }
                }
                Event::Body { kind, body } => {
                    let state = mem::replace(&mut self.state, State::Refused);
                    self.state = match self.receive(state, kind, &body, to_service, below) {
                        Ok(state) => state,
                        Err(Broken) => {
                            self.refuse_rules(below);
                            State::Refused
                        }
                    };
                    self.follow(below);
                }
                Event::Closed { why } => self.ending.end(why, to_service),
                Event::Ended => self.ending.end(Closed::Stream, to_service),
                Event::Unsent { token, why: _ } => self.unsent(token, to_service),
                Event::OutputFailed => to_service.push(OpenEvent::WriteFailed),
                Event::Ping
                | Event::Unsupported { .. }
                | Event::Refused { .. }
                | Event::Sent { .. }
                | Event::Drained => {}
            }
        }
    }

    fn receive(
        &mut self,
        state: State,
        kind: u16,
        body: &[u8],
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<State, Broken> {
        match state {
            State::Opening | State::Refused => Err(Broken),
            State::Opened => {
                if kind != kind::START {
                    return Err(Broken);
                }
                self.start(body, to_service, below)
            }
            State::Started(mut active) => {
                match kind {
                    kind::MESSAGE => self.message(body, to_service)?,
                    kind::CANCEL => cancel(&mut active.cancellation, body, &self.bodies, to_service)?,
                    _ => return Err(Broken),
                }
                Ok(State::Started(active))
            }
            State::Admitted(mut active) => {
                match kind {
                    kind::MESSAGE => self.message(body, to_service)?,
                    kind::HOST_ANSWER => self.host_answer(body, to_service)?,
                    kind::ACKNOWLEDGE => acknowledge(&mut active.turns, body, &self.bodies, to_service)?,
                    kind::GRANT_REFRESH => self.refresh(body, to_service)?,
                    kind::CANCEL => cancel(&mut active.cancellation, body, &self.bodies, to_service)?,
                    _ => return Err(Broken),
                }
                Ok(State::Admitted(active))
            }
            State::Answered(mut turns) => {
                if kind != kind::ACKNOWLEDGE {
                    return Err(Broken);
                }
                acknowledge(&mut turns, body, &self.bodies, to_service)?;
                Ok(State::Answered(turns))
            }
        }
    }

    fn start(
        &mut self,
        body: &[u8],
        to_service: &mut Queue<OpenEvent>,
        below: &mut Queue<Lower>,
    ) -> Result<State, Broken> {
        let start = smith_channel::Start::decode(&self.bodies, &mut Reader::new(body)).or(Err(Broken))?;
        let charter = match decode_charter(start.charter(), &self.charter, &self.endpoints) {
            Ok(charter) => charter,
            Err(invalid) => {
                let reason = invalid_start(invalid, &self.bodies).or(Err(Broken))?;
                let frame = invalid_answer(&self.bodies, reason).ok_or(Broken)?;
                self.machine.down(Request::Send { token: Token::new(0), frame }, &mut self.events, below);
                self.machine.down(Request::Finish, &mut self.events, below);
                return Ok(State::Refused);
            }
        };
        let transcript = match decode_transcript(
            start.transcript(),
            self.resume_turns,
            self.search_hits,
            &self.transcript,
            &self.endpoints,
        ) {
            Ok(transcript) => transcript,
            Err(reason) => {
                self.transcript_refused(reason, below).or(Err(Broken))?;
                return Ok(State::Refused);
            }
        };
        self.stash_initial_grants(&start).or(Err(Broken))?;
        let mut start = start_context(start, charter, transcript).or(Err(Broken))?;
        start.window.turns = start.window.turns.min(self.turn_capacity);
        let active = Active {
            turns: Turns::new(start.window, self.turn_capacity),
            live: start.live.clone(),
            cancellation: Cancellation::Waiting,
            long_open: 0,
        };
        to_service.push(OpenEvent::Start { start: Box::new(start) });
        Ok(State::Started(active))
    }

    fn message(&self, body: &[u8], to_service: &mut Queue<OpenEvent>) -> Result<(), Broken> {
        let message = smith_channel::Message::decode(&self.bodies, &mut Reader::new(body)).or(Err(Broken))?;
        to_service.push(OpenEvent::Message {
            finish_requested: message.finish_requested(),
            name: Token::new(message.name()),
            label: Box::from(message.label()),
            text: Box::from(message.text()),
        });
        Ok(())
    }

    fn host_answer(&mut self, body: &[u8], to_service: &mut Queue<OpenEvent>) -> Result<(), Broken> {
        let answer = smith_channel::HostAnswer::decode(&self.bodies, &mut Reader::new(body)).or(Err(Broken))?;
        let key = CallKey::from_wire(answer.name());
        match self.calls.remove(&key).ok_or(Broken)? {
            CallRoute::Host { relay, send: _ } => {
                let reply = calls::host_reply(answer.reply()).ok_or(Broken)?;
                to_service.push(OpenEvent::HostReturned { relay, reply });
            }
            CallRoute::Delivery { owner, send: _ } => match answer.reply() {
                smith_channel::Reply::Delivery(delivery) => {
                    let delivery = saved_delivery(delivery.value()).or(Err(Broken))?;
                    to_service.push(OpenEvent::Delivered { owner, delivery: Box::new(delivery) });
                }
                smith_channel::Reply::Host(_)
                | smith_channel::Reply::Busy
                | smith_channel::Reply::Unavailable
                | smith_channel::Reply::Withdrawn
                | smith_channel::Reply::TooLarge => return Err(Broken),
            },
        }
        Ok(())
    }

    fn refresh(&mut self, body: &[u8], to_service: &mut Queue<OpenEvent>) -> Result<(), Broken> {
        let refresh = smith_channel::GrantRefresh::decode(&self.bodies, &mut Reader::new(body)).or(Err(Broken))?;
        let grant = refresh.grant();
        let value = crate::GrantValue::from_wire(grant.value()).or(Err(Broken))?;
        self.credentials.keep(grant.account(), grant.generation(), value).or(Err(Broken))?;
        to_service.push(OpenEvent::Grant { grant: smith_protocol_translate::refreshed_grant(grant) });
        Ok(())
    }

    fn unsent(&mut self, token: Token, to_service: &mut Queue<OpenEvent>) {
        let mut found = None;
        for (key, route) in &self.calls {
            let send = match route {
                CallRoute::Host { send, .. } | CallRoute::Delivery { send, .. } => *send,
            };
            if send == token {
                found = Some(*key);
            }
        }
        if let Some(key) = found {
            match self.calls.remove(&key) {
                Some(CallRoute::Host { relay, send: _ }) => to_service
                    .push(OpenEvent::HostReturned { relay, reply: run::HostReply::Unanswered(run::Unanswered::Lost) }),
                Some(CallRoute::Delivery { owner, send: _ }) => to_service.push(OpenEvent::DeliveryUnsent { owner }),
                None => {}
            }
        }
    }

    fn transcript_refused(&mut self, reason: record::Refusal, below: &mut Queue<Lower>) -> Result<(), Error> {
        let reason = match reason {
            record::Refusal::Version => run::TranscriptRefusal::Version,
            record::Refusal::Malformed => run::TranscriptRefusal::Malformed,
            record::Refusal::Unresolved => run::TranscriptRefusal::Unresolved,
            record::Refusal::TooLarge => run::TranscriptRefusal::TooLarge,
        };
        let admitted =
            smith_channel::Admitted::new(&self.bodies, smith_channel::AdmittedParts {}).map_err(Error::ChannelBody)?;
        let admitted = smith_channel::frame(smith_channel::Body::Admitted(admitted)).map_err(Error::Frame)?;
        self.machine.down(Request::Send { token: Token::new(0), frame: admitted }, &mut self.events, below);
        let answer = smith_protocol_translate::answer_record(
            Time::ZERO,
            run::Answer::Failed { failure: run::Failure::Transcript(reason), spent: run::Spend::ZERO, turns: 0 },
            None,
            &self.bodies,
            &self.charter,
        )?;
        let frame = smith_channel::frame(smith_channel::Body::Answer(answer)).map_err(Error::Frame)?;
        self.machine.down(Request::Send { token: Token::new(0), frame }, &mut self.events, below);
        self.machine.down(Request::Finish, &mut self.events, below);
        Ok(())
    }
}

fn cancel(
    cancellation: &mut Cancellation,
    body: &[u8],
    limits: &smith_channel::Limits,
    to_service: &mut Queue<OpenEvent>,
) -> Result<(), Broken> {
    smith_channel::Cancel::decode(limits, &mut Reader::new(body)).or(Err(Broken))?;
    cancellation.request().or(Err(Broken))?;
    to_service.push(OpenEvent::Cancel);
    Ok(())
}

fn acknowledge(
    turns: &mut Turns,
    body: &[u8],
    limits: &smith_channel::Limits,
    to_service: &mut Queue<OpenEvent>,
) -> Result<(), Broken> {
    let acknowledge = smith_channel::Acknowledge::decode(limits, &mut Reader::new(body)).or(Err(Broken))?;
    turns.acknowledge(acknowledge.number()).or(Err(Broken))?;
    to_service.push(OpenEvent::Acknowledged { turn: acknowledge.number() });
    Ok(())
}
