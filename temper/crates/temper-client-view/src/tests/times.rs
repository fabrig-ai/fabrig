use super::fixtures;
use crate::words::TimeForm::{Ago, Clock, Day, Span};
use skein_lib::{Queue, Wall};
use temper_client_domain::address::Filter;
const SECOND: u64 = 1_000_000_000;
const MINUTE: u64 = 60;
const HOUR: u64 = 3600;
const DAY: u64 = 86400;
// Thursday 22 October 2026, noon UTC.
const NOON: u64 = 1_792_670_400;
const MIDNIGHT: u64 = NOON - 12 * HOUR;
const YEAR: u64 = 1_798_761_600; // Friday 1 January 2027, midnight UTC.
fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(SECOND).unwrap())
}
#[test]
fn the_four_time_forms_and_their_next_changes_at_each_bound() {
    let words = fixtures::words();
    for (form, at, now, said, next) in [
        (Ago, NOON, NOON, "just now", NOON + MINUTE),
        (Ago, NOON, NOON + MINUTE - 1, "just now", NOON + MINUTE),
        (Ago, NOON, NOON + MINUTE, "1 min ago", NOON + 2 * MINUTE),
        (Ago, NOON, NOON + 59 * MINUTE, "59 min ago", NOON + HOUR),
        (Ago, NOON, NOON + HOUR, "1 h ago", NOON + HOUR + MINUTE),
        (Ago, NOON, NOON + HOUR + 28 * MINUTE, "1 h 28 min ago", NOON + HOUR + 29 * MINUTE),
        (Ago, MIDNIGHT, NOON - 1, "11 h 59 min ago", NOON),
        (Ago, MIDNIGHT, NOON, "at 00:00", MIDNIGHT + DAY),
        (Ago, MIDNIGHT, MIDNIGHT + DAY - 1, "at 00:00", MIDNIGHT + DAY),
        (Ago, MIDNIGHT, MIDNIGHT + DAY, "yesterday", MIDNIGHT + 2 * DAY),
        (Ago, MIDNIGHT, MIDNIGHT + 2 * DAY - 1, "yesterday", MIDNIGHT + 2 * DAY),
        (Ago, MIDNIGHT, MIDNIGHT + 2 * DAY, "Thu", MIDNIGHT + 3 * DAY),
        (Ago, MIDNIGHT - 7 * DAY, NOON, "Thu", MIDNIGHT + DAY),
        (Ago, MIDNIGHT - 8 * DAY, NOON, "14 October", MIDNIGHT + DAY),
        (Ago, YEAR - 9 * DAY, YEAR - 1, "23 December", YEAR),
        (Ago, YEAR - 9 * DAY, YEAR, "23 December 2026", YEAR + DAY),
        (Ago, NOON + HOUR, NOON, "just now", NOON + HOUR + MINUTE),
        (Span, NOON, NOON, "under 1 min", NOON + MINUTE),
        (Span, NOON, NOON + MINUTE - 1, "under 1 min", NOON + MINUTE),
        (Span, NOON, NOON + MINUTE, "1 min", NOON + 2 * MINUTE),
        (Span, NOON, NOON + 12 * MINUTE, "12 min", NOON + 13 * MINUTE),
        (Span, NOON, NOON + HOUR - 1, "59 min", NOON + HOUR),
        (Span, NOON, NOON + HOUR, "1 h", NOON + HOUR + MINUTE),
        (Span, NOON, NOON + HOUR + 28 * MINUTE, "1 h 28 min", NOON + HOUR + 29 * MINUTE),
        (Span, NOON, NOON + 2 * HOUR, "2 h", NOON + 2 * HOUR + MINUTE),
        (Span, NOON, NOON + DAY - 1, "23 h 59 min", NOON + DAY),
        (Span, NOON, NOON + DAY, "1 d", NOON + 2 * DAY),
        (Span, NOON, NOON + 3 * DAY - 1, "2 d", NOON + 3 * DAY),
        (Span, NOON, NOON + 3 * DAY, "3 d", NOON + 4 * DAY),
        (Span, NOON + HOUR, NOON, "under 1 min", NOON + HOUR + MINUTE),
        (Clock, MIDNIGHT + 7 * HOUR + 20 * MINUTE, NOON, "07:20", MIDNIGHT + DAY),
        (Clock, MIDNIGHT, MIDNIGHT + DAY - 1, "00:00", MIDNIGHT + DAY),
        (Clock, MIDNIGHT, MIDNIGHT + DAY, "Thu 00:00", MIDNIGHT + 2 * DAY),
        (Clock, MIDNIGHT - 2 * DAY + 7 * HOUR + 20 * MINUTE, NOON, "Tue 07:20", MIDNIGHT + DAY),
        (Clock, MIDNIGHT - 6 * DAY, NOON, "Fri 00:00", MIDNIGHT + DAY),
        (Clock, MIDNIGHT - 7 * DAY, NOON, "15 October 00:00", MIDNIGHT + DAY),
        (Clock, YEAR - 8 * DAY, YEAR - 1, "24 December 00:00", YEAR),
        (Clock, YEAR - 8 * DAY, YEAR, "24 December 2026 00:00", YEAR + DAY),
        (Clock, NOON + HOUR, NOON, "13:00", MIDNIGHT + DAY),
        (Day, NOON, NOON, "Today · Thursday 22 October", MIDNIGHT + DAY),
        (Day, MIDNIGHT, MIDNIGHT + DAY - 1, "Today · Thursday 22 October", MIDNIGHT + DAY),
        (Day, MIDNIGHT, MIDNIGHT + DAY, "Yesterday · Thursday 22 October", MIDNIGHT + 2 * DAY),
        (Day, MIDNIGHT, MIDNIGHT + 2 * DAY - 1, "Yesterday · Thursday 22 October", MIDNIGHT + 2 * DAY),
        (Day, MIDNIGHT, MIDNIGHT + 2 * DAY, "Thursday 22 October", MIDNIGHT + 3 * DAY),
        (Day, MIDNIGHT - 2 * DAY, NOON, "Tuesday 20 October", MIDNIGHT + DAY),
        (Day, YEAR - DAY, YEAR - 1, "Today · Thursday 31 December", YEAR),
        (Day, YEAR - DAY, YEAR, "Yesterday · Thursday 31 December 2026", YEAR + DAY),
        (Day, NOON + HOUR, NOON, "Today · Thursday 22 October", MIDNIGHT + DAY),
        // The minute is elapsed from the supplied instant, including its seconds.
        (Ago, NOON + 20, NOON + 79, "just now", NOON + 80),
        (Ago, NOON + 20, NOON + 80, "1 min ago", NOON + 140),
    ] {
        let (text, tick) = words.time(form, wall(at), wall(now));
        assert_eq!(text.as_ref(), said.as_bytes(), "{form:?} at {at}, now {now}");
        assert_eq!(tick, Some(wall(next)), "{form:?} at {at}, now {now}");
    }
}
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
    let c::ReadValue::OwnerProposal(document) = &mut proposal else { panic!("owner proposal") };
    document.key = key;
    let task_root = fixtures::task::held_root();
    let mut task = task_root.pages().task().unwrap().document(task_root.client()).unwrap().clone();
    match task.hold.as_mut().unwrap() {
        c::TaskHold::Owner(hold) => hold.hold_wait_since = Some(Wall::EPOCH),
        c::TaskHold::Document(_) => panic!("owner hold"),
    }
    let mut person_chip = task.chip.clone();
    person_chip.phase = c::Phase::Waiting;
    let person = c::ReadValue::OwnerPersonTask(Box::new(c::OwnerPersonTask {
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
            c::ReadValue::OwnerEscalation(Box::new(c::OwnerEscalation {
                task: Box::new(task),
                acts: List::with_capacity(0),
            })),
        ),
        (c::Read::PersonTask { task: fixtures::INBOX_TASK }, person),
    ] {
        let root = fixtures::inbox_with_document(Filter::ALL, Some((read.clone(), value)));
        let view = at_twelve_minutes(&root);
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Waiting 12 min").is_some(), "{read:?}");
        assert_eq!(view.next_change(), Some(wall(13 * MINUTE)), "{read:?}");
    }
}

#[test]
fn a_build_keeps_the_earliest_change_of_the_forms_it_drew() {
    let words = fixtures::words();
    let mut times = crate::words::Times::new(wall(12 * MINUTE));
    assert_eq!(times.next, None);
    let _clock = times.say(&words, Clock, Wall::EPOCH);
    let _span = times.say(&words, Span, wall(5));
    let _ago = times.say(&words, Ago, Wall::EPOCH);
    assert_eq!(times.next, Some(wall(12 * MINUTE + 5)));
}

#[test]
fn a_turns_bare_clock_stays_fixed_across_midnight_without_its_own_wake() {
    let words = fixtures::words();
    for now in [MIDNIGHT + DAY - 1, MIDNIGHT + DAY] {
        let times = crate::words::Times::new(wall(now));
        assert_eq!(words.turn_clock(wall(NOON)).as_ref(), b"12:00");
        assert_eq!(times.next, None);
    }
}
