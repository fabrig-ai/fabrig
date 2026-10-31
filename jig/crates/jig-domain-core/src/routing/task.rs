//! Task-child decisions routed to people, hosts and connectors
//! (`domain/engine.md`, section 4.4).

mod done;
mod escalations;
mod lifecycle;
mod made;
mod notices;
mod proposals;
mod refused;
mod save;
mod terminals;

use crate::{
    Core, Event, Limits, Request,
    routing::{
        Work,
        flight::{Flight, Internal},
    },
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo};

pub(crate) fn tag_task_outputs(core: &mut Core, env: &Env<Limits>, mut child: Queue<tasks::Request>) -> Queue<Request> {
    let mut pending = Queue::uncapped();
    for _ in 0..child.len() {
        pending.push(child.pop().expect("task output count"));
    }
    let mut out = Queue::uncapped();
    for _ in 0..env.limits.routes {
        let Some(next) = pending.pop() else { break };
        tag_tasks(core, env, next, &mut pending, &mut out);
    }
    let holders = core.tasks.live_agent_tasks();
    if holders != core.settings_holders {
        core.settings_holders = holders;
        core.touch_settings();
    }
    let capacity_mark = core.tasks.capacity_mark();
    if capacity_mark != core.task_capacity_mark || capacity_mark == u64::MAX {
        core.task_capacity_mark = capacity_mark;
        core.touch_deployment();
    }
    assert!(pending.is_empty(), "task_routes_drained");
    out
}

fn route_proposal(core: &mut Core, env: &Env<Limits>, event: tasks::Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let tasks::Event::Make { batch, .. } | tasks::Event::MakeResultFollowups { batch, .. } = &event
        && let Err(problem) = core.batch_accounts(batch)
    {
        let (tasks::Event::Make { reply_to, .. } | tasks::Event::MakeResultFollowups { reply_to, .. }) = event else {
            unreachable!("checked batch event")
        };
        out.push(tasks::Request::Refused { reply_to, problem });
        return tag_task_outputs(core, env, out);
    }
    if let tasks::Event::Amend { task, amendment, funder, .. } = &event
        && let Err(why) = core.amendment_account(*task, amendment, *funder)
    {
        let problem = tasks::Problem::new(Some(*task), why);
        let tasks::Event::Amend { reply_to, .. } = event else { unreachable!("checked amendment event") };
        out.push(tasks::Request::Refused { reply_to, problem });
        return tag_task_outputs(core, env, out);
    }
    super::step::step_tasks(core, env, event, &mut out);
    tag_task_outputs(core, env, out)
}

#[expect(clippy::boxed_local, reason = "the moved route retains its owned event payload")]
pub(crate) fn route_connector_repair(
    core: &mut Core,
    work: &mut Queue<Work>,
    seed: Box<crate::effects::connector::RepairSeed>,
    executor: tasks::Executor,
    spec: tasks::Spec,
    contract: tasks::Contract,
    holdings: Box<[tasks::Holding]>,
) -> Queue<Request> {
    if seed.open_period {
        work.push(Work::Event(Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(core.internal(Internal::Admission)),
            project: seed.project,
            period: seed.period,
            budget: seed.period_budget,
        })));
    }
    work.push(Work::Event(Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(core.internal(Internal::Repair)),
        creator: tasks::Party::Deployment { project: seed.project },
        batch: Box::new([tasks::New {
            deadline: jig_domain_tasks::Deadline::Named,
            label: 0,
            flags: jig_domain_tasks::Flags::PLAIN,
            number: seed.number,
            project: seed.project,
            executor,
            llm: None,
            spec,
            contract,
            numbers: tasks::Numbers { budget: seed.budget, spent: 0, spent_below: 0, reserved: 0 },
            authority: seed.authority,
            funder: tasks::Funder::Period { project: seed.project, period: seed.period },
            dependencies: Box::new([]),
            holdings,
            wake: tasks::WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        }]),
    })));
    Queue::uncapped()
}

pub(crate) fn route_tasks(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    mut event: tasks::Event,
) -> Queue<Request> {
    if let Some(token) = super::flight::request_token(&mut event)
        && let Some(Flight::PersonProposal(_)) = core.flights.get(skein_lib::Id::from_token(token))
    {
        return route_proposal(core, env, event);
    }
    if let tasks::Event::Procedure { task, step, .. } = &event {
        return super::effect::procedure::apply(core, env, *task, *step, event);
    }
    let mut out = Queue::uncapped();
    if let tasks::Event::Amend { task, amendment, funder, .. } = &event
        && let Err(why) = core.amendment_account(*task, amendment, *funder)
    {
        let problem = tasks::Problem::new(Some(*task), why);
        let tasks::Event::Amend { reply_to, .. } = event else { unreachable!("checked amendment event") };
        out.push(tasks::Request::Refused { reply_to, problem });
        return tag_task_outputs(core, env, out);
    }
    if let tasks::Event::Amend { task, amendment, .. } = &event
        && let Some(tasks::Tracking::Tracked { .. }) = amendment.tracked
        && let Some(row) = core.tasks.task(*task)
        && row.tracked.is_none()
        && core.projections.len() >= env.limits.tasks.tasks
    {
        let tasks::Event::Amend { task, reply_to, .. } = event else { unreachable!("checked amendment") };
        out.push(tasks::Request::Refused { reply_to, problem: tasks::Problem::new(Some(task), tasks::Refusal::Goals) });
        return tag_task_outputs(core, env, out);
    }
    if let tasks::Event::Make { batch, .. } = &event {
        let mut tracked = 0_u32;
        for row in batch {
            if row.tracked.is_some() {
                tracked = tracked.checked_add(1).expect("bounded make batch");
            }
        }
        if tracked > env.limits.tasks.tasks.saturating_sub(core.projections.len()) {
            let tasks::Event::Make { reply_to, creator, batch } = event else { unreachable!("checked creation event") };
            let capacity = tasks::Capacity::batch(
                creator,
                &batch,
                None,
                tasks::CapacityCause::Tasks { limit: env.limits.tasks.tasks },
            );
            core.tasks.remember_capacity(env.wall, capacity);
            out.push(tasks::Request::Refused {
                reply_to,
                problem: tasks::Problem::capacity(None, tasks::Refusal::Live, capacity),
            });
            return tag_task_outputs(core, env, out);
        }
    }
    let event = if let tasks::Event::Make { reply_to, creator, mut batch } = event {
        if let Err(problem) = core.resolve_llm(&mut batch) {
            out.push(tasks::Request::Refused { reply_to, problem });
            return tag_task_outputs(core, env, out);
        }
        match crate::effects::resources::defer(
            core,
            env,
            reply_to,
            crate::effects::resources::Kind::Creation { creator },
            batch,
            &mut out,
        ) {
            Some(event) => event,
            None => {
                return tag_task_outputs(core, env, out);
            }
        }
    } else if let tasks::Event::MakeResultFollowups { reply_to, proposer, proposal, mut batch } = event {
        if let Err(problem) = core.resolve_llm(&mut batch) {
            out.push(tasks::Request::Refused { reply_to, problem });
            return tag_task_outputs(core, env, out);
        }
        match crate::effects::resources::defer(
            core,
            env,
            reply_to,
            crate::effects::resources::Kind::Result { proposer, proposal },
            batch,
            &mut out,
        ) {
            Some(event) => event,
            None => {
                return tag_task_outputs(core, env, out);
            }
        }
    } else {
        event
    };
    let event = question_event(core, event);
    super::step::step_tasks(core, env, event, &mut out);
    crate::effects::resources::resume(core, work);
    tag_task_outputs(core, env, out)
}
// Direct-answer authority stays inside the core's question admission.
fn question_event(core: &Core, event: tasks::Event) -> tasks::Event {
    if let tasks::Event::Message { reply_to, project, task, word } = event {
        let person = match core.tasks.task(task) {
            Some(row) => match row.requester {
                tasks::Party::Person(person) => Some(person),
                tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
            },
            None => None,
        };
        if core.person_answers
            && word.kind == tasks::MessageKind::Question
            && let Some(person) = person
        {
            tasks::Event::PersonQuestion { reply_to, project, task, person, word }
        } else {
            tasks::Event::Message { reply_to, project, task, word }
        }
    } else {
        event
    }
}

pub(crate) fn result_words(result: tasks::TaskResult) -> Box<[u8]> {
    match result {
        tasks::TaskResult::Report { words }
        | tasks::TaskResult::Verdict { words, .. }
        | tasks::TaskResult::Change { words, .. } => words,
        tasks::TaskResult::Approved { words, .. } => match words {
            Some(words) => words,
            None => Box::new([]),
        },
        tasks::TaskResult::Failure { reason } => reason,
    }
}

pub(crate) fn ending_words(ending: tasks::Ending) -> Box<[u8]> {
    match ending {
        tasks::Ending::Done(result) => result_words(result),
        tasks::Ending::Failed { reason } => reason,
        tasks::Ending::Cancelled { reason, result, .. } => match result {
            Some(result) => result_words(result),
            None => reason,
        },
    }
}

pub(crate) fn result_notice(ending: tasks::Ending) -> (tasks::ResultKind, Box<[u8]>) {
    match ending {
        tasks::Ending::Done(result) => match result {
            tasks::TaskResult::Approved { state, words } => {
                let words_present = words.is_some();
                let words = match words {
                    Some(words) => words,
                    None => Box::new([]),
                };
                (tasks::ResultKind::Approved { state, words_present }, words)
            }
            tasks::TaskResult::Report { words } => (tasks::ResultKind::Report, words),
            tasks::TaskResult::Verdict { code, words, fields } => (tasks::ResultKind::Verdict { code, fields }, words),
            tasks::TaskResult::Change { connector, kind, resource, words } => {
                (tasks::ResultKind::Change { connector, kind, resource }, words)
            }
            tasks::TaskResult::Failure { reason } => (tasks::ResultKind::Failed, reason),
        },
        tasks::Ending::Failed { reason } => (tasks::ResultKind::Failed, reason),
        tasks::Ending::Cancelled { reason, .. } => (tasks::ResultKind::Cancelled, reason),
    }
}

fn follow(core: &mut Core, env: &Env<Limits>, event: tasks::Event, pending: &mut Queue<tasks::Request>) {
    let (event, procedure) = if let tasks::Event::Procedure { reply_to, task, step, read, decision } = event {
        let token = reply_to.into_token();
        (tasks::Event::Procedure { reply_to: ReplyTo::new(token), task, step, read, decision }, Some(token))
    } else {
        (event, None)
    };
    let mut child = Queue::uncapped();
    super::step::step_tasks(core, env, event, &mut child);
    for _ in 0..child.len() {
        let request = child.pop().expect("continuation output count");
        if let tasks::Request::ProcedureRefused { .. } = &request
            && let Some(token) = procedure
        {
            drop(core.land_task(token));
        }
        pending.push(request);
    }
}

fn tag_tasks(
    core: &mut Core,
    env: &Env<Limits>,
    next: tasks::Request,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    match next {
        tasks::Request::ProcedureRefused { task, step, problem } => {
            refused::procedure_refused(core, task, step, problem, out);
        }
        tasks::Request::Lifted { reply_to, released } => terminals::lifted(core, env, reply_to, released, out),
        tasks::Request::MovedToNewest { .. } => unreachable!("settings move consumes its counts"),
        tasks::Request::Released { revision } => terminals::released(core, env, revision, out),
        tasks::Request::Erase { key } => terminals::erase(key, out),
        tasks::Request::Relay { task, .. } => terminals::relay(core, env, task, out),
        tasks::Request::PersonProposed { reply_to, proposal } => {
            proposals::person_proposed(core, env, reply_to, proposal, pending, out);
        }
        tasks::Request::PersonProposalDecided { reply_to, proposer, number, outcome } => {
            proposals::person_proposal_decided(core, env, reply_to, proposer, number, outcome, out);
        }
        tasks::Request::EscalationDecided { reply_to, task, revision, outcome } => {
            escalations::escalation_decided(core, env, reply_to, task, revision, outcome, out);
        }
        tasks::Request::ResolveUncertain { task: _, entry, again } => {
            terminals::resolve_uncertain(core, entry, again, out);
        }
        tasks::Request::Sent { reply_to, task, word, .. } => terminals::sent(core, env, reply_to, task, word, out),
        tasks::Request::RecurringDue { task, period, members } => {
            lifecycle::recurring_due(core, env, task, period, members, pending);
        }
        tasks::Request::Adopt { task, attempt, kept } => lifecycle::adopt(core, env, task, attempt, kept),
        tasks::Request::Stop { task, attempt } => terminals::stop(task, attempt, out),
        tasks::Request::RestoreRefused { .. } => terminals::restore_refused(core),
        tasks::Request::Waiting { .. }
        | tasks::Request::WriterWaiting { task: None, .. }
        | tasks::Request::EscalationsInspected { .. }
        | tasks::Request::EscalationsRechecked { .. } => {}
        tasks::Request::Taken { task, holdings } => terminals::taken(task, holdings, out),
        tasks::Request::EndTopic { task, subscription, connector } => {
            terminals::end_topic(task, subscription, connector, out);
        }
        tasks::Request::WriterWaiting { task: Some(task), .. } => {
            terminals::writer_waiting(core, env, task, pending, out);
        }
        tasks::Request::EscalationNeeded { context } => {
            escalations::escalation_needed(core, env, context, pending, out);
        }
        tasks::Request::EscalationStalled { task, revision, holder } => {
            escalations::escalation_stalled(core, env, task, revision, holder, pending, out);
        }
        tasks::Request::ProposalStalled { proposer, proposal, holder } => {
            proposals::proposal_stalled(core, env, proposer, proposal, holder, pending);
        }
        tasks::Request::ProposalRerouteNeeded { proposer, proposal } => {
            proposals::proposal_reroute_needed(core, env, proposer, proposal, pending);
        }
        tasks::Request::Made { reply_to, tasks } => made::made(core, env, reply_to, tasks, pending, out),
        tasks::Request::ProposalDecided { reply_to, proposer, number, outcome } => {
            proposals::proposal_decided(core, env, reply_to, proposer, number, outcome, out);
        }
        tasks::Request::CompoundRefused { reply_to, at, problem } => {
            refused::compound_refused(core, env, reply_to, at, problem, out);
        }
        tasks::Request::Done { reply_to } => done::done(core, env, reply_to, pending, out),
        tasks::Request::Save { record } => save::save(core, env, record, pending, out),
        tasks::Request::Acknowledged { reply_to, task, attempt, .. } => {
            terminals::acknowledged(core, reply_to, task, attempt, out);
        }
        tasks::Request::TurnAcknowledged { reply_to, task, attempt, turn, accepted } => {
            terminals::turn_acknowledged(reply_to, task, attempt, turn, accepted, out);
        }
        tasks::Request::Ended { task, requester, ending } => {
            lifecycle::ended(core, env, task, requester, ending, pending, out);
        }
        tasks::Request::Refused { reply_to, problem } => refused::refused(core, env, reply_to, problem, pending, out),
        tasks::Request::PersonStallDue { task } => notices::person_stall_due(core, env, task, pending),
        tasks::Request::QuestionOwedDue { task } => notices::question_owed_due(core, env, task, pending),
        tasks::Request::QuestionEndedDue { task, questions } => {
            notices::question_ended_due(core, env, task, questions, pending);
        }
        tasks::Request::MetDue { task, dependency, removed, by } => {
            notices::met_due(core, env, task, dependency, removed, by, pending);
        }
        tasks::Request::Notify { task, subscription, target, state, words } => {
            let child = notices::notify(core, env, task, subscription, target, state, words);
            append_generated(child, pending);
        }
        tasks::Request::Timer { task, subscription } => notices::timer(core, env, task, subscription, pending),
        tasks::Request::Close { task, ending } => lifecycle::close(core, task, ending, out),
        tasks::Request::Release { task, ending } => lifecycle::release(core, env, task, ending, pending, out),
        tasks::Request::Activate { context } => lifecycle::activate(core, context, out),
    }
}

fn append_generated(mut child: Queue<tasks::Request>, pending: &mut Queue<tasks::Request>) {
    for _ in 0..child.len() {
        let request = child.pop().expect("generated task output count");
        pending.push(request);
    }
}
