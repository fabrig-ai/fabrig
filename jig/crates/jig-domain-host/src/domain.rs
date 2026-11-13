//! The host child domain's state and its entry points (section 3).

use crate::{LinkDown, LinkUp};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Map, Queue, Slab, Token};

use crate::boundary::{Event, Hosting, Reason, Request};
use crate::call::{self, Call};
use crate::facts::{Fact, Facts};
use crate::hosted::{self, Hosted};
use crate::limits::{self, Limits};
use crate::turns::Turns;

/// The host child domain's state.
#[derive(Debug)]
pub struct Hub {
    pub(crate) job_kinds: Box<[crate::JobBound]>,
    pub(crate) sources: crate::sources::Domain,
    /// The run slots.
    pub(crate) hosted: Slab<Hosted>,
    /// The runs not yet answered, by the engine's names for them.
    pub(crate) names: Map<Token, Id<Hosted>>,
    /// Host calls in flight, across runs.
    pub(crate) calls: Slab<Call>,
    /// Runs to cancel, each for its reason, one per resume.
    pub(crate) ready: Map<Id<Hosted>, Reason>,
    pub(crate) facts: Facts,
    pub(crate) turns: Turns,
    pub(crate) answers: Map<(Token, Token), Id<Hosted>>,
    /// The root is shutting down: it admits no more runs.
    pub(crate) shut: bool,
    pub(crate) refusals_lost: u64,
}

impl Hub {
    /// A domain with room for `limits`.
    #[must_use]
    pub fn new(limits: &Limits) -> Hub {
        let calls = limits::calls(limits).expect("valid_hub accepted the limits");
        Hub {
            job_kinds: Box::new([]),
            sources: crate::sources::Domain::new(limits.program_sources),
            hosted: Slab::with_capacity(limits.slots),
            names: Map::with_capacity(limits.slots),
            calls: Slab::with_capacity(calls),
            ready: Map::with_capacity(limits.slots),
            facts: Facts::with_capacity(limits.facts, limits.told),
            turns: Turns::new(limits),
            answers: Map::with_capacity(limits.slots),
            shut: false,
            refusals_lost: 0,
        }
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<skein_lib::Time> {
        self.sources.next_deadline()
    }
    #[must_use]
    pub fn sources_settled(&self) -> bool {
        self.sources.settled()
    }

    /// Grant names held by attempts, including those awaiting acknowledgement (protocol/channel/README.md, 7).
    #[must_use]
    pub fn holds_grant(&self, account: u32, generation: u64) -> bool {
        if self.sources.holds_grant(account, generation) {
            return true;
        }
        for (_, entry) in &self.hosted {
            for grant in entry.grants() {
                if grant.account == account
                    && grant.generation == generation
                    && grant.form.table() == jig_domain_link::GrantTable::Accounts
                {
                    return true;
                }
            }
        }
        false
    }

    /// Runs hosted: the slots taken, closed runs included until they are
    /// reclaimed.
    #[must_use]
    pub fn hosted(&self) -> u32 {
        self.hosted.len()
    }

    /// Runs that have not answered yet: those hosted, closed ones aside.
    #[must_use]
    pub fn unanswered(&self) -> u32 {
        self.names.len()
    }

    /// Host calls in flight, closed ones included until they are reclaimed.
    #[must_use]
    pub fn calls(&self) -> u32 {
        self.calls.len()
    }

    /// The engine's names for the hosted run `owner` (the token the host's
    /// requests for it carry) and where it is in its lifecycle, while it is
    /// hosted: for its parent to tell the engine what the run's agent says
    /// beside the host, its facts.
    #[must_use]
    pub fn hosting(&self, owner: Token) -> Option<Hosting> {
        hosted::hosting(self, owner)
    }

    /// Whether the attempt is already hosted: wire retries are deduplicated
    /// before handing an assignment down.
    #[must_use]
    pub fn is_hosting(&self, run: Token, attempt: Token) -> bool {
        hosted::is_hosting(self, run, attempt)
    }

    /// Whether an answer names a pending relay of this exact attempt.
    #[must_use]
    pub fn owns_relay(&self, run: Token, attempt: Token, call: &[u8]) -> bool {
        hosted::find_relay(self, run, attempt, call).is_some()
    }

    /// Whether an engine answer names this attempt's pending call.
    #[must_use]
    pub fn is_relayed_for(&self, run: Token, attempt: Token, call: &[u8]) -> bool {
        self.owns_relay(run, attempt, call)
    }

    /// Whether this call still waits for the engine's answer.
    #[must_use]
    pub fn is_relayed(&self, run: Token, attempt: Token, call: &[u8]) -> bool {
        let Some(id) = hosted::find_relay(self, run, attempt, call) else {
            return false;
        };
        match self.calls.get(id).expect("found relay").state {
            call::State::Relayed { .. } => true,
            call::State::JobDelivering
            | call::State::Delivering { .. }
            | call::State::Settling
            | call::State::Closed => false,
        }
    }

    /// Whether a run is ready to be cancelled. While one is, the loop resumes
    /// the root domain, which calls [`resume`], at the start of the
    /// domain's stage, before its input events.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        !self.ready.is_empty()
    }

    /// The oldest fact not yet drained. The parent drains them at its own
    /// pace; what does not fit meanwhile is dropped and counted.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.facts.pop()
    }

    /// How many facts were dropped for want of room, since the domain was
    /// made.
    #[must_use]
    pub fn facts_lost(&self) -> u64 {
        self.facts.lost()
    }

    /// The oldest activity or still-current live piece waiting for the engine.
    /// Pieces whose attempts ended or whose turns were taken are fenced here.
    pub fn pop_activity(&mut self) -> Option<LinkUp> {
        crate::live::pop_activity(self)
    }

    /// Activity dropped for lack of room.
    #[must_use]
    pub const fn activity_lost(&self) -> u64 {
        self.facts.activity_lost()
    }

    /// Live pieces dropped for lack of room in the activity queue.
    #[must_use]
    pub const fn pieces_lost(&self) -> u64 {
        self.facts.pieces_lost()
    }

    /// Live pieces dropped at admission or drain for a failed attempt, ask,
    /// size or turn fence.
    #[must_use]
    pub const fn pieces_fenced(&self) -> u64 {
        self.facts.pieces_fenced()
    }

    /// Turns retained until the engine commits them.
    #[must_use]
    pub fn retained_turns(&self) -> u32 {
        self.turns.len()
    }

    /// Whether a turn is still retained for retry.
    #[must_use]
    pub fn has_turn(&self, run: Token, attempt: Token, turn: u32) -> bool {
        self.turns.holds(run, attempt, turn)
    }

    /// Whether any turn of this attempt remains unacknowledged.
    #[must_use]
    pub fn has_turns_for(&self, run: Token, attempt: Token) -> bool {
        self.turns.has_run(run, attempt)
    }

    /// A transmission copy of a retained turn.
    #[must_use]
    pub fn turn(&self, run: Token, attempt: Token, turn: u32) -> Option<crate::Turn> {
        self.turns.get(run, attempt, turn)
    }

    /// A transmission copy of a retained turn by position, for reconnecting.
    #[must_use]
    pub fn turn_at(&self, index: u32) -> Option<LinkUp> {
        let (run, attempt, turn) = self.turns.nth(index)?;
        Some(LinkUp::Turn { run, attempt, turn })
    }

    #[must_use]
    pub fn kept_answers(&self) -> u32 {
        crate::answers::kept(self)
    }

    #[must_use]
    pub fn answer_at(&self, index: u32) -> Option<LinkUp> {
        crate::answers::at(self, index)
    }

    /// Turns abandoned after shutdown past the contact grace.
    pub fn give_up_turns(&mut self) {
        self.turns.give_up();
    }

    /// How many turns were abandoned.
    #[must_use]
    pub const fn turns_abandoned(&self) -> u64 {
        self.turns.abandoned()
    }

    /// Refusals dropped beyond the per-run inbox bound.
    #[must_use]
    pub const fn refusals_lost(&self) -> u64 {
        self.refusals_lost
    }

    /// The reclaim point: frees what closed in this iteration.
    pub fn reclaim(&mut self) {
        self.hosted.reclaim();
        self.calls.reclaim();
    }
}

/// Handles one event and emits its requests.
#[expect(clippy::too_many_lines, reason = "the hub routes the complete event vocabulary in one step")]
pub fn step(domain: &mut Hub, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::ListRead { read, outcome } => crate::sources::list_read(&mut domain.sources, env, read, outcome, out),
        Event::SourceAnswered { owner, answer } => {
            crate::sources::answered(&mut domain.sources, env, owner, answer, out);
        }
        Event::SourceLost => crate::sources::lost(&mut domain.sources, out),
        Event::Agent(crate::FromAgent::Called { owner, call, ask }) => {
            hosted::called(domain, env, owner, call, ask, out);
        }
        Event::Agent(crate::FromAgent::Withdrawn { owner, call }) => hosted::withdrawn(domain, owner, call, out),
        Event::Agent(crate::FromAgent::Turn { owner, turn }) => hosted::turned(domain, env, owner, turn, out),
        Event::Agent(crate::FromAgent::Piece { owner, piece }) => crate::live::reported(domain, env, owner, piece),
        Event::Agent(crate::FromAgent::Activity { owner, activity }) => {
            crate::activity::reported(domain, owner, activity);
        }
        Event::Agent(crate::FromAgent::Unauthorized { owner, account, generation }) => {
            crate::activity::unauthorized(domain, owner, account, generation, out);
        }
        Event::Agent(crate::FromAgent::Limited { owner, account, reset }) => {
            crate::activity::limited(domain, owner, account, reset, out);
        }
        Event::Agent(crate::FromAgent::AccountRefused { owner, account, code, reason }) => {
            crate::activity::account_refused(domain, owner, account, code, reason, out);
        }
        Event::Agent(crate::FromAgent::Finished { owner, turns, spent, read, finish }) => {
            hosted::finished(domain, env, owner, hosted::Last { turns, spent, finish, read }, out);
        }
        Event::RelayCancelled { run, attempt, call } => hosted::relay_cancelled(domain, run, attempt, &call, out),
        Event::Drain => {
            shutting_down(domain, out);
            hosted::drain(domain, out);
        }
        Event::CancelAll { reason } => {
            if reason == Reason::Shutdown {
                shutting_down(domain, out);
            }
            hosted::cancel_all(domain, reason);
        }
        Event::Opened { stop_bound, workstreams, bounds, jobs } => {
            domain.job_kinds = jobs;
            crate::answers::opened(domain, env.limits.slots, stop_bound, workstreams, bounds, out);
            if domain.shut {
                out.push(Request::Up(LinkUp::Draining));
            }
        }
        Event::Job(event) => crate::jobs::reported(domain, env, event, out),
        Event::Prepared { owner, workspace } => {
            if crate::jobs::owns(domain, owner) {
                crate::jobs::prepared(domain, owner, workspace, out);
            } else {
                hosted::prepared(domain, owner, workspace, out);
            }
        }
        Event::Unprepared { owner, failure, detail } => {
            if crate::jobs::owns(domain, owner) {
                crate::jobs::unprepared(domain, env, owner, failure, detail, out);
            } else {
                hosted::unprepared(domain, env, owner, failure, detail, out);
            }
        }
        Event::Agent(crate::FromAgent::Started { owner, agent }) => hosted::started(domain, env, owner, agent, out),
        Event::Agent(crate::FromAgent::StartBusy { owner }) => hosted::start_busy(domain, env, owner, out),
        Event::Agent(crate::FromAgent::StartRefused { owner, why, cause }) => {
            hosted::start_refused(domain, env, owner, why, cause, out);
        }
        Event::Agent(crate::FromAgent::Refused { owner, name, reason }) => {
            hosted::refused_message(domain, owner, name, reason, out);
        }
        Event::Agent(crate::FromAgent::Yielded { owner }) => hosted::yielded(domain, owner, out),
        Event::Agent(crate::FromAgent::Faulted { owner, fault }) => hosted::faulted(domain, env, owner, fault, out),
        Event::Agent(crate::FromAgent::Gone { owner, detail }) => hosted::gone(domain, env, owner, detail, out),
        Event::Delivered { owner, delivery } => {
            if crate::jobs::delivery_owner(domain, owner) {
                crate::jobs::delivered(domain, owner, delivery, out);
            } else {
                hosted::delivered(domain, owner, delivery, out);
            }
        }
        Event::Saved { owner, at } => hosted::saved(domain, owner, at, out),
        Event::Down(down) => match down {
            LinkDown::ReadList { read, endpoint, grant } => {
                crate::sources::read(&mut domain.sources, env, read, endpoint, grant, out);
            }
            LinkDown::Sources { sources } => crate::sources::replace(&mut domain.sources, env, sources, out),
            LinkDown::Check { name } => crate::sources::check(&mut domain.sources, env, name, out),
            LinkDown::SignOut { name } => crate::sources::sign_out(&mut domain.sources, env, name, out),
            LinkDown::Message { run, attempt, name, sender, words, finish_requested } => {
                hosted::inbound_named(
                    domain,
                    env,
                    run,
                    attempt,
                    hosted::NamedEvent { name, event: words, sender: Some(sender), finish_requested },
                    out,
                );
            }
            LinkDown::AssignJob { job } => crate::jobs::assign(domain, env, job, out),
            LinkDown::Assign { assignment } => hosted::assign(domain, env, assignment, out),
            LinkDown::TurnKept { run, attempt, turn } => hosted::acknowledge_turn(domain, env, run, attempt, turn, out),
            LinkDown::Grant { run, attempt, grant } => hosted::grant(domain, run, attempt, grant, out),
            LinkDown::Cancel { run, attempt } => hosted::cancel(domain, env, run, attempt, out),
            LinkDown::Relayed { run, attempt, call, answer } => {
                hosted::relayed(domain, run, attempt, &call, answer, out);
            }
            LinkDown::Acknowledge { run, attempt } => crate::answers::acknowledge(domain, run, attempt),
        },
    }
}

fn shutting_down(domain: &mut Hub, out: &mut Queue<Request>) {
    if !domain.shut {
        out.push(Request::Up(LinkUp::Draining));
        crate::sources::close(&mut domain.sources, out);
        domain.shut = true;
    }
}

/// Cancels a run on the ready list, if one is, for its drain or `CancelAll` reason.
/// Also fires due source checks, independently of the run slots.
pub fn resume(domain: &mut Hub, env: &Env<Limits>, out: &mut Queue<Request>) {
    resume_runs(domain, env, out);
    fire_sources(domain, env, out);
}

/// Cancels a run on the ready list, if one is: `resume` without the
/// source checks, for a root that admits those as new work.
pub fn resume_runs(domain: &mut Hub, env: &Env<Limits>, out: &mut Queue<Request>) {
    hosted::resume(domain, env, out);
}

/// Fires the source checks that are due: new work, which a root admits
/// at its watermark; a check not fired stays due.
pub fn fire_sources(domain: &mut Hub, env: &Env<Limits>, out: &mut Queue<Request>) {
    crate::sources::fire(&mut domain.sources, env, out);
}

/// Whether a source check is due at `now`.
#[must_use]
pub fn sources_due(domain: &Hub, now: skein_lib::Time) -> bool {
    match domain.sources.next_deadline() {
        Some(deadline) => deadline <= now,
        None => false,
    }
}
