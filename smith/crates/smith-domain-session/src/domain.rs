//! The session child domain's state and its entry points (smith/docs/design/domain/session.md,
//! sections 3–7; skein/docs/foundation/programming-model.md, sections 4.5 and 6.3). [`Domain`]
//! retains sessions, tools, alarms, deferred ready work, injected randomness
//! and uncapped facts. [`step`], [`fire`] and [`resume`] emit owned requests.
//! Facts are kept losslessly and drained by their queued count.
//! This module never knows run outcomes, host policy, credentials or provider
//! wire syntax.

use skein_lib::{Deadlines, Env, Queue, Rng, Slab, Time};
use smith_domain_tools as tools;

use crate::boundary::{Event, Request};
use crate::facts::{Fact, Facts};
use crate::limits::{self, Limits};
use crate::session::{self, Alarm, Calls, Ready, Session};

/// The session child domain's state.
#[derive(Debug)]
pub struct Domain {
    pub(crate) sessions: Slab<Session>,
    /// What runs the sessions' tool calls: the tools, which the session owns.
    pub(crate) calls: Calls,
    pub(crate) alarms: Deadlines<Alarm>,
    /// Sessions resting after a batch answered at once.
    pub(crate) ready: Ready,
    pub(crate) rng: Rng,
    pub(crate) facts: Facts,
}

impl Domain {
    /// A domain with room for `limits`, drawing randomness from `seed`.
    #[must_use]
    pub fn new(limits: &Limits, seed: u64) -> Domain {
        let alarms = limits::alarms(limits).expect("valid limits");
        let runs = limits::runs(limits).expect("valid limits");
        let calls =
            Calls { runs: Slab::with_capacity(runs), tools: tools::Domain::new(&limits.tools), out: Queue::uncapped() };
        Domain {
            sessions: Slab::with_capacity(limits.sessions),
            calls,
            alarms: Deadlines::with_capacity(alarms),
            ready: Ready::with_capacity(limits.sessions),
            rng: Rng::new(seed),
            facts: Facts::new(),
        }
    }

    /// Sessions present, closed ones included until they are reclaimed.
    #[must_use]
    pub fn sessions(&self) -> u32 {
        self.sessions.len()
    }

    /// Tool runs present, the tools' and the opener's, ended ones included
    /// until they are reclaimed.
    #[must_use]
    pub fn runs(&self) -> u32 {
        self.calls.runs.len()
    }

    /// The parent opener owning live tool IO, for the root's deadline notices.
    #[must_use]
    pub fn io_opener(&self, owner: skein_lib::Token) -> Option<skein_lib::Token> {
        session::io_opener(self, owner)
    }

    /// Kits present, closed ones included until they are reclaimed.
    #[must_use]
    pub fn kits(&self) -> u32 {
        self.calls.tools.kits()
    }

    /// Calls the tools are running, answered ones included until they are
    /// reclaimed.
    #[must_use]
    pub fn flights(&self) -> u32 {
        self.calls.tools.flights()
    }

    /// When the earliest alarm falls due.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.alarms.next()
    }

    /// Whether an alarm is due at `now`. While one is, the loop fires the
    /// root domain, which calls [`fire`].
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        match self.alarms.next() {
            Some(at) => at <= now,
            None => false,
        }
    }

    /// Whether a session is ready to go on. While one is, the loop resumes the
    /// root domain, which calls [`resume`], at the start of the domain's
    /// stage, before its input events.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.ready.is_ready()
    }

    /// The oldest observation, including the tools' observations.
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
        self.sessions.reclaim();
        self.calls.runs.reclaim();
        self.calls.tools.reclaim();
        self.ready.promote();
    }
}

/// Handles one event, emitting requests into an uncapped queue.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    domain.facts.begin(env.now);
    match event {
        Event::Open { opener, opening } => session::open(domain, env, opener, *opening, out),
        Event::Answered { owner, text, error, spent } => {
            session::delegate_ended(
                domain,
                env,
                owner,
                crate::llm::Returned::Text { text, error, replay: None },
                session::Bill { spent, after: session::After::Complete },
                out,
            );
        }
        Event::AnsweredAndYield { owner, text, error, spent } => {
            session::delegate_ended(
                domain,
                env,
                owner,
                crate::llm::Returned::Text { text, error, replay: None },
                session::Bill { spent, after: session::After::Yield },
                out,
            );
        }
        Event::AnswerWithheld { owner } => session::delegate_withheld(domain, env, owner, out),
        Event::AnswerCancelled { owner, spent } => {
            session::delegate_ended(
                domain,
                env,
                owner,
                crate::llm::Returned::Withdrawn,
                session::Bill { spent, after: session::After::Complete },
                out,
            );
        }
        Event::Continue { session, content } => session::continued(domain, env, session, content, out),
        Event::Yield { session } => session::yield_completion(domain, session, out),
        Event::Close { session } => session::close(domain, env, session, out),
        Event::CancelJobs { session } => session::cancel_jobs(domain, env, session, out),
        Event::ReadVersion { session, place, version } => {
            session::read_version(domain, env, session, place, version, out);
        }
        Event::BuiltinResult { session, id, text, error } => {
            session::builtin_result(domain, env, session, id, text, error, out);
        }
        Event::Completed { owner, completion } => session::completed(domain, env, owner, completion, out),
        Event::Overflowed { owner, end } => session::overflowed(domain, env, owner, end, out),
        Event::BudgetDenied { owner, reason } => session::budget_denied(domain, env, owner, reason, out),
        Event::UnsentClosed { owner } => session::unsent_closed(domain, env, owner, out),
        Event::Narrowed { owner, note, choice, names } => {
            session::narrowed(domain, env, owner, note, choice, names, out);
        }
        Event::Failed { owner, failure, evidence, detail, retry_ok } => {
            session::failed(domain, env, owner, failure, evidence, detail, retry_ok, out);
        }
        Event::Cancelled { owner } => session::cancelled(domain, env, owner, out),
        Event::Done { owner, done } => session::io_done(domain, env, owner, done, out),
    }
    session::pass_on_facts(domain);
}

/// Fires the earliest alarm due at `env.now` into an uncapped request queue.
/// A stage fires its alarms after its input events, so
/// progress that arrived in the same iteration wins over a deadline that passed
/// while the loop waited.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.facts.begin(env.now);
    let Some(alarm) = domain.alarms.expire(env.now) else {
        return;
    };
    match alarm {
        Alarm::Expiry { session } => session::expire(domain, env, session, out),
        Alarm::Retry { session } => session::retry(domain, env, session, out),
    }
    session::pass_on_facts(domain);
}

/// Starts a session on the ready list again, emitting into an uncapped request
/// queue: one that rested in an earlier iteration, after a batch the tools
/// answered within the step that started it.
pub fn resume(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.facts.begin(env.now);
    let Some(id) = domain.ready.pop() else {
        return;
    };
    session::rested(domain, env, id, out);
    session::pass_on_facts(domain);
}
