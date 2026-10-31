use super::*;

#[test]
fn a_snapshot_rounds_used_time_up_to_a_whole_millisecond() {
    for (nanos, millis) in [(0, 0), (1, 1), (1_000_000, 1), (2_000_000_500, 2_001)] {
        let mut harness = Harness::new(LIMITS);
        let (_, main) = harness.running_on(1, 100, agents());
        harness.after(Duration::from_nanos(nanos));
        drop(harness.step(Event::Used { conversation: main, spend: spend(50) }));
        let mut snapshots = List::with_capacity(2);
        for fact in &facts(&mut harness) {
            if let Fact::Budget { standing, .. } = fact {
                snapshots.push(standing.time).expect("two slots admit any duplicate");
            }
        }
        assert_eq!(snapshots.as_slice(), &[Duration::from_millis(millis)]);
    }
}

#[test]
fn reaching_the_scalar_cap_settles_wait_and_fails_only_after_yield() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running(1, 100);
    assert!(harness.step(priced(main, BUDGET.spend, BUDGET.spend)).is_empty());
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Denied(Exhausted::Spend)
    );
    let wait = Event::Delegated {
        conversation: main,
        call: Token::new(91),
        name: crate::CallName { activation: 1, completion: 1, position: 0 },
        ask: Ask::Wait,
        deadline: EXPIRY,
    };
    assert_eq!(harness.step(wait).as_ref(), &[returned(91, Returned::Refused { refusal: AskRefusal::Over })]);
    assert_eq!(harness.step(end_turn(main)).as_ref(), &[Request::Close { peer: PEER }]);
    let answer = answered(harness.step(ended(main, End::Closed, Spend::ZERO))).1;
    assert_eq!(
        answer,
        failed(
            Failure::Budget { exhausted: Exhausted::Spend, used: Some(BUDGET.spend), bound: Some(BUDGET.spend) },
            Spend { units: BUDGET.spend, ..Spend::ZERO }
        )
    );
}

#[test]
fn raw_run_counts_keep_absence_distinct_from_reported_zero_across_gaps() {
    let zero = Spend { turns: 1, input: Some(0), reasoning: Some(0), ..Spend::ZERO };
    assert_eq!(Spend::ZERO.accumulate(zero), Some(zero));
    let gap = Spend { turns: 1, ..Spend::ZERO };
    let known = zero.accumulate(gap).expect("one gap has no reported counts");
    assert_eq!(known, Spend { turns: 2, ..zero });
    let next = Spend { turns: 1, output: Some(3), cache_write: Some(2), reasoning: Some(1), ..Spend::ZERO };
    let total = known.accumulate(next).expect("reported fields remain separate");
    assert_eq!(
        total,
        Spend {
            turns: 3,
            input: Some(0),
            output: Some(3),
            cache_read: None,
            cache_write: Some(2),
            reasoning: Some(1),
            units: 0,
        }
    );
    assert_eq!(
        total.unreported(known),
        Spend {
            turns: 1,
            input: Some(0),
            output: Some(3),
            cache_read: None,
            cache_write: Some(2),
            reasoning: Some(1),
            units: 0,
        }
    );
    assert_eq!(Spend::ZERO.accumulate(gap), Some(gap), "no reported count stays absent");
}

#[test]
fn checked_spend_addition_rejects_the_whole_increment() {
    let known = Spend { turns: 2, input: Some(4), output: Some(u64::MAX), units: 10, ..Spend::ZERO };
    let next = Spend { turns: 1, input: Some(3), output: Some(1), units: 11, ..Spend::ZERO };
    assert_eq!(known.accumulate(next), None);
    let known = Spend { units: u64::MAX, ..Spend::ZERO };
    assert_eq!(known.accumulate(next), None);
    let known = Spend { reasoning: Some(u64::MAX), ..Spend::ZERO };
    assert_eq!(known.accumulate(Spend { reasoning: Some(1), ..Spend::ZERO }), None);
}

#[test]
fn completion_gate_distinguishes_child_withdrawal_from_budget_denial() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    assert_eq!(
        crate::completion_permit(&mut harness.domain, child, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Allowed
    );
    assert_eq!(
        harness.step(Event::Withdraw { conversation: main, call: Token::new(7) }).as_ref(),
        &[Request::Close { peer: Token::new(101) }]
    );
    assert_eq!(
        crate::completion_permit(&mut harness.domain, child, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Closing
    );
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Allowed
    );
    assert_eq!(harness.step(ended(child, End::Closed, Spend::ZERO)).as_ref(), &[returned(7, Returned::Cancelled)]);
}

#[test]
fn run_wide_overflow_preflight_preserves_charged_prefix() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running(1, 100);
    assert!(harness.step(priced(main, u64::MAX, u64::MAX)).is_empty());
    assert_eq!(
        crate::completion_overflow(&harness.domain, main, 1, Spend::ZERO),
        Some(Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Spend), used: None, bound: None })
    );
    assert_eq!(crate::completion_overflow(&harness.domain, main, 0, Spend { turns: 1, ..Spend::ZERO }), None);
    let answer = answered(harness.step(ended(main, End::PriceOverflow, Spend::ZERO))).1;
    assert_eq!(
        answer,
        failed(
            Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Spend), used: None, bound: None },
            Spend { units: u64::MAX, ..Spend::ZERO }
        )
    );
}

#[test]
fn raw_usage_overflow_ends_with_only_the_charged_prefix() {
    for (full, overflow) in [
        (Spend { output: Some(u64::MAX), ..Spend::ZERO }, Spend { output: Some(1), ..Spend::ZERO }),
        (Spend { reasoning: Some(u64::MAX), ..Spend::ZERO }, Spend { reasoning: Some(1), ..Spend::ZERO }),
    ] {
        let mut harness = Harness::new(LIMITS);
        let (_, main) = harness.running(1, 100);
        assert!(harness.step(Event::Used { conversation: main, spend: full }).is_empty());
        assert_eq!(
            crate::completion_overflow(&harness.domain, main, 0, overflow),
            Some(Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Usage), used: None, bound: None })
        );
        assert_eq!(
            harness.step(Event::Used { conversation: main, spend: overflow }).as_ref(),
            &[Request::Close { peer: PEER }]
        );
        let answer = answered(harness.step(ended(main, End::Closed, full))).1;
        assert_eq!(
            answer,
            failed(
                Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Usage), used: None, bound: None },
                full
            )
        );
    }
}

#[test]
fn a_session_usage_overflow_becomes_the_run_failure() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running(1, 100);
    let answer = answered(harness.step(ended(main, End::UsageOverflow, Spend::ZERO))).1;
    assert_eq!(
        answer,
        failed(
            Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Usage), used: None, bound: None },
            Spend::ZERO
        )
    );
}

#[test]
fn final_turns_ignore_ordinary_nudges_and_fail_once_only_after_the_last() {
    let mut harness = Harness::new(Limits { nudges: 0, ..LIMITS });
    let mut selected = charter();
    selected.budget.turns = 4;
    selected.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(10) };
    let (_, main) = harness.running_on(1, 100, selected);
    drop(harness.step(Event::Used { conversation: main, spend: Spend { turns: 2, ..spend(1) } }));
    assert_eq!(
        harness.step(end_turn(main)).as_ref(),
        &[Request::Say { peer: PEER, text: crate::prompt::finish_now() }]
    );
    let first = crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0));
    let crate::CompletionPermit::Final { note: Some(note) } = first else {
        panic!("first final permit");
    };
    assert_eq!(note.as_ref(), b"The ordinary budget is spent. Finish now with what you have. You have at most 2 final completions and 10 seconds to finish. Use only finish, or deliver when offered.");
    for (owner, expected) in [
        (Token::new(500), Request::Say { peer: PEER, text: crate::prompt::finish_now() }),
        (Token::new(501), Request::Close { peer: PEER }),
    ] {
        assert_eq!(
            crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
            crate::CompletionPermit::Final { note: None }
        );
        assert_eq!(crate::reserve(&mut harness.domain, main, owner, crate::Most::Bounded(1)), Ok(()));
        assert!(crate::settle_reservation(&mut harness.domain, main, owner, 1));
        drop(harness.step(Event::Used { conversation: main, spend: spend(1) }));
        let requests = harness.step(end_turn(main));
        assert_eq!(requests.as_ref(), &[expected]);
    }
    let seen = Spend {
        turns: 4,
        input: Some(3),
        output: Some(30),
        cache_read: Some(0),
        cache_write: Some(0),
        reasoning: None,
        units: 3,
    };
    let (_, answer) = answered(harness.step(ended(main, End::Closed, seen)));
    assert_eq!(answer, failed(Failure::Budget { exhausted: Exhausted::Turns, used: Some(4), bound: Some(4) }, seen));
}

#[test]
fn a_report_accepted_in_main_final_work_remains_accepted_once() {
    let mut harness = Harness::new(LIMITS);
    let mut selected = text_charter(false);
    selected.budget.turns = 4;
    selected.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(10) };
    let (run, main) = harness.running_on(1, 100, selected);
    drop(harness.step(Event::Used { conversation: main, spend: Spend { turns: 2, ..Spend::ZERO } }));
    let permit = crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0));
    let crate::CompletionPermit::Final { note: Some(_) } = permit else {
        panic!("the accepted report is in final work");
    };
    assert_eq!(crate::reserve(&mut harness.domain, main, Token::new(500), crate::Most::Bounded(1)), Ok(()));
    assert!(crate::settle_reservation(&mut harness.domain, main, Token::new(500), 0));
    drop(harness.step(Event::Used { conversation: main, spend: Spend { turns: 1, ..Spend::ZERO } }));
    assert_eq!(
        harness.step(finish(main, 7, report())).as_ref(),
        &[returned(7, Returned::Accepted), Request::Close { peer: PEER }]
    );
    let seen = Spend { turns: 3, ..Spend::ZERO };
    assert_eq!(
        answered(harness.step(ended(main, End::Closed, seen))).1,
        Answer::Accepted { outcome: report(), spent: seen, turns: 0 }
    );
    assert!(harness.step(Event::Cancel { run }).is_empty(), "no second answer after acceptance");
}

#[test]
fn ordinary_time_exhaustion_parks_a_wait_and_withholds_its_seen_call() {
    let mut harness = Harness::new(Limits { waiting: Duration::from_secs(600), ..LIMITS });
    let mut selected = charter();
    selected.grants.wait = true;
    selected.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(60) };
    selected.waiting = Duration::from_secs(600);
    let (_, main) = harness.running_on(1, 100, selected);
    assert_eq!(
        harness
            .step(Event::Delegated {
                conversation: main,
                call: Token::new(91),
                name: crate::CallName { activation: 1, completion: 1, position: 0 },
                ask: Ask::Wait,
                deadline: EXPIRY
            })
            .as_ref(),
        &[Request::Waiting { host_run: Token::new(1), read: None }]
    );
    harness.env.now = Time::ZERO.saturating_add(Duration::from_secs(540));
    assert_eq!(harness.fire().as_ref(), &[returned(91, Returned::Withheld), Request::Close { peer: PEER }]);
    let (_, answer) = answered(harness.step(ended(main, End::Closed, Spend::ZERO)));
    assert_eq!(answer, Answer::Parked { spent: Spend::ZERO, turns: 0 });
}

#[test]
fn the_completion_maximum_prices_the_full_window_and_rounds_each_reserved_turn_once() {
    let main = Llm {
        window: 7,
        output: 4,
        prices: crate::Prices { input: 2, cached: 5, cache_write: Some(3), output: 3, unit: 10 },
        ..charter().llm
    };
    assert_eq!(crate::most_costly(&main), Some(5));
    let reserve = crate::charter::Reserve { turns: 3, time: Duration::from_secs(1) };
    assert_eq!(crate::reserve_spend(reserve, &main), Some(15));
    let tiny = Llm { window: 1, output: 1, prices: crate::Prices { unit: 3, ..charter().llm.prices }, ..main.clone() };
    assert_eq!(crate::most_costly(&tiny), Some(1));
    assert_eq!(crate::reserve_spend(reserve, &tiny), Some(3));
    let written = Llm { prices: crate::Prices { cache_write: Some(8), ..main.prices }, ..main.clone() };
    assert_eq!(crate::most_costly(&written), Some(7));
}

#[test]
fn completion_and_reserve_derivation_refuse_unrepresentable_host_units() {
    let main = Llm {
        window: 1,
        output: 1,
        prices: crate::Prices { input: u64::MAX - 1, cached: 1, cache_write: None, output: 1, unit: 1 },
        ..charter().llm
    };
    assert_eq!(crate::most_costly(&main), Some(u64::MAX));
    let reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(1) };
    assert_eq!(crate::reserve_spend(reserve, &main), None);
    assert_eq!(crate::most_costly(&Llm { window: 2, ..main.clone() }), None);
    assert_eq!(crate::most_costly(&Llm { prices: crate::Prices { unit: 0, ..main.prices }, ..main }), None);
}

#[test]
fn entrance_names_spend_for_a_completion_or_an_impossible_finishing_reserve() {
    let main = Llm {
        window: 7,
        output: 4,
        prices: crate::Prices { input: 2, cached: 5, cache_write: Some(3), output: 3, unit: 10 },
        ..charter().llm
    };
    let reserve = crate::charter::Reserve { turns: 3, time: Duration::from_secs(1) };
    assert_eq!(
        crate::admit(&Budget { spend: 4, ..BUDGET }, reserve, &main, &LIMITS.budget, None),
        Err(crate::BudgetRefusal::Completion { most: 5 })
    );
    assert_eq!(
        crate::admit(&Budget { spend: 15, ..BUDGET }, reserve, &main, &LIMITS.budget, None),
        Err(crate::BudgetRefusal::Reserve(crate::Part::Spend))
    );
    assert_eq!(crate::admit(&Budget { spend: 16, ..BUDGET }, reserve, &main, &LIMITS.budget, None), Ok(()));
}

#[test]
fn spend_above_the_reserve_runs_out_then_main_finishes_and_is_accepted() {
    let mut harness = Harness::new(LIMITS);
    let mut selected = text_charter(false);
    selected.budget.spend = 3;
    selected.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(10) };
    let (run, main) = harness.running_on(1, 100, selected);
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(1)),
        crate::CompletionPermit::Allowed
    );
    assert_eq!(crate::reserve(&mut harness.domain, main, Token::new(500), crate::Most::Bounded(1)), Ok(()));
    assert!(crate::settle_reservation(&mut harness.domain, main, Token::new(500), 1));
    drop(harness.step(Event::Used { conversation: main, spend: spend(1) }));
    let permit = crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(1));
    let crate::CompletionPermit::Final { note: Some(_) } = permit else { panic!("spend starts final work") };
    assert_eq!(crate::reserve(&mut harness.domain, main, Token::new(501), crate::Most::Bounded(1)), Ok(()));
    assert!(crate::settle_reservation(&mut harness.domain, main, Token::new(501), 1));
    drop(harness.step(Event::Used { conversation: main, spend: spend(1) }));
    assert_eq!(
        harness.step(finish(main, 7, report())).as_ref(),
        &[returned(7, Returned::Accepted), Request::Close { peer: PEER }]
    );
    let seen = spend(1).accumulate(spend(1)).expect("two bounded charges");
    assert_eq!(
        answered(harness.step(ended(main, End::Closed, seen))).1,
        Answer::Accepted { outcome: report(), spent: seen, turns: 0 }
    );
    assert!(harness.step(Event::Cancel { run }).is_empty());
}

#[test]
fn no_final_turns_keeps_the_spend_trigger_when_the_next_maximum_cannot_fit() {
    let mut harness = Harness::new(LIMITS);
    let mut selected = charter();
    selected.budget.spend = 10;
    selected.llm.prices.unit = 1000;
    let (_, main) = harness.running_on(1, 100, selected);
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(9)),
        crate::CompletionPermit::Allowed
    );
    assert_eq!(crate::reserve(&mut harness.domain, main, Token::new(500), crate::Most::Bounded(9)), Ok(()));
    assert!(crate::settle_reservation(&mut harness.domain, main, Token::new(500), 2));
    drop(harness.step(Event::Used { conversation: main, spend: spend(2) }));
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(9)),
        crate::CompletionPermit::Denied(Exhausted::Spend)
    );
}

#[test]
fn a_child_overflow_cancels_run_alarms_while_main_closes() {
    for end in [End::PriceOverflow, End::UsageOverflow] {
        let mut harness = Harness::new(LIMITS);
        let (_, main) = harness.running_on(1, 100, agents());
        let (child, _) = harness.child(main, 7, families(true, false, false), 101);
        let requests = harness.step(ended(child, end, Spend::ZERO));
        assert!(requests.contains(&Request::Close { peer: PEER }), "child overflow closes main");
        harness.after(BUDGET.time);
        fire(&mut harness.domain, &harness.env, &mut harness.out);
        assert!(harness.drain().is_empty(), "closing main owes its terminal, not an alarm");
        assert!(!harness.domain.is_due(harness.env.now));
    }
}
