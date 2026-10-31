//! Project removal is judged from outside answers and durable evidence.
use jig_conformance::referee::{self as r, Observed, Promise, Referee, Snapshot};
use std::collections::BTreeSet;

fn referee() -> Referee {
    jig_core_world::observations::Observer::new(&jig_core_world::world::config(31)).referee
}

fn durable(referee: &mut Referee, number: u64, snapshot: Snapshot) -> Result<(), r::Violation> {
    referee.observe(number, Observed::Durable { number, snapshot })
}

fn task(referee: &Referee, project: u32) -> r::Task {
    r::Task {
        number: 5,
        project,
        source: r::Source::Deployment,
        scope: referee.policy.projects[&project].clone(),
        executor: (0, 1),
        settings: r::SettingsRevision::default(),
        spec_revision: 0,
        hold_revision: 0,
        placed: None,
        placed_reason: false,
        tracked: false,
        budget: 0,
        spent: 0,
        spent_below: 0,
        reserved: 0,
        run_reserved: 0,
        run_spent: 0,
        attempt: 1,
        period: None,
        phase: r::Phase::Active,
        dependencies: Vec::new(),
        delegates: Vec::new(),
        pools: Vec::new(),
        inbox: Vec::new(),
        proposal: None,
    }
}

#[test]
fn a_removal_with_no_row_left_holds() {
    let mut referee = referee();
    referee.observe(0, Observed::ProjectRemoved { project: 3 }).expect("answer");
    durable(&mut referee, 1, Snapshot::default()).expect("erased snapshot");
    referee.check_at(2).expect("quiescent erasure");
}

#[test]
fn a_row_left_after_a_removal_is_a_violation() {
    let mut referee = referee();
    referee.observe(0, Observed::ProjectRemoved { project: 3 }).expect("answer");
    // The independent walker counted the task's retained History row.
    let snapshot = Snapshot { naming: [(3, 1)].into(), ..Snapshot::default() };
    durable(&mut referee, 1, snapshot).expect("deletion may still be working");
    assert_eq!(referee.check_at(2).expect_err("history remains at quiescence").promise, Promise::Erasure);
}

#[test]
fn a_removed_projects_tasks_may_leave() {
    let mut referee = referee();
    let row = task(&referee, 1);
    let snapshot = Snapshot { tasks: [(row.number, row)].into(), naming: [(1, 1)].into(), ..Snapshot::default() };
    durable(&mut referee, 1, snapshot).expect("live task");
    referee.observe(1, Observed::ProjectRemoved { project: 1 }).expect("answer");
    referee.observe(1, Observed::Restart).expect("removal survives restart");
    durable(&mut referee, 2, Snapshot::default()).expect("removed task may leave");
    referee.check_at(3).expect("quiescent erasure");
}

#[test]
fn another_tasks_leaving_is_still_lost_work() {
    let mut referee = referee();
    let row = task(&referee, 1);
    let snapshot = Snapshot { tasks: [(row.number, row)].into(), ..Snapshot::default() };
    durable(&mut referee, 1, snapshot).expect("live task");
    referee.observe(1, Observed::ProjectRemoved { project: 3 }).expect("another project removed");
    assert_eq!(durable(&mut referee, 2, Snapshot::default()).expect_err("unrelated task lost").promise, Promise::Lost);
}

#[test]
fn rows_before_the_answer_are_not_judged() {
    let mut referee = referee();
    durable(&mut referee, 1, Snapshot { naming: [(3, 7)].into(), ..Snapshot::default() }).expect("live project");
    referee.check_at(2).expect("no answer, no erasure promise");
    referee.observe(2, Observed::ProjectRemoved { project: 3 }).expect("answer starts deletion");
    durable(&mut referee, 2, Snapshot::default()).expect("eventual erasure");
    referee.check_at(3).expect("quiescent erasure");
}

#[test]
fn a_removed_tasks_receipts_may_leave_the_cold_store() {
    let mut referee = referee();
    let row = task(&referee, 1);
    let receipt = r::Receipt::Task(row.number);
    durable(
        &mut referee,
        1,
        Snapshot { tasks: [(row.number, row)].into(), receipts: [receipt.clone()].into(), ..Snapshot::default() },
    )
    .expect("live task");
    referee.observe(1, Observed::Released { requires: vec![receipt] }).expect("outside receipt");
    referee.observe(1, Observed::ProjectRemoved { project: 1 }).expect("answer");
    durable(&mut referee, 2, Snapshot::default()).expect("erased task");
    referee.observe(2, Observed::Restart).expect("cold start");
    referee.observe(2, Observed::ColdStore { receipts: BTreeSet::new() }).expect("erasure survives restart");
}

#[test]
fn both_removal_answer_routes_reach_the_observer() {
    use jig_domain_core as core;
    use jig_domain_people as people;
    use jig_test_domain as root;
    use skein_lib::{ReplyTo, Token};
    let mut observer = jig_core_world::observations::Observer::recording(&jig_core_world::world::config(31));
    for (project, mut delivery) in [
        (
            3,
            root::Delivery::Core(core::Held::Acted {
                to: ReplyTo::new(Token::new(1)),
                outcome: people::Outcome::ProjectRemoved { project: 3 },
            }),
        ),
        (
            4,
            root::Delivery::Core(core::Held::PeopleReply {
                to: ReplyTo::new(Token::new(2)),
                reply: people::Reply::Outcome(people::Outcome::ProjectRemoved { project: 4 }),
            }),
        ),
    ] {
        observer.delivered(7, &mut delivery);
        assert!(
            observer
                .take()
                .iter()
                .any(|seen| matches!(seen, Observed::ProjectRemoved { project: seen } if *seen == project))
        );
    }
}

#[test]
fn another_tasks_receipt_leaving_the_cold_store_is_still_a_violation() {
    let mut referee = referee();
    let row = task(&referee, 1);
    let receipt = r::Receipt::Task(row.number);
    durable(
        &mut referee,
        1,
        Snapshot { tasks: [(row.number, row)].into(), receipts: [receipt.clone()].into(), ..Snapshot::default() },
    )
    .expect("live task");
    referee.observe(1, Observed::Released { requires: vec![receipt] }).expect("outside receipt");
    referee.observe(1, Observed::ProjectRemoved { project: 3 }).expect("another project removed");
    referee.observe(2, Observed::Restart).expect("cold start");
    assert_eq!(
        referee
            .observe(2, Observed::ColdStore { receipts: BTreeSet::new() })
            .expect_err("unrelated task receipt lost")
            .promise,
        Promise::Commit
    );
}
