use super::*;
use std::path::PathBuf;

fn retried_total() -> AttemptResult {
    let mut result =
        crate::read_result(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/results/attempt.json"))
            .expect("result fixture");
    let total = result.tokens.iter_mut().find(|record| record.role == ScopeKind::Total).expect("total");
    total.usage.fresh = Measure::LowerBound { value: 10, missing: vec!["root: response-3: retried failure".into()] };
    total.usage.cache_read =
        Measure::LowerBound { value: 20, missing: vec!["root: response-3: retried failure".into()] };
    total.usage.cache_write =
        Measure::LowerBound { value: 0, missing: vec!["root: response-3: retried failure".into()] };
    total.usage.output = Measure::LowerBound { value: 30, missing: vec!["root: response-3: retried failure".into()] };
    result.identity.model.resolved_model = Measure::Observed { value: "gpt-6.1-sol".into() };
    result.identity.model.resolved_window = Measure::Observed { value: 272_000 };
    result
}

#[test]
fn a_total_missing_only_retried_failures_charges_its_lower_bound() {
    assert_eq!(known_tokens(&retried_total()), Some(60));
}

#[test]
fn a_total_missing_another_count_charges_nothing_known() {
    let mut result = retried_total();
    let total = result.tokens.iter_mut().find(|record| record.role == ScopeKind::Total).expect("total");
    total.usage.output = Measure::LowerBound { value: 30, missing: vec!["root: missing response".into()] };
    assert_eq!(known_tokens(&result), None);
}

fn priced_response() -> crate::ResponseTokens {
    crate::ResponseTokens {
        scope: "response-4".into(),
        model: "gpt-6.1-sol".into(),
        fresh: 10,
        cache_read: 20,
        cache_write: 0,
        output: 30,
    }
}

#[test]
fn pricing_reported_tokens_keeps_the_retried_failure_lower_bound() {
    assert_eq!(
        crate::result::priced_spend(&retried_total(), &[priced_response()]),
        Measure::LowerBound { value: 0.000_322, missing: vec!["root: response-3: retried failure".into()] }
    );
}

#[test]
fn pricing_a_total_with_another_gap_stays_unavailable() {
    let mut result = retried_total();
    let total = result.tokens.iter_mut().find(|record| record.role == ScopeKind::Total).expect("total");
    total.usage.output = Measure::LowerBound { value: 30, missing: vec!["root: missing response".into()] };
    assert_eq!(
        crate::result::priced_spend(&result, &[priced_response()]),
        Measure::unavailable("total output tokens are a lower bound: root: missing response")
    );
}

pub(super) fn no_request_requires_selected_complete_observations_without_inventing_tokens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut value = crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("fixture");
    value.tokens.clear();
    value.outcome.deciding = vec!["no-request".into(), "stream-whole".into()];
    value.outcome.checks = value
        .outcome
        .deciding
        .iter()
        .map(|name| crate::CheckVerdict {
            check: name.clone(),
            passed: Measure::Observed { value: true },
            evidence: vec![],
        })
        .collect();
    assert!(observed_no_request(&value));
    assert_eq!(known_tokens(&value), None, "NoRequest is evidence, not invented numeric usage");
    value.outcome.checks[1].passed = Measure::unavailable("missing authoritative terminal");
    assert!(!observed_no_request(&value));
    value.outcome.checks[1].passed = Measure::LowerBound { value: true, missing: vec!["loss".into()] };
    assert!(!observed_no_request(&value));
    value.outcome.checks[1].passed = Measure::Observed { value: true };
    value.outcome.deciding.pop();
    assert!(!observed_no_request(&value), "an advisory no-request observation cannot lift a token gap");
}

fn startup_refusal_result() -> AttemptResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut result = crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("fixture");
    result.tokens.clear();
    result.identity.agent = crate::Agent::Smith;
    result.outcome.end = End::Refused { setup: "expected invalid setting".into() };
    result.outcome.exit = Measure::Observed { value: crate::Exit::Code(2) };
    result.outcome.forced = crate::Forced::No;
    result.outcome.deciding = ["no-run", "end", "exit-code", "last-line", "tree-ended"].map(str::to_owned).to_vec();
    result.outcome.checks = result
        .outcome
        .deciding
        .iter()
        .map(|name| crate::CheckVerdict {
            check: name.clone(),
            passed: Measure::Observed { value: true },
            evidence: vec![],
        })
        .collect();
    result
}

#[test]
fn expected_startup_refusal_keeps_a_suite_running_without_token_totals() {
    let result = startup_refusal_result();
    let mut batch = Batch { results: vec![result], incomplete: false, probes: BTreeMap::new() };
    assert!(observed_no_request(&batch.results[0]));
    assert_eq!(known_tokens(&batch.results[0]), None);
    assert_eq!(crate::probe::judgment(&batch), crate::Judgment::Pass);
    for index in 0..batch.results[0].outcome.checks.len() {
        batch.results[0].outcome.checks[index].passed =
            Measure::LowerBound { value: true, missing: vec!["missing".into()] };
        assert!(!observed_no_request(&batch.results[0]));
        assert_eq!(crate::probe::judgment(&batch), crate::Judgment::Incomplete);
        batch.results[0].outcome.checks[index].passed = Measure::Observed { value: true };
    }
    batch.results[0].outcome.deciding.pop();
    assert!(!observed_no_request(&batch.results[0]));
    assert_eq!(crate::probe::judgment(&batch), crate::Judgment::Pass);
}

#[test]
fn an_expected_in_run_refusal_passes_only_when_every_deciding_check_passes() {
    let mut result = startup_refusal_result();
    result.outcome.deciding =
        ["end", "run-completed", "no-request", "stream-whole", "tree-ended"].map(str::to_owned).to_vec();
    result.outcome.checks = result
        .outcome
        .deciding
        .iter()
        .map(|name| crate::CheckVerdict {
            check: name.clone(),
            passed: Measure::Observed { value: true },
            evidence: vec![],
        })
        .collect();
    let mut batch = Batch { results: vec![result], incomplete: false, probes: BTreeMap::new() };
    assert!(!expected_startup_refusal(&batch.results[0]));
    assert_eq!(crate::probe::judgment(&batch), crate::Judgment::Pass);
    for index in 0..batch.results[0].outcome.checks.len() {
        batch.results[0].outcome.checks[index].passed = Measure::Observed { value: false };
        assert!(!expected_refusal(&batch.results[0]));
        assert_eq!(crate::probe::judgment(&batch), crate::Judgment::Incomplete);
        batch.results[0].outcome.checks[index].passed = Measure::unavailable("missing deciding observation");
        assert!(!expected_refusal(&batch.results[0]));
        batch.results[0].outcome.checks[index].passed = Measure::Observed { value: true };
    }
}

#[test]
fn a_proved_provider_rejection_charges_no_tokens_without_changing_usage_or_requests() {
    let mut result = startup_refusal_result();
    result.outcome.end = End::Failed { reason: crate::FailureReason::Model, detail: "HTTP 400".into() };
    result.counts.provider_attempts = Measure::Observed { value: 1 };
    let health = &mut result.health;
    health.passive_checks.push(crate::CheckVerdict {
        check: "provider-rejected-before-output".into(),
        passed: Measure::Observed { value: true },
        evidence: vec![],
    });
    assert!(rejected_before_output(&result));
    assert!(!observed_no_request(&result));
    assert_eq!(known_tokens(&result), None);
    assert_eq!(result.counts.provider_attempts, Measure::Observed { value: 1 });
    result.counts.provider_attempts = Measure::LowerBound { value: 1, missing: vec!["loss".into()] };
    assert!(!rejected_before_output(&result));
    result.counts.provider_attempts = Measure::Observed { value: 1 };
    result.health.passive_checks.last_mut().expect("proof").passed =
        Measure::Unavailable { reason: "unobserved".into() };
    assert!(!rejected_before_output(&result));
}

// Two arms, repeated twice, form complete interleaved blocks.
const BLOCK_ARMS: [&str; 2] = ["a", "b"];
const BLOCK_REPETITIONS: u32 = 2;
const BLOCK_SEED: u64 = 71;

fn block_requests() -> (crate::Suite, Vec<Prepared>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut suite = crate::read_suite(&root.join("tests/recorded/suites/sample.toml")).expect("suite");
    suite.design = Design::Interleaved;
    suite.repetitions = BLOCK_REPETITIONS;
    let result = crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("result");
    let mut requests = Vec::new();
    for arm in BLOCK_ARMS {
        for repetition in 0..BLOCK_REPETITIONS {
            let mut identity = result.identity.clone();
            identity.arm = arm.into();
            identity.attempt = format!("{arm}-{repetition}");
            requests.push(crate::request_fixture::request(root.join("tests/recorded/tasks/valid/task.toml"), identity));
        }
    }
    (suite, requests)
}

#[test]
fn seeded_blocks_replay_with_one_request_per_arm() {
    let (suite, requests) = block_requests();
    let first = ordered(&suite, requests.clone(), BLOCK_SEED).expect("interleaving");
    let second = ordered(&suite, requests, BLOCK_SEED).expect("same seed");
    assert_eq!(
        first.iter().map(|request| &request.identity.attempt).collect::<Vec<_>>(),
        second.iter().map(|request| &request.identity.attempt).collect::<Vec<_>>()
    );
    for block in first.chunks_exact(BLOCK_ARMS.len()) {
        assert_ne!(block[0].identity.arm, block[1].identity.arm);
    }
}

#[test]
fn duplicate_attempt_ids_refuse_before_dispatch() {
    let (suite, mut requests) = block_requests();
    requests.push(requests[0].clone());
    assert!(ordered(&suite, requests, BLOCK_SEED).expect_err("duplicate").contains("unique"));
}

#[test]
fn mixed_runs_refuse_before_dispatch() {
    let (suite, mut requests) = block_requests();
    requests[0].identity.run = "another-run".into();
    assert!(ordered(&suite, requests, BLOCK_SEED).expect_err("mixed runs").contains("one run"));
}

#[test]
fn incomplete_repetition_blocks_refuse_before_dispatch() {
    let (suite, mut requests) = block_requests();
    requests.pop();
    assert!(ordered(&suite, requests, BLOCK_SEED).expect_err("missing repetition").contains("repetition"));
}

fn token_result() -> AttemptResult {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut result = crate::read_result(&root.join("tests/recorded/results/attempt.json")).expect("result");
    let total = result.tokens.iter_mut().find(|record| record.role == ScopeKind::Total).expect("total");
    total.usage.fresh = Measure::Observed { value: 10 };
    total.usage.cache_read = Measure::Observed { value: 20 };
    total.usage.output = Measure::Observed { value: 30 };
    result
}

#[test]
fn observed_token_scopes_sum_before_dispatch() {
    // The supplied total contains 10 fresh, 20 cache-read and 30 output tokens.
    const TOKENS: u64 = 60;
    assert_eq!(known_tokens(&token_result()), Some(TOKENS));
}

#[test]
fn missing_token_scopes_keep_the_total_unknown() {
    let mut result = token_result();
    let total = result.tokens.iter_mut().find(|record| record.role == ScopeKind::Total).expect("total");
    total.usage.fresh = Measure::unavailable("missing child");
    assert_eq!(known_tokens(&result), None);
}

#[test]
fn missing_result_recovery_archives_evidence_and_removes_settled_scratch() {
    // The fake fresh attempt creates its scratch, then exits without a
    // result. Settlement must precede recovery and owned scratch deletion;
    // dispatcher resource counts never become agent measurements.
    let (_, requests) = block_requests();
    let request = &requests[0];
    let scratch = skein_scratch::Scratch::new("bench-batch");
    let state = scratch.path().to_path_buf().join(format!("smith-bench-dispatch-{}", std::process::id()));
    let guard = Guard::new(&state.join("user-home"), &state.join("config"), &state.join("state"), &[])
        .expect("synthetic guard");
    crate::isolate::create_directory(&guard, &state).expect("private state");
    let scratch = crate::run::scratch_root(&state, request).expect("planned scratch");
    let relative = scratch.strip_prefix(&state).expect("private scratch").display();
    let archived = format!("runs/{}/{}", request.identity.run, request.identity.attempt);
    fs::write(
        state.join("attempt"),
        format!("umask 077; mkdir -p '{relative}' '{archived}'; cp \"$1\" '{archived}/prepared.json'; printf transient > '{relative}/transient'; printf interrupted; exit 1\n"),
    )
    .expect("fake attempt program");
    let machine = Machine { path: "/usr/bin:/bin".into(), rustup_home: None };
    let recovered = dispatch(&guard, &machine, &state, &Clock::new(), Path::new("/bin/sh"), request, 1_000_000_000)
        .expect("settled missing result");
    assert!(matches!(recovered.outcome.end, End::HarnessError { .. }));
    assert!(matches!(recovered.resources.cpu_ms, Measure::Unavailable { .. }));
    let directory = state.join("runs").join(&request.identity.run).join(&request.identity.attempt);
    assert_eq!(fs::read(directory.join("dispatcher-stdout.log")).expect("retained output"), b"interrupted");
    assert!(!scratch.exists());
    assert!(recovered.artifacts.iter().any(|artifact| {
        artifact.path == Path::new("prepared.json")
            && artifact.sha256 == crate::arm::binary_digest(&directory.join(&artifact.path)).expect("digest")
    }));
}
