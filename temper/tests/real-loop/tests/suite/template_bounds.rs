//! The fixture names a two-call transcript past Small's store page.
#[test]
fn the_two_call_agent_past_smalls_page_is_refused_by_name() {
    use temper_real_loop_world::{fixtures, landing};
    let address = "127.0.0.1:9".parse().expect("local fixture address");
    let (mut agent, mut document) = fixtures::documents(std::path::Path::new("/template-bounds"), address, address);
    landing::configure(&mut document, &mut agent, address);
    agent["profile"]["declared"]["calls_per_response"] = 2.into();
    assert_eq!(agent["profile"]["declared"]["tool_payload"], 4096);
    assert_eq!(agent["endpoints"][0]["models"][0]["output"], 8192);
    let transcript = fixtures::turn_bounds::turn_body(&agent);
    assert_eq!(transcript, 1_792_262);
    fixtures::turn_bounds::apply(&mut document, transcript);
    let given = fixtures::own_defaults(&document, &document["world_settings"]);
    document.as_object_mut().expect("deployment").remove("world_settings");
    let bytes = serde_json::to_vec(&document).expect("old two-call configuration");
    let refused = temper_service::startup::named_with(&bytes, &jig_protocol_config::Directories::working(), &given)
        .expect_err("the old two-call shape exceeds Small's page");
    assert_eq!(
        temper_service::startup::lines(&refused).as_ref(),
        b"application.limits.overrides.root.loads.transcript_bytes: its turn's transcript frame, 1792326 bytes, needs a store page past the profile's ceiling of 1590110 bytes\n"
    );
}
