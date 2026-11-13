//! Admission while the engine drains (domain/engine/restart.md, 1).
use crate::{Core, Event, Held, Limits, Now, Request, effects::projections, routing};
use alloc::boxed::Box;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

impl Core {
    /// Whether placement and activation wait while commits keep flowing.
    #[must_use]
    pub fn settling(&self) -> bool {
        self.settling
    }

    /// Whether new work is refused while the service drains.
    #[must_use]
    pub fn stopping(&self) -> bool {
        self.stopping
    }
}

/// Settle each agent claim on a host in this process as interrupted, then
/// refuse what would commit (jig/docs/design/domain/engine/restart.md, 1).
pub(crate) fn start(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    let mut claims = List::with_capacity(core.proofs.len());
    for (&task, proof) in &core.proofs {
        if !core.start_opened()
            || (!core.workers_in_process && proof.host != fleet::HostKind::Engine)
            || core.unreported_restored.contains_key(&task)
            || core.restoring_proofs.contains_key(&task)
        {
            continue;
        }
        let Some(record) = core.tasks.task(task) else { continue };
        let agent = match record.executor {
            tasks::Executor::Agent { .. } => true,
            tasks::Executor::Job { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => false,
        };
        if agent && tasks::run_attempt(&record.phase) == Some(proof.attempt) {
            claims.push((Token::new(task), Token::new(proof.attempt))).expect("at most one claim per proof");
        }
    }
    let mut out = Queue::uncapped();
    let mut work = Queue::uncapped();
    for &(run, attempt) in &claims {
        let mut forgotten =
            super::children::route_fleet(core, env, &mut work, fleet::Event::Interrupt { run, attempt });
        for _ in 0..forgotten.len() {
            out.push(forgotten.pop().expect("interrupted fleet output count"));
        }
        core.remember_unpriced_terminal(run, attempt, tasks::End::Recheck);
        out.push(Request::Now(Box::new(Now::DropAssignment { task: run.raw() })));
        let token = core.internal(super::flight::Internal::Acknowledge);
        super::children::append_tasks(
            core,
            env,
            tasks::Event::Activation {
                reply_to: ReplyTo::new(token),
                task: run.raw(),
                attempt: attempt.raw(),
                cause: tasks::Cause::Unpriced,
                end: tasks::End::Recheck,
                saved: None,
            },
            &mut out,
        );
    }
    assert!(work.is_empty(), "forgetting an attempt starts no sibling work");
    core.stopping = true;
    core.draining = Some(true);
    out
}

pub(crate) fn interrupted(core: &Core, run: Token, attempt: Token) -> bool {
    if !core.stopping {
        return false;
    }
    let Some(proof) = core.proofs.get(&run.raw()) else { return false };
    if !core.workers_in_process && proof.host != fleet::HostKind::Engine {
        return false;
    }
    match core.committed_terminal(run.raw(), attempt.raw()) {
        Some(terminal) => terminal.end == tasks::End::Recheck,
        None => false,
    }
}

pub(crate) fn empty() -> Queue<Request> {
    answer(None)
}
fn answer(value: Option<Request>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let Some(value) = value {
        out.push(value);
    }
    out
}
fn party(to: ReplyTo) -> Queue<Request> {
    answer(Some(Request::Held(Box::new(Held::PeopleReply {
        to,
        reply: people::Reply::Again(people::Again::NotReady),
    }))))
}
fn watch(watcher: Token) -> Queue<Request> {
    answer(Some(Request::Held(Box::new(Held::WatchEnded { watcher, why: views::End::Stopping }))))
}
fn call(to: ReplyTo) -> Queue<Request> {
    answer(Some(Request::Now(Box::new(Now::CallBusy { to }))))
}

pub(crate) fn gate(core: &mut Core, event: Event) -> Result<Event, Queue<Request>> {
    if !core.stopping {
        return Ok(event);
    }
    match event {
        Event::ChangeModel { reply_to, .. }
        | Event::ChangeSettings { reply_to, .. }
        | Event::ProposeSettings { reply_to, .. }
        | Event::SignIn { reply_to, .. } => Err(party(reply_to)),
        Event::Act { reply_to, carried, .. } => Err(routing::application_routes::first(
            routing::application_routes::number(carried),
            answer(Some(Request::Held(Box::new(Held::Acted {
                to: reply_to,
                outcome: people::Outcome::Again(people::Again::NotReady),
            })))),
        )),
        Event::Key { reply_to, carried, .. } => {
            Err(routing::application_routes::first(Some(carried.number), party(reply_to)))
        }
        Event::Application { reply_to, connector, owner, carried, .. } => {
            let unused = routing::application_routes::number(carried);
            let mut out = routing::application_routes::first(unused, party(reply_to));
            if core.connectors.contains(&connector) {
                out.push(Request::Ask { connector, ask: crate::Ask::DropApplication { owner } });
            }
            Err(out)
        }
        Event::People(event) => people_gate(event),
        Event::Watch { watcher, .. } | Event::WatchRun { watcher, .. } => Err(watch(watcher)),
        Event::NamedAction { to, .. }
        | Event::NamedAnswer { to, .. }
        | Event::DelegateValidated { to, .. }
        | Event::DelegateInputFailed { to, .. }
        | Event::SettledCall { to, .. } => Err(call(to)),
        Event::Fleet(fleet::Event::Hello { worker: 0, channel, .. }) => {
            Err(answer(Some(Request::Held(Box::new(Held::Refuse { channel, hello: None })))))
        }
        Event::Fleet(event) => Ok(Event::Fleet(event)),
        Event::Settling
        | Event::Stopping
        | Event::Activate { .. }
        | Event::BriefAssembled { .. }
        | Event::WorkspacePrepared { .. }
        | Event::ClaimPrepared { .. }
        | Event::ClaimRefused { .. }
        | Event::TranscriptLoaded { .. }
        | Event::ProcedureStep { .. }
        | Event::ProcedureHoldings { .. }
        | Event::StartRecurring { .. }
        | Event::Period { .. } => Err(empty()),
        event @ (Event::Fill { .. }
        | Event::Marked { .. }
        | Event::ReadLoaded { .. }
        | Event::Read { .. }
        | Event::PublicRead { .. }
        | Event::PublicAnswered { .. }
        | Event::PublicDurable { .. }
        | Event::ApplicationPart { .. }
        | Event::ApplicationFinished { .. }
        | Event::ProjectionSettled { .. }
        | Event::EffectStart { .. }
        | Event::SubscribeTopic { .. }
        | Event::EffectConnector(_)
        | Event::EffectDeadline
        | Event::Unwatch { .. }
        | Event::Delivered { .. }
        | Event::StreamDelivered { .. }
        | Event::RunCall { .. }
        | Event::RunInput { .. }
        | Event::RunText { .. }
        | Event::Tasks(_)
        | Event::ConnectorRepair { .. }
        | Event::PersonTask { .. }
        | Event::TurnPayload { .. }
        | Event::AnswerPayload { .. }
        | Event::AcceptedTurn { .. }
        | Event::RefusedPayload { .. }
        | Event::EndedReadLoaded { .. }
        | Event::SeenConversationLoaded { .. }
        | Event::EndedProposal { .. }
        | Event::DecidedProposal { .. }
        | Event::EndedPersonTask { .. }
        | Event::PartyInputsLoaded { .. }
        | Event::EndedTaskLoaded { .. }
        | Event::EndedEscalation { .. }
        | Event::DelegateHoldings { .. }
        | Event::Brief(_)
        | Event::Holdings { .. }
        | Event::ProjectReleased { .. }
        | Event::Account(_)
        | Event::View(_)
        | Event::Notes(_)
        | Event::SettingsCommit { .. }) => Ok(event),
    }
}
fn people_gate(event: people::Event) -> Result<Event, Queue<Request>> {
    match event {
        people::Event::Act { reply_to, .. } => Err(answer(Some(Request::Held(Box::new(Held::Acted {
            to: reply_to,
            outcome: people::Outcome::Again(people::Again::NotReady),
        }))))),
        people::Event::Whose { reply_to, .. }
        | people::Event::Ask { reply_to, .. }
        | people::Event::SignOut { reply_to, .. } => Err(party(reply_to)),
        people::Event::ConversationSeen { .. } => Err(empty()),
        event @ (people::Event::ChatState { .. }
        | people::Event::MoveChat { .. }
        | people::Event::Deleting { .. }
        | people::Event::CloseProject { .. }
        | people::Event::ForgetProject { .. }
        | people::Event::Forget { .. }
        | people::Event::MakeService { .. }
        | people::Event::Seed { .. }
        | people::Event::Waiting { .. }
        | people::Event::ApplyRoles { .. }
        | people::Event::SignedIn { .. }
        | people::Event::Roles { .. }
        | people::Event::Decided { .. }
        | people::Event::Restore { .. }) => Ok(Event::People(event)),
    }
}
/// The journal holds this terminal behind every earlier commit.
pub(crate) fn drained(core: &mut Core, out: &mut Queue<Request>) {
    if core.draining == Some(true) && (core.watch_flights() == 0) {
        core.draining = Some(false);
        out.push(Request::Held(Box::new(Held::Drained)));
    }
}

impl Core {
    /// Reclaim retired child slots at the application's iteration boundary.
    pub fn reclaim(&mut self) {
        self.tasks.reclaim();
        self.people.reclaim();
        self.fleet.reclaim();
        self.views.reclaim();
        self.view_names.reclaim();
        self.reads.reclaim();
    }

    /// Discard expendable child observations without changing a decision.
    pub fn drain_facts(&mut self, limits: &Limits) {
        for _ in 0..limits.tasks.facts {
            let _fact = self.tasks.pop_fact();
        }
        for _ in 0..limits.people.facts {
            let _fact = self.people.pop_fact();
        }
        for _ in 0..limits.fleet.facts {
            let _fact = self.fleet.pop_fact();
        }
        for _ in 0..limits.accounts.facts {
            let _fact = self.accounts.pop_fact();
        }
        for _ in 0..limits.views.facts {
            let _fact = self.views.pop_fact();
        }
    }

    /// Pure, allocation-free finishing derivation (domain/root.md, section 3).
    /// Child-held waits and future alarms alone owe no finishing. Before
    /// omitting a closure the root must also check its current-clock timers.
    #[must_use]
    pub fn finish_owed(&self) -> bool {
        self.counters.dirty()
            || (self.draining == Some(true) && self.watch_flights() == 0)
            || !self.changed_views.is_empty()
            || projections::owed(self)
            || routing::view::marks::retention_owed(self)
            || routing::view::marks::endings_owed(self)
    }

    /// Idle fence including journal, routing and finishing (domain/root.md,
    /// section 3). A host may still hold a live attempt. False can mean a child
    /// awaits input; use `finish_owed` for closure after checking due timers.
    #[must_use]
    pub fn quiescent(&self, journal_idle: bool) -> bool {
        self.counters.quiescent(journal_idle)
            && self.reads.is_empty()
            && self.dependency_results.is_empty()
            && self.pending_calls.is_empty()
            && self.pending_note_briefs.is_empty()
            && self.ending_positions.is_empty()
            && self.flights_idle()
            && self.relaying.is_empty()
            && self.adopted.is_empty()
            && self.due.is_empty()
            && self.claiming.is_empty()
            && self.replaced.is_empty()
            && self.contexts.is_empty()
            && self.workspace_credentials.is_empty()
            && self.job_workspaces.is_empty()
            && self.transcripts.is_empty()
            && self.restoring_proofs.is_empty()
            && self.unreported_restored.is_empty()
            && self.brief.is_idle()
            && !self.fleet.is_ready()
            && self.fleet.turns() == 0
            && self.fleet.calls() == 0
            && self.fleet.next_deadline().is_none()
            && !self.accounts.waiting()
            && !self.finish_owed()
    }
}
