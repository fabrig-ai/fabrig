//! Pushed-head cancellation and held overlap news (testing/reference-case.md).
use temper_real_loop_world::{
    Exit,
    case::{
        self,
        oracle::{self, Observed},
    },
    run_cluster_forge,
};

#[test]
fn a_pushed_change_is_cancelled_under_its_closed_pull_and_an_overlapping_landing_wakes_b_after_release() {
    // The real workers push x3 and a1; the loopback forge keeps x3's head under
    // its closed pull and lands a1 on B's file. B hears the overlap after release.
    // Three one-slot workers let x3's review and h1's coding turn wait together.
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_5_4(),
        case::scripts::step_5_4,
        case::scripts::step_5_4_rules,
        case::acts::step_5_4,
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
    let observed = Observed::from_ran(&ran, case::acts::READS_5_4);
    oracle::assert_met(&oracle::step_5_4(&observed));
}
