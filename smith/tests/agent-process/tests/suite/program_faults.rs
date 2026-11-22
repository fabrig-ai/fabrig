//! Focused observations behind the program sweeps, including unconfigured faults.

use skein_lib::{Duration, Time};
use skein_world::faults::{Counter, FaultLedger, Struck};
use smith_agent_process_world::{
    charter, program,
    program_world::{self, Case},
    wire,
};
use smith_fake_claude::{End, Script, Step, Story, Turn};

fn run(turns: Vec<Turn>, case: Case) -> program_world::Run {
    let charter = wire::edit_charter(&charter(), |parts| wire::budget(parts, 8, 20));
    let story = Story { cued: vec![Script { cue: String::new(), turns }], ..Story::default() };
    program_world::run(case, &story, Some(b"claude.ai"), &charter, &program(b"source"), None)
}

fn finish() -> Turn {
    Turn::new(vec![Step::Outcome { input: serde_json::json!({"outcome":{"form":"report","text":"done","fields":{}}}) }])
}

#[test]
fn played_program_faults_keep_their_kind_time_and_host_account() {
    use skein_llm_kit::faults::{EXPIRED_SIGN_IN, HANG_AFTER_HEAD, RATE_WINDOW, UNSERVED_MODEL};
    for (turn, expected) in [
        (Turn::new(vec![Step::RateLimit { resets_at: None }]), RATE_WINDOW),
        (
            Turn::new(vec![Step::RateLimitWindow {
                status: "rejected".into(),
                rate_limit_type: None,
                resets_at: None,
                utilization: None,
            }]),
            RATE_WINDOW,
        ),
        (Turn::new(vec![Step::LoseLogin, Step::Text("continue".into())]), EXPIRED_SIGN_IN),
        (Turn::new(vec![Step::Hang]), HANG_AFTER_HEAD),
        (Turn { steps: vec![], end: End::edge_refusal() }, smith_faults::EDGE_REFUSAL),
        (Turn { steps: vec![], end: End::billing_refusal() }, smith_faults::BILLING_REFUSAL),
        (Turn { steps: vec![], end: End::unserved_model("fake") }, UNSERVED_MODEL),
    ] {
        let mut ledger = FaultLedger::new();
        let fault = if let Some(step) = turn.steps.first() {
            smith_fake_claude::Fault::Step(step.clone())
        } else {
            smith_fake_claude::Fault::End(turn.end)
        };
        program_world::faults::record_played(
            &mut ledger,
            "host account",
            &[smith_fake_claude::Seen::Fault { fault, at_nanos: 37 }, smith_fake_claude::Seen::Interrupted],
        );
        let fault = ledger.entries().first().expect("played fault");
        assert_eq!(fault.kind, expected);
        assert_eq!(fault.from, Time::from_nanos(37));
        assert_eq!(fault.until, None, "fault remains in force through the run");
        assert_eq!(fault.struck.account.as_deref(), Some("host account"));
        assert_eq!(fault.struck.request.as_deref(), (expected == RATE_WINDOW).then_some("1"));
    }
    let mut ledger = FaultLedger::new();
    for fault in [
        smith_fake_claude::Fault::Step(Step::RateLimitWindow {
            status: "allowed".into(),
            rate_limit_type: None,
            resets_at: None,
            utilization: None,
        }),
        smith_fake_claude::Fault::End(End::Error {
            api_error_status: Some(404),
            terminal_reason: None,
            errors: vec![],
            result: Some("other error".into()),
            resets_at: None,
        }),
    ] {
        program_world::faults::record_played(
            &mut ledger,
            "host account",
            &[smith_fake_claude::Seen::Fault { fault, at_nanos: 38 }],
        );
    }
    assert!(ledger.entries().is_empty(), "allowed windows and other errors are not configured faults");
}

#[test]
fn played_rate_limits_stop_at_the_reset_or_the_moment_played() {
    let start = skein_sim::Config::calm().wall.as_nanos() / 1_000_000_000;
    let played = Time::from_nanos(37);
    for (reset, until) in [(start + 123, Time::from_nanos(123_000_000_000)), (start, played), (start - 1, played)] {
        for step in [
            Step::RateLimit { resets_at: Some(reset) },
            Step::RateLimitWindow {
                status: "rejected".into(),
                rate_limit_type: None,
                resets_at: Some(reset),
                utilization: None,
            },
        ] {
            let mut ledger = FaultLedger::new();
            program_world::faults::record_played(
                &mut ledger,
                "host account",
                &[smith_fake_claude::Seen::Fault { fault: smith_fake_claude::Fault::Step(step), at_nanos: 37 }],
            );
            let fault = ledger.entries().first().expect("played rate limit");
            assert_eq!(fault.kind, skein_llm_kit::faults::RATE_WINDOW);
            assert_eq!(fault.from, played);
            assert_eq!(fault.until, Some(until), "reset {reset}");
            assert_eq!(fault.struck.account.as_deref(), Some("host account"));
            assert_eq!(fault.struck.request.as_deref(), Some("1"));
        }
    }
}

fn response_started(response: u64, turn: u64, attempt: u64, messages: u64) -> smith_events::Record {
    smith_events::Record {
        t_ms: response * 10,
        event: smith_events::Event::ResponseStarted(smith_events::ResponseStarted {
            run: 1,
            conversation: 0,
            response,
            turn,
            attempt,
            window: 1,
            model: smith_events::Model {
                endpoint: Box::default(),
                model: Box::default(),
                window_tokens: 100,
                output_tokens: 10,
                effort: None,
                account: Some(0),
                backend: smith_events::Backend::ClaudeCode,
            },
            messages,
            output_tokens: 10,
            tool_choice: smith_events::ToolChoice::Auto,
            prompt: None,
        }),
    }
}

fn response_failed(response: u64, retry_ms: Option<u64>) -> smith_events::Record {
    smith_events::Record {
        t_ms: response * 10 + 1,
        event: smith_events::Event::ResponseCompleted(smith_events::ResponseCompleted {
            run: 1,
            conversation: 0,
            response,
            outcome: smith_events::Outcome::Failed,
            stop: None,
            blocks: 0,
            calls: 0,
            invalid: 0,
            oversized: 0,
            cut: 0,
            usage: smith_events::Usage {
                input_tokens: None,
                cache_read_tokens: None,
                cache_write_tokens: None,
                output_tokens: None,
                reasoning_tokens: None,
            },
            spent: 0,
            request_bytes: None,
            first_byte_ms: None,
            largest_gap_ms: None,
            duration_ms: 1,
            failure: None,
            retry_ms,
            completion: None,
        }),
    }
}

#[test]
fn response_records_keep_retry_layers_times_and_call_identities() {
    let mut ledger = FaultLedger::new();
    let struck = Struck { account: Some("host account".into()), ..Struck::default() };
    let injection =
        ledger.inject(skein_llm_kit::faults::EXPIRED_SIGN_IN, Time::from_nanos(100_000_000), struck.clone());
    // The offset matters: unshifted record times precede the fault.
    let mut runs = smith_faults::Runs::new();
    program_world::faults::record_retries(
        &mut runs,
        Time::from_nanos(100_000_000),
        "host account",
        &[
            response_started(1, 1, 0, 1),
            response_failed(1, Some(10)),
            response_started(2, 1, 1, 1),
            response_started(3, 2, 0, 1),
            response_failed(3, Some(10)),
            response_started(4, 2, 1, 1),
            response_started(5, 3, 0, 1),
            response_failed(5, Some(10)),
            response_started(6, 3, 1, 0),
            // A response without retry_ms adds no program retry.
            response_started(7, 4, 0, 1),
            response_failed(7, None),
            response_started(8, 4, 1, 0),
        ],
    );
    runs.judge(&mut ledger).expect("one retry per logical call, not per account or response");
    assert_eq!(ledger.get(injection).count(Counter::SessionRetries), 2);
    assert_eq!(ledger.get(injection).count(Counter::ProviderRetries), 1);
}

#[test]
fn program_and_session_retries_are_counted_separately() {
    let mut run = run(
        vec![
            Turn { steps: vec![], end: End::edge_refusal() },
            Turn::new(vec![Step::Retry { attempt: 1, status: 503, delay_ms: Some(125) }]),
            finish(),
        ],
        Case::Events,
    );
    run.runs.judge(&mut run.ledger).expect("edge refusal may recover");
    let fault = run.ledger.entries().first().expect("edge refusal");
    assert_eq!(fault.count(Counter::SessionRetries), 1);
    assert_eq!(fault.count(Counter::ProviderRetries), 1);
}

#[test]
fn a_failed_relay_records_an_untooled_run_without_retry() {
    let mut run = run(vec![finish()], Case::MissingServer);
    run.runs.judge(&mut run.ledger).expect("untooled terminal");
    let fault = run.ledger.entries().first().expect("failed relay");
    assert_eq!(fault.kind, smith_faults::UNTOOLED_PROGRAM);
    assert!(fault.from < Time::ZERO.saturating_add(Duration::from_secs(1)));
    assert_eq!(fault.count(Counter::SessionRetries), 0);
    assert_eq!(fault.count(Counter::ProviderRetries), 0);
    assert_eq!(smith_faults::ended_channel(&run.answer().expect("answer")), smith_faults::Ended::Untooled);
}
