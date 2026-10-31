//! Shared journeys over bytes (jig/docs/design/client/domain/testing.md, 5).

#[test]
fn words_in_a_running_conversation() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::words_in_a_running_conversation(&mut world);
}

#[test]
fn stopping_and_releasing_a_conversation() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::stopping_and_releasing_a_conversation(&mut world);
}

#[test]
fn a_draft_survives_reload() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::a_draft_survives_reload(&mut world);
}

#[test]
fn reading_project_settings() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::reading_project_settings(&mut world);
}

#[test]
fn reading_deployment() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::reading_deployment(&mut world);
}

#[test]
fn reading_a_completed_conversation() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::reading_a_completed_conversation(&mut world);
}

#[test]
fn revisiting_a_completed_conversation() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::revisiting_a_completed_conversation(&mut world);
}

#[test]
fn accepting_a_proposed_goal() {
    let mut world = a_proposals_world();
    jig_client_world::stories::shared::a_proposals::accepting_a_proposed_goal(&mut world);
}

#[test]
fn two_tabs_decide_one_proposal() {
    let mut world = a_proposals_world();
    jig_client_world::stories::shared::a_proposals::two_tabs_decide_one_proposal(&mut world);
}

#[test]
fn an_inbox_decision_updates_its_conversation() {
    let mut world = a_proposals_world();
    jig_client_world::stories::shared::a_proposals::an_inbox_decision_updates_its_conversation(&mut world);
}

#[test]
fn signing_out() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_person::signing_out(&mut world);
}

fn a_proposals_world() -> jig_client_world::World<jig_client_world::Test, jig_client_protocol_world::Protocol> {
    let mut limits = jig_client_protocol_world::fixtures::owner_limits();
    limits.core.people.sign_ins = 3;
    limits.core.views.watchers = 12;
    jig_client_world::World::new(
        jig_client_world::Settings { seed: 8510, ..jig_client_world::Settings::default() },
        jig_client_protocol_world::Protocol::bounded(8510, &limits),
    )
}

#[test]
#[ignore = "printed_link_has_one_identity: this protocol fixture authenticates only its first owner"]
fn roles_a_maintainer_accepts_a_members_proposal() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_roles::a_maintainer_accepts_a_members_proposal(&mut world);
}

#[test]
#[ignore = "printed_link_has_one_identity: this protocol fixture authenticates only its first owner"]
fn roles_two_parties_decide_one_proposal() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_roles::two_parties_decide_one_proposal(&mut world);
}

#[test]
#[ignore = "printed_link_has_one_identity: this protocol fixture authenticates only its first owner"]
fn roles_an_owners_decision_updates_the_members_chat() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_roles::an_owners_decision_updates_the_members_chat(&mut world);
}

#[test]
#[ignore = "printed_link_has_one_identity: this protocol fixture authenticates only its first owner"]
fn roles_answering_a_choice_between_reports() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_roles::answering_a_choice_between_reports(&mut world);
}

#[test]
#[ignore = "printed_link_has_one_identity: this protocol fixture authenticates only its first owner"]
fn roles_a_goal_beyond_a_members_allotment_is_proposed() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_roles::a_goal_beyond_a_members_allotment_is_proposed(&mut world);
}

#[test]
#[ignore = "printed_link_has_one_identity: this protocol fixture authenticates only its first owner"]
fn roles_an_observer_sees_why_creation_is_unavailable() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::a_roles::an_observer_sees_why_creation_is_unavailable(&mut world);
}
