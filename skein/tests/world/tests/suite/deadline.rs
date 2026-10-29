//! Real-clock harness deadlines (skein/docs/design/testing.md, 5).

use skein_lib::Duration;
use skein_shell::Clock;
use skein_world::{Deadline, Expired};

fn failed(deadline: &Deadline) -> skein_world::Figures {
    match deadline.expired() {
        Expired::Failed(figures) => figures,
        Expired::Starved(starved) => starved.pass(),
    }
}

#[test]
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
