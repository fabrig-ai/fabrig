//! Content-free observations and concrete turn identity.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::*;

#[test]
fn a_maximum_refused_batch_keeps_every_call_observation_without_a_fact_allowance() {
    let tools = TOOLS;
    let mut h = Harness::new(&Limits { tools, parallel_tools: MAX_PARALLEL, ..LIMITS });
    let (owner, _) = h.open(1);
    let mut content = List::with_capacity(MAX_PARALLEL);
    for _ in 0..MAX_PARALLEL {
        content
            .push(tool_call(b"c", Call::Read { path: outside(), offset: 1, window: tools::Budget::Default }))
            .expect("batch bound");
    }
    h.keep_facts();
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content.into_boxed(), Stop::ToolUse) }), None);
    let mut facts = 0;
    let mut answered = 0;
    while let Some(fact) = h.domain.pop_fact() {
        facts += 1;
        if let Fact::Tools { fact: tools::FactKind::Answered { .. }, .. } = fact.kind {
            answered += 1;
            assert_eq!(fact.at, h.env.now);
            assert_eq!(fact.call.expect("refused call metadata").id.as_ref(), b"c");
        }
    }
    assert_eq!(facts, MAX_PARALLEL + 2, "one terminal per call plus response and use");
    assert_eq!(answered, MAX_PARALLEL);
    assert_eq!(h.domain.calls.tools.pending_facts(), 0, "every child observation passed to its parent");
}

#[test]
fn a_session_tells_what_happens_as_facts() {
    let mut h = Harness::new(&LIMITS);
    let opener = Token::new(1);
    let (owner, _) = h.open(1);
    h.told(&[
        Fact::Opened { opener },
        Fact::CompletionStarted {
            window: 1,
            opener,
            attempt: 0,
            messages: 1,
            max_tokens: 1024,
            choice: crate::facts::Choice::Auto,
        },
        by_tools(opener, tools::FactKind::Opened { session: owner }),
    ]);

    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    h.told(&[
        Fact::CompletionAnswered { opener, stop: Stop::ToolUse, blocks: 1, calls: 1, invalid: 0 },
        Fact::Used { opener, usage: USAGE },
        by_tools(opener, tools::FactKind::Started { session: owner, tool: tools::Tool::Read }),
    ]);
    drop(calling(h.step(ran(run, b"main.rs"))));
    let verdict = tools::Verdict::Read;
    h.told(&[
        Fact::CompletionStarted {
            window: 1,
            opener,
            attempt: 0,
            messages: 3,
            max_tokens: 1024,
            choice: crate::facts::Choice::Auto,
        },
        by_tools(
            opener,
            tools::FactKind::Answered {
                session: owner,
                tool: tools::Tool::Read,
                verdict,
                bytes: 7,
                omitted_bytes: 0,
                job: None,
                clamped: false,
            },
        ),
    ]);
    drop(yielded(h.step(Event::Completed { owner, completion: done() })));
    h.told(&[
        Fact::CompletionAnswered { opener, stop: Stop::EndTurn, blocks: 1, calls: 0, invalid: 0 },
        Fact::Used { opener, usage: USAGE },
        Fact::Yielded { opener, stop: Yield::Done },
    ]);
    let end = h.step(Event::Close { session: owner });
    let Some(Request::Ended { opener: _, end, turns, usage, .. }) = end else {
        panic!("expected the end, not {end:?}");
    };
    h.told(&[by_tools(opener, tools::FactKind::Closed { session: owner }), Fact::Ended { opener, end, turns, usage }]);
}

#[test]
fn native_call_history_is_complete_without_a_session_fact_allowance() {
    let mut harness = Harness::new(&LIMITS);
    drive(&mut harness);
    harness.keep_facts();
    assert_eq!(harness.observed.len(), 8);
    for _ in 0_u32..8 {
        assert!(harness.pop_fact().is_some());
    }
    assert!(harness.pop_fact().is_none());
}

#[test]
fn delayed_facts_keep_their_times_completion_numbers_and_exact_call_measurements() {
    let mut h = Harness::new(&LIMITS);
    h.env.now = Time::from_nanos(7);
    let (owner, _) = h.open(1);
    h.env.now = Time::from_nanos(11);
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    h.env.now = Time::from_nanos(13);
    drop(calling(h.step(ran(run, b"main.rs"))));
    h.env.now = Time::from_nanos(17);
    drop(yielded(h.step(Event::Completed { owner, completion: done() })));
    h.env.now = Time::from_nanos(19);
    drop(h.step(Event::Close { session: owner }));
    h.env.now = Time::from_nanos(99);
    let facts = observed_facts(&mut h);
    assert_eq!(fact_times(&facts).as_ref(), [7, 7, 7, 11, 11, 11, 13, 13, 17, 17, 17, 19, 19].map(Time::from_nanos));
    for index in [1, 3, 4] {
        assert_eq!(facts[index].response.expect("first response").number, 1);
    }
    for index in [6, 8, 9] {
        assert_eq!(facts[index].response.expect("second response").number, 2);
    }
    assert_eq!(facts[4].response.expect("charged usage").spent, Some(0));
    let call = facts[5].call.as_ref().expect("workspace start metadata");
    assert_eq!(&*call.id, b"c1");
    assert_eq!(&*call.name, b"tool");
    assert_eq!(call.source, crate::ToolSource::Workspace);
    assert_eq!(call.effect, Effect::Read);
    assert_eq!(call.input_bytes, 2);
    assert_eq!(
        call.deadline,
        Time::from_nanos(7).saturating_add(BUDGET.time),
        "remaining conversation time bounds the common tool deadline"
    );
    assert_eq!(facts[7].call.as_ref(), Some(call), "terminal retains metadata after transcript advances");
}

#[test]
fn delegated_facts_keep_identity_source_and_time_at_both_terminals() {
    let mut h = Harness::new(&LIMITS);
    h.env.now = Time::from_nanos(7);
    let (owner, _) = h.open(1);
    h.env.now = Time::from_nanos(11);
    let mut host = delegated(b"host-a", 9, Effect::Read);
    if let Block::ToolCall { call: Decoded::Delegated { source, .. }, .. } = &mut host {
        *source = crate::ToolSource::Host;
    }
    let content = Box::new([host, delegated(b"run-b", 10, Effect::Read)]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    h.env.now = Time::from_nanos(13);
    assert_eq!(h.step(answered(runs.as_slice()[0], 20, 3, true)), None);
    h.env.now = Time::from_nanos(17);
    drop(h.batch(Event::Close { session: owner }));
    h.env.now = Time::from_nanos(19);
    drop(h.step(answer_cancelled(runs.as_slice()[1])));
    h.env.now = Time::from_nanos(99);
    let facts = observed_facts(&mut h);
    assert_eq!(fact_times(&facts).as_ref(), [7, 7, 7, 11, 11, 11, 11, 13, 17, 19, 19].map(Time::from_nanos));
    let host = facts[5].call.as_ref().expect("host start");
    let run = facts[6].call.as_ref().expect("run start");
    assert_eq!(&*host.id, b"host-a");
    assert_eq!(host.source, crate::ToolSource::Host);
    assert_eq!(host.owner, runs.as_slice()[0]);
    assert_eq!(&*run.id, b"run-b");
    assert_eq!(run.source, crate::ToolSource::Run);
    assert_eq!(run.owner, runs.as_slice()[1]);
    assert_eq!(host.effect, Effect::Read);
    assert_eq!(host.input_bytes, 2);
    assert_eq!(host.deadline, Time::from_nanos(7).saturating_add(BUDGET.time));
    assert_eq!(facts[7].call.as_ref(), Some(host));
    assert_eq!(facts[9].call.as_ref(), Some(run));
}

#[test]
fn failed_retried_and_cancelled_facts_keep_one_response_number_and_step_times() {
    let mut h = Harness::new(&LIMITS);
    h.env.now = Time::from_nanos(7);
    let (owner, _) = h.open(1);
    h.env.now = Time::from_nanos(11);
    drop(h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::Overloaded,
        evidence: crate::llm::Evidence::Unknown,
        detail: bytes(b""),
    }));
    h.env.now = h.domain.next_deadline().expect("retry deadline");
    let retried_at = h.env.now;
    drop(calling(h.fire()));
    h.after(Duration::from_nanos(2));
    let closed_at = h.env.now;
    drop(h.step(Event::Close { session: owner }));
    h.after(Duration::from_nanos(2));
    let cancelled_at = h.env.now;
    drop(h.step(Event::Cancelled { owner }));
    h.after(Duration::from_secs(10));
    let facts = observed_facts(&mut h);
    assert_eq!(
        fact_times(&facts).as_ref(),
        [
            Time::from_nanos(7),
            Time::from_nanos(7),
            Time::from_nanos(7),
            Time::from_nanos(11),
            Time::from_nanos(11),
            retried_at,
            closed_at,
            cancelled_at,
            cancelled_at
        ]
    );
    for index in [1, 3, 4, 5, 7] {
        assert_eq!(facts[index].response.expect("same completion").number, 1);
    }
}

#[test]
fn usage_fact_carries_the_exact_accepted_host_unit_charge() {
    let mut h = Harness::new(&LIMITS);
    let mut opened = opening(spec());
    opened.prices = crate::record::Prices { input: 7, cached: 3, cache_write: None, output: 11, unit: 5 };
    let expected = opened.prices.price(USAGE).expect("price fits");
    step(&mut h.domain, &h.env, Event::Open { opener: Token::new(1), opening: opened }, &mut h.out);
    let Some(Request::Opened { session: owner, .. }) = h.next() else { panic!("session opens") };
    drop(calling(h.one()));
    step(&mut h.domain, &h.env, Event::Completed { owner, completion: done() }, &mut h.out);
    while h.out.pop().is_some() {}
    let facts = observed_facts(&mut h);
    let used = &facts[4];
    assert_eq!(used.kind, Fact::Used { opener: Token::new(1), usage: USAGE });
    assert_eq!(used.response, Some(crate::ResponseInfo { number: 1, spent: Some(expected) }));
}

#[test]
fn delayed_workspace_terminal_facts_keep_tool_caps_before_any_render() {
    let tools = tools::Limits { read_bytes: 4, tool_payload: 8, ..TOOLS };
    let mut h = Harness::new(&Limits { tools, ..LIMITS });
    h.env.now = Time::from_nanos(7);
    let (owner, _) = h.open(1);
    h.env.now = Time::from_nanos(11);
    let content = Box::new([tool_call(
        b"windowed",
        Call::Read { path: path(b"main.rs"), offset: 2, window: tools::Budget::Asked(99) },
    )]);
    let (job, _) = running(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }));
    h.env.now = Time::from_nanos(13);
    drop(calling(h.step(ran(job, b"a\nb\nc\nd\ne\nf\n"))));
    h.env.now = Time::from_nanos(17);
    drop(yielded(h.step(Event::Completed { owner, completion: done() })));
    h.env.now = Time::from_nanos(19);
    drop(h.step(Event::Close { session: owner }));
    h.env.now = Time::from_nanos(99);
    let facts = observed_facts(&mut h);
    let mut terminal = None;
    let mut started = None;
    for fact in &facts {
        if let Fact::Tools { fact: tools::FactKind::Answered { .. }, .. } = fact.kind {
            assert!(terminal.replace(fact).is_none(), "one workspace terminal");
        }
        if let Fact::Tools { fact: tools::FactKind::Started { .. }, .. } = fact.kind {
            assert!(started.replace(fact).is_none(), "one workspace start");
        }
    }
    let terminal = terminal.expect("workspace terminal");
    let started = started.expect("workspace start");
    assert_eq!(terminal.at, Time::from_nanos(13));
    assert_eq!(
        terminal.kind,
        by_tools(
            Token::new(1),
            tools::FactKind::Answered {
                session: owner,
                tool: tools::Tool::Read,
                verdict: tools::Verdict::Read,
                bytes: 8,
                omitted_bytes: 2,
                job: None,
                clamped: true,
            }
        )
    );
    let metadata = terminal.call.as_ref().expect("provider identity retained");
    assert_eq!(&*metadata.id, b"windowed");
    assert_eq!(started.at, Time::from_nanos(11));
    assert_eq!(terminal.call, started.call, "the parent call identity survives terminal settlement");
    assert_eq!(metadata.source, crate::ToolSource::Workspace);
    assert_eq!(metadata.effect, Effect::Read);
}
