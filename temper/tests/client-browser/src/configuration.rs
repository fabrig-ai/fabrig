//! The process tier's deployment, copied: `documents` from `bin/temper-box/tests/support/configuration.rs`
//! and `write` from `support::configuration` with `Forge::Unreached`, which are test modules of package
//! `temper` and cannot be imported. The LLM endpoint is unreached at `127.0.0.1:9`; no forge is kept
//! until connected; `application.clients.bundle` names the bundle; and every relative path is made absolute
//! under the scratch, as jig's `tests/client-browser` does, since `temper serve` joins paths to its working
//! directory and the harness starts it in the test's.
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use temper_playground::documents::turn_bounds;

/// A closed loopback port: the forge's and the LLM endpoint's, which no story reaches.
pub(crate) const UNREACHED: &str = "127.0.0.1:9";
/// The paths the deployment names, each relative to its directory.
const RELATIVE: &[&str] = &[
    "secrets/worker-a.digest",
    "app.secret",
    "worker.secret",
    "secrets/forge-1.user",
    "secrets/forge-1.api-token",
    "secrets/forge-1.git-token",
    "state",
    "worker-root",
];

pub(crate) fn documents(directory: &Path, program: &Path, bundle: &Path) -> (Box<[u8]>, Box<[u8]>) {
    let agent = serde_json::json!({
        "profile":{"name":"standard","declared":{"memory":2_147_483_648_u64,"tool_payload":2048,"read_window":256,
            "conversations":1,"calls_per_response":1},
            "policy":{"max_turns":800,"group_stop":10,"inbox":8,"unacknowledged":8,"check_timeout":60000}},
        "endpoints":[{"name":"fake","number":1,"dialect":3,"provider":"openai",
            "address":UNREACHED,"transport":"tls","server_name":"skein.test","trust_der":directory.join("root.der"),
            "connect":1000,"handshake":1000,
            "limits": {"window":1_048_576,"output":8192,"reasoning_item":64,"head":60000,"idle":30000}}],
        "environment":[]});
    let mut value: serde_json::Value =
        serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("whole object").remove("service");
    for name in ["views.watchers", "views.watchers_per_party"] {
        value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides").remove(name);
    }
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine object").remove(member);
    }
    for project in value["engine"]["projects"].as_array_mut().expect("seed projects") {
        project["policy"] = 1.into();
        project.as_object_mut().expect("seed").remove("roles");
    }
    temper_playground::documents::settings::strip_models(&mut value);
    // Admit stock policy 1's fourteen grants and fifteen executor kinds.
    for (name, count) in [
        ("core.policy_bytes", 8192),
        // Holds charter 5's resolved models, shared instructions and four parts.
        ("core.run_bytes", 16_384),
        ("authority.grants", 16),
        // The standing charter, closer procedure and close job are delegation targets.
        ("authority.executors", 18),
        ("tasks.authority_grants", 16),
        // The standing charter, closer procedure and close job are delegation targets.
        ("tasks.executor_kinds", 18),
        ("tasks.authority_bytes", 128),
        ("notes.scopes", 32),
        // Nine slots hold the stock charters.
        ("tasks.charters", 10),
        ("core.templates", 6),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    // A Create member owns stock's grants and executors, plus its words.
    value["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 2048.into();
    let worker: serde_json::Value =
        serde_json::from_slice(jig_channel_kit::fixtures::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"]["program"] = program.to_str().expect("program path").into();
    value["workers"][0]["agent"]["configuration"] = directory.join("agent.json").to_str().expect("agent path").into();
    value["workers"][0]["agent"]["arguments"] = serde_json::json!(["internal", "agent", directory.join("agent.json")]);
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 2046.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    value["engine"]["llm"]["endpoints"][0]["dialect"] = 3.into();
    value["engine"]["llm"]["endpoints"][0]["address"] = serde_json::json!({"socket":UNREACHED});
    value["engine"]["limits"]["overrides"]["core.load_slots"] = 8.into();
    value["engine"]["limits"]["overrides"]["documents.rows"] = 2.into();
    // The creation offer carries nine stock charters and four procedures.
    value["engine"]["limits"]["overrides"]["documents.lines"] = 9.into();
    value["engine"]["limits"]["overrides"]["documents.text_bytes"] = 128.into();
    value["engine"]["limits"]["overrides"]["people.sign_ins"] = 2.into();
    value["application"] = serde_json::json!({
        "welcome":true,
        "webhooks":null,
        "clients":{"listener":{"socket":"127.0.0.1:0"},"public":null,"bundle":bundle},
        "machine":{"github":null},"demo":null,"sign_in":null,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,
            "checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":131_072}},
        "workers":[{"root":"worker-root","git":"/usr/bin/git"}]});
    turn_bounds::apply(&mut value, turn_bounds::turn_body(&agent));
    absolute_paths(&mut value, directory);
    (
        serde_json::to_vec(&agent).expect("agent JSON").into_boxed_slice(),
        serde_json::to_vec(&value).expect("deployment JSON").into_boxed_slice(),
    )
}
/// Every string that is one of the deployment's relative paths becomes that path under `directory`.
fn absolute_paths(value: &mut serde_json::Value, directory: &Path) {
    match value {
        serde_json::Value::String(text) if RELATIVE.contains(&text.as_str()) => {
            directory.join(&*text).to_str().expect("fixture path").clone_into(text);
        }
        serde_json::Value::Array(items) => {
            for item in items {
                absolute_paths(item, directory);
            }
        }
        serde_json::Value::Object(items) => {
            for item in items.values_mut() {
                absolute_paths(item, directory);
            }
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {}
    }
}
pub(crate) fn private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("fixture file");
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
}

/// The deployment's files and its two documents, `agent.json` and `temper.json`, in `directory`.
pub(crate) fn write(directory: &Path, program: &Path, bundle: &Path) {
    assert!(Path::new("/usr/bin/git").is_file(), "/usr/bin/git is required");
    for name in ["", "accounts", "secrets", "state"] {
        fs::create_dir_all(directory.join(name)).expect("deployment directory");
        fs::set_permissions(directory.join(name), fs::Permissions::from_mode(0o700)).expect("private directory");
    }
    for (name, bytes) in jig_channel_kit::fixtures::files(false).entries.into_boxed() {
        if name.starts_with(b"charters/") {
            continue;
        }
        private(&directory.join(std::str::from_utf8(&name).expect("fixture path")), bytes.expose());
    }
    private(&directory.join("worker.secret"), b"worker-secret");
    private(&directory.join("secrets/forge-1.user"), b"42\n");
    private(&directory.join("secrets/forge-1.api-token"), b"forge-1-api-token-value");
    private(&directory.join("secrets/forge-1.git-token"), b"forge-1-git-token-value");
    private(&directory.join("root.der"), skein_tls_kit::pki::ROOT);
    let (agent, value) = documents(directory, program, bundle);
    private(&directory.join("agent.json"), &agent);
    private(&directory.join("temper.json"), &value);
}

/// The three directories share the scratch root, as the process tier's flags do.
pub(crate) fn flags(directory: &Path) -> Vec<std::ffi::OsString> {
    ["--config", "--data", "--cache"].into_iter().flat_map(|flag| [flag.into(), directory.into()]).collect()
}

/// Let init write its first-run deployment, then name the gate's bundle and reserve every loopback port.
pub(crate) fn demo(directory: &Path, program: &Path, bundle: &Path) {
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).expect("private demo directory");
    let initialized = std::process::Command::new(program)
        .arg("init")
        .arg("--box")
        .args(flags(directory))
        .output()
        .expect("initialize the demo");
    assert!(initialized.status.success(), "demo init: {}", String::from_utf8_lossy(&initialized.stderr));
    let path = directory.join("temper.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("demo deployment")).expect("JSON");
    let sockets = [
        &value["application"]["clients"]["listener"]["socket"],
        &value["application"]["machine"]["fake"]["listen"]["socket"],
        &value["application"]["demo"]["llm"]["listen"]["socket"],
        &value["application"]["webhooks"]["listener"]["socket"],
    ];
    let addresses = temper_fixtures::ports::addresses::<4>(&[]);
    let replacements = std::array::from_fn::<_, 4, _>(|index| {
        (sockets[index].as_str().expect("demo socket").to_owned(), addresses[index].to_string())
    });
    replace_sockets(&mut value, &replacements);
    value["application"]["clients"]["bundle"] = serde_json::json!(bundle);
    private(&path, &serde_json::to_vec(&value).expect("demo JSON"));
    let path = directory.join("agent.json");
    let mut agent = serde_json::from_slice(&fs::read(&path).expect("demo agent")).expect("JSON");
    replace_sockets(&mut agent, &replacements);
    private(&path, &serde_json::to_vec(&agent).expect("demo agent JSON"));
}

/// Replace the turns of exactly one request offered by the demo.
pub(crate) fn demo_asked(directory: &Path, cue: &str, turns: serde_json::Value) {
    let path = directory.join("temper.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("demo deployment")).expect("JSON");
    let scripts = value["application"]["demo"]["llm"]["asked"].as_array_mut().expect("asked scripts");
    let script = scripts.iter_mut().find(|script| script["cue"] == cue).expect("offered demo request");
    script["turns"] = turns;
    private(&path, &serde_json::to_vec(&value).expect("scripted demo JSON"));
}

/// Public reading's demo alone (temper/docs/design/protocol/sign-in.md, 2.4;
/// jig/docs/design/domain/people/public-reading.md, 7). Its two-second freshness is the walk's subject.
pub(crate) fn public_demo(directory: &Path, program: &Path, bundle: &Path) {
    demo(directory, program, bundle);
    let path = directory.join("temper.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("public demo deployment")).expect("JSON");
    let overrides = value["engine"]["limits"]["overrides"].as_object_mut().expect("demo overrides");
    for (name, figure) in [
        ("clients.public.rate", 20),
        ("clients.public.rate_burst", 20),
        ("clients.public.bytes", 2_097_152),
        ("clients.public.bytes_burst", 2_097_152),
        // The 20/20/50 board reserves 730,728 bytes; this tier's approved ceiling is 1 MiB.
        ("clients.public.answer_bytes", 1_048_576),
        ("clients.public.reads", 8),
        ("clients.public.max_age", 2),
        ("clients_http.public_calls", 8),
        ("core.public.goals", 20),
        ("core.public.ended", 20),
        ("core.public.lines", 50),
        ("core.public.depth", 16),
    ] {
        overrides.insert(name.into(), figure.into());
    }
    let mut delegates = 0_u32;
    for script in value["application"]["demo"]["llm"]["asked"].as_array_mut().expect("asked scripts") {
        if script["cue"] != "Add CSV export to widget" {
            continue;
        }
        for turn in script["turns"].as_array_mut().expect("CSV turns") {
            for line in turn["lines"].as_array_mut().expect("CSV lines") {
                if line["call"] == "delegate" {
                    for member in line["arguments"]["batch"].as_array_mut().expect("delegate batch") {
                        // delegate's host-tool spelling of tracked priority 1.
                        member["tracked"] = serde_json::json!({"priority": 1});
                        delegates = delegates.checked_add(1).expect("bounded script");
                    }
                }
            }
        }
    }
    assert_eq!(delegates, 1, "one CSV delegate becomes a public goal");
    private(&path, &serde_json::to_vec(&value).expect("public demo JSON"));
}

/// Update references to a reserved listener in both documents, including the forge's git URL.
fn replace_sockets(value: &mut serde_json::Value, replacements: &[(String, String)]) {
    match value {
        serde_json::Value::String(text) => {
            for (before, after) in replacements {
                if text == before {
                    text.clone_from(after);
                    break;
                }
                if text.strip_prefix("http://") == Some(before.as_str()) {
                    *text = format!("http://{after}");
                    break;
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                replace_sockets(item, replacements);
            }
        }
        serde_json::Value::Object(items) => {
            for item in items.values_mut() {
                replace_sockets(item, replacements);
            }
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {}
    }
}

/// Derive the stock charters and their model bounds from the edited deployment.
#[cfg(test)]
pub(crate) fn stock_engine(value: &serde_json::Value) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        &serde_json::to_vec(&value["engine"]).expect("engine JSON"),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("engine document");
    temper_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working()).expect("stock engine")
}

#[cfg(test)]
mod tests {
    use super::stock_engine;

    #[test]
    fn the_picker_fixture_admits_stock_models_for_every_charter() {
        let directory = std::path::Path::new("/fixture");
        let (agent, value) = super::documents(directory, &directory.join("temper"), &directory.join("bundle"));
        let agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
        let value: serde_json::Value = serde_json::from_slice(&value).expect("deployment JSON");
        let engine = stock_engine(&value);
        let core = jig_protocol_config::core(&engine, [0; 16], 1);
        let defaults = &core.defaults;
        let bound = &agent["endpoints"][0]["limits"];
        for charter in &defaults.charters {
            let resolved =
                jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core)
                    .expect("stock charter resolves");
            for model in std::iter::once(&resolved.policy.model).chain(resolved.policy.alternatives.iter()) {
                assert!(bound["window"].as_u64().expect("window") >= u64::from(model.window));
                assert!(bound["output"].as_u64().expect("output") >= u64::from(model.output));
            }
        }
    }

    #[test]
    fn the_browser_fixture_admits_every_stock_charter() {
        let directory = std::path::Path::new("/fixture");
        let (agent, bytes) = super::documents(directory, &directory.join("temper"), &directory.join("bundle"));
        let value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
        let engine = stock_engine(&value);
        let agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
        let turns = agent["profile"]["policy"]["max_turns"].as_u64().expect("agent turn allowance");
        for charter in &engine.charters {
            assert!(u64::from(charter.policy.turns) <= turns, "charter {} exceeds agent turns", charter.number);
        }
        assert_eq!(engine.charters.len(), 10);
        assert_eq!(
            usize::try_from(engine.limits.core.documents.lines).expect("line capacity"),
            engine.charters.iter().filter(|charter| charter.number != 10).count()
        );
    }
}
