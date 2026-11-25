//! Original history revisions, snapshots and lifecycle facts (protocol/client/wire.md, 1).
use super::w;
use crate::generated::client as mapped;
use jig_domain_core_vocabulary::documents as d;
pub fn line(models: &[crate::Model], value: &d::HistoryLine, limits: &w::Limits) -> Result<w::HistoryLine, w::Problem> {
    mapped::history_line_to_record(value, &(models, limits))
}
