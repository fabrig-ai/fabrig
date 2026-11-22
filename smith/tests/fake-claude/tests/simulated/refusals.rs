//! Synthetic refusals and the fault report (temper/docs/design/testing/fakes.md, 1).

use crate::support::print::{self, messages};
use serde_json::{Value, json};
use smith_fake_claude::{End, Fault, Seen, Step, Story, Turn, seen};

fn result(end: End) -> Value {
    let run = print::normal(Story { turns: vec![Turn { steps: vec![], end }], ..Story::default() }, 1);
    let records = messages(&run);
    let result = records.last().expect("final result");
    assert_eq!(result["type"], "result");
    assert_eq!(result["is_error"], true);
    result.clone()
}

#[test]
fn billing_refusal_has_the_documented_envelope() {
    let result = result(End::billing_refusal());
    assert_eq!(result["api_error_status"], 402);
    assert_eq!(
        result["result"],
        r#"API Error: 402 {"type":"error","error":{"type":"billing_error","message":"This account's plan does not cover this request."},"request_id":"req_fake"}"#
    );
}

#[test]
fn edge_refusal_has_the_documented_envelope() {
    let result = result(End::edge_refusal());
    assert_eq!(result["api_error_status"], 403);
    assert_eq!(
        result["result"],
        "API Error: 403 <html><head><title>403 Forbidden</title></head><body><h1>403 Forbidden</h1></body></html>"
    );
}

#[test]
fn unserved_model_has_the_documented_envelope() {
    let model = "unserved-model";
    let mut arguments = print::arguments();
    let index = arguments.iter().position(|arg| arg == "--model").expect("model flag") + 1;
    arguments[index] = model.into();
    let run = print::run(
        Story { turns: vec![Turn { steps: vec![], end: End::unserved_model(model) }], ..Story::default() },
        print::invocation(1).arguments(&arguments.iter().map(String::as_str).collect::<Vec<_>>()),
        None,
        Some(print::MCP),
    );
    let records = messages(&run);
    assert_eq!(records[0]["model"], model);
    let result = records.last().expect("final result");
    assert_eq!(result["type"], "result");
    assert_eq!(result["is_error"], true);
    assert_eq!(result["api_error_status"], 404);
    assert_eq!(
        result["result"],
        r#"API Error: 404 {"type":"error","error":{"type":"not_found_error","message":"model: unserved-model"}}"#
    );
}

#[test]
fn an_error_prints_optional_text_beside_errors() {
    for text in [None, Some("API Error: custom".to_owned())] {
        let result = result(End::Error {
            api_error_status: Some(400),
            terminal_reason: Some("api_error".into()),
            errors: vec!["detail".into()],
            result: text.clone(),
            resets_at: None,
        });
        let mut expected = json!({
            "type":"result", "subtype":"error_during_execution", "is_error":true,
            "api_error_status":400, "terminal_reason":"api_error", "errors":["detail"],
            "session_id":print::SESSION,
        });
        if let Some(text) = text {
            expected["result"] = json!(text);
        }
        assert_eq!(result, expected);
    }
}

fn faults(bytes: &[u8]) -> Vec<(Fault, u64)> {
    seen(bytes)
        .into_iter()
        .filter_map(|seen| match seen {
            Seen::Fault { fault, at_nanos } => Some((fault, at_nanos)),
            _ => None,
        })
        .collect()
}

#[test]
fn played_faults_are_reported_once_in_order_at_the_host_clock() {
    let limit = Step::RateLimit { resets_at: Some(123) };
    let rejected = Step::RateLimitWindow {
        status: "rejected".into(),
        rate_limit_type: Some("five_hour".into()),
        resets_at: Some(456),
        utilization: Some(json!(1)),
    };
    let end = End::billing_refusal();
    let run = print::normal(
        Story {
            init: smith_fake_claude::Init { delay_ms: Some(37), ..Default::default() },
            turns: vec![Turn {
                steps: vec![
                    limit.clone(),
                    Step::RateLimitWindow {
                        status: "allowed".into(),
                        rate_limit_type: None,
                        resets_at: None,
                        utilization: None,
                    },
                    Step::RateLimitWindow {
                        status: "allowed_warning".into(),
                        rate_limit_type: None,
                        resets_at: None,
                        utilization: None,
                    },
                    rejected.clone(),
                    Step::LoseLogin,
                ],
                end: end.clone(),
            }],
            ..Story::default()
        },
        1,
    );
    let faults = faults(run.observations.as_ref().expect("observations"));
    assert_eq!(
        faults.iter().map(|(fault, _)| fault.clone()).collect::<Vec<_>>(),
        [Fault::Step(limit), Fault::Step(rejected), Fault::Step(Step::LoseLogin), Fault::End(end),]
    );
    assert!(faults.iter().all(|(_, at)| *at >= 37_000_000), "moments follow the delayed init: {faults:?}");
    assert!(faults.iter().map(|(_, at)| at).is_sorted());
}

#[test]
fn a_hang_is_reported_once_without_playing_the_unreached_end() {
    let invocation = print::signal(print::invocation(1), skein_io::kernel::ServiceSignal::Interrupt, "assistant");
    let run = print::run(
        Story {
            init: smith_fake_claude::Init { delay_ms: Some(37), ..Default::default() },
            turns: vec![Turn { steps: vec![Step::Text("waiting".into()), Step::Hang], end: End::edge_refusal() }],
            ..Story::default()
        },
        invocation,
        None,
        Some(print::MCP),
    );
    assert!(run.signalled);
    let faults = faults(run.observations.as_ref().expect("observations"));
    assert_eq!(faults.len(), 1);
    assert_eq!(faults[0].0, Fault::Step(Step::Hang));
    assert!(faults[0].1 >= 37_000_000);
}
