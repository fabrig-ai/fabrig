//! Checked receiving and spend admission.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::*;

#[test]
fn the_turn_that_crosses_a_budget_still_has_its_delegated_calls_answered() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open_with(1, budget(Budget { input: 9, ..BUDGET }));
    let content = Box::new([delegated(b"f", 9, Effect::Write)]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let end = h.step(answered(runs.as_slice()[0], 20, 3, false));
    assert_eq!(end, Some(ended(End::Budget { spent: Dimension::Input }, 1)));
}

#[test]
fn specs_beyond_the_limits_are_refused_as_invalid() {
    for (limits, spec) in [(Limits { history_bytes: 16, ..LIMITS }, spec()), (LIMITS, Spec { output: 0, ..spec() })] {
        let mut h = Harness::new(&limits);
        let refused = h.step(Event::Open { opener: Token::new(1), opening: opening(spec) });
        assert_eq!(refused, Some(ended(End::Invalid, 0)));
        assert_eq!(h.domain.sessions(), 0);
    }
}

#[test]
fn every_completion_is_reported_before_what_follows_it() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    step(&mut h.domain, &h.env, Event::Completed { owner, completion: reading() }, &mut h.out);
    assert_eq!(h.next(), Some(Request::Used { opener: Token::new(1), usage: USAGE }));
    let (run, _) = running(h.one());
    drop(calling(h.step(ran(run, b"main.rs"))));
    drop(yielded(h.step(Event::Completed { owner, completion: done() })));
    // The end adds up both completions; the harness tallied the second.
    assert_eq!(h.step(Event::Close { session: owner }), Some(ended(End::Closed, 2)));
    assert_eq!((h.turns, h.usage), (1, USAGE));
}

#[test]
fn a_spec_whose_budget_is_beyond_the_limits_is_refused() {
    let over = Duration::from_nanos(BUDGET.time.as_nanos() + 1);
    let budgets = [
        Budget { turns: BUDGET.turns + 1, ..BUDGET },
        Budget { input: BUDGET.input + 1, ..BUDGET },
        Budget { output: BUDGET.output + 1, ..BUDGET },
        Budget { cache_read: BUDGET.cache_read + 1, ..BUDGET },
        Budget { cache_write: BUDGET.cache_write + 1, ..BUDGET },
        Budget { time: over, ..BUDGET },
    ];
    for asked in budgets {
        let mut h = Harness::new(&LIMITS);
        let refused = h.step(Event::Open { opener: Token::new(1), opening: opening(budget(asked)) });
        assert_eq!(refused, Some(ended(End::Invalid, 0)), "{asked:?}");
    }
}

#[test]
fn a_session_whose_budget_leaves_no_room_for_a_completion_ends_as_it_opens() {
    let cases = [
        (Budget { turns: 0, ..BUDGET }, Dimension::Turns),
        (Budget { input: 0, ..BUDGET }, Dimension::Input),
        (Budget { output: 0, ..BUDGET }, Dimension::Output),
        (Budget { time: Duration::ZERO, ..BUDGET }, Dimension::Time),
    ];
    for (asked, spent) in cases {
        let mut h = Harness::new(&LIMITS);
        step(&mut h.domain, &h.env, Event::Open { opener: Token::new(1), opening: opening(budget(asked)) }, &mut h.out);
        let Some(Request::Opened { .. }) = h.next() else {
            panic!("{asked:?}: the session opens");
        };
        assert_eq!(h.one(), Some(ended(End::Budget { spent }, 0)), "{asked:?}");
    }
}

#[test]
fn the_llm_gets_as_many_turns_as_the_budget_allows() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open_with(1, budget(Budget { turns: 1, ..BUDGET }));
    // The tools of the last turn still run; their results do not go back.
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    let end = h.step(ran(run, b"main.rs"));
    assert_eq!(end, Some(ended(End::Budget { spent: Dimension::Turns }, 1)));

    h.domain.reclaim();
    let session = h.yielded_with(budget(Budget { turns: 1, ..BUDGET }), done());
    let end = h.step(Event::Continue { session, content: bytes(b"go on") });
    assert_eq!(end, Some(ended(End::Budget { spent: Dimension::Turns }, 1)));
}

#[test]
fn a_session_starts_no_completion_once_its_input_or_output_reach_their_budget() {
    let cases =
        [(Budget { input: 10, ..BUDGET }, Dimension::Input), (Budget { output: 5, ..BUDGET }, Dimension::Output)];
    for (asked, spent) in cases {
        let mut h = Harness::new(&LIMITS);
        let session = h.yielded_with(budget(asked), done());
        let end = h.step(Event::Continue { session, content: bytes(b"go on") });
        assert_eq!(end, Some(ended(End::Budget { spent }, 1)), "{asked:?}");
    }
    // Cache tokens that reach their budget, and go no further, stop nothing.
    let mut h = Harness::new(&LIMITS);
    let session =
        h.yielded_with(budget(Budget { input: 11, output: 6, cache_read: 3, cache_write: 2, ..BUDGET }), done());
    drop(calling(h.step(Event::Continue { session, content: bytes(b"go on") })));
}

#[test]
fn the_turn_that_takes_tokens_past_their_budget_runs_its_tools_and_ends_the_session() {
    let cases = [
        (Budget { input: 9, ..BUDGET }, Dimension::Input),
        (Budget { cache_read: 2, ..BUDGET }, Dimension::CacheRead),
        (Budget { cache_write: 1, ..BUDGET }, Dimension::CacheWrite),
        // A zero cache budget is a budget like any other.
        (Budget { cache_read: 0, ..BUDGET }, Dimension::CacheRead),
    ];
    for (asked, spent) in cases {
        // The tools run, and their results are kept, but do not go back.
        let mut h = Harness::new(&LIMITS);
        let (owner, _) = h.open_with(1, budget(asked));
        let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
        let end = h.step(ran(run, b"main.rs"));
        assert_eq!(end, Some(ended(End::Budget { spent }, 1)), "{asked:?}");

        // A yield still yields; the next message ends it.
        let mut h = Harness::new(&LIMITS);
        let session = h.yielded_with(budget(asked), done());
        let end = h.step(Event::Continue { session, content: bytes(b"go on") });
        assert_eq!(end, Some(ended(End::Budget { spent }, 1)), "{asked:?}");
    }
    // An answer past its output budget, which a provider should not give.
    let mut h = Harness::new(&LIMITS);
    let (owner, prompt) = h.open_with(1, budget(Budget { output: 4, ..BUDGET }));
    assert_eq!(prompt.max_tokens, 4);
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    let end = h.step(ran(run, b"main.rs"));
    assert_eq!(end, Some(ended(End::Budget { spent: Dimension::Output }, 1)));

    // Without the cache, a zero cache budget stops nothing.
    let mut h = Harness::new(&LIMITS);
    let uncached =
        Completion { usage: Usage { cache_read_tokens: Some(0), cache_write_tokens: Some(0), ..USAGE }, ..done() };
    let session = h.yielded_with(budget(Budget { cache_read: 0, cache_write: 0, ..BUDGET }), uncached);
    drop(calling(h.step(Event::Continue { session, content: bytes(b"go on") })));
}

#[test]
fn time_does_not_wait_for_the_turn_that_crossed_a_budget() {
    let time = Duration::from_secs(60);
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open_with(1, budget(Budget { input: 9, time, ..BUDGET }));
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    h.after(time);
    assert_eq!(h.fire(), Some(Request::CancelIo { owner: run }));
    assert_eq!(h.step(Event::Done { owner: run, done: Done::Cancelled }), Some(ended(OUT_OF_TIME, 1)));
}

#[test]
fn each_answer_may_take_no_more_than_the_output_budget_left() {
    let mut h = Harness::new(&LIMITS);
    let (owner, prompt) = h.open_with(1, budget(Budget { output: 12, ..BUDGET }));
    assert_eq!(prompt.max_tokens, 12);
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    let (_, prompt) = calling(h.step(ran(run, b"main.rs")));
    assert_eq!(prompt.max_tokens, 7);
    let usage = Usage { output_tokens: Some(7), ..USAGE };
    let cut = Completion { usage, ..completion(Box::new([text(b"I was")]), Stop::MaxTokens) };
    let request = h.step(Event::Completed { owner, completion: cut });
    assert_eq!(yielded(request), (1, Yield::Truncated, bytes(b"I was")));
    let end = h.step(Event::Continue { session: owner, content: bytes(b"go on") });
    let spent = End::Budget { spent: Dimension::Output };
    let usage = usage_sum(USAGE, usage).expect("bounded fixture usage");
    assert_eq!(
        end,
        Some(Request::Ended {
            opener: Token::new(1),
            end: spent,
            turns: 2,
            usage,
            reported: usage,
            detail: Box::default()
        })
    );

    // A spec's own output stays the cap while more is left.
    let mut h = Harness::new(&LIMITS);
    let (_, prompt) = h.open_with(1, Spec { output: 100, ..budget(Budget { output: 1000, ..BUDGET }) });
    assert_eq!(prompt.max_tokens, 100);
}

#[test]
fn the_time_budget_ends_a_session_whatever_it_is_doing() {
    let time = Duration::from_secs(60);
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open_with(1, budget(Budget { time, ..BUDGET }));
    assert_eq!(h.domain.next_deadline(), Some(Time::ZERO.saturating_add(time)));
    h.after(time);
    assert_eq!(h.fire(), Some(Request::Cancel { owner }));
    assert_eq!(h.step(Event::Cancelled { owner }), Some(ended(OUT_OF_TIME, 0)));
}

#[test]
fn invalid_limits_are_refused() {
    assert!(valid(&LIMITS));
    // Heap-total overflow alone is not constructor invalidity.
    assert!(valid(&Limits { history_bytes: u64::MAX, completion_bytes: u64::MAX, answer_bytes: u64::MAX, ..LIMITS }));
    // A batch has positive width.
    assert!(!valid(&Limits { parallel_tools: 0, ..LIMITS }));
    // Batch width stays within the vocabulary cap (with enough calls).
    let wide = Limits { parallel_tools: MAX_PARALLEL + 1, tools: tools::Limits { calls: 9, ..TOOLS }, ..LIMITS };
    assert!(!valid(&wide));
    // The child can accept the whole batch.
    assert!(!valid(&Limits { tools: tools::Limits { calls: 1, ..TOOLS }, ..LIMITS }));
    // A transcript has room for its prompt and answer.
    assert!(!valid(&Limits { history_messages: 1, ..LIMITS }));
    // A transcript can hold a block.
    assert!(!valid(&Limits { history_blocks: 0, ..LIMITS }));
    // Every session can own a kit.
    assert!(!valid(&Limits { tools: tools::Limits { kits: 1, ..TOOLS }, ..LIMITS }));
    // Two generations and completion observations fit the run table.
    assert!(!valid(&Limits { completion_blocks: u32::MAX, ..LIMITS }));
    // Multiplying sessions by the per-session run capacity is checked.
    let many = Limits {
        sessions: u32::MAX / 8,
        completion_blocks: 8,
        tools: tools::Limits { kits: u32::MAX / 8, calls: 2, jobs: 0, ..TOOLS },
        ..LIMITS
    };
    assert!(!valid(&many));
    // Alarm capacity is checked; these limits also exceed the larger run table.
    assert!(!valid(&Limits {
        sessions: u32::MAX / 2 + 1,
        tools: tools::Limits { kits: u32::MAX / 2 + 1, ..TOOLS },
        ..LIMITS
    }));
    // Child validity is independent of session compatibility.
    assert!(!valid(&Limits { tools: tools::Limits { host_path_bytes: u32::MAX, ..TOOLS }, ..LIMITS }));
}

#[test]
fn reservation_prices_input_bytes_allowance_and_maximum_output_once() {
    let mut harness = Harness::new(&LIMITS);
    let mut opening = opening(spec());
    opening.prices = crate::record::Prices { input: 2, cached: 1, cache_write: None, output: 3, unit: 10 };
    opening.budget = 4;
    step(&mut harness.domain, &harness.env, Event::Open { opener: Token::new(1), opening }, &mut harness.out);
    let Some(Request::Opened { session, .. }) = harness.out.pop() else { panic!("recorded opening") };
    let (owner, prompt) = calling(harness.one());
    assert_eq!(owner, session);
    match prompt.input_bytes() {
        Some(bytes) => assert!(bytes >= 7),
        None => panic!("bounded prompt bytes"),
    }
    assert_eq!(crate::preview_reservation(&harness.domain, owner, 7, 5, 4), Some(4));
    assert_eq!(crate::preview_reservation(&harness.domain, owner, 10, 5, 4), None);
}

#[test]
fn unsent_denial_releases_reserved_provider_credit_before_kit_settlement() {
    for (reason, dimension) in [
        (crate::BudgetDenial::Turns, Dimension::Turns),
        (crate::BudgetDenial::Spend, Dimension::Unit),
        (crate::BudgetDenial::Time, Dimension::Time),
    ] {
        let mut harness = Harness::new(&Limits { spend: 100, ..LIMITS });
        let owner = reserved_recorded_call(&mut harness);
        let (reserved, credit) = crate::session::provider_credit_for_test(&harness.domain, owner);
        assert!(reserved > 0);
        assert_eq!(credit, Some(reserved), "the pending provider owns the entire reservation");
        assert_eq!(
            harness.step(Event::BudgetDenied { owner, reason }),
            Some(ended(End::Budget { spent: dimension }, 0)),
        );
        assert_eq!(crate::session::provider_credit_for_test(&harness.domain, owner), (0, None));
        assert_eq!((harness.turns, harness.usage), (0, Usage::ZERO));
        assert_eq!(harness.domain.next_deadline(), None);
        assert_eq!(harness.step(Event::BudgetDenied { owner, reason }), None);
        harness.domain.reclaim();
        assert_eq!((harness.domain.sessions(), harness.domain.kits()), (0, 0));
        assert_eq!(harness.step(Event::BudgetDenied { owner, reason }), None);
    }
}

#[test]
fn raw_overflow_ends_before_the_completion_is_charged_or_told() {
    let budget = Budget { input: u64::MAX, output: u64::MAX, cache_read: u64::MAX, cache_write: u64::MAX, ..BUDGET };
    let mut harness = Harness::new(&Limits { budget, ..LIMITS });
    let (owner, _) = harness.open_with(1, Spec { budget, ..spec() });
    let first = Usage {
        input_tokens: Some(u64::MAX - 1),
        output_tokens: Some(1),
        cache_read_tokens: Some(3),
        cache_write_tokens: Some(4),
        reasoning_tokens: Some(0),
    };
    drop(yielded(harness.step(Event::Completed { owner, completion: Completion { usage: first, ..done() } })));
    drop(calling(harness.step(Event::Continue { session: owner, content: bytes(b"go on") })));
    let second = Usage {
        input_tokens: Some(2),
        output_tokens: Some(2),
        cache_read_tokens: Some(5),
        cache_write_tokens: Some(6),
        reasoning_tokens: Some(0),
    };
    let terminal = harness.step(Event::Completed { owner, completion: Completion { usage: second, ..done() } });
    assert_eq!(harness.usage, first);
    assert_eq!(harness.turn_records.len(), 1);
    assert_eq!(
        terminal,
        Some(Request::Ended {
            opener: Token::new(1),
            end: End::UsageOverflow,
            turns: 1,
            usage: first,
            reported: first,
            detail: Box::default()
        })
    );
    while let Some(fact) = harness.pop_fact() {
        if let Fact::Used { usage, .. } = fact.kind {
            assert_ne!(usage, second);
        }
    }
}

#[test]
fn cache_write_pricing_combines_all_four_rates_and_rounds_once_at_u64() {
    use crate::record::Prices;
    let rates = Prices { input: 1, cached: 2, cache_write: Some(3), output: 5, unit: 7 };
    let usage = Usage {
        input_tokens: Some(1),
        output_tokens: Some(1),
        cache_read_tokens: Some(1),
        cache_write_tokens: Some(1),
        reasoning_tokens: Some(1),
    };
    assert_eq!(rates.price(usage), Some(2), "11/7 rounds once; reasoning is already output");
    assert_eq!(Prices { cache_write: None, ..rates }.price(usage), Some(2));
    let rates = Prices { input: 0, cached: 0, cache_write: Some(u64::MAX), output: 0, unit: 1 };
    assert_eq!(rates.price(Usage { cache_write_tokens: Some(1), ..Usage::ZERO }), Some(u64::MAX));
    assert_eq!(rates.price(Usage { cache_write_tokens: Some(2), ..Usage::ZERO }), None);
    let rates = Prices { input: u64::MAX, ..rates };
    assert_eq!(
        rates.price(Usage { input_tokens: Some(u64::MAX), cache_write_tokens: Some(u64::MAX), ..Usage::ZERO }),
        None
    );
    assert_eq!(Prices { cache_write: Some(0), input: 0, ..rates }.price(Usage::ZERO), Some(0));
}

#[test]
fn completion_reservation_uses_the_highest_rate_for_every_input_kind() {
    for (cached, cache_write, expected) in
        [(1, None, 4), (1, Some(1), 4), (1, Some(11), 15), (11, Some(1), 15), (11, None, 15)]
    {
        let mut harness = Harness::new(&LIMITS);
        let mut opening = opening(spec());
        opening.prices = crate::record::Prices { input: 2, cached, cache_write, output: 3, unit: 10 };
        opening.budget = 15;
        step(&mut harness.domain, &harness.env, Event::Open { opener: Token::new(1), opening }, &mut harness.out);
        let Some(Request::Opened { session, .. }) = harness.out.pop() else { panic!("recorded opening") };
        let (owner, _) = calling(harness.one());
        assert_eq!(owner, session);
        assert_eq!(crate::preview_reservation(&harness.domain, owner, 7, 5, 4), Some(expected));
        assert_eq!(crate::reserve_completion(&mut harness.domain, owner, 7, 5, 4), Some(expected));
    }
}

#[test]
fn narrowing_an_unsent_completion_appends_its_note_and_preserves_definitions() {
    let mut h = Harness::new(&LIMITS);
    let (owner, original) = h.open(1);
    assert_eq!(original.choice, crate::llm::ToolChoice::Auto);
    let note = bytes(b"The ordinary budget is spent. Finish now.");
    let chosen = crate::llm::ToolChoice::Only(Box::new([FINISH.ticket]));
    let Some(Request::Complete { owner: next, prompt, .. }) = h.step(Event::Narrowed {
        owner,
        note: note.clone(),
        choice: chosen.clone(),
        names: Some(Box::new([bytes(b"finish")])),
    }) else {
        panic!("narrowing must reask without a provider terminal");
    };
    assert_eq!(next, owner);
    assert_eq!(h.turns, 0);
    assert_eq!(prompt.choice, chosen);
    assert_eq!(prompt.tools, original.tools);
    assert_eq!(prompt.delegated, original.delegated);
    assert_eq!(prompt.system, original.system);
    assert_eq!(prompt.messages.len(), original.messages.len() + 1);
    assert_eq!(prompt.messages.last().unwrap().content.as_ref(), &[text(&note)]);
}

#[test]
fn a_childs_last_turn_has_one_note_no_effects_and_yields_its_answer_for_budget() {
    for stop in [Stop::ToolUse, Stop::EndTurn, Stop::MaxTokens, Stop::Refusal] {
        let mut harness = Harness::new(&LIMITS);
        let mut selected = spec();
        selected.budget.turns = 1;
        selected.final_turn = Some(crate::FinalTurn { note: bytes(b"Answer now."), allowance: Duration::from_secs(1) });
        let (owner, prompt) = harness.open_with(1, selected);
        assert_eq!(prompt.choice, crate::llm::ToolChoice::None);
        assert_eq!(prompt.delegated.as_ref(), &[FINISH]);
        assert_eq!(prompt.messages.last().unwrap().content.as_ref(), &[text(b"Answer now.")]);
        let answer = completion(Box::new([text(b"What I found."), tool_call(b"outside", list())]), stop);
        assert!(harness.step(Event::Completed { owner, completion: answer }).is_none(), "no IO or delegate starts");
        harness.domain.reclaim();
        assert_eq!(yielded(harness.resume()), (1, Yield::Budget, bytes(b"What I found.")));
        assert_eq!(harness.turns, 1);
        assert_eq!(harness.turn_records.len(), 1);
        let record = harness.turn_records.last().expect("settled final turn");
        let Block::ToolResult { result, .. } = &record.messages.last().unwrap().content[0] else {
            panic!("tool result is retained");
        };
        assert_eq!(result, &Returned::NotRun { why: crate::llm::NotRun::OutsideChoice { choice: None } });
        let ended = harness.step(Event::Continue { session: owner, content: bytes(b"Again.") });
        let Some(Request::Ended { end: End::Budget { spent: Dimension::Turns }, .. }) = ended else {
            panic!("the final answer cannot be continued: {ended:?}");
        };
    }
}

#[test]
fn derived_history_secures_threshold_and_full_receiving_before_effects() {
    let inputs = history_derivation();
    let limits = crate::derive(&inputs).expect("derived window");
    // LINE_BYTES + 600 completion + 256 answer + 6 * 128 wrappers + 512 batch.
    assert_eq!(limits.history_bytes, 2_936);
    // 2936 / 128-byte Block cell + 2.
    assert_eq!(limits.history_messages, 24);
    assert_eq!(limits.history_blocks, 24);
    assert_eq!(limits.completion_blocks, 6);
    assert_eq!(limits.failure_bytes, 128);
    assert_eq!(limits.protocol_allowance, 100);
    assert_eq!(crate::derive(&crate::Derivation { line_bytes: u64::MAX, ..inputs }), Err(crate::Field::HistoryBytes));
    let beyond_cells = (u64::from(u32::MAX) + 1) * u64::try_from(size_of::<Block>()).expect("Block cell fits");
    assert_eq!(
        crate::derive(&crate::Derivation { line_bytes: beyond_cells, ..inputs }),
        Err(crate::Field::HistoryItems)
    );
    assert_eq!(
        crate::derive(&crate::Derivation { completion_bytes: u64::MAX, ..inputs }),
        Err(crate::Field::HistoryBytes)
    );
}

#[test]
fn the_history_holds_the_line_and_one_whole_completion() {
    // (1000 window - 100 output - 50 material) * 8 bytes per token.
    const LINE_BYTES: u64 = 6_800;
    let limits = crate::derive(&crate::Derivation { line_bytes: LINE_BYTES, ..history_derivation() }).unwrap();
    // LINE_BYTES + 600 completion + 256 answer + 6 * 128 wrappers + 512 batch.
    assert_eq!(limits.history_bytes, 8_936);
    // 8936 / 128-byte Block cell + 2.
    assert_eq!(limits.history_messages, 71);
    assert_eq!(limits.history_blocks, 71);
}

#[test]
fn the_check_counts_reported_usage() {
    use crate::window::{Admission, Bytes, Window, check};
    let mut window = Window::fresh(1, 0);
    window.completed(
        Usage {
            input_tokens: Some(900),
            output_tokens: Some(100),
            cache_read_tokens: Some(200),
            cache_write_tokens: Some(300),
            reasoning_tokens: None,
        },
        512,
        2,
        1_400,
    );
    window.appended = Bytes(100);
    assert_eq!(check(&window, 1, 0, LIMITS.token_bytes, 2_000, [4, 5]), Admission::Fits);
    assert_eq!(check(&window, 1, 0, LIMITS.token_bytes, 1_999, [4, 5]), Admission::Compact);
}

#[test]
fn the_check_counts_bytes_without_usage() {
    use crate::window::{Admission, Window, check};
    let mut window = Window::fresh(1, 0);
    window.completed(Usage::NONE, 512, 2, 0);
    assert_eq!(check(&window, 900, 100, LIMITS.token_bytes, 1_250, [4, 5]), Admission::Fits);
    assert_eq!(check(&window, 901, 100, LIMITS.token_bytes, 1_250, [4, 5]), Admission::Compact);
}

#[test]
fn an_overflow_answers_compact() {
    use crate::window::{Admission, Bytes, Tokens, Window, check};
    let mut window = Window::fresh(1, 0);
    window.last_prompt = Some(Tokens(u64::MAX));
    window.last_prompt_bytes = 1;
    window.appended = Bytes(2);
    assert_eq!(check(&window, 0, 0, LIMITS.token_bytes, u32::MAX, [4, 5]), Admission::Compact);
    assert_eq!(check(&Window::fresh(1, 0), u64::MAX, 1, LIMITS.token_bytes, u32::MAX, [4, 5]), Admission::Compact);
}

#[test]
fn a_later_listing_gets_its_share_as_the_lower_byte_cap() {
    let (mut h, session, owner, operation) = later_owned_call(list());
    let Op::List { max_bytes, .. } = operation else { panic!("listing") };
    assert_eq!(max_bytes, 64);
    close_later_call(&mut h, session, owner);
}
