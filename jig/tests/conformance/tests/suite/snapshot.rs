use jig_conformance::{Cut, scenarios::Scenario};
use jig_conformance_world::{Testing, replay};

#[test]
fn a_forked_snapshot_replays_to_the_durable_rows() {
    let world = replay::fresh(Scenario::LateCopy, Cut::None, 81, false).expect("fresh story");
    let rows = world.store.rows.clone();
    let forked = jig_conformance::Harness::<Testing>::from_snapshot(world.cold_snapshot().expect("snapshot readback"))
        .expect("fork restarts");
    assert_eq!(forked.store.rows, rows);
}

#[test]
fn serialized_outside_fields_preserve_values_but_not_list_capacity() {
    use jig_domain_people::{Ask, Operation};
    use skein_lib::List;
    let mut world = replay::fresh(Scenario::LateCopy, Cut::None, 81, false).expect("fresh story");
    let ask = |capacity| {
        let mut operations = List::with_capacity(capacity);
        operations
            .push(Operation::Message { task: 1, words: b"List {\n    capacity: 42,\n}".as_slice().into() })
            .expect("one item fits");
        (1, Ask::Compound { project: 1, operations })
    };
    world.peers.pending_asks.push(ask(1));
    let original = replay::outside(&world);
    let mut bytes = Vec::new();
    original.text().write(&mut bytes).expect("write every field");
    let decoded = replay::OutsideText::read(bytes.as_slice()).expect("decode every field");
    assert_eq!(decoded, original.text());
    world.peers.pending_asks = vec![ask(9)];
    let same = replay::outside(&world);
    assert_eq!(original.first_difference(&same), None, "structural equality ignores List's cap");
    assert_eq!(decoded.first_difference(&same.text()), None, "transport equality does too");
    world.peers.pending_asks.clear();
    let different = replay::outside(&world);
    assert!(original.first_difference(&different).expect("structural difference").starts_with("peers:"));
    assert!(decoded.first_difference(&different.text()).expect("transport difference").starts_with("peers:"));
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(replay::OutsideText::read(trailing.as_slice()).is_err(), "whole observation, no trailing data");
    assert!(replay::OutsideText::read(&bytes[..bytes.len() - 1]).is_err(), "no truncated fields");
}

pub(super) fn compare(scenario: Scenario, seed: u64, selected: Option<&[u64]>) {
    let states = tempfile::tempdir().expect("snapshot observations directory");
    let resumed_states = states.path().to_owned();
    let visit = move |cut, resumed: jig_conformance_world::actions::World| {
        assert!(resumed.outcome().crashes.contains(&cut));
        let file = std::fs::File::create(resumed_states.join(cut.to_string())).expect("resumed observation file");
        let mut file = std::io::BufWriter::new(file);
        replay::outside_text(&resumed).write(&mut file).expect("complete resumed fields");
        std::io::Write::flush(&mut file).expect("flush resumed fields");
    };
    let baseline = replay::fork_snapshots(scenario, seed, Some(true), selected, visit).expect("snapshot cuts");
    let cuts = baseline.forked;
    assert_eq!(cuts, selected.map_or_else(|| (1..=baseline.commits).collect::<Vec<_>>(), <[u64]>::to_vec));
    assert!(baseline.commits >= if selected.is_some() { 3 } else { 1 });
    let fresh = replay::fresh_cuts(scenario, Some(&cuts), seed, Some(true), |cut, fresh| {
        let file = std::fs::File::open(states.path().join(cut.to_string())).expect("resumed selected cut");
        let actual = replay::OutsideText::read(std::io::BufReader::new(file)).expect("complete resumed fields");
        if let Some(difference) = actual.first_difference(&replay::outside_text(fresh)) {
            panic!("{scenario:?}, cut {cut}, seed {seed}: {difference}");
        }
    })
    .expect("fresh selected cuts");
    assert_eq!(fresh.forked, cuts, "every selected cut compared in its fresh child");
}
#[test]
fn late_copy_snapshots_match_complete_fresh_state() {
    compare(Scenario::LateCopy, 81, Some(&[1, 2, 3]));
}
#[test]
fn other_hand_snapshots_match_complete_fresh_state() {
    compare(Scenario::OtherHand, 82, Some(&[1, 2, 3]));
}
#[test]
fn uncertain_restarts_snapshots_match_complete_fresh_state() {
    compare(Scenario::UncertainRestarts, 83, Some(&[1, 2, 3]));
}
#[test]
fn changed_judge_snapshots_match_complete_fresh_state() {
    compare(Scenario::ChangedJudge, 84, Some(&[1, 2, 3]));
}
#[test]
fn shrinking_pool_snapshots_match_complete_fresh_state() {
    compare(Scenario::ShrinkingPool, 85, Some(&[1, 2, 3]));
}
#[test]
fn narrowing_policy_snapshots_match_complete_fresh_state() {
    compare(Scenario::NarrowingPolicy, 86, Some(&[1, 2, 3]));
}
#[test]
fn standing_task_snapshots_match_complete_fresh_state() {
    compare(Scenario::StandingTask, 87, Some(&[1, 2, 3]));
}
#[test]
fn run_budget_snapshots_match_complete_fresh_state() {
    compare(Scenario::RunBudget, 88, Some(&[1, 2, 3]));
}
