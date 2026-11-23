use jig_conformance::referee::Promise;
use jig_conformance_world::negative::{Fault, world};

#[test]
fn application_referee_catches_early_answers_and_redecided_replays() {
    use jig_conformance::referee as r;
    let answer = r::ApplicationAnswer {
        key: r::ApplicationKey { person: 7, key: [4; 16] },
        connector: 2,
        digest: [9; 32],
        token: 5,
        until: 1000,
        outcome: Box::from([1, 8]),
    };
    let request =
        || r::Observed::ApplicationRequested { reply: 600, key: answer.key, connector: 2, digest: answer.digest };
    let reply = || r::Observed::ApplicationAnswered { reply: 600, connector: 2, outcome: answer.outcome.clone() };
    let early = &mut world(Fault::None).referee;
    early.observe(1, request()).expect("offered application");
    early
        .observe(2, r::Observed::ApplicationSubmitted { commit: 1, answer: answer.clone() })
        .expect("store submission");
    assert_eq!(early.observe(3, reply()).expect_err("the root answered before durability").promise, Promise::Commit);

    for token in [5, 6] {
        let repeated = &mut world(Fault::None).referee;
        repeated.observe(1, request()).expect("offered application");
        repeated
            .observe(2, r::Observed::ApplicationSubmitted { commit: 1, answer: answer.clone() })
            .expect("store submission");
        repeated
            .observe(3, r::Observed::Durable { number: 1, snapshot: r::Snapshot::default() })
            .expect("whole commit");
        repeated.observe(4, reply()).expect("durable terminal");
        repeated.observe(5, r::Observed::Restart).expect("outside ledger survives restart");
        repeated.observe(6, request()).expect("same key offered again");
        repeated
            .observe(
                7,
                r::Observed::ApplicationSubmitted {
                    commit: 2,
                    answer: r::ApplicationAnswer { token, ..answer.clone() },
                },
            )
            .expect("faulty root routed replay to decide");
        assert_eq!(
            repeated
                .observe(8, r::Observed::Durable { number: 2, snapshot: r::Snapshot::default() })
                .expect_err("a retained key cannot be decided twice, even under a fresh token")
                .promise,
            Promise::Once
        );
    }
}

#[test]
fn application_referee_accepts_failed_commit_retry_expiry_and_identical_replay() {
    use jig_conformance::referee as r;
    let answer = r::ApplicationAnswer {
        key: r::ApplicationKey { person: 7, key: [4; 16] },
        connector: 2,
        digest: [9; 32],
        token: 5,
        until: 10,
        outcome: Box::from([1, 8]),
    };
    let request =
        || r::Observed::ApplicationRequested { reply: 600, key: answer.key, connector: 2, digest: answer.digest };
    let reply = || r::Observed::ApplicationAnswered { reply: 600, connector: 2, outcome: answer.outcome.clone() };
    let referee = &mut world(Fault::None).referee;
    referee.observe(1, request()).expect("offered application");
    referee
        .observe(2, r::Observed::ApplicationSubmitted { commit: 1, answer: answer.clone() })
        .expect("undurable submission");
    referee.observe(3, r::Observed::Restart).expect("failed submission lost");
    referee.observe(4, request()).expect("retry after failed commit");
    referee
        .observe(4, r::Observed::ApplicationSubmitted { commit: 1, answer: answer.clone() })
        .expect("fresh decision");
    referee.observe(5, r::Observed::Durable { number: 1, snapshot: r::Snapshot::default() }).expect("durable retry");
    referee.observe(6, reply()).expect("durable terminal");
    referee.observe(7, request()).expect("retained replay");
    referee.observe(8, reply()).expect("identical replay needs no new decision");
    referee.observe(10, request()).expect("expired key can be used again");
    referee
        .observe(
            10,
            r::Observed::ApplicationSubmitted {
                commit: 2,
                answer: r::ApplicationAnswer { token: 6, until: 20, ..answer },
            },
        )
        .expect("new retention");
    referee
        .observe(10, r::Observed::Durable { number: 2, snapshot: r::Snapshot::default() })
        .expect("new durable decision");
    referee
        .observe(11, r::Observed::ApplicationAnswered { reply: 600, connector: 2, outcome: Box::from([1, 8]) })
        .expect("new terminal");
}

#[test]
fn a_commit_after_drained_is_a_violation() {
    use jig_conformance::referee as r;
    let mut referee = jig_core_kit::conformance::Observer::new(&jig_test_profile::config(31)).into_referee();
    referee
        .observe(1, r::Observed::Durable { number: 1, snapshot: r::Snapshot::default() })
        .expect("durable evidence accepted");
    referee.observe(2, r::Observed::Drained { at: 1 }).expect("durable evidence accepted");
    let failure = referee
        .observe(3, r::Observed::Durable { number: 2, snapshot: r::Snapshot::default() })
        .expect_err("drain invariant must reject this observation");
    assert_eq!(failure.promise, Promise::Commit);
    assert_eq!(failure.why, "CommitAfterDrained { number: 2 }");
}

#[test]
fn a_drain_before_its_commit_is_durable_is_a_violation() {
    use jig_conformance::referee as r;
    let mut referee = jig_core_kit::conformance::Observer::new(&jig_test_profile::config(31)).into_referee();
    let failure =
        referee.observe(1, r::Observed::Drained { at: 1 }).expect_err("drain invariant must reject this observation");
    assert_eq!(failure.promise, Promise::Commit);
}

#[test]
fn a_cold_store_that_loses_a_drained_party_answer_is_a_violation() {
    use jig_conformance::referee as r;
    let mut referee = jig_core_kit::conformance::Observer::new(&jig_test_profile::config(31)).into_referee();
    let mut snapshot = r::Snapshot::default();
    snapshot.receipts.insert(r::Receipt::Party(7, [2; 16]));
    referee.observe(1, r::Observed::Durable { number: 1, snapshot }).expect("durable evidence accepted");
    referee.observe(2, r::Observed::Drained { at: 1 }).expect("durable evidence accepted");
    referee.observe(3, r::Observed::Restart).expect("durable evidence accepted");
    let failure = referee
        .observe(4, r::Observed::ColdStore { receipts: std::collections::BTreeSet::new() })
        .expect_err("drain invariant must reject this observation");
    assert_eq!(failure.promise, Promise::Commit);
    assert_eq!(failure.why, "cold store lost a drained receipt");
}

fn position_referee() -> jig_conformance::referee::Referee {
    jig_core_kit::conformance::Observer::new(&jig_test_profile::config(31)).into_referee()
}

fn position_snapshot(
    referee: &jig_conformance::referee::Referee,
    phase: jig_conformance::referee::Phase,
) -> jig_conformance::referee::Snapshot {
    use jig_conformance::referee as r;
    let mut snapshot = r::Snapshot::default();
    snapshot.tasks.insert(
        5,
        r::Task {
            settings: r::SettingsRevision::default(),
            spec_revision: 0,
            hold_revision: 0,
            placed: None,
            placed_reason: false,
            tracked: false,
            number: 5,
            project: 1,
            source: r::Source::Deployment,
            scope: referee.policy.projects[&1].clone(),
            executor: (0, 1),
            budget: 0,
            spent: 0,
            spent_below: 0,
            reserved: 0,
            run_reserved: 0,
            run_spent: 0,
            attempt: 1,
            period: None,
            phase,
            dependencies: Vec::new(),
            delegates: Vec::new(),
            pools: Vec::new(),
            inbox: Vec::new(),
            proposal: None,
        },
    );
    snapshot.read_positions.insert((1, 5), 2);
    snapshot.read_positions.insert((2, 5), 7);
    snapshot
}

const POSITION_BOUNDARIES: [bool; 2] = [false, true];

#[test]
fn a_read_position_moving_back_across_an_end_is_a_violation() {
    use jig_conformance::referee as r;
    for restart in POSITION_BOUNDARIES {
        let mut referee = position_referee();
        let before = position_snapshot(&referee, r::Phase::Active);
        let mut after = position_snapshot(&referee, r::Phase::Ended);
        after.read_positions.insert((1, 5), 1);
        after.read_positions.insert((2, 5), 8);
        referee.observe(1, r::Observed::Durable { number: 1, snapshot: before }).expect("live position");
        if restart {
            referee.observe(2, r::Observed::Restart).expect("cold start");
        }
        let failure = referee.observe(3, r::Observed::Durable { number: 2, snapshot: after }).expect_err("regression");
        assert_eq!(failure.promise, Promise::Lost, "restart {restart}");
        assert_eq!(failure.why, "a retained task lost or moved back its party's read position", "restart {restart}");
    }
}

#[test]
fn a_read_position_erased_without_its_task_is_a_violation() {
    use jig_conformance::referee as r;
    let mut referee = position_referee();
    let before = position_snapshot(&referee, r::Phase::Ended);
    let mut after = before.clone();
    after.read_positions.remove(&(1, 5));
    referee.observe(1, r::Observed::Durable { number: 1, snapshot: before }).expect("ended position");
    let failure = referee.observe(2, r::Observed::Durable { number: 2, snapshot: after }).expect_err("lost position");
    assert_eq!(failure.promise, Promise::Lost);
    assert_eq!(failure.why, "a retained task lost or moved back its party's read position");
}

#[test]
fn a_read_position_kept_across_a_restart_passes() {
    use jig_conformance::referee as r;
    let mut referee = position_referee();
    let before = position_snapshot(&referee, r::Phase::Active);
    let after = position_snapshot(&referee, r::Phase::Ended);
    referee.observe(1, r::Observed::Durable { number: 1, snapshot: before }).expect("live position");
    referee.observe(2, r::Observed::Restart).expect("cold start retains seen evidence");
    referee.observe(3, r::Observed::Durable { number: 2, snapshot: after }).expect("same positions after task end");
}

#[test]
fn a_task_and_its_read_positions_can_be_forgotten_together() {
    use jig_conformance::referee as r;
    let mut referee = position_referee();
    let before = position_snapshot(&referee, r::Phase::Ended);
    referee.observe(1, r::Observed::Durable { number: 1, snapshot: before }).expect("ended positions");
    referee.observe(2, r::Observed::Durable { number: 2, snapshot: r::Snapshot::default() }).expect("atomic forget");
}
