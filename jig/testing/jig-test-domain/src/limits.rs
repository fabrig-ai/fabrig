//! The test root's edge validity (service.md, 4).
use crate::Limits;
use jig_domain_core as core;
use jig_test_connector as connector;

/// Whether the root's child limits and output edges are representable.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    checked_valid(limits).is_some()
}
fn checked_valid(limits: &Limits) -> Option<()> {
    if limits.routes == 0 || !core::valid(&limits.core) || !connector::valid(&limits.connector) {
        return None;
    }
    core::reads::answer_bound(&limits.core)?;
    connector::document_bound(&limits.connector)?;
    Some(())
}
