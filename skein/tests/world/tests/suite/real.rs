//! The hosted scripts use real pipes and the same factories as simulated worlds.

use skein_io::kernel::{Exit, Signal, Spawn};
use skein_lib::Duration;
use skein_shell::{Clock, open_root};
use skein_world::{HostedProgram, Inherited, real};
use skein_world_tests::hosted::{Act, Judge, Script};

fn exchange(_spawn: &Spawn, inherited: &Inherited) -> Script {
    Script::child(inherited, &[Act::Read(0), Act::Write(1, b"reply")])
}

fn wait_again(_spawn: &Spawn, inherited: &Inherited) -> Script {
    Script::child(inherited, &[Act::Read(0), Act::Write(1, b"reply"), Act::Read(0)])
}

fn run(make: fn(&Spawn, &Inherited) -> Script, acts: &[Act]) -> real::Outcome<Script> {
    let mut world = real::World::new(Judge);
    world.host(HostedProgram { program: Box::from(&b"hosted"[..]), make, instances: 1, operations: 8 });
    let root = open_root(std::path::Path::new(".")).expect("startup root");
    world.spawn_with_fds(vec![root], || Script::parent(root, acts));
    world.run(&Clock::new(), Duration::from_secs(2))
}

#[test]
fn hosted_parent_and_child_exchange_over_real_pipes() {
    let outcome = run(exchange, &[Act::Spawn, Act::Write(0, b"hello"), Act::Read(1), Act::Read(1), Act::Wait]);
    assert_eq!(outcome.procs[0].received, b"reply");
    assert_eq!(outcome.procs[1].received, b"hello");
    assert_eq!(outcome.procs[0].child_exit, Some(Exit::Code(0)));
    assert!(outcome.killed.is_empty());
}

#[test]
fn kill_drops_child_and_settles_pending_read_before_parent_wait() {
    let outcome = run(
        wait_again,
        &[Act::Spawn, Act::Write(0, b"hello"), Act::Read(1), Act::Signal(Signal::Kill), Act::Read(1), Act::Wait],
    );
    assert_eq!(outcome.procs.len(), 1);
    assert_eq!(outcome.procs[0].received, b"reply");
    assert_eq!(outcome.procs[0].child_exit, Some(Exit::Signal(9)));
    assert_eq!(outcome.killed[0].exit, Exit::Signal(9));
}

fn terminated(_spawn: &Spawn, inherited: &Inherited) -> Script {
    Script::child(inherited, &[Act::ReadSignal, Act::Write(1, b"terminated")])
}

#[test]
fn parent_termination_arrives_as_a_signal_record_on_the_child_pipe() {
    let outcome = run(terminated, &[Act::Spawn, Act::Signal(Signal::Terminate), Act::Read(1), Act::Read(1), Act::Wait]);
    assert_eq!(outcome.procs[0].received, b"terminated");
    assert_eq!(outcome.procs[1].signals, [skein_io::kernel::ServiceSignal::Terminate]);
    assert_eq!(outcome.procs[0].child_exit, Some(Exit::Code(0)));
}

#[test]
fn parent_interrupt_arrives_on_the_hosted_child_signal_pipe() {
    use skein_world_tests::hosted::{check_interrupt, interrupted_child};
    let outcome = run(
        interrupted_child,
        &[
            Act::Spawn,
            Act::Signal(Signal::Interrupt),
            Act::Read(1),
            Act::Write(0, b"hello"),
            Act::Read(1),
            Act::Read(1),
            Act::Wait,
        ],
    );
    check_interrupt(&outcome.procs);
}

struct PlainJudge;

impl skein_world::Referee<skein_world::end_to_end::Binary> for PlainJudge {
    fn observe(&mut self, _now: skein_lib::Time, _procs: &[skein_world::end_to_end::Binary]) {}
    fn passed(&self) -> bool {
        true
    }
}

#[test]
fn an_unhandled_interrupt_ends_a_real_child_with_signal_two() {
    use skein_world::end_to_end::{Binary, Command, Mode};
    for group in [false, true] {
        let command = Command {
            program: skein_scratch::paths::binary("world_process_fixture"),
            arguments: vec!["never".into()],
            environment: vec![],
            directory: "/tmp".into(),
        };
        let mut binary = Binary::start(command, Mode::Pipes, 1024).expect("the plain child starts");
        if group {
            binary.signal_group(Signal::Interrupt);
        } else {
            binary.signal(Signal::Interrupt);
        }
        let mut world = real::World::new(PlainJudge);
        world.spawn_with_fds(binary.descriptors(), || binary);
        let outcome = world.run(&Clock::new(), Duration::from_secs(2));
        assert_eq!(outcome.procs[0].exit_status(), Some(Exit::Signal(2)));
    }
}

#[derive(Debug)]
struct SignalJudge {
    control: real::Controls,
    child: bool,
    sent: bool,
}

impl skein_world::Referee<Script> for SignalJudge {
    fn act(&mut self, _now: skein_lib::Time, procs: &mut [Script]) {
        if self.sent {
            return;
        }
        if self.child {
            // This is the parent's public spawn result, not its service state.
            if !procs[0].results.iter().any(|result| matches!(result, Ok(skein_io::kernel::Done::Spawned { .. }))) {
                return;
            }
            self.control.signal(1, skein_io::kernel::ServiceSignal::Interrupt);
        } else {
            self.control.signal(0, skein_io::kernel::ServiceSignal::Terminate);
            self.control.signal(1, skein_io::kernel::ServiceSignal::Interrupt);
        }
        self.sent = true;
    }
    fn observe(&mut self, _now: skein_lib::Time, procs: &[Script]) {
        if self.child
            && !self.sent
            && procs[0].results.iter().any(|result| matches!(result, Ok(skein_io::kernel::Done::Spawned { .. })))
        {
            self.control.signal(1, skein_io::kernel::ServiceSignal::Interrupt);
            self.sent = true;
        }
    }
    fn passed(&self) -> bool {
        self.sent
    }
}

#[test]
fn referee_real_signal_reaches_only_the_signalfd_reader_and_pipe_signal_reaches_its_host() {
    let mut world = real::World::new_controlled(|control| SignalJudge { control, child: false, sent: false });
    world.spawn_signalfd(|signal| {
        Script::child(&Inherited { roots: vec![], appends: vec![], pipes: vec![], signal }, &[Act::ReadSignal])
    });
    world.spawn_signals(|signal| {
        Script::child(&Inherited { roots: vec![], appends: vec![], pipes: vec![], signal }, &[Act::ReadSignal])
    });
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    assert_eq!(outcome.procs[0].signals, [skein_io::kernel::ServiceSignal::Terminate]);
    assert_eq!(outcome.procs[1].signals, [skein_io::kernel::ServiceSignal::Interrupt]);
}

#[test]
fn referee_signal_reaches_a_hosted_child_through_its_pipe() {
    let mut world = real::World::new_controlled(|control| SignalJudge { control, child: true, sent: false });
    world.host(HostedProgram { program: Box::from(&b"hosted"[..]), make: terminated, instances: 1, operations: 8 });
    let root = open_root(std::path::Path::new(".")).expect("startup root");
    world.spawn_with_fds(vec![root], || Script::parent(root, &[Act::Spawn, Act::Read(1), Act::Read(1), Act::Wait]));
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    assert_eq!(outcome.procs[0].received, b"terminated");
    assert_eq!(outcome.procs[1].signals, [skein_io::kernel::ServiceSignal::Interrupt]);
}

#[test]
fn a_zero_patience_deadline_reports_the_failing_iteration() {
    // Zero patience meets the runnable-wait threshold. Exercise its failure
    // diagnostic outside nextest so a successful process exit cannot skip it.
    const CHILD: &str = "SKEIN_ZERO_PATIENCE_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", "real::a_zero_patience_deadline_reports_the_failing_iteration", "--nocapture"])
            .env(CHILD, "1")
            .env_remove("NEXTEST")
            .output()
            .expect("run the isolated zero-patience diagnostic check");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"), "the child ran its assertions");
        return;
    }
    let failure = std::panic::catch_unwind(|| {
        let mut world = real::World::new(Judge);
        world.spawn(|| Script::new(&[]));
        world.run(&Clock::new(), Duration::ZERO)
    })
    .expect_err("zero patience fails before the first script iteration");
    let text = failure.downcast_ref::<String>().expect("the deadline fails with its diagnostic");
    let accounting = text
        .lines()
        .next()
        .expect("deadline header")
        .strip_prefix("the real loop settles within 0 ms; ")
        .expect("the real deadline names its budget")
        .split_once("; thread CPU elapsed (schedstat): ")
        .expect("the real deadline reports its thread CPU source")
        .1;
    let (cpu, waited) = accounting
        .split_once("; thread runnable wait elapsed (schedstat): ")
        .expect("the deadline also names its runnable wait source");
    for count in [cpu, waited] {
        if count != "UNAVAILABLE (ns)" {
            let nanos = count.strip_suffix(" ns").expect("available accounting has nanosecond units");
            assert!(nanos.parse::<u64>().is_ok(), "available accounting is an unsigned elapsed count: {count}");
        }
    }
}

#[test]
#[cfg(target_os = "linux")]
fn a_starved_referee_drops_hosts_before_reporting_its_figures() {
    use skein_io::kernel::{Complete, Submit};
    use skein_lib::{Queue, Time, Wall};
    use skein_world::{Deadline, Expired, Host, Referee, Starved};
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};

    const CHILD: &str = "SKEIN_STARVED_REFEREE_CHILD";
    static DROPPED: AtomicBool = AtomicBool::new(false);
    static DROPPED_AT_FAILURE: AtomicBool = AtomicBool::new(false);
    static EXPECTED: OnceLock<String> = OnceLock::new();

    #[derive(Debug)]
    struct DroppedHost {
        completions: Queue<Complete>,
        submissions: Queue<Submit>,
    }
    impl Drop for DroppedHost {
        fn drop(&mut self) {
            DROPPED.store(true, Ordering::SeqCst);
        }
    }
    impl Host for DroppedHost {
        fn iterate(&mut self, _now: Time, _wall: Wall) {}
        fn completions(&mut self) -> &mut Queue<Complete> {
            &mut self.completions
        }
        fn submissions(&mut self) -> &mut Queue<Submit> {
            &mut self.submissions
        }
        fn work_pending(&self, _now: Time) -> bool {
            false
        }
        fn next_deadline(&self) -> Option<Time> {
            None
        }
        fn next_policy_deadline(&self) -> Option<Time> {
            None
        }
        fn is_empty(&self) -> bool {
            true
        }
        fn operations(&self) -> u32 {
            1
        }
    }
    struct StarvedJudge(Option<Starved>);
    impl Referee<DroppedHost> for StarvedJudge {
        fn observe(&mut self, _now: Time, _procs: &[DroppedHost]) {
            let deadline = Deadline::arm("the referee's own answer", Duration::ZERO);
            let starved = match deadline.expired() {
                Expired::Starved(starved) => starved,
                Expired::Failed(figures) => panic!("zero patience needs usable scheduler accounting: {figures}"),
            };
            EXPECTED.set(format!("skein-world: starved: {}", starved.figures())).expect("one observation");
            self.0 = Some(starved);
        }
        fn passed(&self) -> bool {
            true
        }
        fn starved(&mut self) -> Option<Starved> {
            self.0.take()
        }
    }

    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", "real::a_starved_referee_drops_hosts_before_reporting_its_figures", "--nocapture"])
            .env(CHILD, "1")
            .env_remove("NEXTEST")
            .output()
            .expect("run the isolated starved-referee check");
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"), "the child ran its assertions");
        return;
    }

    // Linux publishes a new thread's runtime at a context switch; zero
    // runtime makes scheduler accounting unavailable to Deadline.
    std::thread::sleep(core::time::Duration::from_millis(1));
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {
        // Sample before unwinding could drop the world's hosts.
        DROPPED_AT_FAILURE.store(DROPPED.load(Ordering::SeqCst), Ordering::SeqCst);
    }));
    let failure = std::panic::catch_unwind(|| {
        let mut world = real::World::new(StarvedJudge(None));
        world.spawn(|| DroppedHost { completions: Queue::uncapped(), submissions: Queue::uncapped() });
        world.run(&Clock::new(), Duration::from_secs(2))
    });
    std::panic::set_hook(hook);
    let failure = failure.expect_err("the referee's starved verdict fails outside nextest");
    let text = failure.downcast_ref::<String>().expect("the referee's figures");
    assert_eq!(Some(text), EXPECTED.get(), "the referee's observed figures: {text}");
    assert!(DROPPED_AT_FAILURE.load(Ordering::SeqCst), "the host was dropped before failure reporting");
}
