//! A completion names only the charter's bounded call identity and structured evidence.
use super::fixtures::{self, RUN};
use super::harness::{Harness, answered};
use crate::{Event, agent, host};
use skein_lib::Duration;

fn started(main: &[u8]) -> Harness {
    let mut h = Harness::new();
    let (owner, source) = h.decode(Box::new([]));
    let mut start = fixtures::start(RUN, &source);
    let mut charter = start.charter.into_value();
    charter.llm.model = main.into();
    let mut child = charter.llm.clone();
    child.endpoint = smith_domain_run::charter::Endpoint(9);
    child.model = b"child-model".as_slice().into();
    charter.models = Box::new([child]);
    start.charter = agent::Charter::new(charter, 8192).unwrap();
    h.run(Event::Decoded {
        agent: owner,
        start: Ok(Box::new(start)),
        endpoints: Box::new([b"anthropic".as_slice().into(), b"child-endpoint".as_slice().into()]),
    });
    h.say(agent::Up::Admitted);
    h
}

fn completion(
    failure: agent::CompletionFailure,
    evidence: agent::CompletionEvidence,
    model: u32,
    status: Option<u16>,
) -> agent::ModelFault {
    agent::ModelFault::Completion {
        failure,
        evidence,
        model,
        attempts: 3,
        report: agent::Report {
            status,
            code: b"private code".as_slice().into(),
            request: b"private request".as_slice().into(),
            field: b"private field".as_slice().into(),
        },
    }
}

fn finishes(h: &mut Harness, fault: agent::ModelFault, failure: host::RunFailure, model: Option<host::ModelFailure>) {
    h.hold = true;
    h.say(agent::Up::Answer {
        answer: agent::Answer {
            read: None,
            turns: 0,
            spent: 0,
            result: agent::RunResult::Failed { failure: agent::RunFailure::Model(fault) },
        },
    });
    h.gone();
    let rows = h.release();
    assert!(
        answered(
            &rows,
            &host::Ending::Failed {
                failure: host::Failure::Run(failure),
                model,
                detail: Box::new([]),
                work: host::Work { left: None, saved: None },
            }
        ),
        "{rows:?}"
    );
    assert!(h.domain.agents.is_empty(), "call names go with the bound agent");
}

fn named(kind: host::ModelFailureKind, evidence: host::ModelEvidence, status: Option<u16>) -> host::ModelFailure {
    host::ModelFailure {
        endpoint: b"anthropic".as_slice().into(),
        model: b"claude-opus-5-5".as_slice().into(),
        kind,
        evidence,
        status,
    }
}

#[test]
fn completion_names_follow_the_charter_index_and_status_evidence() {
    use agent::CompletionEvidence::{Response, Unknown, Unsent};
    for (index, evidence, report_status, expected_evidence, status) in [
        (0, Unsent, Some(403), host::ModelEvidence::Unsent, Some(403)),
        (0, Response { status: Some(503) }, None, host::ModelEvidence::Response, Some(503)),
        (0, Response { status: Some(503) }, Some(403), host::ModelEvidence::Response, Some(503)),
        (0, Response { status: None }, Some(403), host::ModelEvidence::Response, Some(403)),
        (0, Unknown, None, host::ModelEvidence::Unknown, None),
        (1, Unsent, None, host::ModelEvidence::Unsent, None),
        (2, Unsent, None, host::ModelEvidence::Unsent, None),
    ] {
        let mut h = started(b"claude-opus-5-5");
        let mut model = named(host::ModelFailureKind::Unavailable, expected_evidence, status);
        if index == 1 {
            model.endpoint = b"child-endpoint".as_slice().into();
            model.model = b"child-model".as_slice().into();
        }
        let expected = if index == 2 { None } else { Some(model) };
        finishes(
            &mut h,
            completion(agent::CompletionFailure::Unavailable, evidence, index, report_status),
            host::RunFailure::Model,
            expected,
        );
    }
}

#[test]
fn call_names_are_cut_at_a_character_boundary() {
    for (input, expected) in [
        (&[b'a'; 60][..], &[b'a'; 40][..]),
        ("123456789012345678901234567890123456789éafter".as_bytes(), b"123456789012345678901234567890123456789"),
        (
            "12345678901234567890123456789012345678éafter".as_bytes(),
            "12345678901234567890123456789012345678é".as_bytes(),
        ),
    ] {
        let mut h = started(input);
        let mut model = named(host::ModelFailureKind::Unavailable, host::ModelEvidence::Unsent, None);
        model.model = expected.into();
        finishes(
            &mut h,
            completion(agent::CompletionFailure::Unavailable, agent::CompletionEvidence::Unsent, 0, None),
            host::RunFailure::Model,
            Some(model),
        );
    }
}

#[test]
fn completion_classes_are_named_but_other_model_faults_are_not() {
    use agent::CompletionFailure as C;
    use host::ModelFailureKind as K;
    for (failure, kind) in [
        (C::Unavailable, K::Unavailable),
        (C::Overloaded, K::Overloaded),
        (C::RateLimited { retry_after: Duration::from_secs(1) }, K::RateLimited),
        (C::Exhausted { retry_after: None }, K::Exhausted),
        (C::TimedOut { phase: smith_domain_run::CompletionPhase::Head }, K::TimedOut),
        (C::ContextTooLong, K::ContextTooLong),
        (C::Invalid, K::Invalid),
        (C::Refused, K::Refused),
        (C::Unsupported, K::Unsupported),
        (C::Unauthorized, K::Unauthorized),
        (C::Protocol, K::Protocol),
        (C::Cancelled, K::Cancelled),
        (C::ApplicationLimit, K::Limit),
        (C::Limit { which: agent::Limit::Path(agent::Cap::Request), bound: 2048 }, K::Limit),
    ] {
        let mut h = started(b"claude-opus-5-5");
        finishes(
            &mut h,
            completion(failure, agent::CompletionEvidence::Unsent, 0, None),
            host::RunFailure::Model,
            Some(named(kind, host::ModelEvidence::Unsent, None)),
        );
    }
    for fault in [
        agent::ModelFault::Provider,
        agent::ModelFault::Declined,
        agent::ModelFault::Program { name: Box::new([]), exit: None, detail: Box::new([]) },
    ] {
        let mut h = Harness::new();
        let (_, source) = h.decode(Box::new([]));
        // Program diagnostics have their own operator-detail test.
        let result = agent::RunResult::Failed { failure: agent::RunFailure::Model(fault) };
        let names =
            crate::hand_out::model_names(&fixtures::start(RUN, &source), Box::new([b"anthropic".as_slice().into()]));
        let crate::hand_out::Finish::Failed { failure, model } = crate::hand_out::Finish::of(&result, &names) else {
            panic!("failed")
        };
        assert_eq!(failure, host::RunFailure::Model);
        assert_eq!(model, None);
    }
}
