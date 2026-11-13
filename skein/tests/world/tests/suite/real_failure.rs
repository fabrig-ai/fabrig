//! Failure cleanup precedes the panic hook (skein/docs/design/testing/README.md, 5).

use skein_io::kernel::Fd;
use skein_lib::{Duration, Time};
use skein_shell::Clock;
use skein_world::end_to_end::{Binary, Command, Containment, Expectation, Mode};
use skein_world::{Referee, real};

const MARKER: &str = "every child was reaped before reporting failure";

struct Judge {
    until: Option<Time>,
    terminal: bool,
}

impl Referee<Binary> for Judge {
    fn observe(&mut self, _now: Time, _procs: &[Binary]) {}
    fn passed(&self) -> bool {
        false
    }
    fn next_deadline(&self) -> Option<Time> {
        self.until
    }
    fn overdue(&self, now: Time) -> Option<String> {
        (self.terminal || self.until.is_some_and(|until| until <= now)).then(|| "the child never answered".into())
    }
}

fn children() -> String {
    std::fs::read_to_string("/proc/thread-self/children").expect("this thread's children")
}

fn binary(containment: Containment) -> Binary {
    Binary::start_contained(
        Command {
            program: skein_scratch::paths::binary("world_process_fixture"),
            arguments: vec!["never".into()],
            environment: vec![],
            directory: ".".into(),
        },
        Mode::Pipes,
        1024,
        Expectation::Measure,
        containment,
    )
    .expect("a live owned child")
}

fn failure_hook(pidfd: Fd, expected: &'static str, panic: &std::panic::PanicHookInfo<'_>) -> ! {
    let remaining = children();
    let message = panic.to_string();
    // The hook runs before any unwind, including in panic=abort builds. On a
    // regression, end the fixture here after recording the failed observation.
    if !remaining.trim().is_empty() || !message.contains(expected) {
        skein_shell::keeper::abandon_binary(Some(pidfd), &[]);
        eprintln!("children before failure: {remaining}; {message}");
        std::process::exit(1);
    }
    // A subsequent scenario in this same process observes the same clean
    // entrance as the next test in a one-process binary.
    let next = binary(Containment::Group);
    drop(next);
    if !children().trim().is_empty() {
        std::process::exit(2);
    }
    eprintln!("{MARKER}");
    std::process::exit(0);
}

fn failure(case: &str) {
    let containment = if case.starts_with("walk-") { Containment::Walk } else { Containment::Group };
    let binary = binary(containment);
    let pidfd = *binary.descriptors().first().expect("leader descriptor");
    assert_eq!(skein_shell::keeper::poll_child(pidfd, false), Ok(None), "child is alive before the failure");
    let clock = Clock::new();
    let now = clock.now().now;
    let referee = case.ends_with("referee");
    let terminal = case.ends_with("assertion") || case.ends_with("future");
    let until = (referee || case.ends_with("future")).then(|| now.saturating_add(Duration::from_millis(20)));
    let expected = if referee || terminal { "the referee failed the real loop" } else { "the real loop settles" };
    let mut world = real::World::new(Judge { until, terminal });
    world.spawn_with_fds(binary.descriptors(), || binary);
    std::panic::set_hook(Box::new(move |panic| failure_hook(pidfd, expected, panic)));
    let patience = if case.ends_with("immediate") { Duration::ZERO } else { Duration::from_millis(50) };
    let _outcome = world.run(&clock, patience);
    panic!("the world must fail");
}

// Each entry runs only in the isolated process selected by the parent test.
mod child {
    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn walk_immediate() {
        super::failure("walk-immediate");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn walk_pending() {
        super::failure("walk-pending");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn walk_referee() {
        super::failure("walk-referee");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn walk_assertion() {
        super::failure("walk-assertion");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn walk_future() {
        super::failure("walk-future");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn group_immediate() {
        super::failure("group-immediate");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn group_pending() {
        super::failure("group-pending");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn group_referee() {
        super::failure("group-referee");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn group_assertion() {
        super::failure("group-assertion");
    }

    #[test]
    #[ignore = "selected by the isolated failure-cleanup regression"]
    fn group_future() {
        super::failure("group-future");
    }
}

#[test]
fn failures_reap_children_before_reporting_and_leave_the_next_world_clean() {
    for containment in ["walk", "group"] {
        for failure in ["immediate", "pending", "referee", "assertion", "future"] {
            let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
            if cfg!(panic = "abort") {
                // The isolated hook exits itself; libtest's abort-mode child
                // protocol would otherwise require a normal test return.
                command.args(["-Z", "unstable-options", "--force-run-in-process"]);
            }
            let output = command
                .args(["--exact", "--ignored", "--nocapture", "--test-threads=1"])
                .arg(format!("real_failure::child::{containment}_{failure}"))
                .env_remove("NEXTEST")
                .env_remove("TIERS_ONE_PROCESS")
                .output()
                .expect("run the failure in isolation");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(output.status.success(), "{containment}-{failure}: {stderr}");
            assert!(stderr.contains(MARKER), "the failure hook checked cleanup: {stderr}");
        }
    }
    eprintln!("10 pre-panic cleanup cases passed; panic={}", if cfg!(panic = "abort") { "abort" } else { "unwind" });
}
