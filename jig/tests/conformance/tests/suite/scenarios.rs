use jig_conformance::{
    Cut,
    scenarios::{ALL, Scenario, run},
};
use jig_conformance_world::Testing;

const GRACE_RESTART_COMMIT: u64 = 7;
const COMPOUND_ANSWER_COMMIT: u64 = 10;
const COMPOUND_DELEGATION_COMMIT: u64 = 7;
const COMPOUND_RUNNING_COMMIT: u64 = 9;

#[test]
fn every_shared_adversarial_story_runs_on_the_testing_application() {
    for scenario in ALL {
        run::<Testing>(scenario, Cut::None, 31)
            .unwrap_or_else(|error| panic!("{scenario:?}: {error:?}"))
            .expect("application binds all required kinds");
    }
}

fn chosen_cut(scenario: Scenario) {
    let commit = if scenario == Scenario::RunBudget { 6 } else { 7 };
    run::<Testing>(scenario, Cut::AfterCommit(commit), 31)
        .unwrap_or_else(|error| panic!("{scenario:?}, commit {commit}: {error:?}"))
        .expect("application binds all required kinds");
}

#[test]
fn late_copy_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::LateCopy);
}

#[test]
fn other_hand_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::OtherHand);
}

#[test]
fn uncertain_restarts_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::UncertainRestarts);
}

#[test]
fn changed_judge_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::ChangedJudge);
}

#[test]
fn shrinking_pool_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::ShrinkingPool);
}

#[test]
fn narrowing_policy_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::NarrowingPolicy);
}

#[test]
fn standing_task_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::StandingTask);
}

#[test]
fn run_budget_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::RunBudget);
}

#[test]
fn a_sign_in_past_its_lifetime_is_answered_signed_out() {
    use jig_fake_parties::{Act, Ask, Reply};
    let config = jig_conformance_world::Config::new(39, false, 1, None);
    let lifetime = jig_core_world::world::limits().core.people.sign_in_lifetime.as_nanos();
    let mut world = jig_conformance::Harness::<Testing>::new(config, 39);
    world.drain().expect("signed in and assigned");
    world.advance(lifetime + 1);
    world.drain().expect("expiration committed");
    world.peers.peers.parties[0].extend(Box::new([Act::Request { key: [2; 16], ask: Ask::SeeResults { through: 0 } }]));
    world.drain().expect("expired sign-in answered");
    assert_eq!(world.peers.peers.parties[0].replies.last().map(|(_, reply)| reply), Some(&Reply::SignedOut));
}

#[test]
fn racing_decisions_chosen_effect_admission_and_completion_cut_ends_the_story() {
    chosen_cut(Scenario::RacingDecisions);
}

#[test]
fn conversation_ends_while_read_chosen_cut_ends_the_story() {
    const RESULTS_COMMIT: u64 = 17;
    run::<Testing>(Scenario::ConversationEndsWhileRead, Cut::AfterCommit(RESULTS_COMMIT), 757)
        .expect("results mark and forget survive their commit cut")
        .expect("application binds the conversation story");
}

#[test]
fn query_watched_while_tasks_end_chosen_cut_ends_the_story() {
    chosen_cut(Scenario::QueryWatchedWhileTasksEnd);
}

#[test]
fn compound_chosen_answer_cut_ends_the_story() {
    run::<Testing>(Scenario::Compound, Cut::AfterCommit(COMPOUND_ANSWER_COMMIT), 31)
        .expect("compound commit survives before its answer is released")
        .expect("application binds compound");
}

#[test]
fn grace_chosen_restart_cut_ends_the_story() {
    for seed in [30, 31] {
        run::<Testing>(Scenario::Grace, Cut::AfterCommit(GRACE_RESTART_COMMIT), seed)
            .expect("grace survives its chosen restart cut")
            .expect("application binds grace");
    }
}

#[test]
fn compound_chosen_delegate_admission_cut_restores_live_delegates() {
    jig_conformance_world::replay::chosen_cuts_traced(
        Scenario::Compound,
        301,
        None,
        &[COMPOUND_DELEGATION_COMMIT, COMPOUND_RUNNING_COMMIT],
        |_, _| {},
    )
    .expect("admitted delegates resume before the compound is sent");
}

#[test]
fn project_deleted_chosen_stage_cuts_end_the_story() {
    use jig_conformance_world::{replay, scenarios::project_deletion_stage_cuts};
    use std::{cell::RefCell, rc::Rc};
    const SEED: u64 = 31;
    let baseline = replay::fresh(Scenario::ProjectDeleted, Cut::None, SEED, false).expect("uncut project deletion");
    let stages = project_deletion_stage_cuts(&baseline);
    let surviving = deletion_survivors(&baseline);
    assert!(stages.windows(2).all(|pair| pair[0] < pair[1]), "distinct deciding, erasing, releasing commits");
    drop(baseline);
    let cuts = Rc::new(RefCell::new(Vec::new()));
    let reached = Rc::clone(&cuts);
    replay::chosen_cuts_traced(Scenario::ProjectDeleted, SEED, Some(false), &stages, move |cut, world| {
        assert!(world.outcome().crashes.contains(&cut));
        let resumed = deletion_survivors(&world);
        assert_eq!(resumed.len(), surviving.len(), "same surviving row count at cut {cut}");
        for (resumed, baseline) in resumed.iter().zip(&surviving) {
            assert_eq!(resumed, baseline, "cut {cut}, row {:?}", baseline.key());
        }
        reached.borrow_mut().push(cut);
    })
    .expect("each deletion stage reaches the checked uncut ending");
    assert_eq!(*cuts.borrow(), stages);
}

fn deletion_survivors(world: &jig_conformance_world::actions::World) -> Vec<jig_test_domain::Record> {
    use jig_domain_core as core;
    use jig_domain_people as people;
    use jig_test_domain as root;
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(
                core::Record::Core(core::CoreRecord::Deployment(_))
                | core::Record::People(people::Stored::Acted { .. }),
            ) => None,
            root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Ledger(ledger)))
                if matches!(ledger.funder, jig_domain_tasks::Funder::Period { project: 3, .. }) =>
            {
                // The replacement is made after the crash; compare its funding apart from its opening clock.
                let mut ledger = *ledger;
                ledger.period.as_mut().expect("period times").began = skein_lib::Wall::EPOCH;
                Some(root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Ledger(ledger))))
            }
            root::Record::Core(_) | root::Record::Connector { .. } => Some(row.clone()),
        })
        .collect()
}

#[test]
fn project_deleted_referee_rejects_a_later_task_decision() {
    use jig_conformance::referee::{Observed, Promise, Snapshot};
    let world = jig_conformance_world::replay::fresh(Scenario::ProjectDeleted, Cut::None, 31, false)
        .expect("completed project deletion");
    let mut referee = world.referee.clone();
    let mut snapshot = Snapshot { newest: 1, ..Snapshot::default() };
    for row in world.store.rows.values() {
        if let jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task))) = row
        {
            let mut task = jig_core_world::observations::task(task);
            task.project = 2;
            snapshot.tasks.insert(task.number, task);
        }
    }
    assert_eq!(snapshot.tasks.len(), 1, "control task supplies the forbidden later decision");
    let error = referee
        .observe(world.clock().now, Observed::Durable { number: world.outcome().commits + 1, snapshot })
        .expect_err("deleted project cannot authorize a later task decision");
    assert_eq!(error.promise, Promise::Authority);
}
