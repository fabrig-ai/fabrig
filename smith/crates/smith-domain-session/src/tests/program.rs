//! Recorded transport and portable restoration (smith/docs/design/domain/session.md 8.2).
#![expect(clippy::disallowed_methods, reason = "ordinary deterministic harness, programming-model.md 10.2")]
use super::*;
use crate::record::{Backend, Kind, Transcript};

fn program() -> Backend {
    Backend::Program { name: bytes(b"claude_code") }
}

#[test]
fn a_program_turn_records_its_backend() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open_with(1, Spec { backend: program(), ..spec() });
    h.step(Event::Completed { owner, completion: completion(Box::new([text(b"answer")]), Stop::EndTurn) });
    let turn = h.turn_records.get(0).expect("recorded turn");
    let Kind::First(opener) = &turn.kind else { panic!("first turn") };
    assert_eq!(opener.backend, program());
    assert_eq!(opener.account, spec().account);
}

fn provider_answer() -> Box<[Block]> {
    Box::new([
        Block::Opaque { bytes: bytes(b"private reasoning") },
        Block::Text {
            text: bytes(b"old answer"),
            replay: Some(crate::llm::Replay { bytes: bytes(b"provider envelope") }),
        },
    ])
}

fn carried(from: Backend, to: Backend, account: Option<u32>) {
    carried_content(from, to, account, provider_answer(), Box::new([text(b"old answer")]));
}

fn carried_content(from: Backend, to: Backend, account: Option<u32>, content: Box<[Block]>, expected: Box<[Block]>) {
    let mut old = Harness::new(&LIMITS);
    let (owner, _) = old.open_with(1, Spec { backend: from, account, ..spec() });
    old.step(Event::Completed { owner, completion: completion(content, Stop::EndTurn) });
    let mut h = Harness::new(&LIMITS);
    let mut start = opening(Spec { backend: to.clone(), ..spec() });
    start.transcript = Some(Transcript {
        version: crate::record::VERSION,
        endpoint: Some(Endpoint(0)),
        dialect: 1,
        turns: old.turn_records.to_boxed(),
    });
    step(&mut h.domain, &h.env, Event::Open { opener: Token::new(2), opening: start }, &mut h.out);
    let Some(Request::Opened { .. }) = h.next() else { panic!("restored session opens") };
    let (owner, prompt) = calling(h.one());
    let crate::llm::Purpose::Compaction { .. } = prompt.purpose else { panic!("compaction") };
    let assistant = prompt.messages.iter().find(|message| message.role == Role::Assistant).expect("old answer");
    assert_eq!(assistant.content, expected, "foreign history is portable");
    assert!(prompt.messages.iter().all(|message| !message.content.is_empty()));
    let (_, next) = calling(
        h.step(Event::Completed { owner, completion: completion(Box::new([text(b"summary")]), Stop::EndTurn) }),
    );
    let crate::llm::Purpose::Ordinary { .. } = next.purpose else { panic!("ordinary window") };
    assert!(next.messages[0].content.contains(&text(b"summary")));
    let turn = h.turn_records.last().expect("new window recorded");
    let Kind::Window(opener) = &turn.kind else { panic!("window opening") };
    assert_eq!(opener.backend, to);
    assert_eq!(opener.account, spec().account);
}

#[test]
fn a_window_from_another_backend_is_carried_over() {
    carried(Backend::Direct, program(), spec().account);
}

#[test]
fn a_window_from_another_account_is_carried_over() {
    carried(Backend::Direct, Backend::Direct, Some(7));
}

#[test]
fn a_program_window_is_carried_over_to_the_direct_backend() {
    carried(program(), Backend::Direct, spec().account);
}

#[test]
fn an_empty_program_message_is_marked_when_carried_to_direct() {
    carried_content(
        program(),
        Backend::Direct,
        spec().account,
        Box::new([]),
        Box::new([text(b"[reasoning not carried over]")]),
    );
}

#[test]
fn an_opaque_only_message_is_marked_when_carried_to_another_account() {
    carried_content(
        Backend::Direct,
        Backend::Direct,
        Some(7),
        Box::new([Block::Opaque { bytes: bytes(b"x") }]),
        Box::new([text(b"[reasoning not carried over]")]),
    );
}

#[test]
fn a_window_on_the_same_program_is_kept() {
    let mut old = Harness::new(&LIMITS);
    let (owner, _) = old.open_with(1, Spec { backend: program(), ..spec() });
    old.step(Event::Completed { owner, completion: completion(Box::new([text(b"old answer")]), Stop::EndTurn) });
    let mut h = Harness::new(&LIMITS);
    let mut start = opening(Spec { backend: program(), ..spec() });
    start.transcript = Some(Transcript {
        version: crate::record::VERSION,
        endpoint: Some(Endpoint(0)),
        dialect: 1,
        turns: old.turn_records.to_boxed(),
    });
    step(&mut h.domain, &h.env, Event::Open { opener: Token::new(2), opening: start }, &mut h.out);
    let Some(Request::Opened { .. }) = h.next() else { panic!("opened") };
    let (_, prompt) = calling(h.one());
    let crate::llm::Purpose::Ordinary { .. } = prompt.purpose else { panic!("kept ordinary window") };
}

fn restored_program(fresh: bool) -> (Harness, Token) {
    restored_program_content(fresh, provider_answer())
}

fn restored_program_content(fresh: bool, content: Box<[Block]>) -> (Harness, Token) {
    let mut old = Harness::new(&LIMITS);
    let (owner, _) = old.open_with(1, Spec { backend: program(), ..spec() });
    old.step(Event::Completed { owner, completion: completion(Box::new([text(b"old answer")]), Stop::EndTurn) });
    if fresh {
        let turn = old.turn_records.get_mut(0).expect("first turn");
        let Kind::First(opener) = core::mem::replace(&mut turn.kind, Kind::Ordinary) else { panic!("first") };
        turn.kind = Kind::Window(opener);
        turn.messages = Box::new([Message {
            role: Role::User,
            content: Box::new([text(b"old task"), text(b"preamble"), text(b"old summary")]),
        }]);
    }
    if !fresh {
        old.turn_records.get_mut(0).expect("first turn").messages.last_mut().expect("assistant").content = content;
    }

    let mut h = Harness::new(&LIMITS);
    let mut start = opening(Spec { backend: program(), ..spec() });
    start.transcript = Some(Transcript {
        version: crate::record::VERSION,
        endpoint: Some(Endpoint(0)),
        dialect: 1,
        turns: old.turn_records.to_boxed(),
    });
    step(&mut h.domain, &h.env, Event::Open { opener: Token::new(2), opening: start }, &mut h.out);
    let Some(Request::Opened { .. }) = h.next() else { panic!("opened") };
    let (owner, _) = calling(h.one());
    (h, owner)
}

#[test]
fn a_gone_session_carries_the_window_over() {
    let (mut h, owner) = restored_program(false);
    let (owner, prompt) = calling(h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::SessionGone,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"missing"),
    }));
    let crate::llm::Purpose::Compaction { .. } = prompt.purpose else { panic!("carried over") };
    let assistant = prompt.messages.iter().find(|message| message.role == Role::Assistant).expect("old answer");
    assert_eq!(assistant.content.as_ref(), &[text(b"old answer")], "opaque content and replay are removed");
    let (_, next) = calling(
        h.step(Event::Completed { owner, completion: completion(Box::new([text(b"summary")]), Stop::EndTurn) }),
    );
    let crate::llm::Purpose::Ordinary { .. } = next.purpose else { panic!("new window") };
    assert!(next.messages[0].content.contains(&text(b"summary")));
}

#[test]
fn a_gone_session_marks_an_empty_program_message() {
    let (mut h, owner) = restored_program_content(false, Box::new([]));
    let (_, prompt) = calling(h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::SessionGone,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"missing"),
    }));
    let crate::llm::Purpose::Compaction { .. } = prompt.purpose else { panic!("carried over") };
    let assistant = prompt.messages.iter().find(|message| message.role == Role::Assistant).expect("empty answer");
    assert_eq!(assistant.content.as_ref(), &[text(b"[reasoning not carried over]")]);
    assert!(prompt.messages.iter().all(|message| !message.content.is_empty()));
}

#[test]
fn a_second_gone_session_ends_the_session() {
    let (mut h, owner) = restored_program(false);
    calling(h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::SessionGone,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"missing"),
    }));
    let ended = h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::SessionGone,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"missing again"),
    });
    let Some(Request::Ended { end: End::Failed { failure: Failure::SessionGone, .. }, .. }) = ended else {
        panic!("second missing ends: {ended:?}")
    };
}

#[test]
fn a_gone_session_carries_a_fresh_window_over() {
    let (mut h, owner) = restored_program(true);
    let (owner, prompt) = calling(h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::SessionGone,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"missing"),
    }));
    let crate::llm::Purpose::Compaction { .. } = prompt.purpose else { panic!("fresh window carries over") };
    assert!(prompt.messages[0].content.contains(&text(b"old summary")));
    let (_, next) = calling(
        h.step(Event::Completed { owner, completion: completion(Box::new([text(b"new summary")]), Stop::EndTurn) }),
    );
    assert!(next.messages[0].content.contains(&text(b"new summary")));
    assert!(next.messages[0].content.contains(&text(b"fix the bug")), "waking prompt is held aside");
}

#[test]
fn a_window_from_another_program_is_carried_over() {
    carried(program(), Backend::Program { name: bytes(b"other_program") }, spec().account);
}

fn empty_program_history(turns: u32) -> Transcript {
    let mut old = Harness::new(&LIMITS);
    let (owner, _) = old.open_with(1, Spec { backend: program(), ..spec() });
    old.step(Event::Completed { owner, completion: completion(Box::new([]), Stop::EndTurn) });
    let first = old.turn_records.get(0).expect("recorded empty assistant");
    let mut history = List::with_capacity(turns);
    for sequence in 1..=turns {
        let mut turn = first.clone();
        turn.sequence = sequence;
        if sequence != 1 {
            turn.kind = Kind::Ordinary;
        }
        history.push(turn).expect("one slot per turn");
    }
    Transcript { version: crate::record::VERSION, endpoint: Some(Endpoint(0)), dialect: 1, turns: history.into_boxed() }
}

fn open_history(limits: &Limits, backend: Backend, turns: u32) -> Harness {
    let mut h = Harness::new(limits);
    let mut start = opening(Spec { backend, ..spec() });
    start.transcript = Some(empty_program_history(turns));
    step(&mut h.domain, &h.env, Event::Open { opener: Token::new(2), opening: start }, &mut h.out);
    h
}

fn assert_too_large(h: &mut Harness) {
    let end = h.next();
    let Some(Request::Ended { end: End::TranscriptRefused { reason: crate::record::Refusal::TooLarge }, .. }) = end
    else {
        panic!("expected too-large refusal, got {end:?}")
    };
}

#[test]
fn carried_markers_count_against_restored_history_blocks() {
    // One waking block, one recorded user block, one marker and receiving credit.
    let exact = 3 + LIMITS.completion_blocks * 2;
    let mut fits = open_history(&Limits { history_blocks: exact, ..LIMITS }, Backend::Direct, 1);
    let Some(Request::Opened { .. }) = fits.next() else { panic!("fits at the bound") };
    let (_, prompt) = calling(fits.one());
    assert_eq!(prompt.messages[1].content.as_ref(), &[text(b"[reasoning not carried over]")]);
    let mut over = open_history(&Limits { history_blocks: exact - 1, ..LIMITS }, Backend::Direct, 1);
    assert_too_large(&mut over);
}

#[test]
fn carried_markers_count_against_restored_history_bytes() {
    let recorded_user = crate::session::content_cost(&[text(&spec().prompt)]).expect("user cost");
    let marker = crate::session::content_cost(&[text(b"[reasoning not carried over]")]).expect("marker cost");
    let exact = initial_bytes()
        + spec().authority.namespace_bytes().expect("namespace")
        + recorded_user
        + marker
        + crate::completion_reserve(&LIMITS).expect("receiving credit");
    let mut fits = open_history(&Limits { history_bytes: exact, ..LIMITS }, Backend::Direct, 1);
    let Some(Request::Opened { .. }) = fits.next() else { panic!("fits at the bound") };
    let (_, prompt) = calling(fits.one());
    assert_eq!(prompt.messages[1].content.as_ref(), &[text(b"[reasoning not carried over]")]);
    let mut over = open_history(&Limits { history_bytes: exact - 1, ..LIMITS }, Backend::Direct, 1);
    assert_too_large(&mut over);
}

fn gone(h: &mut Harness, owner: Token) -> Option<Request> {
    h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::SessionGone,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"missing"),
    })
}

#[test]
fn gone_markers_that_exceed_history_blocks_end_with_the_window_limit() {
    let mut h = open_history(
        &Limits { history_messages: 32, history_blocks: 12, completion_blocks: 1, ..LIMITS },
        program(),
        8,
    );
    let Some(Request::Opened { .. }) = h.next() else { panic!("original history fits") };
    let (owner, prompt) = calling(h.one());
    let crate::llm::Purpose::Ordinary { .. } = prompt.purpose else { panic!("kept program window") };
    let end = gone(&mut h, owner);
    let Some(Request::Ended { end, .. }) = end else { panic!("expected window limit") };
    let End::Failed {
        failure: Failure::Limit { which: crate::llm::Limit::Window, bound: 65_536 },
        evidence: crate::llm::Evidence::Unsent,
        attempts: 0,
        ..
    } = end
    else {
        panic!("expected unsent window limit, got {end:?}")
    };
}

#[test]
fn gone_markers_that_exceed_history_bytes_end_with_the_window_limit() {
    let limits = Limits { history_messages: 32, completion_blocks: 1, completion_bytes: 1, answer_bytes: 1, ..LIMITS };
    let recorded_user = crate::session::content_cost(&[text(&spec().prompt)]).expect("user cost");
    // Enough for ordinary receiving credit, a summary cell and a minimal result,
    // but not eight new marker blocks once the program session is gone.
    let original = initial_bytes()
        + spec().authority.namespace_bytes().expect("namespace")
        + u64::try_from(b"claude_code".len()).expect("backend name")
        + recorded_user * 8;
    let bound = original
        + crate::completion_reserve(&limits).expect("receiving credit")
        + u64::try_from(size_of::<Block>()).expect("summary cell")
        + u64::from(TOOLS.path_bytes);
    let mut h = open_history(&Limits { history_bytes: bound, ..limits }, program(), 8);
    let Some(Request::Opened { .. }) = h.next() else { panic!("original history fits") };
    let (owner, prompt) = calling(h.one());
    let crate::llm::Purpose::Ordinary { .. } = prompt.purpose else { panic!("kept program window") };
    let end = gone(&mut h, owner);
    let Some(Request::Ended { end, .. }) = end else { panic!("expected window limit") };
    let End::Failed {
        failure: Failure::Limit { which: crate::llm::Limit::Window, bound: 65_536 },
        evidence: crate::llm::Evidence::Unsent,
        attempts: 0,
        ..
    } = end
    else {
        panic!("expected unsent window limit, got {end:?}")
    };
}

#[test]
fn a_program_window_from_another_account_is_carried_over() {
    carried(program(), program(), Some(7));
}
