//! Bounded compound task operations in the people v3 family.
use super::{Box, List, domain, wire};
pub(super) fn encode(
    values: &List<domain::Operation>,
    limits: &wire::Limits,
) -> Result<List<wire::Operation>, wire::Problem> {
    let mut result = List::with_capacity(values.len());
    for value in values {
        let translated = match value {
            domain::Operation::Cancel { task, reason, grace } => wire::Operation::Cancel(wire::OperationCancel {
                task: *task,
                reason: reason.clone(),
                grace: encode_grace(*grace),
            }),
            domain::Operation::Move { task, reason } => {
                wire::Operation::MoveTask(wire::OperationMove { task: *task, reason: reason.clone() })
            }
            domain::Operation::Amend { task, revision, amendment, release } => {
                wire::Operation::Amend(Box::new(wire::OperationAmend {
                    task: *task,
                    revision: *revision,
                    amendment: super::encode_amendment(amendment, limits)?,
                    release: *release,
                }))
            }
            domain::Operation::Release { task, revision, again } => {
                wire::Operation::Release(wire::OperationRelease { task: *task, revision: *revision, again: *again })
            }
            domain::Operation::Message { task, words } => {
                wire::Operation::Message(wire::OperationMessage { task: *task, words: words.clone() })
            }
            domain::Operation::Stop { task } => wire::Operation::Stop(wire::OperationStop { task: *task }),
        };
        result.push(translated).expect("same operation count");
    }
    Ok(result)
}
pub(super) fn decode(
    values: List<wire::Operation>,
    limits: &wire::Limits,
) -> Result<List<domain::Operation>, wire::Problem> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let translated = match value {
            wire::Operation::Cancel(value) => {
                let wire::OperationCancel { task, reason, grace } = value;
                domain::Operation::Cancel { task, reason, grace: decode_grace(grace) }
            }
            wire::Operation::MoveTask(value) => {
                let wire::OperationMove { task, reason } = value;
                domain::Operation::Move { task, reason }
            }
            wire::Operation::Amend(value) => {
                let wire::OperationAmend { task, revision, amendment, release } = *value;
                domain::Operation::Amend {
                    task,
                    revision,
                    amendment: Box::new(super::decode_amendment(amendment, limits)?),
                    release,
                }
            }
            wire::Operation::Release(value) => {
                let wire::OperationRelease { task, revision, again } = value;
                domain::Operation::Release { task, revision, again }
            }
            wire::Operation::Message(value) => {
                let wire::OperationMessage { task, words } = value;
                domain::Operation::Message { task, words }
            }
            wire::Operation::Stop(value) => {
                let wire::OperationStop { task } = value;
                domain::Operation::Stop { task }
            }
        };
        result.push(translated).expect("same operation count");
    }
    Ok(result)
}

pub(super) fn encode_grace(grace: Option<skein_lib::Duration>) -> Option<u64> {
    Some(grace?.as_nanos())
}
pub(super) fn decode_grace(grace: Option<u64>) -> Option<skein_lib::Duration> {
    Some(skein_lib::Duration::from_nanos(grace?))
}
