//! The checkout child domain's state and its entry point (section 3).

use skein_lib::{Env, Id, Queue, Slab};

use crate::boundary::{Event, Request};
use crate::cache::Cache;
use crate::facts::{Fact, Facts};
use crate::hold::{self, Hold};
use crate::limits::{self, Limits};

/// The checkout child domain's state.
#[derive(Debug)]
pub struct Domain {
    pub(crate) cache: Cache,
    pub(crate) holds: Slab<Hold>,
    /// Preparing and store-waiting holds, each keeping one permit.
    pub(crate) active: u32,
    /// Holds waiting for a preparation permit, oldest first.
    pub(crate) queued: Queue<Id<Hold>>,
    pub(crate) facts: Facts,
}

impl Domain {
    /// A domain with room for `limits`.
    #[must_use]
    pub fn new(limits: &Limits) -> Domain {
        let holds = limits::holds(limits).expect("validated hold-slot limits");
        Domain {
            cache: Cache::with_capacity(limits.workspaces, limits::stores(limits).expect("validated store slots")),
            holds: Slab::with_capacity(holds),
            active: 0,
            queued: Queue::with_capacity(limits.workspaces),
            facts: Facts::with_capacity(limits.facts),
        }
    }

    /// Keep matching clones when an idle workspace is evicted.
    pub fn reuse_workspaces(&mut self) {
        self.cache.reuse_workspaces();
    }

    /// Whether eviction may keep matching clones.
    #[must_use]
    pub const fn reuses_workspaces(&self) -> bool {
        self.cache.reuses_workspaces()
    }

    /// Workspaces in the cache, held or idle.
    #[must_use]
    pub fn workspaces(&self) -> u32 {
        self.cache.len()
    }

    /// Workspaces no client holds: those the cache may evict.
    #[must_use]
    pub fn idle(&self) -> u32 {
        self.cache.idle()
    }

    /// Holds present, released ones included until they are reclaimed.
    #[must_use]
    pub fn holds(&self) -> u32 {
        self.holds.len()
    }

    /// The workstream of the cache's `nth` workspace that holds its
    /// repositories cloned, in byte order, if it has that many: what the
    /// worker tells the engine it holds checkouts for. A workspace whose disk
    /// is not known (being built, or damaged) is not counted.
    #[must_use]
    pub fn workstream(&self, nth: u32) -> Option<&[u8]> {
        self.cache.key(nth)
    }

    /// The oldest fact not yet drained. The parent drains them at its own
    /// pace; what does not fit meanwhile is dropped and counted.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// How many facts were dropped for want of room since the domain was made.
    #[must_use]
    pub fn facts_lost(&self) -> u64 {
        self.facts.lost()
    }

    /// The reclaim point: frees what was released in this iteration.
    pub fn reclaim(&mut self) {
        self.holds.reclaim();
    }
}

/// Handles one event, emitting git operations and client lifecycle answers.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Prepare { client, spec } => hold::prepare(domain, env, client, spec, out),
        Event::PushUnchecked { hold } => hold::unchecked(domain, env, hold, out),
        Event::Push { hold, message } => hold::push(domain, env, hold, message, out),
        Event::Save { hold, message } => hold::save(domain, env, hold, message, out),
        Event::Abort { hold } => hold::abort(domain, env, hold, out),
        Event::Release { hold } => hold::release(domain, env, hold, out),
        Event::Done { owner, done } => hold::done(domain, env, owner, done, out),
    }
}
