//! Whole party-specific offers (`client/domain/contract/documents.md`, 3;
//! `client/domain/objects.md`, 11). The acts owner supplies every judgment.

use super::{DocumentLimits, Missing};
use crate::acts::{self, Admission, Beyond, Bounds, Offered, Recipient};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, length::len_u64};
#[cfg(test)]
mod tests;

pub const MAX_ACTS: u32 = 23;

fn receiving_recipient(value: Recipient, limits: &DocumentLimits) -> Result<Recipient, Missing> {
    match value {
        Recipient::Roles { project, roles } => {
            if roles.len() > limits.lines {
                return Err(Missing::TooLarge);
            }
            let mut received = List::with_capacity(limits.lines);
            for role in &roles {
                received.push(*role).expect("whole policy role list fits");
            }
            Ok(Recipient::Roles { project, roles: received })
        }
        other @ (Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. }) => Ok(other),
    }
}
pub fn receiving(value: Admission, limits: &DocumentLimits) -> Result<Admission, Missing> {
    Ok(match value {
        other @ (Admission::Allowed | Admission::Refused(_)) => other,
        Admission::Proposed { to } => Admission::Proposed { to: receiving_recipient(to, limits)? },
        Admission::Bounded { within, beyond } => {
            let fits = match &within {
                Bounds::Create { charters, procedures, .. } => {
                    charters.len() <= limits.lines && procedures.len() <= limits.lines
                }
                Bounds::Amend { grants, implications, .. } => {
                    grants.len() <= limits.grants && implications.capacity() <= limits.implications
                }
            };
            if !fits {
                return Err(Missing::TooLarge);
            }
            let beyond = match beyond {
                Beyond::Proposed { to } => Beyond::Proposed { to: receiving_recipient(to, limits)? },
                other @ Beyond::Refused(_) => other,
            };
            Admission::Bounded { within, beyond }
        }
    })
}
pub fn heap_bound(limits: &DocumentLimits) -> Option<u64> {
    let authority = super::task_tree::authority_bound(limits)?;
    let grants = List::<super::GrantLine>::price(limits.grants)?
        .checked_add(u64::from(limits.grants).checked_mul(super::task_tree::grant_bound(limits)?)?)?;
    let amended = authority
        .checked_add(u64::try_from(size_of::<super::AuthorityLine>()).ok()?)?
        .checked_mul(2)?
        .checked_add(grants)?
        .checked_add(List::<jig_domain_authority::Implication>::price(limits.implications)?)?;
    let recipient = List::<u32>::price(limits.lines)?;
    // Amend carries the full amendment facts on a task;
    // all other object kinds carry at most one bounded act.
    let goal = List::<u32>::price(1)?
        .checked_add(List::<acts::CreateProcedure>::price(limits.lines)?)?
        .checked_add(u64::try_from(size_of::<super::AuthorityLine>()).ok()?)?
        .checked_add(authority)?;
    List::<Offered>::price(MAX_ACTS)?.checked_add(amended.max(goal).checked_add(recipient)?.checked_mul(2)?)
}

fn path_heap(value: &List<Box<[u8]>>, last: &super::PatternEnd) -> Option<u64> {
    let mut bytes = List::<Box<[u8]>>::price(value.len().checked_add(value.room())?)?;
    for segment in value {
        bytes = bytes.checked_add(len_u64(segment.len()))?;
    }
    let terminal = match last {
        super::PatternEnd::Exact(value) | super::PatternEnd::Open(value) => value.len(),
    };
    bytes.checked_add(u64::try_from(terminal).ok()?)
}
fn grants_heap(value: &List<super::GrantLine>) -> Option<u64> {
    let mut bytes = List::<super::GrantLine>::price(value.len().checked_add(value.room())?)?;
    for grant in value {
        bytes = bytes.checked_add(path_heap(&grant.segments, &grant.last)?)?;
    }
    Some(bytes)
}
fn authority_heap(value: &super::AuthorityLine) -> Option<u64> {
    let mut bytes = grants_heap(&value.grants)?
        .checked_add(List::<tasks::AuthorityExecutor>::price(
            value.executors.len().checked_add(value.executors.room())?,
        )?)?
        .checked_add(List::<super::NoteScope>::price(
            value.note_resources.len().checked_add(value.note_resources.room())?,
        )?)?;
    for scope in &value.note_resources {
        bytes = bytes.checked_add(path_heap(&scope.segments, &scope.last)?)?;
    }
    Some(bytes)
}
fn recipient_heap(value: &Recipient) -> Option<u64> {
    match value {
        Recipient::Roles { roles, .. } => List::<u32>::price(roles.len().checked_add(roles.room())?),
        Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. } => Some(0),
    }
}
fn beyond_heap(value: &Beyond) -> Option<u64> {
    match value {
        Beyond::Proposed { to } => recipient_heap(to),
        Beyond::Refused(_) => Some(0),
    }
}
pub fn heap_bytes(value: &List<Offered>) -> Option<u64> {
    let mut bytes = List::<Offered>::price(value.len().checked_add(value.room())?)?;
    for offer in value {
        bytes = bytes.checked_add(match &offer.admission {
            Admission::Allowed | Admission::Refused(_) => 0,
            Admission::Proposed { to } => recipient_heap(to)?,
            Admission::Bounded { within, beyond } => {
                let inner = match within {
                    Bounds::Create { charters, procedures, authority, .. } => {
                        List::<u32>::price(charters.len().checked_add(charters.room())?)?
                            .checked_add(List::<acts::CreateProcedure>::price(
                                procedures.len().checked_add(procedures.room())?,
                            )?)?
                            .checked_add(u64::try_from(size_of::<super::AuthorityLine>()).ok()?)?
                            .checked_add(authority_heap(authority)?)?
                    }
                    Bounds::Amend { grants, implications, authority, ceiling, .. } => grants_heap(grants)?
                        .checked_add(List::<jig_domain_authority::Implication>::price(
                            implications.len().checked_add(implications.room())?,
                        )?)?
                        .checked_add(u64::try_from(size_of::<super::AuthorityLine>()).ok()?.checked_mul(2)?)?
                        .checked_add(authority_heap(authority)?)?
                        .checked_add(authority_heap(ceiling)?)?,
                };
                inner.checked_add(beyond_heap(beyond)?)?
            }
        })?;
    }
    Some(bytes)
}
