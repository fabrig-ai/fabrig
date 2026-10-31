//! Shared journeys on the joint front (jig/docs/design/client/domain/testing.md, 5).
use super::{bounded_world, turn, world};
use crate::{Test, World, joint::Joint};

#[test]
fn reading_deployment() {
    let mut world = world(turn());
    crate::stories::shared::a_person::reading_deployment(&mut world);
}

#[test]
fn reading_a_completed_conversation() {
    let mut world = world(turn());
    crate::stories::shared::a_person::reading_a_completed_conversation(&mut world);
}

#[test]
fn revisiting_a_completed_conversation() {
    let mut world = world(turn());
    crate::stories::shared::a_person::revisiting_a_completed_conversation(&mut world);
}

#[test]
fn accepting_a_proposed_goal() {
    let mut world = proposals_world();
    crate::stories::shared::a_proposals::accepting_a_proposed_goal(&mut world);
}

#[test]
fn two_tabs_decide_one_proposal() {
    let mut world = proposals_world();
    crate::stories::shared::a_proposals::two_tabs_decide_one_proposal(&mut world);
}

#[test]
fn an_inbox_decision_updates_its_conversation() {
    let mut world = proposals_world();
    crate::stories::shared::a_proposals::an_inbox_decision_updates_its_conversation(&mut world);
}

pub(super) fn proposals_world() -> World<Test, Joint> {
    bounded_world(
        turn(),
        |_| {},
        |limits| {
            limits.core.people.sign_ins = 3;
            limits.core.views.watchers = 12;
        },
    )
}
