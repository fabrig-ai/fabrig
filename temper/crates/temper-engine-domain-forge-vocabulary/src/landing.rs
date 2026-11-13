//! Top-owned landing rows, coherent reads, receipts and effects (domain/landing.md, 2–15).
use crate::{Check, LandingEntry, LandingRow, Limits, Receipt, ReceiptKey};
use core::mem::size_of;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as policy;

pub fn slots(l: &Limits) -> Option<u32> {
    l.landings.checked_mul(l.landing_members)?.checked_mul(l.gates)
}
pub fn result_slots(l: &Limits) -> Option<u32> {
    slots(l)?.checked_mul(4)?.checked_add(l.landings.checked_mul(l.landing_members)?.checked_mul(5)?)
}
pub fn receipt_slots(l: &Limits) -> Option<u32> {
    l.repositories
        .checked_mul(l.gates.checked_add(l.tiers.tiers)?)?
        .checked_add(l.judge_projects.checked_mul(l.follows)?)?
        .checked_mul(l.receipts_per_gate)
}
// Landing row indices and retained table counts must fit before construction.
pub fn valid(l: &Limits) -> bool {
    if l.landing_members > u32::from(u16::MAX)
        || l.gates > u32::from(u16::MAX)
        || l.receipts_per_gate < 2
        || l.trees_in_flight == 0
        || l.key_entries == 0
        || l.name_bytes > 4096
        || l.key_path_bytes == 0
    {
        return false;
    }
    slots(l).is_some() && result_slots(l).is_some() && receipt_slots(l).is_some()
}
pub(crate) fn row_bytes(row: &LandingRow) -> Option<u64> {
    let mut bytes = crate::history::job_bytes(row.job.as_ref())?
        .checked_add(
            skein_lib::len_u64(row.landing.members.len())
                .checked_mul(u64::try_from(size_of::<policy::Member>()).ok()?)?,
        )?
        .checked_add(
            skein_lib::len_u64(row.asks.len()).checked_mul(u64::try_from(size_of::<Option<change::Ask>>()).ok()?)?,
        )?
        .checked_add(
            skein_lib::len_u64(row.entries.len()).checked_mul(u64::try_from(size_of::<LandingEntry>()).ok()?)?,
        )?;
    for ask in &row.asks {
        if let Some(change::Ask::Judge(target)) = ask {
            bytes = bytes
                .checked_add(skein_lib::len_u64(target.branch.len()))?
                .checked_add(skein_lib::len_u64(target.base.len()))?;
        }
    }
    for member in &row.landing.members {
        bytes = bytes.checked_add(skein_lib::len_u64(member.branch.len()))?.checked_add(
            skein_lib::len_u64(member.gates.len()).checked_mul(u64::try_from(size_of::<policy::GateProof>()).ok()?)?,
        )?;
    }
    if let policy::Phase::Held { .. } = row.landing.phase {
        bytes = bytes.checked_add(u64::try_from(size_of::<policy::Phase>()).ok()?)?;
    }
    bytes
        .checked_add(skein_lib::len_u64(row.landing.branch.len()))?
        .checked_add(skein_lib::len_u64(row.landing.round.branch.len()))?
        .checked_add(skein_lib::len_u64(row.landing.round.failure.len()))
}
pub(crate) fn receipt_bytes(key: &ReceiptKey, receipt: &Receipt) -> Option<u64> {
    let name = match &key.check {
        Check::Gate(name) | Check::Tier(name) => name,
    };
    let mut held = skein_lib::len_u64(name.len())
        .checked_add(skein_lib::len_u64(receipt.run.url.len()))?
        .checked_add(skein_lib::len_u64(receipt.logs.len()))?;
    if let Some(failures) = &receipt.failures {
        held = held.checked_add(crate::tiers::failures_bytes(failures)?)?;
    }
    Some(held)
}
