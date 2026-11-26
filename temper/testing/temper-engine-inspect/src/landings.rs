//! The landing liveness finding (temper/docs/design/testing/engine-checks.md, 2.2).
use crate::{View, landing_ended, task_name};
use jig_domain_tasks as tasks;
use skein_forge::api::Repository;
use skein_world::inspect::Inspection;
use temper_engine_domain_forge::LandingRow;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_landing::Phase;

pub(super) fn inspect(view: &View<'_>, into: &mut Inspection, due: impl Fn(u64) -> bool) {
    for (task, row) in view.forge.inspection_landings() {
        if landing_ended(&row.landing.phase)
            || matches!(row.landing.phase, Phase::Held { .. })
            || view.forge.inspection_landing_readback(*task)
            || due(*task)
            || view.forge.inspection_landing_stepping(*task)
        {
            continue;
        }
        if let Some(why) = blocked(view, row) {
            into.violate(task_name(*task), format!("a landing that can't move: {why}"));
            return;
        }
    }
}

fn blocked(view: &View<'_>, row: &LandingRow) -> Option<String> {
    let landing = &row.landing;
    if let Some(root) = view.root {
        let task = root.tasks().task(landing.task);
        if task.is_some_and(held) {
            return None;
        }
        if task.is_none_or(|row| matches!(row.phase, tasks::Phase::Ended(_))) {
            return Some("its task is not live".into());
        }
    }
    for member in &landing.members {
        if member.left.is_some() || member.landed.is_some() {
            continue;
        }
        if let Some(state) = member_blocked(view, member.change) {
            return Some(format!("member change {} has a {state} task", member.change));
        }
        if let Some(owner) = view.forge.landing_of(member.change)
            && owner != landing.task
        {
            return Some(format!("member change {} belongs to landing {owner}", member.change));
        }
    }
    // A single landing waits for the project's turn, not a stacked train.
    if !landing.round.branch.is_empty()
        && let Some(member) = landing.members.first()
        && let Some(owner) = view.forge.train_holder(
            Repository { forge: member.repository.forge, repository: member.repository.repository },
            &landing.branch,
        )
        && owner != landing.task
    {
        return Some(format!("landing {owner} holds its train"));
    }
    None
}

fn held(task: &tasks::TaskRecord) -> bool {
    matches!(task.phase, tasks::Phase::Held { .. }) || task.placed.is_some()
}

fn member_blocked(view: &View<'_>, task: u64) -> Option<&'static str> {
    if let Some(root) = view.root {
        return Some(match root.tasks().task(task) {
            None => "missing",
            Some(row) => match row.phase {
                tasks::Phase::Closing(_) => "closing",
                tasks::Phase::Ended(_) => "ended",
                _ if held(row) => "held",
                _ => return None,
            },
        });
    }
    let Some(row) = view.forge.change(task) else { return Some("missing") };
    if matches!(row.change.state, State::Ended) || row.ending.is_some() {
        Some("ended")
    } else if view.forge.inspection_releases().contains_key(&task) {
        Some("closing")
    } else if matches!(row.change.state, State::Held { .. }) {
        Some("held")
    } else {
        None
    }
}
