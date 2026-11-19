#[test]
fn every_operation_of_a_restart_after_a_torn_decision_can_be_killed_again() {
    crate::faults::restart_operation_sweep();
}
