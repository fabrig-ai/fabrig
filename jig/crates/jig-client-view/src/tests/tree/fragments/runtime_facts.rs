#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public event fixtures inspect supplied owner facts and trees"
)]

use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::tests::tree::{
    cards::{text, words},
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
fn ready(runtime: Runtime, details: ChipPhaseFacts, phase: Phase) -> Fixture {
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
    task.chip.executor = ChipExecutor::Agent { charter: 2 };
    task.chip.phase = phase;
    *task.chip.details = details;
    task.context.as_mut().expect("supplied task context").runtime = runtime;
    task.hold = None;
    deliver(&mut d, watch, Delivery::Task(Box::new(task)));
    d
}

#[test]
fn unobserved_activation_preserves_phase_attempt_and_placement_without_inventing_work_or_start() {
    let mut goldens = Goldens::new("tree/fragments/runtime_facts/unobserved_activation_preserves");
    for stage in [
        RunStage::Idle,
        RunStage::Due,
        RunStage::Preparing,
        RunStage::Claimed { attempt: u64::MAX },
        RunStage::Running { attempt: u64::MAX },
        RunStage::BackingOff { until: Wall::from_nanos(19) },
    ] {
        let tree = shown(active(stage));
        goldens.check(&tree);
    }
}
#[test]
fn observed_work_preserves_actual_optional_start_and_whole_tool_call_identity() {
    let mut goldens = Goldens::new("tree/fragments/runtime_facts/observed_work_preserves");
    for work in [
        RunWork::Preparing,
        RunWork::Starting,
        RunWork::Active,
        RunWork::Writing,
        RunWork::Waiting,
        RunWork::Ending,
        RunWork::Tool { call: current_identity(b"actual-call/23"), tool: text(b"actual-tool") },
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
            goldens.check(&tree);
            if let RunWork::Tool { .. } = &work {
                goldens.check(&tree);
            }
        }
    }
}
#[test]
fn retry_notices_keep_refusal_and_failure_counters_without_fabricated_reasons() {
    let mut goldens = Goldens::new("tree/fragments/runtime_facts/retry_notices_keep");
    for retry in [
        RetryNotice::Refused { count: u32::MAX, reason: None },
        RetryNotice::Failed { class: FailureClass::Stall, count: 7, allowance: 9, reason: None },
    ] {
        let mut value = active(RunStage::BackingOff { until: Wall::from_nanos(29) });
        value.retry = Some(retry);
        let tree = shown(value);
        goldens.check(&tree);
    }
}

fn current_identity(id: &[u8]) -> CallIdentity {
    CallIdentity { owner: Token::new(u64::MAX), sequence: u32::MAX, retry: u32::MAX, id: text(id) }
}

fn shown(value: ActiveRun) -> Tree {
    let d = ready(
        Runtime::Active(value.clone()),
        crate::tests::fixtures::chip_phase(ChipPhase::Active(value.stage)),
        Phase::Running,
    );
    build(&d, 3, words()).0
}

#[test]
fn retry_facts_show_the_preparation_reason_after_the_count() {
    let mut goldens = Goldens::new("tree/fragments/runtime_facts/retry_facts_show");
    let reason = Some(DocumentText::complete(text(b"brief: no brief slot is free")));
    for retry in [
        RetryNotice::OwnerFailed { class: HoldFailure::Transient, count: 2, allowance: 3, reason: reason.clone() },
        RetryNotice::Failed { class: FailureClass::Temporary, count: 2, allowance: 3, reason: reason.clone() },
        RetryNotice::Refused { count: 2, reason },
    ] {
        let mut value = active(RunStage::BackingOff { until: Wall::from_nanos(29) });
        value.retry = Some(retry);
        let tree = shown(value);
        let text: Box<[_]> =
            tree.nodes().iter().flat_map(|node| node.text.iter()).map(|run| run.text.as_ref()).collect();
        let why = text.iter().position(|text| *text == b"Why: brief: no brief slot is free").expect("supplied reason");
        assert!(text[..why].iter().any(|text| skein_lib::bytes::find(text, b"count 2").is_some()));
        goldens.check(&tree);
    }
}
