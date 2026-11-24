//! The forge's milestone row (temper/docs/design/domain/milestones.md, 8).
use super::{d, w};

pub(super) fn encode(row: &d::MilestoneRow) -> w::MilestoneRow {
    crate::generated::store::milestone_row_to_record(row)
}

pub(super) fn decode(row: w::MilestoneRow) -> d::MilestoneRow {
    crate::generated::store::milestone_row_to_domain(row)
}
