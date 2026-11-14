//! Contract regressions for the generic domain-world utilities extracted
//! when smith became temper's second harness consumer (testing-strategy.md, 6).

use skein_lib::{Duration, Rng, Time};
use std::cell::Cell;

use skein_world::domain::{Ledger, Schedule, Span, Stage, Trace, assert_replays, check_replays};

fn at(milliseconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_millis(milliseconds))
}

#[test]
fn same_time_deliveries_keep_send_order_and_a_withdrawn_key_cannot_cancel_a_later_delivery() {
    let mut schedule = Schedule::new();
    let first = schedule.send(at(4), "first");
    let cancelled = schedule.send(at(2), "cancelled");
    let last = schedule.send(at(4), "last");
    assert!(first.serial < cancelled.serial && cancelled.serial < last.serial);
    assert_eq!(schedule.withdraw(cancelled), Some("cancelled"));
    assert_eq!(schedule.next_time(), Some(at(4)));
    assert_eq!(schedule.next(at(3)), None);
    assert_eq!(schedule.next(at(4)), Some("first"));
    let later = schedule.send(at(5), "later");
    assert_ne!(later.serial, cancelled.serial);
    assert_eq!(schedule.withdraw(cancelled), None);
    assert_eq!(schedule.next(at(9)), Some("last"));
    assert_eq!(schedule.next(at(9)), Some("later"));
    assert!(schedule.is_empty());
}

#[test]
fn terminal_races_remove_each_open_obligation_once() {
    let mut ledger = Ledger::new("domain effect");
    ledger.open(1_u64, "read");
    ledger.open(2_u64, "cancelled");
    assert_eq!(ledger.end(1), "read");
    assert_eq!(ledger.take(1), None, "a losing terminal race changes nothing");
    assert_eq!(ledger.take(2), Some("cancelled"));
    ledger.assert_settled();
}

#[test]
#[should_panic(expected = "a domain effect ends once, while in flight: 1")]
fn a_duplicate_terminal_is_a_boundary_failure() {
    let mut ledger = Ledger::new("domain effect");
    ledger.open(1_u64, ());
    ledger.end(1);
    ledger.end(1);
}

#[test]
fn replay_compares_boundary_order_and_the_scenario_outcome() {
    let lines = assert_replays(7, 8, |seed| {
        let mut schedule = Schedule::new();
        let mut trace = Trace::default();
        schedule.send(at(seed), "reply");
        let reply = schedule.next(at(seed)).expect("the scheduled reply is due");
        trace.log(at(seed), reply);
        (trace.lines().to_vec(), schedule.is_empty())
    });
    assert_eq!(lines.len(), 1);
    assert!(lines[0].ends_with("reply"));
}

#[test]
fn settlement_returns_open_keys_until_their_terminals_arrive() {
    let mut ledger = Ledger::new("domain effect");
    assert_eq!(ledger.check_settled(), Ok(()));
    ledger.open(2_u64, ());
    ledger.open(1_u64, ());
    assert_eq!(ledger.check_settled(), Err("every domain effect has ended: [1, 2] have not".to_owned()));
    ledger.end(1);
    assert_eq!(ledger.check_settled(), Err("every domain effect has ended: [2] have not".to_owned()));
    ledger.end(2);
    assert_eq!(ledger.check_settled(), Ok(()));
}

#[test]
fn replay_returns_the_trace_after_checking_both_seeds() {
    // A trace need not implement Debug.
    #[derive(PartialEq)]
    struct Course(u64);

    let seeds = std::cell::RefCell::new(Vec::new());
    let trace = check_replays(7, 8, |seed| {
        seeds.borrow_mut().push(seed);
        (Course(seed), "settled")
    })
    .expect("a seed replays and another takes its own course");
    assert_eq!(trace.0, 7);
    assert_eq!(*seeds.borrow(), [7, 7, 8]);
}

#[test]
fn replay_returns_a_trace_difference_with_its_seed() {
    let calls = Cell::new(0);
    let result = check_replays(7, 8, |_| {
        let call = calls.get();
        calls.set(call + 1);
        (call, ())
    });
    assert_eq!(result, Err("seed 7 replays to the same trace".to_owned()));
    assert_eq!(calls.get(), 2, "a replay failure does not run the other seed");
}

#[test]
fn replay_returns_an_end_difference_with_both_debug_values() {
    let calls = Cell::new(0);
    let result = check_replays(7, 8, |seed| {
        let call = calls.get();
        calls.set(call + 1);
        (seed, ("settled", call))
    });
    assert_eq!(
        result,
        Err("assertion `left == right` failed: seed 7 replays to the same end\n  left: (\"settled\", 1)\n right: (\"settled\", 0)".to_owned())
    );
    assert_eq!(calls.get(), 2, "a replay failure does not run the other seed");
}

#[test]
fn replay_returns_the_two_seeds_when_their_courses_are_the_same() {
    assert_eq!(check_replays(7, 8, |_| ("one course", ())), Err("seeds 7 and 8 take courses of their own".to_owned()));
}

#[test]
fn injected_random_latencies_stay_inside_the_declared_inclusive_span() {
    let span = Span::millis(2, 5);
    let mut rng = Rng::new(19);
    for _ in 0..32 {
        let delay = span.draw(&mut rng);
        assert!(span.min <= delay && delay <= span.max);
    }
    assert_eq!(Span::millis(4, 4).draw(&mut rng), Duration::from_millis(4));
}

#[test]
fn an_uncapped_stage_hands_on_every_event() {
    let mut stage = Stage::<(), u32, (u32, u32)>::uncapped(());
    for event in 0..100 {
        stage.push(event);
    }
    for event in 0..100 {
        assert_eq!(stage.next_event(), Some(event));
        for request in 0..3 {
            stage.out.push((event, request));
        }
    }
    assert_eq!(stage.next_event(), None);
    assert!(!stage.has_events());
    assert_eq!(stage.out.len(), 300);
    for event in 0..100 {
        for request in 0..3 {
            assert_eq!(stage.out.pop(), Some((event, request)));
        }
    }
    assert!(stage.out.is_empty());
}
