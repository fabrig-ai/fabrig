//! A finite busy spell keeps the batch checks (testing/fakes.md, 2).
use skein_fake_forge_domain::Called;
use skein_lib::Duration;
use skein_world::{
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_engine_forge_world::{World, fake_config};
use temper_faults::forge::{BUSY, LANDING, LATE, NO_ROOM};

#[test]
fn busy_and_slow_forges_keep_the_batches_invariants() {
    let mut sweep = Sweep::new("busy_and_slow_forges_keep_the_batches_invariants");
    for kind in BUSY {
        sweep.configure(kind);
    }
    let mut refused_read = false;
    let mut landing_merge = false;
    let mut late_merge = false;
    for seed in 1..=2 {
        eprintln!("seed {seed}");
        let streams = Streams::new(seed);
        let workload = streams.seed_of(Stream::Workload, "landing_batches");
        let mut config = fake_config();
        config.limits.calls = 2;
        config.land_min = Duration::from_secs(1);
        config.land_max = Duration::from_secs(2);
        let mut world =
            World::batch_turn_in(&streams, Some(super::landing_batches::workload_workflow(workload)), config);
        world.landing_step_horizon(120);
        world.crowd_next_read_for(Duration::from_secs(1));
        let mut busy = world.forge_config().clone();
        busy.unavailable = 200;
        busy.timeouts = 200;
        busy.late = 200;
        // Beyond the connector's ten-second call lifetime, not just its ordinary latency.
        busy.late_min = Duration::from_secs(12);
        busy.late_max = Duration::from_secs(14);
        busy.rate_limit = 2;
        busy.rate_window = Duration::from_secs(3);
        let until = world.now().saturating_add(Duration::from_secs(60));
        world.fault_calls_until(until, busy);
        let mut merge = world.forge_config().clone();
        merge.landing = 1000;
        merge.late = 1000;
        merge.late_min = Duration::from_secs(12);
        merge.late_max = Duration::from_secs(14);
        world.fault_next_merge_until(world.now().saturating_add(Duration::from_secs(600)), merge);
        let (world, _) = super::landing_batches::run_in(world, workload, 360);
        refused_read |= world.fell_on(NO_ROOM, Called::Read);
        landing_merge |= world.fell_on(LANDING, Called::Merge);
        late_merge |= world.fell_on(LATE, Called::Merge);
        sweep.ran(world.ledger());
    }
    assert!(refused_read, "reads were refused for lack of call room");
    assert!(landing_merge, "a merge landed after its timeout");
    assert!(late_merge, "a merge answered past the connector's deadline");
    sweep.check_fell().expect("configured busy faults fell");
}
