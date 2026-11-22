//! Root backlog and document bounds (temper/docs/design/domain/root.md, 9).
use crate::domain::Domain;
pub use temper_engine_domain_limits::*;

/// What the root holds of admitted work, in entries (the draft's 4.1):
/// the journal's backlog, the work queue, openings and queued store answers.
pub(crate) fn backlog(domain: &Domain) -> u64 {
    domain
        .journal
        .backlog()
        .checked_add(u64::from(domain.work.len()))
        .expect("four u32 queue counts fit in u64")
        .checked_add(u64::from(domain.openings.len()))
        .expect("queued openings fit u64")
        .checked_add(u64::from(domain.store_answers.len()))
        .expect("five u32 queue counts fit in u64")
}

/// The whole trials document fits the family's row bound (domain/trial.md, 3.1 and 6).
pub(crate) fn trial_limits(limits: &Limits) -> temper_engine_domain_trials::Limits {
    temper_engine_domain_trials::Limits { rows: crate::room::trial_rows(limits) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_read_room_reserves_the_journal_carrier() {
        let mut limits = crate::tests::resume_fixture::limits();
        let carrier =
            u32::try_from(jig_domain_core::reads::run_task::answer_carrier_bound(&core_limits(&limits)).unwrap())
                .unwrap();
        limits.journal.transcript_bytes = carrier + 8;
        assert_eq!(task_read_room(&limits, Some(2048)), 8);
        limits.journal.transcript_bytes = carrier - 1;
        assert_eq!(task_read_room(&limits, Some(2048)), 0, "metadata cannot underflow the door's room");
    }
}
