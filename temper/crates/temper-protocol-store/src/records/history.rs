//! Total landing history translations (temper/docs/design/domain/landing-history.md, 2 and 4).
use super::{FITS, w};
use skein_lib::List;
use temper_engine_domain_forge_vocabulary::history as d;
fn encode_job(value: d::Job) -> w::HistoryJob {
    crate::generated::store::history_job_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_job(value: w::HistoryJob) -> d::Job {
    crate::generated::store::history_job_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
fn encode_job_failure(value: d::JobFailure) -> w::HistoryJobFailure {
    crate::generated::store::history_job_failure_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_job_failure(value: w::HistoryJobFailure) -> d::JobFailure {
    crate::generated::store::history_job_failure_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_job_outcome(value: d::JobOutcome) -> w::HistoryJobOutcome {
    match value {
        d::JobOutcome::Done => w::HistoryJobOutcome::Done,
        d::JobOutcome::Conflict => w::HistoryJobOutcome::Conflict,
        d::JobOutcome::Failed { class: d::JobFailure::Unknown } => w::HistoryJobOutcome::Failed,
        d::JobOutcome::Failed { class } => {
            w::HistoryJobOutcome::FailedClass(w::HistoryJobFailed { class: encode_job_failure(class) })
        }
        d::JobOutcome::Timeout => w::HistoryJobOutcome::Timeout,
        d::JobOutcome::Cancelled => w::HistoryJobOutcome::Cancelled,
        d::JobOutcome::Lost => w::HistoryJobOutcome::Lost,
    }
}
pub(super) fn decode_job_outcome(value: w::HistoryJobOutcome) -> d::JobOutcome {
    match value {
        w::HistoryJobOutcome::Done => d::JobOutcome::Done,
        w::HistoryJobOutcome::Conflict => d::JobOutcome::Conflict,
        w::HistoryJobOutcome::Failed => d::JobOutcome::Failed { class: d::JobFailure::Unknown },
        w::HistoryJobOutcome::FailedClass(value) => d::JobOutcome::Failed { class: decode_job_failure(value.class) },
        w::HistoryJobOutcome::Timeout => d::JobOutcome::Timeout,
        w::HistoryJobOutcome::Cancelled => d::JobOutcome::Cancelled,
        w::HistoryJobOutcome::Lost => d::JobOutcome::Lost,
    }
}
pub(super) fn encode_project(value: &d::Project) -> w::HistoryProject {
    crate::generated::store::history_project_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_project(value: w::HistoryProject) -> d::Project {
    crate::generated::store::history_project_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_change_index(value: &d::ChangeIndex) -> w::HistoryChange {
    crate::generated::store::history_change_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_change_index(value: w::HistoryChange) -> d::ChangeIndex {
    crate::generated::store::history_change_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_landing_index(value: &d::LandingIndex) -> w::HistoryLanding {
    crate::generated::store::history_landing_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_landing_index(value: w::HistoryLanding) -> d::LandingIndex {
    crate::generated::store::history_landing_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_last(value: &d::Last) -> w::HistoryLast {
    crate::generated::store::history_last_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_last(value: w::HistoryLast) -> d::Last {
    crate::generated::store::history_last_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_event(value: &d::Event) -> w::HistoryEvent {
    crate::generated::store::history_event_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_event(value: w::HistoryEvent) -> d::Event {
    crate::generated::store::history_event_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_job_record(job: Option<&d::JobRecord>) -> Option<w::HistoryJobRecord> {
    let job = job?;
    let mut members = List::with_capacity(u32::try_from(job.members.len()).expect(FITS));
    for member in &job.members {
        members.push(*member).expect(FITS);
    }
    Some(w::HistoryJobRecord {
        task: job.step.task,
        attempt: job.step.attempt,
        job: encode_job(job.step.job),
        outcome: match job.step.outcome {
            Some(outcome) => Some(encode_job_outcome(outcome)),
            None => None,
        },
        round: job.round,
        members,
        head: job.head,
        written: job.written,
    })
}
pub(super) fn decode_job_record(job: Option<w::HistoryJobRecord>) -> Option<d::JobRecord> {
    let w::HistoryJobRecord { task, attempt, job, outcome, round, members, head, written } = job?;
    Some(d::JobRecord {
        step: d::JobStep {
            task,
            attempt,
            job: decode_job(job),
            outcome: match outcome {
                Some(outcome) => Some(decode_job_outcome(outcome)),
                None => None,
            },
            head: None,
        },
        round,
        members: members.into_boxed(),
        head,
        written,
    })
}

pub fn history_job_outcome_to_record(
    value: &d::JobOutcome,
    _context: &super::Context,
) -> Result<w::HistoryJobOutcome, w::Problem> {
    Ok(encode_job_outcome(*value))
}

pub fn history_job_outcome_to_domain(
    value: w::HistoryJobOutcome,
    _context: &super::Context,
) -> Result<d::JobOutcome, w::Problem> {
    Ok(decode_job_outcome(value))
}

pub fn history_job_record_to_record(
    value: &d::JobRecord,
    _context: &super::Context,
) -> Result<w::HistoryJobRecord, w::Problem> {
    Ok(encode_job_record(Some(value)).expect("present job"))
}

pub fn history_job_record_to_domain(
    value: w::HistoryJobRecord,
    _context: &super::Context,
) -> Result<d::JobRecord, w::Problem> {
    Ok(decode_job_record(Some(value)).expect("present job"))
}
