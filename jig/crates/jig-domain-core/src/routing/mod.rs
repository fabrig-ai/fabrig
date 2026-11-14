//! Child entry points and their requests in the core's vocabulary
//! (`domain/engine/README.md`, sections 3 and 4). A root receives a whole child
//! request batch and routes it within the same decision.

#![expect(clippy::mod_module_files, reason = "the route split keeps dispatch in routing/mod.rs")]

pub(crate) mod application_routes;
pub(crate) mod children;
pub(crate) mod effect;
pub(crate) mod ended_marks;
pub(crate) mod flight;
mod lists;
pub(crate) mod party;
pub(crate) mod refusals;
pub(crate) mod run;
pub(crate) mod settings;
pub(crate) mod sibling_routes;
pub(crate) mod step;
pub(crate) mod stopping;
pub(crate) mod task;
pub(crate) mod timers;
pub(crate) mod view;

pub use crate::vocabulary::{Event, Held, HoldingsNeed, Limits, Request, Timer, ToolKind, TranscriptLoad};
use crate::{
    CallKey, CallPart, Core, RoutedCall,
    routing::{
        children::{
            route_account, route_brief, route_fleet, route_projection_settled, tag_account, tag_brief, tag_fleet,
        },
        effect::note::route_notes,
        party::{
            ended::{route_ended_escalation, route_ended_proposal},
            holdings::route_holdings,
            people::{route_people, sign_in},
        },
        run::{
            calls::{named_action, route_named_answer, route_refused_payload},
            delegation::{
                delegate_holdings, delegate_input_failed, delegate_validated, procedure_holdings, procedure_step,
                route_period, route_start_recurring,
            },
            payloads::{route_answer_payload, route_turn_payload},
            preparation::{
                brief_assembled, plan_brief, route_activate, route_claim_prepared, route_claim_refused,
                route_preparation_failed, start_brief, workspace_prepared,
            },
        },
        task::{route_connector_repair, route_person_task, route_tasks, tag_task_outputs},
        view::routes::{open, route_view},
    },
};
pub(crate) use crate::{
    routing::{run::calls::named_propose, view::marks::finish_watches},
    vocabulary::Work,
};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

use crate::routing::run::turns::route_accepted_turn;

impl Core {
    /// Retain a numbered connector call until its translated answer arrives.
    #[must_use]
    pub fn reserve_connector_call(&mut self, key: CallKey) -> bool {
        self.start_runtime_input();
        self.pending_calls.insert(key, true) == Ok(None)
    }

    /// Retain a connector subscription's task-hub continuation with its named call.
    #[must_use]
    pub fn reserve_connector_subscription(&mut self, token: Token, key: CallKey, subscription: u64) -> Option<Token> {
        self.start_input();
        if !self.reserve_connector_call(key) {
            return None;
        }
        Some(self.fly_call(token, RoutedCall::Subscribe { key, subscription }))
    }
}

/// Person choice retained only while the task child decides its exact revision.
#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct PersonEscalation {
    pub(crate) request: Token,
    pub(crate) requester: u64,
    pub(crate) person: u64,
    pub(crate) project: u32,
    pub(crate) task: u64,
    pub(crate) revision: u64,
    pub(crate) decision: people::EscalationDecision,
    pub(crate) notice: Option<tasks::Message>,
}

#[derive(Hash, Debug)]
pub(crate) enum CreationKind {
    Batch { person: u64, given: Option<people::Given>, stubs: Box<[tasks::Stub]> },
    Propose { person: u64, reason: Box<[u8]>, to: u32, stubs: Box<[tasks::Stub]> },
}

/// A party's batch waiting for the task child's make result.
#[derive(Hash, Debug, PartialEq, Eq)]
pub(crate) struct Made {
    pub first: u64,
    pub given: Option<people::Given>,
    pub stubs: Box<[tasks::Stub]>,
}

/// One person task pending all numbered connector holdings in this decision.
#[derive(Hash, Debug)]
pub(crate) struct Creation {
    kind: CreationKind,
    members: Box<[tasks::New]>,
    symbols: Box<[Box<[people::Symbol]>]>,
    at: usize,
    remaining: u32,
    holdings: List<tasks::Holding>,
    answered: List<u16>,
    failed: bool,
    refused: Option<crate::HoldingsRefusal>,
}

#[derive(Hash, Debug)]
pub(crate) struct DelegateCreation {
    key: CallKey,
    kind: BatchKind,
    members: Box<[tasks::New]>,
    remaining: u32,
    answered: List<u16>,
    failed: bool,
    refused: Option<crate::HoldingsRefusal>,
}

#[derive(Hash, Debug)]
enum BatchKind {
    Delegation { stubs: Box<[tasks::Stub]>, given: Option<people::Given> },
    Proposal { reason: Box<[u8]>, as_holder: bool },
}

#[derive(Hash, Debug)]
pub(crate) struct ProcedureCreation {
    decision: tasks::ProcedureDecision,
    members: Box<[tasks::New]>,
    remaining: u32,
    answered: List<u16>,
    failed: bool,
    refused: Option<crate::HoldingsRefusal>,
}

impl Core {
    /// Check a named run call against its live task and policy before routing it.
    #[must_use]
    pub fn authorize_tool(&self, key: CallKey, kind: ToolKind) -> Option<CallPart> {
        let Some(task) = self.tasks.task(key.task) else {
            return Some(CallPart::ToolDenied {
                answer: authority::Answer::Refuse,
                findings: Box::new([authority::Finding::UnknownProject]),
            });
        };
        let family = self.tool_family(&kind);
        let call = match kind {
            ToolKind::Message { target } => authority::Call::Message {
                referenced: task.requester == tasks::Party::Task(target)
                    || task.delegates.contains(&target)
                    || task.references.contains(&target),
            },
            ToolKind::Note { scope } => {
                let scope = match scope {
                    notes::Scope::Deployment => authority::NoteScope::Deployment,
                    notes::Scope::Project { project } if project == task.project => authority::NoteScope::Project,
                    notes::Scope::Goal { project, goal }
                        if project == task.project && Some(goal) == self.note_goal(task) =>
                    {
                        authority::NoteScope::Goal
                    }
                    notes::Scope::Resources { project, connector, pattern } if project == task.project => {
                        let last = match pattern.last {
                            notes::Last::Exact(value) => authority::Last::Exact(value),
                            notes::Last::Open(value) => authority::Last::Open(value),
                        };
                        authority::NoteScope::Resources(authority::ResourceScope {
                            connector,
                            pattern: authority::Pattern { segments: pattern.segments, last },
                        })
                    }
                    notes::Scope::Party { party: person } => return party::authorize_note(self, key, person),
                    notes::Scope::Project { .. } | notes::Scope::Goal { .. } | notes::Scope::Resources { .. } => {
                        return Some(CallPart::ToolDenied {
                            answer: authority::Answer::Refuse,
                            findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
                        });
                    }
                };
                authority::Call::Note(scope)
            }
            ToolKind::Recall { by } => {
                let readable = match by {
                    notes::Recall::Name { .. } => true,
                    notes::Recall::Search { scopes, .. } => {
                        let mut readable = true;
                        for scope in &scopes {
                            if !self.note_readable(key.task, scope) {
                                readable = false;
                            }
                        }
                        readable
                    }
                };
                if !readable {
                    return Some(CallPart::ToolDenied {
                        answer: authority::Answer::Refuse,
                        findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
                    });
                }
                authority::Call::Tool
            }
            ToolKind::Delegate
            | ToolKind::Control
            | ToolKind::Decide
            | ToolKind::Propose
            | ToolKind::Subscribe
            | ToolKind::Effect
            | ToolKind::Read
            | ToolKind::Ask
            | ToolKind::Settings => authority::Call::Tool,
        };
        let mut why = Queue::uncapped();
        let answer = authority::check_call(
            &self.authority,
            &authority::CallAsk {
                project: task.project,
                authority: crate::translate::tasks_authority(&task.authority),
                family,
                call,
            },
            &mut why,
        );
        if answer == authority::Answer::Allow {
            return None;
        }
        let mut findings = List::with_capacity(why.len());
        for _ in 0..why.len() {
            findings.push(why.pop().expect("finding count")).expect("finding room");
        }
        Some(CallPart::ToolDenied { answer, findings: findings.into_boxed() })
    }

    fn tool_family(&self, kind: &ToolKind) -> authority::Tools {
        match kind {
            ToolKind::Delegate => self.settings.tools.delegate,
            ToolKind::Message { .. } => self.settings.tools.message,
            ToolKind::Control => self.settings.tools.control,
            ToolKind::Decide => self.settings.tools.decide,
            ToolKind::Propose => self.settings.tools.propose,
            ToolKind::Subscribe => self.settings.tools.subscribe,
            ToolKind::Effect => self.settings.tools.effect,
            ToolKind::Note { .. } => self.settings.tools.note,
            ToolKind::Recall { .. } => self.settings.tools.recall,
            ToolKind::Read => self.settings.tools.read,
            ToolKind::Settings => authority::Tools(self.settings.tools.settings),
            ToolKind::Ask => self.settings.tools.ask,
        }
    }

    /// A named recall may read only notes in this task's project and goal, or deployment notes.
    #[must_use]
    pub fn note_readable(&self, task: u64, scope: &notes::Scope) -> bool {
        let Some(row) = self.tasks.task(task) else { return false };
        match scope {
            notes::Scope::Party { party: person } => party::note_readable(self, task, *person),
            notes::Scope::Deployment => true,
            notes::Scope::Project { project } | notes::Scope::Resources { project, .. } => *project == row.project,
            notes::Scope::Goal { project, goal } => *project == row.project && Some(*goal) == self.note_goal(row),
        }
    }
}

/// The parent caller held while the notes child reads its store pages.
#[derive(Hash, Debug)]
pub(crate) enum NoteRoute {
    Deletion { project: u32 },
    Call { to: ReplyTo, key: CallKey },
    Person { request: Token },
    Brief { task: u64 },
    Preferences { task: u64 },
    Now { task: u64 },
}

/// One task's wait within the live-tasks bound (`limits.tasks.tasks`)
/// (`jig/docs/design/domain/engine/briefs.md`, 2.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum NoteBrief {
    Operation,
    /// The tasks child's next Activate returns to the owner's due queue.
    Activation,
}

impl Core {
    pub(crate) fn begin_note_route(&mut self, route: NoteRoute) -> Token {
        self.fly(flight::Flight::Note(route))
    }
}

/// Route one child event and each sibling continuation within one core decision.
pub fn step(core: &mut Core, env: &Env<Limits>, event: Event) -> Queue<Request> {
    core.start_input();
    if core.start_failure().is_some() {
        return Queue::uncapped();
    }
    if let Some(phase) = core.start_phase() {
        #[expect(clippy::wildcard_enum_match_arm, reason = "only restore entrances need phase validation")]
        match &event {
            Event::Account(accounts::Event::Want) => return want_accounts(core, env),
            Event::Account(accounts::Event::Restore { .. })
            | Event::People(people::Event::Restore { .. })
            | Event::Tasks(tasks::Event::Restore { .. }) => {
                assert!(core.start_loading(), "rows load before Link and outside a seed commit");
            }
            Event::SettingsCommit { .. } => unreachable!("the start commits settings itself"),
            Event::EffectConnector(
                crate::effects::connector::Event::Resource { name, .. }
                | crate::effects::connector::Event::Adopted { name, .. },
            ) => {
                assert!(takes_connector_resource(phase, name.connector), "connector resources load before Link");
            }
            _ => {}
        }
    }
    let oldest = core.current.oldest;
    core.run_now = env.now;
    core.run_wall = env.wall;
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    work.push(Work::Event(event));
    for _ in 0..env.limits.routes {
        // A resumed preparation may be stale or need no notes operation.
        // Drain its continuations before waking the next waiter while notes is idle.
        if work.is_empty() && !core.stopping && !core.settling {
            effect::note::resume_brief(core, &mut work);
        }
        let Some(next) = work.pop() else { break };
        let mut batch = step_work(core, env, next, &mut work);
        for _ in 0..batch.len() {
            let request = batch.pop().expect("core route output count");
            out.push(request);
        }
    }
    assert!(work.is_empty(), "sibling_routes_drained");
    core.finish_extent(oldest, &mut out);
    core.start_collect_failure();
    if core.start_failure().is_some() {
        return Queue::uncapped();
    }
    out
}

/// Want every configured account once, after the restart has restored the core's own ranges.
pub fn want_accounts(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    core.start_input();
    if let Some(phase) = core.start_phase() {
        assert!(phase == crate::start::Phase::Link, "account wants attach after all loading");
        assert!(!core.accounts.restore_finished(), "account wants link once");
    }
    let oldest = core.current.oldest;
    core.run_now = env.now;
    core.run_wall = env.wall;
    let mut work = Queue::uncapped();
    let mut out = route_account(core, env, &mut work, accounts::Event::Want);
    assert!(work.is_empty(), "wanting the accounts answers no party");
    core.finish_extent(oldest, &mut out);
    if core.start_phase().is_some() {
        let mut writes = Queue::uncapped();
        for _ in 0..out.len() {
            let request = out.pop().expect("bounded restored accounts output");
            match request {
                Request::Write(_) => writes.push(request),

                Request::ProjectMade { .. } | Request::Ask { .. } | Request::Held(_) | Request::Now(_) => {
                    core.start.held.push(request);
                }
            }
        }
        core.start_collect_failure();
        return writes;
    }
    out
}

fn takes_connector_resource(phase: crate::start::Phase, connector: u16) -> bool {
    match phase {
        crate::start::Phase::RestoreConnector { connector: expected } => connector == expected,
        crate::start::Phase::AdoptRuns
        | crate::start::Phase::ReadAfresh { .. }
        | crate::start::Phase::SettleOutbox { .. }
        | crate::start::Phase::Open => true,
        crate::start::Phase::LoadCore
        | crate::start::Phase::LoadInbox
        | crate::start::Phase::Link
        | crate::start::Phase::Sweep
        | crate::start::Phase::Switch
        | crate::start::Phase::Wires => false,
    }
}

pub(crate) fn start_task_outputs(
    core: &mut Core,
    env: &Env<Limits>,
    child: Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let mut routed = tag_task_outputs(core, env, child);
    for _ in 0..routed.len() {
        let request = routed.pop().expect("bounded restored task outputs");
        out.push(request);
    }
}

/// Resume the fleet's next held placement or relay through the core's sibling routes.
pub fn resume_fleet(core: &mut Core, env: &Env<Limits>) -> Queue<Request> {
    core.start_runtime_input();
    let oldest = core.current.oldest;
    core.run_now = env.now;
    core.run_wall = env.wall;
    if core.stopping {
        return stopping::empty();
    }
    let mut child = Queue::uncapped();
    step::resume_fleet(core, env, &mut child);
    if !child.is_empty() {
        core.touch_deployment();
        core.touch_run_waits();
    }
    let mut work = Queue::uncapped();
    let mut out = tag_fleet(core, env, &mut work, child);
    assert!(work.is_empty(), "fleet resumption produces no sibling continuations");
    crate::admission::deletion::advance(core, env, &mut out);
    core.finish_extent(oldest, &mut out);
    out
}

fn step_work(core: &mut Core, env: &Env<Limits>, next: Work, work: &mut Queue<Work>) -> Queue<Request> {
    match next {
        Work::StartBrief { .. } | Work::BriefNotes { .. } | Work::PreparationFailed { .. }
            if core.stopping || core.settling =>
        {
            stopping::empty()
        }
        Work::Event(event) => step_one(core, env, event, work),
        Work::StartBrief { task } => start_brief(core, env, work, task),
        Work::BriefNotes { task, lines, more } => plan_brief(core, env, work, task, lines, more),
        Work::PreparationFailed { task, failure } => route_preparation_failed(core, work, task, failure),
        Work::Forget { owner } => crate::reads::forget::forget(core, work, &owner),
    }
}

#[expect(clippy::too_many_lines, reason = "exhaustive event dispatch delegates each concern")]
fn step_one(core: &mut Core, env: &Env<Limits>, event: Event, work: &mut Queue<Work>) -> Queue<Request> {
    let event = match stopping::gate(core, event) {
        Ok(event) => event,
        Err(answer) => return answer,
    };
    match event {
        Event::ChangeModel { reply_to, sign_in, key, digest, act } => settings::dispatch::model_start(
            core,
            env,
            work,
            party::dispatch::Signed { reply_to, sign_in, key },
            digest,
            act,
        ),
        Event::ChangeSettings { reply_to, sign_in, key, digest, change } => settings::dispatch::start(
            core,
            env,
            work,
            party::dispatch::Signed { reply_to, sign_in, key },
            digest,
            change,
        ),
        Event::ProposeSettings { reply_to, sign_in, key, digest, change } => settings::dispatch::propose_start(
            core,
            env,
            work,
            party::dispatch::Signed { reply_to, sign_in, key },
            digest,
            *change,
        ),
        Event::SettingsCommit { room } => core.commit_restored_settings(env, room.saturating_sub(1)),
        Event::ProjectReleased { connector, project } => {
            crate::admission::deletion::released(core, env, connector, project)
        }
        Event::Fill { bytes } => {
            core.fill = bytes;
            stopping::empty()
        }
        Event::Census(value) => {
            core.census = Some(value);
            core.touch_deployment();
            stopping::empty()
        }
        Event::Settling => {
            core.settling = true;
            stopping::empty()
        }
        Event::BriefAssembled { .. }
        | Event::WorkspacePrepared { .. }
        | Event::ClaimPrepared { .. }
        | Event::ClaimRefused { .. }
        | Event::TranscriptLoaded { .. }
        | Event::ProcedureStep { .. }
        | Event::ProcedureHoldings { .. }
        | Event::StartRecurring { .. }
        | Event::Period { .. }
            if core.settling =>
        {
            stopping::empty()
        }
        Event::Stopping => stopping::start(core, env),
        Event::Marked { connector, subjects } => view::routes::marked(core, connector, subjects),
        Event::Act { reply_to, connector, key, act, carried } => {
            party::deployment_acts::start(core, env, work, reply_to, people::ActKey { connector, key }, act, carried)
        }
        Event::Key { reply_to, sign_in, key, ask, carried } => {
            party::key_routes::start(core, env, work, party::dispatch::Signed { reply_to, sign_in, key }, *ask, carried)
        }
        Event::Application { reply_to, sign_in, key, scope, connector, digest, owner, carried } => {
            let input = application_routes::Input { connector, owner, carried: application_routes::number(carried) };
            application_routes::start(
                core,
                env,
                work,
                party::dispatch::Signed { reply_to, sign_in, key },
                scope,
                digest,
                input,
            )
        }
        Event::ApplicationPart { request, ask } => application_routes::part(core, env, work, request, *ask),
        Event::ApplicationFinished { request, outcome } => {
            application_routes::finished(core, env, work, request, outcome)
        }
        Event::ProjectionSettled { goal, connector } => route_projection_settled(core, goal, connector),
        Event::SettledCall { to, key, name, call } => {
            crate::runs::conversation::settle(core, &env.limits, to, key, name, call)
        }
        Event::EffectStart { owner, connector, origin } => crate::effects::start(core, env, owner, connector, origin),
        Event::EffectConnector(event) => crate::effects::connector(core, env, work, event),
        Event::SubscribeTopic { to, key, connector, topic, resources } => {
            effect::topics::subscribe(core, env, work, to, key, (connector, topic), &resources)
        }
        Event::EffectDeadline if !core.ready() => stopping::empty(),
        Event::EffectDeadline => crate::effects::deadline(core, env),
        Event::SignIn { reply_to, .. } | Event::People(people::Event::Ask { reply_to, .. }) if !core.ready() => {
            let mut out = Queue::uncapped();
            out.push(Request::Held(Box::new(Held::PeopleReply {
                to: reply_to,
                reply: people::Reply::Again(people::Again::NotReady),
            })));
            out
        }
        Event::SignIn { reply_to, identity, held } => sign_in(core, env, work, reply_to, identity, held),
        Event::ReadLoaded { load, result } => crate::reads::loaded(core, env, work, load, result),
        Event::PublicRead { reply, read } => crate::public::reads::start(core, reply, read),
        Event::PublicAnswered { reply, answer } => crate::public::reads::answered(core, reply, answer),
        Event::PublicDurable { read } => crate::public::reads::durable(core, env, read),
        Event::Read { reply, sign_in, read } => crate::reads::start(core, env, reply, sign_in, read),
        Event::Watch { watcher, sign_in, subject, window } => open(core, env, watcher, sign_in, subject, window),
        Event::WatchRun { watcher, sign_in, task, turn, offset } => {
            view::routes::open_run(core, env, watcher, sign_in, task, turn, offset)
        }
        Event::Unwatch { watcher } => {
            let Some(watcher) = core.watch_token(watcher) else {
                let mut out = Queue::uncapped();
                out.push(Request::Held(Box::new(Held::WatchEnded { watcher, why: views::End::Unwatched })));
                return out;
            };
            route_view(core, env, views::Event::EndWatch { watcher, end: views::End::Unwatched })
        }
        Event::Delivered { watcher } => {
            let Some(watcher) = core.watch_token(watcher) else { return Queue::uncapped() };
            route_view(core, env, views::Event::Delivered { watcher })
        }
        Event::StreamDelivered { watcher } => {
            let Some(watcher) = core.watch_token(watcher) else { return Queue::uncapped() };
            view::routes::stream_delivered(core, env, watcher)
        }
        Event::RunCall { run, attempt, call } => view::routes::run_call(core, env, run, attempt, call),
        Event::RunInput { run, attempt, turn, retry, offset, id, tool, input } => {
            let written =
                view::routes::Written { run, turn: views::TurnName { attempt: attempt.raw(), turn, retry }, offset };
            view::routes::run_input(core, env, written, id, tool, input)
        }
        Event::RunText { run, attempt, turn, retry, offset, text, call } => {
            let written =
                view::routes::Written { run, turn: views::TurnName { attempt: attempt.raw(), turn, retry }, offset };
            view::routes::run_text(core, env, written, text, call)
        }
        Event::Tasks(event) => {
            let mut out = route_tasks(core, env, work, event);
            crate::admission::deletion::advance(core, env, &mut out);
            out
        }
        Event::StartRecurring { project, authority, template } => {
            route_start_recurring(core, work, project, authority, template)
        }
        Event::BriefAssembled { task, failure } => brief_assembled(core, work, task, failure),
        Event::WorkspacePrepared { task, attempt, writes, holds, credentials, workspace } => workspace_prepared(
            core,
            env,
            work,
            task,
            attempt,
            run::preparation::Prepared { writes, holds, credentials, workspace },
        ),
        Event::ClaimPrepared { task, attempt, budget, writes } => {
            route_claim_prepared(core, env, work, task, attempt, budget, writes)
        }
        Event::ClaimRefused { task } => route_claim_refused(core, work, task),
        Event::PersonTask { seed, executor, llm, spec, contract } => {
            route_person_task(core, work, seed, executor, llm, spec, contract)
        }
        Event::ConnectorRepair { seed, executor, spec, contract, holdings } => {
            route_connector_repair(core, work, seed, executor, spec, contract, holdings)
        }
        Event::Period { project, period, budget, next } => route_period(core, work, project, period, budget, next),
        Event::TranscriptLoaded { task, load } => run::preparation::transcript_loaded(core, env, work, task, load),
        Event::People(event) => route_people(core, env, work, event),
        Event::Holdings { request, connector, holdings, refused } => {
            route_holdings(core, env, work, request, connector, holdings, refused)
        }
        Event::Fleet(event) => {
            let mut out = route_fleet(core, env, work, event);
            crate::admission::deletion::advance(core, env, &mut out);
            out
        }
        Event::TurnPayload { run, attempt, turn, body, read, cumulative, refused } => {
            route_turn_payload(core, work, run, attempt, crate::TurnProof { turn, cumulative, read }, body, refused)
        }
        Event::AnswerPayload {
            run,
            attempt,
            payload,
            cumulative,
            end,
            saved,
            invalid_saved,
            unusable_history,
            read,
            refused,
        } => route_answer_payload(
            core,
            &env.limits.tasks,
            work,
            run,
            attempt,
            run::payloads::Answer { payload, cumulative, end, saved, invalid_saved, unusable_history, read, refused },
        ),
        Event::AcceptedTurn { task, attempt, turn, accepted, cumulative, read, transcript, opens_window } => {
            let record = crate::TurnRecord {
                task,
                attempt,
                turn,
                spent: cumulative,
                read,
                transcript,
                opens_window,
                at: env.wall,
            };
            route_accepted_turn(core, env, accepted, record)
        }
        Event::RefusedPayload { request, problem, payload } => {
            route_refused_payload(core, work, request, problem, payload)
        }
        Event::Activate { context, ready, busy } => route_activate(core, env, work, context, ready, busy),
        Event::EndedReadLoaded { request, task, party, through, busy } => {
            ended_marks::loaded(core, env, work, ended_marks::Answer { request, task, party, through, busy })
        }
        Event::SeenConversationLoaded { request, row, busy } => {
            crate::inbox::seen_conversation_loaded(core, env, work, request, row, busy)
        }
        Event::DecidedProposal { to, key, proposal, row, busy } => {
            party::ended::decided_proposal(core, to, key, proposal, row, busy)
        }
        Event::EndedProposal { request, person, project, proposer, proposal, row, busy } => route_ended_proposal(
            core,
            work,
            party::dispatch::Party { request, person, project, role: None },
            proposer,
            proposal,
            row,
            busy,
        ),
        Event::EndedTaskLoaded { request, row, busy } => run::results::loaded(core, env, work, request, row, busy),
        Event::PartyInputsLoaded { request, row, busy } => {
            let mut routed = party::dispatch::Routed::new(work);
            party::inputs::loaded(core, env, &mut routed, party::inputs::Loaded { request, row, busy });
            routed.finish(work)
        }
        Event::EndedPersonTask { request, person, project, task, row, busy } => {
            let outcome = core.person_task_ended(
                party::dispatch::Party { request, person, project, role: None },
                task,
                row,
                busy,
            );
            work.push(Work::Event(Event::People(people::Event::Decided { request, outcome })));
            Queue::uncapped()
        }
        Event::EndedEscalation { request, person, project, task, revision, row, busy } => route_ended_escalation(
            core,
            env,
            work,
            party::dispatch::Party { request, person, project, role: None },
            party::ended::Escalation { task, revision, row, busy },
        ),
        Event::NamedAnswer { to, key, part } => route_named_answer(core, to, key, part),
        Event::NamedAction { to, key, action } => named_action(core, env, work, to, key, action),
        Event::DelegateValidated { to, key, batch, stubs } => delegate_validated(core, env, to, key, batch, stubs),
        Event::DelegateInputFailed { to, key } => delegate_input_failed(work, to, key),
        Event::DelegateHoldings { request, connector, holdings, refused } => {
            delegate_holdings(core, env, work, request, connector, holdings, refused)
        }
        Event::ProcedureStep { task, step, connector, code, decision: action } => {
            procedure_step(core, env, work, effect::procedure::ProcedureCall { task, step, connector }, code, action)
        }
        Event::ProcedureHoldings { task, step, connector, holdings, refused } => procedure_holdings(
            core,
            env,
            work,
            effect::procedure::ProcedureCall { task, step, connector },
            holdings,
            refused,
        ),
        Event::Brief(event) => route_brief(core, env, event),
        Event::Account(event) => route_account(core, env, work, event),
        Event::View(event) => route_view(core, env, event),
        Event::Notes(event) => route_notes(core, env, work, event),
    }
}

/// Fire one due child timer, returning its whole bounded request batch.
pub fn fire(core: &mut Core, env: &Env<Limits>, timer: Timer) -> Queue<Request> {
    core.start_input();
    if !core.ready() {
        return stopping::empty();
    }
    core.run_now = env.now;
    core.run_wall = env.wall;
    if (core.stopping && jig_domain_core_vocabulary::vocabulary::waits(timer)) || (core.settling && timer.new_work()) {
        return stopping::empty();
    }
    let oldest = core.current.oldest;
    let mut out = match timer {
        Timer::People => {
            let mut out = Queue::uncapped();
            step::fire_people(core, env, &mut out);
            let mut work = Queue::uncapped();
            let requests = party::people::tag_people(core, env, out, &mut work);
            assert!(work.is_empty(), "people expirations produce no sibling continuations");
            requests
        }
        Timer::Tasks => {
            let mut out = Queue::uncapped();
            step::fire_tasks(core, env, &mut out);
            tag_task_outputs(core, env, out)
        }
        Timer::Fleet => {
            let mut out = Queue::uncapped();
            step::fire_fleet(core, env, &mut out);
            if !out.is_empty() {
                core.touch_deployment();
                core.touch_run_waits();
                core.touch_accounts(None);
            }
            let mut work = Queue::uncapped();
            let requests = tag_fleet(core, env, &mut work, out);
            assert!(work.is_empty(), "fleet expirations produce no sibling continuations");
            requests
        }
        Timer::Brief => {
            let mut out = Queue::uncapped();
            step::fire_brief(core, env, &mut out);
            tag_brief(core, env, out)
        }
        Timer::Account if !core.accounts.is_due(env.now) => stopping::empty(),
        Timer::Account => {
            let mut out = Queue::uncapped();
            step::fire_accounts(core, env, &mut out);
            let mut changed = false;
            for request in &out {
                match request {
                    accounts::Request::ReadList { .. } => {}
                    accounts::Request::Answer { .. }
                    | accounts::Request::Programs { .. }
                    | accounts::Request::CheckProgram { .. }
                    | accounts::Request::SignOutProgram { .. }
                    | accounts::Request::Component(_)
                    | accounts::Request::Signing(_)
                    | accounts::Request::Granted { .. }
                    | accounts::Request::Save { .. }
                    | accounts::Request::Erase { .. }
                    | accounts::Request::Closed => changed = true,
                }
            }
            if changed {
                core.touch_deployment();
                core.touch_run_waits();
                core.touch_accounts(None);
            }
            let mut work = Queue::uncapped();
            let requests = tag_account(core, env, &mut work, out);
            assert!(work.is_empty(), "an account's alarm answers no party");
            requests
        }
    };
    crate::admission::deletion::advance(core, env, &mut out);
    core.finish_extent(oldest, &mut out);
    out
}

pub(crate) fn tag_ended_mark(
    core: &mut Core,
    env: &Env<Limits>,
    child: Queue<people::Request>,
    work: &mut Queue<Work>,
) -> Queue<Request> {
    party::people::tag_people(core, env, child, work)
}

/// Drain deletion's bounded sibling decisions without a new root input or extent.
pub(crate) fn deletion_people(core: &mut Core, env: &Env<Limits>, event: people::Event, out: &mut Queue<Request>) {
    children::append_people(core, env, event, out);
}

/// Drain deletion's bounded sibling decisions without a new root input or extent.
pub(crate) fn deletion_event(core: &mut Core, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    let previous = core.deletions.route(true);
    let mut work = Queue::uncapped();
    work.push(Work::Event(event));
    for _ in 0..env.limits.routes {
        let Some(event) = work.pop() else { break };
        let mut batch = step_work(core, env, event, &mut work);
        for _ in 0..batch.len() {
            let request = batch.pop().expect("deletion child output count");
            match request {
                request @ (Request::ProjectMade { .. }
                | Request::Write(_)
                | Request::Ask { .. }
                | Request::Held(_)
                | Request::Now(_)) => out.push(request),
            }
        }
    }
    assert!(work.is_empty(), "deletion sibling routes drained");
    let _previous = core.deletions.route(previous);
}
