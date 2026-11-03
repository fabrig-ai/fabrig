//! The two-call agent fits Small's store page with its derived carriers.
#[test]
fn the_two_call_agent_fits_smalls_page_with_derived_carriers() {
    use temper_real_loop_world::{fixtures, landing};
    let address = "127.0.0.1:9".parse().expect("local fixture address");
    let (mut agent, mut document) = fixtures::documents(std::path::Path::new("/template-bounds"), address, address);
    landing::configure(&mut document, &mut agent, address);
    let stock = serde_json::from_slice(include_bytes!("../../../../crates/temper-init/template/agent.json"))
        .expect("stock init agent");
    let transcript = fixtures::turn_bounds::turn_body(&stock);
    assert_eq!(transcript, 1_662_042);
    fixtures::turn_bounds::apply(&mut document, transcript);
    let given = fixtures::own_defaults(&document, &document["world_settings"]);
    document.as_object_mut().expect("deployment").remove("world_settings");
    let bytes = serde_json::to_vec(&document).expect("two-call configuration");
    temper_service::startup::named_with(&bytes, &jig_protocol_config::Directories::working(), &given)
        .expect("the two-call shape fits Small's page with derived carriers");
}
