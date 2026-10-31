#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public event fixtures inspect supplied owner facts and trees"
)]

use crate::tests::fixtures::Fixture;
use crate::tests::tree::{
    cards::{contains, text, words},
    cards_objects::task_doc,
    pages::{build, deliver},
    work_pages::entered,
};
use crate::{
    Request,
    state::address::{Address, TaskSection},
};
use jig_client::contract::*;
use skein_lib::{Token, Wall};
use skein_ui::Tree;

fn active(stage: RunStage) -> ActiveRun {
    ActiveRun {
        stage,
        phase_since: Wall::from_nanos(7),
        attempt: Some(u64::MAX),
        placement: Some(Wall::from_nanos(13)),
        retry: None,
        observation: None,
    }
}
fn ready(runtime: Runtime, details: ChipPhase, phase: Phase) -> Fixture {
    let (mut d, out) = entered(Address::Task { task: 8, section: TaskSection::Activity });
    let watch =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let mut task = task_doc(&[]);
    task.chip.executor = Executor::Agent { charter: 2 };
    task.chip.phase = phase;
    *task.chip.details = details;
    task.context.as_mut().expect("supplied task context").runtime = runtime;
    task.hold = None;
    deliver(&mut d, watch, Delivery::Task(Box::new(task)));
    d
}

#[test]
fn unobserved_activation_preserves_phase_attempt_and_placement_without_inventing_work_or_start() {
    for stage in [
        RunStage::Idle,
        RunStage::Due,
        RunStage::Preparing,
        RunStage::Claimed { attempt: u64::MAX },
        RunStage::Running { attempt: u64::MAX },
        RunStage::BackingOff { until: Wall::from_nanos(19) },
    ] {
        let tree = shown(active(stage));
        for fact in [
            b"18446744073709551615".as_slice(),
            b"Phase since",
            b"Placement recorded at",
            b"Run observation not supplied",
        ] {
            assert!(contains(&tree, fact));
        }
        assert!(!contains(&tree, b"Observed work: writing"));
        assert!(!contains(&tree, b"Start observed at"));
        assert!(!contains(&tree, b"Preparing a host"));
    }
}
#[test]
fn observed_work_preserves_actual_optional_start_and_whole_tool_call_identity() {
    for (work, expected) in [
        (RunWork::Preparing, b"Observed work: preparing".as_slice()),
        (RunWork::Starting, b"Observed work: starting"),
        (RunWork::Active, b"Observed work: active"),
        (RunWork::Writing, b"Observed work: writing"),
        (RunWork::Waiting, b"Observed work: waiting"),
        (RunWork::Ending, b"Observed work: ending"),
        (
            RunWork::Tool { call: current_identity(b"actual-call/23"), tool: text(b"actual-tool") },
            b"Observed current tool call",
        ),
    ] {
        for started in [None, Some(Wall::EPOCH)] {
            let mut value = active(RunStage::Running { attempt: u64::MAX });
            value.observation = Some(RunObservation {
                channel: Token::new(u64::MAX),
                work: work.clone(),
                since: Wall::from_nanos(17),
                reported_at: Wall::from_nanos(23),
                started,
            });
            let tree = shown(value);
            assert!(contains(&tree, expected));
            if let RunWork::Tool { .. } = &work {
                assert!(contains(&tree, b"actual-tool"));
                assert!(contains(&tree, b"actual-call/23"));
            }
            assert!(contains(&tree, b"Observed work since"));
            assert!(contains(&tree, b"Observation received at"));
            assert_eq!(contains(&tree, b"Start observed at"), started.is_some());
            assert_eq!(contains(&tree, b"Start observation not supplied"), started.is_none());
            if work != RunWork::Writing {
                assert!(!contains(&tree, b"Observed work: writing"));
            }
        }
    }
}
#[test]
fn retry_notices_keep_refusal_and_failure_counters_without_fabricated_reasons() {
    for (retry, expected) in [
        (RetryNotice::Refused { count: u32::MAX }, b"Activation refusal count 4294967295".as_slice()),
        (RetryNotice::Failed { class: FailureClass::Stall, count: 7, allowance: 9 }, b"Failure count 7 of allowance 9"),
    ] {
        let mut value = active(RunStage::BackingOff { until: Wall::from_nanos(29) });
        value.retry = Some(retry);
        let tree = shown(value);
        assert!(contains(&tree, expected));
        assert!(!contains(&tree, b"Reason:"));
        assert!(!contains(&tree, b"Start observed at"));
        match retry {
            RetryNotice::Refused { .. } => assert!(!contains(&tree, b"Failure count")),
            RetryNotice::Failed { .. } | RetryNotice::OwnerFailed { .. } => {}
        }
    }
}

fn current_identity(id: &[u8]) -> CallIdentity {
    CallIdentity { owner: Token::new(u64::MAX), sequence: u32::MAX, retry: u32::MAX, id: text(id) }
}

fn shown(value: ActiveRun) -> Tree {
    let d = ready(
        Runtime::Active(value.clone()),
        crate::tests::fixtures::chip_phase(OwnerChipPhase::Active(value.stage)),
        Phase::Running,
    );
    build(&d, 3, words()).0
}
