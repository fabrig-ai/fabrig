//! Project deletion's first decision (temper/docs/design/draft/playground-reset.md, 5).
use crate::{Core, CoreRecord, Limits, ProjectRemoval, Record, RemovalStage, Request, Requests, Write, reads};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

mod erase;
mod rows;

/// Internal compound replies never enter a party's request table.
pub(super) const CANCEL: Token = Token::new(u64::MAX - 4);

/// What a removal is waiting on; absent means idle. Rebuilt from its stored stage.
#[derive(Debug)]
#[expect(dead_code, reason = "the erasing and finishing stages populate these continuations")]
pub(crate) enum Work {
    EndedPage { after: Option<u64> },
    TaskRows { record: Box<tasks::TaskRecord> },
    Proposals { after: Option<u64> },
    Notes,
}

/// The deletion's first decision: row, fence, cancels; answer held.
pub(crate) fn close(
    core: &mut Core,
    env: &Env<Limits>,
    project: u32,
    out: &mut Queue<Request>,
) -> Result<(), people::Refusal> {
    if !core.projects.as_slice().contains(&project) || core.removing(project) {
        return Err(people::Refusal::Unknown);
    }
    if core.tasks.live_in(project) {
        if env.limits.people.operations == 0 {
            return Err(people::Refusal::Limit { which: people::LimitKind::Operations });
        }
        for task in core.tasks.view_tasks() {
            if task.project == project && core.tasks.task(task.number).expect("live task view").revision == u64::MAX {
                return Err(people::Refusal::Limit { which: people::LimitKind::Arithmetic });
            }
        }
        let messages = u64::from(env.limits.tasks.tasks)
            .checked_mul(2)
            .expect("bounded end notices")
            .checked_add(u64::from(env.limits.settings.proposals))
            .expect("bounded proposal notices");
        if core.counters.deployment().messages.checked_add(messages).is_none() {
            return Err(people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        }
    }
    enter(core, env, project, RemovalStage::Ending, out);
    core.touch_project_parties(project);
    core.touch_frames();
    core.touch_deployment();
    Ok(())
}

/// Move a removal to `stage`, saving its row, and start the stage.
pub(crate) fn enter(core: &mut Core, env: &Env<Limits>, project: u32, stage: RemovalStage, out: &mut Queue<Request>) {
    let row = ProjectRemoval { project, stage };
    core.removals.insert(project, row).expect("one removal per kept project");
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Removal(row)))));
    advance(core, env, project, out);
}

/// Continue a removal at its stored stage.
pub(crate) fn advance(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    if core.removal_work.contains_key(&project) || core.removal_cancelling.contains(&project) {
        return;
    }
    let Some(row) = core.removal(project) else { return };
    match row.stage {
        RemovalStage::Ending => {
            cancel_roots(core, env, project, out);
            if !core.tasks.live_in(project) {
                enter(core, env, project, RemovalStage::Erasing { after: None }, out);
            }
        }
        RemovalStage::Erasing { after } => erase::start(core, env, project, after, out),
        RemovalStage::Notes => rows::start_notes(core, env, project, out),
        RemovalStage::Rows => rows::finish(core, env, project, out),
    }
}

/// One of a removing project's tasks ended; catch any root made in this decision.
pub(crate) fn task_ended(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    if let Some(row) = core.removal(project)
        && row.stage == RemovalStage::Ending
    {
        advance(core, env, project, out);
    }
}

fn cancel_roots(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    let mut roots = List::with_capacity(env.limits.tasks.tasks);
    core.tasks.live_roots(project, &mut roots).expect("roots fit the live task bound");
    let _inserted = core.removal_cancelling.insert(project).expect("one cancellation pass per project");
    let mut operations = List::with_capacity(env.limits.people.operations.min(env.limits.tasks.tasks));
    let mut child = Queue::uncapped();
    for root in &roots {
        let Some(row) = core.tasks.task(*root) else { continue };
        match &row.phase {
            tasks::Phase::Closing(_)
            | tasks::Phase::Ended(_)
            | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => continue,
            tasks::Phase::Waiting
            | tasks::Phase::Active(_)
            | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => {}
        }
        let mut numbers = List::with_capacity(env.limits.tasks.tasks);
        for delegate in &row.delegates {
            numbers.push(*delegate).expect("delegates fit live tasks");
        }
        numbers.push(*root).expect("root and delegates fit live tasks");
        for task in &numbers {
            if operations.room() == 0 {
                compound(core, env, project, operations, &mut child);
                operations = List::with_capacity(env.limits.people.operations.min(env.limits.tasks.tasks));
            }
            operations
                .push(tasks::Operation::Cancel { task: *task, grace: None, message: None, reason: Box::new([]) })
                .expect("bounded cancel compound");
        }
    }
    if !operations.is_empty() {
        compound(core, env, project, operations, &mut child);
    }
    super::start_task_outputs(core, env, child, out);
    let _removed = core.removal_cancelling.remove(&project);
}

fn compound(
    core: &mut Core,
    env: &Env<Limits>,
    project: u32,
    operations: List<tasks::Operation>,
    out: &mut Queue<tasks::Request>,
) {
    let event = tasks::Event::Compound {
        reply_to: ReplyTo::new(CANCEL),
        by: tasks::Party::Deployment { project },
        project,
        operations,
        period: core.period(project),
        pool_budget: 0,
        period_budget: core.period_budget(project),
    };
    tasks::step(&mut core.tasks, &Env { now: env.now, wall: env.wall, limits: env.limits.tasks }, event, out);
}

/// A removal's load answered.
pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: reads::Loaded) -> Requests {
    let mut out = Queue::uncapped();
    let project = removal_of(load).expect("removal load token");
    if let Some(row) = core.removal(project) {
        match row.stage {
            RemovalStage::Erasing { .. } => erase::loaded(core, env, project, result, &mut out),
            RemovalStage::Notes => rows::loaded(core, env, project, result, &mut out),
            RemovalStage::Ending | RemovalStage::Rows => {}
        }
        advance(core, env, project, &mut out);
    }
    Requests::Out(out)
}

/// Reserved slab slot `u32::MAX`, with the project in the generation half.
#[expect(dead_code, reason = "the erasing and finishing stages issue loads in this reserved space")]
pub(crate) fn load_token(project: u32) -> Token {
    Token::new((u64::from(u32::MAX) << 32) | u64::from(project))
}
pub(crate) fn removal_of(load: Token) -> Option<u32> {
    if load.raw() >> 32 == u64::from(u32::MAX) {
        Some(u32::try_from(load.raw() & u64::from(u32::MAX)).expect("low token half"))
    } else {
        None
    }
}
