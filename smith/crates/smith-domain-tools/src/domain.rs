//! The tools child domain's state and its entry point (smith/docs/design/domain/tools.md,
//! sections 3–6; skein/docs/foundation/programming-model.md, sections 4.5 and 6.3). [`Domain`]
//! retains kits, pending IO flights and uncapped facts. [`step`] emits the kit's
//! requests into an uncapped queue and accepts its typed terminals. Facts are
//! kept losslessly and drained by count. This module never knows
//! provider text, run outcomes, credentials or filesystem contents beyond
//! what typed IO terminals supply.

use skein_lib::{Env, Queue, Slab};

use crate::boundary::{Event, Request};
use crate::facts::{Fact, Facts};
use crate::flight::{self, Flight};
use crate::kit::{self, Kit};
use crate::limits::Limits;

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

/// Handles one event, emitting requests into an uncapped queue.
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
        Event::ReadVersion { kit, place, version } => {
            if let Some(kit) = domain.kits.get_mut(skein_lib::Id::from_token(kit)) {
                kit.knowledge.record(place, version);
            }
        }
    }
}
