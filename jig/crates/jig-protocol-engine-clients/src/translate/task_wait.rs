//! Task wait facts are already joined and translated to wall time by the core.
use crate::generated::client as mapped;
use jig_client_protocol_wire as w;
use jig_domain_core_vocabulary::documents as d;

/// Translate a core-derived next-run wait at the current jig wire version.
#[must_use]
pub fn waiting(value: Option<d::TaskRunWait>) -> Option<w::TaskRunWait> {
    Some(run(value?))
}

pub(super) fn run(value: d::TaskRunWait) -> w::TaskRunWait {
    mapped::task_run_wait_to_record(&value)
}
