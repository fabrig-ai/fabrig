//! A start ending before its switch, and going back (protocol/binaries.md; domain/root.md, 8.1).
use crate::{End, engine::Engine};
use jig_protocol_store::{Event, Request};
use skein_lib::{Duration, Queue, Time, Token};

/// The store's ordered progress and the same refusal a serving start would meet.
#[derive(Debug)]
pub struct Report {
    pub events: Queue<Event>,
    pub elapsed: Duration,
    pub end: Option<End>,
    pub restored: bool,
    pub done: bool,
}

#[expect(clippy::struct_excessive_bools, reason = "operation kind, close request, close terminal and back terminal are independent")]
pub(crate) struct State {
    pub(crate) report: Report,
    rehearsal: bool,
    began: Option<Time>,
    closing: bool,
    pub(crate) closed: bool,
    pub(crate) back: bool,
}
impl State {
    pub(crate) fn new(rehearsal: bool) -> Self {
        Self {
            report: Report {
                events: Queue::uncapped(), elapsed: Duration::ZERO, end: None, restored: false, done: false,
            },
            rehearsal, began: None, closing: false, closed: false, back: false,
        }
    }
}

pub(crate) fn prepare(engine: &mut Engine, rehearsal: bool) {
    assert!(!engine.store.opened, "upgrade chosen before the store opens");
    // The serving adapter queues its worker listeners at construction. They are
    // not part of restoring, and no rehearsal submits them, even on a refusal.
    for _ in 0..engine.workers.host.io.len() {
        let Some(skein_io::Request::Listen { .. }) = engine.workers.host.io.pop() else {
            unreachable!("only unstarted worker listeners at construction");
        };
    }
    engine.upgrade = Some(State::new(rehearsal));
}

/// Keep the store alive until Close has emptied its scratch, even after a refusal.
pub(crate) fn advance(engine: &mut Engine) -> bool {
    let Some(state) = &mut engine.upgrade else { return false };
    let began = *state.began.get_or_insert(engine.env.now);
    if state.report.done { return false; }
    if !state.closing && (engine.domain.rehearsed() || engine.end.is_some() || state.back) {
        state.report.restored = engine.domain.rehearsed();
        state.report.end = engine.end.take();
        state.report.elapsed = engine.env.now.saturating_since(began);
        state.closing = true;
        if state.rehearsal && engine.store.host.started() {
            engine.store.host.ask(Request::Close { owner: Token::new(1) });
        } else { state.closed = true; }
    }
    if state.closing && state.closed {
        state.report.done = true;
        engine.end = Some(End::Aborted);
    }
    state.closing
}
