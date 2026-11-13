//! A change's own workflow gate (temper/docs/design/domain/forge/follows.md, 6–7).
use crate::FollowCheck;
use skein_forge::api;

pub(crate) fn stored_bytes(check: &FollowCheck) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(check.definition.len());
    if let Some(words) = &check.held {
        bytes = bytes.checked_add(skein_lib::len_u64(words.len()))?;
    }
    if let Some(flight) = &check.flight {
        bytes = bytes.checked_add(
            skein_lib::len_u64(flight.pending.len()).checked_mul(u64::try_from(size_of::<api::JobAttempt>()).ok()?)?,
        )?;
        if let Some(report) = &flight.report {
            bytes = bytes.checked_add(skein_lib::len_u64(report.url.len()))?;
        }
    }
    Some(bytes)
}
