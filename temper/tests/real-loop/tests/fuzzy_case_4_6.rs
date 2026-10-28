//! A restarted workstation retains the director's question and answers it once.
use temper_real_loop_world::{
    End, Exit,
    case::{
        self, acts,
        oracle::{self, Observed},
    },
    run_cluster_forge,
};

const WORKERS: &[&str] = &["ws", "s1", "s2"];
const FIRST_ACT: usize = 0;
const WORKSTATION_LIVES: usize = 2;
const ONE: usize = 1;

#[test]
fn a_restarted_workstation_keeps_the_directors_question_and_answers_it_once() {
    // the hang past the grace, a fenced attempt still running, a stalled run's bound, three questions, no single point, the reboot's faults: testing.md 5.3's 4.6 row
    // All workers start together. The first configured free slot is ws; its
    // first-life assignments below check that the chat actually started there.
    let cluster = run_cluster_forge(
        WORKERS,
        &[],
        case::fixture_4_6(),
        case::scripts::step_4_6,
        case::scripts::step_4_6_rules,
        || acts::step_4_6(FIRST_ACT),
    );
    for (at, called) in cluster.calls().iter().enumerate() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "call {at}: {}", String::from_utf8_lossy(&called.stderr));
    }
    let reads: Vec<_> = acts::READS_4_6.iter().map(|&(read, call)| (read, FIRST_ACT + call)).collect();
    let observed = Observed::from_ran(&cluster, &reads);
    oracle::assert_met(&oracle::step_4_6(&observed));
    for name in WORKERS {
        assert_eq!(cluster.worker(name).end(), Some(&End::Drained), "{name} drained");
    }
    let ws = cluster.worker("ws");
    let lives = ws.lives();
    assert_eq!(lives.len(), WORKSTATION_LIVES, "ws kept its root across two lives");
    let [first, second] = lives else { panic!("ws has two lives") };
    assert_eq!(first.ran, &[case::HANG], "the chat's first attempt ran on ws");
    assert!(second.ran.is_empty(), "the restarted ws ran no task");
    assert_eq!(ws.linked(), WORKSTATION_LIVES, "ws linked for each life");
    for life in lives {
        assert!(life.drained, "each ws life drained");
        assert_eq!(life.linked, ONE, "each ws life linked once");
    }
    let other =
        ["s1", "s2"].iter().flat_map(|name| cluster.worker(name).ran()).filter(|task| **task == case::HANG).count();
    assert_eq!(other, ONE, "the director had one activation away from ws");
    assert!(cluster.killed().is_empty(), "cancellation was cooperative");
}
