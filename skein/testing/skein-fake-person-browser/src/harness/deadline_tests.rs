//! Close-point diagnostics consume the deadline's verdict, without host starvation.
use super::{Outcome, deadline_outcome, parse_pidfd_pid};
use skein_lib::Duration;
use skein_world::Figures;

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
