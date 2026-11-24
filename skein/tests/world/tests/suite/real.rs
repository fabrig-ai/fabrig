//! The hosted scripts use real pipes and the same factories as simulated worlds.

use skein_io::kernel::{Exit, Signal, Spawn};
use skein_lib::Duration;
use skein_shell::{Clock, open_root};
use skein_world::{HostedProgram, Inherited, real};
use skein_world_tests::hosted::{Act, Judge, Script};

fn exchange(_spawn: &Spawn, inherited: &Inherited) -> Script {
    Script::child(inherited, &[Act::Read(0), Act::Write(1, b"reply")])
}

#[test]
fn hosted_parent_and_child_exchange_over_real_pipes() {
    let mut world = real::World::new(Judge);
    world.host(HostedProgram { program: Box::from(&b"hosted"[..]), make: exchange, instances: 1, operations: 8 });
    let root = open_root(std::path::Path::new(".")).expect("startup root");
    world.spawn_with_fds(vec![root], || {
        Script::parent(root, &[Act::Spawn, Act::Write(0, b"hello"), Act::Read(1), Act::Read(1), Act::Wait])
    });
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    assert_eq!(outcome.procs[0].received, b"reply");
    assert_eq!(outcome.procs[1].received, b"hello");
    assert_eq!(outcome.procs[0].child_exit, Some(Exit::Code(0)));
    assert!(outcome.killed.is_empty());
}

struct PlainJudge;

impl skein_world::Referee<skein_world::end_to_end::Binary> for PlainJudge {
    fn watch(&mut self, _now: skein_lib::Time, _procs: &[skein_world::end_to_end::Binary]) -> Result<(), String> {
        Ok(())
    }
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
    sent: bool,
}

impl skein_world::Referee<Script> for SignalJudge {
    fn act(&mut self, _now: skein_lib::Time, _procs: &mut [Script]) {
        if self.sent {
            return;
        }
        self.control.signal(0, skein_io::kernel::ServiceSignal::Terminate);
        self.control.signal(1, skein_io::kernel::ServiceSignal::Interrupt);
        self.sent = true;
    }
    fn watch(&mut self, _now: skein_lib::Time, _procs: &[Script]) -> Result<(), String> {
        Ok(())
    }
    fn passed(&self) -> bool {
        self.sent
    }
}

#[test]
fn referee_real_signal_reaches_only_the_signalfd_reader_and_pipe_signal_reaches_its_host() {
    let mut world = real::World::new_controlled(|control| SignalJudge { control, sent: false });
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
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads the child marker and TIERS_ONE_PROCESS for the isolated deadline test"
)]
fn a_zero_patience_deadline_reports_the_failing_iteration() {
    // Zero patience meets the runnable-wait threshold. Exercise its failure
    // diagnostic outside the runners so a starved pass cannot skip it.
    const CHILD: &str = "SKEIN_ZERO_PATIENCE_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "real::a_zero_patience_deadline_reports_the_failing_iteration",
                "--include-ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD, "1")
            .env_remove("NEXTEST")
            .env_remove("TIERS_ONE_PROCESS")
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
    std::thread::sleep(core::time::Duration::from_millis(1));
    let mut world = real::World::new(Judge);
    world.spawn(|| Script::new(&[]));
    let text = world
        .try_run(&Clock::new(), Duration::ZERO)
        .expect_err("zero patience fails before the first script iteration");
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
#[expect(clippy::disallowed_methods, reason = "skein's round: reads the isolated starved real-loop child marker")]
fn a_zero_patience_starved_real_loop_ends_its_process() {
    // Both zero-patience checks select only themselves with --exact.
    const CHILD: &str = "SKEIN_ZERO_PATIENCE_CHILD";
    if std::env::var_os(CHILD).is_some() {
        std::thread::sleep(core::time::Duration::from_millis(1));
        let mut world = real::World::new(Judge);
        world.spawn(|| Script::new(&[]));
        let _outcome = world.run(&Clock::new(), Duration::ZERO);
        panic!("the starved real loop must end its process");
    }
    let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args([
            "--exact",
            "real::a_zero_patience_starved_real_loop_ends_its_process",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, "1")
        .env("TIERS_ONE_PROCESS", "1")
        .env_remove("NEXTEST")
        .output()
        .expect("run the isolated starved real-loop check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}{stderr}");
    assert!(
        stderr.lines().last().is_some_and(
            |line| line.starts_with("skein-world: starved: waited ") && line.ends_with(", by the real loop")
        ),
        "{stderr}"
    );
    assert!(!stderr.contains("panicked at"), "{stderr}");
    assert!(!stdout.contains("test result:"), "{stdout}");
}

#[test]
#[cfg(target_os = "linux")]
#[expect(clippy::too_many_lines, reason = "the child checks host drop order for both failure and starved exit")]
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads the child marker and TIERS_ONE_PROCESS for the isolated referee teardown test"
)]
fn a_starved_referee_drops_hosts_before_reporting_its_figures() {
    use skein_io::kernel::{Complete, Submit};
    use skein_lib::{Queue, Time, Wall};
    use skein_world::{Deadline, Expired, Host, Referee, Starved};

    const CHILD: &str = "SKEIN_STARVED_REFEREE_CHILD";

    #[derive(Debug)]
    struct DroppedHost {
        completions: Queue<Complete>,
        submissions: Queue<Submit>,
    }
    impl Drop for DroppedHost {
        fn drop(&mut self) {
            eprintln!("the starved referee's host was dropped");
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
        fn next_mechanism_deadline(&self) -> Option<Time> {
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
        fn watch(&mut self, _now: Time, _procs: &[DroppedHost]) -> Result<(), String> {
            let deadline = Deadline::arm("the referee's own answer", Duration::ZERO);
            let starved = match deadline.expired() {
                Expired::Starved(starved) => starved,
                Expired::Failed(figures) => {
                    return Err(format!("zero patience needs usable scheduler accounting: {figures}"));
                }
            };
            if !skein_world::starved_passes() {
                eprintln!("observed figures: skein-world: starved: {}", starved.figures());
            }
            self.0 = Some(starved);
            Ok(())
        }
        fn passed(&self) -> bool {
            true
        }
        fn starved(&mut self) -> Option<Starved> {
            self.0.take()
        }
    }

    if std::env::var_os(CHILD).is_none() {
        for one_process in [false, true] {
            let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
            command
                .args([
                    "--exact",
                    "real::a_starved_referee_drops_hosts_before_reporting_its_figures",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CHILD, "1")
                .env_remove("NEXTEST")
                .env_remove("TIERS_ONE_PROCESS");
            if one_process {
                command.env("TIERS_ONE_PROCESS", "1");
            }
            let output = command.output().expect("run the isolated starved-referee check");
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            if one_process {
                assert_eq!(output.status.code(), Some(0), "{stdout}{stderr}");
                assert!(!stdout.contains("test result:"), "{stdout}");
                assert!(!stderr.contains("panicked at"), "{stderr}");
                let mut lines = stderr.lines();
                assert_eq!(lines.next(), Some("the starved referee's host was dropped"), "{stderr}");
                let line = lines.next().expect("the starved line follows the host's drop");
                assert!(line.starts_with("skein-world: starved: waited "), "{stderr}");
                assert!(line.ends_with(", by the referee's own answer"), "{stderr}");
                assert_eq!(lines.next(), None, "the starved line is last: {stderr}");
            } else {
                assert!(!output.status.success(), "{stdout}{stderr}");
                let (observed, report) = stderr.split_once('\n').expect("observed figures precede the report");
                let figures = observed.strip_prefix("observed figures: ").expect("the referee observed figures");
                let (dropped, panic) = report.split_once("panicked at").expect("the child failed with a panic report");
                assert!(dropped.contains("the starved referee's host was dropped"), "{stderr}");
                assert!(panic.contains(figures), "the report carries the observed figures: {stderr}");
            }
        }
        return;
    }

    // Linux publishes a new thread's runtime at a context switch; zero
    // runtime makes scheduler accounting unavailable to Deadline.
    std::thread::sleep(core::time::Duration::from_millis(1));
    let mut world = real::World::new(StarvedJudge(None));
    world.spawn(|| DroppedHost { completions: Queue::uncapped(), submissions: Queue::uncapped() });
    let _outcome = world.run(&Clock::new(), Duration::from_secs(2));
    panic!("the starved referee must end its process");
}
