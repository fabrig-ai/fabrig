//! Retained first-edition story (testing/reference-case.md): two landings, with a cross-repository sibling dependency.
use temper_real_loop_world::{Exit, case, run_cluster_forge};

#[test]
fn kit_merges_before_apps_first_push_and_both_land_green_with_the_pin() {
    // Real git in both repositories: app's write waits for kit's merge.
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_6_6(),
        case::scripts::step_6_6,
        case::scripts::step_6_6_rules,
        case::acts::step_6_6,
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
    let observed = case::oracle::Observed::from_ran(&ran, case::acts::READS_6_6);
    case::oracle::assert_met(&case::oracle::step_6_6(&observed));
}
