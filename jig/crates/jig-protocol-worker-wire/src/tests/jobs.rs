//! Jobs retain their own records (jig/docs/design/protocol/channel.md, 5.4).
use super::{fixtures, records::through_bytes};
use crate::{self as wire, v9};
use skein_lib::{Duration, List};

const TASK: u64 = 3;
const ATTEMPT: u64 = 4;
const FIRST_KIND: u32 = 7;
const SECOND_KIND: u32 = 11;
// The fixture's spec bound is 32 bytes.
const SPEC_BYTES: usize = 32;

fn job_parts(spec: &[u8]) -> wire::JobParts {
    let workspace = wire::Workspace::new(
        &fixtures::LIMITS,
        wire::WorkspaceParts { workstream: 7, items: b"workspace-items".as_slice().into() },
    )
    .expect("workspace within limits");
    let mut grants = List::with_capacity(1);
    grants.push(fixtures::grant(fixtures::application_value(), 5, 9)).expect("one application grant");
    wire::JobParts {
        task: TASK,
        attempt: ATTEMPT,
        kind: FIRST_KIND,
        spec: spec.into(),
        deadline: Duration::from_secs(60),
        workspace: Some(workspace),
        grants,
    }
}

fn work() -> wire::Work {
    wire::Work::new(&fixtures::LIMITS, wire::WorkParts { left: Some(b"left-work".as_slice().into()), saved: None })
        .expect("job's opaque work")
}

fn endings() -> [wire::JobEnding; 3] {
    let finished = wire::JobFinished::new(
        &fixtures::LIMITS,
        wire::JobFinishedParts { result: b"opaque-result".as_slice().into(), work: work() },
    )
    .expect("finished job");
    let stale = wire::DeliveryOutcomeValue::new(
        &fixtures::LIMITS,
        wire::DeliveryOutcomeValueParts { value: wire::DeliveryOutcome::Stale },
    )
    .expect("stale delivery");
    let failed = wire::JobFailed::new(
        &fixtures::LIMITS,
        wire::JobFailedParts {
            failure: wire::JobFailure::Undelivered(stale),
            detail: b"bounded-detail".as_slice().into(),
            work: work(),
        },
    )
    .expect("failed job");
    let invalid =
        wire::JobInvalidValue::new(&fixtures::LIMITS, wire::JobInvalidValueParts { value: wire::JobInvalid::Kind })
            .expect("invalid job kind");
    let refused = wire::JobRefusalValue::new(
        &fixtures::LIMITS,
        wire::JobRefusalValueParts { value: wire::JobRefusal::Invalid(invalid) },
    )
    .expect("refused job");
    [wire::JobEnding::Finished(finished), wire::JobEnding::Failed(failed), wire::JobEnding::Refused(refused)]
}

/// The spec holds at its byte bound (jig/docs/design/protocol/channel.md, 5.4).
#[test]
fn a_job_at_its_spec_bound_round_trips() {
    let job = wire::Job::new(&fixtures::LIMITS, job_parts(&[b's'; SPEC_BYTES])).expect("spec at bound");
    let decoded = through_bytes(&job, wire::Job::measure, wire::Job::encode, wire::Job::decode);
    assert_eq!(decoded, job);
}

/// One byte past the spec bound is refused (jig/docs/design/protocol/channel.md, 5.4).
#[test]
fn a_job_one_byte_past_its_spec_bound_is_refused() {
    let problem = wire::Job::new(&fixtures::LIMITS, job_parts(&[b's'; SPEC_BYTES + 1])).expect_err("spec past bound");
    assert_eq!(problem, v9::Problem { path: v9::Path::JobSpec, reason: skein_codec::Reason::Bound });
}

/// Finished, failed and refused answers keep their payloads (jig/docs/design/protocol/channel.md, 5.4).
#[test]
fn job_answers_keep_their_endings_through_bytes() {
    for ending in endings() {
        let answer =
            wire::JobAnswer::new(&fixtures::LIMITS, wire::JobAnswerParts { task: TASK, attempt: ATTEMPT, ending })
                .expect("job answer within limits");
        let decoded =
            through_bytes(&answer, wire::JobAnswer::measure, wire::JobAnswer::encode, wire::JobAnswer::decode);
        assert_eq!(decoded, answer);
    }
}

fn hello_parts(kinds: &[u32]) -> wire::HelloParts {
    let mut parts = fixtures::hello_smallest().into_parts();
    parts.jobs = List::with_capacity(u32::try_from(kinds.len()).expect("small list"));
    for kind in kinds {
        let bound = wire::JobBound::new(
            &fixtures::LIMITS,
            wire::JobBoundParts { kind: *kind, spec: 32, result: 16, deadline: Duration::from_secs(60) },
        )
        .expect("job bound");
        parts.jobs.push(bound).expect("one slot per kind");
    }
    parts
}

/// Two declared kinds hold at the list bound (jig/docs/design/protocol/channel.md, 4.3 and 5.4).
#[test]
fn a_hello_with_two_job_kinds_round_trips() {
    let hello = wire::Hello::new(&fixtures::LIMITS, hello_parts(&[FIRST_KIND, SECOND_KIND])).expect("two job kinds");
    let decoded = through_bytes(&hello, wire::Hello::measure, wire::Hello::encode, wire::Hello::decode);
    assert_eq!(decoded, hello);
}

/// A third kind exceeds the fixture's list bound (jig/docs/design/protocol/channel.md, 5.4).
#[test]
fn a_hello_one_job_kind_past_its_bound_is_refused() {
    const THIRD_KIND: u32 = 19;
    let problem = wire::Hello::new(&fixtures::LIMITS, hello_parts(&[FIRST_KIND, SECOND_KIND, THIRD_KIND]))
        .expect_err("third job kind");
    assert_eq!(problem, v9::Problem { path: v9::Path::HelloJobs, reason: skein_codec::Reason::Bound });
}
