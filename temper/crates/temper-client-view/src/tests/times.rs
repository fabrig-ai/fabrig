use super::fixtures;
use skein_lib::{Queue, Wall};
use temper_client_domain::address::Filter;

const MINUTE: u64 = 60;

// Thursday 22 October 2026, noon UTC.

// Friday 1 January 2027, midnight UTC.

#[test]
fn time_rebuilds_at_the_tick_with_the_domains_shown_count_unchanged() {
    use jig_client::contract as c;
    let root = fixtures::inbox_with_document(
        Filter::ALL,
        Some((
            c::Read::Task { task: fixtures::INBOX_REPLY_TASK },
            c::ReadValue::Task(Box::new({
                let mut task = fixtures::task::document();
                task.chip.task = fixtures::INBOX_REPLY_TASK;
                task.last_activity = Wall::EPOCH;
                task
            })),
        )),
    );
    let shown = temper_client_domain::shown(&root);
    let mut view = fixtures::view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    crate::build(&mut view, &root, fixtures::NOW, &mut patches);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"just now").is_some());
    assert_eq!(view.next_change(), Some(wall(MINUTE)));
    while patches.pop().is_some() {}
    crate::build(&mut view, &root, wall(MINUTE - 1), &mut patches);
    assert!(patches.is_empty());
    crate::build(&mut view, &root, wall(MINUTE), &mut patches);
    assert!(!patches.is_empty());
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"1 min ago").is_some());
    assert_eq!(temper_client_domain::shown(&root), shown);
    assert_eq!(view.next_change(), Some(wall(2 * MINUTE)));
    while patches.pop().is_some() {}
    // A new page resets the accumulator, even when the old page drew time.
    crate::build(&mut view, &fixtures::root_in_project(None), wall(MINUTE), &mut patches);
    assert_eq!(view.next_change(), None);
}

fn at_twelve_minutes(root: &temper_client_domain::Domain) -> crate::View {
    let mut view = fixtures::view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    crate::build(&mut view, root, wall(12 * MINUTE), &mut patches);
    view
}
#[test]
fn the_chat_proposal_says_its_wait_only_on_the_chat_page() {
    let view = at_twelve_minutes(&fixtures::chat::proposal_root());
    let meta = super::inbox::styled(view.tree(), crate::look::META);
    assert_eq!(view.tree().nodes()[meta].name.as_deref(), Some(b"waiting 12 min".as_slice()));
    assert_eq!(view.next_change(), Some(wall(13 * MINUTE)));
}
#[test]
fn the_inbox_says_the_question_wait_and_result_arrival_from_their_objects() {
    use jig_client::contract as c;
    let question = super::inbox::question_root();
    let view = at_twelve_minutes(&question);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Waiting 12 min").is_some());
    for (at, expected) in [(Some(Wall::EPOCH), Some(wall(13 * MINUTE))), (None, None)] {
        let result = c::Result {
            task: fixtures::task::document().chip,
            ending: c::Ending::Done,
            value: None,
            at,
            details: None,
            acts: skein_lib::List::with_capacity(0),
        };
        let root = fixtures::inbox_with_document(
            Filter::ALL,
            Some((c::Read::Result { task: fixtures::INBOX_TASK }, c::ReadValue::Result(Box::new(result)))),
        );
        let view = at_twelve_minutes(&root);
        assert_eq!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"12 min ago").is_some(), at.is_some());
        assert_eq!(view.next_change(), expected);
    }
}
#[test]
fn the_inbox_waits_use_the_proposal_hold_and_person_task_clocks() {
    use jig_client::contract as c;
    use skein_lib::List;
    let proposal_root = fixtures::chat::proposal_root();
    let mut proposal = proposal_root.pages().chat().unwrap().cards.as_slice()[0].page.clone().unwrap();
    let key = c::ProposalKey { proposer: c::Proposer::Task(fixtures::INBOX_TASK), number: fixtures::INBOX_QUESTION };
    let c::ReadValue::Proposal(document) = &mut proposal else { panic!("owner proposal") };
    document.key = key;
    let task_root = fixtures::task::held_root();
    let mut task = task_root.pages().task().unwrap().document(task_root.client()).unwrap().clone();
    task.hold.as_mut().unwrap().hold_wait_since = Some(Wall::EPOCH);
    let mut person_chip = task.chip.clone();
    person_chip.phase = c::Phase::Waiting;
    let person = c::ReadValue::PersonTask(Box::new(c::PersonTask {
        task: person_chip,
        revision: c::Revision { value: 1 },
        requester: task.requester,
        spec: task.spec.clone(),
        address: c::PersonAddress::Party(fixtures::chat::PARTY),
        taken_by: None,
        since: Some(Wall::EPOCH),
        stall: None,
        deadline: None,
        contract: task.result_contract.clone(),
        results: List::with_capacity(0),
        cut: false,
        acts: List::with_capacity(0),
    }));
    for (read, value) in [
        (c::Read::Proposal { proposal: key }, proposal),
        (
            c::Read::Escalation { task: fixtures::INBOX_TASK },
            c::ReadValue::Escalation(Box::new(c::Escalation { task: Box::new(task), acts: List::with_capacity(0) })),
        ),
        (c::Read::PersonTask { task: fixtures::INBOX_TASK }, person),
    ] {
        let root = fixtures::inbox_with_document(Filter::ALL, Some((read.clone(), value)));
        let view = at_twelve_minutes(&root);
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Waiting 12 min").is_some(), "{read:?}");
        assert_eq!(view.next_change(), Some(wall(13 * MINUTE)), "{read:?}");
    }
}

pub(super) use crate::words_tests::times::wall;
