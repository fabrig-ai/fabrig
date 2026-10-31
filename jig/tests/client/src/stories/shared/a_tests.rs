//! Scripted journeys (jig/docs/design/client/domain/testing.md, 4).

#[test]
fn words_in_a_running_conversation() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::words_in_a_running_conversation(&mut world);
}

#[test]
fn stopping_and_releasing_a_conversation() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::stopping_and_releasing_a_conversation(&mut world);
}

#[test]
fn a_draft_survives_reload() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::a_draft_survives_reload(&mut world);
}

#[test]
fn reading_project_settings() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::reading_project_settings(&mut world);
}

#[test]
fn reading_deployment() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::reading_deployment(&mut world);
}

#[test]
fn reading_a_completed_conversation() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::reading_a_completed_conversation(&mut world);
}

#[test]
fn revisiting_a_completed_conversation() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::revisiting_a_completed_conversation(&mut world);
}

#[test]
fn accepting_a_proposed_goal() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_proposals::accepting_a_proposed_goal(&mut world);
}

#[test]
fn two_tabs_decide_one_proposal() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_proposals::two_tabs_decide_one_proposal(&mut world);
}

#[test]
fn an_inbox_decision_updates_its_conversation() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_proposals::an_inbox_decision_updates_its_conversation(&mut world);
}

#[test]
fn signing_out() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_person::signing_out(&mut world);
}

#[test]
fn roles_a_maintainer_accepts_a_members_proposal() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_roles::a_maintainer_accepts_a_members_proposal(&mut world);
}

#[test]
fn roles_two_parties_decide_one_proposal() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_roles::two_parties_decide_one_proposal(&mut world);
}

#[test]
fn roles_an_owners_decision_updates_the_members_chat() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_roles::an_owners_decision_updates_the_members_chat(&mut world);
}

#[test]
fn roles_answering_a_choice_between_reports() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_roles::answering_a_choice_between_reports(&mut world);
}

#[test]
fn roles_a_goal_beyond_a_members_allotment_is_proposed() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_roles::a_goal_beyond_a_members_allotment_is_proposed(&mut world);
}

#[test]
fn roles_an_observer_sees_why_creation_is_unavailable() {
    let mut world = crate::stories::seeded_world(8510);
    super::a_roles::an_observer_sees_why_creation_is_unavailable(&mut world);
}
