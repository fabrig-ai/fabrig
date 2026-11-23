//! The real git/forge seam of the red-main repair (testing/reference-case.md).
use temper_real_loop_world::{Exit, case, run_cluster_forge};

#[test]
fn a_red_tip_is_repaired_before_a_waiting_change_updates_and_lands() {
    // Open the person's existing a4 branch directly: no director, A1 relay
    // or interim timer. The held first review lets the person push red main;
    // the repair coder waits only for the pending-roots read. Exact-head
    // freshness reviews a4 again after real git updates it onto repaired main.
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_6_4(),
        case::scripts::step_6_4,
        case::scripts::step_6_4_rules,
        case::acts::step_6_4,
    );
    for (at, called) in ran.calls().iter().enumerate() {
        assert_eq!(
            called.exit,
            Some(Exit::Code(0)),
            "call {at}: {}\n{}",
            String::from_utf8_lossy(&called.stderr),
            String::from_utf8_lossy(&called.stdout)
        );
    }
    let observed = case::oracle::Observed::from_ran(&ran, case::acts::READS_6_4);
    case::oracle::assert_met(&case::oracle::step_6_4(&observed));
}
