//! Recorded Codex evidence and adapter decisions over a laid filesystem.
//! Contracts: smith/docs/design/benchmarks/README.md, 13; smith/docs/design/benchmarks/agents.md, 1.3.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use skein_fake_machine::Item;
use smith_bench::agent::codex::{
    ChildThread, Link, Observing, PreparedCodex, Rollouts, collect_rollouts_with, parse_rollout, prepare_with,
};
use smith_bench::agent::{Context, PreparedAgent};
use smith_bench::arm::ArmRef;
use smith_bench::files::Files;
use smith_bench::guard::Guard;
use smith_bench::isolate::{Private, SetupStep, remove_owned_tree_with};
use smith_bench::testing::task::parse_task;
use smith_bench::{FaultKind, Measure, ModelChoice, PinnedConfiguration, ScopeKind, Task, read_configuration_with};
use smith_bench_fixtures::files::Tree;

fn recorded_time(bytes: &[u8]) -> smith_bench::Timing {
    let thread = "01a13785-a5a8-7543-ad07-90745b2ada69";
    let rollouts = Rollouts {
        responses: vec![],
        children: vec![],
        files: vec![smith_bench::agent::codex::RolloutFile { thread: thread.into(), bytes: bytes.into() }],
    };
    let mut timing = smith_bench::parse_result(Path::new("fixture"), include_str!("../recorded/results/attempt.json"))
        .expect("result fixture")
        .timing;
    rollouts.timing(thread, &mut timing).expect("recorded timestamps");
    timing
}

#[test]
fn rollout_threads_give_request_tool_and_command_overlap() {
    let timing = recorded_time(include_bytes!("../recorded/codex/repository-04-time.jsonl"));
    let record = &timing.record;
    let value = |measure: &Measure<u64>| match measure {
        Measure::Observed { value } => *value,
        other @ (Measure::LowerBound { .. } | Measure::Unavailable { .. }) => panic!("missing rollout span: {other:?}"),
    };
    assert!((388_000..389_000).contains(&value(&record.requests_ms)));
    let exec = record.tools_ms.iter().find(|(tool, _)| tool == "exec").expect("root exec calls");
    assert!((46_000..47_000).contains(&value(&exec.1)));
    assert!(value(&record.commands_outside_calls_ms) > 110_000);
    assert_eq!(record.compaction_ms, Measure::Observed { value: 0 });
    assert_eq!(record.children_ms, Measure::Observed { value: 0 });
    assert!(matches!(record.first_byte_ms, Measure::Unavailable { .. }));
    assert!(matches!(record.largest_gap_ms, Measure::Unavailable { .. }));
}

#[test]
fn a_stream_without_spans_leaves_the_record_unavailable() {
    for bytes in [b"".as_slice(), include_bytes!("../recorded/codex/completed.jsonl").as_slice()] {
        let timing = recorded_time(bytes);
        assert_eq!(timing.record.requests_ms, Measure::unavailable("root rollout lacks complete timestamped spans"));
        assert!(matches!(timing.record.rest_ms, Measure::Unavailable { .. }));
        assert!(matches!(timing.record.commands_outside_calls_ms, Measure::Unavailable { .. }));
    }
}

#[test]
fn a_summary_keeps_the_time_records_medians() {
    let mut results = Vec::new();
    for value in [100_u64, 300, 200] {
        let mut result =
            smith_bench::parse_result(Path::new("fixture"), include_str!("../recorded/results/attempt.json"))
                .expect("result fixture");
        result.identity.attempt = value.to_string();
        let record = &mut result.timing.record;
        record.requests_ms = Measure::Observed { value };
        record.first_byte_ms = Measure::Observed { value: value / 10 };
        record.largest_gap_ms = Measure::Observed { value: value / 2 };
        record.compaction_ms = Measure::Observed { value: 0 };
        record.children_ms = Measure::Observed { value: 0 };
        record.rest_ms = Measure::Observed { value: value / 5 };
        record.tools_ms = vec![("exec".into(), Measure::Observed { value: value / 4 })];
        results.push(result);
    }
    let summary = smith_bench::summarise(&results, "timing", "working", smith_bench::Design::Single, 17, None, None)
        .expect("summary");
    for (metric, expected) in [
        ("requests_ms", 200.0),
        ("first_byte_ms", 20.0),
        ("largest_gap_ms", 100.0),
        ("compaction_ms", 0.0),
        ("children_ms", 0.0),
        ("rest_ms", 40.0),
        ("tools/exec_ms", 50.0),
    ] {
        assert_eq!(
            summary.groups[0].metrics[&format!("time/{metric}")].median,
            Measure::Observed { value: expected },
            "{metric}"
        );
    }
    assert!(matches!(summary.groups[0].metrics["time/commands_outside_calls_ms"].median, Measure::Unavailable { .. }));
}

fn repository_threads() -> (String, Rollouts) {
    let root = "01a13785-a5a8-7543-ad07-90745b2ada69";
    let sources = [
        (root, include_bytes!("../recorded/codex/repository-04-root.jsonl").as_slice()),
        (
            "01a13785-fa70-7333-8992-f27e7720f4fb",
            include_bytes!("../recorded/codex/repository-04-child-one.jsonl").as_slice(),
        ),
        (
            "01a13786-12c9-7381-a0de-7de1cbc6eb1d",
            include_bytes!("../recorded/codex/repository-04-child-two.jsonl").as_slice(),
        ),
    ];
    let mut rollouts = Rollouts { responses: vec![], children: vec![], files: vec![] };
    for (thread, bytes) in sources {
        // Supply the model context omitted by this usage-only projection.
        let context = b"{\"type\":\"turn_context\",\"payload\":{\"model\":\"gpt-6.1-sol\"}}\n";
        let bytes = [context.as_slice(), bytes].concat();
        let (link, responses) = parse_rollout(&bytes, thread).expect("archived native rollout");
        if thread == root {
            rollouts.responses = responses;
        } else {
            rollouts.children.push(ChildThread { thread: thread.into(), link, responses });
        }
        rollouts.files.push(smith_bench::agent::codex::RolloutFile { thread: thread.into(), bytes });
    }
    (root.into(), rollouts)
}

#[test]
fn linked_codex_children_are_counted_from_their_rollouts() {
    let (root, mut rollouts) = repository_threads();
    let child = &rollouts.children[0].thread;
    rollouts.children.push(ChildThread {
        thread: "grandchild".into(),
        link: Link::Linked { parent: child.clone() },
        responses: vec![],
    });
    rollouts.children.push(ChildThread {
        thread: "unrelated".into(),
        link: Link::Linked { parent: "another-root".into() },
        responses: vec![],
    });
    rollouts.children.push(ChildThread { thread: "unlinked".into(), link: Link::Unlinked, responses: vec![] });
    let mut counts = smith_bench::agent::codex::Observer::default().counts();
    rollouts.counts(&root, &mut counts).expect("native counts");
    assert_eq!(counts.children, Measure::Observed { value: 3 });
    assert_eq!(counts.conversations, Measure::Observed { value: 4 });
}

#[test]
fn codex_responses_per_thread_come_from_token_counts() {
    let (root, rollouts) = repository_threads();
    let counts = rollouts.responses_per_thread(&root).expect("native token-count records");
    for (thread, responses) in [(&root, 55), (&rollouts.children[0].thread, 30), (&rollouts.children[1].thread, 27)] {
        assert_eq!(counts[thread], Measure::Observed { value: responses }, "{thread}");
    }
    let mut total = smith_bench::agent::codex::Observer::default().counts();
    rollouts.counts(&root, &mut total).expect("counts");
    assert_eq!(total.responses, Measure::Observed { value: 112 });
}

#[test]
fn the_codex_pin_names_only_the_keys_an_attempt_must_own() {
    let pin_file = crate::common::repository().join("agents/codex/standard.pin.toml");
    let pin = smith_bench::read_configuration(&pin_file).expect("shipped pin");
    assert_eq!(pin.pin.sha256.as_deref(), Some("12eb3e81114588aca3b7998f4f19e8997b056aca08e57a7ca7c8a3ec8c652aad"));
    assert_eq!(pin.pin.package.as_deref(), Some(Path::new("tools/codex-0.160.0")));
    let document: toml::Table =
        toml::from_str(include_str!("../../agents/codex/overrides.toml")).expect("shipped overrides");
    let keys: Vec<_> = document.keys().map(String::as_str).collect();
    assert_eq!(keys, ["approval_policy", "check_for_update_on_startup", "harness", "mcp_servers", "sandbox_mode"]);
}

#[test]
fn codex_responses_are_priced_from_their_usage_records() {
    let (root, rollouts) = repository_threads();
    let ledger = rollouts.token_ledger(&root).expect("response usage");
    assert_eq!(ledger.responses().len(), 112);
    let value: f64 = ledger
        .responses()
        .iter()
        .map(|response| {
            let prices = smith_local_shell::presets::list_prices("openai", &response.model, response.prompt_tokens())
                .expect("model price");
            let cost = u128::from(response.fresh - response.cache_write) * u128::from(prices.input)
                + u128::from(response.cache_read) * u128::from(prices.cached)
                + u128::from(response.cache_write) * u128::from(prices.cache_write.expect("write price"))
                + u128::from(response.output) * u128::from(prices.output);
            cost.to_string().parse::<f64>().expect("cost") / (f64::from(prices.unit) * 1_000_000.0)
        })
        .sum();
    assert!((value - 1.75468).abs() < 1e-12);
}

fn rollout(thread: &str, parent: Option<&str>) -> Vec<u8> {
    let metadata = serde_json::json!({"type":"session_meta", "payload":{"id":thread, "parent_thread_id":parent}});
    let usage = serde_json::json!({"type":"token_usage_record", "payload":{"thread_id":thread, "response_id":format!("response-{thread}"),
        "usage":{"input_tokens":20,"cached_input_tokens":5,"cache_write_input_tokens":0,"output_tokens":8,
        "reasoning_output_tokens":3,"total_tokens":28}}});
    format!("{metadata}\n{usage}\n{usage}\n").into_bytes()
}

fn immutable_response_ids_deduplicate_without_adding_turn_totals() {
    let (_, responses) = parse_rollout(&rollout("root", None), "root").expect("pinned shape");
    assert_eq!(responses.len(), 1);
    let (link, child_usage) = parse_rollout(&rollout("child", Some("root")), "child").expect("linked child");
    let records = Rollouts {
        responses,
        children: vec![ChildThread { thread: "child".into(), link, responses: child_usage }],
        files: Vec::new(),
    }
    .token_records("root")
    .expect("scoped ledger");
    let total = records.iter().find(|record| record.role == ScopeKind::Total).expect("total");
    assert_eq!(total.usage.fresh, Measure::Observed { value: 30 });
    assert_eq!(total.usage.output, Measure::Observed { value: 16 });
    assert_eq!(total.usage.reasoning, Measure::Observed { value: 6 });
}

fn an_unlinked_child_keeps_its_scope_unavailable_and_the_total_a_lower_bound() {
    for (name, parent) in [("missing", None), ("no-parent", Some(None)), ("unknown-parent", Some(Some("unknown")))] {
        let mut items = vec![
            Item::directory(b"home"),
            Item::directory(b"home/sessions"),
            Item::file(b"home/sessions/root.jsonl", &rollout("root", None)),
        ];
        if let Some(parent) = parent {
            items.push(Item::file(b"home/sessions/child.jsonl", &rollout("child", parent)));
        }
        let tree = Tree::new(&items);
        let mut observing = collection_observer(Path::new("/home"), "root", &["child"]);
        let collected = observing.collect_with(&tree).expect(name);
        assert_eq!(collected.children.len(), 1, "{name}");
        assert_eq!(collected.children[0].thread, "child", "{name}");
        assert_eq!(collected.children[0].link, Link::Unlinked, "{name}");
        let records = collected.token_records("root").expect("honest ledger");
        let child = records.iter().find(|record| record.scope == "child").expect("unavailable child scope");
        assert_eq!(child.role, ScopeKind::UnlinkedChild, "{name}");
        assert!(matches!(child.usage.fresh, Measure::Unavailable { .. }), "{name}");
        let total = records.iter().find(|record| record.role == ScopeKind::Total).expect("total");
        assert!(matches!(total.usage.fresh, Measure::LowerBound { value: 15, .. }), "{name}");
    }
}

fn known_usage_shape_changes_and_foreign_thread_records_refuse() {
    use crate::common::json::edit_lines;
    use serde_json::json;
    let text = String::from_utf8(rollout("root", None)).expect("JSON UTF-8");
    let missing = edit_lines(&text, "/payload/usage/cached_input_tokens", &json!(5), None, 2);
    assert!(parse_rollout(missing.as_bytes(), "root").is_err());
    assert!(parse_rollout(text.as_bytes(), "foreign").is_err());
    let negative = edit_lines(&text, "/payload/usage/output_tokens", &json!(8), Some(&json!(-8)), 2);
    assert!(parse_rollout(negative.as_bytes(), "root").is_err());
    let response = text.lines().nth(1).expect("usage");
    let response = edit_lines(response, "/payload/usage/output_tokens", &json!(8), Some(&json!(9)), 1);
    let response = edit_lines(&response, "/payload/usage/total_tokens", &json!(28), Some(&json!(29)), 1);
    let conflicting = format!("{text}{response}");
    assert!(parse_rollout(conflicting.as_bytes(), "root").is_err());
}

#[derive(Debug)]
struct TreeRow {
    name: &'static str,
    files: Vec<(&'static str, Vec<u8>)>,
    links: Vec<(&'static str, &'static str)>,
    root: &'static str,
    stream_children: Vec<&'static str>,
    expected: Result<TreeEvidence, (&'static str, &'static str)>,
}

#[derive(Debug, PartialEq, Eq)]
struct TreeEvidence {
    children: Vec<(String, Link)>,
    files: Vec<String>,
}

fn rollout_tree(row: &TreeRow) -> Tree {
    let mut directories = BTreeSet::from([PathBuf::from("home")]);
    for name in row.files.iter().map(|(path, _)| path).chain(row.links.iter().map(|(path, _)| path)) {
        let path = Path::new("home").join(name);
        directories.extend(path.ancestors().skip(1).filter(|path| !path.as_os_str().is_empty()).map(Path::to_path_buf));
    }
    let mut items: Vec<_> =
        directories.iter().map(|path| Item::directory(path.as_os_str().as_encoded_bytes())).collect();
    for (path, bytes) in &row.files {
        items.push(Item::file(format!("home/{path}").as_bytes(), bytes));
    }
    for (path, target) in &row.links {
        items.push(Item::link(format!("home/{path}").as_bytes(), format!("/home/{target}").as_bytes()));
    }
    Tree::new(&items)
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table of rollout discovery rules shares its fixture and assertions")]
fn rollout_trees_discover_only_named_threads_and_refuse_unsafe_or_ambiguous_files() {
    let linked = |thread: &str, parent: &str| (thread.to_owned(), Link::Linked { parent: parent.into() });
    let rows = [
        TreeRow {
            name: "archived",
            files: vec![("sessions/a/root.jsonl", rollout("root", None)),
                ("archived_sessions/2026/10/09/rollout-x.jsonl", rollout("child", Some("root")))],
            links: vec![], root: "root", stream_children: vec![],
            expected: Ok(TreeEvidence { children: vec![linked("child", "root")], files: vec!["child".into(), "root".into()] }),
        },
        TreeRow {
            name: "unrelated-openings",
            files: vec![("sessions/root", rollout("root", None)),
                ("sessions/other-kind", b"{\"type\":\"turn_context\",\"payload\":{\"id\":\"foreign\"}}\n{invalid}".to_vec()),
                ("sessions/no-id", b"{\"type\":\"session_meta\",\"payload\":{}}\n{invalid}".to_vec()),
                ("sessions/binary", vec![0xff, 0xfe, b'\n']), ("sessions/empty", vec![])],
            links: vec![], root: "root", stream_children: vec![],
            expected: Ok(TreeEvidence { children: vec![], files: vec!["root".into()] }),
        },
        TreeRow {
            name: "outside-file", files: vec![("outside/file", rollout("root", None))],
            links: vec![("sessions/root", "outside/file")], root: "root", stream_children: vec![],
            expected: Err(("sessions/root", "link")),
        },
        TreeRow {
            name: "outside-directory", files: vec![("outside/file", rollout("root", None))],
            links: vec![("sessions/escape", "outside")], root: "root", stream_children: vec![],
            expected: Err(("sessions/escape", "link")),
        },
        TreeRow {
            name: "dangling", files: vec![], links: vec![("sessions/missing", "absent")],
            root: "root", stream_children: vec![], expected: Err(("sessions/missing", "link")),
        },
        TreeRow {
            name: "session-directory-link", files: vec![("outside/file", rollout("root", None))],
            links: vec![("sessions", "outside")], root: "root", stream_children: vec![],
            expected: Err(("sessions", "link")),
        },
        TreeRow {
            name: "internal-links", files: vec![("sessions/a/root", rollout("root", None)),
                ("archived_sessions/b/child", rollout("child", Some("root")))],
            links: vec![("sessions/alias", "sessions/a/root"), ("archived_sessions/alias", "sessions/a")],
            root: "root", stream_children: vec![],
            expected: Ok(TreeEvidence { children: vec![linked("child", "root")], files: vec!["child".into(), "root".into()] }),
        },
        TreeRow {
            name: "duplicate", files: vec![("sessions/x.jsonl", rollout("dup", Some("root"))),
                ("sessions/y.jsonl", rollout("dup", Some("root")))], links: vec![], root: "root", stream_children: vec![],
            expected: Err(("sessions/y.jsonl", "duplicate")),
        },
        TreeRow {
            name: "grandchild", files: vec![("sessions/root", rollout("root", None)),
                ("sessions/child", rollout("z-child", Some("root"))),
                ("sessions/grandchild", rollout("a-grandchild", Some("z-child")))],
            links: vec![], root: "root", stream_children: vec![],
            expected: Ok(TreeEvidence { children: vec![linked("a-grandchild", "z-child"), linked("z-child", "root")],
                files: vec!["a-grandchild".into(), "root".into(), "z-child".into()] }),
        },
        TreeRow {
            name: "native-parent", files: vec![("sessions/root", rollout("root", None)),
                ("sessions/child", b"{\"type\":\"session_meta\",\"payload\":{\"id\":\"child\",\"parent_thread_id\":\"unknown\",\"source\":{\"subagent\":{\"thread_spawn\":{\"parent_thread_id\":\"root\"}}}}}\n".to_vec())],
            links: vec![], root: "root", stream_children: vec![],
            expected: Ok(TreeEvidence { children: vec![linked("child", "root")], files: vec!["child".into(), "root".into()] }),
        },
        TreeRow {
            name: "unknown-parent", files: vec![("sessions/root", rollout("root", None)),
                ("sessions/foreign", rollout("foreign", Some("unknown")))],
            links: vec![], root: "root", stream_children: vec![],
            expected: Ok(TreeEvidence { children: vec![], files: vec!["root".into()] }),
        },
        TreeRow {
            name: "stream-child", files: vec![("sessions/root", rollout("root", None)),
                ("sessions/child", rollout("child", None)), ("sessions/grandchild", rollout("grandchild", Some("child")))],
            links: vec![], root: "root", stream_children: vec!["child"],
            expected: Ok(TreeEvidence { children: vec![("child".into(), Link::Unlinked), linked("grandchild", "child")],
                files: vec!["child".into(), "grandchild".into(), "root".into()] }),
        },
        TreeRow {
            name: "missing-directories", files: vec![], links: vec![], root: "root", stream_children: vec!["child"],
            expected: Ok(TreeEvidence { children: vec![("child".into(), Link::Unlinked)], files: vec![] }),
        },
    ];
    for row in rows {
        let tree = rollout_tree(&row);
        let mut observing = collection_observer(Path::new("/home"), row.root, &row.stream_children);
        match row.expected {
            Ok(expected) => {
                let collected = observing.collect_with(&tree).unwrap_or_else(|error| panic!("{}: {error}", row.name));
                let evidence = TreeEvidence {
                    children: collected
                        .children
                        .iter()
                        .map(|child| {
                            let link = match &child.link {
                                Link::Linked { parent } => Link::Linked { parent: parent.clone() },
                                Link::Unlinked => Link::Unlinked,
                            };
                            (child.thread.clone(), link)
                        })
                        .collect(),
                    files: collected.files.iter().map(|file| file.thread.clone()).collect(),
                };
                assert_eq!(evidence, expected, "{}", row.name);
            }
            Err((path, word)) => {
                let error = observing.collect_with(&tree).expect_err(row.name).detail;
                assert!(error.contains(path) && error.contains(word), "{}: {error}", row.name);
            }
        }
    }
}

#[test]
fn codex_refuses_later_person_lines_before_reading_a_login() {
    let mut fixture = CodexAttempt::new(&[]);
    fixture.task.person =
        vec![smith_bench::agent::Cue { when: smith_bench::agent::Trigger::Answer, text: "later\n".into() }];
    fixture.tree.never_read(fixture.home.join("auth.json"));
    let refusal = prepare_with(&fixture.tree, &fixture.context()).expect_err("interactive task refused before login");
    assert_eq!(refusal.kind, FaultKind::Setup(SetupStep::Configuration));
    assert!(refusal.detail.contains("later person lines"));
    assert_eq!(fixture.tree.writes(), []);
    fixture.assert_no_login_touched();
}

fn read_only_collection_follows_only_the_named_roots_descendants() {
    let mut items = vec![Item::directory(b"home"), Item::directory(b"home/sessions")];
    let mut before = Vec::new();
    for (thread, parent) in [("root", None), ("child", Some("root")), ("foreign", None)] {
        let path = format!("home/sessions/{thread}.jsonl");
        let bytes = if thread == "foreign" { b"not JSON".to_vec() } else { rollout(thread, parent) };
        items.push(Item::file(path.as_bytes(), &bytes));
        before.push((format!("/{path}"), bytes));
    }
    let tree = Tree::new(&items);
    let collected =
        collect_rollouts_with(&tree, Path::new("/home"), "root").expect("named collection ignores foreign file");
    assert_eq!(collected.children.len(), 1);
    assert_eq!(collected.files.len(), 2);
    assert_eq!(collected.children[0].link, Link::Linked { parent: "root".into() });
    for (file, bytes) in before {
        assert_eq!(tree.read(Path::new(&file)).expect("unchanged rollout tree"), bytes);
    }
    assert_eq!(tree.writes(), []);
}

fn a_named_path_cannot_read_credentials_or_an_unrelated_file() {
    let tree = Tree::new(&[
        Item::directory(b"home"),
        Item::directory(b"home/sessions"),
        Item::file(b"home/auth.json", b"CREDENTIAL_CONTROL_SENTINEL"),
        Item::link(b"home/sessions/root.jsonl", b"/home/auth.json"),
    ]);
    let error = collect_rollouts_with(&tree, Path::new("/home"), "root").expect_err("not a rollout directory");
    assert!(!error.to_string().contains("CREDENTIAL_CONTROL_SENTINEL"));
}

fn the_pinned_adapter_refuses_version_drift_and_isolates_the_user_home() {
    let configuration = b"model=\"user-choice\"\n";
    let fixture = CodexAttempt::new(&[
        Item::file(b"fixture/logins/codex/config.toml", configuration),
        Item::file(b"fixture/logins/codex/AGENTS.md", b"UNRELATED_GLOBAL_INSTRUCTION"),
    ]);
    let tree = &fixture.tree;
    let mut context = Context { installed_version: "codex-cli 0.159.0", ..fixture.context() };
    assert_eq!(
        prepare_with(tree, &context).expect_err("version drift").kind,
        FaultKind::Setup(SetupStep::Configuration)
    );
    context.installed_version = "codex-cli 0.160.0";
    let codex = prepare_with(tree, &context).expect("pinned version");
    assert_eq!(codex.home, fixture.private.root.join("codex"));
    assert_eq!(tree.read_dir(&codex.home).expect("private home").count(), 1);
    let proof = fixture.finish(&codex);
    assert!(proof.source_unchanged);
    assert!(!tree.try_exists(&codex.credential).expect("credential lookup"));
    assert_eq!(tree.read(fixture.home.join("auth.json").as_path()).expect("source unchanged"), synthetic_auth(500));
    let (_, projects) = codex.overrides.iter().find(|(key, _)| key == "projects").expect("whole trust table");
    let parsed: toml::Value = toml::from_str(&format!("projects={projects}")).expect("dynamic path TOML");
    assert_eq!(
        parsed["projects"][fixture.private.workspace.to_str().expect("path")]["trust_level"].as_str(),
        Some("trusted")
    );
    let invocation = PreparedAgent::Codex(codex).command();
    let arguments: Vec<_> = invocation.arguments.iter().map(|argument| argument.to_str().expect("UTF-8")).collect();
    assert!(!arguments.contains(&"--ignore-user-config"));
    assert!(!arguments.contains(&"--ignore-rules"));
    assert_eq!(tree.read(fixture.home.join("config.toml").as_path()).expect("unmodified config"), configuration);
}

fn synthetic_auth(expires: u64) -> Vec<u8> {
    use base64::Engine;
    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!("{{\"exp\":{expires}}}"));
    serde_json::to_vec(&serde_json::json!({"auth_mode":"chatgpt", "last_refresh":"2000-01-01T00:00:00Z",
        "tokens":{"id_token":"UNISSUED_ID", "access_token":format!("header.{claims}.signature"),
        "account_id":"UNISSUED_ACCOUNT", "refresh_token":"UNISSUED_REFRESH_MUST_NOT_COPY"}}))
    .expect("fixture")
}

fn credential_tree(expires: u64, extras: &[Item]) -> Tree {
    let mut items = vec![
        Item::directory(b"fixture").mode(0o700),
        Item::directory(b"fixture/logins"),
        Item::directory(b"fixture/logins/codex"),
        Item::file(b"fixture/logins/codex/auth.json", &synthetic_auth(expires)).mode(0o600),
    ];
    items.extend_from_slice(extras);
    Tree::new(&items)
}

const SOURCE_HOME: &str = "/fixture/logins/codex";
const SOURCE_AUTH: &str = "/fixture/logins/codex/auth.json";
const BORROWED_HOME: &str = "/fixture/attempt-codex";
const WALL: skein_lib::Wall = skein_lib::Wall::from_nanos(100_000_000_000);
const LIFETIME: skein_lib::Duration = skein_lib::Duration::from_secs(20);

fn borrow_login(tree: &Tree, guard: &Guard) -> Result<smith_bench::credential::Borrowed, smith_bench::Fault> {
    smith_bench::credential::codex_with(tree, guard, Path::new(SOURCE_HOME), Path::new(BORROWED_HOME), WALL, LIFETIME)
}

fn access_only_credentials_are_private_and_never_copy_a_refresh_or_other_home_file() {
    let markers: Vec<_> = ["AGENTS.md", "config.toml", "skills", "memories", "plugins"]
        .into_iter()
        .map(|name| Item::file(format!("fixture/logins/codex/{name}").as_bytes(), b"UNRELATED_HOME_MARKER"))
        .collect();
    let tree = credential_tree(500, &markers);
    let guard = fixture_guard(&tree);
    let home = Path::new(BORROWED_HOME);
    let borrowed = borrow_login(&tree, &guard).expect("access-only conversion");
    assert_eq!(tree.metadata(home).expect("home mode").mode, 0o700);
    assert_eq!(tree.metadata(&borrowed.credential).expect("auth mode").mode, 0o600);
    assert_eq!(tree.read_dir(home).expect("initial home").count(), 1);
    let record: serde_json::Value =
        serde_json::from_slice(&tree.read(&borrowed.credential).expect("private auth")).expect("auth shape");
    assert_eq!(record["auth_mode"], "chatgpt");
    assert_eq!(record["tokens"]["refresh_token"], "");
    assert_eq!(record["tokens"]["id_token"], "UNISSUED_ID");
    assert_eq!(record["tokens"]["account_id"], "UNISSUED_ACCOUNT");
    assert_eq!(record["last_refresh"], "1970-01-01T00:01:40Z");
    assert!(!serde_json::to_string(&record).expect("record").contains("UNISSUED_REFRESH"));
    assert!(!format!("{borrowed:?}").contains("UNISSUED"));
    let proof = borrowed.finish_with(&tree, &guard).expect("after tree settlement");
    assert!(proof.source_unchanged && proof.credential_removed && proof.private_refresh_empty);
    assert_eq!(tree.read(Path::new(SOURCE_AUTH)).expect("source unchanged"), synthetic_auth(500));
}

fn credentials_expiring_before_the_hard_deadline_refuse_before_creating_a_home() {
    let tree = credential_tree(110, &[]);
    let refusal = borrow_login(&tree, &fixture_guard(&tree)).expect_err("lapsed token");
    assert_eq!(refusal.kind, FaultKind::Setup(SetupStep::Credentials));
    assert!(refusal.detail.contains(&format!("run `CODEX_HOME={SOURCE_HOME} codex` once")));
    assert!(!refusal.detail.contains("UNISSUED"));
    assert_eq!(tree.writes(), []);
    assert!(!tree.try_exists(Path::new(BORROWED_HOME)).expect("home lookup"));
}

fn changed_user_auth_is_reported_after_private_credentials_are_removed() {
    let tree = credential_tree(500, &[]);
    let guard = fixture_guard(&tree);
    let borrowed = borrow_login(&tree, &guard).expect("conversion");
    tree.write(Path::new(SOURCE_AUTH), &synthetic_auth(501)).expect("user refresh control");
    assert!(
        borrowed
            .finish_with(&tree, &guard)
            .expect_err("source changed")
            .detail
            .contains("stop before further inference")
    );
    assert!(!tree.try_exists(&borrowed.credential).expect("credential lookup"));
}

fn repeated_session_metadata_keeps_its_parent_and_refuses_conflicts() {
    let stream = rollout("root", Some("parent"));
    let text = String::from_utf8(stream).expect("UTF-8");
    let metadata = text.lines().next().expect("metadata");
    let repeated = format!("{text}{metadata}\n");
    assert_eq!(
        parse_rollout(repeated.as_bytes(), "root").expect("same identity"),
        parse_rollout(text.as_bytes(), "root").expect("original")
    );
    let conflicting = format!(
        "{text}{}\n",
        crate::common::json::edit_lines(
            metadata,
            "/payload/parent_thread_id",
            &serde_json::json!("parent"),
            Some(&serde_json::json!("foreign")),
            1
        )
    );
    assert!(
        parse_rollout(conflicting.as_bytes(), "root")
            .expect_err("parent changed")
            .to_string()
            .contains("conflicting repeated metadata")
    );
}

fn collection_observer(home: &Path, root: &str, children: &[&str]) -> Observing {
    let prepared = PreparedAgent::Codex(PreparedCodex {
        overrides: Vec::new(),
        credential: home.join("auth.json"),
        expires: skein_lib::Wall::EPOCH,
        borrowed: None,
        home: home.to_path_buf(),
        final_message: home.join("final.txt"),
        model: smith_bench::ModelChoice { model: Some("offline".into()), effort: Some("low".into()) },
        program: "/bin/false".into(),
        workspace: home.to_path_buf(),
        teardown: skein_lib::Duration::ZERO,
    });
    let smith_bench::agent::Observing::Codex(mut observing) = prepared.observe().expect("prepared Codex observer")
    else {
        panic!("Codex observer");
    };
    observing
        .observer
        .observe_line(&serde_json::json!({"type":"thread.started", "thread_id": root}).to_string())
        .expect("named root");
    if !children.is_empty() {
        observing.observer.observe_line(r#"{"type":"turn.started"}"#).expect("active turn");
        observing
            .observer
            .observe_line(
                &serde_json::json!({
                    "type":"item.completed", "item": {"id":"spawn", "type":"collab_tool_call",
                    "tool":"spawn_agent", "sender_thread_id":root, "receiver_thread_ids":children,
                    "prompt":null, "agents_states":{}, "status":"completed"}
                })
                .to_string(),
            )
            .expect("completed spawn_agent");
    }
    observing
}

fn malformed_named_rollouts_remain_available_for_archive_and_keep_the_cached_fault() {
    let raw = b"{\"type\":\"session_meta\",\"payload\":{\"id\":\"root\"}}\n{invalid JSON}\n";
    let tree = Tree::new(&[
        Item::directory(b"home"),
        Item::directory(b"home/sessions"),
        Item::file(b"home/sessions/root.jsonl", raw),
    ]);
    let mut observing = collection_observer(Path::new("/home"), "root", &[]);
    let first = observing.collect_with(&tree).expect_err("known shape fault").detail;
    assert!(first.contains("thread root"));
    let retained = observing.collected.as_ref().expect("raw retained");
    assert_eq!(retained.files.len(), 1);
    assert_eq!(retained.files[0].bytes, raw);
    tree.never_read("/home/sessions/root.jsonl");
    assert_eq!(observing.collect_with(&tree).expect_err("cached fault").detail, first);
    assert_eq!(tree.unexpected_reads(), []);
}

fn a_childs_inherited_parent_header_never_replaces_the_childs_ownership() {
    let child = String::from_utf8(rollout("child", Some("root"))).expect("child");
    let inherited = r#"{"type":"session_meta","payload":{"id":"root","parent_thread_id":null}}"#;
    let stream = format!("{child}{inherited}\n");
    let (link, usage) = parse_rollout(stream.as_bytes(), "child").expect("native inherited context");
    assert_eq!(link, Link::Linked { parent: "root".into() });
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].response, "response-child");
    assert!(
        parse_rollout(
            crate::common::json::edit_lines(
                &stream,
                "/payload/id",
                &serde_json::json!("root"),
                Some(&serde_json::json!("foreign")),
                1
            )
            .as_bytes(),
            "child"
        )
        .is_err()
    );
}

fn captured_native_child_headers_and_usage_keep_each_response_in_its_own_scope() {
    let root = "01a121d2-5c3c-74b1-865b-66d8c6f41c3a";
    let child = "01a121d2-6e5a-7e30-abd0-133d08c8c960";
    let (_, responses) =
        parse_rollout(include_bytes!("../recorded/codex/live-root-rollout.jsonl"), root).expect("root");
    let (link, child_responses) =
        parse_rollout(include_bytes!("../recorded/codex/live-child-rollout.jsonl"), child).expect("inherited context");
    assert_eq!(link, Link::Linked { parent: root.into() });
    assert_eq!(responses.len(), 4);
    assert_eq!(child_responses.len(), 2);
    let collected = Rollouts {
        responses,
        children: vec![ChildThread { thread: child.into(), link, responses: child_responses }],
        files: vec![
            smith_bench::agent::codex::RolloutFile {
                thread: root.into(),
                bytes: include_bytes!("../recorded/codex/live-root-rollout.jsonl").to_vec(),
            },
            smith_bench::agent::codex::RolloutFile {
                thread: child.into(),
                bytes: include_bytes!("../recorded/codex/live-child-rollout.jsonl").to_vec(),
            },
        ],
    };
    let tokens = collected.token_records(root).expect("scope totals");
    assert!(matches!(
        tokens.iter().find(|r| r.role == ScopeKind::Total).expect("total").usage.output,
        Measure::Observed { value: 186 }
    ));
    let mut counts = smith_bench::agent::codex::Observer::default().counts();
    collected.counts(root, &mut counts).expect("native counts");
    assert_eq!(counts.responses, Measure::Observed { value: 6 });
}

#[test]
fn pinned_adapters_usage_collection_and_private_credentials() {
    immutable_response_ids_deduplicate_without_adding_turn_totals();
    an_unlinked_child_keeps_its_scope_unavailable_and_the_total_a_lower_bound();
    known_usage_shape_changes_and_foreign_thread_records_refuse();
    read_only_collection_follows_only_the_named_roots_descendants();
    a_named_path_cannot_read_credentials_or_an_unrelated_file();
    the_pinned_adapter_refuses_version_drift_and_isolates_the_user_home();
    access_only_credentials_are_private_and_never_copy_a_refresh_or_other_home_file();
    credentials_expiring_before_the_hard_deadline_refuse_before_creating_a_home();
    changed_user_auth_is_reported_after_private_credentials_are_removed();
    repeated_session_metadata_keeps_its_parent_and_refuses_conflicts();
    malformed_named_rollouts_remain_available_for_archive_and_keep_the_cached_fault();
    a_childs_inherited_parent_header_never_replaces_the_childs_ownership();
    captured_native_child_headers_and_usage_keep_each_response_in_its_own_scope();
}

#[test]
fn paths_use_the_declared_working_directory_and_refuse_escaped_paths_before_credentials() {
    let fixture = CodexAttempt::new(&[
        Item::directory(b"fixture/scratch.run/workspace/app"),
        Item::file(
            b"fixture/scratch.run/workspace/app/input.txt",
            include_bytes!("../../tasks/probes/paths/seed/app/input.txt"),
        ),
        Item::directory(b"fixture/scratch.run/workspace/spec").mode(0o500),
        Item::file(
            b"fixture/scratch.run/workspace/spec/rules.txt",
            include_bytes!("../../tasks/probes/paths/seed/spec/rules.txt"),
        )
        .mode(0o400),
        Item::directory(b"fixture/outside"),
        Item::link(b"fixture/scratch.run/workspace/escaped", b"/fixture/outside"),
    ]);
    let private = &fixture.private;
    let tree = &fixture.tree;
    let path = Path::new("tasks/probes/paths/task.toml");
    let task = parse_task(path, include_str!("../../tasks/probes/paths/task.toml")).expect("frozen paths task");
    let context = fixture.context();
    for path in [private.workspace.join("spec"), private.workspace.join("spec/rules.txt")] {
        assert_eq!(tree.metadata(&path).expect("read-only layout").mode & 0o222, 0);
    }
    for task in [&task, &task.with_variant("single").expect("single task")] {
        let context = Context { task, ..context };
        let prepared = prepare_with(tree, &context).expect("Codex preparation, no invocation");
        assert_eq!(prepared.workspace, private.workspace.join("app"));
        let invocation = prepared.command();
        let index = invocation.arguments.iter().position(|argument| argument == "-C").expect("working directory flag");
        assert_eq!(invocation.arguments[index + 1], private.workspace.join("app").as_os_str());
        let (_, projects) = prepared.overrides.iter().find(|(name, _)| name == "projects").expect("trust table");
        let parsed: toml::Value = toml::from_str(&format!("projects={projects}")).expect("TOML trust table");
        assert_eq!(
            parsed["projects"][private.workspace.join("app").to_str().expect("path")]["trust_level"].as_str(),
            Some("trusted")
        );
        assert!(fixture.finish(&prepared).source_unchanged);
        tree.remove_dir_all(&private.root.join("codex")).expect("reset unused adapter");
    }
    let auth = fixture.home.join("auth.json");
    tree.never_read(&auth);
    let before_refusals = tree.writes();
    for path in ["../outside", "/tmp", "absent", "app/input.txt", "escaped"] {
        let mut refused_task = task.clone();
        refused_task.working_directory = Some(path.into());
        let context = Context { task: &refused_task, ..context };
        assert_eq!(
            prepare_with(tree, &context).expect_err("working directory refused before login").kind,
            FaultKind::Setup(SetupStep::Configuration),
            "{path}"
        );
        assert!(!tree.try_exists(&private.root.join("codex/auth.json")).expect("credential lookup"), "{path}");
    }
    assert_eq!(tree.unexpected_reads(), []);
    assert_eq!(tree.writes(), before_refusals);
    assert_eq!(tree.read(&auth).expect("unissued source unchanged"), synthetic_auth(500));
    remove_owned_tree_with(tree, &fixture.guard, &private.root).expect("read-only owned layout cleaned");
    assert!(!tree.try_exists(&private.root).expect("scratch lookup"));
    assert!(tree.try_exists(Path::new("/fixture/outside")).expect("outside lookup"));
}

fn fixture_guard(tree: &Tree) -> Guard {
    Guard::new_with(
        tree,
        Path::new("/fixture/user"),
        Path::new("/fixture/config"),
        Path::new("/fixture/state"),
        &[SOURCE_HOME.into()],
    )
    .expect("fixture guard")
}

struct CodexAttempt {
    tree: Tree,
    guard: Guard,
    private: Private,
    pin_file: PathBuf,
    pin: PinnedConfiguration,
    task: Task,
    model: ModelChoice,
    arm: ArmRef,
    home: PathBuf,
}

impl CodexAttempt {
    fn new(extra: &[Item]) -> Self {
        let root = PathBuf::from("/fixture/scratch.run");
        let private = Private {
            workspace: root.join("workspace"),
            home: root.join("home"),
            temporary: root.join("tmp"),
            cargo_home: root.join("cargo"),
            target: root.join("target"),
            warm_cargo: root.join("warm-cargo"),
            warm_target: root.join("warm-target"),
            config: root.join("config"),
            state: root.join("state"),
            root,
        };
        let metadata = include_str!("../../agents/codex/standard.pin.toml").replace(
            "12eb3e81114588aca3b7998f4f19e8997b056aca08e57a7ca7c8a3ec8c652aad",
            "0d63d62cbfd277a91299c0335296a03bcb6a02c2642a90e2b5815caf71726a8d",
        );
        let mut items = vec![
            Item::directory(b"fixture/pin"),
            Item::file(b"fixture/pin/standard.pin.toml", metadata.as_bytes()),
            Item::file(b"fixture/pin/overrides.toml", include_bytes!("../../agents/codex/overrides.toml")),
            Item::directory(b"fixture/tools"),
            Item::file(b"fixture/tools/codex", b"unexecuted Codex fixture\n").mode(0o700),
            Item::directory(b"fixture/scratch.run").mode(0o700),
            Item::directory(b"fixture/scratch.run/workspace").mode(0o700),
        ];
        // The other private paths are inputs to rendering, not directories these checks use.
        items.extend_from_slice(extra);
        let tree = credential_tree(500, &items);
        let guard = fixture_guard(&tree);
        let pin_file = PathBuf::from("/fixture/pin/standard.pin.toml");
        let pin = read_configuration_with(&tree, &pin_file).expect("fixture pin");
        let arm = ArmRef::Installed { program: "/fixture/tools/codex".into() };
        let task = parse_task(Path::new("recorded/valid/task.toml"), include_str!("../recorded/tasks/valid/task.toml"))
            .expect("fixture task");
        let model = ModelChoice { model: Some("gpt-6-luna".into()), effort: Some("low".into()) };
        Self { tree, guard, private, pin_file, pin, task, model, arm, home: SOURCE_HOME.into() }
    }

    fn context(&self) -> Context<'_> {
        Context {
            guard: &self.guard,
            private: &self.private,
            pin: &self.pin,
            pin_file: &self.pin_file,
            task: &self.task,
            variant: None,
            model: &self.model,
            arm: &self.arm,
            installed_version: "codex-cli 0.160.0",
            home: &self.home,
            wall: WALL,
            lifetime: LIFETIME,
            environment: &[],
        }
    }

    fn finish(&self, prepared: &PreparedCodex) -> smith_bench::credential::Proof {
        prepared
            .borrowed
            .as_ref()
            .expect("private credential")
            .finish_with(&self.tree, &self.guard)
            .expect("settled cleanup")
    }

    fn assert_no_login_touched(&self) {
        assert_eq!(self.tree.unexpected_reads(), []);
        assert!(!self.tree.try_exists(&self.private.root.join("codex")).expect("private home lookup"));
    }
}

#[test]
fn a_codex_binary_whose_digest_differs_from_its_pin_is_refused() {
    let fixture = CodexAttempt::new(&[]);
    let ArmRef::Installed { program } = &fixture.arm else {
        panic!("fixture package");
    };
    fixture.tree.write(program, b"different executable\n").expect("changed executable");
    let before_refusal = fixture.tree.writes();
    fixture.tree.never_read(fixture.home.join("auth.json"));
    let refusal = prepare_with(&fixture.tree, &fixture.context()).expect_err("digest drift before credentials");
    assert_eq!(refusal.kind, smith_bench::FaultKind::Setup(smith_bench::isolate::SetupStep::Configuration));
    assert!(refusal.detail.contains("differs from its pin"));
    assert_eq!(fixture.tree.writes(), before_refusal);
    fixture.assert_no_login_touched();
}

#[test]
fn codex_writable_roots_hold_the_attempts_caches() {
    let fixture = CodexAttempt::new(&[]);
    let prepared = prepare_with(&fixture.tree, &fixture.context()).expect("private preparation");
    let (_, roots) = prepared
        .overrides
        .iter()
        .find(|(key, _)| key == "sandbox_workspace_write.writable_roots")
        .expect("cache roots");
    let document: toml::Table = toml::from_str(&format!("roots={roots}")).expect("TOML roots");
    let paths: Vec<_> = document["roots"]
        .as_array()
        .expect("root array")
        .iter()
        .map(|value| PathBuf::from(value.as_str().expect("root path")))
        .collect();
    assert_eq!(
        paths,
        [fixture.private.cargo_home.clone(), fixture.private.target.clone(), fixture.private.temporary.clone()]
    );
    fixture.finish(&prepared);
}

#[test]
fn codex_trusts_the_attempt_root() {
    let fixture = CodexAttempt::new(&[]);
    let prepared = prepare_with(&fixture.tree, &fixture.context()).expect("private preparation");
    let (_, projects) = prepared.overrides.iter().find(|(key, _)| key == "projects").expect("trust table");
    let document: toml::Table = toml::from_str(&format!("projects={projects}")).expect("TOML projects");
    assert_eq!(
        document["projects"][fixture.private.workspace.to_str().expect("attempt root")]["trust_level"].as_str(),
        Some("trusted")
    );
    fixture.finish(&prepared);
}
