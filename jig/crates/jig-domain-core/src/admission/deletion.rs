//! Kept project deletion (jig/docs/design/domain/people.md, 5.2).
use crate::{Ask, Core, Event, Key, Limits, Request, Write};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, List, Map, Queue, ReplyTo, Token};

/// Per kept deletion: its step, compound, connectors still owed and notes walk.
#[derive(Debug)]
pub(crate) struct Deletions {
    live: Map<u32, Deleting>,
    resume: bool,
    walk: Option<u32>,
    routing: bool,
}
#[derive(Debug)]
struct Deleting {
    step: people::DeletionStep,
    grace: Option<Duration>,
    compound: Option<Token>,
    owed: List<u16>,
    notes: Option<Token>,
}
impl Deletions {
    pub(crate) fn new(projects: u32) -> Self {
        Self { live: Map::with_capacity(projects), resume: false, walk: None, routing: false }
    }

    pub(crate) fn contains(&self, project: u32) -> bool {
        self.live.contains_key(&project)
    }

    pub(crate) fn route(&mut self, active: bool) -> bool {
        core::mem::replace(&mut self.routing, active)
    }
}
impl Deleting {
    fn new(step: people::DeletionStep, grace: Option<Duration>) -> Self {
        Self { step, grace, compound: None, owed: List::with_capacity(0), notes: None }
    }
}
impl Core {
    /// Whether a project takes requests (jig/docs/design/domain/people.md, 5.2, "Closed at once").
    #[must_use]
    pub fn open_project(&self, project: u32) -> bool {
        self.projects.as_slice().contains(&project) && !self.deletions.contains(project)
    }
}

pub(crate) fn begin(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<crate::routing::Work>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
    grace: Option<Duration>,
) -> Result<(), people::Refusal> {
    if !core.open_project(project) {
        return Err(people::Refusal::Unknown);
    }
    if let Some(grace) = grace
        && grace > env.limits.tasks.grace
    {
        return Err(people::Refusal::Limit { which: people::LimitKind::Grace });
    }
    core.deletions.live.insert(project, Deleting::new(people::DeletionStep::Tasks, grace)).expect("bounded projects");
    core.public_index.withdraw(project);
    core.touch_project_parties(project);
    keep(core, env, out, project, Some(people::DeletionStep::Tasks));
    crate::routing::deletion_event(core, env, Event::People(people::Event::CloseProject { project }), out);
    hide(core, project);
    cancel(core, env, out, project);
    work.push(crate::routing::Work::Event(Event::People(people::Event::Decided {
        request,
        outcome: people::Outcome::ProjectRemoved { project },
    })));
    Ok(())
}

fn hide(core: &mut Core, project: u32) {
    let mut projects = List::with_capacity(core.projects.capacity());
    for &other in &core.projects {
        if other != project {
            projects.push(other).expect("existing project count");
        }
    }
    core.projects = projects;
    drop(core.names.remove(&project));
}

fn keep(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    project: u32,
    step: Option<people::DeletionStep>,
) {
    let grace = core.deletions.live.get(&project).expect("kept deletion").grace;
    crate::routing::deletion_people(core, env, people::Event::Deleting { project, step, grace }, out);
}

/// Loading permits a freed name beside its new holder; only open names must be unique.
pub(crate) fn restored(core: &mut Core) -> bool {
    let mut kept = List::with_capacity(core.people.deletions().len());
    for (&project, deletion) in core.people.deletions() {
        kept.push((project, deletion.step, deletion.grace)).expect("kept deletion count");
    }
    for &(project, step, grace) in &kept {
        restore_project(core, project, step, grace);
    }
    core.deletions.resume = !core.deletions.live.is_empty();
    for (&project, name) in &core.names {
        for (&other, previous) in &core.names {
            if project != other && people::same_name(name, previous) {
                core.restore_failure = Some(crate::start::Failure::ProjectPolicy { project });
                return false;
            }
        }
    }
    true
}

/// Restore one kept step projected by people, without deciding before Open.
pub(crate) fn restore_project(core: &mut Core, project: u32, step: people::DeletionStep, grace: Option<Duration>) {
    core.public_index.withdraw(project);
    hide(core, project);
    core.deletions.live.insert(project, Deleting::new(step, grace)).expect("bounded restored projects");
    core.deletions.resume = true;
}

/// One bounded cancellation per deletion, or one whole-task walk at a time.
pub(crate) fn advance(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>) {
    if core.deletions.routing || core.start_phase().is_some() || core.start_failure().is_some() {
        return;
    }
    let resume = core.deletions.resume;
    core.deletions.resume = false;
    let mut projects = List::with_capacity(core.deletions.live.len());
    for (&project, _) in &core.deletions.live {
        projects.push(project).expect("kept deletion count");
    }
    for &project in &projects {
        let state = core.deletions.live.get(&project).expect("kept deletion");
        match state.step {
            people::DeletionStep::Tasks => {
                if state.compound.is_some() || core.deletions.walk == Some(project) {
                    continue;
                }
                if live_tasks(core, project) {
                    cancel(core, env, out, project);
                } else if core.deletions.walk.is_none() {
                    core.deletions.walk = Some(project);
                    crate::routing::task::erase::scan(
                        core,
                        env,
                        out,
                        crate::routing::task::erase::Select::Project(project),
                    );
                }
            }
            people::DeletionStep::Rows => {
                if state.notes.is_none() {
                    rows(core, env, out, project);
                }
            }
            people::DeletionStep::Release => {
                if resume {
                    release(core, env, out, project);
                }
            }
        }
    }
}

fn live_tasks(core: &Core, project: u32) -> bool {
    for task in core.tasks.view_tasks() {
        if task.project == project {
            let row = core.tasks.task(task.number).expect("live task projection");
            match row.phase {
                tasks::Phase::Ended(_) => {}
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Held { .. }
                | tasks::Phase::Closing(_) => return true,
            }
        }
    }
    false
}

fn cancel(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, project: u32) {
    let grace = match core.deletions.live.get(&project).expect("kept deletion").grace {
        Some(grace) => Some(core::cmp::min(grace, env.limits.tasks.grace)),
        None => None,
    };
    let mut operations = List::with_capacity(env.limits.people.operations);
    for task in core.tasks.view_tasks() {
        if task.project != project || operations.len() == operations.capacity() {
            continue;
        }
        let row = core.tasks.task(task.number).expect("live task projection");
        if core.tasks.root(task.number) != Some(task.number) || row.grace.is_some() {
            continue;
        }
        match row.phase {
            tasks::Phase::Ended(_) | tasks::Phase::Closing(_) => continue,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {}
        }
        let conversation = row.flags.conversation && grace.is_some();
        let group = if conversation {
            u32::try_from(row.delegates.len()).expect("bounded delegates").checked_add(1).expect("bounded delegates")
        } else {
            1
        };
        if group <= operations.capacity() {
            if group > operations.capacity().saturating_sub(operations.len()) {
                break;
            }
            if conversation {
                for &delegate in &row.delegates {
                    operations
                        .push(tasks::Operation::Cancel {
                            task: delegate,
                            grace: None,
                            message: None,
                            reason: Box::new([]),
                        })
                        .expect("conversation group fits");
                }
            }
        } else if !operations.is_empty() {
            break;
        }
        let message = if conversation { crate::fresh(&mut core.counters, crate::Family::Message) } else { None };
        let grace = if message.is_some() { grace } else { None };
        operations
            .push(tasks::Operation::Cancel { task: task.number, grace, message, reason: Box::new([]) })
            .expect("operation bound");
        if group > operations.capacity() {
            break;
        }
    }
    if operations.is_empty() {
        return;
    }
    // This synchronous child decision owns its terminal; no people flight is retained.
    let token = core.internal(crate::routing::flight::Internal::Deletion { project });
    core.deletions.live.get_mut(&project).expect("kept deletion").compound = Some(token);
    crate::routing::deletion_event(
        core,
        env,
        Event::Tasks(tasks::Event::Compound {
            reply_to: ReplyTo::new(token),
            by: tasks::Party::Deployment { project },
            project,
            operations,
            period: core.period(project),
            pool_budget: 0,
            period_budget: core.period_budget(project),
        }),
        out,
    );
    core.deletions.live.get_mut(&project).expect("kept deletion").compound = None;
}

/// Continue a project's deletion once its tasks are erased whole.
pub(crate) fn tasks_erased(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, project: u32) {
    let Some(state) = core.deletions.live.get_mut(&project) else { return };
    assert!(state.step == people::DeletionStep::Tasks, "tasks erased before project rows");
    state.step = people::DeletionStep::Rows;
    core.deletions.walk = None;
    keep(core, env, out, project, Some(people::DeletionStep::Rows));
    rows(core, env, out, project);
}

fn rows(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, project: u32) {
    if core.notes.busy() {
        return;
    }
    out.push(Request::Write(Write::Erase(Key::People(people::Key::Policy(project)))));
    out.push(Request::Write(Write::Erase(Key::People(people::Key::Public(project)))));
    let _previous = core.visibility.public.remove(&project);
    let mut facts = Queue::uncapped();
    crate::routing::step::step_authority(core, authority::Event::Dropped { project }, &mut facts);
    drop(core.permission_roles.remove(&project));
    drop(core.template_overrides.remove(&project));
    crate::routing::deletion_event(core, env, Event::Tasks(tasks::Event::CloseProject { project }), out);
    crate::routing::deletion_event(core, env, Event::Account(accounts::Event::ForgetProject { project }), out);
    crate::routing::deletion_event(core, env, Event::People(people::Event::ForgetProject { project }), out);
    let owner = core.begin_note_route(crate::routing::NoteRoute::Deletion { project });
    core.deletions.live.get_mut(&project).expect("kept deletion").notes = Some(owner);
    crate::routing::deletion_event(core, env, Event::Notes(notes::Event::DropProject { owner, project }), out);
}

/// Continue a project's deletion once its notes are dropped.
pub(crate) fn notes_dropped(core: &mut Core, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    if let Some(crate::routing::NoteRoute::Deletion { project }) = core.take_note(owner) {
        let state = core.deletions.live.get_mut(&project).expect("kept deletion");
        state.notes = None;
        state.step = people::DeletionStep::Release;
        keep(core, env, out, project, Some(people::DeletionStep::Release));
        crate::settings::application::drop_project(core, project, out);
        release(core, env, out, project);
    }
}

fn release(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, project: u32) {
    let mut owed = List::with_capacity(u32::try_from(core.connectors.len()).expect("configured connector count"));
    for &connector in &core.connectors {
        owed.push(connector).expect("configured connectors");
        out.push(Request::Ask { connector, ask: Ask::ReleaseProject { project } });
    }
    let empty = owed.is_empty();
    core.deletions.live.get_mut(&project).expect("kept deletion").owed = owed;
    if empty {
        finish(core, env, out, project);
    }
}

pub(crate) fn released(core: &mut Core, env: &Env<Limits>, connector: u16, project: u32) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let Some(state) = core.deletions.live.get_mut(&project)
        && state.step == people::DeletionStep::Release
    {
        let mut owed = List::with_capacity(state.owed.capacity());
        let mut answered = false;
        for &other in &state.owed {
            if other == connector {
                answered = true;
            } else {
                owed.push(other).expect("remaining connectors");
            }
        }
        state.owed = owed;
        if answered && state.owed.is_empty() {
            finish(core, env, &mut out, project);
        }
    }
    out
}

fn finish(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, project: u32) {
    keep(core, env, out, project, None);
    drop(core.deletions.live.remove(&project));
}

/// A failed notes load leaves Rows kept and retries on the next iteration.
pub(crate) fn notes_refused(core: &mut Core, project: u32) {
    core.deletions.live.get_mut(&project).expect("kept deletion").notes = None;
}

/// The child completed the project's bounded cancellation decision.
pub(crate) fn compound_done(core: &mut Core, project: u32) {
    core.deletions.live.get_mut(&project).expect("kept deletion").compound = None;
}
