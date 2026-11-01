//! Project deletion at Ending (temper/docs/design/draft/playground-reset.md, 5;
//! temper/docs/plan/playground-reset-r6/notes/contracts.md, J3–J4).
use super::children::{append_people, append_tasks};
use crate::{Core, Event, Limits, PersonTaskRoute, Request};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(crate) fn remove_project(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
) -> Result<(), people::Refusal> {
    if !core.projects.as_slice().contains(&project) || core.deleting(project) {
        return Err(people::Refusal::Unknown);
    }
    let operations = cancellations(core, project)?;
    append_people(core, env, people::Event::Deletion { project, stage: Some(people::Stage::Ending) }, out);
    cancel(core, env, out, project, request, operations);
    ending_done(core, env, project, out);
    core.touch_project_parties(project);
    core.touch_frames();
    core.touch_accounts(None);
    core.touch_deployment();
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::ProjectRemoved { project } }));
    Ok(())
}

fn cancellations(core: &mut Core, project: u32) -> Result<List<tasks::Operation>, people::Refusal> {
    let rows = core.tasks.view_tasks();
    let mut operations = List::with_capacity(u32::try_from(rows.len()).expect("bounded live tasks"));
    let mut grace = None;
    for setting in &core.current.offered() {
        let crate::settings::Setting::Template(number) = setting else { continue };
        let template = core.current.template(*number).expect("offered template");
        if template.flags.conversation {
            grace = template.grace;
            break;
        }
    }
    for view in &rows {
        let row = core.tasks.task(view.number).expect("live task view");
        if row.project != project || row.root != row.number {
            continue;
        }
        match row.phase {
            tasks::Phase::Closing(_)
            | tasks::Phase::Ended(_)
            | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => continue,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {}
        }
        let grace = if row.flags.conversation { grace } else { None };
        let task = row.number;
        let message = if grace.is_some() {
            Some(
                crate::fresh(&mut core.counters, crate::Family::Message)
                    .ok_or(people::Refusal::Limit { which: people::LimitKind::Arithmetic })?,
            )
        } else {
            None
        };
        operations
            .push(tasks::Operation::Cancel { task, grace, message, reason: b"Project deleted".as_slice().into() })
            .expect("one cancellation per live root");
    }
    if !operations.is_empty() && core.person_tasks.len() == core.person_tasks.capacity() {
        return Err(people::Refusal::Limit { which: people::LimitKind::LiveTasks });
    }
    Ok(operations)
}

fn cancel(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    project: u32,
    request: Token,
    operations: List<tasks::Operation>,
) {
    if operations.is_empty() {
        return;
    }
    assert!(core.person_tasks.insert(request, PersonTaskRoute::Deletion) == Ok(None), "one deletion compound");
    append_tasks(
        core,
        env,
        tasks::Event::Compound {
            reply_to: ReplyTo::new(request),
            by: tasks::Party::Deployment { project },
            project,
            operations,
            period: core.period(project),
            pool_budget: 0,
            period_budget: core.period_budget(project),
        },
        out,
    );
}

pub(crate) fn ending_done(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    if core.people.deletion(project) != Some(people::Stage::Ending) {
        return;
    }
    for view in &core.tasks.view_tasks() {
        if view.project == project {
            return;
        }
    }
    append_people(core, env, people::Event::Deletion { project, stage: Some(people::Stage::Erasing) }, out);
}

pub(crate) fn resume_ending(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut projects = List::with_capacity(core.people.deletions().len());
    for (project, stage) in core.people.deletions() {
        if *stage == people::Stage::Ending {
            projects.push(*project).expect("one per deletion row");
        }
    }
    for project in projects.into_boxed() {
        if let Ok(operations) = cancellations(core, project) {
            // Restart compounds finish synchronously before the next project resumes.
            cancel(core, env, out, project, Token::new(u64::MAX - 4), operations);
            ending_done(core, env, project, out);
        }
    }
}

/// Each terminal checks the ending projects against the child's final live census.
pub(crate) fn ended(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut projects = List::with_capacity(core.people.deletions().len());
    for (project, stage) in core.people.deletions() {
        if *stage == people::Stage::Ending {
            projects.push(*project).expect("one per deletion row");
        }
    }
    for project in projects.into_boxed() {
        ending_done(core, env, project, out);
    }
}
