//! Faults across authenticated tabs use the actual proposal and retained decision.
use super::*;
#[test]
fn two_tabs_decide_one_proposal_and_both_receive_the_original_once_only_decision() {
    let mut world = super::super::a_shared::proposals_world();
    crate::stories::shared::a_proposals::two_tabs_decide_one_proposal(&mut world);
    let first_person = 1;
    let engine = &world.front().session.driver.world;
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::ProposalDecided { by, choice: people::ProposalChoice::Accepted, .. }
            if *by == first_person)))
            .count(),
        2
    );
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.numbers.budget == 50))
            .count(),
        1
    );
    let assignments = &engine.peers.as_ref().expect("actual worker").workers[0].seen;
    assert_eq!(assignments.len(), 3, "the proposing chat resumes after its one accepted goal");
    let tasks: std::collections::BTreeSet<_> = assignments.iter().map(|row| row.task).collect();
    assert_eq!(tasks.len(), 2, "only the proposing chat and its accepted goal");
    let parent = assignments[0].task;
    let parent_attempts: std::collections::BTreeSet<_> =
        assignments.iter().filter(|row| row.task == parent).map(|row| row.attempt).collect();
    assert_eq!(parent_attempts.len(), 2, "the original chat parks and then resumes");
    assert_eq!(assignments.iter().filter(|row| row.task != parent).count(), 1, "one goal assignment");
}

#[test]
fn an_inbox_acceptance_updates_the_same_proposal_still_shown_in_the_chat() {
    let mut world = super::super::world(super::super::turn());
    crate::stories::shared::a_roles::an_owners_decision_updates_the_members_chat(&mut world);
    let engine = &world.front().session.driver.world;
    let person = engine.peers.as_ref().expect("actual parties").parties[0].person.expect("actual owner");
    assert_eq!(engine.store.rows.values().filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. }))
        if matches!(outcome, people::Outcome::ProposalDecided { by, choice: people::ProposalChoice::Accepted, .. } if *by == person))).count(), 1);
    assert_eq!(
        engine
            .store
            .rows
            .values()
            .filter(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ended(task)))
        if task.tracked == Some(7) && task.numbers.budget == 150))
            .count(),
        1
    );
    let assignments = &engine.peers.as_ref().expect("actual worker").workers[0].seen;
    assert_eq!(assignments.len(), 3);
    let tasks: std::collections::BTreeSet<_> = assignments.iter().map(|row| row.task).collect();
    assert_eq!(tasks.len(), 2);
    assert_eq!(assignments.iter().filter(|row| row.task != assignments[0].task).count(), 1);
}
