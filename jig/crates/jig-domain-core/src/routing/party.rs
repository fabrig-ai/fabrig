//! Owned party routing values (domain/engine.md, section 4.4).
use super::Work;
use crate::{Request, compound::PartyTask};
use jig_domain_people as people;
use skein_lib::{Queue, Token};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Party {
    pub request: Token,
    pub person: u64,
    pub project: u32,
    pub role: Option<people::Role>,
}
impl Party {
    pub(crate) fn task(self, task: u64) -> PartyTask {
        PartyTask { person: self.person, project: self.project, role: self.role, task }
    }
}

pub(crate) struct Routed {
    pub work: Queue<Work>,
    pub out: Queue<Request>,
}
impl Routed {
    pub(crate) fn new(work: &mut Queue<Work>) -> Self {
        Self { work: core::mem::replace(work, Queue::uncapped()), out: Queue::uncapped() }
    }
    pub(crate) fn finish(self, work: &mut Queue<Work>) -> Queue<Request> {
        *work = self.work;
        self.out
    }
}

pub(super) struct Amendment {
    pub task: u64,
    pub revision: u64,
    pub change: people::Amendment,
    pub release: Option<u64>,
}

#[derive(Clone, Copy)]
pub(crate) enum Control {
    Stop,
    Cancel { reason_len: usize, grace: Option<skein_lib::Duration> },
    Release { again: Option<bool> },
}

pub(super) struct Proposal {
    pub proposer: people::Proposer,
    pub number: u64,
    pub decision: people::ProposalDecision,
    pub running: Option<people::Running>,
}
