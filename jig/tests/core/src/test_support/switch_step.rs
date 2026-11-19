//! Start phases and durable wires shared by memory and disk stories (jig/docs/design/domain/testing.md, 9).
use crate::world::World;
use jig_domain_core as core;
use jig_test_domain as root;

pub const RESTORES: &[core::start::Phase] = &[
    core::start::Phase::LoadCore,
    core::start::Phase::LoadInbox,
    core::start::Phase::RestoreConnector { connector: 1 },
    core::start::Phase::RestoreConnector { connector: 2 },
    core::start::Phase::Sweep,
    core::start::Phase::Switch,
];

#[must_use]
pub fn mark(world: &World, since: usize) -> &[root::Write] {
    let marks: Vec<_> = world.commits[since..].iter().filter(|writes| writes.contains(&root::Write::Wires)).collect();
    assert_eq!(marks.len(), 1, "one start, one mark");
    marks[0]
}

pub fn wires(world: &mut World) -> Box<[u8]> {
    world
        .store
        .retained()
        .expect("complete retained inventory")
        .into_iter()
        .find_map(|(key, value)| {
            (jig_protocol_store::keys::decode_own(&key) == Some(jig_protocol_store::keys::Own::Wires)).then_some(value)
        })
        .expect("component wires row")
}
