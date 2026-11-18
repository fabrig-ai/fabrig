//! Goals and their reports (temper/docs/design/client/pages/README.md, Goals).
mod rows;
pub use rows::{GroupRow, goal, groups, home_rows};

use super::{Request, ended::EndedRead, report};
use crate::{GoalsFilter, Limits, owner::Owner};
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Wall};
use temper_client_domain_forge::parts;

/// The open project's goal fact, taking held lists (temper/docs/design/client/pages/milestone.md, 1).
#[derive(Debug)]
pub(super) struct HasGoals {
    goals: Fact,
    milestones: Option<Fact>,
}
impl HasGoals {
    pub(super) fn new() -> Self {
        Self { goals: Fact::new(), milestones: None }
    }
    #[expect(clippy::too_many_arguments, reason = "the fact takes both held lists and the client's delivered state")]
    pub(super) fn configure(
        &mut self,
        me: Option<&c::Me>,
        project: Option<u32>,
        goals: Option<&Goals>,
        milestones: Option<&super::milestones::Milestones>,
        client: &Client,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let query = match (me, project) {
            (Some(me), Some(project)) => query(me, project),
            (Some(_), None) | (None, Some(_) | None) => None,
        };
        let list = match goals {
            Some(goals) => Some(goals.has_rows(client).unwrap_or(self.goals.has(client, project.unwrap_or(0)))),
            None => None,
        };
        self.goals.configure(query, list, client, out);
        let offered = match (me, project) {
            (Some(me), Some(project)) => super::milestones::query(me, project).is_some(),
            (Some(_), None) | (None, Some(_) | None) => false,
        };
        if offered && self.milestones.is_none() {
            let live = handle(*next);
            *next = next.checked_add(1).expect("page handles fit");
            let read = handle(*next);
            *next = next.checked_add(1).expect("page handles fit");
            self.milestones = Some(Fact {
                live,
                read,
                milestone: true,
                query: None,
                ended: false,
                borrowed: false,
                until_live: None,
                ended_read: EndedRead::new(),
            });
        }
        if let Some(fact) = &mut self.milestones {
            let query = match (me, project) {
                (Some(me), Some(project)) => super::milestones::query(me, project),
                (Some(_), None) | (None, Some(_) | None) => None,
            };
            let list = match milestones {
                Some(list) => Some(list.has_rows(client).unwrap_or(fact.has(client, project.unwrap_or(0)))),
                None => None,
            };
            fact.configure(query, list, client, out);
        }
    }
    pub(super) fn told(&mut self, told: &Told, client: &Client, out: &mut Queue<Request>) -> bool {
        if self.goals.told(told, client, out) {
            return true;
        }
        match &mut self.milestones {
            Some(fact) => fact.told(told, client, out),
            None => false,
        }
    }
    pub(super) fn has(&self, client: &Client, project: u32) -> bool {
        self.goals.has(client, project)
            || match &self.milestones {
                Some(fact) => fact.has(client, project),
                None => false,
            }
    }
    pub(super) fn leave(&mut self, out: &mut Queue<Request>) {
        self.goals.leave(out);
        if let Some(fact) = &mut self.milestones {
            fact.leave(out);
        }
    }
}
#[derive(Debug)]
struct Fact {
    query: Option<c::TaskQuery>,
    ended: bool,
    borrowed: bool,
    until_live: Option<bool>,
    live: Handle,
    read: Handle,
    milestone: bool,
    ended_read: EndedRead,
}

impl Fact {
    fn new() -> Self {
        Self {
            query: None,
            ended: false,
            borrowed: false,
            until_live: None,
            milestone: false,
            ended_read: EndedRead::new(),
            live: handle(u64::MAX.checked_sub(1).expect("reserved goal watch handle")),
            read: handle(u64::MAX.checked_sub(2).expect("reserved goal read handle")),
        }
    }
    pub(super) fn leave(&mut self, out: &mut Queue<Request>) {
        if self.query.take().is_some() && !self.borrowed {
            for handle in [self.live, self.read] {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
        }
        self.ended = false;
        self.borrowed = false;
        self.until_live = None;
        self.ended_read = EndedRead::new();
    }
    fn configure(&mut self, next: Option<c::TaskQuery>, list: Option<bool>, client: &Client, out: &mut Queue<Request>) {
        let retained = match &next {
            Some(query) if self.query == next => self.has(client, query.project.unwrap_or(0)),
            Some(_) | None => false,
        };
        if self.query == next && self.borrowed == list.is_some() {
            if let Some(has) = list {
                self.ended = has;
            }
            return;
        }
        self.leave(out);
        self.query.clone_from(&next);
        if let Some(has) = list {
            self.borrowed = true;
            self.ended = has;
        } else if let Some(query) = next {
            self.until_live = Some(retained);
            out.push(Request::Client(jig_client::Event::Watch {
                handle: self.live,
                subject: Watchable::Jig(c::Subject::Tasks { query, window: 1 }),
            }));
            self.read_ended(out);
        }
    }
    fn read_ended(&mut self, out: &mut Queue<Request>) {
        if !self.borrowed
            && let Some(query) = &self.query
        {
            self.ended_read.start();
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.read,
                read: Readable::Jig(c::Read::Tasks {
                    query: c::TaskQuery {
                        phase: Some(c::PhaseClass::Ended),
                        order: c::QueryOrder::Ended,
                        ..query.clone()
                    },
                    live: None,
                    ended: None,
                }),
            }));
        }
    }
    pub(super) fn told(&mut self, told: &Told, client: &Client, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Page { handle, page } if *handle == self.read && !self.borrowed => {
                if self.query.is_some() {
                    let mut mark = None;
                    if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                        mark = Some(page.ended_mark);
                        self.ended = match &page.ended {
                            c::EndedGroup::Rows { rows, .. } => !rows.is_empty(),
                            c::EndedGroup::None => false,
                        };
                    }
                    if self.ended_read.answered(mark) && self.milestone {
                        self.read_ended(out);
                    }
                }
                true
            }
            Told::Changed { handle } if *handle == self.live && !self.borrowed => {
                self.until_live = None;
                if self.milestone && self.ended_read.changed(client, self.live) {
                    self.read_ended(out);
                }
                true
            }
            Told::Ended { handle, .. } => *handle == self.live,
            Told::Changed { .. }
            | Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Page { .. } => false,
        }
    }
    pub(super) fn has(&self, client: &Client, project: u32) -> bool {
        let Some(query) = &self.query else { return false };
        if query.project != Some(project) {
            return false;
        }
        if self.ended {
            return true;
        }
        if self.borrowed {
            return false;
        }
        if let Some(previous) = self.until_live {
            return previous;
        }
        if let Some(document) = client.document(self.live)
            && document.state != jig_client::State::Ended
            && let Some(Latest::Jig(c::Delivery::Tasks(tasks))) = document.latest
        {
            return !tasks.rows.is_empty();
        }
        false
    }
}

#[derive(Debug)]
struct Row {
    task: u64,
    report: Option<report::Report>,
}

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "handles and filter are public; read bookkeeping belongs to the page")]
pub struct Goals {
    pub filter: GoalsFilter,
    pub live: Handle,
    pub ended: Handle,
    pub needs: Handle,
    query: Option<c::TaskQuery>,
    page: Option<c::TaskPage>,
    first: bool,
    ended_read: EndedRead,
    page_answered: bool,
    count: u32,
    needs_total: Option<u32>,
    needs_answered: bool,
    inbox_moves: u64,
    selection: Option<List<c::InboxSelection>>,
    rows: List<Row>,
}

impl Goals {
    #[must_use]
    pub fn new(first: u64) -> Self {
        Self {
            filter: GoalsFilter::All,
            live: handle(first),
            ended: handle(first.checked_add(1).expect("page handles fit")),
            needs: handle(first.checked_add(2).expect("page handles fit")),
            query: None,
            page: None,
            first: true,
            ended_read: EndedRead::new(),
            page_answered: false,
            count: 0,
            needs_total: None,
            needs_answered: false,
            inbox_moves: 0,
            selection: None,
            rows: List::with_capacity(0),
        }
    }
    pub fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [self.live, self.ended, self.needs] {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        self.needs.number.checked_add(1).expect("page handles fit")
    }
    pub fn configure(
        &mut self,
        me: Option<&c::Me>,
        project: u32,
        client: &Client,
        limits: &Limits,
        inbox_moves: u64,
        out: &mut Queue<Request>,
    ) {
        let next = match me {
            Some(me) => query(me, project),
            None => None,
        };
        if self.query != next {
            if self.query.is_some() {
                self.leave(out);
            }
            self.query.clone_from(&next);
            self.page = None;
            self.page_answered = false;
            self.ended_read = EndedRead::new();
            self.first = true;
            self.count = 0;
            self.needs_total = None;
            self.selection = None;
            self.needs_answered = false;
            self.rows = List::with_capacity(0);
            if let Some(query) = next {
                out.push(Request::Client(jig_client::Event::Watch {
                    handle: self.live,
                    subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
                }));
                self.read(None, out);
            }
        }
        if self.query.is_none() {
            return;
        }
        if let Some(me) = me
            && (self.needs_total != Some(me.needs_total) || self.inbox_moves != inbox_moves)
        {
            self.needs_total = Some(me.needs_total);
            self.inbox_moves = inbox_moves;
            self.read_needs(out);
        }
        self.reconcile(client);
    }
    fn read(&mut self, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        if let Some(query) = &self.query {
            self.ended_read.start();
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.ended,
                read: Readable::Jig(c::Read::Tasks {
                    query: c::TaskQuery {
                        phase: Some(c::PhaseClass::Ended),
                        order: c::QueryOrder::Ended,
                        ..query.clone()
                    },
                    live: None,
                    ended,
                }),
            }));
        }
    }
    fn read_needs(&self, out: &mut Queue<Request>) {
        if let Some(query) = &self.query {
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.needs,
                read: Readable::Jig(c::Read::Inbox {
                    group: c::InboxReadGroup::Needs,
                    project: query.project,
                    kind: None,
                    goal: None,
                    after: None,
                }),
            }));
        }
    }
    pub(crate) fn older(&mut self, out: &mut Queue<Request>) {
        if !self.ended_read.reading()
            && let Some(page) = &self.page
            && let c::EndedGroup::Rows { after: Some(after), .. } = &page.ended
        {
            let after = after.clone();
            self.first = false;
            self.read(Some(after), out);
        }
    }
    pub(crate) fn told(&mut self, told: Told, client: &Client, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if handle == self.live => {
                if self.first && self.ended_read.changed(client, self.live) {
                    self.read(None, out);
                }
                true
            }
            Told::Page { handle, page } if handle == self.ended => {
                self.page_answered = true;
                let mut mark = None;
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                    if self.first {
                        mark = Some(page.ended_mark);
                    }
                    self.page = Some(*page);
                    let read = u32::try_from(self.ended().len()).expect("goal page fits");
                    self.count =
                        if self.first { read } else { self.count.checked_add(read).expect("goals count fits") };
                }
                if self.ended_read.answered(mark) {
                    self.read(None, out);
                }
                true
            }
            Told::Page { handle, page } if handle == self.needs => {
                self.needs_answered = true;
                self.selection = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) => page.selection,
                    Readout::Jig(c::ReadResult::Value(_) | c::ReadResult::None(_))
                    | Readout::Family { .. }
                    | Readout::NotServed { .. }
                    | Readout::Limit => None,
                };
                true
            }
            Told::Ended { handle, .. } if handle == self.needs => {
                self.needs_answered = true;
                self.selection = None;
                true
            }
            Told::Ended { handle, .. } if handle == self.ended => {
                self.page_answered = true;
                self.ended_read.answered(None);
                true
            }
            Told::Ended { handle, .. } if handle == self.live => true,
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
    fn reconcile(&mut self, client: &Client) {
        let mut rows = List::uncapped();
        let live = match self.live(client) {
            Some(live) => live.rows.as_slice(),
            None => &[],
        };
        for lines in [live, self.ended()] {
            for line in lines {
                let part = match parts::goal(line) {
                    Some(part) => part.into_parts().report,
                    None => None,
                };
                let report = match part {
                    Some(report) => {
                        let report = report.into_parts();
                        Some(report::Report {
                            headline: report.headline,
                            at: Some(Wall::from_nanos(report.at)),
                            paragraphs: List::with_capacity(0),
                            left: None,
                            risk: None,
                            decided: List::with_capacity(0),
                        })
                    }
                    None => None,
                };
                rows.push(Row { task: line.chip.task, report }).expect("live and ended goal rows fit");
            }
        }
        self.rows = rows;
    }
    /// Both goal lists have answered, including refused or failed answers (client/pages/work.md, 1).
    #[must_use]
    pub fn known(&self, client: &Client) -> bool {
        self.query.is_none() || (self.page_answered && super::home::delivered(client, self.live))
    }
    /// The needs read has answered, including refusal or failure.
    #[must_use]
    pub const fn needs_known(&self) -> bool {
        self.query.is_none() || self.needs_answered
    }
    /// Goals' rows and counts wait for their state and milestone placement facts.
    #[must_use]
    pub fn shown(&self, client: &Client, milestones: &super::milestones::Milestones) -> bool {
        self.known(client) && self.needs_known() && milestones.known(client)
    }
    pub(super) fn has_rows(&self, client: &Client) -> Option<bool> {
        if self.count(client) > 0 {
            Some(true)
        } else if self.page_answered || self.live(client).is_some() {
            Some(false)
        } else {
            None
        }
    }
    #[must_use]
    pub fn live<'a>(&self, client: &'a Client) -> Option<&'a c::Tasks> {
        if let Some(Latest::Jig(c::Delivery::Tasks(tasks))) = client.document(self.live)?.latest {
            Some(tasks)
        } else {
            None
        }
    }
    #[must_use]
    pub fn ended(&self) -> &[c::TaskLine] {
        match &self.page {
            Some(page) => match &page.ended {
                c::EndedGroup::Rows { rows, .. } => rows.as_slice(),
                c::EndedGroup::None => &[],
            },
            None => &[],
        }
    }
    #[must_use]
    pub fn more(&self) -> bool {
        match &self.page {
            Some(page) => match &page.ended {
                c::EndedGroup::Rows { after, .. } => after.is_some(),
                c::EndedGroup::None => false,
            },
            None => false,
        }
    }
    #[must_use]
    pub fn count(&self, client: &Client) -> u32 {
        let live = match self.live(client) {
            Some(live) => live.rows.len(),
            None => 0,
        };
        live.checked_add(self.count).expect("goals count fits")
    }
    #[must_use]
    pub fn unread(&self, client: &Client) -> bool {
        let live = match self.live(client) {
            Some(live) => live.more,
            None => false,
        };
        live || self.more()
    }
    #[must_use]
    pub fn report(&self, task: u64) -> Option<&report::Report> {
        for row in &self.rows {
            if row.task == task {
                return row.report.as_ref();
            }
        }
        None
    }
    /// The first selection goal is the need's own task; the rest are its ancestors (client/pages/work.md, 1).
    #[must_use]
    pub fn needs_own(&self, task: u64) -> bool {
        if let Some(selection) = &self.selection {
            for row in selection {
                if row.goals.as_slice().first() == Some(&task) {
                    return true;
                }
            }
        }
        false
    }
    #[must_use]
    pub fn needs_person(&self, task: u64) -> bool {
        if let Some(selection) = &self.selection {
            for row in selection {
                if row.goals.as_slice().contains(&task) {
                    return true;
                }
            }
        }
        false
    }
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}

/// The project's goal template supplies the label (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn query(me: &c::Me, project: u32) -> Option<c::TaskQuery> {
    let mut parts = List::with_capacity(1);
    parts.push(parts::asks(temper_client_protocol_wire::GOAL_PART)).expect("one goal part");
    Some(c::TaskQuery {
        parts,
        project: Some(project),
        label: Some(jig_client::shapes::template(me, project, jig_client::shapes::GOAL)?.label),
        tracked: Some(true),
        order: c::QueryOrder::Priority,
        requester: None,
        root: None,
        parent: None,
        phase: None,
        executor: None,
    })
}
/// A chip's listed wait (temper/docs/design/client/pages/work.md, 1, Goals).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wait {
    pub first: u64,
    pub second: Option<u64>,
    pub count: u32,
    pub more: bool,
}

/// The leading goal id (temper/docs/design/client/pages/work.md, 1, A goal's id).
#[must_use]
pub fn id(title: &[u8]) -> Option<&[u8]> {
    let mut letters = 0_usize;
    for byte in title {
        if !byte.is_ascii_uppercase() {
            break;
        }
        letters = letters.checked_add(1).expect("title fits");
    }
    if !(1..=3).contains(&letters) {
        return None;
    }
    let rest = title.get(letters..)?;
    let mut digits = 0_usize;
    for byte in rest {
        if !byte.is_ascii_digit() {
            break;
        }
        digits = digits.checked_add(1).expect("title fits");
    }
    if !(1..=3).contains(&digits) {
        return None;
    }
    let end = letters.checked_add(digits).expect("goal id fits");
    if let Some(byte) = title.get(end)
        && byte.is_ascii_alphanumeric()
    {
        return None;
    }
    title.get(..end)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Needs { funds: bool },
    Paused,
    Waits(Wait),
    WaitsForRoom,
    Planning,
    Running,
    Landed,
    Closed,
    Failed,
}
/// First match wins (temper/docs/design/client/pages/README.md, Goals).
#[must_use]
pub fn state(chip: &c::Chip, needs: bool, report: bool, delegates_total: u32) -> State {
    if needs {
        let funds = match chip.details.phase {
            c::ChipPhase::Held { reason: c::HoldCategory::Exceeded(_), .. } => true,
            c::ChipPhase::Waiting { .. }
            | c::ChipPhase::Active(_)
            | c::ChipPhase::Person { .. }
            | c::ChipPhase::Closing(_)
            | c::ChipPhase::Held { .. }
            | c::ChipPhase::Ended => false,
        };
        return State::Needs { funds };
    }
    match &chip.details.phase {
        c::ChipPhase::Ended => {
            return match &chip.details.details.ending {
                Some(c::ChipEnding::Done(_)) => State::Landed,
                Some(c::ChipEnding::Cancelled { .. }) => State::Closed,
                Some(c::ChipEnding::Failed(_)) | None => State::Failed,
            };
        }
        c::ChipPhase::Held { .. } => {
            if let Some(hold) = &chip.details.details.hold
                && let Some(placed) = &hold.placed
                && placed.mode == c::HoldMode::Stop
                && placed.scope == (c::HoldScope::Subtree { task: chip.task })
                && !placed.lifted
            {
                return State::Paused;
            }
        }
        c::ChipPhase::Waiting { dependencies, more, holds } => {
            if !dependencies.is_empty() || *more {
                return State::Waits(Wait {
                    first: dependencies.as_slice().first().copied().unwrap_or_default(),
                    second: dependencies.get(1).copied(),
                    count: dependencies.len(),
                    more: *more,
                });
            }
            if *holds {
                return State::WaitsForRoom;
            }
        }
        c::ChipPhase::Active(_) | c::ChipPhase::Person { .. } | c::ChipPhase::Closing(_) => {}
    }
    if !report && delegates_total == 0 { State::Planning } else { State::Running }
}
/// The address's filter selects a goal's state (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn shown(filter: GoalsFilter, state: State) -> bool {
    match filter {
        GoalsFilter::All => true,
        GoalsFilter::Needs => match state {
            State::Needs { .. } => true,
            State::Paused
            | State::Waits(_)
            | State::WaitsForRoom
            | State::Planning
            | State::Running
            | State::Landed
            | State::Closed
            | State::Failed => false,
        },
        GoalsFilter::Live => match state {
            State::Paused | State::Waits(_) | State::WaitsForRoom | State::Planning | State::Running => true,
            State::Needs { .. } | State::Landed | State::Closed | State::Failed => false,
        },
        GoalsFilter::Ended => match state {
            State::Landed | State::Closed | State::Failed => true,
            State::Needs { .. }
            | State::Paused
            | State::Waits(_)
            | State::WaitsForRoom
            | State::Planning
            | State::Running => false,
        },
    }
}
