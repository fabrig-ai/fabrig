//! A rule follows a change's diff, not edits inherited from its base (change/gates.md, 2).
use skein_lib::Wall;
use temper_engine_domain_forge::{self as top, settings};
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{
    World,
    changes::{change_row, change_step, change_step_task, change_step_with_release},
    translate,
};
use temper_protocol_reaches::{Classes, PathSet};

fn classes(pattern: &[u8]) -> settings::Asked {
    settings::Asked {
        kind: 8,
        scope: 5,
        number: 1,
        edit: settings::Edit::Set(settings::SizedValue {
            size: 128,
            value: settings::Value::Classes(Classes {
                items: Box::new([]),
                sets: Box::new([PathSet {
                    number: 1,
                    name: b"protocol".as_slice().into(),
                    patterns: Box::new([pattern.into()]),
                }]),
            }),
        }),
    }
}

fn fixture(path: &[u8]) -> World {
    let mut world = World::new(17);
    world.adopt();
    world.keep_settings(
        1,
        &[
            classes(b"src/protocol.*"),
            settings::Asked {
                kind: 2,
                scope: 5,
                number: 1,
                edit: settings::Edit::Set(settings::SizedValue {
                    size: 128,
                    value: settings::Value::Rule(settings::LandingRule {
                        sets: Box::new([1]),
                        pattern: top::GatePattern {
                            segments: Box::new([b"branch".as_slice().into()]),
                            last: top::GateLast::Exact(b"main".as_slice().into()),
                        },
                        enforce: true,
                        ci: true,
                        up_to_date: true,
                        gates: Box::new([settings::RuleGate {
                            number: 9,
                            blocking: true,
                            freshness: top::JudgeFreshness::Exact,
                        }]),
                        approvals: Box::new([]),
                    }),
                }),
            },
        ],
    );
    world.produce_file(b"temper/51", path, b"change");
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Passed);
    let head = translate::commit(world.branch(b"temper/51"));
    let mut row = change_row();
    row.pull = Some(world.open_pull(b"temper/51", b"main"));
    row.change.last_head = Some(head);
    row.change.state = change::State::Queued { head, ready_since: Wall::EPOCH };
    world.event(top::Event::Change { row });
    world
}

fn asks_gate(world: &mut World, entry: u64) {
    let head = translate::commit(world.branch(b"temper/51"));
    assert_eq!(
        change_step(world, entry, change::Status::Passed, Box::new([])),
        change::Decision::Delegate(change::Delegate::Gate { number: 9, head })
    );
    assert_eq!(world.writes(), 0, "no merge while its blocking gate is short");
}

fn lands_without_gate(world: &mut World, first: u64) {
    for entry in first..first + 16 {
        let decision = change_step(world, entry, change::Status::Passed, Box::new([]));
        assert!(!matches!(decision, change::Decision::Delegate(change::Delegate::Gate { .. })), "{decision:?}");
        if matches!(decision, change::Decision::Finish { .. }) {
            return;
        }
    }
    panic!("untouching change must land");
}

#[test]
fn a_path_gate_waits_only_for_changes_touching_its_set() {
    let mut protocol = fixture(b"src/protocol.c");
    asks_gate(&mut protocol, 1);
    let mut docs = fixture(b"docs/a.md");
    lands_without_gate(&mut docs, 1);
    assert_eq!(docs.writes(), 1);
}

#[test]
fn a_new_head_newly_touching_the_set_asks_its_gate() {
    let mut world = fixture(b"docs/a.md");
    let decision = change_step_task(&mut world, 51, 1, change::Status::Passed, Box::new([]), false, true);
    assert!(!matches!(decision, change::Decision::Delegate(change::Delegate::Gate { .. })));
    assert_eq!(world.writes(), 0);
    world.push_file(b"temper/51", b"src/protocol.h", b"new interface");
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Passed);
    assert_eq!(
        change_step(&mut world, 2, change::Status::Passed, Box::new([])),
        change::Decision::Hold(change::Hold::BranchMoved)
    );
    let head = translate::commit(world.branch(b"temper/51"));
    assert_eq!(
        change_step_with_release(&mut world, 3, change::Status::Passed, Box::new([]), true),
        change::Decision::Delegate(change::Delegate::Gate { number: 9, head })
    );
    assert_eq!(world.writes(), 0);
}

#[test]
fn an_update_does_not_touch_protocol_edits_inherited_from_the_base() {
    let mut world = fixture(b"docs/a.md");
    world.push_file(b"main", b"src/protocol.c", b"base interface");
    world.status(b"main", skein_fake_forge_domain::api::Check::Passed);
    let decision = change_step(&mut world, 1, change::Status::Passed, Box::new([]));
    assert!(matches!(decision, change::Decision::Effect(change::Effect::Update { .. })), "{decision:?}");
    world.status(b"temper/51", skein_fake_forge_domain::api::Check::Passed);
    lands_without_gate(&mut world, 2);
    assert_eq!(world.writes(), 2, "update and merge");
}

#[test]
fn a_refused_comparison_counts_as_touching_every_set() {
    let mut world = fixture(b"docs/a.md");
    world.reject_comparisons();
    asks_gate(&mut world, 1);
}

#[test]
fn widening_a_set_wakes_the_live_change_and_asks_its_gate() {
    let mut world = fixture(b"docs/a.md");
    change_step_task(&mut world, 51, 1, change::Status::Passed, Box::new([]), false, true);
    assert_eq!(world.writes(), 0);
    world.take_seen();
    world.keep_settings(2, &[classes(b"docs/**")]);
    assert!(world.seen().iter().any(|request| matches!(request, top::Request::WakeProcedure { task: 51 })));
    asks_gate(&mut world, 2);
}
