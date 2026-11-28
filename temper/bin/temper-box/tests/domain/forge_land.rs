//! Story discovery and evidence over values and scripted client-tool reads.
use crate::support;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    io,
    path::{Path, PathBuf},
};
use support::forge_land::{
    FileFacts, FileKind, last_readme_line, live_roots, require_chat_progress, require_closed_roots,
};

fn roots_read(tasks: &[u64]) -> Value {
    json!({"tasks": {"rows": tasks.iter().map(|task| json!({"chip": {"task": task}})).collect::<Vec<_>>(), "more": false}})
}

#[test]
fn catalogue_reads_preserve_scripted_call_counts() {
    use skein_fake_llm_domain::api::{Caching, Query, ToolChoice};
    use support::forge_land::{Seen, assert_requests};

    let words = b"the restarted chat";
    let query = Query {
        credential: Some(0),
        caching: Caching::Unscoped,
        model: b"gpt-6-luna".as_slice().into(),
        system: words.as_slice().into(),
        tools: Box::new([]),
        choice: ToolChoice::None,
        messages: Box::new([]),
        max_tokens: 20,
    };
    for lists in [0, 1, 2] {
        let seen = Seen {
            requests: vec![query.clone(), query.clone()],
            answers: vec![Ok(()); 2],
            lists,
            schemes: vec![Some(skein_llm::Scheme::Bearer); 2 + lists],
            credential_indexes: vec![Some(0); 2 + lists],
        };
        assert_requests(&seen, 2, words, false);
    }
}

#[test]
fn a_fresh_sittings_close_leaves_no_live_root() {
    let before = live_roots(&roots_read(&[])).expect("whole fresh roots");
    assert_eq!(before, BTreeSet::new());
    assert_eq!(require_closed_roots(&roots_read(&[]), &before), Ok(()));
    assert_eq!(
        require_closed_roots(&roots_read(&[7]), &before),
        Err("live root tasks this sitting left: [7]; earlier live root tasks missing: []".to_owned())
    );
}

#[test]
fn a_kept_store_keeps_an_earlier_sittings_live_chat() {
    let before = live_roots(&roots_read(&[1])).expect("whole kept roots");
    assert_eq!(before, BTreeSet::from([1]));
    assert_eq!(require_closed_roots(&roots_read(&[1]), &before), Ok(()));
    assert_eq!(
        require_closed_roots(&roots_read(&[1, 4]), &before),
        Err("live root tasks this sitting left: [4]; earlier live root tasks missing: []".to_owned())
    );
}

#[test]
fn a_sitting_never_closes_an_earlier_root() {
    let before = BTreeSet::from([1]);
    assert_eq!(
        require_closed_roots(&roots_read(&[]), &before),
        Err("live root tasks this sitting left: []; earlier live root tasks missing: [1]".to_owned())
    );
}

#[test]
fn a_roots_read_must_be_whole() {
    for read in [
        json!({"tasks": {"rows": [], "more": true}}),
        json!({"tasks": {"more": false}}),
        json!({"tasks": {"rows": [{"chip": {}}], "more": false}}),
    ] {
        assert!(live_roots(&read).is_err(), "{read}");
    }
}

#[test]
fn a_readme_ignores_trailing_empty_lines() {
    for (readme, expected) in [
        ("starter\ntag\n\n", Some("tag")),
        ("tag", Some("tag")),
        ("tag\n", Some("tag")),
        ("tag\n\n\n", Some("tag")),
        ("tag\r\n", Some("tag")),
        ("tag\r\n\r\n", Some("tag")),
        ("", None),
        ("\n", None),
        ("\n\n", None),
        ("\r\n", None),
    ] {
        assert_eq!(last_readme_line(readme.as_bytes()), expected, "README {readme:?}");
    }
}

#[test]
fn a_readme_keeps_its_last_nonempty_line_exactly() {
    for (readme, expected) in [
        ("tag\nother\n\n", "other"),
        ("tag \n\n", "tag "),
        (" tag\n", " tag"),
        ("tag\n  \n", "  "),
        ("tag\n\t\n\n", "\t"),
    ] {
        assert_eq!(last_readme_line(readme.as_bytes()), Some(expected), "README {readme:?}");
    }
}

// The facts read by discovery, in task show --json's owner shape. The encoders live
// in temper-client-tools-jig/src/jig/render/document/{chips,owner_chips,
// owner_authority,tasks,parts,llm}.rs.
fn empty_park() -> Value {
    json!({
        "chip": {"details": {"owner": {
            "phase": {"active": "idle"},
            "details": {"attempt": 1, "live_delegates": 0}
        }}},
        "waits_on": "news",
        "run_wait": null,
        "subscriptions": {"entries": []},
        "cut": false,
        "wake": {
            "words": "immediate", "notices": "immediate", "news": "immediate",
            "results": "last_or_failure", "questions": true, "answers": true, "timers": true
        }
    })
}

#[test]
fn an_empty_park_stops_the_story() {
    assert_eq!(require_chat_progress(&empty_park()), Err("chat parked without delegating a change".to_owned()));
}

fn continues_with(pointer: &str, replacement: Value) {
    let mut task = empty_park();
    *task.pointer_mut(pointer).expect("fixture field") = replacement;
    assert_eq!(require_chat_progress(&task), Ok(()), "{pointer}: {task}");
}

#[test]
fn active_work_and_intentional_waits_continue() {
    continues_with("/chip/details/owner/details/attempt", Value::Null);
    continues_with("/chip/details/owner/details/live_delegates", json!(1));
    for stage in [
        json!("due"),
        json!("preparing"),
        json!({"claimed": {"attempt": 1}}),
        json!({"running": {"attempt": 1}}),
        json!({"backing_off": {"until": 10}}),
    ] {
        continues_with("/chip/details/owner/phase/active", stage);
    }
    let mut running = empty_park();
    running["chip"]["details"]["owner"]["phase"] = json!({"active": {"running": {"attempt": 1}}});
    for work in [
        json!("preparing"),
        json!("starting"),
        json!("active"),
        json!("writing"),
        json!({"tool": {"call": "01", "tool": "02"}}),
        json!("waiting"),
        json!("ending"),
    ] {
        running["chip"]["details"]["owner"]["details"]["observation"] =
            json!({"channel": 1, "work": work, "since": 1, "reported_at": 2, "started": 1});
        assert_eq!(require_chat_progress(&running), Ok(()), "{running}");
    }
    continues_with(
        "/chip/details/owner/phase",
        json!({"waiting": {"dependencies": [2], "more": false, "holds": false}}),
    );
    continues_with(
        "/chip/details/owner/phase",
        json!({"person": {"address": {"party": 1}, "taken_by": null, "since": null}}),
    );
    for waits in [
        json!("nothing"),
        json!("delegates"),
        json!({"dependencies": {"pending": [2], "count": 1}}),
        json!({"party": {"whom": {"person": 1}, "entries": 1, "task": 2, "kind": "question"}}),
    ] {
        continues_with("/waits_on", waits);
    }
    for wait in
        [json!("credential"), json!("host"), json!({"account": {"account": 1, "source": null, "why": "refreshing"}})]
    {
        continues_with("/run_wait", wait);
    }
    for kind in [
        json!({"timer": {"at": 10, "period": null}}),
        json!({"task": {"target": 2, "held": true, "result": true}}),
        json!({"topic": {"connector": 1, "topic": 2}}),
    ] {
        continues_with("/subscriptions/entries", json!([{"number": 1, "kind": kind}]));
    }
    continues_with("/cut", json!(true));
    for key in ["words", "notices", "news"] {
        for rule in [json!("never"), json!({"batch": {"count": 2, "age": 10}}), json!("unknown")] {
            continues_with(&format!("/wake/{key}"), rule);
        }
    }
    for rule in ["never", "each", "unknown"] {
        continues_with("/wake/results", json!(rule));
    }
    for key in ["questions", "answers", "timers"] {
        continues_with(&format!("/wake/{key}"), json!(false));
    }
    for key in ["words", "notices", "news", "results", "questions", "answers", "timers"] {
        let mut task = empty_park();
        task["wake"].as_object_mut().expect("wake object").remove(key);
        assert_eq!(require_chat_progress(&task), Ok(()), "missing wake {key}");
    }
    let mut task = empty_park();
    task["wake"]["future"] = json!(true);
    assert_eq!(require_chat_progress(&task), Ok(()), "unknown wake field");
    continues_with("/wake", Value::Null);
}

#[test]
fn a_terminal_chat_is_not_an_empty_park() {
    for phase in [
        json!({"held": {"reason": "drift", "revision": 1}}),
        json!({"closing": {"run": {"attempt": 1}}}),
        json!({"closing": "delegates"}),
        json!({"closing": "effects"}),
        json!({"closing": "releases"}),
        json!({"closing": "settled"}),
        json!("ended"),
    ] {
        continues_with("/chip/details/owner/phase", phase);
    }
}

#[test]
fn missing_chat_facts_report_the_shape_error() {
    for (parent, key) in [
        ("", "chip"),
        ("/chip", "details"),
        ("/chip/details", "owner"),
        ("/chip/details/owner", "phase"),
        ("/chip/details/owner", "details"),
        ("/chip/details/owner/details", "attempt"),
        ("/chip/details/owner/details", "live_delegates"),
        ("", "waits_on"),
        ("", "run_wait"),
        ("", "subscriptions"),
        ("/subscriptions", "entries"),
        ("", "cut"),
        ("", "wake"),
    ] {
        let mut task = empty_park();
        task.pointer_mut(parent).expect("fixture parent").as_object_mut().expect("object").remove(key);
        assert_eq!(require_chat_progress(&task), Err("chat progress facts missing".to_owned()), "{parent}/{key}");
    }
    for (pointer, malformed) in [
        ("/chip/details/owner/details/attempt", json!("1")),
        ("/chip/details/owner/details/attempt", json!(-1)),
        ("/chip/details/owner/details/live_delegates", Value::Null),
        ("/chip/details/owner/details/live_delegates", json!("0")),
        ("/subscriptions/entries", Value::Null),
        ("/cut", Value::Null),
    ] {
        let mut task = empty_park();
        *task.pointer_mut(pointer).expect("fixture field") = malformed;
        assert_eq!(require_chat_progress(&task), Err("chat progress facts missing".to_owned()), "{pointer}");
    }
}

#[test]
fn a_claude_plan_declares_without_import_or_sign_in() {
    use std::path::Path;
    use support::forge_land::{LiveModel, live_commands};
    let (commands, account_at) = live_commands(true, LiveModel::ClaudeCode { login: Path::new("/held/claude-login") });
    assert_eq!(
        &commands[account_at - 1],
        &["account", "declare", "2", "claude_code", "claude-login", "--key", "live-program"]
    );
    assert!(commands.iter().all(|args| args[0] != "program-login"
        && (args[0] != "account" || !matches!(args[1].as_str(), "connect" | "import"))));
    let (restart, _) = live_commands(false, LiveModel::ClaudeCode { login: Path::new("/held/claude-login") });
    assert!(
        restart.iter().all(
            |args| args[0] != "account" || !matches!(args[1].as_str(), "declare" | "connect" | "import" | "verify")
        )
    );
}

fn live_charter_rows(turns: [u32; 3]) -> [Value; 3] {
    std::array::from_fn(|at| {
        let number = [1, 2, 4][at];
        json!({
            "setting":{"kind":"charter","number":number},
            "value":{"instructions":format!("charter {number}"),"model":{"name":"old"},
                "alternatives":[{"name":"fallback"}],"rest":true,"time":1,
                "turns":turns[at],"reserve_turns":1}
        })
    })
}

#[test]
fn a_fresh_sitting_selects_its_model_in_every_charter() {
    use serde_json::json;
    use std::path::Path;
    use support::forge_land::{LiveChange, LiveModel, live_settings};

    let mut lines: Vec<_> = (1..=8).map(|number| json!({"setting":{"kind":"model","number":number}})).collect();
    let charters = live_charter_rows([64, 800, 800]);
    lines.extend(charters.iter().map(|row| json!({"setting":row["setting"]})));
    let inventory = json!({"newest":0,"settings":lines});
    let stock_model = json!({
        "provider":2,"dialect":2,"endpoint":2,"program":"claude_code","name":"claude-fable-5-1",
        "window":200_000,"output":8192,"input_price":3,"cached_price":1,"cache_write_price":4,
        "output_price":15,"price_unit":1_000_000,"efforts":["low","medium","high","xhigh","max"],"effort":"high"
    });
    let mut gpt_model = stock_model.clone();
    gpt_model["name"] = "gpt-6-luna".into();
    let mut haiku_model = stock_model.clone();
    haiku_model["name"] = "claude-haiku-5-5".into();
    haiku_model["effort"] = "low".into();
    let charter_items = |model: &serde_json::Value| {
        let [chat, producer] = [(1, 5), (2, 10)].map(|(number, turns)| {
            json!({
                "setting":{"kind":"charter","number":number},
                "set":{"instructions":format!("charter {number}"),"model":model,
                    "alternatives":[],"rest":false,"time":1,"turns":turns,"reserve_turns":1}
            })
        });
        let mut reviewer = charters[2]["value"].clone();
        reviewer["turns"] = 4.into();
        [chat, producer, json!({"setting":charters[2]["setting"],"set":reviewer})]
    };
    let gpt_items = charter_items(&gpt_model);
    let haiku_items = charter_items(&haiku_model);
    let claude = LiveModel::ClaudeCode { login: Path::new("/held/claude-login") };
    let cases = [
        (
            LiveModel::ChatGpt { account: None },
            "gpt-6-luna",
            "gpt-6-luna",
            Ok(vec![LiveChange { key: "live-settings".into(), file: json!({"items":gpt_items}) }]),
        ),
        (
            claude,
            "claude-fable-5-1",
            "claude-haiku-5-5",
            Ok(vec![
                LiveChange {
                    key: "live-model".into(),
                    file: json!({"items":[
                        {"setting":{"kind":"model","number":64},"set":{
                            "number":64,"model":haiku_model,"shown":null,"description":null,
                            "picker":true,"withdrawn":false,"source":"person","estimated":true
                        }}
                    ]}),
                },
                LiveChange { key: "live-settings".into(), file: json!({"items":haiku_items}) },
            ]),
        ),
        (
            claude,
            "another-model",
            "claude-haiku-5-5",
            Err("the selected live stock model: claude-fable-5-1".to_owned()),
        ),
    ];
    for (model, row_name, selected, expected) in cases {
        let mut row_model = stock_model.clone();
        row_model["name"] = row_name.into();
        let number = if row_name == "gpt-6-luna" { 7 } else { 3 };
        let row = json!({"setting":{"kind":"model","number":number},"value":{
            "number":number,"model":row_model,"shown":"Stock model","description":"Stock description",
            "picker":false,"withdrawn":true,"source":"stock","estimated":false
        }});
        assert_eq!(model.name(), selected, "chat's --model for {row_name}");
        assert_eq!(live_settings(model, &inventory, charters.each_ref(), &row), expected, "{row_name}");
    }
}

#[test]
fn a_kept_store_updates_only_out_of_bound_turns() {
    use support::forge_land::{kept_turn_settings, live_apply};
    let charters = live_charter_rows([64, 10, 800]);
    let change = kept_turn_settings(charters.each_ref(), 26).expect("two charters need new turns");
    let items = [(0, 5), (2, 4)].map(|(at, turns)| {
        let mut value = charters[at]["value"].clone();
        value["turns"] = turns.into();
        json!({"setting":charters[at]["setting"],"set":value})
    });
    assert_eq!(change.file, json!({"items":items}));
    assert_eq!(
        live_apply(&change, 0, 26, "turns.json"),
        ["settings", "apply", "turns.json", "--against", "26", "--key", "live-turns-26"]
    );
}

#[test]
fn a_kept_store_at_the_bound_applies_nothing() {
    let charters = live_charter_rows([5, 10, 4]);
    assert_eq!(support::forge_land::kept_turn_settings(charters.each_ref(), 26), None);
}

fn activity_turn(attempt: u64, turn: u64) -> Value {
    json!({"event":{"owner":{"source":{"line":{"turn":{"attempt":attempt,"turn":turn}}}}}})
}

#[test]
fn completion_notice_groups_distinct_turns_by_task_and_attempt() {
    use support::live::CompletionTurns;
    let mut turns = CompletionTurns::default();
    for (task, rows) in [
        (9, vec![activity_turn(1, 1)]),
        (7, vec![activity_turn(1, 1), activity_turn(2, 1)]),
        (7, vec![activity_turn(1, 1), activity_turn(1, 2), json!({"event":{"owner":{}}})]),
    ] {
        turns.read(task, &json!({"rows":rows})).expect("activity rows");
    }
    assert_eq!(turns.summary(7, 8), Ok((4, "COMPLETIONS chat=7 change=8 count=4 turns=7/1:2 7/2:1 9/1:1".to_owned())));
}

type SettingsWorld =
    jig_client_kit::harness::World<temper_client_world::app::Temper, temper_client_world::front::Joint>;

fn live_settings_world() -> (SettingsWorld, jig_client_kit::harness::ClientId) {
    use jig_client_kit::harness::{Settings, World};
    use temper_client_world::{fixtures, front};
    use temper_engine_domain_world::{store::Store, walking};

    let init = temper_config_shell::init_document(false);
    let document = jig_protocol_config::document(&init, &support::configuration_fixture::DOCUMENT).expect("init JSON");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("init object")
        .member(&document, b"engine")
        .expect("init engine");
    let given = temper_service::stock::Given::Stock.with_providers(
        temper_service::stock::providers(),
        Box::new([temper_service::stock::scripted(([127, 0, 0, 1], 8370).into())]),
    );
    let engine =
        temper_service::startup::engine_with(&document, at, &jig_protocol_config::Directories::working(), &given)
            .expect("stock engine");
    let mut limits = front::engine_limits();
    limits.people.initial_owners = 2;
    limits.accounts.accounts = 3;
    limits.accounts.holder_accounts = 3;
    limits.accounts.credentials = 3;
    limits.authority.accounts = 3;
    limits.fleet.grants = limits.accounts.credentials + limits.accounts.accounts;
    // Each extra grant reserves room in the fleet's four routing cohorts.
    limits.journal.writes += (limits.fleet.grants - 2) * 4;
    limits.fleet.reads = limits.accounts.accounts;
    limits.tasks.charters = 3;
    limits.templates = 3;
    limits.documents.text_bytes = 8192;
    limits.journal.run_bytes = engine.limits.core.run_bytes;
    let page_bytes = u32::try_from(engine.limits.core.load_bytes).expect("stock settings page fits u32");
    limits.loads.bytes = page_bytes;
    limits.loads.reply_bytes = page_bytes;
    limits.loads.transcript_bytes = page_bytes;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        config.core.owners = Box::new([
            config.core.owners[0].clone(),
            jig_domain_people::InitialOwner { project: 0, identity: config.core.owners[0].identity.clone() },
        ]);
        let core = jig_protocol_config::core(&engine, config.core.deployment, fixtures::SEED);
        config.core.defaults.models = core.defaults.models;
        config.core.defaults.charters = core.defaults.charters.into_vec().into_iter().take(3).collect();
        config.core.defaults.providers = core.defaults.providers;
        config.core.fixed_providers = core.fixed_providers;
        config.core.settings.models = core.settings.models;
        config.core.settings.endpoints = core.settings.endpoints;
        config.core.settings.llm_providers = core.settings.llm_providers;
        for template in &mut config.core.defaults.templates {
            template.llm = None;
        }
        config
    });
    let front = front::Joint::with_engine(
        fixtures::SEED,
        &fixtures::domain_limits(),
        Store::with_limits(&limits),
        configs,
        &limits,
        1,
    );
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    let agent = fixtures::agent(&mut world);
    (world, agent)
}

fn settings_read(world: &mut SettingsWorld, agent: jig_client_kit::harness::ClientId, words: &[&[u8]]) -> Value {
    use temper_client_world::fixtures;
    let call = fixtures::give(world, agent, words);
    let printed = fixtures::answered(world, agent, call);
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(printed.stderr.is_empty(), "{printed:?}");
    serde_json::from_slice(&printed.stdout).expect("settings show JSON")
}

fn settings_apply(
    world: &mut SettingsWorld,
    agent: jig_client_kit::harness::ClientId,
    change: &support::forge_land::LiveChange,
    at: usize,
    newest: u64,
) -> temper_client_tools::jig::Printed {
    use temper_client_world::fixtures;
    let file = format!("{}.json", change.key);
    let argv = support::forge_land::live_apply(change, at, newest, &file);
    let argv: Vec<_> = argv.iter().map(String::as_bytes).collect();
    let bytes = serde_json::to_vec(&change.file).expect("live settings bytes");
    let call = fixtures::give_with_file(world, agent, &argv, &bytes);
    fixtures::answered(world, agent, call)
}

fn settings_fixture(
    world: &mut SettingsWorld,
    agent: jig_client_kit::harness::ClientId,
    model: support::forge_land::LiveModel<'_>,
) -> (u64, [Value; 3], Value, Vec<support::forge_land::LiveChange>) {
    let inventory = settings_read(world, agent, &[b"settings", b"show", b"--json"]);
    let charters = [b"1".as_slice(), b"2", b"4"]
        .map(|number| settings_read(world, agent, &[b"settings", b"show", b"charter", number, b"--json"]));
    let row = settings_read(world, agent, &[b"settings", b"show", b"model", b"3", b"--json"]);
    let changes =
        support::forge_land::live_settings(model, &inventory, charters.each_ref(), &row).expect("live model settings");
    (inventory["newest"].as_u64().expect("newest"), charters, row, changes)
}

#[test]
fn claude_and_key_model_settings_are_admitted_on_stock() {
    use std::path::Path;
    use support::forge_land::LiveModel;
    for model in [
        LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") },
        LiveModel::ClaudeCode { login: Path::new("/held/claude-login") },
    ] {
        let (mut world, agent) = live_settings_world();
        let (newest, charters, row, changes) = settings_fixture(&mut world, agent, model);
        let [add, select] = changes.as_slice() else { panic!("model then charters: {changes:?}") };
        assert_eq!((add.key.as_str(), select.key.as_str()), ("live-model", "live-settings"));
        let mut haiku = row["value"]["model"].clone();
        haiku["name"] = "claude-haiku-5-5".into();
        haiku["effort"] = "low".into();
        let value = json!({"number":64,"model":haiku,"shown":null,"description":null,
            "picker":true,"withdrawn":false,"source":"person","estimated":true});
        assert_eq!(add.file, json!({"items":[{"setting":{"kind":"model","number":64},"set":value}]}));
        let items = std::array::from_fn::<_, 3, _>(|at| {
            let charter = &charters[at];
            let mut selected = charter["value"].clone();
            selected["turns"] = [5, 10, 4][at].into();
            if at != 2 {
                selected["model"] = haiku.clone();
                selected["alternatives"] = json!([]);
                selected["rest"] = false.into();
            }
            json!({"setting":charter["setting"],"set":selected})
        });
        assert_eq!(select.file, json!({"items":items}));
        // temper/docs/design/testing/live-tier.md, 2: two chat activations, a closing reply, a producer and a reviewer.
        let policy = |at: usize, field: &str| select.file["items"][at]["set"][field].as_u64().expect(field);
        let planned_turns =
            2 * policy(0, "turns") + policy(0, "reserve_turns") + policy(1, "turns") + policy(2, "turns");
        assert_eq!(planned_turns, 25);
        for (at, change) in changes.iter().enumerate() {
            let applied = settings_apply(&mut world, agent, change, at, newest);
            assert_eq!(applied.status, 0, "{}: {applied:?}", change.key);
            assert!(applied.stderr.is_empty(), "{applied:?}");
        }
        let inventory = settings_read(&mut world, agent, &[b"settings", b"show", b"--json"]);
        assert_eq!(inventory["newest"], newest + 2);
        let row = settings_read(&mut world, agent, &[b"settings", b"show", b"model", b"64", b"--json"]);
        assert_eq!(row["value"], value);
        for (number, item) in [b"1".as_slice(), b"2", b"4"].into_iter().zip(items) {
            let charter = settings_read(&mut world, agent, &[b"settings", b"show", b"charter", number, b"--json"]);
            // `rest` can add the newly catalogued model to the reviewer's resolved alternatives.
            for field in ["model", "rest", "turns", "reserve_turns"] {
                assert_eq!(charter["value"][field], item["set"][field], "charter field {field}");
            }
            if number != b"4" {
                assert_eq!(charter["value"]["alternatives"], json!([]));
            }
        }
    }
}

#[test]
fn a_live_model_file_reusing_the_stock_name_is_refused() {
    let (mut world, agent) = live_settings_world();
    let model = support::forge_land::LiveModel::AnthropicKey { key: std::path::Path::new("/private/synthetic-key") };
    let (newest, _, _, mut changes) = settings_fixture(&mut world, agent, model);
    changes[0].file["items"][0]["set"]["model"]["name"] = "claude-fable-5-1".into();
    let applied = settings_apply(&mut world, agent, &changes[0], 0, newest);
    assert_eq!(applied.status, 2);
    assert_eq!(applied.stderr.as_ref(), b"settings apply: out of shape; item 0; model 64: shape\n");
    let inventory = settings_read(&mut world, agent, &[b"settings", b"show", b"--json"]);
    assert_eq!(inventory["newest"], newest);
}

#[test]
fn an_anthropic_list_can_make_haiku_the_duplicate_at_model_63() {
    use jig_domain_accounts::{Change, Event, Holder};
    use jig_domain_link::{EndpointBound, ListOutcome, ListedModel};
    use support::forge_land::LiveModel;
    use temper_client_world::fixtures;
    use temper_engine_domain_world::protocol;

    let (mut world, agent) = live_settings_world();
    let before = settings_read(&mut world, agent, &[b"settings", b"show", b"--json"]);
    // The model catalogue describes the program endpoint; a key's list needs the direct endpoint.
    let channel = fixtures::QUESTION_WORKER;
    let engine = &mut world.front().engine;
    engine.send(protocol::lost(channel));
    engine.settle();
    let bounds = jig_domain_link::Bounds {
        endpoints: Box::new([EndpointBound { endpoint: 4, window: 1_048_576, output: 8192 }]),
        turns: 1024,
        spend: 1_000_000_000,
        time: skein_lib::Duration::from_secs(86_400),
        waiting: skein_lib::Duration::from_secs(86_400),
        message_bytes: engine.env.limits.tasks.message_bytes,
        inbox: 128,
        ..protocol::bounds()
    };
    engine.send(protocol::hello_with(skein_lib::Token::new(2001), 0, 1, bounds, Box::new([])));
    engine.settle();
    // Read the world's bootstrap key's list explicitly; restored keys do not read at startup.
    engine.send(temper_engine_domain::Event::Account(Event::Change {
        request: skein_lib::Token::new(2000),
        holder: Holder::Deployment,
        change: Change::ReadLists { provider: 2 },
    }));
    engine.settle();
    assert_eq!(
        engine.protocol.list_reads.len(),
        1,
        "a usable key reads the stock list: {:?}",
        engine.protocol.accounts
    );
    let rows = [b"claude-another-model".as_slice(), b"claude-haiku-5-5"].map(|name| ListedModel {
        name: name.into(),
        shown: None,
        window: None,
        output: None,
        efforts: Box::new([]),
    });
    world.front().answer_read(ListOutcome::Listed(Box::new(rows)));
    let occupied = settings_read(&mut world, agent, &[b"settings", b"show", b"model", b"63", b"--json"]);
    assert_eq!(occupied["value"]["source"], "list");
    assert_eq!(occupied["value"]["model"]["name"], "claude-haiku-5-5");
    let model = LiveModel::AnthropicKey { key: std::path::Path::new("/private/synthetic-key") };
    let (newest, _, _, changes) = settings_fixture(&mut world, agent, model);
    assert_eq!(newest, before["newest"].as_u64().expect("newest") + 1);
    assert_eq!(changes[0].file["items"][0]["setting"]["number"], 62);
    let applied = settings_apply(&mut world, agent, &changes[0], 0, newest);
    // The whole-set check blames the later of the duplicates, not the offered item.
    assert_eq!(applied.status, 2);
    assert_eq!(applied.stderr.as_ref(), b"settings apply: out of shape; item 0; model 63: shape\n");
}

#[test]
fn the_key_bootstrap_admits_only_the_anthropic_list_policy_change() {
    let (mut world, agent) = live_settings_world();
    let inventory = settings_read(&mut world, agent, &[b"settings", b"show", b"--json"]);
    let newest = inventory["newest"].as_u64().expect("newest");
    let provider = &[b"settings".as_slice(), b"show", b"provider", b"2", b"--json"];
    let row = settings_read(&mut world, agent, provider);
    let mut expected = row["value"].clone();
    let value = expected.as_object_mut().expect("stock provider");
    assert_eq!(value.remove("keep"), Some(json!(["claude-*"])));
    assert_eq!(value.remove("drop"), Some(json!(["*-20??????"])));
    let change = support::forge_land::key_provider_change(&row).expect("provider 2 change");
    assert_eq!(change.key, "live-key-provider");
    assert_eq!(change.file, json!({"items":[{"setting":{"kind":"provider","number":2},"set":expected}]}));
    let applied = settings_apply(&mut world, agent, &change, 0, newest);
    assert_eq!(applied.status, 0, "{applied:?}");
    assert!(applied.stderr.is_empty(), "{applied:?}");
    let inventory = settings_read(&mut world, agent, &[b"settings", b"show", b"--json"]);
    assert_eq!(inventory["newest"], newest + 1);
    assert_eq!(settings_read(&mut world, agent, provider)["value"], expected);
}

fn program_accounts() -> Value {
    json!({"accounts":{
        "party":[{"number":7, "provider":2, "sources":[{"kind":"program", "name":"claude-login", "usable":true}]}],
        "program_hosts":[{"name":"worker", "connected":true, "programs":[{"name":"claude-login", "state":"usable"}]}]
    }})
}

#[test]
fn a_claude_source_is_verified_by_its_declared_account() {
    use support::forge_land::program_verification;
    assert_eq!(
        program_verification(std::path::Path::new("/held/claude-login"), &program_accounts()),
        Ok(vec![
            vec!["account", "verify", "7", "--source", "program", "--json"].into_iter().map(str::to_owned).collect(),
            vec!["account", "list", "--until", "usable", "--timeout", "600s", "--json"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            vec!["account", "list", "--json"].into_iter().map(str::to_owned).collect(),
        ])
    );
}

#[test]
fn a_claude_account_needs_its_login_usable_on_worker() {
    use support::forge_land::LiveModel;
    let model = LiveModel::ClaudeCode { login: std::path::Path::new("/held/claude-login") };
    assert_eq!(model.ready(&program_accounts()), Ok(()));
}

#[test]
fn a_claude_plan_refuses_missing_or_unusable_accounts() {
    use support::forge_land::LiveModel;
    let model = LiveModel::ClaudeCode { login: std::path::Path::new("/held/claude-login") };
    for (pointer, value, message) in [
        ("/accounts/party/0/provider", json!(1), "held Claude Code account missing"),
        ("/accounts/party/0/sources/0/name", json!("other-login"), "held Claude Code account missing"),
        ("/accounts/party/0/sources/0/usable", json!(false), "held Claude Code source is not usable"),
        ("/accounts/program_hosts/0/connected", json!(false), "held Claude Code login is not usable on the worker"),
        (
            "/accounts/program_hosts/0/programs/0/state",
            json!("needs_sign_in"),
            "held Claude Code login is not usable on the worker",
        ),
    ] {
        let mut accounts = program_accounts();
        *accounts.pointer_mut(pointer).expect("fixture field") = value;
        assert_eq!(model.ready(&accounts), Err(message.to_owned()), "{pointer}");
    }
}

#[test]
fn a_claude_plan_selects_provider_two_beside_demo_accounts() {
    use support::forge_land::{LiveModel, program_verification};
    let login = std::path::Path::new("/held/claude-login");
    let mut accounts = program_accounts();
    // The playground's key account, if listed, does not select the held program account.
    accounts["accounts"]["party"]
        .as_array_mut()
        .expect("party accounts")
        .insert(0, json!({"number":8, "provider":3, "sources":[{"kind":"key", "usable":true}]}));
    assert_eq!(LiveModel::ClaudeCode { login }.ready(&accounts), Ok(()));
    assert_eq!(
        program_verification(login, &accounts).expect("held account verification")[0],
        ["account", "verify", "7", "--source", "program", "--json"]
    );
}

#[test]
fn an_imported_records_path_is_printed_as_file() {
    use support::forge_land::printed_command;
    let words = |words: &[&str]| words.iter().map(|word| (*word).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        printed_command(&words(&["account", "import", "/held/record", "--key", "connect-plan"])),
        words(&["account", "import", "FILE", "--key", "connect-plan"])
    );
    for kept in [
        &["account", "list", "--json"][..],
        &["account", "connect", "1", "--key", "connect-plan"],
        &["account", "import"],
    ] {
        assert_eq!(printed_command(&words(kept)), words(kept));
    }
}

/// testing/live-tier.md: the chat's delegate is a change or the story stops, naming it.
#[test]
fn the_chats_delegate_is_a_change_or_the_story_names_it() {
    use support::forge_land::change_delegate;
    for (executor, expected) in [
        (json!({"procedure": {"connector": 1, "kind": 2}}), Ok(3)),
        (json!({"agent": {"charter": 1}}), Err("the chat's delegate task 3 is an agent's on charter 1, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
        (Value::Null, Err("delegate executor missing (chip.executor)".to_owned())),
        (json!({"job": {"kind": 1}}), Err("the chat's delegate task 3 is a job's, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
        (json!({"party": {"party": 1}}), Err("the chat's delegate task 3 is a party's, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
        (json!({"role": {"project": 1, "role": 2}}), Err("the chat's delegate task 3 is a role's, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
    ] {
        let row = json!({"chip": {"task": 3, "executor": executor}});
        assert_eq!(change_delegate(&row), expected, "{row}");
    }
}

fn delegate_read(rows: &[u64], cut: bool, after: Option<&str>) -> Value {
    json!({
        "live": [], "live_after": null,
        "ended": {"rows": {
            "rows": rows.iter().map(|task| json!({"chip": {"task": task}})).collect::<Vec<_>>(),
            "cut": cut,
            "after": after.map(|value| json!({"value": value}))
        }}
    })
}

/// testing/live-tier.md: the chat's and the change's ended delegates are read whole,
/// a page at a time, by the cursor each page prints.
#[test]
fn ended_delegates_are_read_whole_by_their_cursor() {
    use support::forge_land::EndedDelegates;
    let mut ended = EndedDelegates::new(2, [2, 3]);
    assert_eq!(ended.first(), ["task", "list", "--project", "2", "--parent", "2", "--ended", "--json"]);
    for (read, expected) in [
        (
            delegate_read(&[3], false, Some("3")),
            vec!["task", "list", "--project", "2", "--parent", "2", "--ended", "--json", "--after", "3"],
        ),
        (
            delegate_read(&[5], true, Some("5")),
            vec!["task", "list", "--project", "2", "--parent", "2", "--ended", "--json", "--after", "5"],
        ),
        (delegate_read(&[], false, None), vec!["task", "list", "--project", "2", "--parent", "3", "--ended", "--json"]),
    ] {
        assert_eq!(ended.read(&read), Ok(Some(expected.into_iter().map(str::to_owned).collect())), "{read}");
    }
    assert_eq!(ended.read(&delegate_read(&[7, 6], false, None)), Ok(None));
    assert_eq!(ended.whole(), [vec![3, 5], vec![6, 7]]);
}

/// testing/live-tier.md: what the story refuses of a delegate page, naming it.
#[test]
fn an_ended_delegate_read_refuses_live_rows_a_cut_without_a_cursor_and_a_33rd_page() {
    use support::forge_land::EndedDelegates;
    let mut live_row = delegate_read(&[], false, None);
    live_row["live"] = json!([{"chip": {"task": 4}}]);
    let mut live_after = delegate_read(&[], false, None);
    live_after["live_after"] = json!({"value": "4"});
    let mut missing_task = delegate_read(&[4], false, None);
    missing_task["ended"]["rows"]["rows"] = json!([{"chip": {}}]);
    let mut missing_rows = delegate_read(&[], false, None);
    missing_rows["ended"]["rows"]["rows"] = Value::Null;
    for (read, expected) in [
        (live_row, "no live delegates"),
        (live_after, "no live delegates"),
        (delegate_read(&[4], true, None), "ended delegate page cut without a cursor"),
        (missing_task, "delegate task missing (chip.task)"),
        (missing_rows, "ended delegate rows missing"),
    ] {
        let mut ended = EndedDelegates::new(2, [2, 3]);
        assert_eq!(ended.read(&read), Err(expected.to_owned()), "{read}");
    }
    // The bound holds within one parent and across both parents.
    for first in [delegate_read(&[3], false, Some("3")), delegate_read(&[], false, None)] {
        let mut ended = EndedDelegates::new(2, [2, 3]);
        assert!(ended.read(&first).expect("first delegate page").is_some());
        for _ in 1..EndedDelegates::PAGES {
            assert!(ended.read(&delegate_read(&[4], false, Some("4"))).expect("within page bound").is_some());
        }
        assert_eq!(
            ended.read(&delegate_read(&[], false, None)),
            Err("ended delegates exceed 32-page bound".to_owned())
        );
    }
}

#[test]
fn setup_resolves_the_adopted_project_on_fresh_and_seeded_deployments() {
    use support::first_repository::project_number;
    for (projects, name, expected) in [
        (json!([{"project":1,"name":{"bytes":"hello"}}]), "hello", 1),
        (json!([{"project":1,"name":{"bytes":"widget"}}]), "widget", 1),
        (json!([{"project":1,"name":{"bytes":"seed"}},{"project":8,"name":{"bytes":"hello"}}]), "hello", 8),
    ] {
        assert_eq!(project_number(&json!({"projects":projects}), name), Ok(expected));
    }
}

#[test]
fn setup_refuses_missing_or_ambiguous_adopted_projects() {
    use support::first_repository::project_number;
    for me in [
        json!({}),
        json!({"projects":[]}),
        json!({"projects":[{"project":1,"name":{"bytes":"seed"}}]}),
        json!({"projects":[{"project":0,"name":{"bytes":"hello"}}]}),
        json!({"projects":[{"project":1,"name":{"bytes":"hello"}},{"project":2,"name":{"bytes":"hello"}}]}),
    ] {
        assert!(project_number(&me, "hello").is_err(), "{me}");
    }
}

fn referee_me() -> jig_client::contract::Me {
    use jig_client::contract as c;
    use skein_lib::List;
    let words = c::DocumentText::complete(c::Text::new(b"Person".as_slice().into(), 6).expect("name"));
    c::Me {
        charters: skein_lib::List::with_capacity(0),
        party: c::MeParty { number: 1, kind: c::PartyKind::Person, login: words.clone(), name: words },
        projects: List::with_capacity(0),
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        words_bytes: 8192,
        sign_in_expires: skein_lib::Wall::from_nanos(86_400_000_000_000),
        needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: skein_lib::Duration::from_secs(60),
        deployment_role: None,
    }
}

fn referee_json(argv: &[&str], page: &jig_client::contract::ActivityPage) -> Value {
    use jig_client::{Event, Up};
    use skein_lib::{Env, List, Queue, Time, Wall};
    use temper_client_tools::{jig::call::Parsed, root};
    let mut tools = temper_client_tools::limits::tools();
    tools.rows = 1; // One row fills a page so the referee follows its cursor.
    let limits = root::Limits::new(temper_client_tools::limits::client(), tools);
    let mut root = root::Root::unlinked(&limits, 1).expect("client limits");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let words: Vec<_> = argv.iter().map(|word| word.as_bytes()).collect();
    let Parsed::Call { call, .. } = temper_client_tools::table::parse(&words).expect("read argv") else {
        panic!("read call")
    };
    let mut pending = std::collections::VecDeque::from([
        root::Event::Client { event: Event::StartUnlinked { kept: List::with_capacity(0) } },
        root::Event::Client { event: Event::SignIn { with: jig_client::With::Transport } },
        root::Event::Client { event: Event::Up { up: Up::Served { served: true } } },
        root::Event::Client {
            event: Event::Up {
                up: Up::Watched {
                    watch: skein_lib::Token::new(0),
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        referee_me(),
                    ))),
                },
            },
        },
        root::Event::Call { number: 1, call, json: true, value: None },
    ]);
    for _ in 0..32 {
        let event = pending.pop_front().expect("read still pending");
        let mut out = Queue::uncapped();
        root::step(&mut root, &env, event, &mut out);
        while let Some(request) = out.pop() {
            if let root::Request::Printed { printed, .. } = request {
                assert_eq!(printed.status, 0);
                return serde_json::from_slice(&printed.stdout).expect("rendered JSON");
            }
            let Some(up) = referee_response(request, page) else { continue };
            pending.push_back(root::Event::Client { event: Event::Up { up } });
        }
    }
    panic!("read did not settle")
}

fn referee_response(
    request: temper_client_tools::root::Request,
    page: &jig_client::contract::ActivityPage,
) -> Option<jig_client::Up> {
    use jig_client::{Down, UnlinkedRequest, Up, contract as c};
    let temper_client_tools::root::Request::Unlinked { request: UnlinkedRequest::Down { down } } = request else {
        return None;
    };
    Some(match down {
        Down::Open { watch, .. } => Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded },
        Down::Read { read, .. } => Up::ReadDone {
            read,
            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(page.clone())))),
        },
        Down::Save { .. } | Down::Erase { .. } | Down::Send { .. } | Down::Continue { .. } | Down::Close { .. } => {
            return None;
        }
    })
}

fn referee_turn() -> jig_client::contract::TurnRead {
    use jig_client::contract as c;
    use skein_lib::List;
    let text = |bytes: &[u8]| c::Text::new(bytes.into(), 128).expect("bounded text");
    let mut blocks = List::with_capacity(4);
    for (number, tool, failed, outcome) in [
        (1, b"change".as_slice(), false, Some(b"created".as_slice())),
        (2, b"delegate".as_slice(), true, Some(b"refused".as_slice())),
        (3, b"delegate".as_slice(), false, None),
    ] {
        blocks
            .push(c::Block::Call(c::Call {
                number,
                tool: text(tool),
                input: text(b"request"),
                outcome: outcome.map(text),
                failed,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("three calls");
    }
    blocks.push(c::Block::Text(text(b"The change landed."))).expect("reply");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).expect("assistant");
    c::TurnRead::Decoded(c::Turn {
        messages,
        used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        llm: None,
        cut: false,
    })
}

fn referee_activity() -> jig_client::contract::ActivityPage {
    use jig_client::contract as c;
    use skein_lib::{List, Wall};
    let turn = referee_turn();
    let name = c::TurnName { attempt: 1, number: 1 };
    let mut rows = List::with_capacity(1);
    rows.push(c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 41,
            position: 1,
            at: Wall::EPOCH,
            line: c::Activity::Turn { reply: 1, attempt: 1, turn: 1, read: None },
        }),
        decoded: Some(c::ArchivedTurn { task: 41, name, cumulative: 0, read: None, at: Wall::EPOCH, turn }),
    })
    .expect("one archived turn");
    c::ActivityPage { cut: false, rows, before: None, after: None }
}

#[test]
fn a_landing_referee_reads_committed_tools_beyond_the_conversation() {
    use support::forge_land::conversation::Evidence;
    let mut page = referee_activity();
    page.before = Some(jig_client::contract::Cursor::new(Box::from([0x11]), 1).expect("cursor"));
    let chat = referee_json(&["chat", "show", "41", "--json"], &page);
    assert!(
        chat.pointer("/activity/rows/0/event/owner/decoded/turn/decoded/messages/0/blocks/0/call/outcome")
            .expect("call")
            .is_null()
    );
    let mut evidence = Evidence::new(41, 0);
    assert_eq!(evidence.read(&chat).expect("conversation page"), Some(read_before("chat", "show")));
    assert!(evidence.tools.is_empty(), "conversation payload omission proves no successful call");
    page.before = None;
    let chat = referee_json(&["chat", "show", "41", "--before", "11", "--json"], &page);
    let detail = evidence.read(&chat).expect("conversation complete").expect("missing detail fetch");
    assert_eq!(detail, ["task", "activity", "41", "--json"]);
    page.before = Some(jig_client::contract::Cursor::new(Box::from([0x11]), 1).expect("cursor"));
    let argv: Vec<_> = detail.iter().map(String::as_str).collect();
    let activity = referee_json(&argv, &page);
    assert_eq!(evidence.read(&activity).expect("detail page"), Some(read_before("task", "activity")));
    assert_eq!(evidence.tools, BTreeSet::from(["change".to_owned()]), "failed and unfinished delegates do not count");
    assert_eq!(evidence.failed, BTreeSet::from(["delegate".to_owned()]), "failed committed calls count");
    assert_eq!(support::forge_land::require_chat_change(&evidence), Ok(()));
    for (tools, failed, expected) in [
        (
            vec!["goal"],
            vec!["delegate"],
            "the chat's committed change call succeeded: its committed calls succeeded [goal] and failed [delegate]",
        ),
        (
            vec!["read_branch", "goal"],
            vec!["change"],
            "the chat's committed change call succeeded: its committed calls succeeded [goal, read_branch] and failed [change]",
        ),
        (
            vec![],
            vec!["read_branch", "change"],
            "the chat's committed change call succeeded: its committed calls succeeded [] and failed [change, read_branch]",
        ),
        (
            vec!["goal"],
            vec![],
            "the chat's committed change call succeeded: its committed calls succeeded [goal] and failed []",
        ),
        (vec![], vec![], "the chat's committed change call succeeded: its committed calls succeeded [] and failed []"),
    ] {
        evidence.tools = tools.into_iter().map(str::to_owned).collect();
        evidence.failed = failed.into_iter().map(str::to_owned).collect();
        assert_eq!(support::forge_land::require_chat_change(&evidence), Err(expected.to_owned()));
    }

    page.before = None;
    let row = page.rows.get_mut(0).expect("activity row");
    row.source.line =
        jig_client::contract::Activity::RunEnded { attempt: 1, end: jig_client::contract::RunEnd::Parked, spent: 0 };
    row.decoded = None;
    let activity = referee_json(&["task", "activity", "41", "--before", "11", "--json"], &page);
    assert_eq!(evidence.read(&activity), Ok(None));
    assert_eq!(evidence.replies.len(), 1, "reply survives the detail reads");
}

fn read_before(group: &str, verb: &str) -> Vec<String> {
    [group, verb, "41", "--json", "--before", "11"].map(str::to_owned).to_vec()
}

#[test]
fn a_landing_referee_keeps_a_successful_raw_delegate_in_its_evidence() {
    let mut evidence = support::forge_land::conversation::Evidence::new(41, 0);
    let mut page = referee_activity();
    let chat = referee_json(&["chat", "show", "41", "--json"], &page);
    assert!(evidence.read(&chat).expect("conversation read").is_some());
    let turn = &mut page.rows.get_mut(0).expect("turn").decoded.as_mut().expect("archive").turn;
    let jig_client::contract::TurnRead::Decoded(turn) = turn else { panic!("decoded") };
    let block = turn.messages.get_mut(0).expect("assistant").blocks.get_mut(1).expect("delegate");
    let jig_client::contract::Block::Call(call) = block else { panic!("call") };
    call.failed = false;
    let activity = referee_json(&["task", "activity", "41", "--json"], &page);
    assert_eq!(evidence.read(&activity), Ok(None));
    assert_eq!(evidence.tools, BTreeSet::from(["change".to_owned(), "delegate".to_owned()]));
}

#[test]
fn a_landing_referee_requires_whole_bounded_activity() {
    use jig_client::contract as c;
    use support::forge_land::conversation::Evidence;
    let chat = referee_json(&["chat", "show", "41", "--json"], &referee_activity());
    for defect in ["page", "turn", "undecodable"] {
        let mut evidence = Evidence::new(41, 0);
        assert!(evidence.read(&chat).expect("conversation").is_some());
        let mut page = referee_activity();
        if defect == "page" {
            page.cut = true;
        } else {
            let turn = &mut page.rows.get_mut(0).expect("row").decoded.as_mut().expect("archive").turn;
            if defect == "undecodable" {
                *turn = c::TurnRead::Undecodable { why: c::TurnProblem::Malformed };
            } else {
                let c::TurnRead::Decoded(turn) = turn else { panic!("decoded") };
                turn.cut = true;
            }
        }
        let detail = referee_json(&["task", "activity", "41", "--json"], &page);
        assert!(evidence.read(&detail).is_err(), "reject incomplete {defect}");
    }
    let mut evidence = Evidence::new(41, 0);
    assert!(evidence.read(&chat).expect("conversation").is_some());
    let mut page = referee_activity();
    page.before = Some(c::Cursor::new(Box::from([0x11]), 1).expect("cursor"));
    let detail = referee_json(&["task", "activity", "41", "--json"], &page);
    for _ in 0..32 {
        assert!(evidence.read(&detail).expect("within page bound").is_some());
    }
    assert_eq!(evidence.read(&detail), Err("bounded conversation evidence pages".to_owned()));
}

#[test]
fn a_key_plan_adds_once_without_import_declaration_or_sign_in_and_prints_file() {
    use std::path::Path;
    use support::forge_land::{LiveModel, live_commands, printed_command};
    let model = LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") };
    let (commands, account_at) = live_commands(true, model);
    assert_eq!(commands[account_at - 2], ["account", "add-key", "2", "/private/synthetic-key", "--key", "live-key"]);
    assert_eq!(commands[account_at - 1], ["account", "list", "--until", "usable", "--timeout", "600s", "--json"]);
    assert_eq!(printed_command(&commands[account_at - 2]), ["account", "add-key", "2", "FILE", "--key", "live-key"]);
    assert!(commands.iter().all(|args| args[0] != "account" || matches!(args[1].as_str(), "add-key" | "list")));
    let (restart, _) = live_commands(false, model);
    assert!(restart.iter().all(|args| args[0] != "account" || args[1] == "list"));
}

#[test]
fn a_key_plan_requires_a_usable_provider_two_key_beside_demo_accounts() {
    use std::path::Path;
    use support::forge_land::LiveModel;
    let model = LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") };
    assert_eq!(model.ready(&json!({})), Err("person's accounts missing".into()));
    for party in [
        json!([]),
        json!([{"provider":3,"sources":[{"kind":"key","usable":true}]}]),
        json!([{"provider":2,"sources":[{"kind":"program","usable":true}]}]),
        json!([{"provider":2}]),
    ] {
        assert_eq!(
            model.ready(&json!({"accounts":{"party":party}})),
            Err("Anthropic provider 2 key source missing".into())
        );
    }
    for usable in [json!(false), Value::Null] {
        assert_eq!(
            model.ready(&json!({"accounts":{"party":[
                {"provider":3,"sources":[{"kind":"key","usable":true}]},
                {"provider":2,"sources":[{"kind":"key","usable":usable}]}
            ]}})),
            Err("Anthropic provider 2 key source is not usable".into())
        );
    }
    assert_eq!(
        model.ready(&json!({"accounts":{"party":[
            {"provider":3,"sources":[{"kind":"key","usable":true}]},
            {"provider":2,"sources":[{"kind":"key","usable":false}]},
            {"provider":2,"sources":[{"kind":"key","usable":true}]}
        ]}})),
        Ok(())
    );
}

#[test]
fn the_live_temper_profile_selects_only_the_key_and_claude_code_stories() {
    let config =
        std::fs::read_to_string(skein_scratch::paths::tree().join("../.config/nextest.toml")).expect("profiles");
    let profile =
        config.split("[profile.live-temper]\n").nth(1).expect("live-temper").split("\n[").next().expect("section");
    let filter = profile.lines().find(|line| line.starts_with("default-filter = ")).expect("default filter");
    assert_eq!(
        filter,
        "default-filter = \"(binary(=live) & (test(=a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge) | test(=a_person_asks_claude_code_to_land_a_change_on_the_owned_forge))) & package(/^temper(-|$)/)\""
    );
    assert!(!filter.contains("chatgpt"));
}

#[test]
fn a_held_chatgpt_record_is_admitted_only_as_a_private_regular_file_never_named() {
    use support::forge_land::held_account_with;
    let file = Path::new("/fixture/private/record");
    assert_eq!(
        held_account_with(file, |path| {
            assert_eq!(path, file);
            Err(io::ErrorKind::NotFound.into())
        }),
        Err("TEMPER_TEST_LIVE_ACCOUNT names no file")
    );
    for (kind, mode, admitted) in [
        (FileKind::File, 0o600, Ok(())),
        (FileKind::File, 0o640, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (FileKind::File, 0o644, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (FileKind::File, 0o400, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (FileKind::File, 0o700, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (FileKind::Symlink, 0o777, Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file")),
        (FileKind::Directory, 0o700, Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file")),
    ] {
        let result = held_account_with(file, |path| {
            assert_eq!(path, file);
            Ok(FileFacts { kind, mode, uid: 12, len: 6 })
        });
        assert_eq!(result, admitted, "{kind:?} at {mode:o}");
    }
}

#[test]
fn an_anthropic_key_is_admitted_by_metadata_without_naming_its_path() {
    use support::forge_land::anthropic_key_with;
    let file = Path::new("/fixture/private/key");
    let user = 12;
    let owned_key = FileFacts { kind: FileKind::File, mode: 0o600, uid: user, len: 14 };
    assert_eq!(
        anthropic_key_with(None, |_| panic!("unset skips the stat"), || panic!("unset skips the user read")),
        Ok(None)
    );
    for (metadata, admitted) in [
        (None, Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY names no file")),
        (Some(owned_key), Ok(Some(file.to_owned()))),
        (Some(FileFacts { mode: 0o644, ..owned_key }), Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not 0600")),
        (
            Some(FileFacts { kind: FileKind::Symlink, mode: 0o777, ..owned_key }),
            Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link"),
        ),
        (
            Some(FileFacts { kind: FileKind::Directory, mode: 0o700, ..owned_key }),
            Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link"),
        ),
        (Some(FileFacts { uid: 13, ..owned_key }), Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not owned by this user")),
        (Some(FileFacts { len: 0, ..owned_key }), Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is empty")),
    ] {
        let result = anthropic_key_with(
            Some(file.into()),
            |path| {
                assert_eq!(path, file);
                metadata.ok_or_else(|| io::ErrorKind::NotFound.into())
            },
            || user,
        );
        assert_eq!(result, admitted, "{metadata:?}");
    }
}

struct StoreFixture {
    root: PathBuf,
    entries: RefCell<BTreeMap<PathBuf, FileFacts>>,
}

impl StoreFixture {
    fn new() -> Self {
        Self { root: PathBuf::from("/fixture"), entries: RefCell::default() }
    }

    fn put(&self, path: PathBuf, kind: FileKind, mode: u32) {
        self.entries.borrow_mut().insert(path, FileFacts { kind, mode, uid: 12, len: 0 });
    }

    fn stat(&self, path: &Path) -> io::Result<FileFacts> {
        self.entries.borrow().get(path).copied().ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn create(&self, path: &Path, mode: u32, recursive: bool) -> io::Result<()> {
        assert!(path.starts_with(&self.root));
        let missing = self.stat(path).is_err();
        if recursive {
            for directory in path.ancestors().take_while(|directory| *directory != self.root) {
                if self.stat(directory).is_err() {
                    self.put(directory.to_owned(), FileKind::Directory, mode);
                }
            }
        } else if missing {
            self.put(path.to_owned(), FileKind::Directory, mode);
        } else {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        Ok(())
    }

    fn story(&self, stores: &Path, model: support::forge_land::LiveModel<'_>) -> Result<(PathBuf, bool), String> {
        support::forge_land::story_store_with(
            stores,
            model,
            |path, mode, recursive| self.create(path, mode, recursive),
            |path| self.stat(path),
            |path| Ok(self.entries.borrow().keys().any(|entry| entry.parent() == Some(path))),
        )
    }
}

#[test]
fn each_landing_story_takes_its_own_store_under_one_live_store() {
    use support::forge_land::LiveModel;
    let files = StoreFixture::new();
    let stores = files.root.join("parent/stores");
    let chatgpt = LiveModel::ChatGpt { account: None };
    let claude = LiveModel::ClaudeCode { login: Path::new("/held/claude-login") };
    let chatgpt_store = stores.join("forge-land");
    assert_eq!(files.story(&stores, chatgpt), Ok((chatgpt_store.clone(), false)));
    for directory in [stores.parent().expect("stores parent"), &stores, &chatgpt_store] {
        let metadata = files.stat(directory).expect("private directory");
        assert_eq!(metadata.kind, FileKind::Directory);
        assert_eq!(metadata.mode, 0o700);
    }
    files.put(chatgpt_store.join("begun"), FileKind::File, 0o600);
    assert_eq!(files.story(&stores, claude), Ok((stores.join("forge-land-claude"), false)));
    let key = LiveModel::AnthropicKey { key: Path::new("/private/key") };
    let key_store = stores.join("forge-land-claude-key");
    assert_eq!(files.story(&stores, key), Ok((key_store.clone(), false)));
    assert_eq!(files.story(&stores, key), Ok((key_store.clone(), false)), "an empty store is fresh");
    files.put(key_store.join("begun"), FileKind::File, 0o600);
    assert_eq!(files.story(&stores, key), Err(format!("kept store {} lacks config", key_store.display())));
    let missing = format!("kept store {} lacks config", chatgpt_store.display());
    assert_eq!(files.story(&stores, chatgpt), Err(missing.clone()));
    assert!(files.stat(&chatgpt_store.join("config")).is_err(), "a kept store is never repaired");
    assert_eq!(files.story(&stores, LiveModel::ChatGpt { account: Some(Path::new("/held/record")) }), Err(missing));
}

#[test]
fn a_key_store_is_kept_without_a_held_record() {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    use support::forge_land::LiveModel;
    let files = StoreFixture::new();
    let stores = files.root.join("stores");
    let model = LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") };
    let (store, kept) = files.story(&stores, model).expect("fresh key store");
    assert!(!kept);
    for directory in ["config", "data", "cache", "data/deployment", "data/forge-1/forgejo", "data/box/started"] {
        files.put(store.join(directory), FileKind::Directory, 0o700);
    }
    for file in [
        format!("config/{}", temper_config_shell::FILE),
        "data/box/started/1".to_owned(),
        "data/forge-1/forgejo/app.ini".to_owned(),
        "data/forge-1/forgejo/forgejo.db".to_owned(),
        format!("cache/forgejo/{VERSION}/forgejo"),
        format!("cache/forgejo-runner/{RUNNER_VERSION}/forgejo-runner"),
    ] {
        files.put(store.join(file), FileKind::File, 0o600);
    }
    assert_eq!(files.story(&stores, model), Ok((store, true)), "a key has no held record to import again");
}

#[test]
fn setup_accepts_a_project_free_first_repository_row() {
    use skein_io::kernel::Exit;
    use support::forge_land::check_forgejo_setup_after_sign_in_with;
    let data = Path::new("/fixture/data");
    let row = json!({"forges":[{
        "number":1,"host":"git.example:3000","mode":"owned","state":"connected",
        "first":{"state":"made","org":"work","repository":"hello","id":1}
    }]});
    let mut calls = [
        String::new(),
        json!({"forges":[]}).to_string(),
        "setting up\nproject: 1 · hello\nAdopted work/hello on forge 1 as owned.\nIt is the project's home.\n"
            .to_owned(),
        String::new(),
        row.to_string(),
        "forge 1 · host: git.example:3000 · Forgejo · owned · Connected as temper (2)".to_owned(),
    ]
    .map(|stdout| support::Ended { exit: Some(Exit::Code(0)), stdout: stdout.into_bytes(), stderr: vec![] });
    let stderr = "forgejo-1: started\nrunner-1: started";
    let mut probes = Vec::new();
    let verdict = check_forgejo_setup_after_sign_in_with(
        "first",
        &calls,
        stderr,
        None,
        data,
        |path| {
            probes.push(path.to_owned());
            false
        },
        |_| {},
    );
    assert_eq!(verdict, Ok(2));
    assert_eq!(
        probes,
        ["admin_token", "api_token", "git_token", "user", "webhook_secret"].map(|name| data.join("forge-1").join(name))
    );
    let mut other_host = row;
    other_host["forges"][0]["host"] = json!("other.example:3000");
    calls[4].stdout = other_host.to_string().into_bytes();
    let finding = check_forgejo_setup_after_sign_in_with(
        "first",
        &calls,
        stderr,
        None,
        data,
        |_| panic!("mismatch precedes leaf checks"),
        |_| {},
    )
    .expect_err("the text row must name the JSON row's resource host");
    assert_eq!(
        finding,
        "first: forges row: forge 1 · host: git.example:3000 · Forgejo · owned · Connected as temper (2); box: forgejo-1: started\nrunner-1: started"
    );
}

struct ListedEntry {
    name: std::ffi::OsString,
    kind: Option<FileKind>,
}

impl support::forge_land::DirectoryEntry for ListedEntry {
    fn name(&self) -> std::ffi::OsString {
        self.name.clone()
    }

    fn kind(&self) -> io::Result<FileKind> {
        Ok(self.kind.expect("an excluded entry's kind is never read"))
    }
}

fn log_entries(names: &[&str]) -> Vec<io::Result<ListedEntry>> {
    names.iter().map(|name| Ok(ListedEntry { name: (*name).into(), kind: Some(FileKind::File) })).collect()
}

#[test]
fn run_failure_lines_keep_only_run_records_in_rotation_order() {
    use support::forge_land::run_failure_lines_with;
    let data = Path::new("/fixture/data");
    let logs = data.join("box/logs");
    let oldest = "temper worker: run 2 attempt 1 failed Unprepared(Transient): host local: git failed";
    let refused = "temper serve: task 7 run 2 attempt 1 refused Invalid(Start(Bounds)): oversized charter";
    let failed = "temper serve: task 7 run 2 attempt 1 failed Run(Model): anthropic 400 invalid_request_error";
    let version = "temper-box serve: task 8 run 3 attempt 2 refused: worker version mismatch";
    let mut text = BTreeMap::from([
        (logs.join("serve.log.2"), format!("ordinary step\n{oldest}\n")),
        (logs.join("serve.log.1"), format!("{refused}\n")),
        (
            logs.join("serve.log"),
            format!(
                "temper serve: listening on 127.0.0.1:8370\n\
                 temper serve: task 7 run 2 attempt 1 started\n\
                 temper serve: task seven run 2 attempt 1 failed Run(Model): not a record\n\
                 ordinary step quotes {failed}\n\
                 temper worker: run 2 attempt 1 refused Busy: not a worker failure\n\
                 {failed}\r\n{version}"
            ),
        ),
    ]);
    let mut opened = Vec::new();
    let lines = run_failure_lines_with(
        data,
        |path| {
            assert_eq!(path, logs);
            Ok(log_entries(&["serve.log.1", "serve.log", "serve.log.3.gz", "worker.log", "serve.log.2"]))
        },
        |path| {
            opened.push(path.to_owned());
            Ok(io::Cursor::new(text.remove(path).expect("only plain-text serve logs opened")))
        },
    );
    assert_eq!(lines, [oldest, refused, failed, version]);
    assert_eq!(opened, ["serve.log.2", "serve.log.1", "serve.log"].map(|name| logs.join(name)));
}

#[test]
fn run_failure_lines_redact_credentials_but_keep_run_context() {
    use std::fmt::Write as _;
    use support::forge_land::run_failure_lines_with;
    let data = Path::new("/fixture/data");
    let logs = data.join("box/logs");
    let context = "temper serve: task 7 run 2 attempt 1 failed Run(Model): ";
    let details = [
        ("secret=abc123 task-12", "secret=… task-12"),
        (r#"{"api_key":"k1"} task-12"#, r#"{"api_key":"…"} task-12"#),
        ("x-api-key: sk-ant-api03-xyz task-12", "x-api-key: … task-12"),
        ("sk-ant-api03-xyz task-12", "… task-12"),
        ("X-Api-Key:\theader-value task-12", "X-Api-Key:\t… task-12"),
        ("(sk-ant-api03-xyz), task-12", "(…), task-12"),
        ("task-12 wordsk-ant-not-a-key _sk-ant-not-a-key", "task-12 wordsk-ant-not-a-key _sk-ant-not-a-key"),
    ];
    let mut text = String::new();
    for (detail, _) in details {
        writeln!(text, "{context}{detail}").expect("fixture line");
    }
    let lines = run_failure_lines_with(
        data,
        |path| {
            assert_eq!(path, logs);
            Ok(log_entries(&["serve.log"]))
        },
        |path| {
            assert_eq!(path, logs.join("serve.log"));
            Ok(io::Cursor::new(text.as_bytes()))
        },
    );
    assert_eq!(lines, details.map(|(_, safe)| format!("{context}{safe}")));
    assert!(lines.iter().all(|line| !line.contains("abc123") && !line.contains("k1") && !line.contains("xyz")));
}

#[test]
fn run_failure_lines_allow_a_box_that_never_started() {
    use support::forge_land::run_failure_lines_with;
    let data = Path::new("/fixture/data");
    let absent = data.join("absent");
    for (root, listing) in [
        (absent.as_path(), Err(io::ErrorKind::NotFound.into())),
        (data, Err(io::ErrorKind::NotFound.into())),
        (data, Ok(log_entries(&[]))),
    ] {
        let lines = run_failure_lines_with(
            root,
            |path| {
                assert_eq!(path, root.join("box/logs"));
                listing
            },
            |path| -> io::Result<io::Cursor<&[u8]>> { panic!("nothing to open: {}", path.display()) },
        );
        assert_eq!(lines, Vec::<String>::new());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DataMember {
    Directory(u32),
    File(&'static [u8]),
    Link(PathBuf),
    Excluded,
}

struct DataFixture {
    source: PathBuf,
    entries: BTreeMap<PathBuf, DataMember>,
    kept: BTreeMap<PathBuf, DataMember>,
    listed: Vec<PathBuf>,
}

impl DataFixture {
    fn new(source: &Path) -> Self {
        use DataMember::{Directory, Excluded, File, Link};
        let entries = [
            ("", Directory(0o755)),
            ("box", Directory(0o755)),
            ("box/logs", Directory(0o755)),
            ("box/logs/serve.log", File(b"ordinary log\n\0\xff")),
            ("box/link", Link("../worker.secret".into())),
            ("box/directory-link", Link("../deployment".into())),
            ("box/nested", Directory(0o755)),
            ("deployment", Directory(0o755)),
            ("deployment/generations", Directory(0o755)),
            ("deployment/generations/1", Directory(0o755)),
            ("deployment/generations/1/store", Directory(0o755)),
            ("deployment/generations/1/store/logs", Directory(0o755)),
            ("deployment/generations/1/store/logs/2", File(b"store log\0\xff")),
            ("deployment/generations/1/secrets", Excluded),
            ("deployment/generations/1/secrets/s", Excluded),
            ("deployment/checkpoints", Excluded),
            ("deployment/checkpoints/1", Excluded),
            ("deployment/checkpoints/1/secrets", Excluded),
            ("deployment/checkpoints/1/secrets/s", Excluded),
            ("programs", Excluded),
            ("programs/live", Excluded),
            ("programs/live/x", Excluded),
            ("forge-1", Excluded),
            ("forge-1/admin_token", Excluded),
            ("forge-1/setup", Excluded),
            ("box/nested/secrets", Excluded),
            ("box/nested/checkpoints", Excluded),
            ("worker.secret", Excluded),
            ("provider.key", Excluded),
        ]
        .into_iter()
        .map(|(name, member)| (source.join(name), member))
        .collect();
        Self { source: source.to_owned(), entries, kept: BTreeMap::new(), listed: Vec::new() }
    }
}

impl support::forge_land::CopyFiles for DataFixture {
    type Entry = ListedEntry;
    type Entries = Vec<io::Result<ListedEntry>>;

    fn root_is_directory(&mut self, path: &Path) -> io::Result<bool> {
        assert_eq!(path, self.source);
        Ok(matches!(self.entries[path], DataMember::Directory(_)))
    }

    fn create_exclusive(&mut self, path: &Path, mode: u32) -> io::Result<()> {
        use std::collections::btree_map::Entry;
        match self.kept.entry(path.to_owned()) {
            Entry::Vacant(entry) => {
                entry.insert(DataMember::Directory(mode));
                Ok(())
            }
            Entry::Occupied(_) => Err(io::ErrorKind::AlreadyExists.into()),
        }
    }

    fn entries(&mut self, path: &Path) -> io::Result<Self::Entries> {
        assert!(matches!(self.entries[path], DataMember::Directory(_)), "only directories are listed");
        self.listed.push(path.to_owned());
        Ok(self
            .entries
            .iter()
            .filter(|(entry, _)| entry.parent() == Some(path))
            .map(|(entry, member)| {
                let name = entry.file_name().expect("listed entry name").into();
                let kind = match member {
                    DataMember::Directory(_) => Some(FileKind::Directory),
                    DataMember::File(_) => Some(FileKind::File),
                    DataMember::Link(_) => Some(FileKind::Symlink),
                    DataMember::Excluded => None,
                };
                Ok(ListedEntry { name, kind })
            })
            .collect())
    }

    fn read_link(&mut self, path: &Path) -> io::Result<PathBuf> {
        let DataMember::Link(target) = &self.entries[path] else { panic!("only links are read as links") };
        Ok(target.clone())
    }

    fn link(&mut self, target: &Path, path: &Path) -> io::Result<()> {
        self.kept.insert(path.to_owned(), DataMember::Link(target.to_owned()));
        Ok(())
    }

    fn copy(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        let DataMember::File(bytes) = self.entries[source] else { panic!("only included files are copied") };
        self.kept.insert(destination.to_owned(), DataMember::File(bytes));
        Ok(())
    }
}

#[test]
fn kept_box_data_is_private_credential_free_and_does_not_follow_links() {
    use support::forge_land::keep_box_data_with;
    let scratch = Path::new("/fixture/scratch");
    let source = scratch.join("source");
    let mut files = DataFixture::new(&source);
    let kept = keep_box_data_with(&source, scratch, &mut files).expect("kept data");
    assert_eq!(kept, scratch.join("data"));
    for log in ["box/logs/serve.log", "deployment/generations/1/store/logs/2"] {
        assert_eq!(files.kept[&kept.join(log)], files.entries[&source.join(log)]);
    }
    for link in ["box/link", "box/directory-link"] {
        let copy = &files.kept[&kept.join(link)];
        assert!(matches!(copy, DataMember::Link(_)));
        assert_eq!(*copy, files.entries[&source.join(link)]);
    }
    for excluded in ["worker.secret", "provider.key", "programs", "forge-1"] {
        assert!(!files.kept.contains_key(&kept.join(excluded)), "excluded {excluded}");
        assert!(files.entries.contains_key(&source.join(excluded)), "source stays: {excluded}");
    }
    assert_eq!(files.kept[&kept], DataMember::Directory(0o700));
    for (path, member) in files.kept {
        assert_ne!(path.file_name().expect("name"), "secrets");
        assert_ne!(path.file_name().expect("name"), "checkpoints");
        if let DataMember::Directory(mode) = member {
            assert_eq!(mode, 0o700, "{}", path.display());
        }
    }
}

#[test]
fn kept_box_data_never_overwrites_an_existing_destination() {
    use support::forge_land::keep_box_data_with;
    let scratch = Path::new("/fixture/scratch");
    let source = scratch.join("source");
    let mut files = DataFixture::new(&source);
    let destination = scratch.join("data");
    let earlier = BTreeMap::from([
        (destination.clone(), DataMember::Directory(0o755)),
        (destination.join("evidence"), DataMember::File(b"keep me")),
    ]);
    files.kept = earlier.clone();
    let why = keep_box_data_with(&source, scratch, &mut files).expect_err("existing destination refused");
    assert!(why.contains(&destination.display().to_string()), "{why}");
    assert!(files.listed.is_empty(), "existing destination prevents traversal");
    assert_eq!(files.kept, earlier, "earlier copy untouched");
}

#[test]
fn a_claude_configuration_replaces_openai_and_keeps_the_demo() {
    use std::path::Path;
    use support::forge_land::{LiveModel, LiveProfile};

    let data = Path::new("/fixture/live-claude/data");
    let executable = Path::new("/usr/bin/true");
    let init = LiveProfile::compose(data, LiveModel::ChatGpt { account: None }, executable);
    let login = data.join("programs/claude-login");
    let claude = LiveProfile::compose(data, LiveModel::ClaudeCode { login: &login }, executable);
    let agent = &claude.agent;
    let demo = init.agent["endpoints"]
        .as_array()
        .expect("init endpoints")
        .iter()
        .find(|endpoint| endpoint["name"] == "demo")
        .expect("init demo");
    assert_eq!(agent["programs"], json!({"claude_code":{"path":"fetched", "directory":data.join("programs")}}));
    assert_eq!(
        agent["endpoints"],
        json!([
            {"name":"claude", "number":2, "dialect":2, "provider":"anthropic", "program":"claude_code",
                "address":"api.anthropic.com:443", "transport":"tls", "server_name":"api.anthropic.com",
                "connect":10000, "handshake":10000, "headers":[],
                "limits": {
                    "window":1_048_576, "output":48_000, "reasoning_item":65536, "head":60000, "idle":300_000,
                    "oversized_reasoning":"fail"}},
            demo
        ])
    );
    assert_eq!(agent["profile"], init.agent["profile"]);
    assert_eq!(agent["environment"], init.agent["environment"]);
    assert_eq!(
        temper_agent_fixtures::providers::flat(&claude.providers),
        json!({
            "providers":[{"name":"anthropic", "number":2, "signed_in_plan":false,
                "manage_usage":"https://claude.ai/settings/usage", "endpoint":"claude",
                "sources":{"programs":["claude_code"]}}],
            "endpoints":[{"name":"claude", "number":2, "dialect":2, "provider":"anthropic", "program":"claude_code",
                "address":{"host":"api.anthropic.com", "port":443}, "transport":"tls", "server_name":"api.anthropic.com",
                "headers":[], "connect_ms":10000, "handshake_ms":10000}]
        })
    );
    assert_eq!(claude.document["application"], init.document["application"], "including init's demo");
    assert_eq!(claude.document["engine"]["limits"], init.document["engine"]["limits"]);
    assert_eq!(claude.document["workers"], init.document["workers"]);
    let prepared = support::prepare_agent(serde_json::to_vec(agent).expect("agent JSON").into());
    assert_eq!(prepared.agent.bounds.program_endpoints.as_ref(), [2]);
}

#[test]
fn stock_charters_keep_seven_models_and_choose_the_boxs_scripted_default() {
    use std::path::Path;
    use support::forge_land::{LiveModel, LiveProfile};

    let data = Path::new("/fixture/live-claude/data");
    let login = data.join("programs/claude-login");
    let profile = LiveProfile::compose(data, LiveModel::ClaudeCode { login: &login }, Path::new("/usr/bin/true"));
    let (engine, given) = profile.engine(&data.with_file_name("config"));
    let defaults = given.defaults(&engine.llm, engine.limits.core.tasks.result_bytes).expect("stock");
    let mut ordinary: Vec<_> = defaults
        .models
        .iter()
        .filter(|entry| entry.number != 8)
        .map(|entry| (entry.model.name.as_ref(), entry.model.endpoint))
        .collect();
    assert_eq!(ordinary.len(), 7);
    ordinary.sort();
    for charter in &defaults.charters {
        assert_eq!((charter.policy.model.name.as_ref(), charter.policy.model.endpoint), (b"demo".as_slice(), 3));
        let resolved = jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core)
            .expect("stock charter");
        let mut alternatives: Vec<_> =
            resolved.policy.alternatives.iter().map(|model| (model.name.as_ref(), model.endpoint)).collect();
        alternatives.sort();
        assert_eq!(alternatives, ordinary);
    }
}

#[test]
fn a_key_configuration_leaves_the_startup_copies_unchanged() {
    use std::path::Path;
    use support::forge_land::{LiveModel, LiveProfile};
    use temper_agent_fixtures::providers;

    let data = Path::new("/fixture/live-key/data");
    let init = LiveProfile::compose(data, LiveModel::ChatGpt { account: None }, Path::new(""));
    let key =
        LiveProfile::compose(data, LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") }, Path::new(""));
    assert_eq!(providers::flat(&key.providers), providers::flat(&init.providers), "the store owns the provider edit");
    assert_eq!(key.document, init.document, "the startup document is unchanged");
    assert_eq!(key.agent, init.agent, "Claude Code's program stays as init writes it");
}

#[test]
fn held_login_returns_before_stop_on_each_ending() {
    use skein_io::kernel::Exit;
    use skein_lib::Time;
    use std::path::Path;
    use support::{
        forge_land::{LiveModel, live_commands},
        held_login::{End, Ending},
    };

    let record = "/fixture/held/record";
    let model = LiveModel::ChatGpt { account: Some(Path::new(record)) };
    for (failed_box, expected) in [
        (None, vec![("connect-plan", Ok(())), ("restart-plan", Ok(()))]),
        (Some(true), vec![("connect-plan", Err("run failed"))]),
        (Some(false), vec![("connect-plan", Ok(())), ("restart-plan", Err("run failed"))]),
    ] {
        let mut ended = Vec::new();
        for first in [true, false] {
            let (commands, _) = live_commands(first, model);
            let import =
                commands.iter().find(|args| args.starts_with(&["account".into(), "import".into()])).expect("import");
            assert_eq!(&import[..4], ["account", "import", record, "--key"], "first={first}");
            let key = import[4].as_str();
            let failed = failed_box == Some(first);
            let now = Time::ZERO;
            let mut ending = Ending::new(true, first, failed, now);
            assert!(matches!(ending.next(now, false, None), End::Wait), "drain ctl before return: {key}");
            assert!(matches!(ending.next(now, true, None), End::Return), "return before stop: {key}");
            assert!(matches!(ending.next(now, true, None), End::Wait), "await give-back: {key}");
            let End::Stop(return_failure) = ending.next(now, true, Some(Some(Exit::Code(0)))) else {
                panic!("stop after the record returns: {key}");
            };
            let outcome = return_failure.or(failed.then_some("run failed")).map_or(Ok(()), Err);
            ended.push((key.to_owned(), outcome));
            if outcome.is_err() {
                break;
            }
        }
        let observations: Vec<_> = ended.iter().map(|(key, outcome)| (key.as_str(), *outcome)).collect();
        assert_eq!(observations, expected, "failed_box={failed_box:?}");
    }
}

#[test]
fn held_login_calls_use_the_sittings_client_directory() {
    use std::path::Path;
    use support::forge_land::{LiveModel, LivePlan};

    let store = Path::new("/fixture/stores/forge-land");
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    let run = store.join("first");
    let record = Path::new("/fixture/held/record");
    let binary = Path::new("/fixture/ctl-stub's path");
    let plan = LivePlan {
        paths: &paths,
        client: &run,
        first: true,
        kept: false,
        model: LiveModel::ChatGpt { account: Some(record) },
        address: None,
        tasks: None,
        baseline: None,
        roots: None,
        words: "",
        line: "",
    };
    let client = run.join("client");
    assert_eq!(plan.client_directory(), client);
    let arguments = ["account", "import", "/fixture/held/record", "--key", "connect-plan"];
    let import = plan.import_script(binary, &arguments);
    assert_eq!(import.call.program(), Some(store.join("import-held").as_path()));
    assert_eq!(import.call.arguments(), arguments.map(|word| Box::from(word.as_bytes())));
    let ctl = format!("'/fixture/ctl-stub'\"'\"'s path' ctl '{}'", client.display());
    assert!(import.script.lines().any(|line| line == format!("exec {ctl} \"$@\" </dev/null")));
    let hook = plan.return_hook(binary).expect("held give-back");
    assert_eq!(hook.path, store.join("give-back"));
    assert!(hook.script.contains(&format!("rung_account=$({ctl} account list --json </dev/null | jq")));
    assert!(
        hook.script
            .lines()
            .any(|line| line == format!("  {ctl} account export \"$rung_account\" '/fixture/held/record' </dev/null"))
    );
    let call = plan.return_call();
    assert_eq!(call.program(), Some(hook.path.as_path()));
    assert_eq!(call.arguments(), [Box::from(b"--box-ended".as_slice())]);
    let restart = LivePlan { first: false, ..plan };
    let call = restart.return_call();
    assert_eq!(call.program(), Some(hook.path.as_path()));
    assert!(call.arguments().is_empty(), "the restart's give-back is terminal");
}
