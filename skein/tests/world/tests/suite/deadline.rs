//! Real-clock harness deadlines (skein/docs/design/testing/README.md, 5).

use skein_lib::Duration;
use skein_shell::Clock;
use skein_world::{Deadline, Expired};

#[test]
#[cfg(target_os = "linux")]
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads the child marker for the isolated starvation verdict test"
)]
fn a_starved_pass_ends_its_process() {
    use skein_world::pass_starved;

    const FOLLOWING: &str = "deadline::remaining_reaches_zero_and_until_uses_the_shells_clock";
    const CHILD: &str = "SKEIN_STARVED_PASS_CHILD";
    const FORWARDED: &str = "skein-world: starved: the helper's unanswered wait";
    if let Some(end) = std::env::var_os(CHILD) {
        // A context switch publishes the new thread's scheduler accounting.
        std::thread::sleep(core::time::Duration::from_millis(1));
        let deadline = Deadline::arm("the zero-patience answer", Duration::ZERO);
        let starved = match deadline.expired() {
            Expired::Starved(starved) => starved,
            Expired::Failed(figures) => panic!("zero patience needs scheduler accounting: {figures}"),
        };
        match end.to_str().expect("child end") {
            "pass" => starved.pass(),
            "exit" => starved.exit(),
            "forward" => pass_starved(FORWARDED),
            other => panic!("unknown child end: {other}"),
        }
    }

    let failures: Vec<_> = [
        ("pass", Some("1"), None, false),
        ("pass", Some(""), None, false),
        ("pass", None, Some("1"), false),
        ("pass", None, Some(""), false),
        ("pass", Some(""), Some(""), false),
        ("pass", None, None, false),
        ("exit", Some("1"), None, false),
        ("exit", Some(""), None, false),
        ("exit", None, Some("1"), false),
        ("exit", None, Some(""), false),
        ("exit", None, None, false),
        ("forward", Some("1"), None, false),
        ("forward", Some(""), None, false),
        ("forward", None, Some("1"), false),
        ("forward", None, Some(""), false),
        ("forward", None, None, false),
        ("pass", Some("1"), None, true),
    ]
    .into_iter()
    .filter_map(|(end, one_process, nextest, following)| {
        let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
        command
            .args(["--exact", "deadline::a_starved_pass_ends_its_process", "--nocapture", "--test-threads=1"])
            .env(CHILD, end)
            .env_remove("TIERS_ONE_PROCESS")
            .env_remove("NEXTEST");
        if following {
            command.arg(FOLLOWING);
        }
        if let Some(value) = one_process {
            command.env("TIERS_ONE_PROCESS", value);
        }
        if let Some(value) = nextest {
            command.env("NEXTEST", value);
        }
        let output = command.output().expect("run the isolated starved-pass check");
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let finding = if one_process.is_some() || nextest.is_some() {
            let last = stderr.lines().last().unwrap_or_default();
            let line_matches = if end == "forward" {
                last == FORWARDED
            } else {
                last.starts_with("skein-world: starved: waited ") && last.ends_with(", by the zero-patience answer")
            };
            if output.status.code() != Some(0) {
                Some("the starved child did not exit 0")
            } else if !line_matches || stderr.contains("panicked at") {
                Some("the starved line must be last, without a panic")
            } else if stdout.contains("test result:") || stdout.contains(FOLLOWING) {
                Some("libtest continued after the starved pass")
            } else {
                None
            }
        } else if output.status.success() {
            Some("outside the runners the child must fail")
        } else if end == "forward" {
            (!stderr.contains(FORWARDED)).then_some("the failure lost the forwarded line")
        } else {
            (!(stderr.contains("skein-world: starved: the zero-patience answer: patience 0 ns; elapsed ")
                && stderr.contains("thread runnable wait ") && stderr.contains("thread CPU ")))
                .then_some("the failure lost its figures")
        };
        finding.map(|why| format!("{end}, one-process={one_process:?}, nextest={nextest:?}, following={following}: {why}: {stdout}{stderr}"))
    })
    .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn failed(deadline: &Deadline) -> skein_world::Figures {
    match deadline.expired() {
        Expired::Failed(figures) => figures,
        Expired::Starved(starved) => starved.pass(),
    }
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn an_idle_unanswered_deadline_fails_with_its_figures() {
    let deadline = Deadline::arm("the idle answer", Duration::from_millis(20));
    std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
    let figures = failed(&deadline);
    assert_eq!(figures.what, "the idle answer");
    assert_eq!(figures.patience, Duration::from_millis(20));
    assert!(figures.elapsed >= figures.patience);
    assert_eq!(figures.tree_waited, None);
    let failure =
        std::panic::catch_unwind(|| panic!("no answer: {figures}")).expect_err("an unanswered idle wait fails");
    let text = failure.downcast_ref::<String>().expect("failure includes its figures");
    assert!(text.starts_with("no answer: the idle answer: patience 20000000 ns; elapsed "), "{text}");
    assert!(text.contains("thread runnable wait ") && text.contains("thread CPU "), "{text}");
    assert!(text.ends_with("tree runnable wait UNAVAILABLE (ns)"), "{text}");
}

#[test]
fn remaining_reaches_zero_and_until_uses_the_shells_clock() {
    let clock = Clock::new();
    let before = clock.now().now;
    let patience = Duration::from_millis(20);
    let deadline = Deadline::arm("the timed wait", patience);
    let after = clock.now().now;
    let until = deadline.until();
    assert!(until >= before.saturating_add(patience));
    assert!(until <= after.saturating_add(patience));
    let remaining = deadline.remaining();
    assert!(remaining <= patience);
    std::thread::sleep(core::time::Duration::from_nanos(remaining.as_nanos()));
    assert!(deadline.passed());
    assert_eq!(deadline.remaining(), Duration::ZERO);
    assert_eq!(deadline.until(), until);
    assert!(clock.now().now >= until);
}

#[test]
fn watching_our_process_measures_its_threads_when_accounting_is_available() {
    let mut deadline = Deadline::arm("our helper threads", Duration::from_millis(20));
    deadline.watch_process(std::process::id());
    std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
    let figures = failed(&deadline);
    if let Some(wait) = figures.tree_waited {
        assert!(wait.as_nanos() < figures.patience.as_nanos().div_ceil(2));
    }
    eprintln!("deadline accounting: own threads: {figures}");
}

#[test]
fn watching_a_binary_keeps_no_borrow_and_cleans_up_before_a_starved_pass() {
    use skein_world::end_to_end::{Binary, Command, Containment, Mode};
    for containment in [Containment::Best, Containment::Group] {
        let command = Command {
            program: "/bin/sh".into(),
            arguments: vec!["-c".into(), "read answer".into()],
            environment: vec![],
            directory: "/tmp".into(),
        };
        let binary = Binary::start_contained(
            command,
            Mode::Pipes,
            1024,
            skein_world::end_to_end::Expectation::Measure,
            containment,
        )
        .expect("start an owned waiting binary");
        let mut deadline = Deadline::arm("the binary's answer", Duration::from_millis(20));
        deadline.watch(&binary);
        std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
        let expired = deadline.expired();
        drop(binary);
        match expired {
            Expired::Failed(figures) => {
                assert_eq!(figures.what, "the binary's answer");
                eprintln!("deadline accounting: {containment:?}: {figures}");
            }
            Expired::Starved(starved) => starved.pass(),
        }
    }
}
