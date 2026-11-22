//! Finite lagging views keep the batch workload's checks (testing/fakes.md, 2).
use skein_fake_forge_domain::{self as forge, Lag};
use skein_lib::Duration;
use skein_world::{
    faults::Kind,
    streams::{Stream, Streams},
    sweep::{Exclusion, Sweep},
};
use temper_engine_forge_world::{World, fake_config};

const HEAD_BEHIND: Kind = Kind { family: "lagging", name: "head_behind" };
const BASE_BEHIND: Kind = Kind { family: "lagging", name: "base_behind" };
const MERGE_LATE: Kind = Kind { family: "lagging", name: "merge_late" };

pub(super) const LATE_RECORD: &str = "a_late_manual_merge_record_finishes_the_landed_batch";
const RECORDING_SQUASH: &[&str] = &["recording a squashed member already landed on its base"];

pub(super) fn exclude_records(sweep: &mut Sweep) {
    sweep
        .exclude(Exclusion::new(
            "a_late_manual_merge_record_finishes_the_landed_batch",
            "landing train",
            MERGE_LATE,
            RECORDING_SQUASH,
        ))
        .expect("narrow late-record scenario");
}

pub(super) fn lag_world(world: &mut World, sweep: &Sweep) {
    let lag = Lag { chance: 1000, min: Duration::from_secs(1), max: Duration::from_secs(3) };
    forge::lag(world.forge_mut(), lag);
    let exclusion = sweep.leaves_out(MERGE_LATE, RECORDING_SQUASH).expect("late-record exclusion");
    world.exclude_lagging_records(lag, exclusion);
}

#[test]
fn lagging_views_keep_the_batches_invariants() {
    let mut sweep = Sweep::new("lagging_views_keep_the_batches_invariants");
    for kind in [HEAD_BEHIND, BASE_BEHIND, MERGE_LATE] {
        sweep.configure(kind);
    }
    exclude_records(&mut sweep);
    for seed in 1..=2 {
        eprintln!("seed {seed}");
        let streams = Streams::new(seed);
        let workload = streams.seed_of(Stream::Workload, "landing_batches");
        let mut world =
            World::batch_turn_in(&streams, Some(super::landing_batches::workload_workflow(workload)), fake_config());
        lag_world(&mut world, &sweep);
        let (world, _) = super::landing_batches::run_in(world, workload, 240);
        sweep.ran(world.ledger());
    }
    sweep.check_fell().expect("configured lagging views fell");
    assert!(sweep.left_out(LATE_RECORD) > 0, "the late-record exclusion was reached");
    eprintln!("{LATE_RECORD}: {} draws left out", sweep.left_out(LATE_RECORD));
}
