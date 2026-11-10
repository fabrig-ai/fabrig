//! Whole project policies in settings decisions (jig/docs/design/domain/engine/settings.md, 2, 6).
use crate::routing::{Work, party::dispatch::Routed};
use crate::{Core, Event, Limits, Request, settings as s};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, ReplyTo};

pub(super) fn changes(core: &Core, items: &[s::Item]) -> bool {
    if s::changes_current(&core.current, items) {
        return true;
    }
    for item in items {
        if let Some(project) = project(item.setting) {
            match value(item) {
                Some(value) => {
                    if core.configured_policy(project).as_ref() != Some(value) {
                        return true;
                    }
                }
                None => return true,
            }
        }
    }
    false
}

pub(super) fn check(core: &Core, limits: &Limits, items: &[s::Item]) -> Result<(), people::Refusal> {
    let rules = match s::check::edit(items, s::Setting::Rules) {
        Some(s::Edit::Set(value)) => value.as_rules(),
        Some(s::Edit::Reset) => match core.current.default_value(s::Setting::Rules) {
            Some(value) => value.as_rules(),
            None => None,
        },
        Some(s::Edit::Removed) | None => None,
    }
    .unwrap_or(core.current.rules());
    for item in items {
        if let Some(project) = project(item.setting)
            && let Some(value) = value(item)
        {
            if !core.open_project(project) {
                return Err(people::Refusal::Unknown);
            }
            for role in &value.policy.roles {
                if !role.authority.models.at_most(value.policy.ceiling.models)
                    || !role.authority.models.at_most(rules.ceiling.models)
                {
                    return Err(super::malformed_refusal(
                        items,
                        s::Failure { setting: item.setting, fault: s::Fault::Reference, detail: None },
                    ));
                }
            }
            if core.check_policy_replacement(limits, project, value, rules).is_err() {
                // Whole-set validation already checked shape, references and ceilings. The
                // remaining owner-edit check is room in the open period and its pools.
                return Err(super::malformed_refusal(
                    items,
                    s::Failure { setting: item.setting, fault: s::Fault::Budget, detail: None },
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn owns(core: &Core, person: u64, items: &[s::Item]) -> Result<(), people::Refusal> {
    if core.people.role(person, people::DEPLOYMENT) != Some(people::Role::Owner) {
        return Err(people::Refusal::Role { held: core.people.role(person, people::DEPLOYMENT) });
    }
    for item in items {
        if let Some(project) = project(item.setting) {
            let held = core.people.role(person, project);
            if held != Some(people::Role::Owner) {
                return Err(people::Refusal::Role { held });
            }
        }
    }
    Ok(())
}

pub(super) fn has_approver(core: &Core, items: &[s::Item]) -> bool {
    for holding in core.people.project_members(people::DEPLOYMENT).unwrap_or(&[]) {
        if owns(core, holding.person, items).is_ok() {
            return true;
        }
    }
    false
}

pub(super) fn commit(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, items: &[s::Item]) {
    for item in items {
        let Some(project) = project(item.setting) else { continue };
        let Some(value) = value(item) else { continue };
        let (snapshot, resize) = core
            .check_policy_replacement(&env.limits, project, value, core.authority.rules())
            .expect("policy replacement preflighted before settings mutate");
        core.commit_policy(project, value.policy.clone(), value.permissions.clone(), value.templates.clone());
        core.touch_project_parties(project);
        let name = core.names.get(&project).expect("live project name").clone();
        routed.out.push(Request::Write(crate::Write::Save(crate::Record::People(people::Stored::Policy {
            project,
            name,
            value: snapshot,
        }))));
        let period = core.period(project);
        let period_before = core.period_budget(project);
        if let Some(budget) = resize.period
            && budget >= period_before
        {
            resize_period(core, routed, project, period, budget);
        }
        for cuts in [true, false] {
            for pool in &resize.pools {
                if (pool.to < pool.from) == cuts {
                    routed.work.push(Work::Event(Event::Tasks(tasks::Event::ResizePool {
                        reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Admission)),
                        project,
                        period,
                        person: pool.person,
                        budget: pool.to,
                    })));
                }
            }
        }
        if let Some(budget) = resize.period
            && budget < period_before
        {
            resize_period(core, routed, project, period, budget);
        }
        let mut reroutes = core.proposal_recheck_project(&env.limits, project, env.wall);
        for _ in 0..reroutes.len() {
            routed.work.push(Work::Event(Event::Tasks(reroutes.pop().expect("policy holder recheck"))));
        }
    }
}

fn resize_period(core: &mut Core, routed: &mut Routed, project: u32, period: u64, budget: u64) {
    routed.work.push(Work::Event(Event::Tasks(tasks::Event::ResizePeriod {
        reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Admission)),
        project,
        period,
        budget,
    })));
}

/// The policy row, one period row, and two funding rows per party pool.
pub(super) fn writes_bound(limits: &Limits) -> Option<u32> {
    limits.people.holdings.checked_mul(2)?.checked_add(2)
}

pub(super) fn project(setting: s::Setting) -> Option<u32> {
    match setting {
        s::Setting::ProjectPolicy { project } => Some(project),
        s::Setting::Charter(_)
        | s::Setting::Template(_)
        | s::Setting::Policy(_)
        | s::Setting::Model(_)
        | s::Setting::Rules
        | s::Setting::Application { .. } => None,
    }
}
fn value(item: &s::Item) -> Option<&crate::ConfiguredPolicy> {
    match &item.edit {
        s::Edit::Set(value) => value.as_policy(),
        s::Edit::Removed | s::Edit::Reset => None,
    }
}
