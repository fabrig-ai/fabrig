//! Whole-batch byte admission and restoration (domain/tasks/README.md, section 4, and domain/tasks/lifecycle.md, section 7, and domain/tasks/README.md, section 6).
use super::revisions::{Harness, LIMITS, task};
use crate::{
    Contract, Domain, Event, InvalidResult, Limits, New, PersonAction, PersonProposal, PersonProposalState, Proposal,
    ProposalAction, ProposalHolder, ProposalState, RecurringOverlap, RecurringTemplate, RestoreBound, RestoreCause,
    RestoreFailure, ResultFollowups, Stored, TaskResult, Verdict,
};
use core::mem::size_of;

fn bytes(batch: &[New]) -> u32 {
    u32::try_from(crate::batch_bytes(batch).unwrap()).unwrap()
}

#[test]
fn a_batch_counts_its_members_templates() {
    let mut member = task(1, &[]);
    let plain = crate::batch_bytes(core::slice::from_ref(&member)).unwrap();
    let mut template_batch: Box<[New]> = Box::new([task(1, &[]), task(2, &[1])]);
    let chosen = crate::ModelEffort { model: 0, effort: None };
    template_batch[0].llm =
        Some(crate::LlmChoice { later: None, account: None, main: chosen, subagents: Box::new([chosen]) });
    let template_bytes = crate::batch_bytes(&template_batch).unwrap();
    member.recurring =
        Some(Box::new(RecurringTemplate { key: 1, batch: template_batch, overlap: RecurringOverlap::Skip }));
    assert_eq!(
        crate::batch_bytes(&[member]),
        Some(plain + u64::try_from(size_of::<RecurringTemplate>()).unwrap() + template_bytes)
    );
    assert_eq!(crate::batch_bytes(&[]), Some(0));
    assert_eq!(plain, u64::try_from(size_of::<New>()).unwrap() + 1);
}

#[test]
fn a_batch_past_its_bytes_is_not_bounded() {
    let batch = [task(1, &[]), task(2, &[1])];
    let held = bytes(&batch);
    for (limit, accepted) in [(held, true), (held - 1, false)] {
        let limits = Limits { batch_bytes: limit, ..LIMITS };
        assert_eq!(crate::bounded_batch(&limits, &batch), accepted);
        assert_eq!(crate::recurring::valid_template(&limits, 1, 200, &batch), accepted);
    }
}

#[test]
fn followups_past_the_batch_bytes_are_invalid() {
    let mut batch: Box<[New]> = Box::new([task(1, &[]), task(2, &[1])]);
    let chosen = crate::ModelEffort { model: 0, effort: None };
    batch[0].llm = Some(crate::LlmChoice { later: None, account: None, main: chosen, subagents: Box::new([chosen]) });
    let held = bytes(&batch);
    let contract =
        Contract::Verdict { choices: Box::new([Verdict { code: 7, fields: Box::new([]), words: 32, followups: 2 }]) };
    let result = TaskResult::Verdict { code: 7, fields: Box::new([]), words: Box::new([]) };
    let proposal = Proposal {
        number: 100,
        proposer: 9,
        project: 1,
        action: ProposalAction::Batch(batch.clone()),
        reason: Box::new([]),
        as_holder: false,
        state: ProposalState::Pending { holder: ProposalHolder::Person(1), since: skein_lib::Wall::EPOCH },
    };
    for followups in [ResultFollowups::Delegates(batch), ResultFollowups::Proposal(Box::new(proposal))] {
        let framing = match &followups {
            ResultFollowups::Delegates(_) => 0,
            ResultFollowups::Proposal(_) => u64::try_from(size_of::<Proposal>()).unwrap(),
        };
        assert_eq!(
            crate::terminal_bytes(&crate::End::FinishedWithFollowups {
                result: result.clone(),
                followups: followups.clone()
            }),
            Some(u64::from(held) + framing)
        );
        for (limit, expected) in [(held - 1, Some(InvalidResult::Followups)), (held, None)] {
            assert_eq!(
                crate::run::invalid_followups(&contract, &result, &followups, &Limits { batch_bytes: limit, ..LIMITS }),
                expected
            );
        }
    }
}

#[test]
fn a_restored_batch_past_its_bytes_is_the_batch_bound() {
    let batch: Box<[New]> = Box::new([task(1, &[]), task(2, &[1])]);
    let held = bytes(&batch);
    for (limit, accepted) in [(held, true), (held - 1, false)] {
        let mut h = Harness::new();
        h.env.limits.batch_bytes = limit;
        h.domain = Domain::new(&h.env.limits, crate::tests::KEPT, 91, 1, super::charters(), Box::new([0]));
        let row = PersonProposal {
            number: 100,
            proposer: 9,
            project: 1,
            action: PersonAction::Batch(batch.clone()),
            reason: Box::new([]),
            state: PersonProposalState::Pending { since: h.env.wall },
        };
        let out = h.send(Event::Restore { record: Stored::PersonProposal(Box::new(row)) });
        if accepted {
            assert_eq!(out.len(), 0);
            assert_eq!(h.domain.restore_failure(), None);
        } else {
            assert_eq!(
                h.domain.restore_failure(),
                Some(RestoreFailure {
                    key: crate::Key::PersonProposal(100),
                    cause: RestoreCause::Bound {
                        which: RestoreBound::Batch,
                        held: u64::from(held),
                        limit: u64::from(limit)
                    }
                })
            );
        }
    }
}
