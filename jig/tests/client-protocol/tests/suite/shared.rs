#[test]
fn signing_in_and_starting_a_chat() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::signing_in_and_starting_a_chat(&mut world);
}
fn world(facts: jig_client_world::stories::shared::b_seed::Facts) -> jig_client_protocol_world::stories::Story {
    jig_client_world::World::new(
        jig_client_world::Settings { seed: 8510, ..jig_client_world::Settings::default() },
        jig_client_protocol_world::Protocol::story(8510, &facts),
    )
}
jig_client_world::person_story_suite!(world; release: [], compound_release: [], inbox_chat: [], close: [], family: []);
