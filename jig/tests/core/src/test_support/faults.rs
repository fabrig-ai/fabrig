//! Commit selection shared by memory and disk cuts (jig/docs/design/domain/testing.md, 9).
use crate::world::World;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

/// The first commit of `world` saving a row `chosen` picks.
pub fn commit_saving(world: &World, chosen: impl Fn(&root::Record) -> bool) -> u64 {
    let from_end = world
        .commits
        .iter()
        .rev()
        .rposition(|writes| writes.iter().any(|write| matches!(write, root::Write::Save(row) if chosen(row))))
        .expect("chosen row is saved");
    // Fixture commits are absent from `commits`; count back from the durable end.
    world.store.applied - u64::try_from(from_end).expect("commit distance")
}

#[must_use]
pub fn chat(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(_))))
}
