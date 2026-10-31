//! Admission and answer lifecycle (smith/docs/design/protocol/channel.md, sections 2–5 and 7).
use super::cancellation::Cancellation;
use super::turns::Turns;
use crate::Error;
use core::mem;
use skein_channel::{Event, Lower, Machine, Request};
use skein_lib::Queue;

#[derive(Debug)]
pub(super) enum State {
    Opening,
    Opened,
    Started(Active),
    Admitted(Active),
    Answered(Turns),
    Refused,
}

#[derive(Debug)]
pub(super) struct Active {
    pub(super) turns: Turns,
    pub(super) live: Option<smith_channel::LiveAsk>,
    pub(super) cancellation: Cancellation,
    pub(super) long_open: u32,
}

impl State {
    pub(super) fn started(&self) -> bool {
        match self {
            State::Started(_) => true,
            State::Opening | State::Opened | State::Admitted(_) | State::Answered(_) | State::Refused => false,
        }
    }

    pub(super) fn admitted(&self) -> bool {
        match self {
            State::Admitted(_) => true,
            State::Opening | State::Opened | State::Started(_) | State::Answered(_) | State::Refused => false,
        }
    }

    pub(super) fn opened(&self) -> bool {
        match self {
            State::Opening => false,
            State::Opened | State::Started(_) | State::Admitted(_) | State::Answered(_) | State::Refused => true,
        }
    }

    pub(super) fn active(&self) -> Result<&Active, Error> {
        match self {
            State::Started(active) | State::Admitted(active) => Ok(active),
            State::Opening | State::Opened | State::Answered(_) | State::Refused => Err(Error::Order),
        }
    }

    pub(super) fn active_mut(&mut self) -> Result<&mut Active, Error> {
        match self {
            State::Started(active) | State::Admitted(active) => Ok(active),
            State::Opening | State::Opened | State::Answered(_) | State::Refused => Err(Error::Order),
        }
    }

    pub(super) fn next_turn(&self) -> Option<u32> {
        match self {
            State::Started(active) | State::Admitted(active) => active.turns.next(),
            State::Answered(turns) => turns.next(),
            State::Opening | State::Opened | State::Refused => None,
        }
    }

    pub(super) fn turns_mut(&mut self) -> Result<&mut Turns, Error> {
        match self {
            State::Started(active) | State::Admitted(active) => Ok(&mut active.turns),
            State::Answered(turns) => Ok(turns),
            State::Opening | State::Opened | State::Refused => Err(Error::Order),
        }
    }

    pub(super) fn admit(&mut self) {
        let state = mem::replace(self, State::Refused);
        *self = match state {
            State::Started(active) => State::Admitted(active),
            State::Opening | State::Opened | State::Admitted(_) | State::Answered(_) | State::Refused => {
                unreachable!("admission checked before its frame is built")
            }
        };
    }

    pub(super) fn answer(&mut self) {
        let state = mem::replace(self, State::Refused);
        *self = match state {
            State::Started(active) | State::Admitted(active) => State::Answered(active.turns),
            State::Opening | State::Opened | State::Answered(_) | State::Refused => {
                unreachable!("answer order checked before its frame is built")
            }
        };
    }

    pub(super) fn follow(&self, machine: &mut Machine, events: &mut Queue<Event>, below: &mut Queue<Lower>) {
        match self {
            State::Opened | State::Started(_) | State::Admitted(_) | State::Answered(_) => {
                machine.down(Request::Read, events, below);
            }
            State::Opening | State::Refused => {}
        }
    }
}
