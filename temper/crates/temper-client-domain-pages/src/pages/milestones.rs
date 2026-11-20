//! Milestone rows, page reads and words (temper/docs/design/client/pages/milestone.md, 1 and 2).
use super::Request;
use crate::{Limits, forge, owner::Owner};
use alloc::boxed::Box;
use core::cmp::Reverse;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Map, Queue, Wall};
use temper_client_domain_forge::{milestones as facts, parts};
use temper_client_protocol_wire::v1 as w;

mod words;
pub use words::{Count, Headline, Mark, Segment, State, count, headline, state, strip};

pub const DRAWN: u32 = 16;
const PAGES: u32 = 4;
const WEEK: u64 = 604_800_000_000_000;

#[must_use]
pub fn query(me: &c::Me, project: u32) -> Option<c::TaskQuery> {
    Some(c::TaskQuery {
        parts: List::from_boxed(Box::new([parts::asks(temper_client_protocol_wire::MILESTONE_PART)])),
        project: Some(project),
        label: Some(jig_client::shapes::template(me, project, facts::TEMPLATE)?.label),
        tracked: Some(true),
        order: c::QueryOrder::Priority,
        requester: None,
        root: None,
        parent: None,
        phase: None,
        executor: None,
    })
}
fn allocate(next: &mut u64) -> Handle {
    let handle = Handle { asker: Owner::Pages.asker(), number: *next };
    *next = next.checked_add(1).expect("page handles fit");
    handle
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
fn read_task(handle: Handle, task: u64, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Task { task }) }));
}
fn document(page: &Readout, number: u64) -> Option<Box<c::Task>> {
    match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task))) if task.chip.task == number => Some(task.clone()),
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(ended))) if ended.matches(c::ObjectKey::Task(number)) => {
            match &ended.value {
                c::EndedValue::Task { document, .. } => Some(document.clone()),
                c::EndedValue::Proposal(_)
                | c::EndedValue::PersonProposal { .. }
                | c::EndedValue::Escalation { .. }
                | c::EndedValue::Question { .. }
                | c::EndedValue::PersonTask { .. } => None,
            }
        }
        Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => None,
    }
}

/// Reads kept by a milestone's own page and by its inbox filter.
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "held milestone values are public; read bookkeeping is private")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "merged completeness, ended cuts, answered goals and opened watches move independently"
)]
pub struct MilestoneRead {
    pub task: u64,
    pub document: Option<Box<c::Task>>,
    pub facts: Option<facts::Facts>,
    pub goals: List<c::TaskLine>,
    pub more: bool,
    pub totals: Option<w::MilestoneTotals>,
    project: u32,
    task_read: Handle,
    live: Handle,
    ended: Handle,
    totals_read: Handle,
    key: Option<(Wall, bool)>,
    document_at: Option<(Wall, bool)>,
    reading_document: Option<(Wall, bool)>,
    totals_at: Option<(Wall, bool)>,
    reading_totals: Option<(Wall, bool)>,
    ended_rows: Map<u64, c::TaskLine>,
    pages: u32,
    ended_read: super::ended::EndedRead,
    ended_mark: Option<u64>,
    ended_more: bool,
    ended_answered: bool,
    watching_goals: bool,
}
impl MilestoneRead {
    #[must_use]
    pub fn new(project: u32, task: u64, next: &mut u64) -> Self {
        Self {
            project,
            task,
            task_read: allocate(next),
            live: allocate(next),
            ended: allocate(next),
            totals_read: allocate(next),
            document: None,
            facts: None,
            goals: List::uncapped(),
            more: false,
            totals: None,
            key: None,
            document_at: None,
            reading_document: None,
            totals_at: None,
            reading_totals: None,
            ended_rows: Map::uncapped(),
            pages: 0,
            ended_read: super::ended::EndedRead::new(),
            ended_mark: None,
            ended_more: false,
            ended_answered: false,
            watching_goals: false,
        }
    }
    fn query(&self) -> c::TaskQuery {
        c::TaskQuery {
            parts: List::from_boxed(Box::new([parts::asks(temper_client_protocol_wire::GOAL_PART)])),
            project: Some(self.project),
            parent: Some(self.task),
            order: c::QueryOrder::Number,
            requester: None,
            root: None,
            label: None,
            phase: None,
            executor: None,
            tracked: None,
        }
    }
    /// A task page already watches the same changed/ended facts in its document.
    pub(super) fn configure_at(
        &mut self,
        changed: Wall,
        ended: bool,
        client: &Client,
        limits: &Limits,
        out: &mut Queue<Request>,
    ) {
        let key = (changed, ended);
        let opening = self.key.is_none();
        self.key = Some(key);
        if opening {
            self.configure_goals(client, limits, out);
        }
        if self.document_at != Some(key) && self.reading_document.is_none() {
            self.reading_document = Some(key);
            read_task(self.task_read, self.task, out);
        }
        if self.totals_at != Some(key) && self.reading_totals.is_none() {
            self.reading_totals = Some(key);
            if let Some(read) = forge::milestones::read(self.project, self.task, &limits.forge) {
                out.push(Request::Client(jig_client::Event::Read { handle: self.totals_read, read }));
            } else {
                self.totals_at = self.reading_totals.take();
            }
        }
        self.merge(client);
    }
    /// A milestone filter needs its merged goals, not its totals or another document read.
    pub fn configure_goals(&mut self, client: &Client, limits: &Limits, out: &mut Queue<Request>) {
        if !self.watching_goals {
            self.watching_goals = true;
            out.push(Request::Client(jig_client::Event::Watch {
                handle: self.live,
                subject: Watchable::Jig(c::Subject::Tasks { query: self.query(), window: limits.client.domain.rows }),
            }));
            self.refresh(out);
        }
        self.merge(client);
    }
    fn refresh(&mut self, out: &mut Queue<Request>) {
        self.pages = 0;
        self.read_ended(None, out);
    }
    fn read_ended(&mut self, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        self.ended_read.start();
        out.push(Request::Client(jig_client::Event::Read {
            handle: self.ended,
            read: Readable::Jig(c::Read::Tasks {
                query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), ..self.query() },
                live: None,
                ended,
            }),
        }));
    }
    pub fn told(&mut self, told: &Told, client: &Client, limits: &Limits, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if *handle == self.live => {
                if self.ended_read.changed(client, self.live) {
                    self.refresh(out);
                }
                true
            }
            Told::Page { handle, page } if *handle == self.task_read => {
                self.document_at = self.reading_document.take();
                if let Some(task) = document(page, self.task) {
                    self.facts = Some(facts::facts(&task));
                    self.document = Some(task);
                }
                true
            }
            Told::Page { handle, page } if *handle == self.totals_read => {
                self.totals_at = self.reading_totals.take();
                match forge::milestones::answer(page, &limits.forge) {
                    Some(w::MilestoneResult::Totals(totals)) if totals.task == self.task => {
                        self.totals = Some(totals);
                    }
                    Some(w::MilestoneResult::NotMilestone) => self.totals = None,
                    Some(w::MilestoneResult::Totals(_)) | None => {}
                }
                true
            }
            Told::Page { handle, page } if *handle == self.ended => {
                let mut following = None;
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                    if self.pages == 0 {
                        self.ended_rows = Map::uncapped();
                        self.ended_more = false;
                        self.ended_mark = None;
                    }
                    self.pages = self.pages.checked_add(1).expect("bounded ended pages");
                    self.ended_answered = true;
                    if self.pages == 1 {
                        self.ended_mark = Some(page.ended_mark);
                    }
                    if let c::EndedGroup::Rows { rows, after, cut } = &page.ended {
                        for row in rows {
                            self.ended_rows.insert(row.chip.task, row.clone()).expect("bounded delivered rows");
                        }
                        self.ended_more |= *cut;
                        if let Some(after) = after {
                            if self.pages < PAGES {
                                following = Some(after.clone());
                            } else {
                                self.ended_more = true;
                            }
                        }
                    }
                }
                if self.ended_read.answered(self.ended_mark) {
                    self.ended_more |= following.is_some();
                    self.refresh(out);
                } else if let Some(after) = following {
                    self.read_ended(Some(after), out);
                }
                true
            }
            Told::Ended { handle, .. }
                if [self.live, self.task_read, self.ended, self.totals_read].contains(handle) =>
            {
                true
            }
            Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Page { .. }
            | Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. } => false,
        }
    }
    fn merge(&mut self, client: &Client) {
        let mut rows = Map::uncapped();
        for (task, row) in &self.ended_rows {
            rows.insert(*task, row.clone()).expect("held rows fit");
        }
        self.more = self.ended_more;
        if let Some(doc) = client.document(self.live)
            && let Some(Latest::Jig(c::Delivery::Tasks(tasks))) = doc.latest
        {
            self.more |= tasks.more;
            for row in &tasks.rows {
                rows.insert(row.chip.task, row.clone()).expect("bounded delivered rows");
            }
        }
        let previous = core::mem::replace(&mut self.goals, List::uncapped());
        for (_, row) in &rows {
            let mut row = row.clone();
            if parts::goal(&row).is_none() {
                for prior in &previous {
                    if prior.chip.task == row.chip.task {
                        row.parts.clone_from(&prior.parts);
                        break;
                    }
                }
            }
            self.goals.push(row).expect("bounded delivered rows");
        }
    }
    /// Both groups of goals and the totals have answered, including a known empty list.
    #[must_use]
    pub fn known(&self, client: &Client) -> bool {
        self.ended_answered && self.totals_at.is_some() && matches_live(client, self.live)
    }
    pub fn leave(&self, out: &mut Queue<Request>) {
        for handle in [self.task_read, self.live, self.ended, self.totals_read] {
            unwatch(handle, out);
        }
    }
}

/// A drawn milestone has no reads of its own (client/forge/parts.md, 5).
#[derive(Debug)]
pub struct Milestone {
    pub task: u64,
    pub facts: Option<RowFacts>,
    pub goals: List<c::TaskLine>,
    pub headlines: Map<u64, Box<[u8]>>,
    pub more: u32,
    pub totals: Option<w::MilestoneTotals>,
}

/// Only the facts a card gets from its line and part; document-only facts stay on its page.
#[derive(Debug)]
pub struct RowFacts {
    pub name: Box<[u8]>,
    pub maker: c::Requester,
    pub whole: Option<Wall>,
}
impl Milestone {
    #[must_use]
    pub fn from_line(line: &c::TaskLine, limits: &jig_client_protocol_link::Limits) -> Self {
        let mut drawn = Self {
            task: line.chip.task,
            facts: None,
            goals: List::uncapped(),
            headlines: Map::uncapped(),
            more: 0,
            totals: None,
        };
        if let Some(part) = parts::milestone(line, limits) {
            drawn.facts = Some(RowFacts {
                name: skein_lib::bytes::copy_of(line.chip.title.bytes()),
                maker: line.requester,
                whole: part.whole,
            });
            drawn.totals = Some(part.totals);
            drawn.more = part.more;
            let mut ordered = Map::uncapped();
            for (goal, headline) in part.goals.into_boxed() {
                let task = goal.chip.task;
                ordered.insert(task, goal).expect("part goals fit");
                if let Some(headline) = headline {
                    drawn.headlines.insert(task, headline).expect("part goals fit");
                }
            }
            for (_, goal) in &ordered {
                drawn.goals.push(goal.clone()).expect("part goals fit");
            }
        }
        drawn
    }
    #[must_use]
    pub fn headline(&self, task: u64) -> Option<&[u8]> {
        match self.headlines.get(&task) {
            Some(headline) => Some(headline),
            None => None,
        }
    }
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "list handles are public; retained reads belong to the list")]
pub struct Milestones {
    pub live: Handle,
    pub ended: Handle,
    query: Option<c::TaskQuery>,
    answered: bool,
    ended_rows: List<c::TaskLine>,
    ended_read: super::ended::EndedRead,
    lines: List<c::TaskLine>,
    drawn: List<Milestone>,
    // Together with drawn, one kept projection per listed task, including old ended rows.
    undrawn: Map<u64, Milestone>,
}
impl Milestones {
    #[must_use]
    pub fn new(next: &mut u64) -> Self {
        Self {
            live: allocate(next),
            ended: allocate(next),
            query: None,
            answered: false,
            ended_rows: List::uncapped(),
            ended_read: super::ended::EndedRead::new(),
            lines: List::uncapped(),
            drawn: List::with_capacity(DRAWN),
            undrawn: Map::uncapped(),
        }
    }
    pub fn configure(
        &mut self,
        me: Option<&c::Me>,
        project: u32,
        client: &Client,
        limits: &Limits,
        wall: Wall,
        out: &mut Queue<Request>,
    ) {
        let query = match me {
            Some(me) => query(me, project),
            None => None,
        };
        if self.query != query {
            self.leave(out);
            self.query.clone_from(&query);
            self.answered = false;
            self.lines.clear();
            self.ended_rows.clear();
            self.ended_read = super::ended::EndedRead::new();
            self.drawn.clear();
            self.undrawn = Map::uncapped();
            if let Some(query) = query {
                out.push(Request::Client(jig_client::Event::Watch {
                    handle: self.live,
                    subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
                }));
                self.read_ended(out);
            }
        }
        if self.query.is_none() {
            return;
        }
        self.lines.clear();
        let mut live = Map::uncapped();
        if let Some(doc) = client.document(self.live)
            && let Some(Latest::Jig(c::Delivery::Tasks(tasks))) = doc.latest
        {
            for line in &tasks.rows {
                self.lines.push(line.clone()).expect("delivered milestones fit");
                live.insert((Reverse(line.last_activity), line.chip.task), line).expect("delivered milestones fit");
            }
        }
        let mut ended = Map::uncapped();
        for line in &self.ended_rows {
            let mut duplicate = false;
            for listed in &self.lines {
                duplicate |= listed.chip.task == line.chip.task;
            }
            if duplicate {
                continue;
            }
            self.lines.push(line.clone()).expect("delivered milestones fit");
            if let Some(at) = line.ended_at
                && at <= wall
                && wall.as_nanos().saturating_sub(at.as_nanos()) <= WEEK
            {
                ended.insert((Reverse(at), line.chip.task), line).expect("delivered milestones fit");
            }
        }
        let mut drawn = List::with_capacity(DRAWN);
        let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges)
            .expect("validated client limits");
        let previous = core::mem::replace(&mut self.drawn, List::with_capacity(DRAWN));
        for milestone in previous.into_boxed() {
            self.undrawn.insert(milestone.task, milestone).expect("held milestones fit");
        }
        let mut held = Map::uncapped();
        for line in &self.lines {
            let fresh = Milestone::from_line(line, &link);
            let milestone =
                if fresh.totals.is_some() { fresh } else { self.undrawn.remove(&line.chip.task).unwrap_or(fresh) };
            held.insert(line.chip.task, milestone).expect("one part per listed milestone");
        }
        for ordered in [&live, &ended] {
            for (_, line) in ordered {
                if drawn.room() == 0 {
                    break;
                }
                if let Some(milestone) = held.remove(&line.chip.task) {
                    drawn.push(milestone).expect("drawn bound checked");
                }
            }
        }
        self.undrawn = held;
        self.drawn = drawn;
    }
    fn read_ended(&mut self, out: &mut Queue<Request>) {
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
                    ended: None,
                }),
            }));
        }
    }
    pub fn told(&mut self, told: &Told, client: &Client, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if *handle == self.live => {
                if self.ended_read.changed(client, self.live) {
                    self.read_ended(out);
                }
                return true;
            }
            Told::Page { handle, page } if *handle == self.ended => {
                self.answered = true;
                let mut mark = None;
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                    mark = Some(page.ended_mark);
                    self.ended_rows = match &page.ended {
                        c::EndedGroup::Rows { rows, .. } => rows.clone(),
                        c::EndedGroup::None => List::uncapped(),
                    };
                }
                if self.ended_read.answered(mark) {
                    self.read_ended(out);
                }
                return true;
            }
            Told::Ended { handle, .. } if *handle == self.ended => {
                self.answered = true;
                self.ended_read.answered(None);
                return true;
            }
            Told::Ended { handle, .. } if *handle == self.live => return true,
            Told::Page { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. } => {}
        }
        false
    }
    pub fn leave(&self, out: &mut Queue<Request>) {
        if self.query.is_some() {
            unwatch(self.live, out);
            unwatch(self.ended, out);
        }
    }
    pub(super) fn has_rows(&self, client: &Client) -> Option<bool> {
        if !self.ended_rows.is_empty() {
            return Some(true);
        }
        if let Some(document) = client.document(self.live)
            && let Some(Latest::Jig(c::Delivery::Tasks(tasks))) = document.latest
        {
            return Some(!tasks.rows.is_empty());
        }
        if self.answered { Some(false) } else { None }
    }
    /// The project's milestone list has answered; a project without its template has none.
    #[must_use]
    pub fn known(&self, client: &Client) -> bool {
        if self.query.is_none() {
            return true;
        }
        if !self.answered {
            return false;
        }
        super::home::delivered(client, self.live)
    }
    #[must_use]
    pub fn lines(&self) -> &[c::TaskLine] {
        self.lines.as_slice()
    }
    #[must_use]
    pub fn drawn(&self) -> &[Milestone] {
        self.drawn.as_slice()
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code uses no closures")]
    pub fn milestone(&self, task: u64) -> Option<&Milestone> {
        for read in &self.drawn {
            if read.task == task {
                return Some(read);
            }
        }
        None
    }
    #[must_use]
    pub fn of(&self, line: &c::TaskLine, goals: &super::goals::Goals) -> Option<u64> {
        if let Some(milestone) = goals.membership(line.chip.task) {
            return milestone;
        }
        if let Some(part) = parts::goal(line) {
            return part.milestone;
        }
        for milestone in &self.drawn {
            for goal in &milestone.goals {
                if goal.chip.task == line.chip.task {
                    return Some(milestone.task);
                }
            }
        }
        None
    }
}

fn matches_live(client: &Client, handle: Handle) -> bool {
    if let Some(document) = client.document(handle)
        && let Some(Latest::Jig(c::Delivery::Tasks(_))) = document.latest
    {
        return true;
    }
    false
}
