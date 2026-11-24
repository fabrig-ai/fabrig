//! Volatile party inbox projections from the task child's durable rows.
//! The core routes these sibling facts on save and restore (domain/engine/vocabulary.md, 1.4).

use crate::Limits;

/// Inbox words, role-routed proposals and the five scalar references: a
/// person-held proposal, escalation, person task address and offer, and stall update.
pub fn waiting_capacity(limits: &Limits) -> Option<u32> {
    u32::try_from(waiting_requirement(limits)).ok()
}

/// The same capacity in a wider count, including requirements no u32 can hold.
pub fn waiting_requirement(limits: &Limits) -> u64 {
    let words = u64::from(limits.tasks.inbox_messages).checked_mul(2).expect("twice u32 fits u64");
    words
        .checked_add(u64::from(limits.authority.roles))
        .expect("three u32 counts fit u64")
        .checked_add(5)
        .expect("three u32 counts and five scalar references fit u64")
}

pub fn question_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.tasks.checked_mul(limits.tasks.inbox_messages)
}
