use super::*;

#[test]
fn serve_prepares_sources_and_places_only_the_fetched_launcher() {
    let data = Path::new("/fixture/data");
    let programs = data.join("programs");
    let bin = b"/fixture/program-bin";
    let on_path = "/fixture/program-bin/claude";
    let environment = [(b"PATH".as_slice().into(), bin.as_slice().into())];
    let (agent, _) = crate::tests::worker_document(data);
    let mut document: serde_json::Value = serde_json::from_slice(&agent).expect("agent fixture");
    for endpoint in document["endpoints"].as_array_mut().expect("agent endpoints") {
        endpoint["address"] = "127.0.0.1:7100".into();
        endpoint["transport"] = "plaintext".into();
        let endpoint = endpoint.as_object_mut().expect("agent endpoint");
        endpoint.remove("trust_der");
        endpoint.remove("server_name");
    }
    document["environment"] = serde_json::json!([]);

    for found_on_path in [true, false] {
        let selected = find_claude_with(&environment, |path| {
            assert_eq!(path, Path::new(on_path));
            found_on_path
        });
        document["programs"] = serde_json::json!({
            "claude_code": {"path": selected.as_deref().unwrap_or(Path::new("fetched")), "directory": programs}
        });
        let bytes = serde_json::to_vec(&document).expect("agent fixture").into_boxed_slice();
        let prepared = agent::prepare_bytes_with(bytes, &mut Vec::new(), |path| {
            assert_eq!(path, programs);
            Ok(())
        })
        .expect("prepared agent");
        let mut placements = Vec::new();
        let sources = source_inputs_with(&prepared.agent, prepared.environment, 1000, &[], false, |directory| {
            placements.push(directory.to_path_buf());
            Ok(())
        })
        .expect("prepared sources");
        assert!(sources.resources.is_some());
        if found_on_path {
            assert!(placements.is_empty(), "the PATH row has no launcher or version directory");
        } else {
            assert_eq!(placements, [data.join("claude-code")], "serve requests placement, not a fetch");
        }
    }
}
