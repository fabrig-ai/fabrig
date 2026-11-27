//! The fleet and named-call records as read in the core world.

use jig_core_world::fixtures::{ask, finish_chat};
use jig_domain_people as people;
use skein_world::inspect::{Inspection, Release, Table};
use smith_channel::Record;

use super::{ReplyTo, Setup, World, core, inspect, root};

pub(super) fn table<'a>(inspection: &'a Inspection, name: &str) -> &'a Table {
    inspection.tables().iter().find(|table| table.name == name).unwrap_or_else(|| panic!("missing {name} table"))
}

pub(super) fn assert_call(inspection: &Inspection, key: core::CallKey, holds: &[&str]) {
    let records: Vec<_> =
        inspection.relateds().iter().filter(|record| record.relation == "a run's host calls").collect();
    assert_eq!(records.len(), holds.len(), "{records:?}");
    let core::CallKey { task, attempt, completion, position } = key;
    for (record, holds) in records.into_iter().zip(holds) {
        assert_eq!(record.key, format!("task {task} attempt {attempt} completion {completion} position {position}"));
        assert_eq!(record.holds, *holds);
    }
}

#[test]
fn a_running_attempt_is_placed_until_its_terminal() {
    let seed = 11742;
    let mut world = World::new(Setup { seed, ..Setup::default() });
    let (run, attempt) = world.assignments[0];
    let name = format!("run {run} attempt {attempt}");
    let inspection = inspect(&world, seed);
    let placements = table(&inspection, "placements");
    assert_eq!(placements.entries.len(), 1);
    assert_eq!(placements.entries[0].key, name);
    assert_eq!(placements.entries[0].rule, "its attempt's end");
    assert_eq!(placements.entries[0].release, Release::Pending);
    let [placement] = inspection.relateds() else { panic!("one placement: {:?}", inspection.relateds()) };
    assert_eq!(placement.relation, "a run's placement");
    assert_eq!(placement.key, name);
    assert_eq!(placement.holds, "a placement");
    finish_chat(&mut world, 0, b"done");
    let inspection = inspect(&world, seed);
    assert_eq!(table(&inspection, "placements").entries, []);
    assert_eq!(inspection.relateds(), []);
}

#[test]
fn a_relayed_call_leaves_an_answer_for_the_next_attempt() {
    let seed = 11743;
    let mut world = World::new(Setup { seed, ..Setup::default() });
    let key = world.call_key(2);
    let name = smith_channel::CallName { activation: key.attempt, completion: key.completion, position: key.position }
        .encode(&smith_channel::CEILINGS)
        .expect("a Smith call name");
    let right = world.host_call(&name, b"deliver", b"opaque input", true);
    assert_call(&inspect(&world, seed), key, &["a relay in flight"]);
    world.send(root::Event::Core(core::Event::NamedAnswer {
        to: ReplyTo::new(right),
        key,
        part: core::CallPart::Unavailable,
    }));
    world.send(root::Event::Core(core::Event::SettledCall {
        to: ReplyTo::new(right),
        key,
        name,
        call: core::SettledCall {
            serial: 0,
            tool: b"deliver".as_slice().into(),
            answer: core::SettledAnswer::Delivery { outcome: core::DeliveryOutcome::Delivered, evidence: Box::new([]) },
        },
    }));
    world.park_for_message(0);
    world.say(74);
    let successor = world.assigned.as_ref().expect("successor");
    assert_ne!(successor.attempt, key.attempt);
    assert_eq!(successor.answered.len(), 1);
    assert_eq!(successor.answered[0].key, key);
    assert_call(&inspect(&world, seed), key, &["an answer retained for replay"]);
}

#[test]
fn cancelling_a_run_names_its_unsettled_call_as_cancelled() {
    let seed = 11744;
    let mut world = World::new(Setup { seed, ..Setup::default() });
    let key = world.call_key(1);
    world.hold_writes = true;
    world.call(key.completion, 206);
    assert_eq!(world.answers, [], "the effect still owes its answer");
    assert_call(&inspect(&world, seed), key, &["a call record"]);
    ask(&mut world, 92, people::Ask::Cancel { project: 1, task: key.task, reason: Box::new([]), grace: None });
    assert_call(&inspect(&world, seed), key, &["a cancelled run's call"]);
}
