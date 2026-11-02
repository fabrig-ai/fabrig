//! Program routing at the root (smith/docs/design/protocol/programs.md 2, 5.7).
use super::*;

fn program() -> Harness {
    let mut h = Harness::new();
    h.domain.config.programs = Box::new([crate::Program { endpoint: Endpoint(1), name: bytes(b"claude_code") }]);
    h
}

#[test]
fn a_program_completion_skips_the_session_reservation() {
    let mut h = program();
    let (_, owner, _) = h.admit(7, charter());
    assert_eq!(session::preview_completion(&h.domain.session, owner, Usage::NONE), Err(session::End::PriceOverflow));
}

#[test]
fn a_program_completion_without_usage_overflows() {
    let mut h = program();
    let (_, owner, _) = h.admit(7, charter());
    let emitted = h.step(Event::Below(crate::Below::Completed {
        owner,
        completion: Completion {
            reasoning_dropped: Box::default(),
            content: Box::default(),
            stop: Stop::EndTurn,
            usage: Usage::NONE,
        },
    }));
    let [Request::Answer { answer: run::Answer::Failed { failure, .. }, .. }] = emitted.as_ref() else {
        panic!("overflow answer: {emitted:?}");
    };
    assert_eq!(
        *failure,
        run::Failure::Budget { exhausted: run::Exhausted::Overflow(run::Overflow::Spend), used: None, bound: None }
    );
}

#[test]
fn a_program_overrun_is_spent() {
    let mut h = program();
    let mut selected = charter();
    selected.llm.prices.unit = 1;
    let (_, owner, _) = h.admit(7, selected);
    let usage = Usage { input_tokens: Some(10_001), output_tokens: Some(1), ..Usage::ZERO };
    let emitted = h.step(Event::Below(crate::Below::Completed {
        owner,
        completion: Completion {
            reasoning_dropped: Box::default(),
            content: Box::default(),
            stop: Stop::EndTurn,
            usage,
        },
    }));
    assert!(emitted.is_empty());
    let emitted = h.next();
    let [Request::Answer { answer: run::Answer::Failed { spent, .. }, .. }] = emitted.as_ref() else {
        panic!("spent answer: {emitted:?}");
    };
    assert_eq!(spent.units, 10_002);
}

#[test]
fn a_program_main_offers_its_sub_agents() {
    let mut h = program();
    let (_, _, prompt) = h.admit(7, charter());
    assert!(prompt.served.contains(&Served::SubAgent));
}

#[test]
fn a_direct_main_keeps_its_sub_agents() {
    let mut h = Harness::new();
    let (_, _, prompt) = h.admit(7, charter());
    assert!(prompt.served.contains(&Served::SubAgent));
}

#[test]
fn a_program_on_an_unknown_endpoint_is_invalid() {
    let mut h = program();
    h.domain.config.programs[0].endpoint = Endpoint(99);
    assert!(!h.domain.config.valid(&LIMITS));
}

#[test]
fn the_run_s_time_cancels_a_program_completion() {
    let mut h = program();
    let (_, owner, _) = h.admit(7, charter());
    h.env.now = Time::ZERO.saturating_add(BUDGET.time);
    assert!(h.fire().is_empty());
    assert_eq!(h.fire().as_ref(), &[Request::Lower(crate::Lower::Cancel { owner })]);
    assert!(h.next().is_empty());
    let emitted = h.step(Event::Below(crate::Below::Cancelled { owner }));
    let [
        Request::Answer {
            answer: run::Answer::Failed { failure: run::Failure::Budget { exhausted: run::Exhausted::Time, .. }, .. },
            ..
        },
    ] = emitted.as_ref()
    else {
        panic!("time budget answer: {emitted:?}");
    };
}

#[test]
fn a_program_spec_names_its_backend() {
    let mut h = program();
    let (_, owner, _) = h.admit(7, charter());
    h.step(Event::Below(crate::Below::Completed {
        owner,
        completion: Completion {
            reasoning_dropped: Box::default(),
            content: Box::new([Said::Text { text: bytes(b"answer"), replay: None }]),
            stop: Stop::EndTurn,
            usage: Usage::ZERO,
        },
    }));
    let turn = h.turns.last().expect("program turn");
    let session::record::Kind::First(opener) = &turn.kind else { panic!("opening") };
    assert_eq!(opener.backend, session::record::Backend::Program { name: bytes(b"claude_code") });
}

fn open_policy() -> Charter {
    Charter { mode: run::Mode::Open, ..charter() }
}

#[test]
fn an_open_program_main_keeps_real_turns_and_waits() {
    let mut h = program();
    let (_, owner, prompt) = h.admit(7, open_policy());
    assert_eq!(prompt.asker, crate::llm::Asker::Main);
    assert!(!prompt.served.contains(&Served::Finish) && !prompt.served.contains(&Served::Deliver));
    assert!(prompt.served.contains(&Served::Wait));
    let emitted = h.says(owner, b"Hello");
    let [Request::Waiting { read, .. }] = emitted.as_ref() else { panic!("one waiting notice") };
    assert_eq!(*read, None);
    assert_eq!(h.turns.len(), 1);
    assert!(h.next().is_empty());
}

#[test]
fn open_program_final_choice_permits_only_text() {
    let mut h = program();
    let mut selected = open_policy();
    selected.budget.turns = 2;
    selected.reserve = run::charter::Reserve { turns: 1, time: Duration::from_secs(60) };
    let (_, owner, _) = h.admit(7, selected);
    let emitted = h.step(Event::Below(crate::Below::Completed {
        owner,
        completion: Completion {
            reasoning_dropped: Box::default(),
            content: Box::new([Said::Text { text: bytes(b"partial"), replay: None }]),
            stop: Stop::MaxTokens,
            usage: USAGE,
        },
    }));
    let [Request::Lower(crate::Lower::Complete { owner, prompt, .. })] = emitted.as_ref() else {
        panic!("last prompt: {emitted:?}")
    };
    assert_eq!(prompt.choice, crate::llm::ToolChoice::None);
    assert_eq!(prompt.asker, crate::llm::Asker::Main);
    assert!(!prompt.served.contains(&Served::Finish));
    let owner = *owner;
    let emitted = h.says(owner, b"Last reply");
    let [Request::Waiting { .. }] = emitted.as_ref() else { panic!("one waiting notice") };
    let emitted = h.next();
    let [Request::Answer { answer: run::Answer::Parked { turns: 2, .. }, .. }] = emitted.as_ref() else {
        panic!("one parked terminal");
    };
}

#[test]
fn opaque_and_refusal_blocks_never_become_open_text_replies() {
    let contents: [Box<[Said]>; 2] = [
        Box::new([Said::Opaque { bytes: bytes(b"reasoning metadata") }]),
        Box::new([Said::Refusal { text: bytes(b"explicit refusal"), replay: None }]),
    ];
    for content in contents {
        let mut h = program();
        let (_, owner, _) = h.admit(7, open_policy());
        let emitted = h.step(Event::Below(crate::Below::Completed {
            owner,
            completion: Completion { reasoning_dropped: Box::default(), content, stop: Stop::EndTurn, usage: USAGE },
        }));
        let [Request::Lower(crate::Lower::Complete { prompt, .. })] = emitted.as_ref() else {
            panic!("nudge: {emitted:?}")
        };
        let message = prompt.messages.last().expect("nudge user message");
        let [Block::Text { text, .. }] = message.content.as_ref() else { panic!("one nudge text") };
        assert_eq!(text.as_ref(), b"You ended your turn with no text, so you have not replied. Carry on, and reply with text or call `wait`. This is reminder 1 of 1: if you stop again without replying after the last one, the run ends unfinished.");
    }
}

#[test]
fn an_open_finish_yields_the_main_completion_owner_once() {
    let mut harness = program();
    let (run, owner, _) = harness.admit(7, open_policy());
    drop(harness.step(Event::Message {
        run,
        name: Token::new(99),
        label: bytes(b"person"),
        text: bytes(b"Please finish"),
        finish_requested: true,
    }));
    assert_eq!(harness.next().as_ref(), &[Request::Lower(crate::Lower::Yield { owner })]);
    assert!(harness.next().is_empty());
}

/// A direct main with a named model for its child on endpoint one.
fn child_policy() -> Charter {
    let mut selected = charter();
    selected.models = Box::new([Llm { model: bytes(b"model-b"), ..selected.llm.clone() }]);
    selected.llm.endpoint = Endpoint(2);
    selected
}

fn named_child() -> Ask {
    Ask::SubAgent {
        brief: bytes(b"Find the bug."),
        families: Families { tools: Tools { inspect: true, modify: false, shell: false }, agents: false },
        llm: Some(bytes(b"model-b")),
        share: run::Share::UNASKED,
    }
}

#[test]
fn a_program_child_that_answers_sends_close_once() {
    let mut h = program();
    let (_, main, prompt) = h.admit(7, child_policy());
    assert_eq!(prompt.endpoint, crate::llm::Endpoint(2));
    let (child, prompt) = completing(h.answer(main, Box::new([served(b"a1", named_child())])));
    assert_ne!(child, main);
    assert_eq!(prompt.endpoint, crate::llm::Endpoint(1));
    assert_eq!(prompt.asker, crate::llm::Asker::Child);
    assert!(h.next().is_empty(), "no close before the child's completion terminal");
    assert!(h.says(child, b"It is in main.rs.").is_empty());
    assert_eq!(h.next().as_ref(), &[Request::Lower(crate::Lower::Close { owner: child })]);
    let (owner, prompt) = completing(h.next());
    assert_eq!(owner, main);
    let [Block::ToolResult { id, result: Returned::Text { text, error: false, replay: None } }] = last(&prompt) else {
        panic!("child answer: {:?}", last(&prompt));
    };
    assert_eq!(id.as_ref(), b"a1");
    assert_eq!(text.as_ref(), b"It is in main.rs.\n\nChild usage: used 1 turns and 0 seconds; stopped for budget: no; ordinary left 8 turns and 600 seconds.");
    assert!(h.next().is_empty(), "close is sent once");
    assert!(h.answer(main, Box::new([served(b"f1", verdict(b"approve"))])).is_empty());
    let emitted = h.next();
    let [Request::Answer { answer: run::Answer::Accepted { .. }, .. }] = emitted.as_ref() else {
        panic!("accepted run: {emitted:?}");
    };
    assert!(h.next().is_empty());
}

#[test]
fn a_cancelled_program_child_sends_close_after_its_cancel() {
    let mut h = program();
    let (run, main, _) = h.admit(7, child_policy());
    let (child, prompt) = completing(h.answer(main, Box::new([served(b"a1", named_child())])));
    assert_eq!(prompt.asker, crate::llm::Asker::Child);
    assert_eq!(prompt.endpoint, crate::llm::Endpoint(1));
    assert!(h.step(Event::Cancel { run }).is_empty());
    assert!(h.next().is_empty(), "main's close withdraws its child call");
    assert_eq!(h.next().as_ref(), &[Request::Lower(crate::Lower::Cancel { owner: child })]);
    assert!(h.next().is_empty(), "close waits for the cancellation terminal");
    assert_eq!(
        h.step(Event::Below(crate::Below::Cancelled { owner: child })).as_ref(),
        &[Request::Lower(crate::Lower::Close { owner: child })]
    );
    let emitted = h.next();
    let [Request::Answer { answer: run::Answer::Failed { failure: run::Failure::Cancelled, .. }, .. }] =
        emitted.as_ref()
    else {
        panic!("cancelled run: {emitted:?}");
    };
    assert!(h.next().is_empty(), "close is sent once");
}

#[test]
fn a_direct_child_and_main_send_no_close() {
    let mut h = Harness::new();
    let (_, main, _) = h.admit(7, child_policy());
    let (child, prompt) = completing(h.answer(main, Box::new([served(b"a1", named_child())])));
    assert_eq!(prompt.asker, crate::llm::Asker::Child);
    assert!(h.says(child, b"It is in main.rs.").is_empty());
    assert!(h.next().is_empty(), "a direct child ends without Close");
    let (owner, prompt) = completing(h.next());
    assert_eq!(owner, main);
    let [Block::ToolResult { result: Returned::Text { error: false, .. }, .. }] = last(&prompt) else {
        panic!("child answer: {:?}", last(&prompt));
    };
    assert!(h.next().is_empty());
    assert!(h.answer(main, Box::new([served(b"f1", verdict(b"approve"))])).is_empty());
    let emitted = h.next();
    let [Request::Answer { answer: run::Answer::Accepted { .. }, .. }] = emitted.as_ref() else {
        panic!("accepted run: {emitted:?}");
    };
    assert!(h.next().is_empty(), "main ends without Close");
}

#[test]
fn a_program_child_compacts_without_closing_its_conversation() {
    let mut h = program();
    let (_, main, _) = h.admit(7, child_policy());
    let (child, _) = completing(h.answer(main, Box::new([served(b"a1", named_child())])));
    assert!(h.answer(child, Box::new([served(b"f1", verdict(b"approve"))])).is_empty());
    let (owner, _) = completing(h.next());
    assert_eq!(owner, child);
    let (owner, prompt) = completing(h.step(Event::Below(crate::Below::Failed {
        owner: child,
        failure: crate::llm::Failure::ContextTooLong,
        evidence: crate::llm::Evidence::Unsent,
        detail: Box::default(),
        report: crate::llm::Report::none(),
    })));
    assert_eq!(owner, child);
    let crate::llm::Purpose::Compaction { .. } = prompt.purpose else { panic!("summary prompt") };
    assert!(h.next().is_empty(), "asking for a summary sends no Close");
    let (owner, prompt) = completing(h.says(child, b"Summary of the investigation."));
    assert_eq!(owner, child);
    let crate::llm::Purpose::Ordinary { .. } = prompt.purpose else { panic!("ordinary prompt after summary") };
    assert!(h.next().is_empty(), "the compaction terminal sends no Close");
    assert!(h.says(child, b"It is in main.rs.").is_empty());
    assert_eq!(h.next().as_ref(), &[Request::Lower(crate::Lower::Close { owner: child })]);
    let (owner, _) = completing(h.next());
    assert_eq!(owner, main);
    assert!(h.next().is_empty(), "only the conversation's end sends Close");
}
