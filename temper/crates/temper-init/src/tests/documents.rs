use super::fixtures::{self, DIRECTORIES, cases, content, parse};
use crate::{files, template};
use serde_json::{Value, json};
use skein_lib::bytes;

fn set(value: &mut Value, path: &str, replacement: Value) {
    *value.pointer_mut(path).expect("the member exists in the owned example") = replacement;
}

#[test]
fn engine_figures_keep_the_hub_capacity_required_by_the_stock_agent() {
    let facts = fixtures::facts();
    let boxed = files(&facts, &DIRECTORIES, &[0; 32]).unwrap();
    let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).unwrap();
    for files in [boxed.files.as_ref(), bare.as_ref()] {
        let document = parse(file_bytes(files, b"temper.json"));
        let overrides = &document["engine"]["limits"]["overrides"];
        assert_eq!(overrides["hub.accounts"], 4_u32);
        assert_eq!(overrides["hub.event_bytes"], 10660_u32);
        assert_eq!(overrides["hub.run_calls"], 48_u32);
        for worker in document["workers"].as_array().unwrap() {
            assert!(worker.get("limits").is_none());
        }
    }
}

#[test]
fn written_workers_outlast_an_engine_restart_below_the_fleet_grace() {
    let facts = fixtures::facts();
    let boxed = files(&facts, &DIRECTORIES, &[0; 32]).unwrap();
    let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).unwrap();
    for files in [boxed.files.as_ref(), bare.as_ref()] {
        let document = parse(file_bytes(files, b"temper.json"));
        let limits = &document["engine"]["limits"];
        let profile = jig_protocol_config::stock();
        let grace = profile.core.fleet.grace.as_millis();
        let engine = &limits["overrides"];
        let turn = engine["hub.turn_bytes"].as_u64().unwrap();
        let resume = (turn + 4).max(4_194_304);
        let window = resume + (u64::try_from(size_of::<Box<[u8]>>()).unwrap() - 4) * (resume / 4);
        assert_eq!(engine["core.resume_bytes"], resume);
        assert_eq!(engine["hub.transcript_bytes"], window);
        assert_eq!(document["application"]["limits"]["overrides"]["root.journal.transcript_bytes"], resume);
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
}

#[test]
fn the_box_has_room_for_reference_creating_executors() {
    let answered = files(&fixtures::facts(), &DIRECTORIES, &[0; 32]).unwrap();
    let written = parse(content(&answered, b"temper.json"));
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
    let bytes = serde_json::to_vec(&written["engine"]["limits"]).unwrap();
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
    )
    .unwrap();
    let parsed = jig_protocol_config::overrides::parse(&document, 0, &jig_protocol_config::Path::root());
    assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
    let limits = parsed.limits.core;
    jig_protocol_config::check_effective_limits(&limits).expect("deployment receiving limits fit the codecs");
    assert_eq!(limits.authority.executors, 22);
    assert_eq!(limits.tasks.executor_kinds, 22);
    assert_eq!(limits.documents.lines, 16);
    assert!(usize::try_from(limits.documents.lines).unwrap() >= charters.len());
    assert!(limits.documents.lines >= limits.tasks.charters);
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
fn the_box_runs_four_agents_and_keeps_eight_streams_and_four_calls_beside_them() {
    for source in [fixtures::OWNED, template::TEMPER, template::BARE] {
        let value = parse(source);
        let limits = &value["engine"]["limits"]["overrides"];
        assert_eq!(value["workers"][0]["slots"], 4_u32, "workers[0].slots");
        for name in ["checkout.workspaces", "git_component.operations"] {
            assert_eq!(value["application"]["limits"]["overrides"][name], 4_u32, "{name}");
        }
        for name in ["fleet.slots", "hub.slots"] {
            assert!(limits.get(name).is_none(), "engine overrides {name}");
        }
        assert!(value["workers"][0]["limits"]["overrides"].get("hub.slots").is_none(), "worker overrides hub.slots");
        for name in ["clients.streams", "clients_http.streams"] {
            assert_eq!(limits[name], 8_u32, "{name}");
        }
        for name in ["clients.exchanges", "clients_http.calls"] {
            assert_eq!(limits[name], 12_u32, "{name}");
        }
    }
}

#[test]
fn the_owned_example_offers_openai_keys_and_chatgpts_sign_in() {
    let owned = parse(fixtures::OWNED);
    assert_eq!(
        owned["engine"]["llm"]["providers"],
        json!([{
            "name": "openai", "number": 1_u32, "signed_in_plan": true,
            "manage_usage": "https://chatgpt.com/settings/usage",
            "keep": ["gpt-*"],
            "drop": ["*audio*", "*image*", "*realtime*", "*search*", "*transcribe*", "*tts*", "*-20??-??-??"],
            "endpoint": "openai", "sources": {"keys": true, "sign_in": {
                "issuer": "https://auth.openai.com", "client": "dynamic_agent_client", "name_hint": "temper",
                "authorization": "https://auth.openai.com/api/accounts/authorize",
                "token": "https://auth.openai.com/api/accounts/oauth/token",
                "revocation": "https://auth.openai.com/api/accounts/oauth/revoke",
                "keys": "https://auth.openai.com/.well-known/jwks.json",
                "transport": "tls", "server_name": "auth.openai.com",
                "scopes": ["openid", "profile", "email", "offline_access", "resource.invoke", "chatgpt.tokens.use.direct"],
                "must_grant": ["chatgpt.tokens.use.direct"], "resource": "https://api.openai.com/v1",
                "redirect_path": "auth/callback", "redirect_port": 1_455_u16
            }}
        }])
    );
    let endpoint = &owned["engine"]["llm"]["endpoints"][0];
    assert_eq!(endpoint["name"], "openai");
    assert_eq!(endpoint["provider"], "openai");
    assert_eq!(endpoint["dialect"], 3_u32);
    assert_eq!(endpoint["address"], json!({"host": "api.openai.com", "port": 443_u16}));
    assert_eq!(endpoint["transport"], "tls");
    assert_eq!(endpoint["server_name"], "api.openai.com");
    assert_eq!(endpoint["headers"], parse(b"[]"));
    assert!(owned["engine"]["llm"].get("models").is_none());
}

#[test]
fn the_default_keeps_the_owned_structure_with_stock_models() {
    let owned = parse(fixtures::OWNED);
    assert_eq!(owned["engine"]["memory_bytes"], 1_073_741_824_u64);
    for worker in owned["workers"].as_array().unwrap() {
        assert!(worker.get("memory_bytes").is_none());
    }
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid facts");
        let actual = parse(content(&answered, b"temper.json"));
        assert!(actual["application"].get("demo").is_none(), "{row}");
        for name in ["forges", "identities", "setup"] {
            assert!(actual["application"].get(name).is_none(), "{row}: {name}");
        }
        let ports = ["/application/clients/listener/socket", "/application/webhooks/listener/socket"]
            .map(|path| actual.pointer(path).expect("written listener").as_str().expect("socket"));
        for (index, port) in ports.iter().enumerate() {
            assert!(!ports[..index].contains(port), "{row}: three distinct ports");
        }
        assert!(actual["application"].get("machine").is_none(), "{row}");
        let mut expected = owned.clone();
        assert_ne!(actual["engine"]["owners"].as_array().unwrap().len(), 0);
        // The startup turn_bounds writer holds these to each document's own
        // agent; init adds a demo endpoint, so smith derives a different T.
        let template = parse(include_bytes!("../../template/temper.json"));
        for pointer in ["/engine/limits/overrides", "/application/limits/overrides"] {
            for (name, bytes) in template.pointer(pointer).expect("template overrides").as_object().expect("limit map")
            {
                if name.starts_with("hub.turn")
                    || name == "hub.transcript_bytes"
                    || name == "core.resume_bytes"
                    || name.starts_with("root.loads.")
                    || name == "root.journal.transcript_bytes"
                    || name == "io.output"
                {
                    expected.pointer_mut(pointer).expect("owned overrides")[name] = bytes.clone();
                }
            }
        }
        expected["application"]["limits"]["overrides"]["forge.client.repositories"] = json!(3_u32);
        expected["application"]["limits"]["overrides"]["forge.repositories"] = json!(3_u32);
        expected["application"]["limits"]["overrides"]["forge.output"] = json!(82_u32);
        expected["engine"]["owners"][0]["subject"] = json!("ada@localhost");
        expected["engine"]["clients"]["provider"] = json!({
            "key": "provider.key", "addresses": [1_u32],
            "clients": parse(b"[]")
        });
        let openai = owned["engine"]["llm"]["providers"][0].clone();
        for (path, value) in [
            (
                "/engine/llm/providers",
                json!([
                    openai,
                    anthropic("providers"),
                    {"name": "demo", "number": 3_u64, "signed_in_plan": false,
                     "endpoint": "demo", "sources": {"keys": true}}
                ]),
            ),
            (
                "/engine/llm/endpoints",
                json!([
                    owned["engine"]["llm"]["endpoints"][0],
                    anthropic("endpoints"),
                    anthropic("direct"),
                    {"name": "demo", "number": 3_u64, "dialect": 3_u64, "provider": "demo",
                     "address": {"socket": format!("127.0.0.1:{}", facts.ports.llm)},
                     "transport": "plaintext", "headers": parse(b"[]"), "connect_ms": 10_000_u64, "handshake_ms": 10_000_u64}
                ]),
            ),
            ("/application/webhooks", json!({"listener": {"socket": format!("127.0.0.1:{}", facts.ports.webhooks)}})),
            ("/application/clients/listener/socket", json!(format!("127.0.0.1:{}", facts.ports.clients))),
            ("/application/clients/bundle", Value::Null),
            ("/workers/0/secret", json!("worker.secret")),
        ] {
            set(&mut expected, path, value);
        }
        for member in
            ["charters", "templates", "policies", "rules", "deployment", "providers", "connectors", "acts", "projects"]
        {
            expected["engine"].as_object_mut().unwrap().remove(member);
            assert!(actual["engine"].get(member).is_none(), "{row}: {member}");
        }
        assert!(actual["application"].get("demo").is_none(), "the box builds its playground in code");
        assert!(actual["application"].get("workers").is_none(), "{row}: programs are machine facts");
        assert_eq!(actual["workers"][0]["name"], "local", "{row}");
        assert!(actual["engine"]["hosts"].get("workers").is_none(), "{row}");
        assert_eq!(actual, expected, "{row}");
    }
}

#[test]
fn facts_are_written_as_json_text() {
    // configuration/moved.md, 2: machine paths stay out of the deployment document.
    for (row, binary) in [
        ("quote and backslash", b"/opt/tem\"per\\bin/temper".as_slice()),
        ("control bytes and UTF-8", b"/opt/\n\t\x01/\xc3\xa9/temper".as_slice()),
    ] {
        let facts = crate::Facts { binary, git: Some(binary), claude: Some(binary), ..fixtures::facts() };
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid text paths");
        let parsed = parse(content(&answered, b"temper.json"));
        assert!(parsed["application"].get("workers").is_none(), "{row}");
        assert_eq!(parsed["engine"]["llm"]["endpoints"][3]["address"], json!({"socket": "127.0.0.1:8370"}), "{row}");
        let crate::Content::Link { target } = &fixtures::file(&answered.files, b"demo/bin/git").content else {
            panic!("the fake git is a launcher link")
        };
        assert_eq!(target.as_ref(), binary, "{row}: the launcher preserves the machine path");
    }
}

#[test]
fn no_placeholder_is_left() {
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32]).expect("valid facts");
        assert_eq!(bytes::find(content(&answered, b"temper.json"), b"\"$"), None, "{row}");
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
        let document = parse(content(&answer, b"temper.json"));
        assert_eq!(document["engine"]["owners"][0]["subject"], json!(format!("{written}@localhost")), "{row}");
    }
}

#[test]
fn the_template_seeds_no_project_and_no_settings_members() {
    for template in [template::TEMPER, template::BARE] {
        let document = parse(template);
        for member in
            ["charters", "templates", "policies", "rules", "deployment", "providers", "connectors", "acts", "projects"]
        {
            assert!(document["engine"].get(member).is_none());
        }
        assert!(document["application"].get("landing").is_none());
        assert!(document["application"].get("welcome").is_none());
        assert!(document["application"].get("charter").is_none());
        for member in ["heartbeat_ms", "link", "approval"] {
            assert!(document["engine"]["clients"].get(member).is_none());
        }
    }
}

#[test]
fn the_bare_document_is_the_box_less_its_parts() {
    for (row, facts) in cases() {
        let boxed = files(&facts, &DIRECTORIES, &[0; 32]).expect("boxed files");
        let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).expect("bare files");
        let mut expected = parse(content(&boxed, b"temper.json"));
        assert!(expected["application"].get("welcome").is_none(), "{row}");
        expected["engine"]["clients"].as_object_mut().unwrap().remove("provider");
        expected["engine"]["owners"][0]["subject"] = json!("owner-a");
        expected["engine"].as_object_mut().unwrap().remove("acts");
        for member in ["providers", "endpoints"] {
            expected["engine"]["llm"][member].as_array_mut().unwrap().retain(|item| item["name"] != "demo");
        }
        let names = [b"worker.secret".as_slice(), b"temper.json"];
        assert_eq!(bare.len(), names.len(), "{row}");
        for (file, name) in bare.iter().zip(names) {
            assert_eq!(file.name, name, "{row}");
            let source = fixtures::file(&boxed.files, name);
            assert_eq!(file.directory, source.directory, "{row}");
            let crate::Content::Bytes { bytes, private } = &file.content else { panic!("bare files hold bytes") };
            assert_eq!(bytes::find(bytes, b"\"$"), None, "{row}: no placeholder");
            match name {
                b"temper.json" => {
                    assert_eq!(parse(bytes), expected, "{row}");
                    assert!(!private);
                }
                _ => assert_eq!(file, source, "{row}: shared secret and digest"),
            }
        }
    }
}

// protocol/configuration/stock.md: the seven declarations, including their document order.

fn anthropic(member: &str) -> Value {
    match member {
        "providers" => {
            parse(br#"{"name":"anthropic","number":2,"signed_in_plan":false,"manage_usage":"https://claude.ai/settings/usage","endpoint":"anthropic","direct":"anthropic-api","keep":["claude-*"],"drop":["*-20??????"],"sources":{"programs":["claude_code"],"keys":true}}"#)
        }
        "endpoints" => {
            parse(br#"{"name":"anthropic","number":2,"dialect":2,"provider":"anthropic","program":"claude_code","address":{"host":"api.anthropic.com","port":443},"transport":"tls","headers":[],"connect_ms":10000,"handshake_ms":10000,"server_name":"api.anthropic.com"}"#)
        }
        "direct" => {
            let mut endpoint = anthropic("endpoints");
            endpoint["name"] = json!("anthropic-api");
            endpoint["number"] = json!(4_u32);
            endpoint.as_object_mut().unwrap().remove("program");
            endpoint
        }
        _ => panic!("known LLM member"),
    }
}
#[test]
fn every_deployment_offers_claude_found_or_fetched() {
    // protocol/setup/init.md, 3: OpenAI, Anthropic and demo fit in one offer.
    const PROVIDERS: u32 = 3;
    for (row, facts) in cases() {
        let boxed = files(&facts, &DIRECTORIES, &[0; 32]).unwrap();
        let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).unwrap();
        for files in [boxed.files.as_ref(), bare.as_ref()] {
            let document = parse(file_bytes(files, b"temper.json"));

            // Providers and endpoints keep their order beside the seven model declarations.
            for (member, at) in [("providers", 1), ("endpoints", 1)] {
                assert_eq!(document["engine"]["llm"][member][at], anthropic(member), "{row}: {member}");
            }
            assert!(document["engine"]["llm"].get("models").is_none());
            assert_eq!(document["engine"]["llm"]["endpoints"][2], anthropic("direct"), "{row}: direct");
            assert!(document["application"].get("workers").is_none(), "{row}: programs are found at start");
            for file in files {
                assert_ne!(file.name, b"agent.json");
            }
            for member in ["agent", "limits"] {
                assert!(document["workers"][0].get(member).is_none(), "{row}: {member}");
            }
            let bytes = serde_json::to_vec(&document["engine"]["limits"]).unwrap();
            let parsed = jig_protocol_config::document(
                &bytes,
                &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
            )
            .unwrap();
            let parsed = jig_protocol_config::overrides::parse(&parsed, 0, &jig_protocol_config::Path::root());
            assert!(parsed.problems.is_empty(), "{row}: {:?}", parsed.problems);
            assert!(parsed.limits.core.documents.lines >= PROVIDERS, "{row}");
        }
    }
}

fn file_bytes<'a>(files: &'a [crate::File], name: &[u8]) -> &'a [u8] {
    let crate::Content::Bytes { bytes, .. } = &fixtures::file(files, name).content else { panic!("document bytes") };
    bytes
}

#[test]
fn a_box_keeps_an_empty_provider_for_clients_added_at_setup() {
    let facts = fixtures::facts();
    let answer = files(&facts, &DIRECTORIES, &[0; 32]).unwrap();
    let document = parse(content(&answer, b"temper.json"));
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
    let facts = crate::Facts { sign_in: false, ..fixtures::facts() };
    let answer = files(&facts, &DIRECTORIES, &[0; 32]).unwrap();
    let document = parse(content(&answer, b"temper.json"));
    assert!(document["engine"]["clients"]["provider"].is_null());
    assert!(document["application"].get("machine").is_none());
    assert_eq!(document["engine"]["owners"][0]["subject"], "ada@localhost");
}
