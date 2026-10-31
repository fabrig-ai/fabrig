use super::end_failure;
use smith_events::{CompletionFailure, Evidence, FailureClass, FailureLimit};

#[test]
fn an_unsent_window_check_does_not_take_an_earlier_transient_failure_as_its_cause() {
    let projected = CompletionFailure {
        field: None,
        code: None,
        request_id: None,
        class: FailureClass::Limit,
        evidence: Evidence::Unsent,
        limit: Some(FailureLimit { which: b"window".as_slice().into(), bound: 6_144 }),
        status: None,
        detail: None,
        phase: None,
        retry_after_ms: None,
    };
    let observed = CompletionFailure {
        field: None,
        code: None,
        request_id: None,
        class: FailureClass::RateLimited,
        evidence: Evidence::Response,
        limit: None,
        status: Some(429),
        detail: Some(b"retry later".as_slice().into()),
        phase: None,
        retry_after_ms: Some(1),
    };
    assert_eq!(end_failure(projected.clone(), observed), projected);
}
