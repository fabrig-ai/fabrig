use super::fixtures::{self, DIRECTORIES, parse};
use crate::files;
use serde_json::json;

#[test]
fn engine_figures_keep_the_hub_capacity_required_by_the_stock_agent() {
    let document = parse(&temper_config_shell::init_document(false));
    assert!(document["engine"].get("limits").is_none());
    let mut engine = temper_service_core::derived::stock();
    temper_service_core::derived::engine(&mut engine, temper_service_core::derived::STOCK).unwrap();
    assert_eq!(engine.hub.accounts, 4);
    // Fabrig's own development's 63,740-byte payload, less the separator.
    assert_eq!(engine.hub.event_bytes, 63_738);
    assert_eq!(engine.hub.run_calls, 48);
    for worker in document["workers"].as_array().unwrap() {
        assert!(worker.get("limits").is_none());
    }
}

#[test]
fn written_workers_outlast_an_engine_restart_below_the_fleet_grace() {
    let document = parse(&temper_config_shell::init_document(false));
    assert!(document["engine"].get("limits").is_none());
    assert!(document["engine"].get("memory_bytes").is_none());
    assert!(document["application"].get("limits").is_none());
    let profile = jig_protocol_config::stock();
    let grace = profile.core.fleet.grace.as_millis();
    for worker in document["workers"].as_array().unwrap() {
        assert!(worker.get("graces").is_none());
        assert!(worker.get("programs").is_none());
        assert!(worker.get("agent").is_none());
        assert!(worker.get("limits").is_none());
        let bytes = serde_json::to_vec(worker).unwrap();
        let document = jig_protocol_config::document(
            &bytes,
            &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
        )
        .unwrap();
        let worker = jig_protocol_config::worker(
            &document,
            0,
            &jig_protocol_config::Directories::working(),
            &temper_service_core::startup::fixed_worker(),
        )
        .unwrap();
        let stop = jig_protocol_config::stop_bound(&worker.graces, profile.link.ping, profile.link.silence)
            .unwrap()
            .as_millis();
        assert_eq!(stop, 326_000);
        assert!(stop < grace);
    }
}

#[test]
fn the_box_has_room_for_reference_creating_executors() {
    let written = parse(&temper_config_shell::init_document(false));
    let reference = parse(include_bytes!("../../../../docs/examples/reference/settings.json"));
    let mut charters = vec![1, 2, 4, 5, 9, 10, 11, 12, 13, 14, 15, 16];
    for item in reference["items"].as_array().unwrap() {
        if item["setting"]["kind"] == "charter" {
            charters.push(item["setting"]["number"].as_u64().unwrap());
        }
    }
    charters.sort_unstable();
    charters.dedup();
    assert_eq!(charters, [1, 2, 4, 5, 9, 10, 11, 12, 13, 14, 15, 16]);
    assert!(written["engine"].get("limits").is_none());
    let limits = jig_protocol_config::stock().core;
    jig_protocol_config::check_effective_limits(&limits).expect("deployment receiving limits fit the codecs");
    assert!(limits.authority.executors >= 22);
    assert!(limits.tasks.executor_kinds >= 22);
    assert_eq!(limits.documents.lines, 16);
    assert!(usize::try_from(limits.documents.lines).unwrap() >= charters.len());
    // The closer adds procedure p6 beside p2, p3 and p4 on each configured connector.
    let procedures = 4;
    assert!(usize::try_from(limits.documents.lines).unwrap() >= procedures);
    // The reference omits the trial, drafter and general standing role from delegation.
    let mut delegated = 0;
    for number in &charters {
        if ![10, 12, 13].contains(number) {
            delegated += 1;
        }
    }
    let executors = delegated + procedures + 2 + 3; // Merge/close jobs and owner/maintainer/member roles.
    assert_eq!(executors, 18);
    assert!(executors <= usize::try_from(limits.authority.executors).unwrap());
    assert!(executors <= usize::try_from(limits.tasks.executor_kinds).unwrap());
}

#[test]
fn the_start_keeps_places_and_limits_out_of_the_document() {
    let value = parse(&temper_config_shell::init_document(false));
    assert_eq!(value["workers"][0]["slots"], 1_u32);
    assert!(value["engine"].get("limits").is_none());
    assert!(value["application"].get("limits").is_none());
    assert!(value["workers"][0]["limits"]["overrides"].get("hub.slots").is_none());
}

#[test]
fn the_git_link_preserves_the_machine_path() {
    // configuration/moved.md, 2: machine paths stay out of the deployment document.
    for (row, binary) in [
        ("quote and backslash", b"/opt/tem\"per\\bin/temper".as_slice()),
        ("control bytes and UTF-8", b"/opt/\n\t\x01/\xc3\xa9/temper".as_slice()),
    ] {
        let facts = crate::Facts { binary, git: Some(binary), claude: Some(binary), ..fixtures::facts() };
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid text paths");
        let crate::Content::Link { target } = &fixtures::file(&answered.files, b"demo/bin/git").content else {
            panic!("the fake git is a launcher link")
        };
        assert_eq!(target.as_ref(), binary, "{row}: the launcher preserves the machine path");
    }
}

#[test]
fn the_person_keeps_a_forgejo_login_or_falls_back() {
    use crate::Person::{FellBack, Found};
    for (row, login, written, person) in [
        ("valid", Some(b"ada".as_slice()), "ada", Found),
        ("case kept", Some(b"Ada".as_slice()), "Ada", Found),
        ("reserved", Some(b"admin".as_slice()), "person", FellBack),
        ("reserved case", Some(b"Admin".as_slice()), "person", FellBack),
        ("reserved pattern", Some(b"x.keys".as_slice()), "person", FellBack),
        ("pattern case", Some(b"X.GPG".as_slice()), "person", FellBack),
        ("pattern is suffix", Some(b"x.keys.other".as_slice()), "x.keys.other", Found),
        ("invalid start", Some(b"-ada".as_slice()), "person", FellBack),
        (
            "at bound",
            Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_slice()),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            Found,
        ),
        ("past bound", Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_slice()), "person", FellBack),
        ("bot", Some(b"temper".as_slice()), "person", FellBack),
        ("admin account", Some(b"temper-admin".as_slice()), "person", FellBack),
        ("org", Some(b"work".as_slice()), "person", FellBack),
        ("bot case", Some(b"Temper".as_slice()), "person", FellBack),
        ("empty", Some(b"".as_slice()), "person", FellBack),
        ("nontext", Some(b"\xff".as_slice()), "person", FellBack),
        ("absent", None, "person", FellBack),
    ] {
        let facts = crate::Facts { login, ..fixtures::facts() };
        let answer = files(&facts, &DIRECTORIES, &[0; 32]).expect("login never refuses init");
        assert_eq!(answer.person, person, "{row}");
        assert_eq!(crate::person::login(login).0, written.as_bytes(), "{row}");
    }
}

#[test]
fn the_template_seeds_no_project_and_no_settings_members() {
    let document = parse(&temper_config_shell::init_document(false));
    for member in [
        "llm",
        "charters",
        "templates",
        "policies",
        "rules",
        "deployment",
        "providers",
        "connectors",
        "acts",
        "projects",
    ] {
        assert!(document["engine"].get(member).is_none());
    }
    assert!(document["application"].get("landing").is_none());
    assert!(document["application"].get("welcome").is_none());
    assert!(document["application"].get("charter").is_none());
    for member in ["heartbeat_ms", "link", "approval"] {
        assert!(document["engine"]["clients"].get(member).is_none());
    }
}

#[test]
fn the_bare_document_is_the_box_less_its_parts() {
    let facts = fixtures::facts();
    let boxed = files(&facts, &DIRECTORIES, &[0; 32]).expect("boxed files");
    let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).expect("bare files");
    assert_eq!(bare.len(), 1);
    assert_eq!(bare[0], boxed.files[0]);
    assert_eq!(bare[0].name, b"worker.secret");
}

#[test]
fn a_box_keeps_an_empty_provider_for_clients_added_at_setup() {
    let document = parse(&temper_config_shell::init_document(true));
    assert_eq!(document["engine"]["owners"][0]["subject"], "ada@localhost");
    assert_eq!(
        document["engine"]["clients"]["provider"],
        json!({
            "key": "provider.key", "addresses": [1_u32],
            "clients": parse(b"[]")
        })
    );
    assert!(document["application"].get("machine").is_none());
    assert!(document["application"]["sign_in"].is_null());
}

#[test]
fn a_box_without_a_key_keeps_its_provider_and_source_null() {
    let document = parse(&temper_config_shell::init_document(false));
    assert!(document["engine"]["clients"]["provider"].is_null());
    assert!(document["application"].get("machine").is_none());
    assert_eq!(document["engine"]["owners"][0]["subject"], "ada@localhost");
}
