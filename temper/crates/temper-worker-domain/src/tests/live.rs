//! A run's live content through the root: its ask out with its start, and
//! smith's pieces up to the hub, each also progress for the agent's watchdog
//! (`domain/worker.md`, section 7; jig's `domain/hosts.md`, 6.5).

use super::fixtures::{ATTEMPT, RUN, assignment};
use super::harness::Harness;
use crate::{Event, Request, host};
use alloc::boxed::Box;
use skein_lib::{Duration, Time, Token};

const TEXT: host::LiveAsk = host::LiveAsk { text: true, reasoning: false, input: false };
/// How long the agent waits before its piece comes.
const LATER: Duration = Duration::from_secs(1);

fn piece() -> host::Piece {
    host::Piece { turn: 1, retry: 0, kind: host::LiveKind::Text, offset: 0, bytes: Box::from(&b"Hel"[..]) }
}

/// The run's pieces among `rows`, as they go up the link.
fn sent(rows: &[Request]) -> u32 {
    let mut pieces: u32 = 0;
    for row in rows {
        if let Request::Link(host::LinkUp::Piece { run, attempt, piece: sent }) = row {
            assert_eq!((*run, *attempt, sent), (RUN, ATTEMPT, &piece()));
            pieces = pieces.saturating_add(1);
        }
    }
    pieces
}

/// A started run whose agent says `piece()` a second after its watchdog was
/// last fed, from the agent `from` (the run's own when `None`): what the root
/// sent, and the watchdog's deadline before and after.
fn said(live: host::LiveAsk, from: Option<Token>) -> (Box<[Request]>, Option<Time>, Option<Time>) {
    let mut h = Harness::new();
    h.started_asking(live);
    let before = h.domain.next_deadline();
    h.env.now = h.env.now.saturating_add(LATER);
    let agent = match from {
        Some(agent) => agent,
        None => h.agent.expect("a spawned agent"),
    };
    let rows = h.step(Event::Live { agent, piece: piece() });
    (rows, before, h.domain.next_deadline())
}

#[test]
fn the_assignments_ask_goes_out_with_its_start() {
    for live in [host::LiveAsk::NONE, TEXT] {
        let mut h = Harness::new();
        let mut assigned = assignment(false, Box::new([]));
        assigned.assignment.live = live;
        let mut asked = None;
        for row in h.step(Event::Link(host::LinkDown::Assign { assignment: assigned })) {
            if let Request::Decode { start, .. } = row {
                asked = Some(start.live);
            }
        }
        assert_eq!(asked, Some(live));
    }
}

#[test]
fn an_asked_runs_piece_goes_up_and_feeds_the_watchdog() {
    let (rows, before, after) = said(TEXT, None);
    assert_eq!(sent(&rows), 1);
    let before = before.expect("the watchdog is armed");
    assert_eq!(after, Some(before.saturating_add(LATER)));
}

#[test]
fn an_unasked_runs_piece_goes_no_further_but_feeds_the_watchdog() {
    let (rows, before, after) = said(host::LiveAsk::NONE, None);
    assert_eq!(sent(&rows), 0);
    let before = before.expect("the watchdog is armed");
    assert_eq!(after, Some(before.saturating_add(LATER)));
}

#[test]
fn a_piece_from_an_agent_the_root_does_not_know_goes_nowhere() {
    let (rows, before, after) = said(TEXT, Some(Token::new(u64::MAX)));
    assert!(rows.is_empty(), "{rows:?}");
    assert_eq!(after, before);
}
