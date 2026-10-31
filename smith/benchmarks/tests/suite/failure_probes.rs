//! Failure probes inspect outside terminals; synthetic records are not live proof.


use smith_bench::agent::smith::events::Observer;
use smith_bench::{End, EventCheck, Exit, Forced, Measure, OutcomeCheck};
use smith_events::{AnswerClass, AnswerFailure, Status};

use crate::common;
use crate::common::observe;

#[test]
fn failure_variants_keep_their_class_codes_and_do_not_invent_startup_terminals() {
    let root = crate::common::repository();
    let task = smith_bench::check_task(&root.join("tasks/probes/failures/task.toml"), &root.join("../docs/design"))
        .expect("authored task and unchanged seed");
    for name in ["unknown-model", "one-turn"] {
        let variant = task.with_variant(name).expect("interactive failure variant");
        assert!(!variant.deciding.iter().any(|name| name == "exit-code" || name == "last-line"));
    }
    for name in ["derived-limit", "bad-setting"] {
        let variant = task.with_variant(name).expect("startup failure variant");
        assert!(variant.outcome.contains(&OutcomeCheck::ExitCode { code: 2 }));
        assert!(variant.deciding.iter().any(|name| name == "last-line"));
        assert!(variant.event.contains(&EventCheck::NoRun));
    }
    let one_turn = task.with_variant("one-turn").expect("typed reserve refusal");
    assert!(one_turn.event.contains(&EventCheck::RunCompleted {
        status: "refused".into(),
        class: Some("invalid".into()),
        which: Some("budget_reserve_turns".into()),
        reason: None,
    }));
    let unknown = task.with_variant("unknown-model").expect("provider failure");
    assert_eq!(unknown.smith["model"].as_str(), Some("codex/smith-bench-nonexistent-model-2026-10-11"));
    let derived = task.with_variant("derived-limit").expect("named relationship");
    assert_eq!(derived.smith["deployment.read_window"].as_integer(), Some(40961));
    assert_eq!(derived.smith["deployment.tool_payload"].as_integer(), Some(40960));
    assert!(!derived.event.iter().any(|check| matches!(check, EventCheck::RunCompleted { .. })));
    let bad = task.with_variant("bad-setting").expect("invalid native setting");
    assert_eq!(bad.smith["models.\"{model}\".output"].as_integer(), Some(0));
    assert!(bad.event.contains(&EventCheck::NoRun));
    let mut empty = Observer::new(&common::events::LIMITS).expect("reader");
    empty.finish().expect("empty native capture framed completely");
    assert!(matches!(empty.check(&EventCheck::NoRequest, "task").passed, Measure::Unavailable { .. }));
    assert_eq!(empty.check(&EventCheck::NoRun, "task").passed, Measure::Observed { value: true });
    assert!(matches!(empty.classify(Exit::Code(2), Forced::No).end, End::HarnessError { .. }));
    empty.feed(b"{\"v\":4,\"type\":\"future.record\",\"t_ms\":0}\n").expect("unknown native record");
    empty.finish().expect("complete unknown line");
    assert_eq!(empty.check(&EventCheck::NoRun, "task").passed, Measure::Observed { value: false });
}

#[test]
fn model_failure_requires_the_run_class_and_the_observed_matching_exit() {
    let mut records = common::events::balanced();
    let failure = AnswerFailure {
        state: None,
        code: None,
        program: None,
        exit: None,
        detail: None,
        reset_ms: None,
        used: None,
        bound: None,
        dropped: None,
        ended: None,
        turn: None,
        class: AnswerClass::Model,
        endpoint: None,
        model: None,
        attempts: None,
        completion: None,
        account: None,
        which: None,
        reason: Some(Box::from(&b"unknown model"[..])),
    };
    {
        let value = crate::common::edit::run_completed(&mut records[6]);
        value.status = Status::Failed;
        value.result = None;
        value.failure = Some(failure);
    }
    {
        let value = crate::common::edit::session_ended(&mut records[7]);
        value.exit = 1;
    }
    let check =
        EventCheck::RunCompleted { status: "failed".into(), class: Some("model".into()), which: None, reason: None };
    let observer = observe(&records);
    assert_eq!(observer.check(&check, "task").passed, Measure::Observed { value: true });
    assert!(matches!(observer.classify(Exit::Code(1), Forced::No).end, End::Failed { .. }));
    assert!(matches!(observer.classify(Exit::Code(0), Forced::No).end, End::HarnessError { .. }));
    {
        let value = crate::common::edit::session_started(&mut records[0]);
        value.mode = smith_events::Mode::Chat;
    }
    {
        let value = crate::common::edit::session_ended(&mut records[7]);
        value.exit = 0;
    }
    assert!(matches!(observe(&records).classify(Exit::Code(0), Forced::No).end, End::Failed { .. }));
    assert!(matches!(observe(&records).classify(Exit::Code(1), Forced::No).end, End::HarnessError { .. }));
    {
        let value = crate::common::edit::run_completed(&mut records[6]);
        value.failure.as_mut().expect("failure").class = AnswerClass::Policy;
    }
    assert_eq!(observe(&records).check(&check, "task").passed, Measure::Observed { value: false });
}

#[test]
fn provider_failure_uses_typed_status_and_retained_bounded_detail() {
    let check = EventCheck::ProviderFailure { status_at_least: 400, status_at_most: 499, detail_bytes_at_most: 1024 };
    let mut records = common::events::balanced();
    let mut failure = smith_events::CompletionFailure {
        field: None,
        code: None,
        request_id: None,
        class: smith_events::FailureClass::Invalid,
        evidence: smith_events::Evidence::Response,
        retry_after_ms: None,
        phase: None,
        limit: None,
        status: Some(404),
        detail: Some(Box::from(&b"provider rejected unknown model"[..])),
    };
    set_completion_failure(&mut records, &failure);
    let verdict = observe_provider(&records).check(&check, "task");
    assert_eq!(verdict.passed, Measure::Observed { value: true });
    assert!(verdict.evidence[0].contains("HTTP=404 detail_bytes=31 bound=1024"));
    for (status, detail, expected) in [
        (Some(400), Some(vec![b'x'; 1024]), Some(true)),
        (Some(200), Some(b"HTTP 404 unknown model".to_vec()), Some(false)),
        (Some(404), Some(vec![b'x'; 1025]), Some(false)),
        (Some(404), Some(vec![]), Some(false)),
        (Some(400), Some(b"provider HTTP error 400".to_vec()), Some(false)),
        (Some(400), Some(b"provider HTTP error 404".to_vec()), Some(false)),
        (Some(400), Some(br#"{"detail":"unknown model"}"#.to_vec()), Some(true)),
        (Some(400), Some(b"unknown model from upstream".to_vec()), Some(true)),
        (None, Some(b"HTTP 404 unknown model".to_vec()), None),
        (Some(404), None, None),
    ] {
        failure.status = status;
        failure.detail = detail.map(Vec::into_boxed_slice);
        set_completion_failure(&mut records, &failure);
        let got = observe_provider(&records).check(&check, "task").passed;
        match expected {
            Some(value) => assert_eq!(got, Measure::Observed { value }),
            None => assert!(matches!(got, Measure::Unavailable { .. })),
        }
    }
    {
        let run = crate::common::edit::run_completed(&mut records[6]);
        run.failure.as_mut().expect("answer failure").completion = None;
    }
    assert!(matches!(observe_provider(&records).check(&check, "task").passed, Measure::Unavailable { .. }));
}

fn set_completion_failure(records: &mut [smith_events::Record], completion: &smith_events::CompletionFailure) {
    {
        let run = crate::common::edit::run_completed(&mut records[6]);
        run.status = Status::Failed;
        run.result = None;
        run.failure = Some(AnswerFailure {
            state: None,
            code: None,
            program: None,
            exit: None,
            detail: None,
            reset_ms: None,
            used: None,
            bound: None,
            dropped: None,
            ended: None,
            turn: None,
            class: AnswerClass::Model,
            endpoint: Some(Box::from(&b"codex"[..])),
            model: Some(Box::from(&b"smith-bench-nonexistent-model-2026-10-11"[..])),
            attempts: Some(1),
            completion: Some(completion.clone()),
            account: None,
            which: None,
            reason: None,
        });
    }
    {
        let session = crate::common::edit::session_ended(&mut records[7]);
        session.exit = 1;
    }
}


// The fixture envelope admits a deliberate 1025-byte defect. The authored
// predicate still enforces the independently derived native 1024-byte cap.
fn observe_provider(records: &[smith_events::Record]) -> Observer {
    let limits = smith_events::Limits { string: 2048, content: 2048, items: 16 };
    crate::common::captured(records, &smith_events::Capture::Everything, &limits)
}

#[test]
fn reserve_refusal_requires_exact_typed_identity_under_every_capture() {
    let check = EventCheck::RunCompleted {
        status: "refused".into(),
        class: Some("invalid".into()),
        which: Some("budget_reserve_turns".into()),
        reason: None,
    };
    let original = common::events::balanced();
    let mut records = vec![original[0].clone(), original[6].clone(), original[7].clone()];
    {
        let run = crate::common::edit::run_completed(&mut records[1]);
        run.status = Status::Refused;
        run.result = None;
        run.failure = Some(AnswerFailure {
            state: None,
            code: None,
            program: None,
            exit: None,
            detail: None,
            reset_ms: None,
            used: None,
            bound: None,
            dropped: None,
            ended: None,
            turn: None,
            class: AnswerClass::Invalid,
            endpoint: None,
            model: None,
            attempts: None,
            completion: None,
            account: None,
            which: Some(Box::from(&b"budget_reserve_turns"[..])),
            reason: None,
        });
    }
    {
        let session = crate::common::edit::session_ended(&mut records[2]);
        session.exit = 2;
    }
    common::events::inventory(&mut records);
    for capture in [smith_events::Capture::None, smith_events::Capture::Calls, smith_events::Capture::Everything] {
        let observer = observe_capture(&records, &capture);
        assert_eq!(observer.check(&check, "task").passed, Measure::Observed { value: true });
        assert_eq!(observer.check(&EventCheck::NoRequest, "task").passed, Measure::Observed { value: true });
        assert!(matches!(observer.classify(Exit::Code(2), Forced::No).end, End::Refused { .. }));
        for which in [None, Some("budget"), Some("budget_reserve_spend"), Some("budget_reserve_time")] {
            let mut changed = records.clone();
            {
                let run = crate::common::edit::run_completed(&mut changed[1]);
                let failure = run.failure.as_mut().expect("refusal");
                failure.which = which.map(|value| Box::from(value.as_bytes()));
                // Suggestive prose cannot replace the typed identity.
                failure.reason = Some(Box::from(&b"budget reserve turns"[..]));
            }
            let got = observe_capture(&changed, &capture).check(&check, "task").passed;
            match which {
                None => assert!(matches!(got, Measure::Unavailable { .. })),
                Some(_) => assert_eq!(got, Measure::Observed { value: false }),
            }
        }
        let mut changed = records.clone();
        {
            let run = crate::common::edit::run_completed(&mut changed[1]);
            run.failure.as_mut().expect("refusal").class = AnswerClass::Budget;
        }
        assert_eq!(
            observe_capture(&changed, &capture).check(&check, "task").passed,
            Measure::Observed { value: false }
        );
        {
            let run = crate::common::edit::run_completed(&mut changed[1]);
            run.failure.as_mut().expect("refusal").class = AnswerClass::Invalid;
            run.status = Status::Failed;
        }
        assert_eq!(
            observe_capture(&changed, &capture).check(&check, "task").passed,
            Measure::Observed { value: false }
        );
    }
}

fn observe_capture(records: &[smith_events::Record], capture: &smith_events::Capture) -> Observer {
    crate::common::captured(records, capture, &common::events::LIMITS)
}
