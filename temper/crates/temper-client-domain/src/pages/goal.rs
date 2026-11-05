//! A goal's reads and pure page facts (temper/docs/design/client/README.md, 5.8).
use super::{Request, delegation, goals, report, task};
use crate::owner::Owner;
use alloc::boxed::Box;
use jig_client::{Handle, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, Wall, bytes};

/// A goal's direct delegates, cancelled ones left out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    pub done: u32,
    pub total: u32,
}

/// What a goal's page reads beside its task page's own.
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the view reads the report and line; read bookkeeping belongs to the page"
)]
pub struct Goal {
    pub report: Option<report::Report>,
    pub line: Option<c::TaskLine>,
    report_handle: Handle,
    line_handle: Handle,
    project: u32,
    number: u64,
    query: c::TaskQuery,
    activity: Wall,
    changed: Wall,
    report_at: Option<Wall>,
    reporting: Option<Wall>,
    line_at: Option<(Wall, Wall)>,
    searching: Option<(Wall, Wall)>,
    pages: u32,
    ended_group: bool,
}
impl Goal {
    pub(super) fn new(task: &c::Task, next: &mut u64, out: &mut Queue<Request>) -> Self {
        let report_handle = handle(*next);
        *next = next.checked_add(1).expect("goal handles fit");
        let line_handle = handle(*next);
        *next = next.checked_add(1).expect("goal handles fit");
        let mut goal = Self {
            report: None,
            line: None,
            report_handle,
            line_handle,
            project: task.project,
            number: task.chip.task,
            query: own_line_query(task),
            activity: task.last_activity,
            changed: task.changed,
            report_at: None,
            reporting: None,
            line_at: None,
            searching: None,
            pages: 0,
            ended_group: false,
        };
        goal.reconcile(out);
        goal
    }
    pub(super) fn changed(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        self.activity = task.last_activity;
        self.changed = task.changed;
        self.query = own_line_query(task);
        self.reconcile(out);
    }
    fn reconcile(&mut self, out: &mut Queue<Request>) {
        if self.reporting.is_none() && self.report_at != Some(self.activity) {
            self.reporting = Some(self.activity);
            read(self.report_handle, report::read(self.project, self.number), out);
        }
        if self.searching.is_none() && self.line_at != Some((self.changed, self.activity)) {
            self.searching = Some((self.changed, self.activity));
            self.pages = 0;
            self.ended_group = false;
            self.line = None;
            self.read_line(None, None, out);
        }
    }
    fn read_line(&mut self, live: Option<c::Cursor>, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        self.pages = self.pages.checked_add(1).expect("at most eight goal pages");
        read(self.line_handle, c::Read::Tasks { query: self.query, live, ended }, out);
    }
    pub(super) fn told(&mut self, told: &Told, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Page { handle, page } if *handle == self.report_handle => {
                if let Some(at) = self.reporting.take() {
                    self.report_at = Some(at);
                    self.report = match page {
                        Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) => {
                            match report::newest(&page.rows) {
                                Some(note) => Some(report::parse(note)),
                                None => None,
                            }
                        }
                        Readout::Jig(c::ReadResult::Value(_) | c::ReadResult::None(_))
                        | Readout::Family { .. }
                        | Readout::NotServed { .. }
                        | Readout::Limit => None,
                    };
                    self.reconcile(out);
                }
                true
            }
            Told::Page { handle, page } if *handle == self.line_handle => {
                if let Some(at) = self.searching {
                    if at == (self.changed, self.activity)
                        && let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page
                    {
                        let ended = match &page.ended {
                            c::EndedGroup::Rows { rows, .. } => rows.as_slice(),
                            c::EndedGroup::None => &[],
                        };
                        for row in page.live.iter().chain(ended) {
                            if row.chip.task == self.number {
                                self.line = Some(row.clone());
                                break;
                            }
                        }
                        if self.line.is_none() && self.pages < 8 {
                            if !self.ended_group
                                && let Some(after) = &page.live_after
                            {
                                self.read_line(Some(after.clone()), None, out);
                                return true;
                            }
                            if let c::EndedGroup::Rows { after: Some(after), .. } = &page.ended {
                                self.ended_group = true;
                                self.read_line(None, Some(after.clone()), out);
                                return true;
                            }
                        }
                    }
                    self.line_at = self.searching.take();
                    self.reconcile(out);
                }
                true
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => false,
        }
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        for handle in [self.report_handle, self.line_handle] {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }
    #[must_use]
    pub fn counts(&self, task: &c::Task, ended: Option<&c::TaskPage>) -> Counts {
        if let Some(line) = &self.line {
            return Counts { done: line.delegates_done, total: line.delegates_total };
        }
        let mut counts = Counts { done: 0, total: task.chip.details.details.live_delegates };
        if let Some(page) = ended
            && let c::EndedGroup::Rows { rows, .. } = &page.ended
        {
            for row in rows {
                match line(&row.chip, false) {
                    Some(Line::Landed) => {
                        counts.done = counts.done.checked_add(1).expect("delegate count fits");
                        counts.total = counts.total.checked_add(1).expect("delegate count fits");
                    }
                    None => {}
                    Some(
                        Line::Running
                        | Line::WaitsOn(_)
                        | Line::WaitsOnTasks(_)
                        | Line::WaitsOnYou
                        | Line::WaitsOnResources
                        | Line::Held
                        | Line::Failed
                        | Line::Waits,
                    ) => counts.total = counts.total.checked_add(1).expect("delegate count fits"),
                }
            }
        }
        counts
    }
    #[must_use]
    pub fn state(&self, task: &c::Task, ended: Option<&c::TaskPage>, needs: bool) -> goals::State {
        goals::state(&task.chip, needs, self.report.is_some(), self.counts(task, ended).total)
    }
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn read(handle: Handle, read: c::Read, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
}

/// The goal's own query keys, ordered by task number.
#[must_use]
pub fn own_line_query(task: &c::Task) -> c::TaskQuery {
    c::TaskQuery {
        project: Some(task.project),
        requester: Some(task.requester),
        label: Some(task.label),
        tracked: Some(true),
        order: c::QueryOrder::Number,
        root: None,
        parent: None,
        phase: None,
        executor: None,
    }
}

/// The draft's path and chat's number, each by a line's start alone.
#[must_use]
pub fn draft_of(words: &[u8]) -> (Option<Box<[u8]>>, Option<u64>) {
    let mut draft = None;
    let mut chat = None;
    let mut start = 0;
    for end in 0..=words.len() {
        if end != words.len() && words.get(end) != Some(&b'\n') {
            continue;
        }
        let line = words.get(start..end).expect("line boundaries are within the words");
        start = end.saturating_add(1);
        if let Some(path) = line.strip_prefix(b"Draft: ") {
            draft = Some(bytes::copy_of(path.trim_ascii_end()));
        } else if let Some(number) = line.strip_prefix(b"Chat: T") {
            chat = decimal(number.trim_ascii_end());
        }
    }
    (draft, chat)
}
/// A path's last segment.
#[must_use]
pub fn file_of(path: &[u8]) -> &[u8] {
    let mut start = 0;
    for (at, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            start = at.checked_add(1).expect("slash lies within the path");
        }
    }
    path.get(start..).expect("the last segment starts within the path")
}
fn decimal(words: &[u8]) -> Option<u64> {
    if words.is_empty() || (words.len() > 1 && words.first() == Some(&b'0')) {
        return None;
    }
    let mut value = 0_u64;
    for byte in words {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}
/// Whether a resource's path ends in PREFIX/root/groot, with equal numbers.
#[must_use]
pub fn holds_branch(task: &c::Task) -> bool {
    for resource in &task.resources {
        if let c::PartKey::Path(path) = &resource.part.key {
            let mut tail = path.iter().rev();
            if let (Some(goal), Some(root), Some(prefix)) = (tail.next(), tail.next(), tail.next())
                && prefix.as_ref() == b"temper"
                && let Some(goal) = goal.strip_prefix(b"g")
                && let (Some(root), Some(goal)) = (decimal(root), decimal(goal))
                && root == goal
            {
                return true;
            }
        }
    }
    false
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    Running,
    WaitsOn(u64),
    WaitsOnTasks(u32),
    WaitsOnYou,
    WaitsOnResources,
    Held,
    Failed,
    Waits,
    Landed,
}
/// A delegate's words, cancelled delegates left out; first match wins.
#[must_use]
pub fn line(chip: &c::Chip, yours: bool) -> Option<Line> {
    match &chip.details.phase {
        c::ChipPhase::Ended => {
            return match &chip.details.details.ending {
                Some(c::ChipEnding::Done(_)) => Some(Line::Landed),
                Some(c::ChipEnding::Failed(_)) => Some(Line::Failed),
                Some(c::ChipEnding::Cancelled { .. }) => None,
                None => Some(Line::Waits),
            };
        }
        c::ChipPhase::Active(_)
        | c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Held { .. } => {}
    }
    if chip.phase == c::Phase::Running {
        return Some(Line::Running);
    }
    if yours {
        return Some(Line::WaitsOnYou);
    }
    if chip.phase == c::Phase::Held {
        return Some(Line::Held);
    }
    Some(match task::row_state(chip) {
        task::State::WaitingFor(task) => Line::WaitsOn(task),
        task::State::WaitingForTasks { count, .. } => Line::WaitsOnTasks(count),
        task::State::WaitingForHoldings => Line::WaitsOnResources,
        task::State::Phase(_)
        | task::State::WaitingForDelegates(_)
        | task::State::WaitingForYou
        | task::State::WaitingForParty(_)
        | task::State::WaitingForNews
        | task::State::Running
        | task::State::Writing
        | task::State::Tool(_)
        | task::State::Ended(_) => Line::Waits,
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fallback {
    Planning { draft: Option<Box<[u8]>> },
    Stopped,
    Landed { first_line: Option<Box<[u8]>> },
    Failed { reason: Box<[u8]> },
    Tasks(Counts),
}
#[must_use]
pub fn fallback(state: goals::State, task: &c::Task, counts: Counts) -> Fallback {
    match state {
        goals::State::Landed => Fallback::Landed {
            first_line: match said(task) {
                Some(words) => Some(bytes::copy_of(first_line(words))),
                None => None,
            },
        },
        goals::State::Stopped => Fallback::Stopped,
        goals::State::Failed => Fallback::Failed { reason: bytes::copy_of(said(task).unwrap_or_default()) },
        goals::State::Planning => planning(task),
        goals::State::Needs { .. } | goals::State::Paused | goals::State::Running => {
            if counts.total == 0 {
                planning(task)
            } else {
                Fallback::Tasks(counts)
            }
        }
    }
}
fn first_line(words: &[u8]) -> &[u8] {
    match bytes::find(words, b"\n") {
        Some(end) => words.get(..end).expect("newline lies within the result"),
        None => words,
    }
}
fn planning(task: &c::Task) -> Fallback {
    let (draft, _) = draft_of(task.spec.specification.words.bytes.bytes());
    Fallback::Planning {
        draft: match draft {
            Some(path) => Some(bytes::copy_of(file_of(&path))),
            None => None,
        },
    }
}
fn said(task: &c::Task) -> Option<&[u8]> {
    match delegation::facts(task).said {
        Some(delegation::Said::Words(words) | delegation::Said::Merge(words)) => Some(words),
        None => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    Landed(Option<Wall>),
    Stopped,
    Failed,
    Whole { total: u32 },
    Each,
}
#[must_use]
pub fn landing(state: goals::State, task: &c::Task, ended_at: Option<Wall>, counts: Counts) -> Landing {
    match state {
        goals::State::Landed => Landing::Landed(ended_at),
        goals::State::Stopped => Landing::Stopped,
        goals::State::Failed => Landing::Failed,
        goals::State::Needs { .. } | goals::State::Paused | goals::State::Planning | goals::State::Running => {
            if holds_branch(task) {
                Landing::Whole { total: counts.total }
            } else {
                Landing::Each
            }
        }
    }
}
