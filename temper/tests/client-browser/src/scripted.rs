//! A world's scripted provider, installed through ctl (protocol/configuration/providers.md, 6).
use crate::{Tier, WITHIN, configuration, temper};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

pub(crate) const SCRIPTS: &str = "browser-scripts.json";

pub(crate) fn document(socket: &str, key: &Path) -> Value {
    json!({"listen":{"socket":socket}, "key":key, "tls":null,
        "models":[{"name":"demo","window":1_048_576,"output":8192}],
        "asked":[], "cued":[], "otherwise":["scripted model"]})
}

pub(crate) fn write(directory: &Path, scripts: &Value) {
    configuration::private(&directory.join(SCRIPTS), &serde_json::to_vec(scripts).expect("world scripts"));
}

pub(crate) fn limits(directory: &Path, figures: &[(&str, u64)]) {
    let overrides: Vec<_> =
        figures.iter().map(|(name, figure)| ("limits", *name, *figure, "browser fixture")).collect();
    temper_box_kit::limit_settings::hold(directory, &overrides);
    let mut limits: Vec<_> =
        figures.iter().map(|(name, figure)| json!({"name":name,"value":figure,"reason":"browser fixture"})).collect();
    limits.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    configuration::private(
        &directory.join("browser-limits.json"),
        &serde_json::to_vec(&limits).expect("browser limits"),
    );
}

pub(crate) fn settings(socket: &str) -> Value {
    let stock = temper_agent_fixtures::providers::stock();
    let mut anthropic = temper_box_kit::provider_setup::items(&stock)
        .into_iter()
        .find(|item| item["setting"]["number"] == 2)
        .expect("stock Anthropic");
    anthropic["set"].as_object_mut().expect("provider").remove("direct");
    anthropic["set"]["endpoints"].as_array_mut().expect("endpoints").retain(|endpoint| endpoint["number"] != 4);
    // The agent's standard profile admits four endpoints. Keep Anthropic's program
    // endpoint and free its unused direct endpoint for this world's scripted provider.
    json!({"items":[anthropic,
        {"setting":{"kind":"provider","number":4},"set":{
            "number":4,"name":"browser","endpoint":"browser","signed_in_plan":false,"sources":{"keys":true},
            "endpoints":[{"name":"browser","number":5,"dialect":3,"address":{"socket":socket},
                "transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}},
        {"setting":{"kind":"model","number":8},"set":{
            "number":8,"model":{"provider":4,"endpoint":5,"dialect":3,"program":null,"name":"demo",
                "window":1_048_576,"output":8192,"input_price":1,"cached_price":1,"cache_write_price":null,
                "output_price":1,"price_unit":1000,"efforts":[],"effort":null},
            "shown":null,"description":null,"picker":true,"withdrawn":false,"source":"person","estimated":false}}
    ]})
}

impl Tier {
    /// Run the standalone scripted model until this guard leaves the story.
    #[must_use]
    pub fn scripted_model(&self) -> temper_box_kit::parent::Guard {
        let directory = self.scratch.path();
        let [socket] = temper_fixtures::ports::addresses::<1>(&[]);
        let key = directory.join("browser-model.key");
        let doc = directory.join("browser-model.json");
        configuration::private(
            &doc,
            &serde_json::to_vec(&document(&socket.to_string(), &key)).expect("model document"),
        );
        let mut settings = settings(&socket.to_string());
        let limits = directory.join("browser-limits.json");
        if limits.is_file() {
            let values: Value =
                serde_json::from_slice(&fs::read(limits).expect("browser limits")).expect("limits JSON");
            settings["items"].as_array_mut().expect("items").push(json!({"setting":{"kind":"limits"},"set":values}));
        }
        configuration::private(
            &directory.join("browser-model-settings.json"),
            &serde_json::to_vec(&settings).expect("model settings"),
        );
        let log = directory.join("browser-model.log");
        let output = fs::File::create(&log).expect("model log");
        let child = Command::new(temper())
            .args(["fake", "llm", "--document"])
            .arg(doc)
            .arg("--scripts")
            .arg(directory.join(SCRIPTS))
            .args(["--ready", "browser model "])
            .stdin(Stdio::null())
            .stdout(output.try_clone().expect("log clone"))
            .stderr(output)
            .spawn()
            .expect("standalone scripted model");
        let mut patience = temper_box_kit::patience::Patience::arm("the browser's scripted model", WITHIN);
        patience.watch_process(child.id());
        let guard = temper_box_kit::parent::Guard::new(child);
        loop {
            let text = fs::read_to_string(&log).expect("model readiness log");
            if text.contains("browser model ") {
                break;
            }
            if let Some(figures) = patience.failed() {
                panic!("scripted model did not listen: {text}; {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        bootstrap(directory, &key);
        guard
    }
}

/// A no-task sitting commits the settings and restarts before any browser link is held.
fn bootstrap(directory: &Path, key: &Path) {
    use temper_box_kit::{
        boxed,
        ctl::{Call, Step},
    };
    let settings = directory.join("browser-model-settings.json");
    let settings_calls = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&[
            "settings",
            "apply",
            settings.to_str().expect("settings path"),
            "--against",
            "1",
            "--key",
            "browser-model",
        ])),
        Step::Run(Call::new(&["sign-out"])),
    ];
    let account_calls = vec![
        Step::Run(Call::sign_in()),
        Step::Run(Call::new(&[
            "account",
            "add-key",
            "4",
            key.to_str().expect("model key path"),
            "--key",
            "browser-model-key",
        ])),
        Step::Run(Call::new(&["account", "list", "--until", "usable"])),
        Step::Run(Call::new(&["sign-out"])),
    ];
    for calls in [settings_calls, account_calls] {
        let count = calls.len();
        let flags = configuration::flags(directory)
            .into_iter()
            .map(|word| word.into_string().expect("directory flag"))
            .collect();
        let outcome = boxed::run_ctl(directory, flags, directory, calls, boxed::BOX_DEADLINE, |_| ());
        for call in &outcome.calls {
            assert_eq!(
                call.exit,
                Some(skein_io::kernel::Exit::Code(0)),
                "scripted setup: {}",
                String::from_utf8_lossy(&call.stderr)
            );
        }
        assert_eq!(outcome.calls.len(), count, "every bootstrap call ran");
        assert_eq!(outcome.exit, Some(skein_io::kernel::Exit::Code(0)), "{}", String::from_utf8_lossy(&outcome.stderr));
        assert_eq!(outcome.left, Some(Vec::new()), "scripted setup drained");
        let held = directory.join("limits-held");
        if held.is_file() {
            fs::remove_file(held).expect("release browser limit settings");
        }
    }
}
