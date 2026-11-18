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
/// Leave the agent's carriers to the product's stock, derivations, overrides order
/// (temper/docs/design/protocol/configuration/derived.md, 4 and 5.1).
pub fn use_derived(document: &mut temper_playground::documents::Configuration) {
    document.use_stock_carriers();
}

// Eight native exchanges beside Small's two event streams (jig/docs/design/protocol/client/default-http.md, 5).
const NATIVE_HTTP_CALLS: u32 = 10;
#[must_use]
pub fn documents(directory: &Path, program: &str) -> (Vec<u8>, Vec<u8>) {
    let (agent, value) = typed_documents(directory, program);
    (agent, serde_json::to_vec(&value).expect("deployment JSON"))
}
#[must_use]
pub fn typed_documents(directory: &Path, _program: &str) -> (Vec<u8>, temper_playground::documents::Configuration) {
    let mut value = temper_playground::documents::Configuration::from_jig(
        serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("engine JSON"),
    );
    value.as_object_mut().expect("whole object").remove("service");
    use_stock(&mut value);
    // A Create member owns stock's fourteen grants and eighteen executors, plus its words.
    value.jig_over("people.amendment_bytes", 2048);
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_kit::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value.jig_over("tasks.inbox_bytes", 4096);
    value.jig_over("tasks.run_inbox", 8);
    value.jig_over("tasks.run_inbox", 8);
    value.jig_over("hub.inbox", 8);
    value.jig_over("hub.held", 8);
    value.jig_over("tasks.result_bytes", 512);
    for worker in value["workers"].as_array_mut().expect("workers") {
        let worker = worker.as_object_mut().expect("worker");
        worker.remove("memory_bytes");
        worker.remove("graces");
        worker.remove("programs");
    }
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0].as_object_mut().expect("worker").remove("agent");
    value["workers"][0].as_object_mut().expect("worker").remove("limits");
    value.jig_over("hub.accounts", 4);
    value.jig_over("hub.held", 8);
    value["workers"][0]["name"] = "local".into();
    value.jig_over("fleet.slots", 4);
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["llm"]["endpoints"] = serde_json::json!([{
        "name":"fake","number":1,"dialect":3,"provider":"openai",
        "address":{"socket":"127.0.0.1:0"},"transport":"tls","server_name":"skein.test",
        "trust":directory.join("root.der"),"connect_ms":1000,"handshake_ms":1000,"headers":[]
    }]);
    // The stock carries twelve charters, fourteen grants and twenty-two executor kinds.
    value.jig_over("tasks.charters", 16);
    value.jig_over("core.templates", 7);
    // The standing charters, closer procedure and close job are delegation targets.
    value.jig_over("authority.executors", 22);
    // The standing charters, closer procedure and close job are delegation targets.
    value.jig_over("tasks.executor_kinds", 22);
    value.jig_over("authority.grants", 16);
    // A chat copies stock policy 1's fourteen grants into its task authority.
    value.jig_over("tasks.authority_grants", 16);
    // Fourteen literal "forge" segments need seventy bytes, beyond the kit fixture's 64.
    value.jig_over("tasks.authority_bytes", 128);
    value.jig_over("notes.scopes", 32);
    value.jig_over("core.policy_bytes", 8192);
    value.jig_over("core.load_slots", 8);
    value.jig_over("documents.rows", 2);
    // The creation offer carries nine stock charters and four procedures.
    value.jig_over("documents.lines", 12);
    value.jig_over("documents.text_bytes", 128);
    value.jig_over("people.sign_ins", 2);
    for name in ["clients.streams", "clients_http.streams"] {
        value.jig_over(name, 2);
    }
    for name in ["clients.streams", "clients_http.streams"] {
        value.jig_over(name, 2);
    }
    for name in ["clients.exchanges", "clients_http.calls"] {
        value.jig_over(name, NATIVE_HTTP_CALLS);
    }
    // Projects 0 and 1, and the one forge-setup makes for the first repository.
    value.jig_over("people.projects", 3);
    value.jig_over("authority.projects", 3);
    value.jig_over("tasks.funders", 4);
    // The journal's charter carrier matches this fixture's core.run_bytes.
    value["application"] = serde_json::json!({
        "webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:0"},"public":null,"bundle":null},
        "demo":null,"sign_in":null});
    value.temper_over("forge.client.answer_bytes", 65_536_u64);
    value.temper_over("forge.client.op_bytes", 512_u64);
    value.temper_over("forge_component.body_bytes", 16_384_u64);
    value.temper_over("root.journal.run_bytes", 32_768_u64);
    value.temper_over("forge.name_bytes", 64_u64);
    value.temper_over("forge.judge_projects", 3_u64);
    value.temper_over("forge.landing_members", 3_u64);
    value.temper_over("forge.resources_per_task", 4_u64);

    let mut agent = temper_agent_fixtures::configuration(&value);
    agent["profile"]["policy"]["check_timeout"] = 60_000.into();
    agent["endpoints"][0]["limits"]["idle"] = 30_000.into();
    agent["programs"] = serde_json::json!({});
    value.jig_over("hub.run_calls", agent["profile"]["declared"]["calls_per_response"].as_u64().expect("agent calls"));
    value.jig_over("hub.inbox", agent["profile"]["policy"]["inbox"].as_u64().expect("agent inbox"));
    use_derived(&mut value);
    turn_bounds::apply(&mut value, turn_bounds::turn_body(&agent));
    (serde_json::to_vec(&agent).expect("agent JSON"), value)
}

/// Keep a copied fixture's startup members and use temper's stock settings.
pub fn use_stock(value: &mut temper_playground::documents::Configuration) {
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine object").remove(member);
    }
    temper_playground::documents::settings::strip_models(value);
    value.jig_over("authority.roles", 4);
    value.jig_over("core.run_bytes", 32_768);
    value.jig_over("tasks.charters", 16);
    value.jig_over("core.templates", 7);
    // The standing charters, closer procedure and close job are delegation targets.
    value.jig_over("authority.executors", 22);
    // The standing charters, closer procedure and close job are delegation targets.
    value.jig_over("tasks.executor_kinds", 22);
    value.jig_over("documents.lines", 12);
}

/// Check startup and its in-memory construction without opening a store or running a service.
pub fn admission(directory: &Path, configuration: &temper_playground::documents::Configuration) -> Result<(), String> {
    use skein_lib::{List, Secret};
    use temper_service::{settings, startup};
    let directories = temper_service_shell::jig_directories(&directories(directory));
    let bytes = serde_json::to_vec(configuration).expect("fixture JSON");
    let text = |problems: startup::Problems| String::from_utf8_lossy(&startup::lines(&problems)).into_owned();
    let named = startup::named_with_limits(
        &bytes,
        &directories,
        &temper_service::stock::Given::Stock,
        &configuration.jig,
        &configuration.limits,
    )
    .map_err(text)?;
    let files =
        temper_service_shell::read_files_with_asked(named.files, named.asked_files, skein_shell::effective_user())?;
    let agent = temper_agent_fixtures::prepared(directory).agent;
    let seeds = settings::Seeds {
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
        link_secret: None,
        local_secret: None,
    };
    let mut checked = startup::checked_start_with_limits(
        &bytes,
        &directories,
        &files,
        Some(&agent),
        &seeds,
        List::with_capacity(0),
        None,
        &temper_service::stock::Given::Stock,
        None,
        &configuration.jig,
        &configuration.limits,
    )
    .map_err(text)?;
    let runtime = temper_service::WorkerRuntime {
        wires: None,
        root: skein_io::kernel::Fd::new(9),
        path: b"/worker".as_slice().into(),
        agent: temper_playground::documents::pinned_agent(),
    };
    let accounts = temper_engine_shell::serve::accounts(&checked.engine_mut().jig)?;
    startup::from_checked(
        checked,
        &files,
        Some(&agent),
        &seeds,
        Some(accounts),
        Some(runtime),
        None,
        None,
        Box::new([]),
    )
    .map_err(text)?;
    Ok(())
}

/// The cluster's engine figures; workers receive them from their engine.
#[expect(clippy::too_many_lines, reason = "the three subprocess worlds share one retained numeric fixture")]
pub fn cluster_limits(value: &mut temper_playground::documents::Configuration, turn: bool, github: bool) {
    for (name, figure) in [
        ("authority.executors", 22),
        ("authority.grants", 15),
        ("authority.projects", 2),
        ("authority.requirements", 3),
        ("authority.roles", 4),
        ("authority.segments", 8),
        ("brief.briefs", 2),
        ("brief.sections", 10),
        ("core.call_answer_bytes", 4096),
        ("core.load_bytes", 8192),
        ("core.load_slots", 8),
        ("core.policy_bytes", 8192),
        ("core.resume_bytes", 4_194_304),
        ("core.run_bytes", 32_768),
        ("core.templates", 7),
        ("documents.chips", 2),
        ("documents.lines", 12),
        ("documents.rows", 32),
        ("documents.text_bytes", 128),
        ("fleet.attempts", if turn { 4 } else { 2 }),
        ("fleet.calls", 1),
        ("fleet.turns", 8),
        ("hub.transcript_bytes", 16_777_216),
        ("hub.turn_bytes", 1_327_470),
        ("hub.turn_queue_bytes", 10_619_760),
        ("hub.turns", 1),
        ("io.output", if turn { 1_327_857 } else { 1_328_262 }),
        ("link.ping_ms", 100),
        ("link.silence_ms", 1000),
        ("notes.entries_per_scope", 1),
        ("notes.party_entries", 1),
        ("notes.recalled", 1),
        ("notes.scopes", 18),
        ("people.amendment_bytes", 2048),
        ("people.holdings", 1),
        ("people.inbox_tasks", 16),
        ("people.operations", 5),
        ("people.pending", 4),
        ("people.people", 1),
        ("people.projects", 2),
        ("people.sign_ins", 2),
        ("people.waiters", 4),
        ("people.words", 512),
        ("tasks.authority_bytes", 512),
        ("tasks.authority_grants", 15),
        ("tasks.authority_segments", 9),
        ("tasks.batch", 1),
        ("tasks.charters", 16),
        ("tasks.contract_choices", 2),
        ("tasks.delegates", 4),
        ("tasks.dependencies", 1),
        ("tasks.executor_kinds", 22),
        ("tasks.exemptions", 8),
        ("tasks.funders", 4),
        ("tasks.hold_bytes", 64),
        ("tasks.hold_segments", 8),
        ("tasks.message_bytes", 4094),
        ("tasks.parameters", 5),
        ("tasks.project_goals", 1),
        ("tasks.project_tasks", 8),
        ("tasks.result_bytes", 128),
        ("tasks.saved_resources", 4),
        ("tasks.spec_bytes", 512),
        ("tasks.tasks", 8),
        ("tasks.inbox_bytes", 4096),
        ("tasks.run_inbox", 8),
        ("tasks.tree_tasks", 8),
        ("views.runs", if turn { 2 } else { 1 }),
        ("views.text_bytes", 1024),
    ] {
        value.jig_over(name, figure);
    }
    let grace = temper_service::startup::worker_fleet_grace(&value.jig);
    value.jig_over("fleet.grace_ms", grace.as_millis());
    if !github {
        value.jig_over("fleet.engine_slots", 0);
    }
    for (name, figure) in [
        ("checkout.message_bytes", 4096),
        ("checkout.name_bytes", 256),
        ("checkout.repositories", 2),
        ("forge.landing_members", 1),
        ("forge.name_bytes", 64),
        ("forge.resources_per_task", 2),
        ("git_component.stdout_bytes", 131_072),
        ("root.journal.transcript_bytes", 4_194_304),
        ("root.loads.bytes", 1_327_718),
        ("root.loads.read_bytes", 2_654_940),
        ("root.loads.reply_bytes", 1_327_718),
        ("root.loads.transcript_bytes", 1_327_470),
        ("tool_payload", 4096),
        ("transcript_memory", 4_503_599_627_370_496_u64),
    ] {
        value.temper_over(name, figure);
    }
    if !github {
        value.temper_over("forge.client.calls", 4);
        value.temper_over("forge.client.pending", 8);
        value.temper_over("forge_component.connections", 4);
    }
    if turn {
        value.temper_over("forge.client.poll_ms", 50);
    }
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
            "connect_ms":10_000, "handshake_ms":10_000}]
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
