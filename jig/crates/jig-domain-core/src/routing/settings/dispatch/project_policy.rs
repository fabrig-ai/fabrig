//! Whole project policies in settings decisions (jig/docs/design/domain/engine.md, 17.2, 17.6).
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
            match &item.edit {
                s::Edit::Set(s::Value::Policy(value)) => {
                    if core.configured_policy(project).as_ref() != Some(value.as_ref()) {
                        return true;
                    }
                }
                s::Edit::Set(
                    s::Value::Charter(_) | s::Value::Template(_) | s::Value::Rules(_) | s::Value::Application(_),
                )
                | s::Edit::Removed
                | s::Edit::Reset => return true,
            }
        }
    }
    false
}

pub(super) fn check(core: &Core, limits: &Limits, items: &[s::Item]) -> Result<(), people::Refusal> {
    let mut rules = core.current.rules();
    for item in items {
        if item.setting == s::Setting::Rules {
            rules = match &item.edit {
                s::Edit::Set(s::Value::Rules(value)) => value,
                s::Edit::Reset => match core.current.default_value(s::Setting::Rules).expect("default rules") {
                    s::Value::Rules(value) => value,
                    s::Value::Charter(_) | s::Value::Template(_) | s::Value::Policy(_) | s::Value::Application(_) => {
                        unreachable!("checked rules")
                    }
                },
                s::Edit::Set(
                    s::Value::Charter(_) | s::Value::Template(_) | s::Value::Policy(_) | s::Value::Application(_),
                )
                | s::Edit::Removed => unreachable!("checked rules edit"),
            };
        }
    }
    for item in items {
        if let Some(project) = project(item.setting) {
            if !core.open_project(project) {
                return Err(people::Refusal::Unknown);
            }
            let value = value(item);
            for role in &value.policy.roles {
                if !role.authority.models.at_most(value.policy.ceiling.models)
                    || !role.authority.models.at_most(rules.ceiling.models)
                {
                    return Err(super::malformed_refusal(s::Failure {
                        setting: item.setting,
                        fault: s::Fault::Reference,
                    }));
                }
            }
            if core.check_policy_replacement(limits, project, value, rules).is_err() {
                // Whole-set validation already checked shape, references and ceilings. The
                // remaining owner-edit check is room in the open period and its pools.
                return Err(super::malformed_refusal(s::Failure { setting: item.setting, fault: s::Fault::Budget }));
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
        let value = value(item);
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
        | s::Setting::Rules
        | s::Setting::Application { .. } => None,
    }
}
fn value(item: &s::Item) -> &crate::ConfiguredPolicy {
    match &item.edit {
        s::Edit::Set(s::Value::Policy(value)) => value,
        s::Edit::Set(s::Value::Charter(_) | s::Value::Template(_) | s::Value::Rules(_) | s::Value::Application(_))
        | s::Edit::Removed
        | s::Edit::Reset => unreachable!("checked project policy edit"),
    }
}
