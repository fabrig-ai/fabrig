//! Retained first-edition story (testing/reference-case.md): Stop holds B and its delegates; one release resumes their work.
use temper_real_loop_world::{
    Exit,
    case::{
        self,
        oracle::{self, Observed},
    },
    run_cluster_forge,
};

#[test]
fn a_stopped_owners_reviewed_change_lands_and_its_news_arrives_after_release() {
    // A held review is the in-flight seam; the person's one release resumes
    // the whole subtree before the change lands over the loopback forge.
    let ran = run_cluster_forge(
        case::PAUSE_WORKERS,
        &[],
        case::fixture_5_2(),
        case::scripts::step_5_2,
        case::scripts::step_5_2_rules,
        case::acts::step_5_2,
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
    let observed = Observed::from_ran(&ran, case::acts::READS_5_2);
    oracle::assert_met(&oracle::step_5_2(&observed));
}
