//! Replaying controls (smith/docs/design/testing.md, 2).

use smith_bench::Measure;
use smith_bench::agent::codex::{ChildThread, Link, Rollouts, parse_rollout};
use std::path::Path;

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
