//! Template creation shared by the domain worlds.
use super::{config, limits};
use jig_domain_people as people;
use std::collections::BTreeMap;
use temper_engine_domain::{self as engine, Record};
/// Fill the configured chat template as jig's core worlds do.
#[must_use]
pub fn chat(project: u32, words: &[u8]) -> people::Ask {
    let creating = jig_core_world::fixtures::Creating::new(&config(91).core, limits().tasks.tree_tasks);
    jig_core_world::fixtures::create_chat(&creating, project, 0, words.into())
}

/// Fill the goal template with its supplied tracked terms.
#[must_use]
pub fn goal(
    project: u32,
    words: &[u8],
    executor: people::TaskExecutor,
    terms: jig_core_world::fixtures::Goal,
) -> people::Ask {
    goal_from(chat(project, words), executor, terms)
}

#[must_use]
pub fn goal_from(
    ask: people::Ask,
    executor: people::TaskExecutor,
    terms: jig_core_world::fixtures::Goal,
) -> people::Ask {
    let people::Ask::Create { project, mut batch } = ask else { unreachable!() };
    let member = &mut batch[0];
    member.executor = executor;
    member.budget = people::Budget::Named;
    member.authority.spend = terms.budget;
    member.authority.deadline = terms.deadline;
    member.tracked = Some(terms.priority);
    member.flags.ended_by = people::EndedBy::Result;
    member.flags.conversation = false;
    member.flags.listed = true;
    member.label = 2;
    people::Ask::Create { project, batch }
}

/// Fill a chat with the world's configured authority, including durable policy changes.
#[must_use]
pub fn configured_chat(
    creating: &jig_core_world::fixtures::Creating,
    rows: &BTreeMap<engine::Key, Record>,
    tasks: u32,
    project: u32,
    words: &[u8],
) -> people::Ask {
    let people::Ask::Create { mut batch, .. } =
        jig_core_world::fixtures::create_chat(creating, project, 0, words.into())
    else {
        unreachable!()
    };
    if let Some(Record::Core(jig_domain_core::Record::People(people::Stored::Policy { value, .. }))) =
        rows.get(&engine::Key::Core(jig_domain_core::Key::People(people::Key::Policy(project))))
        && let Some(role) = value.roles.iter().find(|role| role.number == 0)
    {
        batch[0].authority = role.authority.clone();
        batch[0].authority.spend = 0;
        batch[0].authority.delegation.tasks = role.authority.delegation.tasks.min(tasks).saturating_sub(1);
        batch[0].authority.delegation.depth = role.authority.delegation.depth.saturating_sub(1);
    }
    people::Ask::Create { project, batch }
}
