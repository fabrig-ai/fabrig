//! A project closes in its removal decision (jig/docs/design/domain/core.md, 11;
//! jig/docs/design/domain/people.md, 5.2).
use crate::{
    Ask, Core, CoreKey, CoreRecord, Event, Held, Key, Limits, Record, Removal, RemovalStage, Request, Requests, Write,
    connector, reads::Sweep,
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};

// Outside the people's slab tokens, beside the core's other internal reply tokens.
pub(super) const CANCEL_REQUEST: Token = Token::new(u64::MAX - 4);

pub(super) fn remove_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    person: u64,
    project: u32,
) {
    match crate::acts::admit(
        core,
        &env.limits,
        person,
        crate::acts::Act::RemoveProject,
        &crate::documents::Subject::Project { project },
        env.wall,
    ) {
        crate::acts::Admission::Allowed => remove_act(core, env, work, out, request, project),
        crate::acts::Admission::Proposed { .. } | crate::acts::Admission::Bounded { .. } => {
            unreachable!("project removal has no proposal or draft bounds")
        }
        crate::acts::Admission::Refused(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

pub(super) fn remove_act(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
) {
    if !core.projects.as_slice().contains(&project) || core.removals.contains_key(&project) {
        work.push(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Unknown),
        }));
        return;
    }
    if core.removals.len() == core.removals.capacity() {
        work.push(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Again(people::Again::Busy),
        }));
        return;
    }
    close(core, env, work, out, request, project);
}

/// The decision both share: row, closing, cancels, answer.
fn close(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
) {
    let removal = Removal { project, stage: RemovalStage::Ending };
    core.removals.insert(project, removal.clone()).expect("removal room checked");
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Removal(removal)))));
    core.touch_project_parties(project);
    let mut projects = List::with_capacity(core.projects.capacity());
    for number in &core.projects {
        if *number != project {
            projects.push(*number).expect("subset of projects");
        }
    }
    core.projects = projects;
    drop(core.names.remove(&project));
    drop(core.permission_roles.remove(&project));
    drop(core.template_overrides.remove(&project));
    authority::step(&mut core.authority, authority::Event::Dropped { project }, &mut Queue::uncapped());
    work.push(Event::People(people::Event::ProjectClosed { project }));
    cancel_roots(core, env, work, out, project);
    task_ended(core, env, work, out, project);
    core.touch_deployment();
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::ProjectRemoved { project } }));
}

/// The ended-family cursor belongs to one removing project, including while
/// that project's selected task is being erased.
#[derive(Debug)]
pub(crate) struct EndedSweep {
    project: u32,
    after: u64,
}

fn cancel_roots(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>, out: &mut Queue<Request>, project: u32) {
    for task in core.tasks.project_roots(project) {
        let mut cancels = Queue::uncapped();
        if super::deployment_acts::cancel_live(
            core,
            &mut cancels,
            CANCEL_REQUEST,
            project,
            task,
            None,
            b"project deleted".as_slice().into(),
        )
        .is_err()
        {
            // The kept Ending row lets deletion resume any cancel that did not fit.
            break;
        }
        for _ in 0..cancels.len() {
            let Event::Tasks(event) = cancels.pop().expect("one compound cancel") else {
                unreachable!("cancel_live emits task events");
            };
            let Requests::Out(mut routed) = super::task_routes::route_tasks(core, env, work, event);
            for _ in 0..routed.len() {
                out.push(routed.pop().expect("routed cancel output"));
            }
        }
    }
}

fn save_stage(core: &mut Core, out: &mut Queue<Request>, project: u32, stage: RemovalStage) {
    let row = core.removals.get_mut(&project).expect("kept removal");
    row.stage = stage;
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Removal(row.clone())))));
}

/// Continue Ending only once every live task, delegates included, has ended.
pub(crate) fn task_ended(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    project: u32,
) {
    if stage(core, project) != Some(RemovalStage::Ending) {
        return;
    }
    for task in core.tasks.view_tasks() {
        if task.project == project {
            return;
        }
    }
    save_stage(core, out, project, RemovalStage::Erasing);
    begin_sweep(core, env, work, out, project);
}

fn token(core: &mut Core) -> Token {
    let request = Token::new(core.next_result_request);
    core.next_result_request = core.next_result_request.checked_add(1).expect("transient removal token space");
    request
}

fn begin_sweep(core: &mut Core, _env: &Env<Limits>, _work: &mut Queue<Event>, out: &mut Queue<Request>, project: u32) {
    let owner = token(core);
    core.removal_sweeps.insert(owner, EndedSweep { project, after: 0 }).expect("one sweep per removal");
    sweep(out, owner, 0);
}

fn sweep(out: &mut Queue<Request>, owner: Token, after: u64) {
    out.push(Request::Held(Box::new(Held::Sweep { owner, sweep: Sweep::Ended { after } })));
}

/// Resume every kept stage when the core opens; no volatile cursor or answer survives a start.
pub(crate) fn resume(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>, out: &mut Queue<Request>) {
    let mut rows = List::with_capacity(core.removals.len());
    for (_, row) in &core.removals {
        rows.push(row.clone()).expect("bounded removal snapshot");
    }
    for row in rows.into_boxed() {
        match row.stage {
            RemovalStage::Ending => {
                cancel_roots(core, env, work, out, row.project);
                task_ended(core, env, work, out, row.project);
            }
            RemovalStage::Erasing => begin_sweep(core, env, work, out, row.project),
            RemovalStage::Releasing => release(core, work, out, row.project),
        }
    }
}

/// Erase the first matching ended task in a page, or advance past the page.
pub(crate) fn swept(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    owner: Token,
    rows: Box<[Record]>,
    more: bool,
) -> Requests {
    let mut out = Queue::uncapped();
    let Some(state) = core.removal_sweeps.get_mut(&owner) else { return Requests::Out(out) };
    let project = state.project;
    for row in rows {
        assert!(Sweep::Ended { after: state.after }.admits(&row.key()), "ordered ended sweep");
        let Record::Tasks(tasks::Stored::Ended(task)) = row else { unreachable!("ended sweep row") };
        state.after = task.number;
        if task.project == project {
            crate::erase::erase_task(core, env, work, &mut out, task);
            return Requests::Out(out);
        }
    }
    if more {
        sweep(&mut out, owner, state.after);
    } else {
        let _state = core.removal_sweeps.remove(&owner);
        crate::project_rows::erase_rows(core, env, work, &mut out, project);
    }
    Requests::Out(out)
}

/// Resume the ended sweep after the task whose owner record was erased last.
pub(crate) fn erased(
    core: &mut Core,
    _env: &Env<Limits>,
    _work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    project: u32,
    task: u64,
) {
    for (&owner, state) in &core.removal_sweeps {
        if state.project == project {
            assert!(state.after == task, "one task at a time per removing project");
            sweep(out, owner, task);
            return;
        }
    }
}

/// Project-owned rows are gone before any connector is released.
pub(crate) fn rows_erased(
    core: &mut Core,
    _env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    project: u32,
) {
    if stage(core, project) != Some(RemovalStage::Erasing) {
        return;
    }
    save_stage(core, out, project, RemovalStage::Releasing);
    release(core, work, out, project);
}

fn release(core: &mut Core, work: &mut Queue<Event>, out: &mut Queue<Request>, project: u32) {
    for index in 0..core.connectors.len() {
        let connector = *core.connectors.get(index).expect("configured connector index");
        let request = token(core);
        core.removal_releases.insert(request, project).expect("one release per configured connector and removal");
        out.push(Request::Ask { connector, ask: Ask::Effect(connector::Ask::ReleaseProject { request, project }) });
    }
    finish_release(core, work, out, project);
}

/// A late answer to an earlier send cannot mark a current release.
pub(crate) fn released(
    core: &mut Core,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
) {
    if core.removal_releases.get(&request) != Some(&project) {
        return;
    }
    let _project = core.removal_releases.remove(&request);
    finish_release(core, work, out, project);
}

fn finish_release(core: &mut Core, work: &mut Queue<Event>, out: &mut Queue<Request>, project: u32) {
    for (_, owner) in &core.removal_releases {
        if *owner == project {
            return;
        }
    }
    work.push(Event::People(people::Event::ProjectGone { project }));
    let _row = core.removals.remove(&project).expect("releasing removal");
    out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Removal(project)))));
}

#[expect(clippy::manual_map, reason = "step code selects the kept stage without a closure")]
fn stage(core: &Core, project: u32) -> Option<RemovalStage> {
    match core.removals.get(&project) {
        Some(row) => Some(row.stage),
        None => None,
    }
}
