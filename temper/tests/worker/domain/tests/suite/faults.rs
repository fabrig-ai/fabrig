//! Whole-worker fault stories (temper/docs/design/testing/worker.md, 3).
use skein_lib::{Duration, Time, Token};
use skein_world::inspect::{Moment, Release, Wake, check_tables, check_waits};
use temper_worker_domain_world::{builder::Builder, whole::faults};
use temper_worker_inspect::faults::{DISK_FULL, FETCH_CUT, PLACEMENT_BUSY};

#[test]
fn build_places_pass_to_checks_in_the_order_they_asked() {
    let mut builder = Builder::new(1);
    let (first, second, third) = (Token::new(1), Token::new(2), Token::new(3));
    let at = Time::from_nanos(7);
    assert!(builder.ask(first, at));
    assert!(!builder.ask(second, at));
    assert!(!builder.ask(third, at));
    let inspection = builder.inspection();
    assert_eq!(inspection.component(), "builder");
    assert_eq!(inspection.waits()[0].kind, "a check waiting for a build place");
    assert_eq!(inspection.waits()[0].wakes, [Wake::Waiter("run 1".into())]);
    let table = &inspection.tables()[0];
    assert_eq!((table.name, table.capacity), ("build places", 1));
    assert_eq!(table.entries[0].key, "run 1");
    assert_eq!(table.entries[0].release, Release::Pending);
    assert_eq!(table.refused[0].since, at);
    let moment = Moment { seed: 0, at, iteration: 1 };
    check_waits(&[&inspection], moment).expect("live holder wakes queued checks");
    check_tables(&[&inspection], moment).expect("full table may refuse");
    assert_eq!(builder.finish(first), Some(second));
    assert_eq!(builder.finish(second), Some(third));
    assert_eq!(builder.finish(third), None);
    assert_eq!(builder.inspection().tables()[0].entries.len(), 0);
}

#[test]
fn a_build_place_held_after_its_run_ended_is_due() {
    let mut builder = Builder::new(1);
    let run = Token::new(4);
    assert!(builder.ask(run, Time::ZERO));
    builder.ended(run);
    let inspection = builder.inspection();
    assert_eq!(inspection.tables()[0].entries[0].release, Release::Due);
    let finding =
        check_tables(&[&inspection], Moment { seed: 4, at: Time::ZERO, iteration: 1 }).expect_err("ended hold");
    assert!(finding.contains("builder build places"), "{finding}");
}

#[test]
fn a_cut_fetch_fails_prepare_and_the_next_attempt_prepares_afresh() {
    let ledger = faults::cut_prepare(3, false).expect("world checks");
    let [fault] = ledger.entries() else { panic!("one cut") };
    assert_eq!(fault.kind, FETCH_CUT);
    assert_eq!(fault.until, Some(fault.from));
    assert!(fault.struck.request.as_ref().expect("operation").ends_with(" fetch"));
}

#[test]
fn a_full_disk_fails_prepare_with_gits_words_and_releases_every_hold() {
    let ledger = faults::full_prepare(5, false).expect("world checks");
    let [fault] = ledger.entries() else { panic!("one full disk") };
    assert_eq!(fault.kind, DISK_FULL);
    assert_eq!(fault.until, None);
}

#[test]
fn a_busy_placement_is_admitted_once_a_slot_frees() {
    let ledger = faults::busy_placement(7, true, Duration::from_secs(1)).expect("world checks");
    let [fault] = ledger.entries() else { panic!("one busy interval") };
    assert_eq!(fault.kind, PLACEMENT_BUSY);
    assert_eq!(fault.struck.request.as_deref(), Some("run 83 attempt 13"));
    assert_eq!(fault.from, Time::ZERO);
    assert_eq!(fault.until, Some(Time::ZERO.saturating_add(Duration::from_secs(2))));
}
