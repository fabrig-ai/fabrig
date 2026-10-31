use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{AnsweredByEngine, Fixture, ReadPage, Reported};
use crate::tests::state::domain::{
    confirmation::task,
    objects::live,
    spine::{LIMITS, drain, drive},
};
use crate::{
    Event, Request,
    state::{
        action::Act,
        intents::{Progress, send_card},
        objects::{Body, Card, Object, Source, answer::recovery},
    },
    tree::notices::kept::Kind,
};
use jig_client::contract::*;
use skein_lib::{Id, Queue, Time, Token, Wall};
pub(crate) mod person_refresh;
fn install(domain: &mut Fixture, body: Body) -> Id<Object> {
    let key = body.key();
    let id = domain.objects.name(key, Source::Window { number: 4 }).unwrap();
    assert!(domain.objects.put(key, body));
    id
}
fn send(domain: &mut Fixture, key: ObjectKey) -> Token {
    let mut out = Queue::uncapped();
    let token = send_card(
        &mut domain.ui,
        key,
        ActKind::Stop,
        Box::new(Ask::Stop { project: 1, task: 8 }),
        &mut out,
        &domain.client,
    )
    .unwrap();
    assert_eq!(drain(out).len(), 1);
    token
}
fn answered(domain: &mut Fixture, request: Token, answer: Answer) -> Box<[Request]> {
    drive(domain, Time::ZERO, AnsweredByEngine { request, answer })
}
fn read_token(rows: &[Request]) -> jig_client::Handle {
    for row in rows {
        if let Request::Client { asked: jig_client::Event::Read { handle: read, .. } } = row {
            return *read;
        }
    }
    unreachable!("refresh read emitted")
}

#[test]
fn refusals_keep_every_engine_fact_and_moved_reads_the_current_object_without_losing_the_refusal() {
    let names = crate::tests::kit::list([7, 5]);
    let refusals = [
        Refusal::Circle { tasks: names.clone(), cut: true },
        Refusal::Kept { pending: names },
        Refusal::Dependency { dependency: 7, how: TaskStatus::Failed },
        Refusal::Role { held: Some(HeldRole::Policy { role: 2 }) },
        Refusal::Role { held: None },
        Refusal::Taken { by: 71 },
        Refusal::NotWaiting,
        Refusal::NoFurther,
        Refusal::Authority { lacking: Lacking::Deadline { asked: None, latest: None }, proposable: None },
        Refusal::Limit { which: Limit::Admission { which: AdmissionLimit::InboxFull } },
        Refusal::Moved {
            now: Revision { value: 12 },
            what: Moved::Hold { reason: Some(CurrentHold::Reason(HoldCategory::Uncertain { entry: 123 })) },
        },
        Refusal::Ended { how: Ending::Decided { by: 71, party: None, how: None, at: Wall::EPOCH } },
        Refusal::Ended {
            how: Ending::Cancelled { by: Actor::ProjectDeployment { project: 1 }, reason: None, at: Some(Wall::EPOCH) },
        },
        Refusal::Ended { how: Ending::Deleted },
        Refusal::Authority { lacking: Lacking::Budget { needed: 40, left: 12 }, proposable: None },
        Refusal::Limit { which: Limit::LiveTasks { project: 1, limit: 2 } },
        Refusal::Moved { now: Revision { value: 12 }, what: Moved::Spec },
        Refusal::Ended { how: Ending::Done },
        Refusal::Unknown,
        Refusal::NotOffered,
        Refusal::KeyConflict,
        Refusal::Llm { model: Text::new(Box::from(*b"m"), 1).unwrap(), why: LlmWhy::Unconfigured },
        Refusal::Account { account: 42, why: AccountWhy::Unknown },
        Refusal::Account { account: 42, why: AccountWhy::Provider },
        Refusal::Account { account: 42, why: AccountWhy::Uses },
        Refusal::Account { account: 42, why: AccountWhy::Limited },
    ];
    for refusal in refusals {
        let (mut domain, _) = live();
        let id = install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        let rows = answered(&mut domain, request, refusal_answer(refusal.clone()));
        let object = domain.objects().get(id).unwrap();
        assert_eq!(object.card, Card::Refused { refusal: refusal.clone() });
        assert_eq!(object.answer, Some(refusal_answer(refusal.clone())));
        match refusal {
            Refusal::Stale(_) | Refusal::Moved { .. } => {
                assert!(recovery(&object).look_again);
                assert!(
                    drive(&mut domain, Time::ZERO, Event::Act { action: Act::LookAgain { object: id.token() } })
                        .is_empty()
                );
                let mut current = task();
                current.spec_revision = Revision { value: 12 };
                drive(
                    &mut domain,
                    Time::ZERO,
                    ReadPage {
                        read: read_token(&rows),
                        result: ReadResult::Value(ReadValue::Task(Box::new(current.clone()))),
                    },
                );
                assert_eq!(domain.objects().get(id).unwrap().body, Some(Body::Task(Box::new(current))));
                assert_eq!(domain.objects().get(id).unwrap().card, Card::Refused { refusal });
            }
            Refusal::Full(_)
            | Refusal::Circle { .. }
            | Refusal::Dependency { .. }
            | Refusal::Kept { .. }
            | Refusal::Authority { .. }
            | Refusal::Limit { .. }
            | Refusal::Llm { .. }
            | Refusal::Account { .. } => {
                assert!(recovery(&object).change_ask);
                assert!(rows.is_empty());
            }
            Refusal::Malformed { .. }
            | Refusal::Taken { .. }
            | Refusal::NotWaiting
            | Refusal::NoFurther
            | Refusal::Role { .. }
            | Refusal::Ended { .. }
            | Refusal::SignInRedirect { .. }
            | Refusal::AccountMove { .. }
            | Refusal::Provider
            | Refusal::Unknown
            | Refusal::NameTaken
            | Refusal::NotOffered
            | Refusal::NoEscalationHolder
            | Refusal::KeyConflict => {
                assert!(!recovery(&object).look_again && !recovery(&object).change_ask);
                assert!(rows.is_empty());
            }
        }
    }
}
#[test]
fn retention_expiry_retires_once_keeps_unknown_distinct_and_inspection_never_resends() {
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::Task(Box::new(task())));
    let key = ObjectKey::Task(8);
    let request = send(&mut domain, key);
    assert!(drive(&mut domain, Time::ZERO, Reported { request, progress: Progress::Unknown }).is_empty());
    assert_eq!(
        domain.objects().get(id).unwrap().card,
        Card::Unknown { act: ActKind::Stop, look: Read::Task { task: 8 } }
    );
    assert!(domain.objects().get(id).unwrap().answer.is_none());
    assert!(
        answered(&mut domain, request, Answer::Done(Outcome::Document(DocumentOutcome::Released { task: 8 })))
            .is_empty()
    );
    let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::LookAgain { object: id.token() } });
    assert_eq!(rows.len(), 1);
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read: read_token(&rows), result: ReadResult::None(ReadNone::Unreachable) },
    );
    assert!(recovery(&domain.objects().get(id).unwrap()).look_again);

    let rows = drive(&mut domain, Time::ZERO, Event::Act { action: Act::LookAgain { object: id.token() } });
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read: read_token(&rows), result: ReadResult::Value(ReadValue::Task(Box::new(task()))) },
    );
    assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
    domain.intents.reclaim();
}
#[test]
fn full_requests_refuse_without_queueing_and_hidden_answers_and_unknowns_have_notices() {
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::Task(Box::new(task())));
    for number in 1..=LIMITS.intents {
        let mut out = Queue::uncapped();
        domain
            .intents
            .ask(
                7,
                crate::state::intents::Origin::Card { object: ObjectKey::Task(u64::from(number)), act: ActKind::Stop },
                Box::new(Ask::Stop { project: 1, task: 8 }),
                &mut out,
            )
            .unwrap();
    }
    let mut out = Queue::uncapped();
    assert!(
        send_card(
            &mut domain.ui,
            ObjectKey::Task(8),
            ActKind::Stop,
            Box::new(Ask::Stop { project: 1, task: 8 }),
            &mut out,
            &domain.client
        )
        .is_none()
    );
    assert_eq!(domain.objects().get(id).unwrap().card, Card::NotSent { why: crate::state::intents::NotSent::Limit });
    assert_eq!(drain(out).as_ref(), &[]);
    for unknown in [false, true] {
        let (mut domain, _) = live();
        install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        drive(&mut domain, Time::ZERO, Event::Went { place: crate::state::address::Place::Elsewhere });
        if unknown {
            drive(&mut domain, Time::ZERO, Reported { request, progress: Progress::Unknown });
        } else {
            answered(&mut domain, request, Answer::Done(Outcome::Document(DocumentOutcome::Released { task: 8 })));
        }
        let (notice, ()) = domain.notices().ids().iter().next().unwrap();
        let expected = if unknown {
            Kind::Unknown { object: ObjectKey::Task(8), act: ActKind::Stop }
        } else {
            Kind::Answer {
                object: ObjectKey::Task(8),
                answer: Answer::Done(Outcome::Document(DocumentOutcome::Released { task: 8 })),
            }
        };
        assert_eq!(domain.notices().get(*notice).unwrap().kind, expected);
        assert!(domain.objects().find(ObjectKey::Task(8)).is_none());
    }
}

#[test]
fn every_owner_outcome_remains_whole_on_its_original_card_after_duplicate_terminals() {
    use DocumentOutcome as O;
    let outcomes = [
        O::MarkedSeen { seen: SeenThrough::Results { through: 103 } },
        O::MarkedSeen { seen: SeenThrough::Conversation { task: 104, through: 105 } },
        O::NoteEdited { name: 106 },
        O::ServiceMade { person: 107 },
        O::Adopted { project: 108 },
        O::Stopped { task: 111 },
        O::Cancelled { task: 112 },
        O::Released { task: 113 },
        O::PersonTaken { task: 114 },
        O::PersonHandedBack { task: 115 },
        O::PersonAnswered { task: 116 },
        O::Moved { task: 117 },
        O::ProposalDecided {
            repeated: false,
            proposal: ProposalKey { proposer: Proposer::Party(118), number: 119 },
            by: 120,
            at: Wall::from_nanos(121),
            choice: ProposalChoice::Stale,
        },
        O::Said { task: 122, message: 123 },
        O::QuestionAnswered { task: 124, question: 125, message: 126 },
        O::Amended { task: 128 },
        O::Proposed { proposal: 130, to: 0 },
        O::RolesSet { project: 131 },
        O::PolicyChanged { project: 132 },
        O::PoolSet { project: 133, person: 134 },
        O::AccountSignInStarted { source: 135, address: b"http://issuer/visit".as_slice().into() },
        O::AccountSignedOut { confirmed: true, failed_hosts: Box::new([]) },
        O::EscalationDecided {
            task: 135,
            revision: Revision { value: 136 },
            by: 137,
            at: Wall::from_nanos(138),
            choice: EscalationChoice::Passed,
        },
    ];
    for value in outcomes {
        let (mut domain, _) = live();
        let id = install(&mut domain, Body::Task(Box::new(task())));
        let request = send(&mut domain, ObjectKey::Task(8));
        let answer = Answer::Done(Outcome::Document(value));
        assert!(answered(&mut domain, request, answer.clone()).is_empty());
        assert_eq!(domain.objects().get(id).unwrap().card, Card::Open);
        assert_eq!(domain.objects().get(id).unwrap().answer.as_ref(), Some(&answer));
        assert!(answered(&mut domain, request, refusal_answer(Refusal::Unknown)).is_empty());
        assert_eq!(domain.objects().get(id).unwrap().answer, Some(answer));
    }
}

#[test]
fn a_moving_out_card_keeps_numbers_without_the_record() {
    let (mut domain, _) = live();
    let id = install(&mut domain, Body::Task(Box::new(task())));
    let request = send(&mut domain, ObjectKey::Task(8));
    answered(
        &mut domain,
        request,
        Answer::Done(Outcome::Made {
            what: Made::MovingOut {
                account: 19,
                number: 23,
                record: skein_lib::Secret::new(Box::from(b"private moved record".as_slice())),
            },
        }),
    );
    assert_eq!(
        domain.objects().get(id).unwrap().answer,
        Some(Answer::Done(Outcome::Made {
            what: Made::MovingOut { account: 19, number: 23, record: skein_lib::Secret::new(Box::new([])) },
        }))
    );
}
