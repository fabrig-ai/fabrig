#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary native proofs query original keyed requests and select the three offered decisions"
)]
use crate::state::objects;
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, ReadPage, Watched};
use crate::tests::state::domain::{
    objects::{live, read_token, text, watched},
    spine::drive,
};
use crate::{
    Event, Request,
    state::{
        action::Act,
        fields::Purpose,
        objects::{Body, Card, Source},
    },
};
use jig_client::contract::*;
use skein_lib::{List, Time, Wall};
fn value(maker: Proposer, offered: ActKind) -> OwnerProposal {
    let acts = crate::tests::kit::list_in(2, [Offered { effect: None, act: offered, admission: Admission::Allowed }]);
    OwnerProposal {
        key: ProposalKey { proposer: maker, number: u64::MAX },
        project: u32::MAX,
        reason: DocumentText { bytes: text(UTF8_TEXTS[1]), cut: true },
        action: OwnerProposalAction::Batch(List::with_capacity(2)),
        as_holder: false,
        waiting: OwnerProposalWaiting {
            holder: OwnerHistoryHolder::Person(u64::MAX),
            since: Wall::from_nanos(u64::MAX),
        },
        funding: List::with_capacity(2),
        acts,
    }
}
#[test]
fn owner_proposal_original_maker_keys_all_three_decisions_and_reason_survive_double_confirmation_and_late_answers() {
    for act in [ActKind::Accept, ActKind::Reject, ActKind::PassUp] {
        let (mut domain, _) = live();
        let mut pending = List::with_capacity(2);
        for maker in [Proposer::Task(u64::MAX), Proposer::Party(u64::MAX)] {
            let supplied = value(maker, act);
            let key = ObjectKey::Proposal(supplied.key);
            let id = domain.objects.name(key, Source::Window { number: 4 }).unwrap();
            assert!(domain.objects.put(key, Body::OwnerProposal(Box::new(supplied.clone()))));
            let object = domain.objects().get(id).unwrap();
            assert_eq!(object.project, Some(u32::MAX));
            assert_eq!(objects::proposal::owner_facts(&object), Some(&supplied));
            assert_eq!(object.body.as_ref().unwrap().ending(), None);
            drive(&mut domain, Time::ZERO, Event::Act { action: Act::Intend { object: id.token(), act } });
            assert_eq!(domain.confirmation().unwrap().revision, None);
            if act == ActKind::Reject {
                drive(
                    &mut domain,
                    Time::ZERO,
                    Event::Act { action: Act::Edit { field: Purpose::Reason, text: text(UTF8_TEXTS[0]) } },
                );
            }
            let sent = drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm });
            assert_eq!(sent.len(), 1);
            let Request::Client {
                asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), key: request, .. },
            } = &sent[0]
            else {
                panic!("one original keyed request")
            };
            let decision = match act {
                ActKind::Accept => Decision::Accepted,
                ActKind::Reject => Decision::Rejected { reason: text(UTF8_TEXTS[0]) },
                ActKind::PassUp => Decision::PassedUp,
                _ => unreachable!(),
            };
            assert_eq!(
                ask.as_ref(),
                &Ask::Decide {
                    project: u32::MAX,
                    on: DecideOn::Proposal { proposal: supplied.key, decision, running: None }
                }
            );
            assert!(drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).is_empty());
            pending.push((id, *request)).unwrap();
        }
        assert_ne!(pending.get(0).unwrap().0, pending.get(1).unwrap().0);
        assert_ne!(pending.get(0).unwrap().1, pending.get(1).unwrap().1);
        for (id, request) in pending.into_boxed() {
            let ObjectKey::Proposal(proposal) = domain.objects().get(id).unwrap().key else { panic!("proposal") };
            let answer = Answer::Done(Outcome::Owner(OwnerOutcome::ProposalDecided {
                proposal,
                by: 7,
                choice: OwnerProposalChoice::Accepted,
                at: Wall::EPOCH,
                repeated: false,
            }));
            drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer: answer.clone() });
            assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer.clone()));
            assert!(match domain.objects().get(id).unwrap().card {
                Card::Leaving { .. } => true,
                Card::NotSent { .. }
                | Card::Open
                | Card::Refused { .. }
                | Card::Unknown { .. }
                | Card::Deciding { .. } => false,
            });
            drive(&mut domain, Time::ZERO, AnsweredByEngine { request, answer: refusal_answer(Refusal::KeyConflict) });
            assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer.clone()));
        }
    }
}
#[test]
fn owner_proposal_native_read_consumes_once_watch_keeps_same_card_and_wrong_key_or_removed_offer_cannot_send() {
    let (mut domain, watch) = live();
    let original = value(Proposer::Task(u64::MAX), ActKind::Accept);
    let key = ObjectKey::Proposal(original.key);
    let reads = watched(&mut domain, watch, &[key]);
    let read = read_token(&reads);
    let out = drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(original.clone()))) },
    );
    assert!(!out.iter().any(side_or_send));
    let id = domain.objects().find(key).unwrap();
    assert_eq!(objects::proposal::owner_facts(&domain.objects().get(id).unwrap()), Some(&original));
    let mut changed = original.clone();
    changed.as_holder = true;
    changed.reason = DocumentText::complete(text(b"new independent source"));
    drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(changed.clone()))) },
    );
    assert_eq!(domain.objects().find(key), Some(id));
    assert_eq!(objects::proposal::owner_facts(&domain.objects().get(id).unwrap()), Some(&changed));
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read, result: ReadResult::Value(ReadValue::OwnerProposal(Box::new(original))) },
    );
    assert_eq!(
        objects::proposal::owner_facts(&domain.objects().get(id).unwrap()),
        Some(&changed),
        "retired terminal cannot overwrite a newer watch"
    );
    let mut wrong = changed.clone();
    wrong.key.proposer = Proposer::Party(u64::MAX);
    assert!(!domain.objects.put(key, Body::OwnerProposal(Box::new(wrong))));
    assert_eq!(objects::proposal::owner_facts(&domain.objects().get(id).unwrap()), Some(&changed));
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::Intend { object: id.token(), act: ActKind::Accept } });
    changed.acts = List::with_capacity(2);
    assert!(domain.objects.put(key, Body::OwnerProposal(Box::new(changed.clone()))));
    drive(
        &mut domain,
        Time::ZERO,
        Watched { watch, event: WatchEvent::Delivered(Delivery::OwnerProposal(Box::new(changed.clone()))) },
    );
    let out = drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm });
    assert!(!out.iter().any(sent));
    assert_eq!(objects::proposal::owner_facts(&domain.objects().get(id).unwrap()), Some(&changed));
    assert_eq!(
        domain.confirmation().unwrap().missing,
        Some(crate::state::confirm::Missing::Refused { why: Refusal::NotOffered })
    );
    assert_eq!(
        domain.objects().get(id).unwrap().card,
        Card::Deciding { request: None, progress: crate::state::intents::Progress::Confirming }
    );
    for admission in
        [Admission::Refused(Refusal::NotOffered), Admission::Refused(Refusal::Limit { which: Limit::Objects })]
    {
        changed.acts.push(Offered { effect: None, act: ActKind::Reject, admission }).unwrap();
        assert!(domain.objects.put(key, Body::OwnerProposal(Box::new(changed.clone()))));
        drive(&mut domain, Time::ZERO, Event::Act { action: Act::Intend { object: id.token(), act: ActKind::Reject } });
        assert!(!drive(&mut domain, Time::ZERO, Event::Act { action: Act::Confirm }).iter().any(sent));
        changed.acts.clear();
    }
}

#[expect(clippy::match_like_matches_macro, reason = "The native test subset excludes the matches macro.")]
fn sent(request: &Request) -> bool {
    if let Request::Client { asked: jig_client::Event::Send { .. } } = request { true } else { false }
}
fn side_or_send(request: &Request) -> bool {
    if let Request::ShowPart { .. } = request { true } else { sent(request) }
}

const UTF8_TEXTS: [&[u8]; 2] = [b"!\0!", b"~\0?"];

use jig_client::contract::DecideOn;
