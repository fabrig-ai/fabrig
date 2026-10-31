//! Child entry points and their shared ordering (jig/docs/design/domain/engine.md, 3 and 4.4).

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

pub(crate) fn step_tasks(core: &mut Core, env: &Env<Limits>, event: tasks::Event, out: &mut Queue<tasks::Request>) {
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
    fleet::resume(&mut core.fleet, &env.with(env.limits.fleet), out);
}
