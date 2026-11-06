#[test]
fn signing_in_and_starting_a_chat() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_kit::stories::shared::signing_in_and_starting_a_chat(&mut world);
}

mod a;

fn world(facts: jig_client_kit::stories::shared::b_seed::Facts) -> jig_client_protocol_world::stories::Story {
    jig_client_kit::World::new(
        jig_client_kit::Settings { seed: 8510, ..jig_client_kit::Settings::default() },
        jig_client_protocol_world::Protocol::story(8510, &facts),
    )
}
jig_client_kit::person_story_suite!(world; release: [], compound_release: [], inbox_chat: [], close: [], family: []);
