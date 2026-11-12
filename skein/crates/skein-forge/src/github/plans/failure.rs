//! Read refusals, transport failures and table invalidation (forge/github.md, 7).
use super::tables::Tables;
use crate::api::Excess;
pub use crate::plan::transport;
use crate::{
    api,
    github::{
        decode::Malformed,
        document::Refusal,
        failure::{self, Asked, Drop, Verdict},
        header::Head,
    },
};

/// Classifies a read's status, applying the dialect's repository invalidation.
pub fn read(
    asked: Asked,
    status: u16,
    head: &Head,
    refusal: Option<&Refusal>,
    repository: api::Repository,
    tables: &mut Tables,
) -> api::Error {
    match failure::classify(asked, status, head, refusal) {
        Verdict::Fail(error, invalidation) => {
            match invalidation {
                Drop::Nothing => {}
                Drop::Row => tables.drop_repository(repository),
            }
            error
        }
        Verdict::Enqueue | Verdict::BranchExists => api::Error::InvalidAnswer,
    }
}
#[must_use]
pub fn malformed(malformed: Malformed) -> api::Error {
    match malformed {
        Malformed::DocumentTooLong { limit, size } => {
            api::Error::TooLarge(Excess::at_least(api::Bound::Forge, u64::from(limit), size))
        }
        Malformed::Bound { .. }
        | Malformed::TooLong { .. }
        | Malformed::Json(skein_json::tokenizer::Error::TooLong) => api::Error::TooLarge(Excess::unnamed()),
        Malformed::Json(_)
        | Malformed::Shape { .. }
        | Malformed::Missing { .. }
        | Malformed::Range { .. }
        | Malformed::Id { .. }
        | Malformed::Time { .. }
        | Malformed::Value { .. } => api::Error::InvalidAnswer,
    }
}
