//! Parity's offline verdict uses attempt rows and committed source identities.
mod common;

use common::Rows;

#[test]
fn a_summary_with_a_retried_failure_row_verifies_against_its_own_section() {
    const SMITH_FIXTURE_WALL_MS: u64 = 1_739_248;
    const CODEX_FIXTURE_WALL_MS: u64 = 1_748_166;
    let mut summary = common::parity_summary(Rows {
        smith_wall_ms: SMITH_FIXTURE_WALL_MS,
        codex_wall_ms: CODEX_FIXTURE_WALL_MS,
        ..Rows::equal()
    });
    let limits = {
        let mut fixture_limits = smith_events_fixtures::SMALL;
        fixture_limits.string = 524_288;
        fixture_limits.content = 4096;
        fixture_limits.items = 512;
        fixture_limits
    };
    let mut observer =
        smith_bench::agent::smith::events::Observer::with_ceiling(&limits, 16_384).expect("response bounds");
    observer.feed(include_bytes!("../recorded/retried-failure-17d63afd-02.jsonl")).expect("response excerpt");
    observer.finish().expect("whole lines");
    let tokens = observer.tokens().expect("recorded tokens");
    let total = tokens.iter().find(|record| record.role == smith_bench::ScopeKind::Total).expect("total");
    let row = summary.attempts.iter_mut().find(|row| row.agent == "smith").expect("smith row");
    row.fresh_tokens = total.usage.fresh.clone();
    row.cache_read_tokens = total.usage.cache_read.clone();
    row.output_tokens = total.usage.output.clone();
    row.priced_usd = Measure::LowerBound {
        value: 1.0,
        missing: vec!["session-0/run-1/conversation-0: session-0/run-1/response-3: retried failure".into()],
    };
    summary.parity = Some(recompute(&summary));
    let scratch = skein_scratch::Scratch::new("bench-parity");
    let file = scratch.path().to_path_buf().join(format!("smith-bench-parity-roundtrip-{}.json", std::process::id()));
    smith_bench::write_summary(&file, &summary).expect("write summary");
    let read_back = smith_bench::read_summary(&file).expect("read summary");
    std::fs::remove_file(file).expect("remove summary");
    assert_eq!(parity::verify(&read_back, &"0".repeat(40)).expect("section agrees").verdict, Verdict::OnPar);
}

#[test]
fn the_change_command_refuses_a_section_that_disagrees() {
    let (mut summary, scratch) = common::named_change("disagrees");
    let directory = scratch.path().join(format!("smith-change-{}-{}", "disagrees", std::process::id()));
    summary.change.as_mut().expect("section").ratios[0].value.statistic = 2.0;
    let file = directory.join("change.json");
    smith_bench::write_summary(&file, &summary).expect("changed section");
    let code = parity::change::command(&file);
    assert_eq!(code, 3);
    assert_eq!(
        parity::change::verify(&summary),
        Err(smith_bench::Fault::new(
            smith_bench::FaultKind::Mismatch,
            "committed change section disagrees with rows or inputs"
        ))
    );
}

#[test]
fn the_change_command_agrees_with_a_stopped_section() {
    const CHANGE_WALL_MS: u64 = 200_000;
    let (mut summary, scratch) = common::named_change("stopped");
    let directory = scratch.path().join(format!("smith-change-{}-{}", "stopped", std::process::id()));
    for row in &mut summary.attempts {
        if row.agent_version == common::CHANGE_COMMIT {
            row.task_wall_ms = Measure::Observed { value: CHANGE_WALL_MS };
        }
    }
    let baseline = directory.join("baseline.json");
    parity::change::attach(&mut summary, &parity::change::Inputs { baseline_run: Some(baseline), landed: Vec::new() })
        .expect("updated section");
    let file = directory.join("change.json");
    smith_bench::write_summary(&file, &summary).expect("stopped section");
    let code = parity::change::command(&file);
    assert_eq!(code, 0);
    assert_eq!(summary.change.expect("section").stop, Some(parity::change::Stop::Moved { criterion: "FT".into() }));
}

#[test]
fn a_change_command_refuses_an_input_whose_hash_differs() {
    let (summary, scratch) = common::named_change("digest");
    let directory = scratch.path().join(format!("smith-change-{}-{}", "digest", std::process::id()));
    let file = directory.join("change.json");
    smith_bench::write_summary(&file, &summary).expect("stored section");
    std::fs::write(directory.join("baseline.json"), b"changed input").expect("changed baseline");
    let code = parity::change::command(&file);
    assert_eq!(code, 3);
}

#[test]
fn a_change_command_refuses_a_missing_input() {
    let (summary, scratch) = common::named_change("missing");
    let directory = scratch.path().join(format!("smith-change-{}-{}", "missing", std::process::id()));
    let file = directory.join("change.json");
    smith_bench::write_summary(&file, &summary).expect("stored section");
    std::fs::remove_file(directory.join("baseline.json")).expect("remove baseline");
    let code = parity::change::command(&file);
    assert_eq!(code, 3);
}

use smith_bench_fixtures::parity::recompute;

use smith_bench::Measure;
use smith_bench::parity::{self, Verdict};
