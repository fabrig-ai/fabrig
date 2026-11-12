//! A genuine member's bounded goal is disclosed as a proposal before sending.
use super::*;

#[test]
fn an_outside_allotment_goal_is_shown_as_proposed_before_send_and_named_in_the_answer() {
    let mut world = super::super::world(super::super::turn());
    let member = crate::stories::shared::a_roles::prepare_goal(&mut world);
    assert!(!world.front().session.driver.world.store.rows.values().any(|row| matches!(
        row,
        jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer { .. }))
    )));
    crate::stories::shared::a_roles::confirm_goal(&mut world, member);
    let answers: Vec<_> = world
        .front()
        .session
        .driver
        .world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            jig_test_domain::Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
                outcome, ..
            })) => Some(outcome.clone()),
            jig_test_domain::Record::Core(_) | jig_test_domain::Record::Connector { .. } => None,
        })
        .collect();
    assert_eq!(answers, vec![people::Outcome::Proposed { proposal: 1, to: 0 }]);
    assert_eq!(world.front().session.driver.world.peers.as_ref().expect("actual worker").workers[0].seen.len(), 0);
}
