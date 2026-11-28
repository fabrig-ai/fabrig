//! Browser settings on the real loop, without running a browser or a box process.
use serde_json::Value;
use skein_io::kernel::Exit;
use skein_scratch::Scratch;
use std::{fs, path::Path};
use temper_box_kit::{
    ctl::{self, Call, Step},
    hosted::{self, Sitting},
};
use temper_client_browser_world::testing::{self as configuration, ENDPOINT, PROVIDER, UNREACHED};

fn deployment(name: &str) -> Scratch {
    let scratch = Scratch::new(name);
    let bundle = scratch.path().join("bundle");
    fs::create_dir_all(bundle.join("0123456789abcdef")).expect("inert bundle directory");
    fs::write(bundle.join("index.html"), b"<html><head></head></html>").expect("inert bundle page");
    configuration::write(scratch.path(), &bundle);
    scratch
}
fn sit(directory: &Path, steps: Vec<Step>) -> Sitting {
    hosted::run_ctl(directory, configuration::flags(directory), directory, steps)
        .unwrap_or_else(|why| panic!("settings sitting: {why}"))
}
fn document(file: &Path) -> Value {
    serde_json::from_slice(&fs::read(file).expect("settings fixture")).expect("fixture JSON")
}

#[test]
fn the_next_start_reads_the_applied_settings() {
    let scratch = deployment("applied-sitting");
    let directory = scratch.path();
    let applied = configuration::install(directory).expect("first install sitting");
    assert_eq!(applied.calls.iter().map(|call| call.exit).collect::<Vec<_>>(), [Some(Exit::Code(0)); 4]);
    assert_eq!(applied.exit, Exit::Code(0), "serve drained: {}", applied.ending);
    assert_eq!(applied.left, 0);
    assert!(directory.join("browser-settings-applied").is_file());
    for copy in ["providers.setting", "agents.setting"] {
        assert!(directory.join(copy).is_file(), "serve kept {copy} for its next start");
    }
    let against = ctl::newest(&applied.calls[1]);
    let changed = format!("revision {}\n", against + 1);
    assert_eq!(applied.calls[2].stdout, changed.as_bytes(), "apply followed the inventory read");

    let next = sit(
        directory,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["settings", "show", "--json"])),
            // The inventory lists identities and revisions, not whole values. Read the named revision as well.
            Step::Then(Box::new(|calls| {
                Call::new(&["revision", "show", &ctl::newest(&calls[1]).to_string(), "--json"])
            })),
            Step::Run(Call::new(&["sign-out"])),
        ],
    );
    assert_eq!(next.failure, None);
    assert_eq!(next.left, 0);
    let inventory = ctl::json(&next.calls[1]);
    assert_eq!(inventory["newest"], against + 1);
    let revision = ctl::json(&next.calls[2]);
    let items = revision["items"].as_array().expect("revision items");
    let agents = items.iter().find(|item| item["setting"]["kind"] == "agents").expect("agents item");
    assert_eq!(agents["set"]["profile"]["policy.check_timeout"], 60_000, "browser agent check time");
    let provider = items
        .iter()
        .find(|item| item["setting"]["kind"] == "provider" && item["setting"]["number"] == PROVIDER)
        .expect("browser provider");
    assert_eq!(provider["set"]["endpoints"][0]["address"]["socket"], UNREACHED);
    let name = "tasks.charters";
    let figures = document(&directory.join("fixture-limits.json"));
    let expected = figures["limits"]
        .as_array()
        .expect("fixture overrides")
        .iter()
        .find(|item| item["name"] == name)
        .expect("browser charter limit");
    let limits = items.iter().find(|item| item["setting"]["kind"] == "limits").expect("limits item");
    assert!(limits["set"].as_array().expect("applied overrides").contains(expected), "kept {name} from fixture");
    let model = items
        .iter()
        .find(|item| item["setting"]["kind"] == "model" && item["set"]["model"]["provider"] == PROVIDER)
        .expect("stock model for the browser provider");
    assert_eq!(model["set"]["model"]["endpoint"], ENDPOINT);
    assert_eq!(model["set"]["model"]["dialect"], 3, "browser endpoint dialect");
}

#[test]
fn a_refused_call_reports_its_words_and_drains_serve() {
    let scratch = deployment("refused-sitting");
    let directory = scratch.path();
    let file = directory.join("browser-agents.json");
    let sitting = sit(
        directory,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&[
                "settings",
                "apply",
                file.to_str().expect("settings path"),
                "--against",
                "99",
                "--key",
                "stale",
            ])),
            Step::Run(Call::new(&["sign-out"])),
        ],
    );
    let [sign_in, refused] = sitting.calls.as_slice() else { panic!("stopped at the second call") };
    assert_eq!(sign_in.exit, Some(Exit::Code(0)));
    let words = std::str::from_utf8(&refused.stderr).expect("refusal words");
    assert_eq!(refused.exit, Some(Exit::Code(2)), "stale revision: {words}");
    assert_eq!(words, "settings apply: stale; newest revision 1; read settings show again\n");
    assert!(sitting.failure.expect("the sitting failed").ends_with(words));
    assert_eq!(sitting.left, 1, "sign-out did not start");
    assert_eq!(sitting.exit, Exit::Code(0), "serve drained after the refusal: {}", sitting.ending);
}

/// The forge's settings judgement and the root's kept value share the apply's decision.
#[test]
fn a_forge_setting_removal_survives_a_real_loop_restart() {
    let scratch = deployment("remove-forge-setting");
    let directory = scratch.path();
    let installed = configuration::install(directory).expect("install with an agents setting");
    assert_eq!(installed.exit, Exit::Code(0));
    assert_eq!(installed.failure, None);
    let against = ctl::newest(&installed.calls[1]) + 1;
    let removal = against + 1;
    let file = directory.join("remove-agents.json");
    fs::write(&file, br#"{"items":[{"setting":{"kind":"agents"},"edit":"removed"}]}"#).expect("agents removal fixture");
    let changed = sit(
        directory,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&[
                "settings",
                "apply",
                file.to_str().expect("settings path"),
                "--against",
                &against.to_string(),
                "--key",
                "remove-agents",
            ])),
            Step::Run(Call::new(&["sign-out"])),
        ],
    );
    assert_eq!(changed.failure, None);
    assert_eq!(changed.calls[1].exit, Some(Exit::Code(0)));
    assert_eq!(changed.calls[1].stdout, format!("revision {removal}\n").as_bytes());
    assert_eq!(changed.left, 0);

    let reopened = sit(
        directory,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["settings", "show", "--json"])),
            Step::Run(Call::new(&["settings", "show"])),
            Step::Run(Call::new(&["revision", "show", &removal.to_string(), "--json"])),
            Step::Run(Call::new(&["sign-out"])),
        ],
    );
    assert_eq!(reopened.failure, None);
    assert_eq!(reopened.left, 0);
    let inventory = ctl::json(&reopened.calls[1]);
    assert_eq!(inventory["newest"], removal, "restart made no revision");
    let settings = inventory["settings"].as_array().expect("inventory settings");
    let agents = settings.iter().find(|item| item["setting"]["kind"] == "agents").expect("agents inventory row");
    assert_eq!(agents["follows"], "change");
    assert_eq!(agents["last"], removal);
    // The JSON inventory groups removals under "change"; text distinguishes them.
    let text = std::str::from_utf8(&reopened.calls[2].stdout).expect("inventory text");
    let removed = format!("  agents: changed in revision {removal}, removed");
    assert!(text.lines().any(|line| line == removed), "missing inventory line: {removed}\n{text}");
    let revision = ctl::json(&reopened.calls[3]);
    let items = revision["items"].as_array().expect("revision items");
    let agents = items.iter().find(|item| item["setting"]["kind"] == "agents").expect("agents revision item");
    assert_eq!(agents["application"], "removed", "agents revision item: {agents}");
}
