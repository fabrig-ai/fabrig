//! Close-point diagnostics consume the deadline's verdict, without host starvation.
use super::{
    CLOSE_WINDOW, DEFAULT_HORIZON, DeadlineDecision, IMAGE, Outcome, SNAPSHOT, StepFailure, bounds,
    closing::{Capture, CloseWindow},
    deadline_outcome, parse_pidfd_pid,
};
use skein_lib::{Duration, Time};
use skein_world::Figures;

#[test]
fn the_horizon_reserves_the_same_close_window() {
    // ui/web/testing.md, 2: the default, a person's ten-minute run, and the close window's bound.
    for (horizon, close, deadline) in
        [(DEFAULT_HORIZON, 36, 40), (Duration::from_secs(600), 596, 600), (Duration::from_secs(4), 0, 4)]
    {
        assert_eq!(bounds(horizon), (Duration::from_secs(close), Duration::from_secs(deadline)), "{horizon:?}");
    }
}

fn figures() -> Figures {
    Figures {
        what: "the browser harness run",
        patience: Duration::from_secs(36),
        elapsed: Duration::from_secs(36),
        thread_waited: Some(Duration::from_secs(18)),
        thread_cpu: Some(Duration::from_secs(1)),
        tree_waited: None,
    }
}

#[test]
fn a_starved_close_point_does_not_make_a_failure_report() {
    assert_eq!(
        deadline_outcome(None, 2, 3, "step expiry", &figures(), true),
        DeadlineDecision { outcome: Outcome::Done, close_starved: true },
    );
}

#[test]
fn a_failed_close_point_keeps_the_report_and_appends_the_figures() {
    let figures = Figures { thread_waited: None, thread_cpu: None, ..figures() };
    let Outcome::Failed { person, step, report } =
        deadline_outcome(Some(Outcome::Done), 2, 3, "browser harness deadline passed", &figures, false).outcome
    else {
        panic!("a failed close point fails")
    };
    assert_eq!((person, step), (2, 3));
    assert_eq!(report, format!("browser harness deadline passed\n{figures}"));
    assert!(report.contains("thread runnable wait UNAVAILABLE"));
}

#[test]
fn a_known_failure_survives_either_deadline_verdict() {
    for starved in [true, false] {
        let failed = Outcome::Failed { person: 2, step: 3, report: "wrong answer".into() };
        assert_eq!(
            deadline_outcome(Some(failed), 0, 0, "browser harness deadline passed", &figures(), starved),
            DeadlineDecision {
                outcome: Outcome::Failed { person: 2, step: 3, report: "wrong answer".into() },
                close_starved: false,
            },
        );
    }
}

#[test]
fn an_immediate_failure_does_not_read_scheduler_accounting() {
    let failure = StepFailure { step: 3, why: "wrong answer".into(), bound: None };
    assert!(failure.classify(|_| panic!("an immediate failure was classified")).is_none());
    let failure = StepFailure { bound: Some(Duration::from_secs(1)), ..failure };
    let deadline = failure.classify(|bound| skein_world::Expired::Failed(Figures { patience: bound, ..figures() }));
    let Some(skein_world::Expired::Failed(figures)) = deadline else { panic!("a bound is classified") };
    assert_eq!(figures.patience, Duration::from_secs(1));
}

#[test]
fn missing_captures_and_unfinished_closes_have_failure_local_cutoffs() {
    let start = Time::from_nanos(42);
    let cutoff = start.saturating_add(Duration::from_secs(5));
    let before = Time::from_nanos(cutoff.as_nanos() - 1);
    for (arrived, missing) in [(SNAPSHOT, IMAGE), (IMAGE, SNAPSHOT)] {
        let mut capture = Capture::new(start);
        assert!(capture.answer(arrived, before));
        assert!(!capture.answer(arrived, before), "one capture cannot complete twice");
        assert!(!capture.done(before));
        assert!(capture.done(cutoff));
        assert!(!capture.answer(missing, cutoff), "the other capture stays unavailable");
    }
    let mut capture = Capture::new(start);
    assert!(capture.answer(SNAPSHOT, before));
    assert!(capture.answer(IMAGE, before));
    assert!(capture.done(before), "capture need not spend its entire window");

    let drain = Duration::from_secs(2);
    let closing = CloseWindow::new(cutoff, drain);
    let force = cutoff.saturating_add(Duration::from_secs(2));
    let end = cutoff.saturating_add(CLOSE_WINDOW);
    assert!(!closing.force(Time::from_nanos(force.as_nanos() - 1)));
    assert!(closing.force(force), "force with room for io to drain");
    assert!(!closing.done(Time::from_nanos(end.as_nanos() - 1)));
    assert!(closing.done(end), "return without waiting for the horizon");
}

#[test]
fn pidfd_accounting_uses_only_a_positive_pid_in_our_namespace() {
    assert_eq!(parse_pidfd_pid("pos:\t0\nPid:\t123\nNSpid:\t123\n"), Some(123));
    for source in ["", "Pid:\t-1\n", "Pid:\t0\n", "Pid:\tbad\n", "NSpid:\t123\n"] {
        assert_eq!(parse_pidfd_pid(source), None, "{source}");
    }
}
