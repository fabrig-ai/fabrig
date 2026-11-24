//! Child entry points and their shared ordering (jig/docs/design/domain/engine/README.md, 3, and jig/docs/design/domain/engine/vocabulary.md, 1.4).

use crate::{Core, Limits};
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_fleet as fleet;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_domain_views as views;
use skein_lib::{Env, Queue};

impl Core {
    /// Waits by reading use the current root requester's role, never the executor's
    /// (jig/docs/design/domain/tasks/messages.md, 5).
    pub(crate) fn task_reads_project(&self, task: u64) -> bool {
        let Some(row) = self.tasks.task(task) else { return false };
        if !self.open_project(row.project) {
            return false;
        }
        let Some(root) = self.tasks.task(row.root) else { return false };
        match root.requester {
            tasks::Party::Person(person) => self.people.role(person, row.project).is_some(),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => false,
        }
    }

    pub(crate) fn creator_reads_project(&self, creator: tasks::Party) -> bool {
        match creator {
            tasks::Party::Task(task) => self.task_reads_project(task),
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => false,
        }
    }
}

pub(crate) fn step_tasks(core: &mut Core, env: &Env<Limits>, mut event: tasks::Event, out: &mut Queue<tasks::Request>) {
    // These events can arrive after loads or effects. Stamp the fact only at the child entrance.
    if let tasks::Event::Make { creator, maker, reads_project, .. } = &mut event {
        *reads_project = maker.is_none() && core.creator_reads_project(*creator);
    } else if let tasks::Event::Amend { by, reads_project, .. } = &mut event {
        *reads_project = core.creator_reads_project(*by);
    } else if let tasks::Event::Subscribe { task, reads_project, .. }
    | tasks::Event::Procedure { task, reads_project, .. }
    | tasks::Event::MakeResultFollowups { proposer: task, reads_project, .. } = &mut event
    {
        *reads_project = core.task_reads_project(*task);
    } else if let tasks::Event::Activation { task, end, reads_project, .. } = &mut event {
        *reads_project = false;
        if let tasks::End::FinishedWithFollowups { followups: tasks::ResultFollowups::Delegates(_), .. } = end {
            *reads_project = core.task_reads_project(*task);
        }
    }
    tasks::step(&mut core.tasks, &env.with(env.limits.tasks), event, out);
}

pub(crate) fn step_people(core: &mut Core, env: &Env<Limits>, event: people::Event, out: &mut Queue<people::Request>) {
    super::view::marks::touch_people_event(core, &env.limits, &event);
    people::step(&mut core.people, &env.with(env.limits.people), event, out);
}

pub(crate) fn step_fleet(core: &mut Core, env: &Env<Limits>, event: fleet::Event, out: &mut Queue<fleet::Request>) {
    fleet::step(&mut core.fleet, &env.with(env.limits.fleet), event, out);
}

pub(crate) fn step_views(core: &mut Core, env: &Env<Limits>, event: views::Event, out: &mut Queue<views::Request>) {
    views::step(&mut core.views, &env.with(env.limits.views), event, out);
}

pub(crate) fn step_accounts(
    core: &mut Core,
    env: &Env<Limits>,
    event: accounts::Event,
    out: &mut Queue<accounts::Request>,
) {
    accounts::step(&mut core.accounts, &env.with(env.limits.accounts), event, out);
}

pub(crate) fn step_notes(core: &mut Core, env: &Env<Limits>, event: notes::Event, out: &mut Queue<notes::Request>) {
    notes::step(&mut core.notes, &env.with(env.limits.notes), event, out);
}

pub(crate) fn step_brief(
    core: &mut Core,
    env: &Env<Limits>,
    event: brief::GatherEvent,
    out: &mut Queue<brief::GatherRequest>,
) {
    brief::gather_step(&mut core.brief, &env.with(env.limits.brief), event, out);
}

pub(crate) fn step_authority(core: &mut Core, event: authority::Event, out: &mut Queue<authority::PolicyFact>) {
    authority::step(&mut core.authority, event, out);
}

pub(crate) fn fire_tasks(core: &mut Core, env: &Env<Limits>, out: &mut Queue<tasks::Request>) {
    tasks::fire(&mut core.tasks, &env.with(env.limits.tasks), out);
}

pub(crate) fn fire_people(core: &mut Core, env: &Env<Limits>, out: &mut Queue<people::Request>) {
    people::fire(&mut core.people, &env.with(env.limits.people), out);
}

pub(crate) fn fire_fleet(core: &mut Core, env: &Env<Limits>, out: &mut Queue<fleet::Request>) {
    fleet::fire(&mut core.fleet, &env.with(env.limits.fleet), out);
}

pub(crate) fn fire_accounts(core: &mut Core, env: &Env<Limits>, out: &mut Queue<accounts::Request>) {
    accounts::fire(&mut core.accounts, &env.with(env.limits.accounts), out);
}

pub(crate) fn fire_brief(core: &mut Core, env: &Env<Limits>, out: &mut Queue<brief::GatherRequest>) {
    brief::gather_fire(&mut core.brief, &env.with(env.limits.brief), out);
}

pub(crate) fn resume_fleet(core: &mut Core, env: &Env<Limits>, out: &mut Queue<fleet::Request>) {
    fleet::resume(&mut core.fleet, &env.with(env.limits.fleet), !core.settling, out);
}
