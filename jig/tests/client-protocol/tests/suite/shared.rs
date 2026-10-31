#[test]
fn signing_in_and_starting_a_chat() {
    let mut world = jig_client_protocol_world::fixtures::world(8510);
    jig_client_world::stories::shared::signing_in_and_starting_a_chat(&mut world);
}

mod a;
