//! Exhaustive child routing and synchronous handoffs (jig's domain/root.md, 5).
mod applications;
pub(crate) mod approvals;
mod asks;
pub(crate) mod calendar;
mod diagnostics;
pub(crate) mod escalation;
pub(crate) mod forge_route;
pub(crate) mod held;
pub(crate) mod host_route;
mod making;
pub(crate) mod playground;
pub(crate) mod standing;
pub use host_route::{
    AskHow, ForgePerson, HostDelivery, HostRequest, OfferedModel, PersonForge, ProjectPerson, RepositoryIdentity,
};
mod decided;
pub(crate) mod document;
mod finding_tool;
pub(crate) mod findings;
mod follow_tool;
mod goal;
mod kinds;
pub(crate) mod landing;
mod now;
mod panels;
pub(crate) mod policy_translate;
pub(crate) mod proposals;
pub(crate) mod public;
pub(crate) mod remember;
pub(crate) mod see;
pub(crate) mod ways;
pub use see::{
    SeeAnswer, SeeLanding, SeeName, SeeNames, SeeNotRead, SeePerson, SeeProposal, SeeRead, SeeRepository, SeeSettings,
    SeeWords,
};
pub(crate) mod settings_parts;
mod tools;
pub(crate) mod trials;
use crate::archive::take_read;
use crate::assemble::{brief_outputs, take_payload};
use crate::boundary::{
    Call, EscalationChoice, Event, ForgeRepository, ForgeStart, ForgeWorkspace, InputCheck, MessageForm, Payload,
    PreparedWorkspace, ProposalChoice, ProposedAction, Read, Request, Tool, Work,
};
use crate::domain::{
    Domain, close, emit, forge_release_ending, internal, journal_outputs, now, route, save, step_released,
};
use crate::limits::{Limits, core_limits, environment_core};
use crate::restart::{accept_load, connector_start_done, load_decision, load_outputs, start_action};
use crate::translate::{
    call_needs_input, checked_call, connector_answer, core_call, environment_forge, keep_connector_answer, named_call,
    relay_call, tool_kind,
};
use crate::{CallAnswer, CallKey, Decision, Delivery, Family, Key, Range, Record, Write, loads};
use alloc::boxed::Box;
pub use forge_route::adopt_repository_ask;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_core::{Delegate, ProcedureStep, StepEnd};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
pub use landing::{Approval, Freshness, Gate, LandingRule};
use skein_lib::{Env, Id, List, Queue, ReplyTo, Token};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as forge_change;
use temper_engine_domain_forge_issues as forge_issues;
pub(crate) use tools::relay_payload;

/// Route one admitted input and its synchronous continuations in one decision.
/// Admission is `try_step`'s, including public reading's immediate busy answer
/// (jig/docs/design/domain/root.md, 3).
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive input match keeps every input before a single decision close"
)]
pub(crate) fn step_routed(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    crate::domain::invalidate_input(domain, &event);
    assert!(domain.application.is_none(), "application value cleared in its admission step");
    assert!(domain.limits == env.limits, "root uses configured limits");
    if domain.journal.stopped() || domain.start_stop().is_some() {
        discard_after_stop(domain, event);
        return;
    }
    let event = match crate::stopping::defer(domain, event) {
        Ok(event) => event,
        Err(forge) => {
            out.push(Request::Busy(forge));
            return;
        }
    };
    match event {
        Event::Setup { .. } => return, // The root keeps nothing of a plan's terminal.
        Event::Resume => {
            crate::domain::resume_routed(domain, env, out);
            return;
        }
        Event::Released(released) => {
            step_released(domain, env, released, out);
            return;
        }
        Event::Forge(event) => {
            if !domain.ready()
                && let forge::Event::Hint { .. } = &*event
            {
                return;
            }
            domain.work.push(Work::Forge(*event));
        }
        Event::Stop => crate::settling::start(domain),
        Event::Credential { number, generation, valid } => {
            domain.work.push(Work::Core(core::Event::Account(jig_domain_accounts::Event::Credential {
                number,
                generation,
                valid,
            })));
        }
        Event::Account(event) => domain.work.push(Work::Core(core::Event::Account(event))),
        Event::Party(crate::PartyInput::PublicRead { reply, read }) => {
            let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
            public::start(domain, env, &mut decision, reply, read);
            route_into(domain, env, &mut decision);
            close(domain, env, decision, out);
            return;
        }
        Event::Party(crate::PartyInput::StartRecurring { project, authority, template }) => {
            domain.work.push(Work::Core(core::Event::StartRecurring { project, authority, template }));
        }
        Event::Party(crate::PartyInput::Period { project, period, budget, next }) => match next {
            None if project != 0 && period >= domain.core.period(project) => {
                let times = match domain.core.tasks.funding(tasks::Funder::Period { project, period }) {
                    Some(row) => row.period,
                    None => None,
                };
                match times {
                    Some(times) if times.next.is_some() => {
                        // Replaying the first source cannot move a durable boundary.
                        domain.work.push(Work::Core(core::Event::Period { project, period, budget, next: times.next }));
                    }
                    Some(times) => calendar::route(
                        domain,
                        env,
                        temper_engine_domain_calendar::Event::Restored {
                            project,
                            period,
                            began: times.began,
                            next: None,
                        },
                        Some(budget),
                    ),
                    None => calendar::route(
                        domain,
                        env,
                        temper_engine_domain_calendar::Event::Began { project, period, began: env.wall },
                        Some(budget),
                    ),
                }
            }
            None | Some(_) => domain.work.push(Work::Core(core::Event::Period { project, period, budget, next })),
        },
        Event::Party(crate::PartyInput::ProcedureStep { task, step, connector, code, decision }) => {
            domain.work.push(Work::Core(core::Event::ProcedureStep { task, step, connector, code, decision }));
        }
        Event::Party(crate::PartyInput::Whose { reply_to, sign_in }) => {
            domain.work.push(Work::People(people::Event::Whose { reply_to, sign_in }));
        }
        Event::Party(crate::PartyInput::ProposeSettings { reply_to, sign_in, key, digest, change, kinds }) => {
            domain.kinds = kinds;
            match room_refusal(domain, &change.items) {
                Some(why) => {
                    let request = Request::Party(crate::PartyRequest::Deliver(Delivery::WebReply {
                        to: reply_to,
                        reply: people::Reply::Refused(why),
                    }));
                    assert!(now(domain, request), "settings refusal door reserved");
                }
                None => domain.work.push(Work::Core(core::Event::ProposeSettings {
                    reply_to,
                    sign_in,
                    key,
                    digest,
                    change,
                })),
            }
        }
        Event::Party(crate::PartyInput::ChangeModel { reply_to, sign_in, key, digest, act }) => {
            domain.work.push(Work::Core(core::Event::ChangeModel { reply_to, sign_in, key, digest, act }));
        }
        Event::Party(crate::PartyInput::ChangeSettings { reply_to, sign_in, key, digest, change, kinds }) => {
            domain.kinds = kinds;
            match room_refusal(domain, &change.items) {
                Some(why) => {
                    let request = Request::Party(crate::PartyRequest::Deliver(Delivery::WebReply {
                        to: reply_to,
                        reply: people::Reply::Refused(why),
                    }));
                    assert!(now(domain, request), "settings refusal door reserved");
                }
                None => {
                    domain.work.push(Work::Core(core::Event::ChangeSettings {
                        reply_to,
                        sign_in,
                        key,
                        digest,
                        change,
                    }));
                }
            }
        }
        Event::Party(crate::PartyInput::SignOut { reply_to, sign_in }) => {
            domain.work.push(Work::People(people::Event::SignOut { reply_to, sign_in }));
        }
        Event::Party(crate::PartyInput::SignedIn { reply_to, identity, held }) => {
            domain.work.push(Work::Core(core::Event::SignIn { reply_to, identity, held }));
        }
        Event::Party(crate::PartyInput::Application { reply_to, sign_in, key, digest, scope, request, carried }) => {
            let admission = applications::Admission { reply_to, sign_in, key, digest, scope, carried };
            applications::start(domain, admission, request);
        }
        Event::Party(crate::PartyInput::Ask { reply_to, sign_in, key, ask, models }) => {
            domain.work.push(Work::People(people::Event::Ask { reply_to, sign_in, key, ask, models }));
        }
        Event::Party(crate::PartyInput::Key { reply_to, sign_in, key, ask, carried }) => {
            domain.work.push(Work::Core(core::Event::Key { reply_to, sign_in, key, ask, carried }));
        }
        Event::Party(crate::PartyInput::ForgeRead { reply, sign_in, scope, query }) => {
            domain.work.push(Work::ForgeRead { reply, sign_in, scope, query });
        }
        Event::Party(crate::PartyInput::Read { reply_to, sign_in, read }) => {
            domain.work.push(Work::Core(core::Event::Read { reply: reply_to, sign_in, read }));
        }
        Event::Party(crate::PartyInput::Watch { watcher, sign_in, subject, window }) => {
            domain.work.push(Work::Core(core::Event::Watch { watcher, sign_in, subject, window }));
        }
        Event::Party(crate::PartyInput::WatchRun { watcher, sign_in, task, turn, offset }) => {
            domain.work.push(Work::Core(core::Event::WatchRun { watcher, sign_in, task, turn, offset }));
        }
        Event::Party(crate::PartyInput::Unwatch { watcher }) => {
            domain.work.push(Work::Core(core::Event::Unwatch { watcher }));
        }
        Event::Party(crate::PartyInput::Delivered { watcher }) => {
            domain.work.push(Work::Core(core::Event::Delivered { watcher }));
        }
        Event::Party(crate::PartyInput::StreamDelivered { watcher }) => {
            domain.work.push(Work::Core(core::Event::StreamDelivered { watcher }));
        }
        Event::Store(crate::StoreInput::Committed { number, fill }) => {
            crate::committed(&mut domain.journal, number);
            if domain.journal.stopped() {
                domain.journal_stop = Some(crate::JournalStop::Refused {
                    commit: number,
                    sent: domain.sent_commit,
                    durable: domain.durable_commit,
                });
                out.push(Request::Stop);
                return;
            }
            domain.durable_commit = number;
            if domain.start_runtime() {
                let _filled = core::step(&mut domain.core, &environment_core(env), core::Event::Fill { bytes: fill });
            } else {
                domain.startup_fill = Some(fill);
            }
            return;
        }
        Event::Store(crate::StoreInput::Uncommitted { number }) => {
            let mut journal_out = Queue::with_capacity(1);
            crate::uncommitted(&mut domain.journal, number, &mut journal_out);
            journal_outputs(&mut journal_out, out);
            return;
        }
        Event::Store(crate::StoreInput::Loaded { owner, rows, next, texts }) => {
            let mut decision = Some(load_decision(domain, &env.limits));
            let mut load_out = Queue::with_capacity(1);
            loads::loaded(&mut domain.loads, owner, rows, next, &mut load_out);
            load_outputs(domain, env, &mut load_out, texts, &mut decision, out);
            accept_load(domain, env, decision);
            return;
        }
        Event::Store(crate::StoreInput::Unloaded { owner }) => {
            let mut decision = Some(load_decision(domain, &env.limits));
            let mut load_out = Queue::with_capacity(1);
            loads::unloaded(&mut domain.loads, owner, &mut load_out);
            load_outputs(domain, env, &mut load_out, None, &mut decision, out);
            accept_load(domain, env, decision);
            return;
        }
        Event::Store(crate::StoreInput::MissingLink { owner, record, target }) => {
            if loads::startup_owner(&domain.loads, owner) {
                domain.core.start_missing(record, target);
            }
            return;
        }
        Event::Store(crate::StoreInput::Switched) => {
            let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
            domain.work.push(Work::StartDone(core::start::Phase::Switch));
            route_into(domain, env, &mut decision);
            close(domain, env, decision, out);
            return;
        }
        Event::Store(crate::StoreInput::ReadLoaded { load, result }) => {
            domain.work.push(Work::Core(core::Event::ReadLoaded { load, result }));
        }
        Event::Restart { pending } => {
            if domain.start_wall.is_some() {
                return;
            }
            domain.start_wall = Some(env.wall);
            domain.start_pending = pending;
            domain.forge.restore_begin(env.wall);
            domain.settled = None;
            let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
            let action = domain.core.start_begin(pending, env.wall);
            start_action(domain, env, &mut decision, action);
            route_into(domain, env, &mut decision);
            close(domain, env, decision, out);
            return;
        }
        Event::Worker(crate::WorkerInput::Opened { worker, statement }) => {
            domain.openings.push(crate::WorkerOpening { worker, statement, at: env.wall });
            return;
        }
        Event::Worker(crate::WorkerInput::Link { channel, link }) => {
            // Admission keeps room for the record's bytes; past it the
            // record is dropped and its host sends it again after a hello.
            if let Some(event) = crate::link::receive(domain, channel, link) {
                domain.work.push(Work::Core(event));
            }
        }
        Event::Worker(crate::WorkerInput::DecodedHostCall { to, body }) => {
            domain.work.push(Work::Decoded { to, body });
        }
        Event::Worker(crate::WorkerInput::RenderedHostCall { to, answer }) => host_route::rendered(domain, to, answer),
        Event::Worker(crate::WorkerInput::ResultRead { payload, end }) => {
            if let Some(event) = crate::link::result_event(domain, payload, end) {
                domain.work.push(Work::Core(event));
            }
        }
    }
    let decision = route(domain, env);
    close(domain, env, decision, out);
}

fn keep_produced_title(domain: &mut Domain, record: &core::Record) {
    if let core::Record::Tasks(tasks::Stored::Ended(row)) = record
        && let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { title, words, .. })) = &row.phase
        && let tasks::Party::Task(task) = row.requester
    {
        domain.work.push(Work::Forge(forge::Event::ProducedTitle {
            task,
            child: row.number,
            title: title.words.clone(),
            report: words.clone(),
        }));
    }
}

// A done end refreshes the change before its queued Met activation can step it
// (temper/docs/design/domain/forge/change/repairs.md).
fn dependencies_met(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, record: &core::Record) {
    let core::Record::Tasks(tasks::Stored::Ended(ended)) = record else { return };
    let tasks::Phase::Ended(tasks::Ending::Done(_)) = ended.phase else { return };
    for live in domain.core.tasks.view_tasks() {
        let Some(row) = domain.core.tasks.task(live.number) else { continue };
        if let tasks::Phase::Held { why: tasks::Hold::Dependency(_), .. } = row.phase {
            continue;
        }
        if row.dependencies.contains(&ended.number)
            && row.waiting_on.is_empty()
            && domain.forge.change(live.number).is_some()
        {
            forge_event(domain, env, decision, forge::Event::DependenciesMet { task: live.number });
        }
    }
}

/// Route one core batch: writes join the decision, asks and outputs go to their owners.
pub(crate) fn route_core_requests(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    requests: Queue<core::Request>,
) {
    standing::saved(domain, env, decision, &requests);
    let mut output = requests;
    for _ in 0..output.len() {
        match output.pop().expect("core output count") {
            core::Request::Write(core::Write::Save(record)) => {
                forge_route::follow_route::saved(domain, &record);
                trials::saved(domain, env, &record);
                ways::saved(domain, &record);
                keep_produced_title(domain, &record);
                dependencies_met(domain, env, decision, &record);
                forge_route::close_saved(domain, &record);
                forge_route::job_history::saved(domain, &record);
                if let core::Record::Core(core::CoreRecord::Call(row)) = &record {
                    let answer = forge_route::effect_answer(domain, row.key, &row.part);
                    if let core::CallPart::Effect { .. } = row.part {
                        keep_connector_answer(domain, row.key, answer).expect("admitted effect call payload");
                    }
                    save_connector_answer(domain, &env.limits, decision, row.key);
                }
                let replaced = save(decision, &env.limits, Write::Save(Record::Core(record)));
                domain.release_bytes(replaced, "replaced write's turn body was held");
            }
            core::Request::Write(core::Write::DropTranscript { task }) => {
                save(decision, &env.limits, Write::DropTranscript { task });
            }
            core::Request::Write(core::Write::CopySettingsRevision(number)) => {
                save(decision, &env.limits, Write::CopySettingsRevision(number));
            }
            core::Request::Write(core::Write::Erase(key)) => {
                if let core::Key::Accounts(jig_domain_accounts::Key::Account { number }) = &key {
                    trials::route(domain, env, temper_engine_domain_trials::Event::AccountGone { account: *number });
                }
                if let core::Key::Core(core::CoreKey::Call(call)) = key
                    && domain.forge.forget_named_answer(named_call(call)).is_some()
                {
                    save(decision, &env.limits, Write::Erase(Key::Forge(forge::Key::Call(named_call(call)))));
                }
                let replaced = save(decision, &env.limits, Write::Erase(Key::Core(key)));
                domain.release_bytes(replaced, "erased write's turn body was held");
            }
            core::Request::Ask { connector, ask } => asks::ask(domain, env, decision, connector, ask),
            core::Request::Held(output) => {
                let Some(output) = making::held(domain, *output) else { continue };
                if let core::Held::PublicRead { reply, answer } = output {
                    emit(decision, &env.limits, Delivery::Public { reply, answer, admitted: true });
                } else {
                    held::held(domain, env, decision, output);
                }
            }
            core::Request::Now(output) => now::now(domain, env, decision, *output),
            core::Request::ProjectMade { project } => emit(decision, &env.limits, Delivery::ProjectMade { project }),
        }
    }
    crate::assemble::retire_document_titles(domain);
}

#[expect(clippy::too_many_lines, reason = "one exhaustive internal work dispatch, including startup refusals")]
pub(crate) fn route_into(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    loop {
        if domain.start_stop().is_some() {
            return;
        }
        if domain.core.start_failure().is_some() {
            domain.stop_core();
            return;
        }
        if refuse_forge_fit(domain) {
            return;
        }
        trials::observe(domain, env);
        let Some(work) = domain.work.pop() else {
            if domain.ready() && forge_route::form_landings(domain, env) {
                continue;
            }
            break;
        };
        match work {
            Work::StartDone(phase) => {
                if phase == core::start::Phase::LoadCore
                    && let Err(short) = domain.check_room()
                {
                    domain.stop_start(crate::StartStop::Room(short));
                    return;
                }
                let mut output = Queue::uncapped();
                let action = domain.core.start_done(&environment_core(env), phase, &mut output);
                route_core_requests(domain, env, decision, output);
                if phase == core::start::Phase::Open && domain.start_phase().is_none() {
                    domain.opened = true;
                    // Open admits task control; a later resume takes the first due task.
                    trials::restarted(domain);
                }
                domain.work.push(Work::Start(action));
            }
            Work::Start(action) => start_action(domain, env, decision, action),
            Work::Decoded { to, body } => host_route::decoded(domain, env, decision, to, body),
            Work::ForgeRead { reply, sign_in, scope, query } => {
                kinds::read(domain, env, decision, reply, sign_in, scope, query);
            }
            Work::Core(event) => {
                if let core::Event::AcceptedTurn { accepted: tasks::Accepted::Already, transcript, .. } = &event {
                    domain.release_bytes(skein_lib::len_u64(transcript.len()), "already accepted turn body was held");
                }
                let period = if let core::Event::Period { project, .. } = &event { Some(*project) } else { None };
                let routed = forge_route::step_core(domain, env, event);
                route_core_requests(domain, env, decision, routed);
                if let Some(project) = period {
                    standing::move_roles(domain, project, None);
                }
            }
            Work::Tasks(event) => {
                let routed = forge_route::step_core(domain, env, core::Event::Tasks(event));
                route_core_requests(domain, env, decision, routed);
            }
            Work::People(event) => {
                let routed = forge_route::step_core(domain, env, core::Event::People(event));
                route_core_requests(domain, env, decision, routed);
            }
            Work::Fleet(event) => {
                let routed = forge_route::step_core(domain, env, core::Event::Fleet(event));
                route_core_requests(domain, env, decision, routed);
            }
            Work::Brief(event) => {
                let routed = forge_route::step_core(domain, env, core::Event::Brief(event));
                route_core_requests(domain, env, decision, routed);
            }
            Work::ForgeRestored => {
                settings_parts::restored(domain);
                domain.forge.link_names(&domain.core.tasks, &env.limits.forge);
                domain.forge.link_tiers(&domain.core.tasks, &env.limits.forge);
                if domain.forge.fit_restored(&env.limits.forge).is_err() {
                    refuse_forge_fit(domain);
                    return;
                }
                if let Some(link) = domain.forge.missing_link(&domain.core.tasks) {
                    domain.stop_start(crate::StartStop::ForgeLink { phase: domain.start_phase(), link });
                    return;
                }
                let party = domain.core.people.sole_owner();
                forge_event(domain, env, decision, forge::Event::Owner { party });
                forge_event(
                    domain,
                    env,
                    decision,
                    forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Wall },
                );
            }
            Work::Forge(event) => forge_event(domain, env, decision, event),
            Work::ForgeSettings(mut output) => forge_route::outputs(domain, env, decision, &mut output),
            Work::ProjectGoal(goal) => forge_route::project_goal(domain, env, &goal),
            Work::GoalSubscribe(subscriber) => forge_route::goal_subscribed(domain, env, subscriber),
            Work::AgreedGoal(read) => goal::ready(domain, env, decision, *read),
            Work::Activate { context, ready } => activate(domain, env, decision, context, ready),
            Work::EscalationLoaded { waiter, rows } => escalation::loaded(domain, env, waiter, rows),
            Work::EscalationFailed { waiter } => escalation::failed(domain, waiter),
            Work::ProposalLoaded { waiter, rows } => proposals::historical_loaded(domain, waiter, rows),
            Work::ProposalFailed { waiter } => proposals::historical_failed(domain, waiter),
            Work::TaskProposalLoaded { waiter, rows } => proposals::task_loaded(domain, waiter, rows),
            Work::EndedLoaded { waiter, rows } => crate::archive::ended_loaded(domain, waiter, rows),
            Work::EndedFailed { waiter } => crate::archive::ended_failed(domain, waiter),
        }
    }
}

/// One activation's step: the load it allows finds both its read slot and its load slot.
fn activate(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    context: Box<tasks::RunContext>,
    ready: bool,
) {
    let due = match domain.core.tasks.task(context.task) {
        Some(task) => task.phase == tasks::Phase::Active(tasks::Active::Due),
        None => false,
    };
    if !domain.core.settling() && ready && due && domain.forge.design_workspace_waiting(context.task) {
        forge_event(domain, env, decision, forge::Event::GoalReleased { task: context.task });
    }
    let busy = domain.archive_reads.is_full() || !domain.loads.has_room();
    let routed = core::step(&mut domain.core, &environment_core(env), core::Event::Activate { context, ready, busy });
    route_core_requests(domain, env, decision, routed);
}

pub(crate) fn call_answer(domain: &Domain, key: CallKey) -> Option<CallAnswer> {
    match domain.core.replay_call(key) {
        Some(core::CallReplay::Core(part)) => Some(forge_route::effect_answer(domain, key, &part)),
        Some(core::CallReplay::Connector { .. }) => connector_answer(domain, key),
        None => None,
    }
}

pub(crate) fn decide_call(
    domain: &mut Domain,
    limits: &Limits,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    answer: CallAnswer,
) {
    let part = answer.core_part(domain.config.forge_connector);
    let connector_owned = match &part {
        core::CallPart::Connector { .. } | core::CallPart::Effect { .. } => true,
        core::CallPart::EffectDenied { .. }
        | core::CallPart::ToolDenied { .. }
        | core::CallPart::EscalationDecided { .. }
        | core::CallPart::EscalationRefused(_)
        | core::CallPart::Proposed { .. }
        | core::CallPart::ProposalDecided { .. }
        | core::CallPart::ProposalRefused(_)
        | core::CallPart::Again { .. }
        | core::CallPart::PassedUp { .. }
        | core::CallPart::Revised { .. }
        | core::CallPart::Moved { .. }
        | core::CallPart::Controlled
        | core::CallPart::EffectRefused(_)
        | core::CallPart::ControlRefused(_)
        | core::CallPart::ControlDenied { .. }
        | core::CallPart::Sent { .. }
        | core::CallPart::Introduced
        | core::CallPart::MessageRefused(_)
        | core::CallPart::Subscribed { .. }
        | core::CallPart::Unsubscribed
        | core::CallPart::SubscriptionRefused(_)
        | core::CallPart::Delegated(_)
        | core::CallPart::DelegationDenied { .. }
        | core::CallPart::DelegationRefused(_)
        | core::CallPart::NoteWritten { .. }
        | core::CallPart::NoteDeleted { .. }
        | core::CallPart::NoteRecalled { .. }
        | core::CallPart::NoteRefused(_)
        | core::CallPart::SettingsRefused(_)
        | core::CallPart::HoldOwner { .. }
        | core::CallPart::Unavailable => false,
    };
    if !domain.core.decide_named_call(key, part) {
        relay_call(domain, limits, decision, to, answer);
        return;
    }
    if connector_owned {
        assert!(keep_connector_answer(domain, key, answer.clone()) == Ok(None), "connector answer room reserved");
    }
    save_connector_answer(domain, limits, decision, key);
    save(
        decision,
        limits,
        Write::Save(Record::Core(core::Record::Core(core::CoreRecord::Call(
            domain.core.call_record(key).expect("admitted core call"),
        )))),
    );
    relay_call(domain, limits, decision, to, answer);
}

#[expect(clippy::too_many_arguments, reason = "validated historical input pointers travel with the delegated call")]
pub(crate) fn delegate_call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    batch: Box<[Delegate]>,
    validated: bool,
    stubs: Box<[tasks::Stub]>,
) {
    if let Some(context) = domain.core.tasks.delegation(key.task)
        && let Err(body) = forge_route::follow_route::check_batch(
            domain,
            env,
            context.project,
            tasks::Party::Task(key.task),
            &batch,
            &[],
        )
    {
        host_route::rendered(domain, to, core::SettledAnswer::Host { error: true, body });
        return;
    }
    if let Err(part) = domain.core.delegate_preflight(&env.limits.tasks, key, &batch, env.wall) {
        domain.work.push(Work::Core(core::Event::NamedAnswer { to, key, part: *part }));
        return;
    }
    if !validated {
        let (project, ids) = match domain.core.delegate_inputs(&env.limits.tasks, key, &batch) {
            Ok(value) => value,
            Err(part) => {
                domain.work.push(Work::Core(core::Event::NamedAnswer { to, key, part: *part }));
                return;
            }
        };
        if let Some(&first) = ids.first() {
            let capacity = batch
                .len()
                .checked_mul(skein_lib::usize_of(env.limits.tasks.inputs))
                .expect("bounded batch input count");
            let to = to.into_token();
            let read = InputCheck {
                to,
                key,
                batch,
                ids,
                at: 0,
                project,
                stubs: List::with_capacity(u32::try_from(capacity).expect("bounded input IDs")),
            };
            let waiter =
                domain.archive_reads.insert(Some(Read::InputCheck(read))).expect("preflighted input read slot").token();
            assert!(domain.core.reserve_connector_call(key), "reserved call record room");
            emit(
                decision,
                &env.limits,
                Delivery::Load {
                    waiter,
                    range: Range::Core(crate::CoreRange::TaskResult { task: first }),
                    after: None,
                },
            );
            return;
        }
    }
    domain.work.push(Work::Core(core::Event::DelegateValidated { to, key, batch, stubs }));
}

pub(crate) fn discard_after_stop(domain: &mut Domain, event: Event) {
    match event {
        Event::Store(crate::StoreInput::Loaded { owner, rows, next, .. }) => {
            loads::abandon(&mut domain.loads, owner);
            let mut out = Queue::with_capacity(1);
            loads::loaded(&mut domain.loads, owner, rows, next, &mut out);
            assert!(out.is_empty(), "halted waiter emits no delivery");
        }
        Event::Store(crate::StoreInput::Unloaded { owner }) => {
            loads::abandon(&mut domain.loads, owner);
            let mut out = Queue::with_capacity(1);
            loads::unloaded(&mut domain.loads, owner, &mut out);
            assert!(out.is_empty(), "halted waiter emits no delivery");
        }
        Event::Released(_)
        | Event::Worker(_)
        | Event::Party(_)
        | Event::Forge(_)
        | Event::Credential { .. }
        | Event::Setup { .. }
        | Event::Restart { .. }
        | Event::Resume
        | Event::Stop
        | Event::Store(
            crate::StoreInput::MissingLink { .. }
            | crate::StoreInput::Switched
            | crate::StoreInput::Committed { .. }
            | crate::StoreInput::Uncommitted { .. }
            | crate::StoreInput::ReadLoaded { .. },
        )
        | Event::Account(_) => {}
    }
}

pub(crate) fn current_proof(domain: &Domain, task: u64, attempt: u64) -> bool {
    domain.core.current_proof(task, attempt)
}

pub(crate) fn save_connector_answer(domain: &Domain, limits: &Limits, decision: &mut Decision, key: CallKey) {
    if let Some(answer) = domain.forge.named_answer(named_call(key)) {
        save(
            decision,
            limits,
            Write::Save(Record::Forge { row: Box::new(forge::Stored::Call { key: named_call(key), answer }) }),
        );
    }
}

fn forge_event(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, event: forge::Event) {
    let mut out = Queue::uncapped();
    if let forge::Event::PersonOf { .. } = &event {
        let party = domain.core.people.sole_owner();
        forge::step(&mut domain.forge, &environment_forge(env), forge::Event::Owner { party }, &mut out);
    }
    forge::step(&mut domain.forge, &environment_forge(env), event, &mut out);
    if refuse_forge_fit(domain) {
        return;
    }
    forge_route::outputs(domain, env, decision, &mut out);
}

// Refuse at the step still waiting for this connector, before any output translation.
pub(crate) fn refuse_forge_fit(domain: &mut Domain) -> bool {
    if domain.forge_fit_failure().is_none() {
        return false;
    }
    let failure = *domain.forge_fit_failure().expect("forge fit failure");
    domain.stop_start(crate::StartStop::ForgeFit { phase: domain.start_phase(), failure });
    true
}

/// Only charter values set at an entrance are checked; resets and stored
/// proposal acceptances retain their admitted values (temper/docs/design/domain/agent/tools.md).
pub(crate) fn room_refusal(domain: &Domain, items: &[core::settings::Item]) -> Option<people::Refusal> {
    use core::settings::{Edit, Setting, Value};
    let host = domain.host_part?;
    for (index, item) in items.iter().enumerate() {
        if let Setting::Charter(_) = item.setting
            && let Edit::Set(value) = &item.edit
        {
            let made;
            let charter = match value {
                Value::Charter(charter) => charter,
                Value::Overlay(overlay) => {
                    let Some(Value::Charter(default)) = domain.core.current.default_value(item.setting) else {
                        // Jig refuses an overlay without a default as reference.
                        continue;
                    };
                    made = core::settings::overlaid(default, overlay);
                    &made
                }
                Value::Template(_) | Value::Policy(_) | Value::Rules(_) | Value::Model(_) | Value::Application(_) => {
                    continue;
                }
            };
            if let Err(short) = host.check(charter) {
                return Some(people::Refusal::Judged {
                    at: u32::try_from(index).expect("bounded settings items"),
                    setting: item.setting,
                    why: short.words(),
                });
            }
        }
    }
    None
}
