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
pub fn documents(directory: &Path, _program: &str) -> (Vec<u8>, Vec<u8>) {
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":4096,"read_window":256,
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
    for name in ["views.watchers", "views.watchers_per_party", "core.call_answer_bytes"] {
        value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides").remove(name);
    }
    use_stock(&mut value);
    // A Create member owns stock's fourteen grants and eighteen executors, plus its words.
    value["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 2048.into();
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_kit::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = 4096.into();
    value["engine"]["limits"]["overrides"]["tasks.run_inbox"] = 8.into();
    value["workers"][0]["limits"]["overrides"]["tasks.run_inbox"] = 8.into();
    value["workers"][0]["limits"]["overrides"]["hub.inbox"] = 8.into();
    value["workers"][0]["limits"]["overrides"]["hub.held"] = 8.into();
    value["engine"]["limits"].as_object_mut().expect("engine limits").remove("profile");
    value["workers"][0]["limits"].as_object_mut().expect("worker limits").remove("profile");
    value["workers"][0]["limits"]["overrides"]
        .as_object_mut()
        .expect("worker overrides")
        .entry("tasks.result_bytes")
        .or_insert(512.into());
    for worker in value["workers"].as_array_mut().expect("workers") {
        let worker = worker.as_object_mut().expect("worker");
        worker.remove("graces");
        worker.remove("programs");
    }
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0].as_object_mut().expect("worker").remove("agent");
    value["workers"][0].as_object_mut().expect("worker").remove("limits");
    value["engine"]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["engine"]["limits"]["overrides"]["hub.run_calls"] = 1.into();
    value["engine"]["limits"]["overrides"]["hub.inbox"] = 8.into();
    value["engine"]["limits"]["overrides"]["hub.held"] = 8.into();
    value["workers"][0]["name"] = "local".into();
    value["engine"]["limits"]["overrides"]["fleet.slots"] = 4.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["llm"]["endpoints"][0]["dialect"] = 3.into();
    value["engine"]["llm"]["endpoints"][0]["provider"] = "openai".into();
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":"127.0.0.1:0"});
    // The stock carries twelve charters, fourteen grants and twenty-two executor kinds.
    value["engine"]["limits"]["overrides"]["tasks.charters"] = 16.into();
    value["engine"]["limits"]["overrides"]["core.templates"] = 7.into();
    // The standing charters, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["authority.executors"] = 22.into();
    // The standing charters, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["tasks.executor_kinds"] = 22.into();
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
    value["engine"]["limits"]["overrides"]["documents.lines"] = 12.into();
    value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    for name in ["clients.streams", "clients_http.streams"] {
        value["engine"]["limits"]["overrides"][name] = 2.into();
    }
    for name in ["clients.streams", "clients_http.streams"] {
        value["engine"]["limits"]["overrides"][name] = 2.into();
    }
    for name in ["clients.exchanges", "clients_http.calls"] {
        value["engine"]["limits"]["overrides"][name] = NATIVE_HTTP_CALLS.into();
    }
    // Projects 0 and 1, and the one forge-setup makes for the first repository.
    value["engine"]["limits"]["overrides"]["people.projects"] = 3.into();
    value["engine"]["limits"]["overrides"]["authority.projects"] = 3.into();
    value["engine"]["limits"]["overrides"]["tasks.funders"] = 4.into();
    // The journal's charter carrier matches this fixture's core.run_bytes.
    value["application"] = serde_json::json!({
        "webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:0"},"public":null,"bundle":null},
        "demo":null,"sign_in":null,
        "limits":{"overrides":{"forge.client.answer_bytes":65_536,"forge.client.op_bytes":512,"forge_component.body_bytes":16_384,"root.journal.run_bytes":32_768,"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.judge_projects":3,"forge.landing_members":3,"forge.resources_per_task":4,"git_component.stdout_bytes":131_072}}});
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
    value["engine"]["limits"]["overrides"]["tasks.charters"] = 16.into();
    value["engine"]["limits"]["overrides"]["core.templates"] = 7.into();
    // The standing charters, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["authority.executors"] = 22.into();
    // The standing charters, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["tasks.executor_kinds"] = 22.into();
    value["engine"]["limits"]["overrides"]["documents.lines"] = 12.into();
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

/// Write a test's immutable setup before any forge command reads it.
pub fn setup(directory: &Path, forge: u16, mut value: serde_json::Value) {
    use std::os::unix::fs::PermissionsExt;
    let kind = value["kind"].as_str().expect("setup kind");
    let prefix = if kind == "fake" {
        "playground"
    } else {
        assert_eq!(kind, "owned");
        "forge"
    };
    let directory = directory.join(format!("{prefix}-{forge}"));
    temper_shell::directories::make_private(&directory, skein_shell::effective_user())
        .expect("private setup directory");
    value["version"] = 1.into();
    let path = directory.join("setup");
    std::fs::write(&path, serde_json::to_vec(&value).expect("setup JSON")).expect("setup file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("private setup file");
}

/// A deployment without a forge; setup files belong to the test that starts one.
#[must_use]
pub fn check_document(directory: &Path) -> serde_json::Value {
    let (_, bytes) = documents(directory, "/fixture/temper");
    serde_json::from_slice(&bytes).expect("fixture JSON")
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
