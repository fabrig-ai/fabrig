//! Whole-task erasure and project release through the child's entry point.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "entry-point test scripts use ordinary Rust (programming-model.md, 10.2)"
)]
use super::revisions::{Harness, task, to};
use crate::{
    Control, Event, Funder, Key, Mode, Numbers, Owner, Party, PersonAction, PersonProposal, PersonProposalState,
    PlacedHold, Request, Scope, Status, Stored, Stub, TaskRecord,
};
use skein_lib::{Token, Wall};

const TASK: u64 = 1;
const DELEGATE: u64 = 2;
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const PERSON: u64 = 9;
const OWNER_HOLD: u64 = 10;
const SCOPE_HOLD: u64 = 11;
const OTHER_HOLD: u64 = 12;
const PROPOSAL: u64 = 20;
const OTHER_PROPOSAL: u64 = 21;

fn end(test: &mut Harness, number: u64) -> Vec<Request> {
    test.send(Event::Control {
        reply_to: to(),
        by: Party::Person(PERSON),
        task: number,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    test.send(Event::EffectsSettled { task: number });
    test.send(Event::Settled { task: number })
}

fn ended(test: &Harness, number: u64) -> Box<TaskRecord> {
    let Some(Stored::Ended(record)) = test.stored.get(&Key::Ended(number)) else {
        panic!("task ended");
    };
    record.clone()
}

fn erases(out: &[Request]) -> Vec<Key> {
    out.iter()
        .map(|request| match request {
            Request::Erase { key } => *key,
            _ => panic!("erasure emits only erases: {request:?}"),
        })
        .collect()
}

fn ended_saves(out: &[Request]) -> Vec<Key> {
    out.iter()
        .filter_map(|request| match request {
            Request::Save {
                record:
                    record @ (Stored::Ended(_)
                    | Stored::EndedByParty(_)
                    | Stored::EndedByProject(_)
                    | Stored::EndedDelegate(_)),
            } => Some(record.key()),
            _ => None,
        })
        .collect()
}

#[test]
fn erasing_an_ended_person_task_erases_what_its_end_saved() {
    let mut test = Harness::new();
    let mut listed = task(TASK, &[]);
    listed.flags.listed = true;
    listed.label = 7;
    test.env.wall = Wall::from_nanos(1000);
    test.make([listed].into());
    let saves = end(&mut test, TASK);
    let record = ended(&test, TASK);
    let before = test.stored.clone();
    let out = test.send(Event::Erase { record: record.clone() });
    let mut expected = ended_saves(&saves);
    let mut keys = erases(&out);
    expected.sort();
    keys.sort();
    assert_eq!(keys, expected);
    assert!(keys.contains(&Key::Ended(TASK)));
    assert!(keys.contains(&crate::EndedByParty { party: PERSON, label: 7, ended_at: test.env.wall, task: TASK }.key()));
    assert!(
        keys.contains(&crate::EndedByProject { project: PROJECT, label: 7, ended_at: test.env.wall, task: TASK }.key())
    );
    assert_eq!(test.stored, before.into_iter().filter(|(key, _)| !keys.contains(key)).collect());
    assert!(test.domain.task(TASK).is_none());
    assert!(test.domain.stub(TASK).is_none());
    assert_eq!(erases(&test.send(Event::Erase { record })), erases(&out));
}

#[test]
fn erasing_an_ended_delegate_erases_its_edges_under_its_parent() {
    let mut test = Harness::new();
    test.make([task(TASK, &[])].into());
    test.send(Event::Make { reply_to: to(), creator: Party::Task(TASK), batch: Box::new([task(DELEGATE, &[])]) });
    let saves = end(&mut test, DELEGATE);
    let record = ended(&test, DELEGATE);
    let parent = test.domain.task(TASK).unwrap().clone();
    let mut expected = ended_saves(&saves);
    expected.push(Key::Delegate { requester: TASK, task: DELEGATE });
    expected.sort();
    let mut keys = erases(&test.send(Event::Erase { record }));
    keys.sort();
    assert_eq!(keys, expected);
    assert_eq!(
        keys,
        [
            Key::Ended(DELEGATE),
            Key::EndedDelegate { requester: TASK, task: DELEGATE },
            Key::Delegate { requester: TASK, task: DELEGATE }
        ]
    );
    assert_eq!(test.domain.task(TASK), Some(&parent));
}

fn hold(number: u64, project: u32, owner: Owner, scope: Scope) -> PlacedHold {
    PlacedHold {
        number,
        project,
        owner,
        mode: Mode::Stop,
        scope,
        reason: Box::new([]),
        placed_at: 0,
        revision: 1,
        exempt: Box::new([]),
        lifted: false,
    }
}

fn retain_hold(test: &mut Harness, hold: PlacedHold) {
    test.stored.insert(Key::PlacedHold(hold.number), Stored::PlacedHold(Box::new(hold.clone())));
    test.domain.placed.insert(hold.number, hold).unwrap();
}

#[test]
fn erase_drops_its_stub_and_holds_by_owner_or_subtree() {
    let mut test = Harness::new();
    test.make([task(TASK, &[])].into());
    end(&mut test, TASK);
    let record = ended(&test, TASK);
    let stub = Stub { task: TASK, phase: Status::Cancelled, result: Token::new(TASK) };
    test.domain.stubs.insert(TASK, stub).unwrap();
    test.stored.insert(Key::Stub(TASK), Stored::Stub(stub));
    retain_hold(&mut test, hold(OWNER_HOLD, PROJECT, Owner::Task { task: TASK }, Scope::Project));
    retain_hold(&mut test, hold(SCOPE_HOLD, PROJECT, Owner::Party { person: PERSON }, Scope::Subtree { task: TASK }));
    let other = hold(OTHER_HOLD, PROJECT, Owner::Party { person: PERSON }, Scope::Project);
    retain_hold(&mut test, other.clone());
    let keys = erases(&test.send(Event::Erase { record }));
    assert_eq!(
        keys,
        [
            Key::Ended(TASK),
            crate::EndedByParty { party: PERSON, label: 0, ended_at: test.env.wall, task: TASK }.key(),
            Key::Stub(TASK),
            Key::PlacedHold(OWNER_HOLD),
            Key::PlacedHold(SCOPE_HOLD)
        ]
    );
    assert!(test.domain.stub(TASK).is_none());
    assert!(test.domain.placed_hold(OWNER_HOLD).is_none());
    assert!(test.domain.placed_hold(SCOPE_HOLD).is_none());
    assert_eq!(test.domain.placed_hold(OTHER_HOLD), Some(&other));
}

#[test]
fn erase_ignores_a_number_that_is_live() {
    let mut test = Harness::new();
    test.make([task(TASK, &[])].into());
    let live = test.domain.task(TASK).unwrap().clone();
    let before = test.stored.clone();
    assert!(test.send(Event::Erase { record: Box::new(live.clone()) }).is_empty());
    assert_eq!(test.stored, before);
    assert_eq!(test.domain.task(TASK), Some(&live));
}

#[test]
fn forgetting_a_project_closes_its_live_ledgers_and_keeps_their_spend() {
    let mut test = Harness::new();
    test.send(Event::CarvePool { reply_to: to(), project: PROJECT, period: 0, person: PERSON, budget: 100 });
    test.send(Event::OpenPeriod { reply_to: to(), project: OTHER_PROJECT, period: 0, budget: 100 });
    let period = Funder::Period { project: PROJECT, period: 0 };
    let pool = Funder::Pool { project: PROJECT, period: 0, person: PERSON };
    let recurring = Funder::Recurring { project: PROJECT, period: 0, task: TASK };
    let mut row = *test.domain.funding(pool).unwrap();
    row.funder = recurring;
    test.domain.funding.insert(recurring, row).unwrap();
    let sources = [pool, recurring, period];
    let mut expected = Vec::new();
    for source in sources {
        let row = test.domain.funding.get_mut(&source).unwrap();
        row.numbers = Numbers { budget: 100, spent: 0, spent_below: 23, reserved: 0 };
        row.made = 3;
        let mut closed = *row;
        closed.closed = true;
        expected.push(Request::Erase { key: Key::Ledger(source) });
        expected.push(Request::Save { record: Stored::ClosedLedger(closed) });
    }
    let other = *test.domain.funding(Funder::Period { project: OTHER_PROJECT, period: 0 }).unwrap();
    let out = test.send(Event::ForgetProject { project: PROJECT });
    assert_eq!(out, expected);
    for source in sources {
        assert!(test.domain.funding(source).is_none());
        assert!(!test.stored.contains_key(&Key::Ledger(source)));
        let Some(Stored::ClosedLedger(row)) = test.stored.get(&Key::ClosedLedger(source)) else {
            panic!("closed funding stays in history");
        };
        assert_eq!(row.numbers, Numbers { budget: 100, spent: 0, spent_below: 23, reserved: 0 });
    }
    assert_eq!(test.domain.funding(other.funder), Some(&other));
}

fn proposal(number: u64, project: u32) -> PersonProposal {
    let mut member = task(TASK, &[]);
    member.project = project;
    PersonProposal {
        number,
        proposer: PERSON,
        project,
        action: PersonAction::Batch(Box::new([member])),
        reason: Box::new([]),
        state: PersonProposalState::Pending { since: Wall::EPOCH },
    }
}

fn retain_proposal(test: &mut Harness, proposal: PersonProposal) {
    test.stored.insert(Key::PersonProposal(proposal.number), Stored::PersonProposal(Box::new(proposal.clone())));
    test.domain.person_proposals.insert(proposal.number, proposal).unwrap();
}

#[test]
fn forgetting_a_project_drops_its_person_proposals_and_holds() {
    let mut test = Harness::new();
    retain_proposal(&mut test, proposal(PROPOSAL, PROJECT));
    let other_proposal = proposal(OTHER_PROPOSAL, OTHER_PROJECT);
    retain_proposal(&mut test, other_proposal.clone());
    retain_hold(&mut test, hold(OWNER_HOLD, PROJECT, Owner::Party { person: PERSON }, Scope::Project));
    let other_hold = hold(OTHER_HOLD, OTHER_PROJECT, Owner::Party { person: PERSON }, Scope::Project);
    retain_hold(&mut test, other_hold.clone());
    let out = test.send(Event::ForgetProject { project: PROJECT });
    let erasures = out
        .iter()
        .filter_map(|request| match request {
            Request::Erase { key } => Some(*key),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        erasures,
        [
            Key::Ledger(Funder::Period { project: PROJECT, period: 0 }),
            Key::PersonProposal(PROPOSAL),
            Key::PlacedHold(OWNER_HOLD)
        ]
    );
    assert!(test.domain.person_proposal(PERSON, PROPOSAL).is_none());
    assert!(test.domain.placed_hold(OWNER_HOLD).is_none());
    assert_eq!(test.domain.person_proposal(PERSON, OTHER_PROPOSAL), Some(other_proposal));
    assert_eq!(test.domain.placed_hold(OTHER_HOLD), Some(&other_hold));
    assert!(!test.stored.contains_key(&Key::PersonProposal(PROPOSAL)));
    assert!(!test.stored.contains_key(&Key::PlacedHold(OWNER_HOLD)));
}

#[test]
fn forgetting_a_project_with_a_live_task_keeps_its_state() {
    let mut test = Harness::new();
    test.make([task(TASK, &[])].into());
    let live = test.domain.task(TASK).unwrap().clone();
    let ledger = *test.domain.funding(live.funder).unwrap();
    let before = test.stored.clone();
    assert!(test.send(Event::ForgetProject { project: PROJECT }).is_empty());
    assert_eq!(test.stored, before);
    assert_eq!(test.domain.task(TASK), Some(&live));
    assert_eq!(test.domain.funding(live.funder), Some(&ledger));
}
