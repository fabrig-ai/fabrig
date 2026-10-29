//! Checked real-process ownership, including buffers returned after a kill.

use skein_io::kernel::{Exit, Signal, Spawn};
use skein_lib::Duration;
use skein_shell::{Clock, open_root};
use skein_world::{HostedProgram, Inherited, real};
use skein_world_tests::hosted::{Act, Judge, Script};

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
        let peak = outcome.killed[0].peak().expect("killed child checked on release");
        assert!(peak >= 60 * 1024, "the child held its full payload before the kill");
        assert!(peak <= CEILING);
        assert_eq!(outcome.peaks().expect("surviving root metered").len(), 1);
        drop(outcome);
    }
}
