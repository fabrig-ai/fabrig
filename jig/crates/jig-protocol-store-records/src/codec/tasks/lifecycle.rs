//! Attempt and hold rows (domain/tasks/lifecycle.md, 12.1 and 12.2).
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use jig_domain_tasks as domain;

pub(super) fn suspension(source: Option<domain::Suspension>) -> Option<wire::Suspension> {
    match source {
        Some(value) => Some(mapped::suspension_to_record(&value)),
        None => None,
    }
}

pub(super) fn read_suspension(source: Option<wire::Suspension>) -> Option<domain::Suspension> {
    match source {
        Some(value) => Some(mapped::suspension_to_domain(value)),
        None => None,
    }
}

pub(super) fn open(row: Option<&domain::OpenAttempt>) -> Result<Option<wire::OpenAttempt>, wire::Problem> {
    match row {
        Some(value) => Ok(Some(mapped::open_attempt_to_record(value)?)),
        None => Ok(None),
    }
}
pub(super) fn read_open(row: Option<wire::OpenAttempt>) -> Result<Option<domain::OpenAttempt>, wire::Problem> {
    match row {
        Some(value) => Ok(Some(mapped::open_attempt_to_domain(value)?)),
        None => Ok(None),
    }
}
