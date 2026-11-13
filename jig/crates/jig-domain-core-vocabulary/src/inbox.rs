//! Volatile party inbox projections from the task child's durable rows.
//! The core routes these sibling facts on save and restore (domain/engine/README.md, 4.4).

use crate::Limits;

/// Inbox words, role-routed proposals and the four scalar references: a
/// person-held proposal, escalation, answerable person task and stall update.
pub fn waiting_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.inbox_messages.checked_mul(2)?.checked_add(limits.authority.roles)?.checked_add(4)
}

pub fn question_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.tasks.checked_mul(limits.tasks.inbox_messages)
}
