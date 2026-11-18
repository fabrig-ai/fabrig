//! A charter's brief room, by its host part's costs
//! (temper/docs/design/domain/agent/room.md, 4).

pub use temper_engine_domain_room::{BRIEF_ROOM, Cost, HostPart, Measure, Refusal, Short, Unfit};

/// Trials fit their accounts, the family wire and one whole delivery (domain/trial.md, 3.1 and 6).
pub(crate) fn trial_rows(limits: &crate::Limits) -> u32 {
    let row = u32::try_from(size_of::<crate::TrialRow>()).expect("trial row size fits u32");
    let room = limits
        .journal
        .result_bytes
        .max(limits.journal.transcript_bytes)
        .checked_div(row)
        .expect("a trial row has fields");
    limits.accounts.accounts.min(64).min(room)
}

/// Approvals fit the wire and one whole delivery at the largest row (domain/forge/change/questions.md, 2).
pub(crate) fn approval_rows(limits: &crate::Limits) -> u32 {
    let row = u32::try_from(size_of::<crate::ApprovalRow>())
        .expect("approval row size fits u32")
        .saturating_add(limits.forge.name_bytes.saturating_mul(2))
        .saturating_add(u32::try_from(temper_engine_domain_forge::LOGGED_TITLE_BYTES).expect("bounded title"))
        .saturating_add(
            u32::try_from(size_of::<crate::ApprovalSet>())
                .expect("approval set size fits u32")
                .saturating_add(40)
                .saturating_mul(6),
        )
        .saturating_add(
            u32::try_from(size_of::<crate::ApprovalPerson>())
                .expect("approval person size fits u32")
                .saturating_add(64)
                .saturating_mul(16),
        );
    limits
        .journal
        .result_bytes
        .max(limits.journal.transcript_bytes)
        .checked_div(row)
        .expect("an approval row has fields")
        .min(32)
}

#[cfg(test)]
#[path = "tests/room.rs"]
mod tests;
