//! Fenced procedure decisions at the tasks hub (domain/tasks.md, sections 5.3, 5.7 and 8.2).
//! The owner keeps procedure state and chooses a step; tasks keeps only the
//! executor identity, live topology, result contract and the step's offered inbox fence. It never
//! knows connector facts or effects. A due step may make one whole batch,
//! finish after its delegates, hold, or wait for changed facts.
#![cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::result_large_err, reason = "bounded owner refusal carries exact capacity diagnostics inline")
)]
use crate::domain::{Domain, make_admitted, publish, record, task_mut};
use crate::{
    Active, Closing, Ending, Executor, Limits, Party, Phase, ProcedureDecision, ProcedureEnd, Refusal, Request, Stage,
    TaskResult,
};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, ReplyTo};

#[expect(clippy::too_many_arguments, reason = "one fenced procedure decision")]
pub(crate) fn stepped(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    step: u64,
    read: Option<u64>,
    decision: ProcedureDecision,
    out: &mut Queue<Request>,
) {
    if let Err(problem) = fence(domain, number, step, read) {
        return out.push(Request::ProcedureRefused { task: number, step, problem });
    }
    task_mut(domain, number).expect("fenced procedure").record.room_wait = None;
    let staged = match check(domain, env, number, step, read, &decision) {
        Ok(staged) => staged,
        Err(problem) => {
            if let Some(room) = problem.freeing_count() {
                crate::room::wait(domain, env, number, room, out);
            }
            return out.push(Request::ProcedureRefused { task: number, step, problem });
        }
    };
    crate::inbox::keep_refusals(domain, number, &[]);
    let started_from = staged.record(domain, number).expect("staged procedure task").started_from.clone();
    let live = task_mut(domain, number).expect("procedure step preflighted");
    live.record.started_from = started_from;
    if let Some(words) = decision.words {
        live.record.spec.words = words;
    }
    live.record.attempt = step;
    crate::inbox::take(&mut live.record, read);
    live.procedure_offered = None;
    task_mut(domain, number).expect("procedure task live").record.phase = Phase::Active(Active::Idle);
    let numbers = if decision.batch.is_empty() {
        None
    } else {
        Some(make_admitted(domain, env, Party::Task(number), decision.batch, out))
    };
    for plan in staged.plans.into_boxed() {
        crate::compound::apply(domain, env, Party::Task(number), plan, out);
    }
    end_step(domain, env, number, decision.end, out);
    crate::wake::after_step(domain, env, number, out);
    match numbers {
        Some(tasks) => out.push(Request::Made { reply_to: to, tasks }),
        None => out.push(Request::Done { reply_to: to }),
    }
}

fn fence(domain: &Domain, number: u64, step: u64, read: Option<u64>) -> Result<(), crate::Problem> {
    if !domain.ready() {
        return Err(problem(number, Refusal::NotReady));
    }
    let task = record(domain, number).ok_or(problem(number, Refusal::Unknown))?;
    match task.executor {
        Executor::Procedure { .. } => {}
        Executor::Agent { .. } | Executor::Person(_) | Executor::Job { .. } => {
            return Err(problem(number, Refusal::Executor));
        }
    }
    if task.phase != Phase::Active(Active::Due) || task.attempt.checked_add(1) != Some(step) {
        return Err(problem(number, Refusal::Attempt));
    }
    if read != domain.procedure_offered(number) {
        return Err(problem(number, Refusal::Read));
    }
    Ok(())
}

fn problem(task: u64, why: Refusal) -> crate::Problem {
    crate::Problem { task: Some(task), why, blocked_by: None, capacity: None }
}

fn check(
    domain: &Domain,
    env: &Env<Limits>,
    number: u64,
    step: u64,
    read: Option<u64>,
    decision: &ProcedureDecision,
) -> Result<crate::compound::Staged, crate::Problem> {
    let task = record(domain, number).expect("step task checked");
    check_words(task, &env.limits, decision.words.as_deref())?;
    check_shape(&env.limits, number, decision)?;
    if let ProcedureEnd::Result(result) = &decision.end {
        let end = crate::End::Finished { result: result.clone(), cancel_delegates: false };
        if crate::run::ends::kept(domain, number).parks(&end) {
            return Err(domain.waiting_effect_problem(number).expect("kept task names its reason"));
        }
        if !task.delegates.is_empty() || !decision.batch.is_empty() {
            return Err(crate::Problem::live_delegates(number, &task.delegates));
        }
        if !crate::run::valid_result(&task.contract, result, &env.limits) {
            return Err(problem(number, Refusal::Contract));
        }
    }
    if !decision.batch.is_empty() {
        crate::batch::check(domain, &env.limits, Party::Task(number), &decision.batch)?;
    }
    let bound = env.limits.tasks.max(env.limits.batch.saturating_mul(3).saturating_add(1));
    let mut staged = crate::compound::Staged::new(bound);
    let mut parent = task.clone();
    let mut started_from = skein_lib::List::with_capacity(env.limits.dependencies);
    for dependency in &task.started_from {
        started_from.push(*dependency).expect("marks bounded by dependencies");
    }
    for dependency in &decision.started_from {
        if !task.dependencies.contains(dependency) {
            return Err(crate::Problem::naming(Some(number), Refusal::Dependencies, &[*dependency]));
        }
        if !started_from.as_slice().contains(dependency) {
            started_from.push(*dependency).expect("marks are a subset of dependencies");
        }
    }
    parent.started_from = started_from.into_boxed();
    parent.attempt = step;
    parent.phase = Phase::Active(Active::Idle);
    crate::inbox::take(&mut parent, read);
    let mut delegates = skein_lib::List::with_capacity(env.limits.delegates);
    for child in &parent.delegates {
        delegates.push(*child).expect("bounded delegates");
    }
    for member in &decision.batch {
        delegates.push(member.number).expect("batch checked");
        staged.batch_members.push((member.number, member.executor)).expect("admitted batch bound");
        staged.batch_goals = staged.batch_goals.checked_add(u32::from(member.tracked.is_some())).expect("batch bound");
        let mut numbers = match staged.funding_numbers(member.funder) {
            Some(numbers) => numbers,
            None => staged.numbers(domain, member.funder).expect("batch funder checked"),
        };
        numbers.reserved = numbers.reserved.checked_add(member.numbers.budget).expect("batch checked");
        staged.funding.push((member.funder, numbers)).expect("batch bound");
    }
    parent.delegates = delegates.into_boxed();
    staged.rows.push(parent).expect("one procedure projection");
    check_parts(domain, &mut staged, env, number, decision)?;
    Ok(staged)
}

fn check_parts(
    domain: &Domain,
    staged: &mut crate::compound::Staged,
    env: &Env<Limits>,
    number: u64,
    decision: &ProcedureDecision,
) -> Result<(), crate::Problem> {
    let project = record(domain, number).expect("step task checked").project;
    for part in &decision.amendments {
        let plan = match crate::control::check_amend(
            domain,
            staged,
            env,
            Party::Task(number),
            crate::control::Amend {
                places: Box::new([]),
                number: part.task,
                message: part.message,
                stop_run: part.stop_run,
                funder: None,
                amendment: part.amendment.clone(),
            },
        ) {
            Ok(plan) => crate::compound::Plan::Amend(Box::new(plan)),
            Err(problem) => return Err(*problem),
        };
        retain(domain, staged, env, plan)?;
    }
    for part in &decision.messages {
        if part.word.from != Party::Task(number) {
            return Err(problem(part.task, Refusal::Reference));
        }
        let plan = match crate::inbox::check_message(
            domain,
            staged,
            env,
            crate::inbox::Message { project, number: part.task, word: part.word.clone(), authorized: false },
        ) {
            Ok(plan) => crate::compound::Plan::Message(Box::new(plan)),
            Err(problem) => return Err(*problem),
        };
        retain(domain, staged, env, plan)?;
    }
    for proposal in &decision.proposals {
        if proposal.proposer != number {
            return Err(problem(number, Refusal::Reference));
        }
        if let Some(problem) = crate::proposals::admission_problem_staged(domain, staged, &env.limits, proposal) {
            return Err(problem);
        }
        retain(domain, staged, env, crate::compound::Plan::Proposal(Box::new(proposal.clone())))?;
    }
    for part in &decision.cancels {
        if record(domain, part.task).is_none() {
            return Err(problem(part.task, Refusal::Unknown));
        }
        if !crate::control::allowed(domain, part.task, Party::Task(number), env.limits.tasks) {
            return Err(problem(part.task, Refusal::Reference));
        }
        if staged.closing.as_slice().contains(&part.task) {
            return Err(problem(part.task, Refusal::State));
        }
        let plan = match crate::control::check_cancel(
            domain,
            staged,
            env,
            Party::Task(number),
            crate::control::Cancel {
                number: part.task,
                reason: part.reason.clone(),
                grace: part.grace,
                message: part.message,
            },
        ) {
            Ok(plan) => crate::compound::Plan::Cancel(Box::new(plan)),
            Err(problem) => return Err(*problem),
        };
        retain(domain, staged, env, plan)?;
    }
    Ok(())
}

fn retain(
    domain: &Domain,
    staged: &mut crate::compound::Staged,
    env: &Env<Limits>,
    plan: crate::compound::Plan,
) -> Result<(), crate::Problem> {
    if let Err(problem) = staged.check_placed_change(&plan) {
        return Err(*problem);
    }
    staged.after(domain, &env.limits, &plan);
    assert!(staged.plans.push(plan).is_ok(), "bounded procedure parts");
    Ok(())
}

fn check_words(task: &crate::TaskRecord, limits: &Limits, words: Option<&[u8]>) -> Result<(), crate::Problem> {
    let Some(words) = words else { return Ok(()) };
    let mut spec = task.spec.clone();
    spec.words = Box::from(words);
    if !crate::batch::valid_spec(limits, &spec) {
        return Err(problem(task.number, Refusal::Spec));
    }
    Ok(())
}

fn end_step(domain: &mut Domain, env: &Env<Limits>, number: u64, end: ProcedureEnd, out: &mut Queue<Request>) {
    match end {
        ProcedureEnd::Result(result) => {
            let ending = match result {
                TaskResult::Failure { reason } => Ending::Failed { reason },
                TaskResult::Approved { .. }
                | TaskResult::Report { .. }
                | TaskResult::Verdict { .. }
                | TaskResult::Job { .. }
                | TaskResult::NoChange { .. }
                | TaskResult::Change { .. } => Ending::Done(result),
            };
            task_mut(domain, number).expect("procedure task live").record.phase =
                Phase::Closing(Closing { stage: Stage::Delegates, ending });
            publish(domain, env, number, out);
        }
        ProcedureEnd::Hold(why) => crate::run::hold(domain, env, number, why, out),
        ProcedureEnd::HoldFrom { why, source } => crate::run::hold_from(domain, env, number, why, Some(source), out),
        ProcedureEnd::Wait => publish(domain, env, number, out),
    }
}

fn check_shape(limits: &Limits, number: u64, decision: &ProcedureDecision) -> Result<(), crate::Problem> {
    if decision.cancels.len() > skein_lib::length::usize_of(limits.batch)
        || decision.amendments.len() > skein_lib::length::usize_of(limits.batch)
        || decision.messages.len() > skein_lib::length::usize_of(limits.batch)
        || decision.proposals.len() > 1
    {
        return Err(problem(number, Refusal::Batch));
    }
    if let ProcedureEnd::HoldFrom { why, source } = &decision.end
        && (!why.valid() || crate::placed::is_placed(*why) || !crate::hold_origin::valid(limits, *why, Some(source)))
    {
        return Err(problem(number, Refusal::Holds));
    }
    Ok(())
}
