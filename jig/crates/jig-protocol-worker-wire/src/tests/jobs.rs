//! Jobs retain their own records (jig/docs/design/protocol/channel/records.md, 4).
use super::{fixtures, records::through_bytes};
use crate::v17::Record;
use crate::{self as wire, v17};
use skein_lib::{Duration, List};

const TASK: u64 = 3;
const ATTEMPT: u64 = 4;
const FIRST_KIND: u32 = 7;
const SECOND_KIND: u32 = 11;
// The fixture's spec bound is 32 bytes.
const SPEC_BYTES: usize = 32;

fn job_parts(spec: &[u8]) -> wire::Job {
    let workspace = wire::Workspace { workstream: 7, items: b"workspace-items".as_slice().into() };
    let mut grants = List::with_capacity(1);
    grants.push(fixtures::grant(fixtures::application_value(), 5, 9)).expect("one application grant");
    wire::Job {
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
    wire::Work { left: Some(b"left-work".as_slice().into()), saved: None }
}

fn endings() -> [wire::JobEnding; 3] {
    let finished = wire::JobFinished { result: b"opaque-result".as_slice().into(), work: work() };
    let stale = wire::DeliveryOutcomeValue { value: wire::DeliveryOutcome::Stale };
    let failed = wire::JobFailed {
        failure: wire::JobFailure::Undelivered(stale),
        detail: b"bounded-detail".as_slice().into(),
        work: work(),
    };
    let invalid = wire::JobInvalidValue { value: wire::JobInvalid::Kind };
    let refused = wire::JobRefusalValue { value: wire::JobRefusal::Invalid(invalid) };
    [wire::JobEnding::Finished(finished), wire::JobEnding::Failed(failed), wire::JobEnding::Refused(refused)]
}

/// The spec holds at its byte bound (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_at_its_spec_bound_round_trips() {
    let job = job_parts(&[b's'; SPEC_BYTES]);
    let decoded = through_bytes(&job);
    assert_eq!(decoded, job);
}

/// One byte past the spec bound is refused (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_job_one_byte_past_its_spec_bound_is_refused() {
    let problem = (job_parts(&[b's'; SPEC_BYTES + 1])).encode(&fixtures::LIMITS).expect_err("spec past bound");
    assert_eq!(problem, v17::Problem { path: v17::paths::JOB_SPEC, reason: skein_codec::Reason::Bound });
}

/// Finished, failed and refused answers keep their payloads (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn job_answers_keep_their_endings_through_bytes() {
    for ending in endings() {
        let answer = wire::JobAnswer { task: TASK, attempt: ATTEMPT, ending };
        let decoded = through_bytes(&answer);
        assert_eq!(decoded, answer);
    }
}

fn hello_parts(kinds: &[u32]) -> wire::Hello {
    let mut parts = fixtures::hello_smallest();
    parts.jobs = List::with_capacity(u32::try_from(kinds.len()).expect("small list"));
    for kind in kinds {
        let bound = wire::JobBound { kind: *kind, spec: 32, result: 16, deadline: Duration::from_secs(60) };
        parts.jobs.push(bound).expect("one slot per kind");
    }
    parts
}

/// Two declared kinds hold at the list bound (jig/docs/design/protocol/channel/README.md, 4.3, and jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_hello_with_two_job_kinds_round_trips() {
    let hello = hello_parts(&[FIRST_KIND, SECOND_KIND]);
    let decoded = through_bytes(&hello);
    assert_eq!(decoded, hello);
}

/// A third kind exceeds the fixture's list bound (jig/docs/design/protocol/channel/records.md, 4).
#[test]
fn a_hello_one_job_kind_past_its_bound_is_refused() {
    const THIRD_KIND: u32 = 19;
    let problem =
        (hello_parts(&[FIRST_KIND, SECOND_KIND, THIRD_KIND])).encode(&fixtures::LIMITS).expect_err("third job kind");
    assert_eq!(problem, v17::Problem { path: v17::paths::HELLO_JOBS, reason: skein_codec::Reason::Bound });
}
