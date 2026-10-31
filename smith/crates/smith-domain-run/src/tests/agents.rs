use super::*;

#[test]
fn a_child_closed_after_its_last_charge_tells_the_budget_of_its_return() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    drop(harness.step(Event::Used { conversation: child, spend: spend(50) }));
    drop(harness.step(Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"Found.") }));
    harness.after(Duration::from_nanos(2_000_000_500));
    let requests = harness.step(ended(child, End::Closed, spend(50)));
    let [Request::Return { result: Returned::Answered { left, .. }, .. }] = &*requests else {
        panic!("expected the child's answer, got {requests:?}");
    };
    assert_eq!(*left, Budget { turns: 9, spend: 9_950, time: Duration::from_millis(597_999) });
    let mut told = List::with_capacity(2);
    for fact in &facts(&mut harness) {
        if let Fact::Budget { standing, call: Some(call), .. } = fact {
            told.push((*call, standing.time)).expect("two slots admit any duplicate");
        }
    }
    assert_eq!(told.as_slice(), &[(Token::new(7), Duration::from_millis(2_001))], "one snapshot names the call");
}

#[test]
fn a_childs_return_pushes_one_budget_even_with_its_stage_change() {
    let mut harness = Harness::new(LIMITS);
    let mut charter = agents();
    charter.reserve.turns = 2;
    let (_, main) = harness.running_on(1, 100, charter);
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    drop(harness.step(Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"Found.") }));
    drop(facts(&mut harness));
    drop(harness.step(Event::Ended {
        conversation: child,
        end: End::Closed,
        spend: Spend { turns: 8, ..spend(50) },
        detail: Box::default(),
    }));
    let mut snapshots = List::with_capacity(2);
    for fact in &facts(&mut harness) {
        if let Fact::Budget { standing, call, .. } = fact {
            snapshots.push((standing.stage, *call)).expect("two slots admit any duplicate");
        }
    }
    assert_eq!(snapshots.as_slice(), &[(crate::Stage::WindDown, Some(Token::new(7)))]);
}

#[test]
fn a_sub_agent_answers_with_its_last_message_once_it_has_ended() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    drop(harness.step(Event::Used { conversation: main, spend: spend(100) }));
    let (child, opening) = harness.child(main, 7, families(true, false, false), 101);
    assert_eq!((&opening.llm, opening.finish, opening.families), (&agents().llm, false, families(true, false, false)));
    let left = Budget { turns: BUDGET.turns - 1, spend: BUDGET.spend - 100, ..BUDGET };
    assert_eq!(opening.budget, left, "its share is what the run has left");
    assert!(opening.system.starts_with(b"Find the parser."), "its brief is its asker's");
    drop(harness.step(Event::Used { conversation: child, spend: spend(50) }));
    let yielded =
        Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"The parser is in src/parse.rs.") };
    assert_eq!(&*harness.step(yielded), &[Request::Close { peer: Token::new(101) }], "a sub-agent that yields is done");
    let requests = harness.step(ended(child, End::Closed, spend(50)));
    let answer = Returned::Answered {
        text: bytes(b"The parser is in"),
        cut: 14,
        stop: Stop::EndTurn,
        used: crate::facts::Used { turns: 1, time: Duration::ZERO, spend: 50 },
        left: Budget { turns: 8, spend: 9850, ..BUDGET },
    };
    assert_eq!(
        &*requests,
        &[Request::Return { call: Token::new(7), result: answer, spent: 50 }],
        "its answer is cut at the limit and carries the exact priced subtree bill"
    );
    harness.domain.reclaim();
    assert_eq!((harness.domain.conversations(), harness.domain.calls()), (1, 0));
    let mut returned = 0_u32;
    while let Some(fact) = harness.pop_fact() {
        if let crate::facts::FactKind::Returned { call, result: Return::Answered { cut, .. }, .. } = fact.kind
            && call == Token::new(7)
        {
            assert_eq!(cut, 14, "child cap omission");
            returned += 1;
        }
    }
    assert_eq!(returned, 1, "the child cap reaches its fact once");
}

#[test]
fn writing_children_hold_the_lease_until_their_seen_end_even_after_withdrawal() {
    for tools in
        [Tools { inspect: true, modify: true, shell: false }, Tools { inspect: true, modify: false, shell: true }]
    {
        let mut harness = Harness::new(LIMITS);
        let mut charter = agents();
        charter.grants.tools.modify = true;
        let (run, main) = harness.running_on(1, 100, charter);
        let run = skein_lib::Id::from_token(run);
        assert_eq!(harness.domain.runs.get(run).unwrap().lease, crate::lease::Lease::Free);
        let (child, _) = harness.child(main, 7, crate::charter::Families { tools, agents: false }, 101);
        let call = harness.domain.calls.find(skein_lib::Id::from_token(main), Token::new(7)).unwrap();
        let held = crate::lease::Lease::Child { call };
        assert_eq!(harness.domain.runs.get(run).unwrap().lease, held);
        assert_eq!(
            harness.step(Event::Withdraw { conversation: main, call: Token::new(7) }).as_ref(),
            &[Request::Close { peer: Token::new(101) }]
        );
        assert_eq!(harness.domain.runs.get(run).unwrap().lease, held);
        assert_eq!(harness.step(ended(child, End::Closed, Spend::ZERO)).len(), 1);
        assert_eq!(harness.domain.runs.get(run).unwrap().lease, crate::lease::Lease::Free);
    }
}

#[test]
fn inspect_only_children_keep_the_lease_free_while_their_calls_overlap() {
    let mut harness = Harness::new(LIMITS);
    let (run, main) = harness.running_on(1, 100, agents());
    let run = skein_lib::Id::from_token(run);
    let (first, _) = harness.child(main, 7, families(true, false, false), 101);
    let (second, _) = harness.child(main, 8, families(true, false, false), 102);
    assert_eq!(harness.domain.calls(), 2);
    assert_eq!(harness.domain.runs.get(run).unwrap().lease, crate::lease::Lease::Free);
    for child in [second, first] {
        drop(harness.step(Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"found") }));
        drop(harness.step(ended(child, End::Closed, Spend::ZERO)));
        assert_eq!(harness.domain.runs.get(run).unwrap().lease, crate::lease::Lease::Free);
    }
}

#[test]
#[should_panic(expected = "one write holder over overlapping directories")]
fn opening_a_second_writing_child_is_a_broken_schedule() {
    let mut harness = Harness::new(LIMITS);
    let mut charter = agents();
    charter.grants.tools.modify = true;
    let (_, main) = harness.running_on(1, 100, charter);
    drop(harness.child(main, 7, families(true, true, false), 101));
    drop(harness.child(
        main,
        8,
        crate::charter::Families { tools: Tools { inspect: true, modify: false, shell: true }, agents: false },
        102,
    ));
}

#[test]
fn simultaneous_sessions_reserve_one_run_budget_and_return_unused_credit() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    let first = Token::new(801);
    let second = Token::new(802);

    assert_eq!(crate::reserve(&mut harness.domain, main, first, crate::Most::Bounded(6_000)), Ok(()));
    assert_eq!(crate::reserve(&mut harness.domain, child, second, crate::Most::Bounded(5_000)), Err(Exhausted::Spend));
    assert!(crate::settle_reservation(&mut harness.domain, main, first, 2_000));
    drop(harness.step(Event::Priced { conversation: main, own_spent: 2_000, subtree_spent: 2_000 }));
    assert_eq!(crate::reserve(&mut harness.domain, child, second, crate::Most::Bounded(5_000)), Ok(()));
    assert!(!crate::settle_reservation(&mut harness.domain, child, second, 5_001));
    assert_eq!(crate::reserve(&mut harness.domain, child, Token::new(803), crate::Most::Bounded(8_000)), Ok(()));
}

#[test]
fn a_sub_agent_runs_on_the_llm_named_for_it_among_the_charters() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let requests = harness.step(ask(main, 7, families(true, false, false), Some(bytes(b"model-b")), None));
    let [Request::Open { opening, .. }] = &*requests else { panic!("expected the sub-agent to open") };
    assert_eq!(&opening.llm, &agents().models[0]);
    let share = crate::Share { turns: Some(2), time: Some(Duration::from_secs(30)) };
    let requests = harness.step(ask(main, 8, families(true, false, false), None, Some(share)));
    let [Request::Open { opening, .. }] = &*requests else { panic!("expected the sub-agent to open") };
    let budget = Budget { turns: 2, spend: BUDGET.spend, time: Duration::from_secs(30) };
    assert_eq!(opening.budget, budget, "a share asked for, no larger than what is left");
}

#[test]
fn a_child_with_no_requested_share_gets_only_the_ordinary_pool() {
    let mut harness = Harness::new(LIMITS);
    let mut charter = agents();
    charter.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(60) };
    let (_, main) = harness.running_on(1, 100, charter);
    drop(harness.step(Event::Used { conversation: main, spend: spend(100) }));
    let (_, opening) = harness.child(main, 7, families(true, false, false), 101);
    assert_eq!(opening.budget, Budget { turns: 7, spend: BUDGET.spend - 102, time: Duration::from_secs(540) });
}

#[test]
fn asked_child_parts_are_clamped_independently_and_name_the_cause() {
    use crate::{Granted, Set, Share};

    let mut charter = agents();
    charter.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(60) };
    let ordinary = Duration::from_secs(540);
    for (asked, call_left, turns, time, turns_set, time_set) in [
        (Share::UNASKED, ordinary, 8, ordinary, Set::Pool, Set::Pool),
        (
            Share { turns: Some(3), time: Some(Duration::from_secs(4)) },
            ordinary,
            3,
            Duration::from_secs(4),
            Set::Asked,
            Set::Asked,
        ),
        (Share { turns: Some(8), time: Some(ordinary) }, ordinary, 8, ordinary, Set::Asked, Set::Asked),
        (Share { turns: Some(u32::MAX), time: Some(BUDGET.time) }, ordinary, 8, ordinary, Set::Reserve, Set::Reserve),
        (Share { turns: Some(3), time: None }, ordinary, 3, ordinary, Set::Asked, Set::Pool),
        (
            Share { turns: None, time: Some(Duration::from_secs(4)) },
            ordinary,
            8,
            Duration::from_secs(4),
            Set::Pool,
            Set::Asked,
        ),
        (Share::UNASKED, Duration::from_secs(1), 8, Duration::from_secs(1), Set::Pool, Set::Deadline),
        (
            Share { turns: Some(3), time: Some(Duration::from_secs(4)) },
            Duration::from_secs(1),
            3,
            Duration::from_secs(1),
            Set::Asked,
            Set::Deadline,
        ),
    ] {
        let plan = crate::agent::plan(
            &charter,
            crate::agent::Means { spent: Spend::ZERO, conversations: 1, left: BUDGET.time, call_left },
            families(true, false, true),
            families(true, false, false),
            None,
            asked,
            &LIMITS,
        )
        .expect("positive ordinary pool");
        assert_eq!(plan.granted, Granted { turns, time, spend: BUDGET.spend - 2, turns_set, time_set });
        assert_eq!(plan.budget, Budget { turns, time, spend: BUDGET.spend - 2 });
    }
}

#[test]
fn the_entrance_names_a_reserve_that_its_budget_cannot_hold() {
    for (reserve, part) in [
        (crate::charter::Reserve { turns: BUDGET.turns, time: Duration::ZERO }, crate::Part::Turns),
        (crate::charter::Reserve { turns: 0, time: BUDGET.time }, crate::Part::Time),
    ] {
        let mut charter = charter();
        charter.reserve = reserve;
        assert_eq!(
            answered(Harness::new(LIMITS).start(7, charter)),
            (7, Answer::Refused(Refusal::Invalid(Invalid::Budget(crate::BudgetRefusal::Reserve(part)))))
        );
    }
}

#[test]
fn change_contracts_reserve_the_checks_and_other_contracts_need_no_check_allowance() {
    let mut policy = coding();
    let reserve = Duration::from_nanos(LIMITS.check_timeout.as_nanos() - 1_000_000);
    policy.reserve.time = reserve;
    assert_eq!(
        answered(Harness::new(LIMITS).start(7, policy)),
        (
            7,
            Answer::Refused(Refusal::Invalid(Invalid::Budget(crate::BudgetRefusal::Checks {
                reserve,
                checks: LIMITS.check_timeout
            })))
        )
    );
    let equal = Harness::new(LIMITS).start(7, coding());
    let [Request::Admitted { .. }, ..] = equal.as_ref() else { panic!("equal checks reserve: {equal:?}") };
    let mut policy = charter();
    policy.reserve.time = Duration::from_millis(1);
    let report = Harness::new(LIMITS).start(7, policy);
    let [Request::Admitted { .. }, ..] = report.as_ref() else { panic!("report reserve: {report:?}") };
}

#[test]
fn pending_children_cannot_reserve_the_same_last_ordinary_turn() {
    let mut harness = Harness::new(LIMITS);
    let mut charter = agents();
    charter.budget.turns = 3;
    charter.reserve = crate::charter::Reserve { turns: 1, time: Duration::ZERO };
    let (_, main) = harness.running_on(1, 100, charter);
    drop(harness.step(Event::Used { conversation: main, spend: spend(1) }));
    let (first, _) = harness.child(main, 7, families(true, false, false), 101);
    let (second, _) = harness.child(main, 8, families(true, false, false), 102);
    let owner = Token::new(701);
    assert_eq!(crate::reserve(&mut harness.domain, first, owner, crate::Most::Bounded(1)), Ok(()));
    assert_eq!(
        crate::completion_permit(&mut harness.domain, second, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Closing
    );
    assert_eq!(
        crate::reserve(&mut harness.domain, second, Token::new(702), crate::Most::Bounded(1)),
        Err(Exhausted::Turns)
    );
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Held
    );
    assert!(crate::settle_reservation(&mut harness.domain, first, owner, 1));
    drop(harness.step(Event::Used { conversation: first, spend: spend(1) }));
    assert_eq!(harness.resume().as_ref(), &[Request::Close { peer: Token::new(101) }]);
    assert_eq!(harness.resume().as_ref(), &[Request::Close { peer: Token::new(102) }]);
    let second_return = harness.step(ended(second, End::Closed, Spend::ZERO));
    let [Request::Return { call, result: Returned::Unanswered { end, .. }, .. }] = second_return.as_ref() else {
        panic!("one settled child return: {second_return:?}");
    };
    assert_eq!((*call, end.clone()), (Token::new(8), End::Budget(Exhausted::Turns)));
    let first_return = harness.step(ended(first, End::Closed, spend(1)));
    let [
        Request::Return { call, result: Returned::Unanswered { end, .. }, .. },
        Request::CompletionReady { conversation },
    ] = first_return.as_ref()
    else {
        panic!("last child releases main: {first_return:?}");
    };
    assert_eq!((*call, end.clone(), *conversation), (Token::new(7), End::Budget(Exhausted::Turns), main));
    let note = crate::prompt::wind_down_note(
        crate::Mode::Finite,
        crate::wind_down::Cause::Budget(Exhausted::Turns),
        crate::charter::Reserve { turns: 1, time: Duration::ZERO },
    );
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Final { note: Some(note) }
    );
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Final { note: None }
    );
    assert_eq!(crate::reserve(&mut harness.domain, main, Token::new(703), crate::Most::Bounded(1)), Ok(()));
    assert!(crate::settle_reservation(&mut harness.domain, main, Token::new(703), 1));
    drop(harness.step(Event::Used { conversation: main, spend: spend(1) }));
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Denied(Exhausted::Turns)
    );
}

#[test]
fn ordinary_time_closes_a_child_then_main_receives_its_final_permit() {
    let mut harness = Harness::new(LIMITS);
    let mut charter = agents();
    charter.reserve = crate::charter::Reserve { turns: 2, time: Duration::from_secs(60) };
    let (_, main) = harness.running_on(1, 100, charter);
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    harness.env.now = Time::ZERO.saturating_add(Duration::from_secs(540));
    assert_eq!(harness.fire().as_ref(), &[]);
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Held
    );
    assert_eq!(harness.resume().as_ref(), &[Request::Close { peer: Token::new(101) }]);
    let returned = harness.step(ended(child, End::Closed, Spend::ZERO));
    let [Request::Return { result: Returned::Unanswered { end, .. }, .. }, Request::CompletionReady { conversation }] =
        returned.as_ref()
    else {
        panic!("last child releases main: {returned:?}");
    };
    assert_eq!((end.clone(), *conversation), (End::Budget(Exhausted::Time), main));
    let note = crate::prompt::wind_down_note(
        crate::Mode::Finite,
        crate::wind_down::Cause::Budget(Exhausted::Turns),
        crate::charter::Reserve { turns: 2, time: Duration::from_secs(60) },
    );
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Final { note: Some(note) }
    );
    harness.env.now = EXPIRY;
    assert_eq!(
        crate::completion_permit(&mut harness.domain, main, harness.env.now, crate::Most::Bounded(0)),
        crate::CompletionPermit::Denied(Exhausted::Time)
    );
}

#[test]
fn an_ask_the_run_cannot_grant_returns_why_and_the_run_goes_on() {
    // Not granted sub-agents, or wider families than its own.
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, charter());
    assert_eq!(
        &*harness.step(ask(main, 7, families(true, false, false), None, None)),
        &[returned(7, refused(AskRefusal::NotGranted))]
    );
    let (_, main) = harness.running_on(2, 101, agents());
    assert_eq!(
        &*harness.step(ask(main, 8, families(true, true, false), None, None)),
        &[returned(8, refused(AskRefusal::NotGranted))]
    );
    // An LLM the charter does not list.
    assert_eq!(
        &*harness.step(ask(main, 9, families(true, false, false), Some(bytes(b"model-z")), None)),
        &[returned(9, refused(AskRefusal::UnknownLlm))]
    );
    // A share with no turn in it.
    let none = crate::Share { turns: Some(0), time: None };
    assert_eq!(
        &*harness.step(ask(main, 10, families(true, false, false), None, Some(none))),
        &[returned(10, refused(AskRefusal::Unworkable))]
    );

    // Too many.
    let mut harness = Harness::new(Limits { run_conversations: 2, ..LIMITS });
    let (_, main) = harness.running_on(1, 100, agents());
    drop(harness.child(main, 7, families(true, false, false), 101));
    assert_eq!(
        &*harness.step(ask(main, 8, families(true, false, false), None, None)),
        &[returned(8, refused(AskRefusal::TooMany))]
    );

    // No room for the call or the conversation.
    let mut harness = Harness::new(Limits { calls: 1, run_conversations: 3, ..LIMITS });
    let (_, main) = harness.running_on(1, 100, agents());
    drop(harness.child(main, 7, families(true, false, false), 101));
    assert_eq!(&*harness.step(ask(main, 8, families(true, false, false), None, None)), &[returned(8, Returned::Busy)]);

    // Past the budget, new sub-agent work is refused as over.
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    drop(harness.step(Event::Used { conversation: main, spend: spend(BUDGET.spend + 1) }));
    assert_eq!(
        &*harness.step(ask(main, 7, families(true, false, false), None, None)),
        &[returned(7, refused(AskRefusal::Over))]
    );
}

#[test]
fn a_sub_agent_that_ends_without_answering_returns_how_it_ended() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    let requests = harness.step(ended(child, End::Fault(Fault::Provider), Spend::ZERO));
    assert_eq!(
        &*requests,
        &[returned(
            7,
            Returned::Unanswered {
                end: End::Fault(Fault::Provider),
                used: crate::facts::Used { turns: 0, time: Duration::ZERO, spend: 0 },
                left: BUDGET
            }
        )]
    );
    assert_eq!(harness.step(end_turn(main)).len(), 1, "the run goes on: main is nudged");
}

#[test]
fn a_withdrawn_sub_agent_is_closed_and_its_call_returns_once_it_has_ended() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    assert_eq!(
        &*harness.step(Event::Withdraw { conversation: main, call: Token::new(7) }),
        &[Request::Close { peer: Token::new(101) }]
    );
    // Its answer crossed the close: the call is cancelled all the same.
    let yielded = Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"late") };
    assert!(harness.step(yielded).is_empty(), "closing already");
    let requests = harness.step(ended(child, End::Closed, Spend::ZERO));
    assert_eq!(&*requests, &[returned(7, Returned::Cancelled)]);
    harness.domain.reclaim();

    // One withdrawn before it starts is closed once it does.
    let requests = harness.step(ask(main, 8, families(true, false, false), None, None));
    let [Request::Open { conversation: child, .. }] = &*requests else { panic!("expected the sub-agent to open") };
    let child = *child;
    assert!(harness.step(Event::Withdraw { conversation: main, call: Token::new(8) }).is_empty(), "not started yet");
    assert_eq!(
        &*harness.step(Event::Started { conversation: child, peer: Token::new(102) }),
        &[Request::Close { peer: Token::new(102) }]
    );
    let requests = harness.step(ended(child, End::Closed, Spend::ZERO));
    assert_eq!(&*requests, &[returned(8, Returned::Cancelled)]);
}

#[test]
fn a_sub_agent_past_its_deadline_is_closed_and_returns_timed_out_once_it_has_ended() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let minute = Duration::from_secs(60);
    let requests = harness.step(ask_by(main, 7, Time::ZERO.saturating_add(minute)));
    let [Request::Open { conversation: child, opening }] = &*requests else { panic!("expected the sub-agent to open") };
    assert_eq!(opening.budget.time, minute, "its time runs out by its call's deadline");
    let child = *child;
    drop(harness.step(Event::Started { conversation: child, peer: Token::new(101) }));
    harness.after(minute);
    assert_eq!(&*harness.fire(), &[Request::Close { peer: Token::new(101) }]);
    // Its answer crossed the close.
    let yielded = Event::Yielded { conversation: child, stop: Stop::EndTurn, text: bytes(b"late") };
    assert!(harness.step(yielded).is_empty(), "closing already");
    let requests = harness.step(ended(child, End::Closed, Spend::ZERO));
    assert_eq!(&*requests, &[returned(7, Returned::TimedOut)]);
    assert_eq!(harness.step(end_turn(main)).len(), 1, "the run goes on: main is nudged");
    harness.domain.reclaim();

    // One withdrawn first returns as cancelled, its deadline disarmed.
    let requests = harness.step(ask_by(main, 8, harness.env.now.saturating_add(minute)));
    let [Request::Open { conversation: child, .. }] = &*requests else { panic!("expected the sub-agent to open") };
    let child = *child;
    drop(harness.step(Event::Started { conversation: child, peer: Token::new(102) }));
    drop(harness.step(Event::Withdraw { conversation: main, call: Token::new(8) }));
    harness.after(minute);
    assert!(!harness.domain.is_due(harness.env.now), "a withdraw cancels its call's deadline");
    let requests = harness.step(ended(child, End::Closed, Spend::ZERO));
    assert_eq!(&*requests, &[returned(8, Returned::Cancelled)]);
}

#[test]
fn a_sub_agent_that_spends_past_the_budget_winds_the_run_down() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    let requests = harness.step(Event::Used { conversation: child, spend: spend(BUDGET.spend + 1) });
    assert!(requests.is_empty(), "the crossing child and main settle their admitted turn");
    drop(harness.step(Event::Withdraw { conversation: main, call: Token::new(7) }));
    drop(harness.step(ended(child, End::Closed, spend(BUDGET.spend + 1))));
    assert_eq!(&*harness.step(end_turn(main)), &[Request::Close { peer: PEER }]);
    let requests = harness.step(ended(main, End::Closed, Spend::ZERO));
    assert_eq!(
        answered(requests),
        (
            1,
            failed(
                Failure::Budget {
                    exhausted: Exhausted::Spend,
                    used: Some(BUDGET.spend + 1),
                    bound: Some(BUDGET.spend)
                },
                spend(BUDGET.spend + 1)
            )
        )
    );
}

#[test]
fn a_supplied_identity_reaches_main_and_children_with_ordinals_never_reused() {
    let identity = crate::Identity::new([0; 16]);
    let mut harness = Harness::new(LIMITS);
    let (_, main, opening) = identity_open(&mut harness, Some(identity));
    assert_eq!((opening.identity, opening.ordinal), (identity, 0));
    assert_eq!(identity.bytes(), [0; 16]);
    let (first, opening) = harness.child(main, 7, families(false, false, false), 101);
    assert_eq!((opening.identity, opening.ordinal), (identity, 1));
    drop(harness.step(ended(first, End::Closed, Spend::ZERO)));
    harness.domain.reclaim();
    let (_, second) = harness.child(main, 8, families(false, false, false), 102);
    assert_eq!((second.identity, second.ordinal), (identity, 2), "ended children never free their ordinal");
    assert_eq!(harness.domain.calls(), 1);
}

#[test]
fn absent_identities_draw_deterministically_only_for_admitted_unspecified_starts() {
    let mut first = Harness::new(LIMITS);
    first.domain = Domain::new(&LIMITS, 91);
    let (_, _, first_opening) = identity_open(&mut first, None);
    let mut same = Harness::new(LIMITS);
    same.domain = Domain::new(&LIMITS, 91);
    let mut invalid = charter();
    invalid.budget.turns = 0;
    let refused = same.step(carried_start(invalid, Box::default(), None));
    assert_eq!(refused.len(), 1);
    assert_eq!(same.domain.runs(), 0, "a refused start admits no run or identity");
    let (_, _, same_opening) = identity_open(&mut same, None);
    assert_eq!(first_opening.identity, same_opening.identity, "a refused start does not consume randomness");
    let mut supplied = Harness::new(LIMITS);
    supplied.domain = Domain::new(&LIMITS, 91);
    let (_, _, opening) = identity_open(&mut supplied, Some(crate::Identity::new([255; 16])));
    assert_eq!(opening.identity.bytes(), [255; 16]);
    let (_, _, opening) = identity_open(&mut supplied, None);
    assert_eq!(opening.identity, first_opening.identity, "a host identity consumes no admission randomness");
    let (_, _, next) = identity_open(&mut first, None);
    assert_ne!(next.identity, first_opening.identity, "successive admissions draw independent opaque bytes");
    let mut different = Harness::new(LIMITS);
    different.domain = Domain::new(&LIMITS, 92);
    assert_ne!(identity_open(&mut different, None).2.identity, first_opening.identity);
}

#[test]
fn child_wrap_up_is_an_exact_quarter_of_750ms_and_sixty_seconds_of_one_hour() {
    assert_eq!(
        crate::agent::final_allowance(Duration::from_millis(750), Duration::from_secs(60)),
        Duration::from_nanos(187_500_000)
    );
    assert_eq!(
        crate::agent::final_allowance(Duration::from_secs(3600), Duration::from_secs(60)),
        Duration::from_secs(60)
    );
    for (time, expected) in [
        (Duration::from_millis(750), Duration::from_nanos(187_500_000)),
        (Duration::from_secs(3600), Duration::from_secs(60)),
    ] {
        let mut harness = Harness::new(Limits {
            wrap_up: Duration::from_secs(60),
            budget: Budget { time: Duration::from_secs(7200), ..LIMITS.budget },
            ..LIMITS
        });
        let mut selected = agents();
        selected.budget.time = Duration::from_secs(7200);
        let (_, main) = harness.running_on(1, 100, selected);
        let mut request =
            ask(main, 7, families(true, false, false), None, Some(crate::Share { turns: Some(2), time: Some(time) }));
        let Event::Delegated { deadline, .. } = &mut request else { unreachable!() };
        *deadline = Time::ZERO.saturating_add(Duration::from_secs(7200));
        let requests = harness.step(request);
        let [Request::Open { opening, .. }] = requests.as_ref() else {
            panic!("child opening");
        };
        assert_eq!(opening.budget.time, time);
        let final_turn = opening.final_turn.as_ref().expect("child gets its final allowance");
        assert_eq!(final_turn.allowance, expected);
        assert_eq!(
            final_turn.note.as_ref(),
            b"Your share is ending. This is your final turn. Answer now with what you have. No tool calls will run."
        );
    }
}

#[test]
fn a_capped_child_answer_keeps_its_budget_stop_until_the_seen_end_and_returns_once() {
    let mut harness = Harness::new(LIMITS);
    let (_, main) = harness.running_on(1, 100, agents());
    let (child, _) = harness.child(main, 7, families(true, false, false), 101);
    drop(harness.step(Event::Used { conversation: child, spend: spend(50) }));
    assert_eq!(
        harness
            .step(Event::Yielded { conversation: child, stop: Stop::Budget, text: bytes(b"What I found.") })
            .as_ref(),
        &[Request::Close { peer: Token::new(101) }]
    );
    let requests = harness.step(ended(child, End::Closed, spend(50)));
    let [Request::Return { call, result: Returned::Answered { text, stop, used, .. }, spent }] = requests.as_ref()
    else {
        panic!("parent receives one answered terminal: {requests:?}");
    };
    assert_eq!(*call, Token::new(7));
    assert_eq!(text.as_ref(), b"What I found.");
    assert_eq!(*stop, Stop::Budget);
    assert_eq!(used.turns, 1);
    assert_eq!(*spent, 50);
    assert!(harness.step(ended(child, End::Closed, spend(50))).is_empty(), "stale end cannot answer twice");
}
