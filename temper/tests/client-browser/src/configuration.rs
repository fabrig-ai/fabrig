//! The browser tier's typed deployment follows `testing/temper-box-kit/src/configuration.rs`
//! with `Forge::Unreached`. The LLM endpoint is unreached at `127.0.0.1:9`; no forge is kept
//! until connected; `application.clients.bundle` names the bundle; and every relative path is made absolute
//! under the scratch, as jig's `tests/client-browser` does, since `temper serve` joins paths to its working
//! directory and the harness starts it in the test's. The first-run demo reserves every listener init wrote
//! through `temper_fixtures::ports` (temper/docs/design/testing/process-tier.md, Three at a time).
//! Ctl commits the world's kept providers before a restart (temper/docs/design/protocol/configuration/providers.md, 3, 6).
use std::{fs, net::SocketAddr, os::unix::fs::PermissionsExt, path::Path};
use temper_playground::documents::turn_bounds;

/// A closed loopback port: the forge's and the LLM endpoint's, which no story reaches.
pub(crate) const UNREACHED: &str = "127.0.0.1:9";
/// Keep the stock identities: stock's three endpoints and the box's one fill the agent's four slots.
pub(crate) const PROVIDER: u16 = 1;
pub(crate) const WELCOME_PROVIDER: u16 = 2;
pub(crate) const ENDPOINT: u16 = 1;
pub(crate) const WELCOME_ENDPOINT: u16 = 2;
/// The paths the deployment names, each relative to its directory.
const RELATIVE: &[&str] = &[
    "app.secret",
    "worker.secret",
    "secrets/forge-1.user",
    "secrets/forge-1.api-token",
    "secrets/forge-1.git-token",
    "state",
    "workspaces",
];

pub(crate) fn typed_documents(
    directory: &Path,
    bundle: &Path,
) -> (Box<[u8]>, temper_playground::documents::Configuration) {
    let mut value = temper_playground::documents::Configuration::from_jig(
        serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("engine JSON"),
    );
    value.as_object_mut().expect("whole object").remove("service");
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine object").remove(member);
    }
    value["world_providers"] =
        value["engine"].as_object_mut().expect("engine").remove("llm").expect("world provider inputs");
    temper_playground::documents::settings::strip_models(&mut value);
    // Admit stock policy 1's fourteen grants and fifteen executor kinds.
    for (name, count) in [
        ("core.policy_bytes", 8192),
        // Holds charter 5's resolved models, shared instructions and four parts.
        ("core.run_bytes", 32_768),
        ("authority.grants", 16),
        // The standing charters, closer procedure and close job are delegation targets.
        ("authority.executors", 22),
        ("tasks.authority_grants", 16),
        // The standing charters, closer procedure and close job are delegation targets.
        ("tasks.executor_kinds", 22),
        ("tasks.authority_bytes", 128),
        ("notes.scopes", 32),
        // Nine slots hold the stock charters.
        ("tasks.charters", 16),
        ("core.templates", 7),
    ] {
        value.jig_over(name, count);
    }
    // A Create member owns stock's grants and executors, plus its words.
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
    value["workers"][0]["name"] = "local".into();
    value["workers"][0].as_object_mut().expect("worker").remove("agent");
    value["workers"][0].as_object_mut().expect("worker").remove("limits");
    value.jig_over("hub.accounts", 4);
    value.jig_over("hub.event_bytes", 2046);
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    temper_agent_fixtures::providers::input(&mut value)["endpoints"][0]["dialect"] = 3.into();
    let providers = temper_agent_fixtures::providers::input(&mut value);
    providers["providers"][0]["number"] = PROVIDER.into();
    providers["providers"][0]["name"] = "openai".into();
    providers["providers"][0]["endpoint"] = "openai".into();
    providers["endpoints"][0]["provider"] = "openai".into();
    providers["endpoints"][0]["name"] = "openai".into();
    temper_agent_fixtures::providers::input(&mut value)["endpoints"][0]["number"] = ENDPOINT.into();
    temper_agent_fixtures::providers::input(&mut value)["endpoints"][0]["address"] =
        serde_json::json!({"socket":UNREACHED});
    value.jig_over("core.load_slots", 8);
    value.jig_over("documents.rows", 2);
    // The creation offer carries nine stock charters and four procedures.
    value.jig_over("documents.lines", 11);
    value.jig_over("documents.text_bytes", 128);
    value.jig_over("people.sign_ins", 2);
    value["application"] = serde_json::json!({
        "webhooks":null,
        "clients":{"listener":{"socket":"127.0.0.1:0"},"public":null,"bundle":bundle},
        "sign_in":null});
    value.temper_over("forge.client.answer_bytes", 65_536_u64);
    value.temper_over("forge.client.op_bytes", 512_u64);
    value.temper_over("forge_component.body_bytes", 16_384_u64);
    value.temper_over("checkout.name_bytes", 256_u64);
    value.temper_over("checkout.repositories", 2_u64);
    value.temper_over("checkout.message_bytes", 4096_u64);
    value.temper_over("forge.name_bytes", 64_u64);
    value.temper_over("forge.landing_members", 1_u64);
    value.temper_over("forge.resources_per_task", 2_u64);
    value.temper_over("git_component.stdout_bytes", 131_072_u64);

    let mut agent = temper_agent_fixtures::configuration(&value);
    agent["profile"]["policy"]["check_timeout"] = 60_000.into();
    agent["endpoints"][0]["limits"]["idle"] = 30_000.into();
    agent["programs"] = serde_json::json!({});
    value.jig_over("hub.run_calls", agent["profile"]["declared"]["calls_per_response"].as_u64().expect("agent calls"));
    value.jig_over("hub.inbox", agent["profile"]["policy"]["inbox"].as_u64().expect("agent inbox"));
    value.use_stock_carriers();
    turn_bounds::apply(&mut value, turn_bounds::turn_body(&agent));
    absolute_paths(&mut value, directory);
    (serde_json::to_vec(&agent).expect("agent JSON").into_boxed_slice(), value)
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

fn keep_providers(directory: &Path, document: &serde_json::Value) {
    if let Some(wanted) = document.get("world_providers") {
        let providers: Vec<_> =
            temper_box_kit::provider_setup::items(wanted).into_iter().map(|item| item["set"].clone()).collect();
        private(
            &directory.join("world-providers.json"),
            &serde_json::to_vec(&serde_json::json!({"providers":providers})).expect("world inputs"),
        );
        let settings = directory.join("browser-agents.json");
        if settings.is_file() {
            let mut value = serde_json::from_slice(&fs::read(&settings).expect("ctl settings")).expect("settings JSON");
            provider_settings(directory, &mut value);
            private(&settings, &serde_json::to_vec(&value).expect("agents and providers JSON"));
        }
    }
}

/// The deployment and its prospective agents and providers, applied by ctl before the ordinary stories.
pub(crate) fn write(directory: &Path, bundle: &Path) -> temper_playground::documents::Configuration {
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
    let (agent, mut value) = typed_documents(directory, bundle);
    let agent = serde_json::from_slice(&agent).expect("agent JSON");
    turn_bounds::apply(&mut value, turn_bounds::turn_body(&agent));
    let settings = agent_settings(&agent);
    private(&directory.join("browser-agents.json"), &serde_json::to_vec(&settings).expect("agents JSON"));
    persist(directory, &value);
    value
}

/// Keep the typed figures beside the document for the owner's setup sitting.
pub(crate) fn persist(directory: &Path, value: &temper_playground::documents::Configuration) {
    temper_box_kit::limit_settings::remember_figures(directory, value);
    keep_providers(directory, value);
    let mut file = if directory.join(temper_config_shell::FILE).is_file() {
        read_init(directory).file
    } else {
        temper_config_shell::File {
            owner: Some("owner-a@localhost".into()),
            webhooks: Some("127.0.0.1:0".parse().expect("webhooks")),
            ..Default::default()
        }
    };
    file.listen =
        Some(value["application"]["clients"]["listener"]["socket"].as_str().expect("listen").parse().expect("socket"));
    file.internal.bundle = value["application"]["clients"]["bundle"].as_str().map(Into::into);
    file.test.github =
        value["application"]["test"]["github"].as_object().map(|github| temper_config_shell::TestGitHub {
            api: github["api"]["socket"].as_str().expect("fake API socket").into(),
            transport: temper_config_shell::Transport::Plaintext,
            remote: github["remote"].as_str().expect("fake remote").into(),
            web: github["web"].as_str().expect("fake web").into(),
            git: None,
        });
    write_file(directory, &file);
}

pub(crate) fn write_file(directory: &Path, file: &temper_config_shell::File) {
    private(
        &directory.join(temper_config_shell::FILE),
        &temper_config_shell::render(file, temper_config_shell::Shape::Standalone),
    );
}

/// Restore the start's document from the file, and the world's typed limits from its setup inputs.
pub(crate) fn configuration(directory: &Path) -> temper_playground::documents::Configuration {
    let mut value = temper_playground::documents::Configuration::stock(
        serde_json::from_slice(&read_init(directory).document).expect("file's start"),
    );
    let path = directory.join("fixture-limits.json");
    if path.is_file() {
        let figures: serde_json::Value =
            serde_json::from_slice(&fs::read(path).expect("fixture limits")).expect("limits JSON");
        for item in figures["limits"].as_array().expect("jig figures") {
            jig_protocol_config::overrides::apply(
                &mut value.jig,
                item["name"].as_str().expect("name").as_bytes(),
                item["value"].as_u64().expect("figure"),
            )
            .expect("jig figure");
        }
        for item in figures["temper-limits"].as_array().expect("temper figures") {
            temper_playground::documents::Configuration::temper_over(
                &mut value,
                item["name"].as_str().expect("name"),
                item["value"].as_u64().expect("figure"),
            );
        }
    }
    value
}

/// Commit the prospective settings in a no-task sitting, then let the browser's start take them.
/// Providers and agent endpoints take effect at the next start, not at the decision.
pub(crate) fn install(directory: &Path, program: &Path) {
    use temper_box_kit::ctl::{Call, Step};
    let settings = directory.join("browser-agents.json");
    if !settings.is_file() || directory.join("browser-settings-applied").is_file() {
        return;
    }
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&settings).expect("settings")).expect("JSON");
    let figures: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("fixture-limits.json")).expect("figures")).expect("JSON");
    value["items"].as_array_mut().expect("items").extend(limit_items(&figures));
    let client = directory.join("settings-client");
    fs::create_dir(&client).expect("settings client");
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).expect("private client");
    let mut steps = vec![Step::Run(Call::sign_in().by(program))];
    // The seven numbered stock models keep their identities and figures; their endpoint bindings follow settings.
    for number in 1..=7 {
        steps.push(Step::Run(Call::new(&["settings", "show", "model", &number.to_string(), "--json"]).by(program)));
    }
    steps.push(Step::Run(Call::new(&["settings", "show", "--json"]).by(program)));
    let file = settings.clone();
    let executable = program.to_path_buf();
    let wanted = temper_agent_fixtures::providers::read(directory);
    steps.push(Step::Then(Box::new(move |calls| {
        for call in &calls[1..8] {
            let read = temper_box_kit::ctl::json(call);
            let mut model = read["value"].clone();
            bind_model(&mut model, &wanted);
            value["items"]
                .as_array_mut()
                .expect("items")
                .push(serde_json::json!({"setting":read["setting"],"set":model}));
        }
        private(&file, &serde_json::to_vec(&value).expect("settings JSON"));
        temper_box_kit::ctl::settings_change(calls.last().expect("inventory"), &file, "browser-settings")
            .by(&executable)
    })));
    steps.push(Step::Run(Call::new(&["sign-out"]).by(program)));
    let flags = flags(directory).into_iter().map(|word| word.into_string().expect("flag")).collect();
    let seen = temper_box_kit::boxed::run_ctl_from(program, directory, flags, &client, steps, |_| ());
    for call in &seen.calls {
        assert_eq!(
            call.exit,
            Some(skein_io::kernel::Exit::Code(0)),
            "settings setup: {}",
            String::from_utf8_lossy(&call.stderr)
        );
    }
    assert_eq!(seen.exit, Some(skein_io::kernel::Exit::Code(0)), "{}", String::from_utf8_lossy(&seen.stderr));
    assert_eq!(
        seen.calls.len(),
        11,
        "sign in, read seven models and inventory, apply, sign out; {}",
        String::from_utf8_lossy(&seen.stderr)
    );
    assert_eq!(seen.left, Some(Vec::new()), "settings sitting drained");
    private(&directory.join("browser-settings-applied"), b"applied\n");
}

fn limit_items(figures: &serde_json::Value) -> Vec<serde_json::Value> {
    ["limits", "temper-limits"]
        .into_iter()
        .map(|kind| {
            let mut overrides: Vec<_> = figures[kind]
                .as_array()
                .expect("figures")
                .iter()
                .filter(|item| {
                    // The start derives these carriers from its agent, not the composition's document.
                    item["name"] != "core.call_answer_bytes" && item["name"] != "forge.client.answer_bytes"
                })
                .cloned()
                .collect();
            overrides.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
            serde_json::json!({"setting":{"kind":kind},"set":overrides})
        })
        .collect()
}

fn bind_model(value: &mut serde_json::Value, wanted: &serde_json::Value) {
    let number = value["model"]["provider"].clone();
    let Some(provider) =
        wanted["providers"].as_array().expect("providers").iter().find(|provider| provider["number"] == number)
    else {
        return;
    };
    let endpoint = wanted["endpoints"]
        .as_array()
        .expect("endpoints")
        .iter()
        .find(|endpoint| endpoint["provider"] == provider["name"])
        .expect("world endpoint");
    value["model"]["endpoint"] = endpoint["number"].clone();
    value["model"]["dialect"] = endpoint["dialect"].clone();
    value["model"]["program"] = endpoint["program"].clone();
}

/// Keep ctl's prospective providers aligned with the world's kept inputs, including fixture edits.
fn provider_settings(directory: &Path, settings: &mut serde_json::Value) {
    let providers = temper_agent_fixtures::providers::read(directory);
    let items = settings["items"].as_array_mut().expect("settings items");
    items.retain(|item| item["setting"]["kind"] != "provider");
    items.extend(temper_box_kit::provider_setup::items(&providers));
}

/// Persist the prospective startup figures through the owner's settings decision as well
/// (temper/docs/design/draft/config.md, 6.3; protocol/configuration/settings-kinds.md, 1, kind 21).
fn agent_settings(agent: &serde_json::Value) -> serde_json::Value {
    let mut profile = serde_json::Map::new();
    for group in ["declared", "policy"] {
        for (name, value) in agent["profile"][group].as_object().expect("agent profile group") {
            profile.insert(format!("{group}.{name}"), value.clone());
        }
    }
    let mut endpoints = serde_json::Map::new();
    for endpoint in agent["endpoints"].as_array().expect("agent endpoints") {
        let mut limits = endpoint["limits"].clone();
        let object = limits.as_object_mut().expect("agent endpoint limits");
        for (from, to) in [("head", "head_ms"), ("idle", "idle_ms")] {
            let value = object.remove(from).expect("agent endpoint time");
            object.insert(to.into(), value);
        }
        object.remove("oversized_reasoning");
        object.insert("oversized".into(), "fail".into());
        endpoints.insert(endpoint["name"].as_str().expect("endpoint name").into(), limits);
    }
    let fallback = endpoints.values().next().expect("one endpoint").clone();
    serde_json::json!({"items":[{"setting":{"kind":"agents"},"set":{
        "profile":profile,"endpoints":endpoints,"fallback":fallback
    }}]})
}

/// The three directories share the scratch root, as the process tier's flags do.
pub(crate) fn flags(directory: &Path) -> Vec<std::ffi::OsString> {
    ["--config", "--data", "--cache"].into_iter().flat_map(|flag| [flag.into(), directory.into()]).collect()
}

/// Let init write its first-run deployment, then reserve every listener it wrote and name the gate's bundle.
pub(crate) fn demo(directory: &Path, program: &Path, bundle: &Path) {
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).expect("private demo directory");
    let initialized = std::process::Command::new(program)
        .arg("init")
        .arg("--box")
        .args(flags(directory))
        .output()
        .expect("initialize the demo");
    assert!(initialized.status.success(), "demo init: {}", String::from_utf8_lossy(&initialized.stderr));
    let path = directory.join(temper_config_shell::FILE);
    let mut file = read_init(directory).file;
    own_demo_listeners(&mut file, temper_fixtures::ports::addresses::<2>);
    file.internal.bundle = Some(bundle.into());
    private(&path, &temper_config_shell::render(&file, temper_config_shell::Shape::Standalone));
}

pub(crate) fn read_init(directory: &Path) -> temper_config_shell::Read {
    use std::os::unix::ffi::OsStrExt;
    let arguments: Vec<Box<[u8]>> = [
        b"--config".as_slice(),
        directory.as_os_str().as_bytes(),
        b"--data",
        directory.as_os_str().as_bytes(),
        b"--cache",
        directory.as_os_str().as_bytes(),
    ]
    .into_iter()
    .map(Box::from)
    .collect();
    let (dirs, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("demo directories");
    temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file")
}

/// Reserve both init listeners, leaving every other explicit value unchanged.
fn own_demo_listeners(file: &mut temper_config_shell::File, reserve: impl FnOnce(&[SocketAddr]) -> [SocketAddr; 2]) {
    let sockets = [
        file.listen.expect("init listener"),
        file.webhooks.unwrap_or_else(|| {
            let mut address = file.listen.expect("init listener");
            address.set_port(address.port().checked_add(1).expect("init default webhook port"));
            address
        }),
    ];
    let [listen, webhooks] = reserve(&sockets);
    file.listen = Some(listen);
    file.webhooks = Some(webhooks);
}

/// Replace the turns of exactly one request offered by the demo.
pub(crate) fn demo_asked(directory: &Path, cue: &str, turns: serde_json::Value) {
    let mut value: serde_json::Value = serde_json::from_slice(temper_fake::demo::SCRIPTS).expect("embedded scripts");
    let scripts = value["asked"].as_array_mut().expect("asked scripts");
    let script = scripts.iter_mut().find(|script| script["cue"] == cue).expect("offered demo request");
    script["turns"] = turns;
    crate::scripted::write(directory, &value);
}

/// Public reading's demo and its admitted board bounds (temper/docs/design/protocol/sign-in.md, 2.4;
/// jig/docs/design/domain/people/public-reading.md, 7).
pub(crate) fn public_demo(directory: &Path, program: &Path, bundle: &Path) {
    demo(directory, program, bundle);
    public_limits(directory);
}

fn public_limits(directory: &Path) {
    let figures = [
        ("clients.public.rate", 20),
        ("clients.public.rate_burst", 20),
        ("clients.public.bytes", 2_097_152),
        ("clients.public.bytes_burst", 2_097_152),
        // The 20/20/50 board reserves 730,728 bytes; this tier's approved ceiling is 1 MiB.
        ("clients.public.answer_bytes", 1_048_576),
        // A zero-age record expires as soon as the page receives it.
        ("clients.public.max_age", 2),
        ("clients.public.reads", 8),
        ("clients_http.public_calls", 8),
        ("core.public.goals", 20),
        ("core.public.ended", 20),
        ("core.public.lines", 50),
        ("core.public.depth", 16),
    ];
    crate::scripted::limits(directory, &figures);
}

/// Derive the stock charters and their model bounds from the edited deployment.
#[cfg(test)]
pub(crate) fn stock_engine(value: &temper_playground::documents::Configuration) -> jig_protocol_config::Engine {
    let document = jig_protocol_config::document(
        &serde_json::to_vec(&value["engine"]).expect("engine JSON"),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("engine document");
    let mut value = value.clone();
    let given = temper_agent_fixtures::providers::given(&mut value, Path::new("/fixture"), None);
    temper_engine_service::startup::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        &given,
        &mut value.jig.clone(),
    )
    .expect("stock engine")
}

#[cfg(test)]
pub(crate) fn shape(mut value: serde_json::Value) -> temper_service_core::startup::Shape {
    let given = temper_agent_fixtures::providers::given(&mut value, Path::new("/fixture"), None);
    temper_service::startup::shape_with(
        &serde_json::to_vec(&value).expect("deployment JSON"),
        &jig_protocol_config::Directories::working(),
        &given,
    )
    .unwrap_or_else(|problems| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems))))
}

#[cfg(test)]
mod tests {
    use super::stock_engine;

    #[test]
    fn world_settings_preserve_stock_provider_identities() {
        use jig_domain_core::settings::{Current, Edit, Item, Setting, Value};
        let scratch = skein_scratch::Scratch::new("browser-stock-identities");
        let mut value = super::write(scratch.path(), &scratch.path().join("bundle"));
        let engine = stock_engine(&value);
        let core = jig_protocol_config::core(&engine, [0; 16], 1);
        let mut defaults = core.defaults;
        defaults.providers = temper_service::stock::providers();
        let current = Current::new(defaults, &engine.limits.core, &[]);
        for accounts in [false, true] {
            if accounts {
                let sign_in = crate::account_registration("http://127.0.0.1:9");
                crate::account_provider(&mut value, &sign_in);
                crate::welcome_provider(&mut value, &sign_in);
                super::persist(scratch.path(), &value);
            }
            let settings: serde_json::Value =
                serde_json::from_slice(&std::fs::read(scratch.path().join("browser-agents.json")).expect("settings"))
                    .expect("JSON");
            let providers: Vec<_> = settings["items"]
                .as_array()
                .expect("items")
                .iter()
                .filter(|item| item["setting"]["kind"] == "provider")
                .map(|item| item["set"].clone())
                .collect();
            let bytes = serde_json::to_vec(&serde_json::json!({"providers":providers})).expect("provider JSON");
            let items: Vec<_> = temper_agent_fixtures::providers::parse(&bytes)
                .into_vec()
                .into_iter()
                .map(|provider| Item {
                    setting: Setting::Provider(provider.provider.number),
                    edit: Edit::Set(Value::Provider(Box::new(provider))),
                })
                .collect();
            jig_domain_core::settings::providers::check_change(&current, &items)
                .expect("world providers must not rename stock numbers");
        }
        assert!(!scratch.path().join("providers.setting").exists(), "only the serving engine writes its kept copy");
    }

    #[test]
    fn settings_file_uses_the_public_agents_form() {
        let scratch = skein_scratch::Scratch::new("browser-agents-form");
        super::write(scratch.path(), &scratch.path().join("bundle"));
        let bytes = std::fs::read(scratch.path().join("browser-agents.json")).expect("settings");
        settings_form::file::decode(&bytes, &settings_form::Limits::standard())
            .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&why)));
    }

    #[test]
    fn a_file_edit_preserves_explicit_keys_and_the_settings_inputs() {
        let scratch = skein_scratch::Scratch::new("browser-file-edit");
        super::write(scratch.path(), &scratch.path().join("bundle"));
        let mut file = super::read_init(scratch.path()).file;
        file.public = Some("http://localhost:7100".into());
        super::write_file(scratch.path(), &file);
        let mut value = super::configuration(scratch.path());
        value["application"]["clients"]["listener"]["socket"] = "127.0.0.1:42004".into();
        super::persist(scratch.path(), &value);
        assert!(!scratch.path().join(temper_config_shell::FORMER).exists());
        let read = super::read_init(scratch.path());
        assert_eq!(read.file.public, file.public);
        assert_eq!(read.file.listen, Some("127.0.0.1:42004".parse().expect("listener")));
        assert_eq!(read.file.internal.bundle, file.internal.bundle);
        assert_eq!(super::configuration(scratch.path()).jig, value.jig);
        let start: serde_json::Value = serde_json::from_slice(&read.document).expect("start");
        assert!(start.get("world_providers").is_none());
        let settings: serde_json::Value =
            serde_json::from_slice(&std::fs::read(scratch.path().join("browser-agents.json")).expect("settings"))
                .expect("JSON");
        assert!(settings["items"].as_array().expect("items").iter().any(|item| item["setting"]["kind"] == "provider"));
    }

    #[test]
    fn subprocess_figures_survive_document_edits() {
        let scratch = skein_scratch::Scratch::new("browser-figures");
        super::write(scratch.path(), &scratch.path().join("bundle"));
        let mut value = super::configuration(scratch.path());
        assert_eq!(value.jig.core.documents.lines, 11);
        crate::account_provider(&mut value, &crate::account_registration("http://127.0.0.1:9"));
        super::persist(scratch.path(), &value);
        let restored = super::configuration(scratch.path());
        assert_eq!(restored.jig, value.jig);
        assert_eq!(restored.limits, value.limits);
        assert_eq!(restored.limits.git_component.operations, 1);
    }

    #[test]
    fn limit_settings_keep_ordered_small_figures_without_underived_carriers() {
        let scratch = skein_scratch::Scratch::new("browser-limit-form");
        super::write(scratch.path(), &scratch.path().join("bundle"));
        let figures: serde_json::Value =
            serde_json::from_slice(&std::fs::read(scratch.path().join("fixture-limits.json")).expect("figures"))
                .expect("JSON");
        for item in super::limit_items(&figures) {
            let overrides: Vec<_> = item["set"]
                .as_array()
                .expect("overrides")
                .iter()
                .map(|row| jig_domain_core::settings::Override {
                    name: row["name"].as_str().expect("name").as_bytes().into(),
                    value: row["value"].as_u64().expect("figure"),
                    reason: row["reason"].as_str().expect("reason").as_bytes().into(),
                })
                .collect();
            jig_domain_core::settings::check_overrides(&overrides).expect("limits form");
            assert!(overrides.iter().all(|row| row.name.as_ref() != b"core.call_answer_bytes"
                && row.name.as_ref() != b"forge.client.answer_bytes"));
            for row in overrides {
                assert!(
                    figures[item["setting"]["kind"].as_str().expect("kind")]
                        .as_array()
                        .expect("figures")
                        .iter()
                        .any(|original| original["name"].as_str().expect("name").as_bytes() == row.name.as_ref()
                            && original["value"] == row.value),
                    "other small figures stay unchanged"
                );
            }
        }
    }

    #[test]
    fn public_board_reads_have_time_to_draw_before_they_expire() {
        let scratch = skein_scratch::Scratch::new("browser-public-limits");
        super::public_limits(scratch.path());
        let bytes = std::fs::read(scratch.path().join("browser-limits.json")).expect("fixture limit settings");
        let rows: Vec<serde_json::Value> = serde_json::from_slice(&bytes).expect("limit rows");
        let mut limits = jig_protocol_config::stock();
        for row in &rows {
            let name = row["name"].as_str().expect("limit name");
            let value = row["value"].as_u64().expect("limit value");
            jig_protocol_config::overrides::apply(&mut limits, name.as_bytes(), value).expect("public limit");
        }
        assert_eq!(limits.clients.public.max_age, 2);
    }

    #[test]
    fn stock_models_follow_the_world_endpoint_bindings() {
        let mut value = super::typed_documents(std::path::Path::new("/fixture"), std::path::Path::new("/bundle")).1;
        let mut model = serde_json::json!({"number":2,"model":{"provider":1,"endpoint":1,"dialect":2,"program":"claude_code","name":"stock-name","output":8192}});
        super::bind_model(&mut model, &value["world_providers"]);
        assert_eq!(
            model,
            serde_json::json!({"number":2,"model":{"provider":super::PROVIDER,"endpoint":super::ENDPOINT,"dialect":3,"program":null,"name":"stock-name","output":8192}})
        );
        let sign_in = crate::account_registration("http://127.0.0.1:9");
        crate::account_provider(&mut value, &sign_in);
        crate::welcome_provider(&mut value, &sign_in);
        model["model"]["provider"] = 2.into();
        super::bind_model(&mut model, &value["world_providers"]);
        assert_eq!(model["model"]["provider"], super::WELCOME_PROVIDER);
        assert_eq!(model["model"]["endpoint"], super::WELCOME_ENDPOINT);
    }

    #[test]
    fn ctl_settings_commit_the_world_providers_before_a_restart() {
        let scratch = skein_scratch::Scratch::new("browser-provider-settings");
        let directory = scratch.path();
        super::write(directory, &directory.join("bundle"));
        let settings: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("browser-agents.json")).expect("ctl settings file"))
                .expect("settings JSON");
        let items = settings["items"].as_array().expect("settings items");
        let providers: Vec<_> = items.iter().filter(|item| item["setting"]["kind"] == "provider").cloned().collect();
        let wanted = temper_agent_fixtures::providers::read(directory);
        assert_eq!(providers, temper_box_kit::provider_setup::items(&wanted), "ctl must commit the kept providers");
        assert_eq!(providers.len(), 1, "the world has one non-scripted provider");
        assert_eq!(providers[0]["setting"], serde_json::json!({"kind":"provider","number":super::PROVIDER}));
        assert_eq!(providers[0]["set"]["endpoints"][0]["address"]["socket"], super::UNREACHED);
        assert!(providers.iter().all(|item| item["setting"]["number"] != 3), "the scripted provider is fixed");
        assert_eq!(items.iter().filter(|item| item["setting"]["kind"] == "agents").count(), 1);
    }

    #[test]
    fn demo_reserves_every_init_listener_and_moves_its_references() {
        const EPHEMERAL_FIRST: u16 = 32_768;
        let mut file = temper_config_shell::File {
            listen: Some("127.0.0.1:8367".parse().expect("listen")),
            webhooks: Some("127.0.0.1:8371".parse().expect("webhooks")),
            ..Default::default()
        };
        super::own_demo_listeners(&mut file, |_| {
            [10_001, 10_002].map(|port| (std::net::Ipv4Addr::LOCALHOST, port).into())
        });
        let text = temper_config_shell::render(&file, temper_config_shell::Shape::Standalone);
        let facts = temper_config_shell::Facts { login: "ada".into(), hostname: "fixture".into(), provider_key: true };
        let read =
            temper_config_shell::parse(&text, temper_config_shell::Shape::Standalone, &facts).expect("demo file");
        let value: serde_json::Value = serde_json::from_slice(&read.document).expect("demo start");
        let text = serde_json::to_string(&value).expect("document JSON");
        for port in 8367..=8371 {
            let old = format!("127.0.0.1:{port}");
            assert!(!text.contains(&old), "init socket remains: {old}");
        }
        let mut seen = std::collections::BTreeSet::new();
        for pointer in ["/application/clients/listener/socket", "/application/webhooks/listener/socket"] {
            let address: std::net::SocketAddr =
                value.pointer(pointer).expect("listener").as_str().expect("socket").parse().expect("address");
            assert!(address.ip().is_loopback());
            assert!((10_000..EPHEMERAL_FIRST).contains(&address.port()));
            assert!(seen.insert(address), "listeners must be distinct: {address}");
        }
        assert!(value["application"].get("machine").is_none());
        assert_eq!(value["engine"]["clients"]["provider"]["clients"], serde_json::json!([]));
    }

    #[test]
    fn the_picker_fixture_admits_stock_models_for_every_charter() {
        let directory = std::path::Path::new("/fixture");
        let (agent, value) = super::typed_documents(directory, &directory.join("bundle"));
        let agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
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
        let (agent, bytes) = super::typed_documents(directory, &directory.join("bundle"));
        let value = bytes;
        let engine = stock_engine(&value);
        let agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
        let turns = agent["profile"]["policy"]["max_turns"].as_u64().expect("agent turn allowance");
        for charter in &engine.charters {
            assert!(u64::from(charter.policy.turns) <= turns, "charter {} exceeds agent turns", charter.number);
        }
        assert_eq!(engine.charters.len(), 12);
        assert_eq!(
            usize::try_from(engine.limits.core.documents.lines).expect("line capacity"),
            engine.charters.iter().filter(|charter| charter.number != 10).count()
        );
    }
}
