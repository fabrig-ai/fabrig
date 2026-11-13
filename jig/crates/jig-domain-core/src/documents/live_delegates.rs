//! Complete close-disposal identities and their owner funding
//! (`jig/docs/design/client/domain/objects.md`, 11).

use crate::{Core, Limits, documents::Missing};
pub use jig_domain_core_vocabulary::documents::live_delegates::*;
use jig_domain_tasks as tasks;
use skein_lib::List;

pub(super) fn from_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
) -> Result<List<LiveDelegate>, Missing> {
    if row.delegates.len() > skein_lib::length::usize_of(limits.tasks.delegates) {
        return Err(Missing::TooLarge);
    }
    let mut delegates = List::with_capacity(limits.tasks.delegates);
    for number in &row.delegates {
        let delegate = super::small::visible(core, party, *number)?;
        if delegate.requester != tasks::Party::Task(row.number) {
            return Err(Missing::Unknown);
        }
        delegates
            .push(LiveDelegate {
                chip: super::small::chip_with_core(core, limits, party, delegate)?,
                takeover_budget: core.tasks.takeover_budget(*number).ok_or(Missing::Unknown)?,
            })
            .expect("complete delegates fit checked bound");
    }
    Ok(delegates)
}
