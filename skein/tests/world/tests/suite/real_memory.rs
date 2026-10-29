//! Checked real-process ownership, including buffers returned after a kill.

use skein_io::kernel::{Exit, Signal, Spawn};
use skein_lib::Duration;
use skein_shell::{Clock, open_root};
use skein_world::{HostedProgram, Inherited, real};
use skein_world_tests::hosted::{Act, Judge, Script};

#[test]
fn a_ring_the_loop_grows_is_charged_to_its_process() {
    use skein_world::LastWord;
    use skein_world_tests::{Later, Script};

    const OPERATIONS: u32 = 64;

    let mut world = real::World::new_checked(Later(LastWord::LastExpectation));
    world.spawn(|| Script::usage_batch(OPERATIONS));
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    let process = outcome.procs[0].batch.as_ref().expect("usage batch");
    assert_eq!(process.completed, OPERATIONS);
    assert_eq!(process.largest_delivery, OPERATIONS, "one delivery grows through several ring capacities");
    let before_growth = process.before_growth;
    let peak = outcome.heap.as_ref().expect("checked process")[0].0;
    drop(outcome);
    assert!(peak > before_growth, "the process's peak includes the ring the loop grew");
}

fn exchange(_spawn: &Spawn, inherited: &Inherited) -> Script {
    Script::child(inherited, &[Act::Read(0), Act::Write(1, b"reply")])
}

fn waiting(_spawn: &Spawn, inherited: &Inherited) -> Script {
    let mut script = Script::child(inherited, &[Act::Read(0), Act::Write(1, b"reply"), Act::Read(0)]);
    script.hog = 60 * 1024;
    script
}

fn run(make: fn(&Spawn, &Inherited) -> Script, acts: &[Act], ceiling: Option<u64>) -> real::CheckedOutcome<Script> {
    let mut world = match ceiling {
        Some(ceiling) => real::World::new_within(Judge, ceiling),
        None => real::World::new_checked(Judge),
    };
    world.host(HostedProgram { program: Box::from(&b"hosted"[..]), make, instances: 1, operations: 8 });
    let root = open_root(std::path::Path::new(".")).expect("startup root");
    world.spawn_with_fds(vec![root], || Script::parent(root, acts));
    world.run(&Clock::new(), Duration::from_secs(2))
}

#[test]
fn normal_hosted_exit_preserves_separate_peaks_and_exact_final_release() {
    // Each real exchange process: measured peak 2,256 B plus 25%, rounded up.
    const CEILING: u64 = 2_900;
    let outcome =
        run(exchange, &[Act::Spawn, Act::Write(0, b"hello"), Act::Read(1), Act::Read(1), Act::Wait], Some(CEILING));
    assert_eq!(outcome.procs[0].received, b"reply");
    assert_eq!(outcome.procs[1].received, b"hello");
    assert_eq!(outcome.procs[0].child_exit, Some(Exit::Code(0)));
    let peaks = outcome.heap.as_ref().expect("checked processes");
    assert_eq!(peaks.len(), 2);
    for (peak, bound) in peaks {
        assert_eq!(*bound, CEILING);
        assert!(*peak > 0 && *peak <= CEILING);
    }
    assert_eq!(outcome.held.as_ref().expect("final ownership ledger").len(), 2);
    drop(outcome);
}

#[test]
fn kill_returns_pending_buffers_before_exact_child_release() {
    // Each process through the kill: measured peak 62,528 B plus 25%, rounded up.
    const CEILING: u64 = 79_000;
    for _ in 0..16 {
        let outcome = run(
            waiting,
            &[Act::Spawn, Act::Write(0, b"hello"), Act::Read(1), Act::Signal(Signal::Kill), Act::Read(1), Act::Wait],
            Some(CEILING),
        );
        assert_eq!(outcome.procs.len(), 1);
        assert_eq!(outcome.procs[0].child_exit, Some(Exit::Signal(9)));
        assert_eq!(outcome.killed.len(), 1);
        let (peak, bound) = outcome.killed[0].heap.expect("killed child checked on release");
        assert_eq!(bound, CEILING);
        assert!(peak >= 60 * 1024, "the child held its full payload before the kill");
        assert!(peak <= CEILING);
        assert_eq!(outcome.heap.as_ref().expect("surviving root metered").len(), 1);
        drop(outcome);
    }
}

fn hogging(_spawn: &Spawn, inherited: &Inherited) -> Script {
    let mut script = Script::child(inherited, &[]);
    script.hog = 64 * 1024;
    script
}

#[test]
#[should_panic(expected = "past its scenario's ceiling")]
fn hosted_iteration_cannot_exceed_its_scenarios_ceiling() {
    // Below the child's 64 KiB hog, above its construction peak.
    const CEILING: u64 = 32 * 1024;
    drop(run(hogging, &[Act::Spawn, Act::Read(1), Act::Wait], Some(CEILING)));
}

fn leaky(_spawn: &Spawn, inherited: &Inherited) -> Script {
    let mut script = Script::child(inherited, &[]);
    script.hog = 1;
    script.leak = true;
    script
}

#[test]
#[should_panic(expected = "must release exactly its own metered heap")]
fn settled_child_leak_is_detected_at_final_drop() {
    drop(run(leaky, &[Act::Spawn, Act::Read(1), Act::Wait], None));
}

fn leaky_killed(_spawn: &Spawn, inherited: &Inherited) -> Script {
    let _leaked = Box::leak(Box::new([0_u8; 64]));
    Script::child(inherited, &[Act::Read(0)])
}

#[test]
#[should_panic(expected = "a leak")]
fn killed_child_leak_is_detected_after_kernel_settlement() {
    drop(run(leaky_killed, &[Act::Spawn, Act::Signal(Signal::Kill), Act::Read(1), Act::Wait], None));
}

#[test]
#[should_panic(expected = "past its scenario's ceiling")]
fn root_construction_cannot_exceed_its_scenarios_ceiling() {
    // Construction holds 64 KiB; the scenario permits half of it.
    const CEILING: u64 = 32 * 1024;
    let mut world = real::World::new_within(Judge, CEILING);
    world.spawn(|| {
        let mut script = Script::new(&[]);
        script.held = vec![0; 64 * 1024].into_boxed_slice();
        script
    });
}

#[derive(Debug)]
struct AllocatingJudge {
    storage: Vec<u8>,
}

impl skein_world::Referee<Script> for AllocatingJudge {
    fn act(&mut self, _now: skein_lib::Time, _procs: &mut [Script]) {
        self.storage.resize(128 * 1024, 1);
    }
    fn observe(&mut self, _now: skein_lib::Time, _procs: &[Script]) {}
    fn next_deadline(&self) -> Option<skein_lib::Time> {
        None
    }
    fn overdue(&self, _now: skein_lib::Time) -> Option<String> {
        None
    }
    fn passed(&self) -> bool {
        true
    }
}

#[test]
fn referee_storage_is_excluded_from_process_accounting() {
    let mut world = real::World::new_controlled_checked(|_| AllocatingJudge { storage: Vec::new() });
    world.spawn(|| Script::new(&[]));
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    assert!(outcome.heap.as_ref().expect("root checked")[0].0 < 64 * 1024);
    drop(outcome);
}
