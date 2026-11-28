//! Exact, content-free room refusals (client/domain/pages.md, section 7;
//! domain/tasks/README.md, section 6). Only the owner check which found no room
//! attaches a cause. The bounded live history changes neither admission nor
//! diagnostic draining, and retains no payload or reply right.
pub use jig_domain_contract::ops::{Capacity, CapacityAction, CapacityCause};

use crate::{Domain, Limits, New, Party, Problem, Refusal, Request};
use skein_lib::{Env, List, Queue, ReplyTo, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CapacityRefusal {
    pub request: Capacity,
    pub at: Wall,
}

impl Domain {
    /// Oldest to newest admitted facts; no diagnostic or admission state is drained.
    #[must_use]
    pub fn capacity_refusals(&self) -> List<CapacityRefusal> {
        let mut rows = List::with_capacity(self.room_refusals.capacity());
        for row in &self.room_refusals {
            rows.push(*row).expect("bounded capacity history snapshot");
        }
        rows
    }
    #[must_use]
    pub const fn capacity_refusals_lost(&self) -> u64 {
        self.room_refusals_lost
    }
    /// Content revision for the core's level-triggered deployment watch.
    #[must_use]
    pub const fn capacity_mark(&self) -> u64 {
        self.room_refusals_mark
    }
    /// Record an owner-proven refusal, also used by the core's projection-room check.
    pub fn remember_capacity(&mut self, at: Wall, request: Capacity) {
        self.room_refusals_mark = self.room_refusals_mark.saturating_add(1);
        if self.room_refusals.capacity() == 0 {
            self.room_refusals_lost = self.room_refusals_lost.saturating_add(1);
            return;
        }
        if self.room_refusals.room() == 0 {
            self.room_refusals.pop();
            self.room_refusals_lost = self.room_refusals_lost.saturating_add(1);
        }
        self.room_refusals.push(CapacityRefusal { request, at });
    }
}

pub(crate) fn refused(domain: &mut Domain, env: &Env<Limits>, to: ReplyTo, problem: Problem, out: &mut Queue<Request>) {
    if let Some(request) = problem.capacity {
        domain.remember_capacity(env.wall, request);
    }
    out.push(Request::Refused { reply_to: to, problem });
}

pub(crate) fn refuse(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    task: Option<u64>,
    why: Refusal,
    request: Capacity,
    out: &mut Queue<Request>,
) {
    refused(domain, env, to, Problem::capacity(task, why, request), out);
}

pub(crate) fn inbox(task: u64, limits: &Limits) -> CapacityCause {
    CapacityCause::Inbox { task, messages: limits.inbox_messages, bytes: limits.inbox_bytes }
}

pub(crate) fn batch_problem(
    task: Option<u64>,
    why: Refusal,
    by: Party,
    batch: &[New],
    cause: CapacityCause,
) -> Problem {
    Problem::capacity(task, why, Capacity::batch(by, batch, task, cause))
}
