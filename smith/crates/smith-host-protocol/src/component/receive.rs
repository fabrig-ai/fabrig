//! Checked record dispatch (protocol/channel.md, sections 3–4).
//! The channel state admits a record before its handler changes the call,
//! turn, live-piece or long-operation ledger. No host policy is interpreted.

use super::{CallKey, Component, OpenEvent, State};
use crate::Error;
use alloc::boxed::Box;
use skein_lib::{Reader, Token};
use smith_host_domain::channel;

const ADMITTED: u16 = 0x0106;
const CALL: u16 = 0x0107;
const WITHDRAW: u16 = 0x0108;
const TURN: u16 = 0x0109;
const WAITING: u16 = 0x010a;
const LONG: u16 = 0x010b;
const LONG_DONE: u16 = 0x010c;
const UNAUTHORIZED: u16 = 0x010d;
const LIMITED: u16 = 0x010e;
const FACT: u16 = 0x010f;
const ANSWER: u16 = 0x0110;
const MESSAGE_REFUSED: u16 = 0x0111;
const LIVE: u16 = 0x0112;
const ACCOUNT_REFUSED: u16 = 0x0113;

/// What a refusal says of a record it could not take: its order, its call, or its kind.
pub(super) fn refusal_text(kind: u16, error: Error) -> &'static [u8] {
    match error {
        Error::Order => return b"record out of order",
        Error::Calls => return b"duplicate call or call capacity exhausted",
        Error::Codec(_)
        | Error::Channel(_)
        | Error::MissingValue
        | Error::Grants
        | Error::Body(_)
        | Error::Frame(_)
        | Error::Write(_)
        | Error::InvalidAnswer
        | Error::Value => {}
    }
    match kind {
        ADMITTED => b"invalid admission",
        CALL => b"invalid call",
        WITHDRAW => b"invalid withdrawal",
        TURN => b"invalid turn",
        WAITING => b"invalid waiting record",
        LONG | LONG_DONE => b"invalid long operation",
        UNAUTHORIZED | LIMITED | ACCOUNT_REFUSED => b"invalid account notice",
        FACT => b"invalid fact",
        ANSWER => b"invalid answer",
        MESSAGE_REFUSED => b"invalid message refusal",
        LIVE => b"invalid live piece",
        _ => b"unknown record",
    }
}

impl Component {
    pub(super) fn receive(&mut self, kind: u16, body: Box<[u8]>) -> Result<Option<OpenEvent>, Error> {
        let event = match kind {
            ADMITTED => self.admitted(&body)?,
            ANSWER => self.answer(&body)?,
            _ => {
                if self.state != State::Admitted {
                    return Err(Error::Order);
                }
                match kind {
                    CALL => self.call(&body)?,
                    WITHDRAW => return self.withdraw(&body),
                    TURN => self.turn(&body)?,
                    WAITING => self.waiting(&body)?,
                    LONG => self.long(&body)?,
                    LONG_DONE => self.long_done(&body)?,
                    UNAUTHORIZED | LIMITED | ACCOUNT_REFUSED => super::notices::decode(&self.bodies, kind, &body)?,
                    FACT => {
                        match smith_channel::Fact::decode(&self.bodies, &mut Reader::new(&body)) {
                            Ok(_) => (),
                            Err(problem) => return Err(Error::Body(problem)),
                        }
                        OpenEvent::Fact { body }
                    }
                    MESSAGE_REFUSED => self.message_refused(&body)?,
                    LIVE => self.live(&body)?,
                    _ => return Err(Error::Order),
                }
            }
        };
        Ok(Some(event))
    }

    fn admitted(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        if self.state != State::Started {
            return Err(Error::Order);
        }
        match smith_channel::Admitted::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(_) => (),
            Err(problem) => return Err(Error::Body(problem)),
        }
        self.state = State::Admitted;
        Ok(OpenEvent::Admitted)
    }

    fn answer(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        let record = match smith_channel::Answer::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let permitted = match record.result() {
            smith_channel::RunResult::Refused(_) => self.state == State::Started,
            smith_channel::RunResult::Accepted(_)
            | smith_channel::RunResult::Parked
            | smith_channel::RunResult::Failed(_) => self.state == State::Admitted,
        };
        if !permitted {
            return Err(Error::Order);
        }
        let answer = crate::answer::decode_answer(&record, &self.charter)?;
        self.state = State::Answered;
        Ok(OpenEvent::Answer { answer, record })
    }

    fn call(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        let record = match smith_channel::Call::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let key = CallKey::from_wire(record.name());
        if self.calls_by_name.contains_key(&key) || self.calls_by_name.len() >= self.calls_by_name.capacity() {
            return Err(Error::Calls);
        }
        let ask = crate::translate::decode_ask(&record)?.ok_or(Error::InvalidAnswer)?;
        let call = Token::new(self.next_call);
        self.next_call = self.next_call.checked_add(1).ok_or(Error::Calls)?;
        if self.calls_by_name.insert(key, call).is_err() {
            return Err(Error::Calls);
        }
        if self.calls_by_token.insert(call, key).is_err() {
            return Err(Error::Calls);
        }
        Ok(OpenEvent::Call { call, name: key.domain(), deadline: self.now.saturating_add(record.deadline()), ask })
    }

    fn withdraw(&self, body: &[u8]) -> Result<Option<OpenEvent>, Error> {
        let record = match smith_channel::Withdraw::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let key = CallKey::from_wire(record.name());
        Ok(match self.calls_by_name.get(&key) {
            Some(call) => Some(OpenEvent::Withdraw { call: *call }),
            // Crossing the host's answer settles the withdrawal without spending its read.
            None => None,
        })
    }

    fn waiting(&self, body: &[u8]) -> Result<OpenEvent, Error> {
        let record = match smith_channel::Waiting::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let read = match record.last_read() {
            Some(name) => Some(Token::new(*name)),
            None => None,
        };
        Ok(OpenEvent::Waiting { read })
    }

    fn turn(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        let turn = match smith_channel::Turn::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let sequence = self.sequence.checked_add(1).ok_or(Error::Value)?;
        let Ok(value) = smith_protocol_channel::decode_turn(turn.body(), sequence, &self.transcript, &self.endpoints)
        else {
            return Err(Error::Value);
        };
        self.sequence = value.sequence;
        self.live.turn(turn.number());
        let body =
            smith_host_domain::TurnValue::new(value, u64::MAX).expect("codec bounds ensure representable ownership");
        Ok(OpenEvent::Turn {
            turn: channel::Turn {
                number: turn.number(),
                opens_window: turn.opens_window(),
                spent: turn.spent(),
                read: match turn.last_read() {
                    Some(name) => Some(Token::new(*name)),
                    None => None,
                },
                body,
            },
        })
    }

    fn long(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        let record = match smith_channel::Long::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let operation = match record.operation() {
            smith_channel::Operation::Check => smith_domain::Operation::Checks,
            smith_channel::Operation::Completion => smith_domain::Operation::Completion,
            smith_channel::Operation::Command => smith_domain::Operation::Command,
        };
        self.long_open = self.long_open.checked_add(1).ok_or(Error::Order)?;
        Ok(OpenEvent::Long { operation, span: record.span() })
    }

    fn long_done(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        match smith_channel::LongDone::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(_) => (),
            Err(problem) => return Err(Error::Body(problem)),
        }
        self.long_open = self.long_open.checked_sub(1).ok_or(Error::Order)?;
        Ok(OpenEvent::LongDone)
    }

    fn message_refused(&self, body: &[u8]) -> Result<OpenEvent, Error> {
        let record = match smith_channel::MessageRefused::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        let reason = match record.reason() {
            smith_channel::MessageRefusal::TooLarge => smith_host_domain::MessageRefusal::TooLarge,
            smith_channel::MessageRefusal::Full => smith_host_domain::MessageRefusal::Full,
            smith_channel::MessageRefusal::NameInUse => smith_host_domain::MessageRefusal::NameInUse,
            smith_channel::MessageRefusal::Ending => smith_host_domain::MessageRefusal::Ending,
        };
        Ok(OpenEvent::MessageRefused { name: Token::new(record.name()), reason })
    }

    fn live(&mut self, body: &[u8]) -> Result<OpenEvent, Error> {
        let piece = match smith_channel::Live::decode(&self.bodies, &mut Reader::new(body)) {
            Ok(record) => record,
            Err(problem) => return Err(Error::Body(problem)),
        };
        if self.live.check(&piece).is_err() {
            return Err(Error::Order);
        }
        Ok(OpenEvent::Live {
            turn: piece.turn(),
            retry: piece.retry(),
            kind: piece.kind().clone(),
            offset: piece.offset(),
            bytes: Box::from(piece.bytes()),
            call: piece.call().clone(),
        })
    }
}
