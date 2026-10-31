use crate::tests::fixtures::{Fixture, ReadPage, Watched};
pub(in crate::tests) use crate::tests::kit::text;
use crate::tests::state::domain::spine::{LIMITS, delivered, drive, frame, open_token, start};
use crate::{
    Request,
    state::objects::{Card, Left},
};
use jig_client::contract::*;
use skein_lib::{List, Time, Wall};

pub(in crate::tests) fn chip(task: u64) -> Chip {
    Chip {
        task,
        title: DocumentText::complete(text(b"title")),
        executor: ChipExecutor::Party { party: 7 },
        phase: Phase::Held,
        details: Box::new(phase_details(Phase::Held)),
    }
}
pub(in crate::tests) fn phase_details(phase: Phase) -> ChipPhaseFacts {
    let owner = match phase {
        Phase::Waiting => ChipPhase::Waiting { dependencies: List::with_capacity(0), more: false, holds: false },
        Phase::Idle => ChipPhase::Active(RunStage::Idle),
        Phase::Due => ChipPhase::Active(RunStage::Due),
        Phase::Preparing => ChipPhase::Active(RunStage::Preparing),
        Phase::Claimed => ChipPhase::Active(RunStage::Claimed { attempt: 1 }),
        Phase::Running => ChipPhase::Active(RunStage::Running { attempt: 1 }),
        Phase::BackingOff => ChipPhase::Active(RunStage::BackingOff { until: Wall::EPOCH }),
        Phase::Active => {
            ChipPhase::Person { address: PersonAddress::Party(7), taken_by: None, since: Some(Wall::EPOCH) }
        }
        Phase::Closing => ChipPhase::Closing(ClosingStage::Settled),
        Phase::Held => ChipPhase::Held { reason: HoldCategory::Stopped, revision: 3 },
        Phase::Ended | Phase::Done | Phase::Failed | Phase::Cancelled => ChipPhase::Ended,
    };
    let mut value = crate::tests::fixtures::chip_phase(owner);
    value.details.project = 1;
    value.details.executor = Executor::Person(PersonAddress::Party(7));
    if phase == Phase::Held {
        let mut hold = crate::tests::fixtures::hold();
        hold.reason = HoldCategory::Stopped;
        hold.revision = 3;
        value.details.hold = Some(hold);
    }
    value
}

pub(in crate::tests) fn owner_authority() -> AuthorityDocument {
    AuthorityDocument {
        authority: Authority {
            models: None,
            tools: 0,
            grants: List::with_capacity(0),
            delegation: Delegation { kinds: List::with_capacity(0), tasks: 1, depth: 1 },
            spend: 10,
            deadline: None,
            notes: 0,
            note_resources: List::with_capacity(0),
        },
        cut: false,
    }
}

pub(in crate::tests) fn question() -> Question {
    Question {
        task: chip(8),
        from: None,
        message: 1,
        words: DocumentText::complete(text(b"whole question")),
        since: Wall::EPOCH,
        acts: List::with_capacity(0),
        ending: None,
    }
}

pub(in crate::tests) fn result() -> Result {
    Result {
        task: chip(8),
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText::complete(text(b"report")))),
        at: Some(Wall::EPOCH),
        acts: List::with_capacity(0),
        details: None,
    }
}
fn inbox(keys: &[ObjectKey]) -> Delivery {
    let mut entries = List::with_capacity(2);
    for key in keys {
        let mut entry = crate::tests::fixtures::inbox_entry(*key);
        entry.project = Some(1);
        entries.push(entry).unwrap();
    }
    Delivery::Inbox(Box::new(crate::tests::fixtures::inbox(entries)))
}

pub(in crate::tests) fn live() -> (Fixture, jig_client::Handle) {
    live_with(&LIMITS)
}
pub(in crate::tests) fn live_with(limits: &crate::tests::fixtures::Profile) -> (Fixture, jig_client::Handle) {
    let mut domain = Fixture::new(limits);
    let f = start(&mut domain, List::with_capacity(0));
    let out = delivered(&mut domain, Time::ZERO, f, frame(7));
    let page = open_token(&out);
    crate::tests::state::domain::pages::finish_initial_inbox_reads(&mut domain, &out);
    (domain, page)
}
pub(in crate::tests) fn watched(domain: &mut Fixture, page: jig_client::Handle, keys: &[ObjectKey]) -> Box<[Request]> {
    drive(domain, Time::ZERO, Watched { watch: page, event: WatchEvent::Delivered(inbox(keys)) })
}
pub(in crate::tests) fn read_token(rows: &[Request]) -> jig_client::Handle {
    for row in rows {
        if let Request::Client { asked: jig_client::Event::Read { handle: read, .. } } = row {
            return *read;
        }
    }
    unreachable!("an object read was emitted")
}

#[test]
fn a_drop_reads_how_the_card_ended_and_a_preexisting_read_can_supply_the_terminal() {
    for read_before_drop in [true, false] {
        let (mut domain, page) = live();
        let key = ObjectKey::Question { task: 8, message: 1 };
        let first = watched(&mut domain, page, &[key]);
        let initial = read_token(&first);
        if !read_before_drop {
            drive(
                &mut domain,
                Time::ZERO,
                ReadPage { read: initial, result: ReadResult::Value(ReadValue::Question(Box::new(question()))) },
            );
        }
        let dropped = watched(&mut domain, page, &[]);
        let terminal = if read_before_drop {
            assert!(dropped.is_empty());
            initial
        } else {
            read_token(&dropped)
        };
        let id = domain.objects().find(key).unwrap();
        let Card::Leaving { how, .. } = &domain.objects().get(id).unwrap().card else {
            panic!("terminal read keeps card leaving");
        };
        assert_eq!(*how, Left::Looking);
        let ending = Ending::Decided {
            by: frame(9).party.number,
            party: Some(crate::tests::state::domain::spine::party(9)),
            how: Some(Decision::Accepted),
            at: Wall::EPOCH,
        };
        let mut ended = question();
        ended.ending = Some(ending.clone());
        drive(
            &mut domain,
            Time::from_nanos(30),
            ReadPage { read: terminal, result: ReadResult::Value(ReadValue::Question(Box::new(ended))) },
        );
        assert_eq!(
            domain.objects().get(id).unwrap().card,
            Card::Leaving { how: Left::Ended { how: ending }, until: Time::from_nanos(5_000_000_030) }
        );
        assert_ne!(domain.page(), &crate::state::page::Page::Missing);
        assert_eq!(domain.objects().get(id).unwrap().sources.len(), 0);
    }
}

#[test]
fn a_party_proposal_maker_does_not_supply_a_task_address() {
    for number in [0, u64::MAX] {
        assert_eq!(
            crate::state::page::inbox::address(ObjectKey::Proposal(ProposalKey {
                proposer: Proposer::Task(number),
                number: u64::MAX,
            })),
            Some(crate::state::address::Address::task(number))
        );
        assert_eq!(
            crate::state::page::inbox::address(ObjectKey::Proposal(ProposalKey {
                proposer: Proposer::Party(number),
                number: u64::MAX,
            })),
            None
        );
    }
}
