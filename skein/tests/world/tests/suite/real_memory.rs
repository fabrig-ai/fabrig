//! Checked real-process ownership, including buffers returned after a kill.

use skein_io::kernel::{Exit, Signal, Spawn};
use skein_lib::Duration;
use skein_shell::{Backend, Clock, Config, Kernel, open_root};
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
fn readiness_startup_calls_release_exactly_their_own_heap() {
    let mut world = real::World::new_checked(Judge).on(Backend::Readiness);
    // The first metered constructor closes before even opening its nested
    // startup kernel: both must find the backend's shared tables already made.
    world.spawn(|| {
        skein_shell::close_startup_fd(open_root(std::path::Path::new(".")).expect("ordinary startup descriptor"));
        let _kernel = Kernel::open_on(Config { operations: 1 }, Backend::Readiness).expect("nested startup kernel");
        let (reader, writer) = skein_shell::keeper::open_signal_pipe().expect("startup signal pipe");
        skein_shell::close_startup_fd(reader);
        skein_shell::close_startup_fd(writer);
        let root = open_root(std::path::Path::new(".")).expect("child launch root");
        let mut spawn = Spawn {
            reach: None,
            program: env!("CARGO_BIN_EXE_world_process_fixture").as_bytes().into(),
            args: Box::new([b"exit".as_slice().into(), b"0".as_slice().into()]),
            env: Box::new([]),
            root,
            dir: Box::new([]),
            pipes: Box::new([]),
        };
        let (child, terminal) = skein_shell::keeper::start_binary(&mut spawn, false).expect("real child");
        assert_eq!(terminal, None);
        assert!(skein_shell::keeper::wait_readable(child, Duration::from_secs(2)).expect("child exit readiness"));
        let exit = skein_shell::keeper::poll_child(child, true).expect("reap own child");
        skein_shell::close_startup_fd(child);
        skein_shell::close_startup_fd(root);
        assert_eq!(exit, Some(Exit::Code(0)));
        Script::new(&[])
    });
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    assert_eq!(outcome.try_release(), Ok(()));
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
