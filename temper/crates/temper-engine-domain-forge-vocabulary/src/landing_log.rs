//! Retained landing titles (temper/docs/design/domain/after-landing.md, 10).
use crate::{LandingLog, LoggedChange, LoggedLanding};
use core::mem::size_of;

/// A logged title's UTF-8 byte bound (temper/docs/design/domain/after-landing.md, 10).
pub const LOGGED_TITLE_BYTES: usize = 256;
pub(crate) fn row_bytes(row: &LandingLog) -> Option<u64> {
    let mut held =
        skein_lib::len_u64(row.landings.len()).checked_mul(u64::try_from(size_of::<LoggedLanding>()).ok()?)?;
    for landing in &row.landings {
        held = held.checked_add(skein_lib::len_u64(landing.branch.len()))?;
        held = held.checked_add(
            skein_lib::len_u64(landing.changes.len()).checked_mul(u64::try_from(size_of::<LoggedChange>()).ok()?)?,
        )?;
        for change in &landing.changes {
            held = held.checked_add(skein_lib::len_u64(change.title.len()))?;
            held = held.checked_add(
                skein_lib::len_u64(change.signed.len())
                    .checked_mul(u64::try_from(size_of::<crate::SignOff>()).ok()?)?,
            )?;
        }
    }
    Some(held)
}
