//! Stopped store tools: rehearsal, going back and restore (protocol/binaries.md; protocol/checkpoints.md, 2.5).
use crate::{End, engine::Engine};
use alloc::boxed::Box;
use jig_protocol_store::{Event, Request};
use skein_lib::{Duration, List, Queue, Time, Token};

/// What the configured engine would require at an opening (protocol/wires.md, 10.2).
#[derive(Debug)]
pub struct Workers {
    pub room: u32,
    pub schema: skein_channel::Schema,
}

/// The store's ordered progress and terminal, retained through stopped-tool teardown.
#[derive(Debug)]
pub struct Report {
    pub events: Queue<Event>,
    /// The requested checkpoint, retained to name a restore's source refusal.
    pub checkpoint: Option<jig_protocol_store::CheckpointName>,
    pub workers: Workers,
    pub elapsed: Duration,
    pub end: Option<End>,
    pub restored: bool,
    /// Each kept forge's catching-up state at the opening, in number order.
    pub catching_up: Box<[(u16, Option<temper_engine_domain_forge::CatchingUp>)]>,
    pub done: bool,
    pub open_failure: Option<jig_protocol_store::restore::OpenFailure>,
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "operation kind, close request, close terminal and stopped operation terminal are independent"
)]
pub(crate) struct State {
    pub(crate) report: Report,
    rehearsal: bool,
    began: Option<Time>,
    closing: bool,
    pub(crate) closed: bool,
    pub(crate) terminal: bool,
    pub(crate) undecodable: bool,
}
impl State {
    pub(crate) fn new(rehearsal: bool, workers: Workers) -> Self {
        Self {
            report: Report {
                events: Queue::uncapped(),
                checkpoint: None,
                workers,
                elapsed: Duration::ZERO,
                end: None,
                restored: false,
                catching_up: Box::new([]),
                done: false,
                open_failure: None,
            },
            rehearsal,
            began: None,
            closing: false,
            closed: false,
            terminal: false,
            undecodable: false,
        }
    }
}

pub(crate) fn prepare(engine: &mut Engine, rehearsal: bool) {
    prepare_stopped(engine);
    engine.forge.rehearsal = rehearsal;
    engine.upgrade = Some(State::new(rehearsal, engine.workers.judgement()));
}

/// A rehearsal whose copy its shell opened counts from before that opening.
pub(crate) fn prepare_opened(engine: &mut Engine, began: Time) {
    prepare(engine, true);
    engine.upgrade.as_mut().expect("prepared rehearsal").began = Some(began);
}

/// Shut down every unstarted owner except the store, for a stopped-deployment tool.
pub(crate) fn prepare_stopped(engine: &mut Engine) {
    assert!(!engine.store.opened, "upgrade chosen before the store opens");
    // The serving adapter queues its worker listeners at construction. They are
    // not part of restoring, and no rehearsal submits them, even on a refusal.
    for _ in 0..engine.workers.host.io.len() {
        let Some(skein_io::Request::Listen { .. }) = engine.workers.host.io.pop() else {
            unreachable!("only unstarted worker listeners at construction");
        };
    }
    engine.clients.abort(true);
    // Restoring owns only the store; the other owners close before any I/O.
    if let Some(accounts) = &mut engine.accounts {
        accounts.abort();
    }
    engine.workers.abort(engine.env.now, &mut engine.events);
    engine.setup.close();
    engine.forge.down(skein_forge_connection::Request::Close);
    crate::engine::forge_pass(engine);
}

/// Keep the store alive until Close has emptied its scratch, even after a refusal.
pub(crate) fn advance(engine: &mut Engine) -> bool {
    let Some(state) = &mut engine.upgrade else { return false };
    let began = *state.began.get_or_insert(engine.env.now);
    if state.report.done {
        return false;
    }
    if !state.closing && (engine.domain.rehearsed() || engine.end.is_some() || state.terminal || state.undecodable) {
        state.report.restored = engine.domain.rehearsed();
        if state.report.restored {
            let forges = engine.domain.forges();
            let mut catching_up =
                List::with_capacity(u32::try_from(forges.rows.len()).expect("configured forges fit u32"));
            for row in forges.rows {
                catching_up.push((row.number, row.catching_up)).expect("one state per kept forge");
            }
            state.report.catching_up = catching_up.into_boxed();
        }
        state.report.end = engine.end.take();
        state.report.elapsed = engine.env.now.saturating_since(began);
        state.closing = true;
        if state.rehearsal && !state.closed && engine.store.host.started() {
            engine.store.host.ask(Request::Close { owner: Token::new(1) });
        } else {
            state.closed = true;
        }
    }
    if state.closing && state.closed {
        state.report.done = true;
        engine.end = Some(End::Aborted);
    }
    state.closing
}
