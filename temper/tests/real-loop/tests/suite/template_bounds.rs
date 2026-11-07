//! Init's shipped agent fits Standard's store page with its derived carriers.
#[test]
fn the_shipped_agent_fits_standards_page_with_derived_carriers() {
    use temper_real_loop_world::{fixtures, landing};
    let address = "127.0.0.1:9".parse().expect("local fixture address");
    let (mut agent, mut document) = fixtures::documents(std::path::Path::new("/template-bounds"), address, address);
    landing::configure(&mut document, &mut agent, address);
    document["engine"]["limits"]["profile"] = "standard".into();
    document["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = (82 * 4094).into();
    for worker in document["workers"].as_array_mut().expect("workers") {
        worker["limits"]["profile"] = "standard".into();
        worker["limits"]["overrides"]["tasks.message_bytes"] = 262_144.into();
    }
    document["application"]["limits"]["profile"] = "standard".into();
    let mut stock: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../crates/temper-init/template/agent.json"))
            .expect("stock init agent");
    // Only declarations determine the turn bound; use inert direct endpoints.
    stock["programs"] = serde_json::json!({});
    for endpoint in stock["endpoints"].as_array_mut().expect("stock endpoints") {
        endpoint.as_object_mut().expect("endpoint").remove("program");
    }
    let transcript = fixtures::turn_bounds::turn_body(&stock);
    assert_eq!(transcript, 16_803_810);
    fixtures::turn_bounds::apply(&mut document, transcript);
    let given = fixtures::own_defaults(&document, &document["world_settings"]);
    temper_playground::documents::settings::strip_models(&mut document);
    document.as_object_mut().expect("deployment").remove("world_settings");
    let bytes = serde_json::to_vec(&document).expect("shipped agent configuration");
    temper_service::startup::named_with(&bytes, &jig_protocol_config::Directories::working(), &given)
        .expect("the shipped agent fits Standard's page with derived carriers");
}
