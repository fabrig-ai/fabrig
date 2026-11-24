//! Story discovery and evidence over values and scripted client-tool reads.
use crate::support;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use support::forge_land::{last_readme_line, live_roots, require_chat_progress, require_closed_roots};

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

#[test]
fn a_fresh_sitting_selects_its_model_in_every_charter() {
    use serde_json::json;
    use std::path::Path;
    use support::forge_land::{LiveChange, LiveModel, live_settings};

    let mut lines: Vec<_> = (1..=8).map(|number| json!({"setting":{"kind":"model","number":number}})).collect();
    lines.extend([json!({"setting":{"kind":"charter","number":1}}), json!({"setting":{"kind":"charter","number":2}})]);
    let inventory = json!({"newest":0,"settings":lines});
    let charters = [1, 2].map(|number| {
        json!({
            "setting":{"kind":"charter","number":number},
            "value":{"instructions":format!("charter {number}"),"model":{"name":"old"},
                "alternatives":[{"name":"fallback"}],"rest":true,"time":1}
        })
    });
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
        [1, 2].map(|number| {
            json!({
                "setting":{"kind":"charter","number":number},
                "set":{"instructions":format!("charter {number}"),"model":model,
                    "alternatives":[],"rest":false,"time":1}
            })
        })
    };
    let gpt_items = charter_items(&gpt_model);
    let haiku_items = charter_items(&haiku_model);
    let claude = LiveModel::ClaudeCode { login: Path::new("/held/claude-login") };
    let cases = [
        (
            LiveModel::ChatGpt { account: None },
            "gpt-6-luna",
            "gpt-6-luna",
            Ok(vec![LiveChange { key: "live-settings", file: json!({"items":gpt_items}) }]),
        ),
        (
            claude,
            "claude-fable-5-1",
            "claude-haiku-5-5",
            Ok(vec![
                LiveChange {
                    key: "live-model",
                    file: json!({"items":[
                        {"setting":{"kind":"model","number":64},"set":{
                            "number":64,"model":haiku_model,"shown":null,"description":null,
                            "picker":true,"withdrawn":false,"source":"person","estimated":true
                        }}
                    ]}),
                },
                LiveChange { key: "live-settings", file: json!({"items":haiku_items}) },
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
        assert_eq!(live_settings(model, &inventory, [&charters[0], &charters[1]], &row), expected, "{row_name}");
    }
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
        root::Event::Client { event: Event::Up { up: Up::Served { served: true, versions: None } } },
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
fn a_key_configuration_changes_only_anthropics_list_policy() {
    use std::path::Path;
    use support::forge_land::{LiveModel, LiveProfile};
    use temper_agent_fixtures::providers;

    let data = Path::new("/fixture/live-key/data");
    let init = LiveProfile::compose(data, LiveModel::ChatGpt { account: None }, Path::new(""));
    let key =
        LiveProfile::compose(data, LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") }, Path::new(""));
    let mut expected_providers = providers::flat(&init.providers);
    let provider = expected_providers["providers"]
        .as_array_mut()
        .expect("providers")
        .iter_mut()
        .find(|provider| provider["name"] == "anthropic")
        .expect("Anthropic")
        .as_object_mut()
        .expect("provider");
    assert!(provider.remove("keep").is_some());
    assert!(provider.remove("drop").is_some());
    assert_eq!(providers::flat(&key.providers), expected_providers, "only Anthropic's list changes");
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
