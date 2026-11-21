//! Offline checks of sign-in plan diagnostics.
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
