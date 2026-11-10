//! Admission bounds for a hosted run (domain/hosts/hub.md, section 2).
//! Workspace items belong to the application and stay opaque here.

use crate::boundary::{Invalid, RunAssignment};
use crate::limits::Limits;
use skein_lib::length::len_u64;

/// Whether `assignment` fits `limits`, and what about it does not.
pub(crate) fn check(assignment: &RunAssignment, limits: &Limits) -> Result<(), Invalid> {
    if len(&assignment.charter) > limits.charter_bytes {
        return Err(Invalid::Charter);
    }
    if len_u64(assignment.grants.len()) > u64::from(limits.accounts) {
        return Err(Invalid::Grants);
    }
    for (index, grant) in assignment.grants.iter().enumerate() {
        for other in assignment.grants.iter().skip(index.saturating_add(1)) {
            if grant.form.table() == other.form.table() && grant.account == other.account {
                return Err(Invalid::Grants);
            }
        }
    }
    Ok(())
}

/// The length of `bytes`.
pub(crate) fn len(bytes: &[u8]) -> u64 {
    len_u64(bytes.len())
}

/// A start carries at most one inbox of individually bounded messages.
pub(crate) fn messages_fit(messages: &[crate::StartMessage], limits: &Limits) -> bool {
    if len_u64(messages.len()) > u64::from(limits.inbox) {
        return false;
    }
    for message in messages {
        match len(&message.sender).checked_add(len(&message.words)) {
            Some(bytes) if bytes <= limits.event_bytes => {}
            Some(_) | None => return false,
        }
    }
    true
}
