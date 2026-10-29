//! The tools child domain's state and its entry point (domain/tools.md,
//! sections 3–6; programming-model.md, sections 4.5 and 6.3). [`Domain`]
//! retains kits, pending IO flights and uncapped facts. [`step`] emits the kit's
//! bounded requests and accepts its typed terminals. This module never knows
//! provider text, run outcomes, credentials or filesystem contents beyond
//! what typed IO terminals supply.

use skein_lib::{Env, Queue, Slab};

use crate::boundary::{Event, Request};
use crate::facts::{Fact, Facts};
use crate::flight::{self, Flight};
use crate::kit::{self, Kit};
use crate::limits::Limits;

/// The most requests one call of [`step`] emits under `limits`: closing a kit
/// cancels its ordinary calls and held owners. An ended job can invalidate
/// the reads queued behind its answer, then close its kit. The allowance covers
/// all calls plus two outputs per job, or two outputs when that is larger.
/// The session reads this compatibility allowance in its parent-facing formula.
#[must_use]
pub const fn max_out(limits: &Limits) -> u32 {
    let owners = limits.calls.saturating_add(limits.jobs.saturating_mul(2));
    if owners > 2 { owners } else { 2 }
}

/// Close can report every retained terminal beside its lifecycle facts.
#[must_use]
pub const fn max_facts(limits: &Limits) -> u32 {
    limits.jobs.saturating_add(2)
}

/// The tools child domain's state.
#[derive(Debug)]
pub struct Domain {
    pub(crate) kits: Slab<Kit>,
    pub(crate) flights: Slab<Flight>,
    pub(crate) facts: Facts,
}

impl Domain {
    /// A domain with room for `limits`.
    #[must_use]
    pub fn new(limits: &Limits) -> Domain {
        let flights = flight::slots(limits).expect("valid limits");
        Domain { kits: Slab::with_capacity(limits.kits), flights: Slab::with_capacity(flights), facts: Facts::new() }
    }

    /// Kits present, closed ones included until they are reclaimed.
    #[must_use]
    pub fn kits(&self) -> u32 {
        self.kits.len()
    }

    /// IO owners retained, including held jobs and slots awaiting reclaim.
    #[must_use]
    pub fn flights(&self) -> u32 {
        self.flights.len()
    }

    /// The session owning a live IO operation, for its parent's deadline notices.
    #[must_use]
    pub fn io_session(&self, owner: skein_lib::Token) -> Option<skein_lib::Token> {
        flight::io_session(self, owner)
    }

    /// The oldest observation queued for the parent.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// The observations queued for the parent.
    #[must_use]
    pub fn pending_facts(&self) -> u32 {
        self.facts.len()
    }

    /// The reclaim point: frees what closed in this iteration.
    pub fn reclaim(&mut self) {
        self.flights.reclaim();
        self.kits.reclaim();
    }
}

/// Handles one event, emitting at most [`max_out`] requests.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    domain.facts.begin(env.now);
    match event {
        Event::Open { session, jobs, authority } => kit::open(domain, env, session, jobs, authority, out),
        Event::Call { kit, reply_to, observation, call, deadline } => {
            kit::call(domain, env, kit, reply_to, observation, call, deadline, out);
        }
        Event::Close { kit } => kit::close(domain, kit, &env.limits, out),
        Event::CancelJobs { kit } => crate::jobs::cancel(domain, kit, &env.limits, out),
        Event::EndWait { kit, call } => crate::jobs::end_wait(domain, kit, call, out),
        Event::Done { owner, done } => flight::done(domain, env, owner, done, out),
    }
}
