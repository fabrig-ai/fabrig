//! Turns and parts retained for the engine until it acknowledges them (domain/hosts/hub.md,
//! section 4). The root handles link retries when needed; the host owns the
//! bytes and the credit bound that pauses an agent reader.

use skein_lib::bytes::copy_of;
use skein_lib::{Map, Token, length::len_u64};

use crate::boundary::Turn;
use crate::{LinkUp, Request, ToAgent};
use jig_domain_link::Part;

#[derive(Debug)]
pub(crate) enum Record {
    Turn(Turn),
    Part(Part),
}

impl Record {
    fn bytes(&self) -> u64 {
        match self {
            Self::Turn(turn) => len(&turn.body),
            Self::Part(part) => len(&part.body),
        }
    }

    fn up(&self, run: Token, attempt: Token) -> LinkUp {
        match self {
            Self::Turn(turn) => LinkUp::Turn { run, attempt, turn: copy(turn) },
            Self::Part(part) => LinkUp::Part { run, attempt, part: part.clone() },
        }
    }
}
use crate::limits::Limits;

/// A turn's name on the engine link.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Name {
    pub(crate) run: Token,
    pub(crate) attempt: Token,
    pub(crate) order: u64,
}

/// A turn staged from the agent or admitted into the hub's retention window.
#[derive(Debug)]
pub(crate) struct Pending {
    agent: Token,
    record: Record,
    admitted: bool,
}

/// The bounded turns retained across a channel loss and an ended run.
#[derive(Debug)]
pub(crate) struct Turns {
    pending: Map<Name, Pending>,
    abandoned: u64,
    next: u64,
}

impl Turns {
    pub(crate) fn new(limits: &Limits) -> Turns {
        let capacity = limits
            .slots
            .checked_mul(limits.turns)
            .expect("checked per-run capacity")
            .checked_mul(2)
            .expect("valid_hub accepted turn capacity");
        Turns { pending: Map::with_capacity(capacity), abandoned: 0, next: 0 }
    }

    pub(crate) fn len(&self) -> u32 {
        {
            let mut count = 0_u32;
            for (_, pending) in &self.pending {
                if pending.admitted {
                    count = count.checked_add(1).expect("bounded turns");
                }
            }
            count
        }
    }

    pub(crate) const fn abandoned(&self) -> u64 {
        self.abandoned
    }

    /// Reserve one whole next turn before acknowledging it to the agent.
    pub(crate) fn credit(&self, run: Token, attempt: Token, limits: &Limits) -> bool {
        let mut count = 0_u32;
        let mut bytes = 0_u64;
        for (name, pending) in &self.pending {
            if name.run == run && name.attempt == attempt && pending.admitted {
                count = count.checked_add(1).expect("bounded turns");
                bytes = bytes.checked_add(pending.record.bytes()).expect("bounded retained bytes");
            }
        }
        count < limits.turns
            && match bytes.checked_add(limits.turn_bytes) {
                Some(next) => next <= limits.turn_queue_bytes,
                None => false,
            }
    }

    /// Stage a turn until normal retention room is free; stopping takes it at once.
    pub(crate) fn retain(&mut self, agent: Token, run: Token, attempt: Token, record: Record) {
        let name = Name { run, attempt, order: self.next };
        self.next = self.next.checked_add(1).expect("record order fits the process lifetime");
        let old = self
            .pending
            .insert(name, Pending { agent, record, admitted: false })
            .expect("retention plus one agent window was reserved");
        assert!(old.is_none(), "consecutive turn names do not repeat");
    }

    /// Take staged turns in order, acknowledging the agent once the hub holds them.
    pub(crate) fn admit(
        &mut self,
        run: Token,
        attempt: Token,
        stopping: bool,
        limits: &Limits,
        out: &mut skein_lib::Queue<Request>,
    ) {
        for _ in 0..self.pending.capacity() {
            if !stopping && !self.credit(run, attempt, limits) {
                break;
            }
            let mut next = None;
            for (name, pending) in &self.pending {
                if name.run == run && name.attempt == attempt && !pending.admitted {
                    next = Some(*name);
                    break;
                }
            }
            let Some(name) = next else {
                break;
            };
            let pending = self.pending.get_mut(&name).expect("staged turn exists");
            pending.admitted = true;
            out.push(Request::Up(pending.record.up(run, attempt)));
            let ack = match &pending.record {
                Record::Turn(turn) => ToAgent::Acknowledge { agent: pending.agent, turn: turn.turn },
                Record::Part(part) => ToAgent::AcknowledgePart { agent: pending.agent, number: part.number },
            };
            out.push(Request::Agent(ack));
        }
    }

    /// Drop a turn the engine committed.
    pub(crate) fn acknowledge(&mut self, run: Token, attempt: Token, turn: u32) {
        let mut found = None;
        for (name, pending) in &self.pending {
            if name.run == run && name.attempt == attempt {
                match &pending.record {
                    Record::Turn(value) if value.turn == turn => {
                        found = Some(*name);
                        break;
                    }
                    Record::Turn(_) | Record::Part(_) => {}
                }
            }
        }
        if let Some(name) = found {
            self.pending.remove(&name);
        }
    }

    pub(crate) fn holds(&self, run: Token, attempt: Token, turn: u32) -> bool {
        for (name, pending) in &self.pending {
            if name.run == run && name.attempt == attempt {
                match &pending.record {
                    Record::Turn(value) if value.turn == turn => return true,
                    Record::Turn(_) | Record::Part(_) => {}
                }
            }
        }
        false
    }

    pub(crate) fn has_run(&self, run: Token, attempt: Token) -> bool {
        for (name, _) in &self.pending {
            if name.run == run && name.attempt == attempt {
                return true;
            }
        }
        false
    }

    /// A copy for one transmission; the retained bytes remain until ACK.
    pub(crate) fn get(&self, run: Token, attempt: Token, turn: u32) -> Option<Turn> {
        for (name, pending) in &self.pending {
            if name.run == run && name.attempt == attempt && pending.admitted {
                match &pending.record {
                    Record::Turn(value) if value.turn == turn => return Some(copy(value)),
                    Record::Turn(_) | Record::Part(_) => {}
                }
            }
        }
        None
    }

    /// A copy of the `index`th retained turn, in name order, for rehello.
    pub(crate) fn nth(&self, index: u32) -> Option<LinkUp> {
        let mut at = 0_u32;
        for (name, pending) in &self.pending {
            if !pending.admitted {
                continue;
            }
            if at == index {
                return Some(pending.record.up(name.run, name.attempt));
            }
            at = at.checked_add(1).expect("bounded turns");
        }
        None
    }

    /// A part acknowledgement covers every earlier part in its attempt.
    pub(crate) fn acknowledge_part(&mut self, run: Token, attempt: Token, number: u32) {
        for _ in 0..self.pending.capacity() {
            let mut found = None;
            for (name, pending) in &self.pending {
                if name.run == run && name.attempt == attempt {
                    match &pending.record {
                        Record::Part(part) if part.number <= number => {
                            found = Some(*name);
                            break;
                        }
                        Record::Turn(_) | Record::Part(_) => {}
                    }
                }
            }
            let Some(name) = found else { break };
            self.pending.remove(&name);
        }
    }

    pub(crate) fn part(&self, run: Token, attempt: Token, number: u32) -> Option<Part> {
        for (name, pending) in &self.pending {
            if name.run == run && name.attempt == attempt && pending.admitted {
                match &pending.record {
                    Record::Part(part) if part.number == number => return Some(part.clone()),
                    Record::Turn(_) | Record::Part(_) => {}
                }
            }
        }
        None
    }

    /// Count each turn or part abandoned after shutdown and a lost channel.
    pub(crate) fn give_up(&mut self) {
        for _ in 0..self.pending.capacity() {
            let Some((name, _)) = self.pending.first() else {
                break;
            };
            let name = *name;
            self.pending.remove(&name);
            self.abandoned = self.abandoned.saturating_add(1);
        }
    }
}

fn copy(turn: &Turn) -> Turn {
    Turn {
        turn: turn.turn,
        spent: turn.spent,
        read: turn.read,
        opens_window: turn.opens_window,
        refused: turn.refused.clone(),
        body: copy_of(&turn.body),
    }
}

fn len(bytes: &[u8]) -> u64 {
    len_u64(bytes.len())
}
