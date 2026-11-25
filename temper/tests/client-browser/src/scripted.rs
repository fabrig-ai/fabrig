//! A world's scripted provider, installed through ctl (protocol/configuration/providers.md, 6).
use crate::{Tier, WITHIN, configuration, temper};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

pub(crate) const SCRIPTS: &str = "browser-scripts.json";
pub(crate) const KEY: &str = "browser-model.key";
pub(crate) const PROVIDER: u16 = 4;

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
        {"setting":{"kind":"provider","number":PROVIDER},"set":{
            "number":PROVIDER,"name":"browser","endpoint":"browser","signed_in_plan":false,"sources":{"keys":true},
            "endpoints":[{"name":"browser","number":5,"dialect":3,"address":{"socket":socket},
                "transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}},
        {"setting":{"kind":"model","number":8},"set":{
            "number":8,"model":{"provider":PROVIDER,"endpoint":5,"dialect":3,"program":null,"name":"demo",
                "window":1_048_576,"output":8192,"input_price":1,"cached_price":1,"cache_write_price":null,
                "output_price":1,"price_unit":1000,"efforts":[],"effort":null},
            "shown":null,"description":null,"picker":true,"withdrawn":false,"source":"person","estimated":false}}
    ]})
}

impl Tier {
    /// Run the standalone scripted model until this guard leaves the story.
    #[must_use]
    pub fn scripted_model(&self) -> temper_box_kit::parent::Guard {
        self.scripted_model_with_parking(None)
    }

    pub(crate) fn scripted_model_with_parking(&self, charter: Option<u32>) -> temper_box_kit::parent::Guard {
        let directory = self.scratch.path();
        let [socket] = temper_fixtures::ports::addresses::<1>(&[]);
        let key = directory.join(KEY);
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
        let log = directory.join("browser-model.log");
        let output = fs::File::create(&log).expect("model log");
        let mut command = Command::new(temper());
        command
            .args(["fake", "llm", "--document"])
            .arg(doc)
            .arg("--scripts")
            .arg(directory.join(SCRIPTS))
            .args(["--ready", "browser model "])
            .stdin(Stdio::null())
            .stdout(output.try_clone().expect("log clone"))
            .stderr(output);
        let child = spawn_model(&mut command).expect("standalone scripted model");
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
        bootstrap(directory, settings, charter);
        guard
    }
}

fn spawn_model(command: &mut Command) -> std::io::Result<std::process::Child> {
    // A harness exit can bypass the guard. Bind this separately launched peer to its owner too.
    skein_shell::spawn_dying_with_parent(command, skein_io::kernel::Signal::Kill)
}

/// A no-task sitting commits the settings; the story's start takes the provider.
fn bootstrap(directory: &Path, settings: Value, charter: Option<u32>) {
    use temper_box_kit::{
        boxed,
        ctl::{Call, Step},
    };
    let file = directory.join("browser-model-settings.json");
    let mut settings_calls = vec![Step::Run(Call::sign_in())];
    if let Some(charter) = charter {
        settings_calls.push(Step::Run(Call::new(&["settings", "show", "charter", &charter.to_string(), "--json"])));
        settings_calls.push(Step::Then(Box::new(move |calls| {
            let read = temper_box_kit::ctl::json(&calls[1]);
            apply_model_settings(&file, &parking_settings(settings.clone(), read))
        })));
    } else {
        settings_calls.push(Step::Run(apply_model_settings(&file, &settings)));
    }
    settings_calls.push(Step::Run(Call::new(&["sign-out"])));
    let count = settings_calls.len();
    let flags =
        configuration::flags(directory).into_iter().map(|word| word.into_string().expect("directory flag")).collect();
    let outcome = boxed::run_ctl(directory, flags, directory, settings_calls, boxed::BOX_DEADLINE, |_| ());
    for call in &outcome.calls {
        assert_eq!(
            call.exit,
            Some(skein_io::kernel::Exit::Code(0)),
            "scripted setup: {}",
            temper_box_kit::without_secrets(&call.stderr)
        );
    }
    assert_eq!(outcome.calls.len(), count, "every bootstrap call ran");
    assert_eq!(
        outcome.exit,
        Some(skein_io::kernel::Exit::Code(0)),
        "{}",
        temper_box_kit::without_secrets(&outcome.stderr)
    );
    assert_eq!(outcome.left, Some(Vec::new()), "scripted setup drained");
    let held = directory.join("limits-held");
    if held.is_file() {
        fs::remove_file(held).expect("release browser limit settings");
    }
}

fn parking_settings(mut settings: Value, mut read: Value) -> Value {
    read["value"]["waiting"] = json!(0);
    let tools = &mut read["value"]["tools"];
    *tools = json!(tools.as_u64().expect("charter tool families") | crate::public::SUBSCRIBE);
    settings["items"]
        .as_array_mut()
        .expect("settings items")
        .push(json!({"setting":read["setting"],"set":read["value"]}));
    settings
}

fn apply_model_settings(file: &Path, settings: &Value) -> temper_box_kit::ctl::Call {
    configuration::private(file, &serde_json::to_vec(settings).expect("model settings"));
    temper_box_kit::ctl::Call::new(&[
        "settings",
        "apply",
        file.to_str().expect("settings path"),
        "--against",
        "1",
        "--key",
        "browser-model",
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    #[expect(clippy::disallowed_methods, reason = "Linux delivers parent death on the spawning thread's exit")]
    fn the_scripted_model_dies_with_its_spawning_thread() {
        let thread = std::thread::spawn(|| {
            let child =
                spawn_model(Command::new("/bin/cat").stdin(Stdio::piped()).stdout(Stdio::null())).expect("model child");
            let guard = temper_box_kit::parent::Guard::new(child);
            let pid = guard.child.as_ref().expect("guarded model").id();
            let fd = skein_shell::keeper::open_child_pidfd(pid, std::process::id()).expect("model identity");
            (guard, fd)
        });
        // The still-open stdin keeps cat alive if the parent-death signal was not armed.
        let (guard, fd) = thread.join().expect("model's owner thread");
        let exited = skein_shell::keeper::wait_readable(fd, skein_lib::Duration::from_secs(1));
        skein_shell::keeper::close_keeper_fd(fd);
        drop(guard);
        assert_eq!(exited, Ok(true), "the child must die even when its owner skips dropping the guard");
    }

    #[test]
    fn a_parked_charter_preserves_the_rest_of_the_model_settings() {
        let settings = settings("127.0.0.1:8370");
        let charter = json!({"number":2,"waiting":10_000_000_000_u64,"tools":514,"instructions":"Produce the change."});
        let setting = json!({"kind":"charter","number":2});
        let mut patched = parking_settings(settings.clone(), json!({"setting":setting,"value":charter}));
        let item = patched["items"].as_array_mut().expect("settings items").pop().expect("charter update");
        assert_eq!(patched, settings);
        assert_eq!(
            item,
            json!({"setting":setting,"set":{"number":2,"waiting":0,"tools":546,"instructions":"Produce the change."}})
        );
    }
}
