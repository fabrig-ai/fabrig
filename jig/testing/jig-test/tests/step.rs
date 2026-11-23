//! Offline checks of sign-in and shutdown diagnostics.
use jig_test_kit::diagnostics::{STDERR_TAIL, premature_exit};
use skein_io::kernel::Exit;

#[test]
fn a_premature_exit_names_its_status_milestone_and_safe_stderr_tail() {
    assert_eq!(
        premature_exit(
            Exit::Code(1),
            "sign-in asked",
            b"jig-test: stopped: Listener: an io event no component owns\n\xff"
        ),
        "binary exited before SIGTERM: status=Code(1); last milestone=sign-in asked; stderr tail (up to 4096 bytes):\njig-test: stopped: Listener: an io event no component owns\n\u{fffd}"
    );
    assert_eq!(
        premature_exit(
            Exit::Signal(9),
            "client live",
            b"link=\"https://issuer.example/authorize?secret=private\"\nlast error"
        ),
        "binary exited before SIGTERM: status=Signal(9); last milestone=client live; stderr tail (up to 4096 bytes):\nlink=\"[address omitted]\"\nlast error"
    );
    let mut stderr = vec![b'x'; STDERR_TAIL];
    stderr.extend_from_slice(b"last error");
    let finding = premature_exit(Exit::Code(2), "binary started", &stderr);
    assert_eq!(
        finding.split_once('\n').expect("stderr follows heading").1,
        format!("{}last error", "x".repeat(STDERR_TAIL - 10))
    );
    let mut stderr = b"https://issuer.example/authorize?state=".to_vec();
    stderr.extend(vec![b's'; STDERR_TAIL]);
    stderr.extend_from_slice(b"\nlast error");
    assert_eq!(
        premature_exit(Exit::Code(1), "AccountSignInStarted seen", &stderr),
        "binary exited before SIGTERM: status=Code(1); last milestone=AccountSignInStarted seen; stderr tail (up to 4096 bytes):\n[address omitted]\nlast error"
    );
}

mod live_sign_in {
    use jig_client::contract as c;
    use jig_test_kit::live_sign_in::{failed, run_line};

    mod tests {
        use super::*;

        /// The diagnostic checked by the refused-plan process story is formatted without a process.
        #[test]
        fn a_refused_plan_is_reported_with_the_run_failure() {
            let mut task = jig_client_protocol_link::translate::documents::task(
                jig_client_test_records::task_with_hold_record(),
                &jig_client_protocol_kit::fixtures::link_limits(),
            )
            .expect("owner task fixture");
            let c::ChipPhaseFacts { phase, details } = task.chip.details.as_mut();
            *phase = c::ChipPhase::Active(c::RunStage::BackingOff { until: skein_lib::Wall::from_nanos(17) });
            details.attempt = Some(1);
            details.retry =
                Some(c::RetryNotice::OwnerFailed { class: c::HoldFailure::Run, count: 1, allowance: 3, reason: None });
            details.hold = None;
            details.ending = None;
            details.observation = None;
            task.run_wait = None;
            assert!(failed(&task));
            assert_eq!(
                run_line(&task),
                "phase=Active(BackingOff { until: Wall(17) }); attempt=Some(1); retry=Some(OwnerFailed { class: Run, count: 1, allowance: 3, reason: None }); work=None; hold=None; ending=none; wait=None"
            );
        }
    }
}
