//! A run replaces only its own pending settings proposal (domain/tasks.md, section 9).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "step tests inspect selected boundary records in ordinary Rust"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Change, Closing, Ending, Event, Key, Party, Phase, Proposal, ProposalAction, ProposalHolder, ProposalKind,
    ProposalState, Refusal, Request, Stage, Stored, domain,
};
use skein_lib::Wall;

fn settings() -> ProposalAction {
    ProposalAction::Settings { against: 1, items: Box::default(), projects: Box::default() }
}
fn proposal(proposer: u64, number: u64, action: ProposalAction) -> Proposal {
    let is_settings = match action {
        ProposalAction::Settings { .. } => true,
        _ => false,
    };
    Proposal {
        proposer,
        number,
        project: u32::from(!is_settings),
        action,
        reason: b"review".as_slice().into(),
        as_holder: false,
        state: ProposalState::Pending {
            holder: if is_settings {
                ProposalHolder::Policy { project: 0, kind: ProposalKind::Settings }
            } else {
                ProposalHolder::Person(9)
            },
            since: Wall::EPOCH,
        },
    }
}
fn pending() -> Harness {
    let mut h = Harness::new();
    h.make(vec![task(1, &[])]);
    h.send(Event::Propose { reply_to: to(), proposal: proposal(1, 100, settings()) });
    assert!(h.domain.proposal(1, 100).is_some());
    h
}
fn actions() -> [ProposalAction; 8] {
    [
        settings(),
        ProposalAction::Batch(Box::new([task(3, &[])])),
        ProposalAction::Effect {
            connector: 1,
            authority: task(3, &[]).authority,
            attempt: 1,
            completion: 1,
            position: 1,
            purpose: 1,
        },
        ProposalAction::ProcedureEffect { connector: 1, authority: task(3, &[]).authority, step: 1, purpose: 1 },
        ProposalAction::Amend { task: 1, revision: 0, amendment: super::revisions::amendment() },
        ProposalAction::Widen { task: 1, authority: task(1, &[]).authority },
        ProposalAction::Release { task: 1 },
        ProposalAction::ReleasePlaced { task: 1, hold: 1, revision: 1 },
    ]
}
#[test]
fn only_settings_over_settings_replaces_a_pending_proposal() {
    for prior in actions() {
        for fresh in actions() {
            let admitted = match (&prior, &fresh) {
                (ProposalAction::Settings { .. }, ProposalAction::Settings { .. }) => true,
                _ => false,
            };
            let mut h = Harness::new();
            h.make(vec![task(1, &[])]);
            let old = proposal(1, 100, prior.clone());
            domain::task_mut(&mut h.domain, 1).expect("live").record.proposal = Some(Box::new(old.clone()));
            let offered = proposal(1, 101, fresh.clone());
            let out = h.send(Event::Propose { reply_to: to(), proposal: offered.clone() });
            if admitted {
                assert_eq!(h.domain.proposal(1, 101), Some(offered));
                let Stored::History(history) = &h.stored[&Key::History { task: 1, revision: 1 }] else {
                    panic!("withdrawal")
                };
                assert_eq!(history.change, Change::ProposalWithdrawn);
                assert_eq!(history.by, Party::Task(1));
                assert_eq!(history.proposal.as_ref().expect("withdrawn").state, ProposalState::Withdrawn);
            } else {
                assert!(
                    out.iter().any(|r| match r {
                        Request::Refused { problem, .. } => problem.why == Refusal::Busy,
                        _ => false,
                    }),
                    "{prior:?} over {fresh:?}: {out:?}"
                );
                assert_eq!(h.domain.proposal(1, 100), Some(old));
            }
        }
    }
}
#[test]
fn a_replacement_needs_room_for_two_task_revisions() {
    let mut h = Harness::new();
    h.make(vec![task(1, &[])]);
    domain::task_mut(&mut h.domain, 1).expect("live").record.revision = u64::MAX - 1;
    h.send(Event::Propose { reply_to: to(), proposal: proposal(1, 100, settings()) });
    assert!(h.domain.proposal(1, 100).is_some(), "a first proposal needs only one revision");
    assert_eq!(h.domain.task(1).expect("live").revision, u64::MAX);
    for revision in [u64::MAX - 2, u64::MAX - 1, u64::MAX] {
        let mut h = pending();
        domain::task_mut(&mut h.domain, 1).expect("live").record.revision = revision;
        let before = h.stored.clone();
        let out = h.send(Event::Propose { reply_to: to(), proposal: proposal(1, 101, settings()) });
        if revision == u64::MAX - 2 {
            assert!(h.domain.proposal(1, 101).is_some());
            assert_eq!(h.domain.task(1).expect("live").revision, u64::MAX);
        } else {
            assert!(
                out.iter().any(|r| match r {
                    Request::Refused { .. } => true,
                    _ => false,
                }),
                "{out:?}"
            );
            assert!(h.domain.proposal(1, 100).is_some());
            assert_eq!(h.stored, before);
        }
    }
}
#[test]
fn a_closing_task_cannot_replace_its_pending_settings() {
    let mut h = pending();
    domain::task_mut(&mut h.domain, 1).expect("live").record.phase = Phase::Closing(Closing {
        stage: Stage::Effects,
        ending: Ending::Failed { reason: b"closed".as_slice().into() },
    });
    let before = h.domain.task(1).expect("live").clone();
    let out = h.send(Event::Propose { reply_to: to(), proposal: proposal(1, 101, settings()) });
    assert!(
        out.iter().any(|r| match r {
            Request::Refused { problem, .. } => problem.why == Refusal::State,
            _ => false,
        }),
        "{out:?}"
    );
    assert_eq!(h.domain.task(1), Some(&before));
}
#[test]
fn replacing_one_tasks_settings_leaves_another_tasks_pending() {
    let mut h = pending();
    h.make(vec![task(2, &[])]);
    let other = proposal(2, 102, settings());
    h.send(Event::Propose { reply_to: to(), proposal: other.clone() });
    let before = h.domain.task(2).expect("other task").clone();
    let party = crate::PersonProposal {
        number: 103,
        proposer: 9,
        project: 0,
        action: crate::PersonAction::Settings { against: 1, items: Box::default() },
        reason: Box::default(),
        state: crate::PersonProposalState::Pending { since: Wall::EPOCH },
    };
    h.send(Event::ProposePerson { reply_to: to(), proposal: party.clone() });
    assert_eq!(h.domain.person_proposal(9, 103), Some(party.clone()));
    h.send(Event::Propose { reply_to: to(), proposal: proposal(1, 101, settings()) });
    assert!(h.domain.proposal(1, 100).is_none());
    assert!(h.domain.proposal(1, 101).is_some());
    assert_eq!(h.domain.proposal(2, 102), Some(other));
    assert_eq!(h.domain.task(2), Some(&before));
    assert_eq!(h.domain.person_proposal(9, 103), Some(party));
}
#[test]
fn replacement_reuses_the_pending_settings_inbox_reservation() {
    let mut h = pending();
    h.env.limits.inbox_messages = 1;
    h.env.limits.inbox_bytes = h.env.limits.message_bytes;
    let before = h.domain.task(1).expect("live").inbox.clone();
    let out = h.send(Event::Propose { reply_to: to(), proposal: proposal(1, 101, settings()) });
    assert!(
        out.iter().any(|r| match r {
            Request::Done { .. } => true,
            _ => false,
        }),
        "{out:?}"
    );
    assert!(h.domain.proposal(1, 101).is_some());
    assert_eq!(h.domain.task(1).expect("live").inbox, before);
}
