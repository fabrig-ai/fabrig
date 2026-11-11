//! A commit's news uses only Checks, and counts the ends its hints promised.
use super::*;
use crate::tests::fixtures;
use crate::{Event, resume, step};
use skein_forge::api::{Answer, Change, Hint, Op, Read};
use skein_lib::{Token, Wall};
use temper_engine_domain_forge_change as change;
const REPO: Provider = Provider { forge: 1, repository: 2 };

fn rows(ends: usize, pending: bool) -> Box<[Status]> {
    let mut rows = List::with_capacity(4);
    for index in 0..ends.saturating_add(usize::from(pending)) {
        rows.push(Status {
            at: Time::ZERO,
            context: Box::new([u8::try_from(index).unwrap()]),
            check: if index < ends { Check::Passed } else { Check::Pending },
            description: Box::new([]),
            url: Box::new([]),
            job: None,
        })
        .unwrap();
    }
    rows.into_boxed()
}
fn world(head: Commit) -> (Domain, Env<Limits>) {
    let mut d = crate::tests::domain();
    let mut e = crate::tests::env();
    e.limits.client.hinted = Duration::from_secs(1);
    e.limits.client.poll = Duration::from_secs(5);
    e.limits.client.settle = Duration::from_secs(30);
    let mut out = Queue::uncapped();
    step(&mut d, &e, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, &mut out);
    step(&mut d, &e, Event::ReadAfresh, &mut out);
    step(&mut d, &e, Event::SettleOutbox, &mut out);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.change.state = change::State::Checking { head: fixtures::HEAD };
    row.change.last_head = Some(fixtures::HEAD);
    d.changes.insert(row.task, row.clone()).unwrap();
    crate::settle::observe(&mut d, &e, &row, None, fixtures::BASE);
    observe(&mut d, &e, fixtures::CHANGE, head, &rows(0, true), Ci::Pending);
    (d, e)
}
fn at(e: &Env<Limits>, seconds: u64) -> Env<Limits> {
    Env {
        now: e.now.saturating_add(Duration::from_secs(seconds)),
        wall: Wall::from_nanos(e.wall.as_nanos().saturating_add(Duration::from_secs(seconds).as_nanos())),
        limits: e.limits,
    }
}
fn delivery(d: &mut Domain, e: &Env<Limits>, head: Commit) {
    let mut out = Queue::uncapped();
    step(d, e, Event::Hint { hint: Hint { repository: REPO, change: Change::Commit(head), key: None } }, &mut out);
    for request in &out {
        assert!(!matches!(request, Request::WakeProcedure { .. }));
    }
    assert!(d.procedure_wakes.is_empty(), "a hint alone is not a verdict");
}
fn call(d: &mut Domain, e: &Env<Limits>, head: Commit) -> Token {
    let mut out = Queue::uncapped();
    super::super::fire(d, e, &mut out);
    for _ in 0_u32..8 {
        if d.is_ready() {
            resume(d, e, &mut out);
        }
    }
    let mut call = None;
    for request in &out {
        if let Request::Call { call: token, op, .. } = request {
            assert_eq!(*op, Op::Read(Read::Checks { commit: head }), "never a whole fresh gather");
            assert!(call.replace(*token).is_none(), "one commit read");
        }
        assert!(!matches!(request, Request::WakeProcedure { .. }), "pending is believed");
    }
    call.expect("one Checks read at hinted")
}
fn answer(d: &mut Domain, e: &Env<Limits>, call: Token, rows: Box<[Status]>) {
    let mut out = Queue::uncapped();
    step(
        d,
        e,
        Event::Client(skein_forge_client::Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(Answer::Checks(rows)),
        }),
        &mut out,
    );
}

#[test]
fn a_shown_end_is_read_once_and_its_pending_wakes_no_procedure() {
    let (mut d, e) = world(fixtures::HEAD);
    delivery(&mut d, &e, fixtures::HEAD);
    let one = at(&e, 1);
    let read = call(&mut d, &one, fixtures::HEAD);
    answer(&mut d, &one, read, rows(1, true));
    assert_eq!(deadline(&d), None, "one shown end settles the hint even with another pending job");
    assert!(d.procedure_wakes.is_empty());
}

#[test]
fn a_late_end_retries_with_growing_waits_and_a_second_hint_does_not_restart_them() {
    let (mut d, e) = world(fixtures::HEAD);
    delivery(&mut d, &e, fixtures::HEAD);
    for second in [1, 2] {
        let now = at(&e, second);
        let read = call(&mut d, &now, fixtures::HEAD);
        answer(&mut d, &now, read, rows(0, true));
    }
    assert_eq!(deadline(&d), Some(at(&e, 4).now));
    // The first status is written at +3 s; the +4 read sees it. A second
    // delivery in the window still owes another end, and reads at hinted.
    delivery(&mut d, &at(&e, 3), fixtures::HEAD);
    let four = at(&e, 4);
    let read = call(&mut d, &four, fixtures::HEAD);
    answer(&mut d, &four, read, rows(1, true));
    assert_eq!(deadline(&d), Some(at(&e, 8).now), "second hint adds an end, not a new doubling");
    let eight = at(&e, 8);
    let read = call(&mut d, &eight, fixtures::HEAD);
    answer(&mut d, &eight, read, rows(2, true));
    assert_eq!(deadline(&d), None);
    assert!(d.procedure_wakes.is_empty());
}

#[test]
fn an_unshown_end_becomes_news_at_settle_not_a_perpetual_reread() {
    let (mut d, e) = world(fixtures::HEAD);
    delivery(&mut d, &e, fixtures::HEAD);
    let one = at(&e, 1);
    let read = call(&mut d, &one, fixtures::HEAD);
    answer(&mut d, &one, read, rows(0, true));
    let mut out = Queue::uncapped();
    fire(&mut d, &at(&e, 30), &mut out);
    assert_eq!(d.procedure_wakes.get(&fixtures::CHANGE), Some(&true));
    assert_eq!(deadline(&d), None);
}

#[test]
fn a_commit_no_longer_used_by_its_change_sends_no_expiry_news() {
    let (mut d, e) = world(fixtures::HEAD);
    delivery(&mut d, &e, fixtures::HEAD);
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    row.change.last_head = Some([9; 32]);
    row.change.state = change::State::Checking { head: [9; 32] };
    fire(&mut d, &at(&e, 30), &mut Queue::uncapped());
    assert!(d.procedure_wakes.is_empty(), "old head is no longer a decision input");
    assert_eq!(deadline(&d), None);
}

#[test]
fn a_head_or_base_verdict_wakes_only_its_changes_and_their_decision_reads_afresh() {
    for head in [fixtures::HEAD, fixtures::BASE] {
        for check in [Check::Passed, Check::Failed] {
            let (mut d, e) = world(head);
            delivery(&mut d, &e, head);
            let one = at(&e, 1);
            let read = call(&mut d, &one, head);
            let Answer::Checks(mut rows) = fixtures::checks(check) else { unreachable!() };
            rows[0].context = Box::new([0]);
            answer(&mut d, &one, read, rows);
            assert_eq!(d.procedure_wakes.len(), 1);
            assert_eq!(d.procedure_wakes.get(&fixtures::CHANGE), Some(&true));
            d.procedure_wakes.remove(&fixtures::CHANGE);
            fixtures::step_change(&mut d);
            let (_, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Branch { branch: Box::from(b"temper/51".as_slice()) }));
        }
    }
}
