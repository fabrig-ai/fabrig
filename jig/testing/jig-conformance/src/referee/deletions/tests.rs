use crate::referee::{
    Naming, Observed, Outlives, Phase, Policy, Promise, Receipt, Referee, Row, Scope, Snapshot, Source, Task,
};
use std::collections::{BTreeMap, BTreeSet};

const AT: u64 = 0;
const PROJECT: u32 = 1;
const TASK: u64 = 7;

#[test]
fn only_ended_deletions_retain_their_proven_acceptance_during_cleanup() {
    for phase in [Phase::Active, Phase::Ended] {
        for deleting in [false, true] {
            let (_, mut before) = baseline();
            before.tasks.get_mut(&TASK).expect("baseline task").phase = phase;
            let acceptance = crate::referee::Acceptance { proposal: 1, by: Source::Person(3) };
            before.accepted_tasks.insert(TASK, acceptance);
            let mut after = before.clone();
            after.accepted_tasks.clear();
            if deleting {
                after.deleting.insert(PROJECT);
            }
            super::Deletions::default().retain_acceptances(&before, &mut after);
            assert_eq!(
                after.accepted_tasks.get(&TASK).copied(),
                (deleting && phase == Phase::Ended).then_some(acceptance)
            );
        }
    }
}

fn task(number: u64) -> Task {
    Task {
        settings: crate::referee::SettingsRevision::default(),
        spec_revision: 0,
        hold_revision: 0,
        placed: None,
        placed_reason: false,
        tracked: false,
        number,
        project: PROJECT,
        source: Source::Deployment,
        scope: Scope::default(),
        executor: (0, 1),
        budget: 0,
        spent: 0,
        spent_below: 0,
        reserved: 0,
        run_reserved: 0,
        run_spent: 0,
        attempt: 0,
        period: None,
        phase: Phase::Done,
        dependencies: vec![],
        delegates: vec![],
        pools: vec![],
        inbox: vec![],
        proposal: None,
    }
}

fn baseline() -> (Referee, Snapshot) {
    let mut referee = Referee::new(Policy {
        projections: BTreeMap::new(),
        deployment: Scope::default(),
        projects: BTreeMap::from([(PROJECT, Scope::default()), (2, Scope::default())]),
        people: BTreeMap::new(),
        effect_accepters: BTreeSet::new(),
        requirements: vec![],
        implications: BTreeSet::new(),
        kinds: BTreeMap::new(),
        owned: vec![],
        participating: BTreeSet::new(),
        mechanics: BTreeSet::new(),
        delivery_bound: 20,
        uncertainty_bound: 30,
        story_steps: 100,
    });
    let snapshot = Snapshot {
        tasks: BTreeMap::from([(TASK, task(TASK))]),
        receipts: BTreeSet::from([Receipt::Task(TASK), Receipt::Ended(TASK)]),
        read_positions: BTreeMap::from([((3, TASK), 4)]),
        ..Snapshot::default()
    };
    referee.observe(AT, Observed::Durable { number: 1, snapshot: snapshot.clone() }).expect("known ended task");
    referee
        .observe(AT, Observed::Released { requires: snapshot.receipts.iter().cloned().collect() })
        .expect("receipts seen");
    referee.observe(AT, Observed::Result { task: TASK, to: Source::Deployment }).expect("result delivered");
    (referee, snapshot)
}

fn removed() -> (Referee, Snapshot) {
    let (mut referee, mut snapshot) = baseline();
    snapshot.deleting.insert(PROJECT);
    referee.observe(AT, Observed::Durable { number: 2, snapshot: snapshot.clone() }).expect("deletion kept");
    referee.observe(AT, Observed::ProjectRemoved { project: PROJECT }).expect("deletion answered");
    (referee, snapshot)
}

fn row(project: Option<u32>, task: Option<u64>, outlives: Option<Outlives>) -> Naming {
    Naming { row: Row::Application("row-key".into()), project, task, outlives }
}

fn assert_lost(referee: &mut Referee, snapshot: Snapshot, number: u64) {
    let violation = referee.observe(AT, Observed::Durable { number, snapshot }).expect_err("leftover row");
    assert_eq!(violation.promise, Promise::Lost);
    assert_eq!(violation.why, "a deleted project's row outlived it: row-key");
}

#[test]
fn a_core_row_outliving_its_project_is_named_by_its_key() {
    let (mut referee, _) = removed();
    let snapshot = Snapshot {
        naming: vec![Naming {
            row: Row::Core(jig_domain_core_vocabulary::Key::Tasks(jig_domain_tasks::Key::Live(TASK))),
            project: Some(PROJECT),
            task: None,
            outlives: None,
        }],
        ..Snapshot::default()
    };
    let violation = referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect_err("leftover core row");
    assert_eq!(violation.promise, Promise::Lost);
    assert_eq!(violation.why, "a deleted project's row outlived it: Tasks(Live(7))");
}

#[test]
fn a_deletion_that_leaves_nothing_named_passes() {
    let (mut referee, _) = removed();
    referee.observe(AT, Observed::Durable { number: 3, snapshot: Snapshot::default() }).expect("whole erasure");
    referee.observe(AT, Observed::ColdStore { receipts: BTreeSet::new() }).expect("erased receipts stay erased");
    referee.observe(AT, Observed::Spend { task: TASK, spent: 0, lost_bound: 0 }).expect("ended budget retained");
    referee.observe(AT, Observed::Drained { at: 3 }).expect("whole deletion drains");
    let (mut referee, _) = removed();
    assert_lost(&mut referee, Snapshot { naming: vec![row(Some(PROJECT), None, None)], ..Snapshot::default() }, 3);
}

#[test]
fn a_row_naming_a_deleted_project_after_its_row_is_gone_fails() {
    for project in [2, PROJECT] {
        let (mut referee, _) = removed();
        let snapshot = Snapshot { naming: vec![row(Some(project), None, None)], ..Snapshot::default() };
        if project == PROJECT {
            assert_lost(&mut referee, snapshot, 3);
        } else {
            referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect("another project's row");
        }
    }
}

#[test]
fn a_row_naming_a_deleted_projects_task_fails() {
    for number in [99, TASK] {
        let (mut referee, _) = baseline();
        referee
            .observe(AT, Observed::Durable { number: 2, snapshot: Snapshot::default() })
            .expect("task erased before answer");
        referee.observe(AT, Observed::ProjectRemoved { project: PROJECT }).expect("deletion answered");
        let snapshot = Snapshot { naming: vec![row(None, Some(number), None)], ..Snapshot::default() };
        if number == TASK {
            assert_lost(&mut referee, snapshot, 3);
        } else {
            referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect("an unrelated task's row");
        }
    }
}

#[test]
fn spend_and_a_sole_use_outlive_the_project() {
    for outlives in [Some(Outlives::Spend), Some(Outlives::SoleUse), None] {
        let (mut referee, _) = removed();
        let snapshot = Snapshot { naming: vec![row(Some(PROJECT), None, outlives)], ..Snapshot::default() };
        if outlives.is_some() {
            referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect("permitted remainder");
            referee.observe(AT, Observed::Drained { at: 3 }).expect("permitted remainder drains");
        } else {
            assert_lost(&mut referee, snapshot, 3);
        }
    }
}

#[test]
fn a_new_task_in_a_closed_project_fails() {
    for number in [TASK, 8] {
        let (mut referee, mut snapshot) = removed();
        referee.observe(AT, Observed::Restart).expect("restart keeps closure");
        referee.observe(AT, Observed::ProjectRemoved { project: PROJECT }).expect("resend keeps evidence");
        snapshot.tasks.insert(number, task(number));
        let result = referee.observe(AT, Observed::Durable { number: 3, snapshot });
        if number == TASK {
            result.expect("known task remains while deletion runs");
        } else {
            let violation = result.expect_err("new task after answer");
            assert_eq!(violation.promise, Promise::Ownership);
            assert_eq!(violation.why, "a closed project took a new task");
        }
    }
    let (mut referee, mut snapshot) = removed();
    let mut unrelated = task(8);
    unrelated.project = 2;
    snapshot.tasks.insert(8, unrelated);
    referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect("another project still takes tasks");
}

#[test]
fn rows_may_name_the_project_while_its_deletion_is_kept() {
    for kept in [true, false] {
        let (mut referee, _) = removed();
        let snapshot = Snapshot {
            deleting: if kept { BTreeSet::from([PROJECT]) } else { BTreeSet::new() },
            naming: vec![row(Some(PROJECT), Some(TASK), None)],
            ..Snapshot::default()
        };
        if kept {
            referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect("cleanup rows remain until whole");
        } else {
            assert_lost(&mut referee, snapshot, 3);
        }
    }
}

#[test]
fn draining_with_a_deletion_kept_fails() {
    for kept in [false, true] {
        let (mut referee, _) = removed();
        referee.observe(AT, Observed::Restart).expect("cut deletion");
        let snapshot = Snapshot {
            deleting: if kept { BTreeSet::from([PROJECT]) } else { BTreeSet::new() },
            ..Snapshot::default()
        };
        referee.observe(AT, Observed::Durable { number: 3, snapshot }).expect("resumed deletion");
        let result = referee.observe(AT, Observed::Drained { at: 3 });
        if kept {
            let violation = result.expect_err("incomplete deletion");
            assert_eq!(violation.promise, Promise::Order);
            assert_eq!(violation.why, "a deletion was cut and never resumed");
        } else {
            result.expect("resumed deletion ended whole");
        }
    }
}

#[test]
fn only_an_ended_task_in_a_kept_deletion_can_drop_its_read_position() {
    for phase in [Phase::Active, Phase::Ended] {
        for deleting in [false, true] {
            let (_, mut before) = baseline();
            before.tasks.get_mut(&TASK).expect("baseline task").phase = phase;
            let mut after = before.clone();
            after.read_positions.clear();
            if deleting {
                after.deleting.insert(PROJECT);
            }
            let result = super::super::positions::check(&before, &after);
            assert_eq!(result.is_ok(), deleting && phase == Phase::Ended);
        }
    }
}

#[test]
fn only_ended_deletions_retire_erased_turn_and_terminal_prerequisites() {
    for phase in [Phase::Active, Phase::Ended] {
        for deleting in [false, true] {
            let (_, mut before) = baseline();
            before.tasks.get_mut(&TASK).expect("baseline task").phase = phase;
            if deleting {
                before.deleting.insert(PROJECT);
            }
            let mut seen = BTreeSet::from([Receipt::Turn(TASK, 1, 1), Receipt::Terminal(TASK, 1)]);
            super::Deletions::default().retire_receipts(&before, &before, &mut seen);
            assert_eq!(seen.is_empty(), deleting && phase == Phase::Ended);
        }
    }
}
