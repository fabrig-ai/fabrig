//! Sessions read back from committed activity (client/pages/team.md, 1, Its sessions).
use super::super::{Request, activity::Asking};
use super::new_handle;
use alloc::boxed::Box;
use jig_client::{Handle, Readout, Told, contract as c};
use skein_lib::{Map, Queue, Set, Time, Wall};

const WEEK: u64 = 604_800_000_000_000;
const MOST: u32 = 8;
const PAGES: u32 = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub start: Wall,
    /// None while any run has no matching end after its beginning.
    pub end: Option<Wall>,
    pub words: Option<Box<[u8]>>,
    pub spent: u64,
}
#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Began(u64),
    Ended { attempt: u64, spent: u64 },
    Stood { attempt: u64, words: Box<[u8]> },
    NowWritten { attempt: u64, line: Box<[u8]> },
    Boundary,
    Other,
}
#[derive(Clone, Debug)]
pub(crate) struct Line {
    pub at: Wall,
    pub kind: Kind,
}
pub(crate) fn retain(line: &c::ActivityLine) -> Line {
    let kind = match &line.line {
        c::Activity::RunBegan { attempt, .. } => Kind::Began(*attempt),
        c::Activity::RunEnded { attempt, spent, .. } => Kind::Ended { attempt: *attempt, spent: *spent },
        c::Activity::Stood { attempt, words } => Kind::Stood { attempt: *attempt, words: words.bytes().into() },
        c::Activity::NowWritten { attempt, line } => Kind::NowWritten { attempt: *attempt, line: line.bytes().into() },
        c::Activity::SessionEnded { .. } => Kind::Boundary,
        c::Activity::TalkOpened { .. }
        | c::Activity::TalkEnded { .. }
        | c::Activity::Words { .. }
        | c::Activity::Heard(_)
        | c::Activity::Notified { .. }
        | c::Activity::Turn { .. }
        | c::Activity::Decided { .. }
        | c::Activity::Milestone { .. }
        | c::Activity::Kept { .. } => Kind::Other,
    };
    Line { at: line.at, kind }
}
fn recent(at: Wall, wall: Wall) -> bool {
    at.as_nanos() >= wall.as_nanos().saturating_sub(WEEK) && at <= wall
}
/// Commit order, not wall-time order, determines the boundaries and last words.
#[must_use]
pub(crate) fn group(lines: &Map<u64, Line>, start: bool, wall: Wall) -> skein_lib::List<Session> {
    let mut all = skein_lib::List::uncapped();
    let mut bounded = start;
    let mut session: Option<Session> = None;
    let mut wrote_now = false;
    let mut live = Set::with_capacity(lines.len());
    for (_, line) in lines {
        match &line.kind {
            Kind::Boundary => {
                finish(&mut all, session.take(), bounded, live.is_empty(), wall);
                live = Set::with_capacity(lines.len());
                bounded = true;
                wrote_now = false;
            }
            Kind::Began(attempt) => {
                if session.is_none() {
                    session = Some(Session { start: line.at, end: None, words: None, spent: 0 });
                }
                live.insert(*attempt).expect("at most one live attempt per held line");
            }
            Kind::Ended { attempt, spent } => {
                live.remove(attempt);
                if let Some(session) = &mut session {
                    session.end = Some(line.at);
                    session.spent = session.spent.saturating_add(*spent);
                }
            }
            Kind::Stood { attempt, words } => {
                let _: u64 = *attempt;
                if !wrote_now && let Some(session) = &mut session {
                    session.words = one_line(words);
                }
            }
            Kind::NowWritten { attempt, line } => {
                let _: u64 = *attempt;
                if let Some(session) = &mut session {
                    session.words = one_line(line);
                    wrote_now = true;
                }
            }
            Kind::Other => {}
        }
    }
    finish(&mut all, session, bounded, live.is_empty(), wall);
    let mut rows = skein_lib::List::with_capacity(MOST);
    for row in all.iter().rev().take(usize::try_from(MOST).expect("eight fits")) {
        rows.push(row.clone()).expect("at most eight sessions");
    }
    rows
}
fn one_line(words: &[u8]) -> Option<Box<[u8]>> {
    if words.is_empty() {
        return None;
    }
    let mut line = skein_lib::List::with_capacity(u32::try_from(words.len()).expect("activity words fit text"));
    let mut carriage_return = false;
    for byte in words {
        if *byte == b'\n' && carriage_return {
            carriage_return = false;
            continue;
        }
        carriage_return = *byte == b'\r';
        line.push(if *byte == b'\n' || carriage_return { b' ' } else { *byte }).expect("line never grows");
    }
    Some(line.into_boxed())
}
fn finish(rows: &mut skein_lib::List<Session>, session: Option<Session>, bounded: bool, ended: bool, wall: Wall) {
    if let Some(mut session) = session
        && bounded
        && recent(session.start, wall)
    {
        if !ended {
            session.end = None;
        }
        rows.push(session).expect("uncapped grouped sessions");
    }
}

/// Unlike a conversation Thread, this retains no decoded turns or transcript window.
/// One bounded backwards read, refreshed by position, with the conversation's retry policy.
#[derive(Debug)]
pub(super) struct Reader {
    task: Option<u64>,
    handle: Option<Handle>,
    activity: Option<u64>,
    loaded_activity: Option<u64>,
    reading: Option<ReadAt>,
    asked: Asking,
    lines: Map<u64, Line>,
    fresh: Map<u64, Line>,
    start: bool,
    pages: u32,
    readable: bool,
    wall: Wall,
}
#[derive(Debug)]
struct ReadAt {
    activity: Option<u64>,
}
impl Default for Reader {
    fn default() -> Self {
        Self {
            task: None,
            handle: None,
            activity: None,
            loaded_activity: None,
            reading: None,
            asked: Asking::new(),
            lines: Map::uncapped(),
            fresh: Map::uncapped(),
            start: false,
            pages: 0,
            readable: false,
            wall: Wall::EPOCH,
        }
    }
}
impl Reader {
    pub(super) fn configure(
        &mut self,
        wanted: Option<(u64, Option<u64>)>,
        wall: Wall,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let task = if let Some((task, _)) = wanted { Some(task) } else { None };
        if self.task != task {
            self.close(out);
            self.task = task;
        }
        self.wall = wall;
        let Some((task, activity)) = wanted else { return };
        self.activity = activity;
        if self.handle.is_none() || (self.reading.is_none() && self.activity != self.loaded_activity) {
            if let Some(handle) = self.handle.take() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
            let handle = new_handle(next);
            self.handle = Some(handle);
            self.reading = Some(ReadAt { activity });
            self.pages = 0;
            self.fresh = Map::uncapped();
            self.asked.ask(handle, c::Read::Activity { task, before: None }, out);
        }
    }
    pub(super) fn close(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        *self = Self::default();
    }
    pub(super) const fn wall(&self) -> Wall {
        self.wall
    }
    pub(super) fn rows(&self, wall: Wall) -> skein_lib::List<Session> {
        if !self.readable {
            return skein_lib::List::with_capacity(0);
        }
        if self.lines.is_empty() { group(&self.fresh, false, wall) } else { group(&self.lines, self.start, wall) }
    }
    pub(super) fn told(&mut self, now: Time, wall: Wall, told: &Told, out: &mut Queue<Request>) -> bool {
        self.wall = wall;
        match told {
            Told::Page { handle, page } if self.handle == Some(*handle) && self.reading.is_some() => {
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = page {
                    self.accept(page, out);
                } else {
                    self.readable = false;
                    self.asked.ended(now);
                }
                true
            }
            Told::Ended { handle, .. } if self.handle == Some(*handle) && self.reading.is_some() => {
                self.readable = false;
                self.asked.ended(now);
                true
            }
            Told::Page { .. }
            | Told::Ended { .. }
            | Told::Changed { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Link { .. }
            | Told::Text { .. } => false,
        }
    }
    fn accept(&mut self, page: &c::ActivityPage, out: &mut Queue<Request>) {
        self.asked.answered();
        self.readable = true;
        self.pages = self.pages.saturating_add(1);
        let mut joined = false;
        let mut old = false;
        for row in &page.rows {
            if Some(row.source.task) != self.task {
                continue;
            }
            joined |= self.lines.contains_key(&row.source.position);
            let line = retain(&row.source);
            match line.kind {
                Kind::Began(_) | Kind::Boundary => {
                    old |= line.at.as_nanos() < self.wall.as_nanos().saturating_sub(WEEK);
                }
                Kind::Ended { .. } | Kind::Stood { .. } | Kind::NowWritten { .. } | Kind::Other => {}
            }
            self.fresh.insert(row.source.position, line).expect("uncapped activity");
        }
        if joined {
            for (position, line) in &self.lines {
                if !self.fresh.contains_key(position) {
                    self.fresh.insert(*position, line.clone()).expect("uncapped activity");
                }
            }
        }
        let start = page.before.is_none() || (joined && self.start);
        let stop = joined || start || old || self.pages >= PAGES || group(&self.fresh, start, self.wall).len() >= MOST;
        if stop {
            self.lines = core::mem::replace(&mut self.fresh, Map::uncapped());
            self.start = start;
            self.readable = true;
            if let Some(reading) = self.reading.take() {
                self.loaded_activity = reading.activity;
            }
        } else if let (Some(task), Some(handle)) = (self.task, self.handle) {
            self.asked.ask(handle, c::Read::Activity { task, before: page.before.clone() }, out);
        }
    }
    pub(super) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            self.asked.fire(now, handle, out);
        }
    }
    pub(super) const fn next_deadline(&self) -> Option<Time> {
        self.asked.next_deadline()
    }
}
