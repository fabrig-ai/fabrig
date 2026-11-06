//! Close-point diagnostics consume the deadline's verdict, without host starvation.
use super::{DEFAULT_HORIZON, Outcome, bounds, deadline_outcome, parse_pidfd_pid};
use skein_lib::Duration;
use skein_world::Figures;

#[test]
fn the_horizon_reserves_the_same_close_window() {
    // ui-web.md, 14.2: the default, a person's ten-minute run, and the close window's bound.
    for (horizon, close, deadline) in
        [(DEFAULT_HORIZON, 36, 40), (Duration::from_secs(600), 596, 600), (Duration::from_secs(4), 0, 4)]
    {
        assert_eq!(bounds(horizon), (Duration::from_secs(close), Duration::from_secs(deadline)), "{horizon:?}");
    }
}

#[test]
#[should_panic(expected = "horizon Duration(3999999999) is shorter than the close window")]
fn a_horizon_shorter_than_the_close_window_is_refused() {
    // One nanosecond below io's four-second close window.
    bounds(Duration::from_nanos(3_999_999_999));
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
    assert_eq!(deadline_outcome(None, 0, 0, "browser harness deadline passed", &figures(), true), Outcome::Done);
}

#[test]
fn a_failed_close_point_keeps_the_report_and_appends_the_figures() {
    let figures = Figures { thread_waited: None, thread_cpu: None, ..figures() };
    let Outcome::Failed { person, step, report } =
        deadline_outcome(Some(Outcome::Done), 2, 3, "browser harness deadline passed", &figures, false)
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
            Outcome::Failed { person: 2, step: 3, report: "wrong answer".into() },
        );
    }
}

#[test]
fn pidfd_accounting_uses_only_a_positive_pid_in_our_namespace() {
    assert_eq!(parse_pidfd_pid("pos:\t0\nPid:\t123\nNSpid:\t123\n"), Some(123));
    for source in ["", "Pid:\t-1\n", "Pid:\t0\n", "Pid:\tbad\n", "NSpid:\t123\n"] {
        assert_eq!(parse_pidfd_pid(source), None, "{source}");
    }
}
