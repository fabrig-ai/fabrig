//! Pure configuration fixtures shared by unit and process tests.
use std::path::Path;
/// Turn measurements retain model declarations with inert direct endpoints.
pub mod turn_bounds {
    pub use temper_playground::documents::turn_bounds::apply;

    #[must_use]
    pub fn turn_body(agent: &serde_json::Value) -> u32 {
        let mut agent = agent.clone();
        agent["programs"] = serde_json::json!({});
        for endpoint in agent["endpoints"].as_array_mut().expect("agent endpoints") {
            endpoint.as_object_mut().expect("endpoint").remove("program");
        }
        temper_playground::documents::turn_bounds::turn_body(&agent)
    }
}
// Eight native exchanges beside Small's two event streams (jig/docs/design/protocol/client/default-http.md, 5).
const NATIVE_HTTP_CALLS: u32 = 10;
#[must_use]
pub fn documents(directory: &Path, program: &str) -> (Vec<u8>, Vec<u8>) {
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":2048,"read_window":256,
            "conversations":1,"calls_per_response":1},
            "policy":{"max_turns":800,"check_timeout":60000,"group_stop":10,"inbox":8,"unacknowledged":8}},
        "endpoints":[{"name":"fake","number":1,"dialect":3,"provider":"openai",
            "address":"127.0.0.1:0","transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),
            "connect":1000,"handshake":1000,
            "limits": {"window":1_048_576,"output":8192,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[]});
    let mut value: serde_json::Value =
        serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("whole object").remove("service");
    for name in ["views.watchers", "views.watchers_per_party"] {
        value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides").remove(name);
    }
    use_stock(&mut value);
    for project in value["engine"]["projects"].as_array_mut().expect("seed projects") {
        project["policy"] = 1.into();
        project.as_object_mut().expect("seed").remove("roles");
    }
    // A Create member owns stock's fourteen grants and eighteen executors, plus its words.
    value["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 2048.into();
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_kit::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"]["program"] = program.into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["internal", "agent", directory.join("agent.json")]);
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 2046.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["dialect"] = 3.into();
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:0"});
    // The stock carries ten charters, fourteen grants and eighteen executor kinds.
    value["engine"]["limits"]["overrides"]["tasks.charters"] = 10.into();
    value["engine"]["limits"]["overrides"]["core.templates"] = 6.into();
    // The standing charter, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["authority.executors"] = 18.into();
    // The standing charter, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["tasks.executor_kinds"] = 18.into();
    value["engine"]["limits"]["overrides"]["authority.grants"] = 16.into();
    // A chat copies stock policy 1's fourteen grants into its task authority.
    value["engine"]["limits"]["overrides"]["tasks.authority_grants"] = 16.into();
    // Fourteen literal "forge" segments need seventy bytes, beyond the kit fixture's 64.
    value["engine"]["limits"]["overrides"]["tasks.authority_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["notes.scopes"] = 32.into();
    value["engine"]["limits"]["overrides"]["core.policy_bytes"] = 8192.into();
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["documents.rows"] = 2.into();
    // The creation offer carries nine stock charters and four procedures.
    value["engine"]["limits"]["overrides"]["documents.lines"] = 9.into();
    value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    for name in ["clients.exchanges", "clients_http.calls"] {
        value["engine"]["limits"]["overrides"][name] = NATIVE_HTTP_CALLS.into();
    }
    // Projects 0 and 1, and the one forge-setup makes for the first repository.
    value["engine"]["limits"]["overrides"]["people.projects"] = 3.into();
    value["engine"]["limits"]["overrides"]["authority.projects"] = 3.into();
    value["engine"]["limits"]["overrides"]["tasks.funders"] = 4.into();
    value["application"] = serde_json::json!({
        "webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:0"},"public":null,"bundle":null},
        "machine":{"github":null},"demo":null,"sign_in":null,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.judge_projects":3,"forge.resources_per_task":2,"git_component.stdout_bytes":131_072}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    turn_bounds::apply(&mut value, turn_bounds::turn_body(&agent));
    (serde_json::to_vec(&agent).expect("agent JSON"), serde_json::to_vec(&value).expect("deployment JSON"))
}

/// Keep a copied fixture's startup members and use temper's stock settings.
pub fn use_stock(value: &mut serde_json::Value) {
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine object").remove(member);
    }
    temper_playground::documents::settings::strip_models(value);
    value["engine"]["limits"]["overrides"]["authority.roles"] = 4.into();
    value["engine"]["limits"]["overrides"]["core.run_bytes"] = 32_768.into();
    value["engine"]["limits"]["overrides"]["tasks.charters"] = 10.into();
    value["engine"]["limits"]["overrides"]["core.templates"] = 6.into();
    // The standing charter, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["authority.executors"] = 18.into();
    // The standing charter, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["tasks.executor_kinds"] = 18.into();
    value["engine"]["limits"]["overrides"]["documents.lines"] = 9.into();
}

/// All directories are explicit and belong to this fixture's scratch.
#[must_use]
pub fn directories(directory: &Path) -> temper_box_shell::directories::Directories {
    use std::os::unix::ffi::OsStrExt;
    let values = [
        b"--config".as_slice(),
        directory.as_os_str().as_bytes(),
        b"--data",
        directory.as_os_str().as_bytes(),
        b"--cache",
        directory.as_os_str().as_bytes(),
    ];
    temper_box_shell::directories::resolve(&values.iter().map(|v| Box::from(*v)).collect::<Vec<_>>(), &[], b"/")
        .expect("fixture directories")
        .0
}
#[must_use]
pub fn flags(directory: &Path) -> Vec<std::ffi::OsString> {
    use std::os::unix::ffi::OsStringExt;
    directories(directory).flags().into_vec().into_iter().map(|v| std::ffi::OsString::from_vec(v.into_vec())).collect()
}

/// Set the machine's fake listener and remote together.
pub fn fake_entry(entry: &mut serde_json::Value, listen: &str) {
    *entry = serde_json::json!({"listen":{"socket":listen}, "remote":format!("http://{listen}"), "git":"demo/bin/git"});
}

/// The explicit old-form fake and managed command fixtures, without creating files.
#[must_use]
pub fn old_form_check_document(directory: &Path) -> serde_json::Value {
    check_document(directory)
}

/// The fake starts from records; its identity joins the forge explicitly.
#[must_use]
pub fn check_document(directory: &Path) -> serde_json::Value {
    let (_, bytes) = documents(directory, "/fixture/temper");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("fixture JSON");
    value["application"]["machine"]["fake"] = serde_json::json!({
        "listen":{"socket":"127.0.0.1:9"}, "remote":"http://127.0.0.1:9", "git":"demo/bin/git"
    });
    value
}

/// A demo model whose one plaintext endpoint listens at the supplied address.
#[must_use]
pub fn demo_document(directory: &Path, listen: &str) -> serde_json::Value {
    let mut value = check_document(directory);
    let llm = serde_json::json!({
        "providers":[{"name":"demo", "number":3, "endpoint":"demo", "sources":{"keys":true}}],
        "endpoints":[{"name":"demo", "number":3, "dialect":3, "provider":"demo",
            "address":{"socket":listen}, "transport":"plaintext", "headers":[],
            "connect_ms":10000, "handshake_ms":10000}]
    });
    for field in ["providers", "endpoints"] {
        value["engine"]["llm"][field]
            .as_array_mut()
            .expect("LLM table")
            .extend(llm[field].as_array().expect("demo table").iter().cloned());
    }
    value["application"]["demo"] = serde_json::json!({"llm":{
        "listen":{"socket":listen}, "provider":3, "key":"demo/llm.key",
        "asked":[{"cue":"say hello", "turns":["hello from QA"]}], "otherwise":["fallback"]}});
    value
}
