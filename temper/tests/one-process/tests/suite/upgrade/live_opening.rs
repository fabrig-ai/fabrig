//! The settled image and the stopped deployment's startup facts.
use skein_fake_machine::{Item, Machine};
use temper_config_shell::{Facts, File, Shape, TestGitHub, Transport};
use temper_engine_service::stock::Given;
use temper_one_process_world::testing::{Rehearsal, USER};
use temper_shell::directories::Directories;

pub(super) fn settled(in_data: bool) -> Machine {
    macro_rules! files {
        ($($path:literal),+ $(,)?) => { [$(($path, include_bytes!(concat!("../../../golden/settled/state/", $path)).as_slice())),+] };
    }
    let files = files![
        "active",
        "generations/1/manifest",
        "generations/1/secrets/log-00000000000000000001",
        "generations/1/store/log-00000000000000000001",
        "generations/1/store/logs/0000000000000000/0000000000000002",
        "generations/1/store/logs/0000000000000000/0000000000000003",
        "generations/1/store/logs/0000000000000000/0000000000000004",
        "generations/1/store/logs/0000000000000000/0000000000000006",
        "generations/1/store/logs/0000000000000000/0000000000000007",
        "generations/1/store/logs/0000000000000000/0000000000000008",
        "generations/1/store/logs/0000000000000000/0000000000000009",
        "generations/1/store/logs/0000000000000000/000000000000000a",
        "generations/1/store/logs/0000000000000000/000000000000000b",
        "generations/1/store/logs/0000000000000000/000000000000000c",
        "generations/1/store/logs/0000000000000000/000000000000000d",
    ];
    let mut files = std::collections::BTreeMap::from(files);
    let mut items = Vec::new();
    if in_data {
        items.push(Item::directory(b"state").mode(0o700));
    }
    for line in include_str!("../../../golden/settled/modes.txt").lines() {
        let (facts, path) = line.rsplit_once(' ').expect("manifest path");
        let Some(relative) = path.strip_prefix("state/") else { continue };
        let name = if in_data { path } else { relative };
        let item = if facts.starts_with("d ") {
            Item::directory(name.as_bytes()).mode(0o700)
        } else {
            let bytes = files.remove(relative).expect("settled deployment file");
            Item::file(name.as_bytes(), bytes).mode(0o600)
        };
        items.push(item);
    }
    assert!(files.is_empty(), "all embedded files are in the settled manifest");
    let mut machine = Machine::for_user(USER);
    for items in [&items[..], &[]] {
        let root = machine.lay(items);
        machine.close(root);
    }
    machine
}

fn given() -> Given {
    let registration = serde_json::json!({
        "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
        "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
        "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
        "redirect_path":"callback"
    });
    let mut stock = temper_agent_fixtures::providers::stock();
    stock["providers"].as_array_mut().expect("providers").truncate(1);
    stock["endpoints"].as_array_mut().expect("endpoints").truncate(1);
    stock["providers"][0]["sources"]["sign_in"] = registration;
    stock["providers"][0]["manage_usage"] = "https://accounts.example.invalid/usage".into();
    let endpoint = &mut stock["endpoints"][0];
    endpoint.as_object_mut().expect("endpoint").remove("server_name");
    endpoint["address"] = serde_json::json!({"socket":"127.0.0.1:1"});
    endpoint["transport"] = "plaintext".into();
    let mut endpoint = endpoint.clone();
    endpoint["name"] = "second".into();
    endpoint["number"] = 2.into();
    endpoint["provider"] = "second".into();
    stock["endpoints"].as_array_mut().expect("endpoints").push(endpoint);
    let mut provider = stock["providers"][0].clone();
    provider["name"] = "second".into();
    provider["number"] = 2.into();
    provider["endpoint"] = "second".into();
    stock["providers"].as_array_mut().expect("providers").push(provider);
    Given::Stock.with_providers(temper_agent_fixtures::providers::values(&stock), Box::new([]))
}

fn configuration() -> (Box<[u8]>, Directories) {
    let mut file = File {
        listen: Some("127.0.0.1:7100".parse().expect("clients listener")),
        workers: Some("127.0.0.1:7103".parse().expect("workers listener")),
        owner: Some("a@b".into()),
        ..File::default()
    };
    file.test.github = Some(TestGitHub {
        api: "127.0.0.1:1".into(),
        transport: Transport::Plaintext,
        remote: "http://127.0.0.1:1".into(),
        web: "http://127.0.0.1:1".into(),
        git: Some("/usr/bin/git".into()),
    });
    let text = temper_config_shell::render(&file, Shape::EngineAlone);
    let facts = Facts { login: "fixture".into(), hostname: "fixture".into(), provider_key: false };
    let read = temper_config_shell::parse(&text, Shape::EngineAlone, &facts).expect("stopped engine file");
    let directories = Directories {
        config: b"/config".as_slice().into(),
        data: b"/data".as_slice().into(),
        cache: b"/cache".as_slice().into(),
        origins: [skein_shell::Origin::Flag; 3],
    };
    (read.document, directories)
}

#[test]
fn a_live_change_and_uncertain_write_reach_the_opening_without_network() {
    let (document, directories) = configuration();
    temper_one_process_world::testing::a_live_change_and_uncertain_write_reach_the_opening_without_network(
        settled(false),
        &document,
        &directories,
        &given(),
    );
}

pub(super) fn rehearsal() -> Rehearsal {
    let (document, directories) = configuration();
    Rehearsal::new(document, directories, given())
}

#[test]
fn upgrade_tools_leave_an_empty_deployment_empty() {
    rehearsal().empty();
}

#[test]
fn a_refused_person_record_leaves_the_source_whole() {
    rehearsal().person();
}

#[test]
fn kept_and_moved_setting_numbers_both_restore_in_scratch() {
    rehearsal().kept_and_moved();
}

#[test]
fn a_charter_past_own_numbers_leaves_the_source_whole() {
    rehearsal().past();
}

#[test]
fn a_charter_taken_by_stock_leaves_the_source_whole() {
    rehearsal().taken();
}

#[test]
fn empty_limits_held_refuses_versions_above_the_stock() {
    rehearsal().limits_held();
}
