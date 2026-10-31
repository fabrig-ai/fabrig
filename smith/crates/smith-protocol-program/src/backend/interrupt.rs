//! One program control request (protocol/programs.md, 5.7 and 5.9).
//! Keeps its process-local sequence, unsent line and acknowledgement together.
//! A result may precede the acknowledgement; neither consumes the other's data.
//! `prepare`, `offer`, `send` and `acknowledge` own the wire transitions.

use alloc::boxed::Box;
use core::mem;
use skein_io as io;
use skein_lib::{Queue, Token, stream};

use super::{INTERRUPT_RIGHT, stream as records};

pub(super) struct Interrupt {
    state: State,
}

enum State {
    Idle { sequence: u64 },
    Prepared { sequence: u64, line: Box<[u8]> },
    Offering { sequence: u64, line: Box<[u8]>, pipe: Token },
    Offered { sequence: u64, line: Box<[u8]> },
    Sent { sequence: u64 },
    PreparedAcknowledged { sequence: u64, line: Box<[u8]> },
    OfferingAcknowledged { sequence: u64, line: Box<[u8]>, pipe: Token },
    OfferedAcknowledged { sequence: u64, line: Box<[u8]> },
}

impl Interrupt {
    pub fn new() -> Self {
        Self { state: State::Idle { sequence: 0 } }
    }

    pub fn sequence(&self) -> u64 {
        match self.state {
            State::Idle { sequence }
            | State::Prepared { sequence, .. }
            | State::Offering { sequence, .. }
            | State::Offered { sequence, .. }
            | State::Sent { sequence }
            | State::PreparedAcknowledged { sequence, .. }
            | State::OfferingAcknowledged { sequence, .. }
            | State::OfferedAcknowledged { sequence, .. } => sequence,
        }
    }

    pub fn restart(&mut self, below: &mut Queue<io::Request>) {
        self.clear(0, below);
    }

    pub fn forget(&mut self, below: &mut Queue<io::Request>) {
        self.clear(self.sequence(), below);
    }

    fn clear(&mut self, sequence: u64, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Idle { sequence: 0 });
        self.state = match state {
            State::Idle { .. }
            | State::Prepared { .. }
            | State::Offered { .. }
            | State::Sent { .. }
            | State::PreparedAcknowledged { .. }
            | State::OfferedAcknowledged { .. } => State::Idle { sequence },
            State::Offering { .. } | State::OfferingAcknowledged { .. } => {
                unreachable!("follow offers before another entrance")
            }
        };
        follow(self, below);
    }

    pub fn prepare(&mut self, maximum: u32, below: &mut Queue<io::Request>) -> Option<()> {
        let sequence = self.sequence().checked_add(1)?;
        let line = records::interrupt_request(sequence, maximum)?;
        let state = mem::replace(&mut self.state, State::Idle { sequence: 0 });
        self.state = match state {
            State::Idle { .. }
            | State::Prepared { .. }
            | State::Offered { .. }
            | State::Sent { .. }
            | State::PreparedAcknowledged { .. }
            | State::OfferedAcknowledged { .. } => prepared(sequence, line),
            State::Offering { .. } | State::OfferingAcknowledged { .. } => {
                unreachable!("follow offers before another entrance")
            }
        };
        follow(self, below);
        Some(())
    }

    pub fn offer(&mut self, pipe: Option<Token>, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Idle { sequence: 0 });
        self.state = match state {
            State::Prepared { sequence, line } => {
                State::Offering { sequence, line, pipe: pipe.expect("interrupt input") }
            }
            State::PreparedAcknowledged { sequence, line } => {
                State::OfferingAcknowledged { sequence, line, pipe: pipe.expect("interrupt input") }
            }
            state @ (State::Idle { .. }
            | State::Offered { .. }
            | State::Sent { .. }
            | State::OfferedAcknowledged { .. }) => state,
            State::Offering { .. } | State::OfferingAcknowledged { .. } => {
                unreachable!("follow offers before another entrance")
            }
        };
        follow(self, below);
    }

    pub fn send(&mut self, pipe: Token, below: &mut Queue<io::Request>) {
        let state = mem::replace(&mut self.state, State::Idle { sequence: 0 });
        self.state = match state {
            State::Prepared { sequence, line } | State::Offered { sequence, line } => sent(sequence, line, pipe, below),
            State::PreparedAcknowledged { sequence, line } | State::OfferedAcknowledged { sequence, line } => {
                send(line, pipe, below);
                State::Idle { sequence }
            }
            state @ (State::Idle { .. } | State::Sent { .. }) => state,
            State::Offering { .. } | State::OfferingAcknowledged { .. } => {
                unreachable!("follow offers before another entrance")
            }
        };
        follow(self, below);
    }

    pub fn acknowledge(&mut self, id: Option<&[u8]>, below: &mut Queue<io::Request>) -> bool {
        let sequence = match self.state {
            State::Prepared { sequence, .. } | State::Offered { sequence, .. } | State::Sent { sequence } => sequence,
            State::Idle { .. } | State::PreparedAcknowledged { .. } | State::OfferedAcknowledged { .. } => {
                return false;
            }
            State::Offering { .. } | State::OfferingAcknowledged { .. } => {
                unreachable!("follow offers before another entrance")
            }
        };
        if id != Some(records::interrupt_id(sequence).as_ref()) {
            return false;
        }
        let state = mem::replace(&mut self.state, State::Idle { sequence: 0 });
        self.state = match state {
            State::Prepared { sequence, line } => State::PreparedAcknowledged { sequence, line },
            State::Offered { sequence, line } => State::OfferedAcknowledged { sequence, line },
            State::Sent { sequence } => State::Idle { sequence },
            State::Idle { .. }
            | State::PreparedAcknowledged { .. }
            | State::OfferedAcknowledged { .. }
            | State::Offering { .. }
            | State::OfferingAcknowledged { .. } => unreachable!("a matching response consumes one pending request"),
        };
        follow(self, below);
        true
    }
}

fn prepared(sequence: u64, line: Box<[u8]>) -> State {
    State::Prepared { sequence, line }
}

fn sent(sequence: u64, line: Box<[u8]>, pipe: Token, below: &mut Queue<io::Request>) -> State {
    send(line, pipe, below);
    State::Sent { sequence }
}

fn send(bytes: Box<[u8]>, pipe: Token, below: &mut Queue<io::Request>) {
    below.push(io::Request::Output { stream: pipe, down: stream::OutputDown::Send { right: INTERRUPT_RIGHT, bytes } });
}

fn offer(pipe: Token, line: &[u8], below: &mut Queue<io::Request>) {
    below.push(io::Request::Output {
        stream: pipe,
        down: stream::OutputDown::Room {
            right: INTERRUPT_RIGHT,
            bytes: u32::try_from(line.len()).expect("bounded interrupt line"),
        },
    });
}

/// An offered line requests room once and stays owned until its grant arrives.
fn follow(interrupt: &mut Interrupt, below: &mut Queue<io::Request>) {
    let state = mem::replace(&mut interrupt.state, State::Idle { sequence: 0 });
    interrupt.state = match state {
        State::Offering { sequence, line, pipe } => {
            offer(pipe, &line, below);
            State::Offered { sequence, line }
        }
        State::OfferingAcknowledged { sequence, line, pipe } => {
            offer(pipe, &line, below);
            State::OfferedAcknowledged { sequence, line }
        }
        state @ (State::Idle { .. }
        | State::Prepared { .. }
        | State::Offered { .. }
        | State::Sent { .. }
        | State::PreparedAcknowledged { .. }
        | State::OfferedAcknowledged { .. }) => state,
    };
}

#[cfg(test)]
mod tests;
