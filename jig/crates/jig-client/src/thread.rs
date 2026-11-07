//! A task's committed conversation, newest first, in one sliding window.
//! Keeps opaque read cursors and page-owned call folds; knows no transcript
//! encoding, application values or engine internals. `moved` reads a new tail
//! only while the window reaches it; `older` and `newer` read beside lines let go.
//! `accept` consumes one requested page and may ask for another new-tail page.
//! The owner routes reads and keeps returned object names live in its working set.
//! See client/domain/client.md, 7 and 10, and objects.md, 10.
//!
//! | State | Event | Next | Emits |
//! |---|---|---|---|
//! | idle, at newest | activity moved | latest read | activity read |
//! | idle | older, cursor | older read | activity read |
//! | idle | older, dropped edge | older read | read beside oldest line |
//! | idle | newer, cursor or dropped edge | newer read | read beside newest line |
//! | idle | latest | replacing read | activity read |
//! | latest read | page before known tail | latest read or idle | continuation or names |
//! | edge read | page | idle | names |
//! | any read | failed | idle, failed end recorded | nothing |
use self::window::{Edge, Edges, End, TurnWindow};
use crate::contract::{
    ActivityPage, ApplicationPart, ArchivedTurn, Block, CallIdentity, Cursor, DecodedActivityLine, LiveCall, ObjectKey,
    ProvisionalTurnName, Read, RunState, RunText, Side, Text, TurnName, TurnRead,
};
use skein_lib::{List, Map, Wall, bytes};
mod model;
mod order;
use order::order;
mod shown;
pub mod window;
pub mod words;
pub use model::Line;
pub use words::{KeptWords, Sent, Words};

/// Bounds on retained activity, provisional text and submitted words.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub lines: u32,
    pub bytes: u64,
    pub text: u32,
    /// The most bytes a turn being written keeps, from its start.
    pub live: u32,
}

/// The page's presentation of one call, independent of committed turn facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fold {
    /// One summary line.
    Folded,
    /// Input and outcome details are visible.
    Open,
}
/// A call is named within a turn, so calls in different attempts stay distinct.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallName {
    pub attempt: u64,
    pub turn: u32,
    pub call: u64,
}
/// The one activity read currently requested by this conversation.
#[derive(Debug)]
pub enum Reading {
    /// The newest pages, stopping at the already held committed position.
    Latest { known: u64 },
    /// Older activity at the end of the displayed window.
    Older,
    /// Activity nearer the present, after scrolling into history.
    Newer,
}
/// One turn's text is provisional until its committed turn is read.
#[derive(Clone, Debug)]
pub struct LiveTurn {
    pub name: ProvisionalTurnName,
    pub began: Option<Wall>,
    pub text: Text,
    /// The stream offset where the retained prefix begins.
    pub from: u32,
    /// The newest stream offset accepted for this retry.
    pub offset: u32,
    pub cut: bool,
    pub call: Option<LiveCall>,
}
/// Committed lines and page state for one task conversation.
#[derive(Debug)]
pub struct Thread {
    pub live: List<LiveTurn>,
    pub limits: Limits,
    pub attempt: Option<u64>,
    /// The end whose last read failed, until that end is read again.
    pub read_failed: Option<End>,
    /// Loss reported by retained activity pages, independent of turn and text loss.
    pub page_cut: bool,
    pub task: u64,
    pub lines: TurnWindow,
    /// The newest position announced by the task document.
    pub committed: u64,
    /// The newest position this conversation has read.
    pub loaded: u64,
    pub reading: Option<Reading>,
    /// Whole new-tail pages waiting to meet the held committed position.
    pub fresh: TurnWindow,
    /// The displayed calls whose details the person opened.
    pub opened: Map<CallName, ()>,
    /// Opaque current-call folds are equality-only and independent of transcript numbers.
    pub current_opened: List<CallIdentity>,
    /// The latest committed turn read, fencing stale stream deliveries.
    pub committed_turn: Option<TurnName>,
    /// The task ended its run; unread committed activity is taken before cleanup.
    pub run_finished: bool,
    /// Pending snapshots and durable words retained until the agent read them.
    pub words: KeptWords,
    /// One live chips source for tasks named by calls in this window.
    pub chips_watch: Option<crate::Handle>,
    /// Task membership shared by all displayed engine calls.
    pub chips: List<u64>,
    /// Membership is waiting for the old watch terminal or available watch room.
    pub chips_pending: bool,
}
impl Thread {
    /// Allocates the activity window and equally bounded staging and fold state.
    #[must_use]
    pub fn new(task: u64, limits: &Limits) -> Self {
        let rows = limits.lines;
        Self {
            live: List::with_capacity(rows),
            limits: *limits,
            attempt: None,
            read_failed: None,
            page_cut: false,
            task,
            lines: TurnWindow::new(rows),
            committed: 0,
            loaded: 0,
            reading: None,
            fresh: TurnWindow::new(rows),
            opened: Map::with_capacity(rows),
            current_opened: List::with_capacity(rows),
            committed_turn: None,
            run_finished: false,
            words: KeptWords::new(rows),
            chips_watch: None,
            chips: List::with_capacity(rows),
            chips_pending: false,
        }
    }
    /// Reads a newer committed tail once, leaving a ended window in place.
    pub fn moved(&mut self, committed: u64) -> Option<Read> {
        self.committed = self.committed.max(committed);
        if self.reading.is_some() || !self.lines.edges.reaches(End::Before) || self.committed <= self.loaded {
            return None;
        }
        self.clear_failed(End::Before);
        self.fresh.rows.clear();
        self.fresh.edges = Edges { before: Edge::End, after: Edge::End };
        self.reading = Some(Reading::Latest { known: self.loaded });
        Some(Read::Activity { task: self.task, before: None })
    }
    /// Reads older rows by cursor, or beside the oldest held line after a drop.
    pub fn older(&mut self) -> Option<Read> {
        if self.reading.is_some() {
            return None;
        }
        let read = match &self.lines.edges.after {
            Edge::End => return None,
            Edge::More(cursor) => Read::Activity { task: self.task, before: Some(cursor.clone()) },
            Edge::Dropped => Read::ActivityBeside {
                task: self.task,
                position: self.lines.rows.get(self.lines.rows.len().checked_sub(1)?)?.source.position,
                side: Side::Earlier,
            },
        };
        self.clear_failed(End::After);
        self.reading = Some(Reading::Older);
        Some(read)
    }
    /// Reads beside the newest held line, or checks for a new tail at the end.
    pub fn newer(&mut self) -> Option<Read> {
        if self.reading.is_some() {
            return None;
        }
        match &self.lines.edges.before {
            Edge::End => return self.moved(self.committed),
            Edge::More(_) | Edge::Dropped => {}
        }
        let position = self.lines.rows.get(0)?.source.position;
        self.clear_failed(End::Before);
        self.reading = Some(Reading::Newer);
        Some(Read::ActivityBeside { task: self.task, position, side: Side::Later })
    }
    /// Reads the list's end again, replacing the held window with its page.
    pub fn latest(&mut self) -> Option<Read> {
        if self.reading.is_some() {
            return None;
        }
        self.clear_failed(End::Before);
        self.fresh.rows.clear();
        self.fresh.edges = Edges { before: Edge::End, after: Edge::End };
        self.reading = Some(Reading::Latest { known: 0 });
        Some(Read::Activity { task: self.task, before: None })
    }
    fn clear_failed(&mut self, end: End) {
        if self.read_failed == Some(end) {
            self.read_failed = None;
        }
    }
    /// Takes a requested page; repeated rows never duplicate a committed line.
    /// The owner has validated its row and payload bounds before this call.
    pub fn accept(&mut self, page: ActivityPage) -> Option<Read> {
        let reading = self.reading.take()?;
        self.clear_failed(read_end(&reading));
        self.page_cut |= page.cut;
        self.words.observe(&page);
        let edges = page_edges(page.after, page.before);
        match reading {
            Reading::Latest { known } => {
                let available = self.fresh.rows.capacity().saturating_sub(self.fresh.rows.len());
                let mut rows = List::with_capacity(self.lines.rows.capacity());
                let mut reached = false;
                for row in page.rows.into_boxed() {
                    if row.source.position <= known {
                        reached = true;
                    } else if rows.push(row).is_err() {
                        break;
                    }
                }
                order(&mut rows);
                let fits = rows.len() <= available
                    && (self.fresh.rows.is_empty()
                        || window_weight(&self.fresh.rows).saturating_add(window_weight(&rows)) <= self.limits.bytes);
                if fits {
                    let dropped = self.fresh.slide(rows, edges, End::After);
                    assert!(dropped == 0, "whole refresh pages fit staging room");
                } else {
                    // Keep the previous page's cursor: a cursor after this page
                    // would skip its rows if only part of it fitted the window.
                    reached = false;
                }
                if known > 0
                    && fits
                    && !reached
                    && self.fresh.rows.len() < self.fresh.rows.capacity()
                    && window_weight(&self.fresh.rows) < self.limits.bytes
                {
                    match &self.fresh.edges.after {
                        Edge::More(cursor) => {
                            let cursor = cursor.clone();
                            self.reading = Some(Reading::Latest { known });
                            return Some(Read::Activity { task: self.task, before: Some(cursor) });
                        }
                        Edge::End | Edge::Dropped => {}
                    }
                }
                if let Some(row) = self.fresh.rows.get(0) {
                    self.loaded = self.loaded.max(row.source.position);
                }
                let rows = core::mem::replace(&mut self.fresh.rows, List::with_capacity(self.lines.rows.capacity()));
                let edges = core::mem::replace(&mut self.fresh.edges, Edges { before: Edge::End, after: Edge::End });
                if !reached {
                    self.lines.rows.clear();
                    self.lines.edges = Edges { before: Edge::End, after: Edge::End };
                }
                slide(&mut self.lines, rows, edges, self.limits.bytes, End::Before);
            }
            Reading::Older | Reading::Newer => {
                let end = match reading {
                    Reading::Older => End::After,
                    Reading::Newer => End::Before,
                    Reading::Latest { .. } => unreachable!("latest handled above"),
                };
                let mut rows = List::with_capacity(self.lines.rows.capacity());
                for row in page.rows.into_boxed() {
                    let mut held = false;
                    for old in &self.lines.rows {
                        if old.source.position == row.source.position {
                            held = true;
                            break;
                        }
                    }
                    if !held && rows.push(row).is_err() {
                        break;
                    }
                }
                order(&mut rows);
                if end == End::Before
                    && let Some(row) = rows.get(0)
                {
                    self.loaded = self.loaded.max(row.source.position);
                }
                slide(&mut self.lines, rows, edges, self.limits.bytes, end);
            }
        }
        // Staged pages fence live text only once their rows enter the window.
        for at in 0..self.lines.rows.len() {
            let row = self.lines.rows.get(at).expect("bounded window row exists");
            if let Some(name) = row.committed_turn() {
                self.committed(name);
            }
        }
        self.prune_live();
        self.moved(self.committed)
    }
    /// Allows a failed or refused read to be retried without inventing a page.
    pub fn failed(&mut self) {
        if let Some(reading) = self.reading.take() {
            self.read_failed = Some(read_end(&reading));
        }
        self.fresh.rows.clear();
    }
    /// Ends the provisional presentation after its committed activity has been read.
    pub fn end_run(&mut self) {
        self.live.clear();
        self.prune_folds(&[], None);
    }
    fn committed(&mut self, name: TurnName) {
        match self.committed_turn {
            Some(old) => {
                if later(name, old) {
                    self.committed_turn = Some(name);
                }
            }
            None => self.committed_turn = Some(name),
        }
    }
    /// Connector-owned parts carried by calls, results or news, requested by the root.
    #[must_use]
    pub fn parts(&self, capacity: u32, live: &[LiveTurn]) -> List<ApplicationPart> {
        let mut parts = List::with_capacity(capacity);
        for line in &self.lines.rows {
            if let Some(ArchivedTurn { turn: TurnRead::Decoded(turn), .. }) = &line.decoded {
                for message in &turn.messages {
                    for block in &message.blocks {
                        match block {
                            Block::Call(call) => {
                                for part in &call.parts {
                                    add_part(&mut parts, part);
                                }
                            }
                            Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
                        }
                    }
                }
            }
        }
        for live in live {
            if let Some(call) = &live.call {
                match call {
                    LiveCall::Decoded(call) => {
                        for part in &call.parts {
                            add_part(&mut parts, part);
                        }
                    }
                    LiveCall::Raw(_) => {}
                }
            }
        }
        parts
    }
    /// Default folds are implicit; only opened calls consume page fold room.
    #[must_use]
    pub fn fold(&self, name: CallName) -> Fold {
        if self.opened.contains_key(&name) { Fold::Open } else { Fold::Folded }
    }
    fn displayed_call(&self, name: CallName, live: &[LiveTurn]) -> bool {
        if has_call(&self.lines.rows, name) {
            return true;
        }
        for live in live {
            if live.name.attempt == name.attempt
                && live.name.number == name.turn
                && let Some(LiveCall::Decoded(call)) = &live.call
                && call.number == name.call
            {
                return true;
            }
        }
        false
    }
    /// Opens or closes one displayed call, refusing unknown calls or full fold room.
    pub fn set_fold(&mut self, name: CallName, fold: Fold, live: &[LiveTurn]) -> bool {
        if !self.displayed_call(name, live) {
            return false;
        }
        match fold {
            Fold::Folded => {
                self.opened.remove(&name);
                true
            }
            Fold::Open => {
                if !self.opened.contains_key(&name)
                    && match self.opened.len().checked_add(self.current_opened.len()) {
                        Some(total) => total >= self.opened.capacity(),
                        None => true,
                    }
                {
                    return false;
                }
                self.opened.insert(name, ()).is_ok()
            }
        }
    }
    /// Presentation state for a raw call is keyed by its supplied opaque triple.
    #[must_use]
    pub fn current_fold(&self, identity: &CallIdentity) -> Fold {
        if self.current_opened.as_slice().contains(identity) { Fold::Open } else { Fold::Folded }
    }
    fn displayed_current_call(identity: &CallIdentity, live: &[LiveTurn], current_run: Option<&RunState>) -> bool {
        if let Some(run) = current_run {
            return match &run.call {
                Some(call) => &call.identity == identity,
                None => false,
            };
        }
        for live in live {
            if let Some(LiveCall::Raw(call)) = &live.call
                && &call.identity == identity
            {
                return true;
            }
        }
        false
    }
    /// Opening an unknown identity or exceeding the bounded fold table is refused.
    pub fn set_current_fold(
        &mut self,
        identity: CallIdentity,
        fold: Fold,
        live: &[LiveTurn],
        current_run: Option<&RunState>,
    ) -> bool {
        if !Self::displayed_current_call(&identity, live, current_run) {
            return false;
        }
        match fold {
            Fold::Open => {
                if self.current_opened.as_slice().contains(&identity) {
                    return true;
                }
                if match self.opened.len().checked_add(self.current_opened.len()) {
                    Some(total) => total >= self.current_opened.capacity(),
                    None => true,
                } {
                    return false;
                }
                self.current_opened.push(identity).is_ok()
            }
            Fold::Folded => {
                let mut kept = List::with_capacity(self.current_opened.capacity());
                for name in &self.current_opened {
                    if name != &identity {
                        kept.push(name.clone()).expect("folds remain bounded");
                    }
                }
                self.current_opened = kept;
                true
            }
        }
    }
    pub fn prune_folds(&mut self, live: &[LiveTurn], current_run: Option<&RunState>) {
        let mut gone = List::with_capacity(self.opened.capacity());
        for (name, ()) in &self.opened {
            if !self.displayed_call(*name, live) {
                gone.push(*name).expect("removed folds fit fold capacity");
            }
        }
        for name in gone.into_boxed() {
            self.opened.remove(&name);
        }
        let mut kept = List::with_capacity(self.current_opened.capacity());
        for identity in &self.current_opened {
            if Self::displayed_current_call(identity, live, current_run) {
                kept.push(identity.clone()).expect("remaining current folds fit");
            }
        }
        self.current_opened = kept;
    }
    /// Names cards produced by calls and heard results, deduplicated by object key.
    #[must_use]
    pub fn names(&self, capacity: u32, live: &[LiveTurn]) -> List<ObjectKey> {
        let mut names = List::with_capacity(capacity);
        for line in &self.lines.rows {
            if let Some(ArchivedTurn { turn: TurnRead::Decoded(turn), .. }) = &line.decoded {
                for message in &turn.messages {
                    for block in &message.blocks {
                        match block {
                            Block::Call(call) => {
                                for key in &call.objects {
                                    add_name(&mut names, *key);
                                }
                            }
                            Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
                        }
                    }
                }
            }
        }
        for live in live {
            if let Some(call) = &live.call {
                match call {
                    LiveCall::Decoded(call) => {
                        for key in &call.objects {
                            add_name(&mut names, *key);
                        }
                    }
                    LiveCall::Raw(_) => {}
                }
            }
        }
        names
    }
}
fn add_name(names: &mut List<ObjectKey>, key: ObjectKey) {
    if !names.as_slice().contains(&key) {
        match names.push(key) {
            Ok(()) | Err(_) => {}
        }
    }
}
fn page_edges(newer: Option<Cursor>, older: Option<Cursor>) -> Edges {
    Edges {
        before: match newer {
            Some(cursor) => Edge::More(cursor),
            None => Edge::End,
        },
        after: match older {
            Some(cursor) => Edge::More(cursor),
            None => Edge::End,
        },
    }
}
fn has_call(rows: &List<DecodedActivityLine>, name: CallName) -> bool {
    for row in rows {
        if let Some((turn_name, turn)) = row.decoded_turn()
            && turn_name.attempt == name.attempt
            && turn_name.number == name.turn
        {
            for message in &turn.messages {
                for block in &message.blocks {
                    match block {
                        Block::Call(call) => {
                            if call.number == name.call {
                                return true;
                            }
                        }
                        Block::Text(_) | Block::Result { .. } | Block::Opaque { .. } => {}
                    }
                }
            }
        }
    }
    false
}

#[must_use]
pub fn live_turn(value: RunText, limit: u32) -> LiveTurn {
    let cut = value.cut || value.from > 0 || !value.text.fits(limit);
    let text = prefix(value.text.bytes(), limit);
    LiveTurn {
        name: value.turn,
        began: value.began,
        text,
        from: value.from,
        offset: value.through,
        cut,
        call: value.call,
    }
}

fn extend_live(held: &mut LiveTurn, value: &RunText, limit: u32) {
    if value.from == 0 {
        *held = live_turn(value.clone(), limit);
        return;
    }
    if value.through < held.offset {
        return;
    }
    held.began = value.began.or(held.began);
    held.call.clone_from(&value.call);
    held.offset = value.through;
    let end = held
        .from
        .checked_add(u32::try_from(held.text.bytes().len()).expect("bounded prefix fits"))
        .expect("prefix is within stream offsets");
    let Some(skip) = end.checked_sub(value.from) else {
        held.cut = true;
        return;
    };
    let Some(suffix) = value.text.bytes().get(skein_lib::length::usize_of(skip)..) else { return };
    let room = skein_lib::length::usize_of(limit).saturating_sub(held.text.bytes().len());
    let kept = bytes::utf8_prefix(suffix, room);
    if !kept.is_empty() {
        let joined = bytes::concat(&[held.text.bytes(), kept]);
        held.text = Text::new(joined, limit).expect("joined prefix stays within its bound");
    }
    held.cut |= kept.len() < suffix.len();
}

#[must_use]
pub fn later(name: TurnName, other: TurnName) -> bool {
    name.attempt > other.attempt || (name.attempt == other.attempt && name.number > other.number)
}
fn prefix(value: &[u8], limit: u32) -> Text {
    let kept = bytes::utf8_prefix(value, skein_lib::length::usize_of(limit));
    Text::new(bytes::copy_of(kept), limit).expect("prefix respects its byte bound")
}
fn add_part(parts: &mut List<ApplicationPart>, part: &ApplicationPart) {
    if !parts.as_slice().contains(part) {
        match parts.push(part.clone()) {
            Ok(()) | Err(_) => {}
        }
    }
}

fn read_end(reading: &Reading) -> End {
    match reading {
        Reading::Older => End::After,
        Reading::Newer | Reading::Latest { .. } => End::Before,
    }
}
fn window_weight(rows: &List<DecodedActivityLine>) -> u64 {
    let mut weight = 0_u64;
    for row in rows {
        weight = weight.saturating_add(row.weight());
    }
    weight
}
fn slide(window: &mut TurnWindow, rows: List<DecodedActivityLine>, edges: Edges, bytes: u64, end: End) {
    let keep = rows.len();
    let mut dropped = window.slide(rows, edges, end);
    let mut weight = window_weight(&window.rows);
    let mut removed = 0_u32;
    for offset in 0..window.rows.len().saturating_sub(keep) {
        if weight <= bytes {
            break;
        }
        let at = match end {
            End::After => offset,
            End::Before => window.rows.len().saturating_sub(offset).saturating_sub(1),
        };
        weight = weight.saturating_sub(window.rows.get(at).expect("bounded row exists").weight());
        removed = removed.saturating_add(1);
    }
    if removed > 0 {
        let count = window.rows.len().saturating_sub(removed);
        let start = match end {
            End::After => removed,
            End::Before => 0,
        };
        let old = core::mem::replace(&mut window.rows, List::with_capacity(0));
        window.rows = List::with_capacity(old.capacity());
        for (at, row) in old.into_boxed().into_iter().enumerate() {
            let at = u32::try_from(at).expect("bounded row index fits");
            if at >= start && at < start.saturating_add(count) {
                window.rows.push(row).expect("retained rows fit");
            }
        }
        dropped = dropped.saturating_add(removed);
    }
    if dropped > 0 {
        match end {
            End::After => window.edges.before = Edge::Dropped,
            End::Before => window.edges.after = Edge::Dropped,
        }
    }
}
