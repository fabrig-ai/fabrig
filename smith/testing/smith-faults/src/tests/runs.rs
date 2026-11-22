use skein_llm_kit::faults::{
    CUT_BEFORE_OUTPUT, CUT_MID_CALL, CUT_MID_TEXT, EXPIRED_SIGN_IN, HANG_AFTER_HEAD, IDLE_MID_STREAM, RATE_WINDOW,
    SPENT_CREDITS, UNSERVED_MODEL,
};
use skein_world::faults::{Counter, FaultLedger, Kind};

use super::{inject, time};
use crate::{BILLING_REFUSAL, CHECK_WAITS, EDGE_REFUSAL, Ended, Retried, Runs, UNTOOLED_PROGRAM, WINDOW_ELSEWHERE};

#[test]
fn retries_count_each_struck_fault_on_the_account_in_its_closed_interval() {
    let mut ledger = FaultLedger::new();
    let first = inject(&mut ledger, EDGE_REFUSAL, 10, Some(40), "a");
    let second = inject(&mut ledger, HANG_AFTER_HEAD, 20, None, "a");
    let other = inject(&mut ledger, SPENT_CREDITS, 10, None, "b");
    let mut runs = Runs::new();
    for at in [9, 10, 20, 40, 41] {
        runs.retried(time(at), "a", "call", Retried::Session);
    }
    runs.retried(time(30), "a", "call", Retried::Program);
    runs.judge(&mut ledger).expect("transient faults allow retries");
    assert_eq!(ledger.get(first).count(Counter::SessionRetries), 3);
    assert_eq!(ledger.get(first).count(Counter::ProviderRetries), 1);
    assert_eq!(ledger.get(second).count(Counter::SessionRetries), 3);
    assert_eq!(ledger.get(second).count(Counter::ProviderRetries), 1);
    assert_eq!(ledger.get(other).count(Counter::SessionRetries), 0);
    assert_eq!(ledger.get(other).count(Counter::ProviderRetries), 0);
    assert_eq!(ledger.get(first).count(Counter::TaskTries), 0);
}

#[test]
fn each_fault_row_limits_retries() {
    for (kind, allowed, refused) in [
        (RATE_WINDOW, 0, Some(1)),
        (SPENT_CREDITS, 0, Some(1)),
        (EXPIRED_SIGN_IN, 1, Some(2)),
        (UNSERVED_MODEL, 0, Some(1)),
        (BILLING_REFUSAL, 0, Some(1)),
        (EDGE_REFUSAL, 3, None),
        (HANG_AFTER_HEAD, 3, None),
        (IDLE_MID_STREAM, 3, None),
        (CUT_BEFORE_OUTPUT, 3, None),
        (CUT_MID_TEXT, 3, None),
        (CUT_MID_CALL, 3, None),
        (WINDOW_ELSEWHERE, 0, Some(1)),
        (UNTOOLED_PROGRAM, 0, Some(1)),
    ] {
        for by in [Retried::Session, Retried::Program] {
            for (count, passes) in [(allowed, true)].into_iter().chain(refused.map(|count| (count, false))) {
                let mut ledger = FaultLedger::new();
                let fault = inject(&mut ledger, kind, 10, Some(40), "a");
                let mut runs = Runs::new();
                for _ in 0..count {
                    runs.retried(time(12), "a", "run 1 call 2", by);
                }
                assert_eq!(runs.judge(&mut ledger).is_ok(), passes, "{kind:?}, {by:?}, {count}");
                let counter = match by {
                    Retried::Session => Counter::SessionRetries,
                    Retried::Program => Counter::ProviderRetries,
                };
                assert_eq!(ledger.get(fault).count(counter), count);
            }
        }
    }
    let mut ledger = FaultLedger::new();
    let fault = inject(&mut ledger, RATE_WINDOW, 10, Some(40), "a");
    let mut runs = Runs::new();
    runs.retried(time(41), "a", "call after window", Retried::Session);
    runs.judge(&mut ledger).expect("retry after the window is allowed");
    assert_eq!(ledger.get(fault).count(Counter::SessionRetries), 0);
}

#[test]
fn an_ongoing_window_strike_allows_a_retry_after_its_refusal_resets() {
    let mut ledger = FaultLedger::new();
    let strike = ledger.inject(
        RATE_WINDOW,
        time(1),
        skein_world::faults::Struck { account: Some("a".into()), ..Default::default() },
    );
    let refusal = inject(&mut ledger, RATE_WINDOW, 10, Some(20), "a");
    let mut runs = Runs::new();
    runs.started(time(5), "run 1", "a");
    runs.retried(time(21), "a", "run 1 call 2", Retried::Session);
    runs.ended(time(30), "run 1", "a", Ended::Completed);
    runs.judge(&mut ledger).expect("retry after reset, despite ongoing strike");
    assert_eq!(ledger.get(strike).count(Counter::SessionRetries), 0);
    assert_eq!(ledger.get(refusal).count(Counter::SessionRetries), 0);
}

#[test]
fn an_ongoing_window_strike_finds_a_retry_before_its_refusal_resets() {
    let mut ledger = FaultLedger::new();
    ledger.inject(
        RATE_WINDOW,
        time(1),
        skein_world::faults::Struck { account: Some("a".into()), ..Default::default() },
    );
    let refusal = inject(&mut ledger, RATE_WINDOW, 10, Some(20), "a");
    let mut runs = Runs::new();
    runs.retried(time(15), "a", "run 1 call 2", Retried::Session);
    let finding = runs.judge(&mut ledger).expect_err("retry inside refusal");
    assert!(finding.contains("call \"run 1 call 2\": retry 1 is not allowed"), "{finding}");
    assert_eq!(ledger.get(refusal).count(Counter::SessionRetries), 1);
}

#[test]
fn a_rate_limited_end_requires_a_refusal_reset_after_the_run() {
    for (end, passes) in [(19, true), (20, false), (21, false)] {
        let mut ledger = FaultLedger::new();
        ledger.inject(
            RATE_WINDOW,
            time(1),
            skein_world::faults::Struck { account: Some("a".into()), ..Default::default() },
        );
        inject(&mut ledger, RATE_WINDOW, 10, Some(20), "a");
        let mut runs = Runs::new();
        runs.started(time(5), "run 1", "a");
        runs.ended(time(end), "run 1", "a", Ended::Limited);
        assert_eq!(runs.judge(&mut ledger).is_ok(), passes, "run end {end}");
    }
}

#[test]
fn expired_sign_in_allows_one_retry_per_call_and_per_injection() {
    let mut ledger = FaultLedger::new();
    inject(&mut ledger, EXPIRED_SIGN_IN, 10, Some(20), "a");
    inject(&mut ledger, EXPIRED_SIGN_IN, 30, Some(40), "a");
    let mut runs = Runs::new();
    runs.retried(time(12), "a", "call 1", Retried::Session);
    runs.retried(time(13), "a", "call 2", Retried::Program);
    runs.retried(time(32), "a", "call 1", Retried::Program);
    runs.judge(&mut ledger).expect("each call and each sign-in gets one retry");

    // Changing retrying layers does not buy the same call another retry.
    runs.retried(time(14), "a", "call 1", Retried::Program);
    let mut ledger = FaultLedger::new();
    inject(&mut ledger, EXPIRED_SIGN_IN, 10, Some(20), "a");
    assert!(runs.judge(&mut ledger).is_err());
}

#[test]
fn each_fault_row_limits_run_ends() {
    for (kind, allowed) in [
        (RATE_WINDOW, &[Ended::Completed][..]),
        (SPENT_CREDITS, &[Ended::Limited][..]),
        (EXPIRED_SIGN_IN, &[Ended::Completed, Ended::Unauthorized][..]),
        (UNSERVED_MODEL, &[Ended::Refused][..]),
        (BILLING_REFUSAL, &[Ended::Refused][..]),
        (EDGE_REFUSAL, &[Ended::Completed, Ended::Outage][..]),
        (HANG_AFTER_HEAD, &[Ended::Completed, Ended::Outage][..]),
        (IDLE_MID_STREAM, &[Ended::Completed, Ended::Outage][..]),
        (CUT_BEFORE_OUTPUT, &[Ended::Completed][..]),
        (CUT_MID_TEXT, &[Ended::Completed][..]),
        (CUT_MID_CALL, &[Ended::Completed][..]),
        (WINDOW_ELSEWHERE, &[Ended::Completed][..]),
        (UNTOOLED_PROGRAM, &[Ended::Untooled][..]),
    ] {
        for end in [
            Ended::Completed,
            Ended::Limited,
            Ended::Refused,
            Ended::Unauthorized,
            Ended::Untooled,
            Ended::Outage,
            Ended::Failed,
        ] {
            let mut ledger = FaultLedger::new();
            inject(&mut ledger, kind, 10, Some(40), "a");
            let mut runs = Runs::new();
            runs.started(time(5), "run 1", "a");
            runs.ended(time(50), "run 1", "a", end);
            assert_eq!(runs.judge(&mut ledger).is_ok(), allowed.contains(&end), "{kind:?}, {end:?}");
        }
    }
}

#[test]
fn findings_name_the_fault_interval_and_call_or_run() {
    let mut ledger = FaultLedger::new();
    inject(&mut ledger, RATE_WINDOW, 10, Some(40), "a");
    let mut runs = Runs::new();
    runs.retried(time(12), "a", "run 1 call 2", Retried::Session);
    let finding = runs.judge(&mut ledger).expect_err("retry within a rate window");
    for part in ["provider/rate window", "10000000000 ns..40000000000 ns", "run 1 call 2"] {
        assert!(finding.contains(part), "missing {part}: {finding}");
    }

    let mut ledger = FaultLedger::new();
    inject(&mut ledger, SPENT_CREDITS, 10, None, "a");
    let mut runs = Runs::new();
    runs.started(time(5), "run 2", "a");
    runs.ended(time(50), "run 2", "a", Ended::Failed);
    let finding = runs.judge(&mut ledger).expect_err("spent credits must end limited");
    for part in ["provider/spent credits", "10000000000 ns..ongoing", "run 2"] {
        assert!(finding.contains(part), "missing {part}: {finding}");
    }
}

#[test]
fn latest_intersecting_fault_decides_the_end_not_insertion_order() {
    let mut ledger = FaultLedger::new();
    inject(&mut ledger, SPENT_CREDITS, 15, Some(20), "a");
    inject(&mut ledger, CUT_MID_CALL, 10, Some(11), "a");
    inject(&mut ledger, CUT_MID_TEXT, 1, Some(9), "a");
    inject(&mut ledger, CUT_BEFORE_OUTPUT, 31, None, "a");
    inject(&mut ledger, CUT_BEFORE_OUTPUT, 25, None, "b");
    let mut runs = Runs::new();
    runs.started(time(10), "run", "a");
    runs.ended(time(30), "run", "a", Ended::Limited);
    runs.judge(&mut ledger).expect("latest matching cause is spent credits, even after it stops");

    // Intervals touch at the run's end, and at its start.
    for (from, until) in [(30, Some(30)), (1, Some(10))] {
        let mut ledger = FaultLedger::new();
        inject(&mut ledger, SPENT_CREDITS, from, until, "a");
        let mut runs = Runs::new();
        runs.started(time(10), "run", "a");
        runs.ended(time(30), "run", "a", Ended::Failed);
        assert!(runs.judge(&mut ledger).is_err(), "touching interval {from}..{until:?} strikes");
    }
    // A run on another account, or with no intersection, is not blamed.
    for (account, from, until) in [("b", 10, Some(30)), ("a", 1, Some(9)), ("a", 31, None)] {
        let mut ledger = FaultLedger::new();
        inject(&mut ledger, SPENT_CREDITS, from, until, account);
        let mut runs = Runs::new();
        runs.started(time(10), "run", "a");
        runs.ended(time(30), "run", "a", Ended::Failed);
        runs.judge(&mut ledger).expect("unrelated fault cannot blame the run");
    }
}

#[test]
fn unjudged_kinds_are_counted_and_can_be_the_latest_cause() {
    for kind in [
        CHECK_WAITS,
        Kind { family: "provider", name: "overloaded" },
        Kind { family: "provider", name: "unavailable" },
        Kind { family: "elsewhere", name: "rate window" },
    ] {
        let mut ledger = FaultLedger::new();
        inject(&mut ledger, SPENT_CREDITS, 1, Some(5), "a");
        let fault = inject(&mut ledger, kind, 10, None, "a");
        let mut runs = Runs::new();
        runs.started(time(1), "run", "a");
        runs.retried(time(12), "a", "call", Retried::Session);
        runs.retried(time(13), "a", "call", Retried::Program);
        runs.ended(time(20), "run", "a", Ended::Failed);
        runs.judge(&mut ledger).expect("latest cause has no run-blame row");
        assert_eq!(ledger.get(fault).count(Counter::SessionRetries), 1);
        assert_eq!(ledger.get(fault).count(Counter::ProviderRetries), 1);
    }
}
