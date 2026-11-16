//! Shared milestone reads and words (temper/docs/design/client/pages/milestone.md, 1 and 2).
use super::{Request, goals::Goals};
use crate::{Limits, forge, owner::Owner};
use alloc::boxed::Box;
use core::cmp::Reverse;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Map, Queue, Wall};
use temper_client_domain_forge::milestones as facts;
use temper_client_protocol_wire::v1 as w;

mod words;
pub use words::{Count, Headline, Mark, Segment, State, count, headline, state, strip};

pub const DRAWN: u32 = 16;
const HELD: u32 = 16;
const PAGES: u32 = 4;
const WEEK: u64 = 604_800_000_000_000;

#[must_use]
pub fn query(me: &c::Me, project: u32) -> Option<c::TaskQuery> {
    Some(c::TaskQuery {
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

/// Usable by a milestone's own page as well as by the shared list.
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "held milestone values are public; read bookkeeping is private")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "merged completeness, paging in flight, queued refresh and ended completeness move independently"
)]
pub struct Milestone {
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
    reading_ended: bool,
    refresh_ended: bool,
    ended_more: bool,
    watching_goals: bool,
}
impl Milestone {
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
            reading_ended: false,
            refresh_ended: false,
            ended_more: false,
            watching_goals: false,
        }
    }
    fn query(&self) -> c::TaskQuery {
        c::TaskQuery {
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
    pub fn configure(&mut self, line: &c::TaskLine, client: &Client, limits: &Limits, out: &mut Queue<Request>) {
        let key = (line.changed, line.chip.phase == c::Phase::Ended);
        let ending = key.1
            && match self.key {
                Some(old) => !old.1,
                None => false,
            };
        let opening = self.key.is_none();
        self.key = Some(key);
        if opening {
            self.configure_goals(client, limits, out);
        }
        if ending {
            self.refresh_ended = true;
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
                self.totals = None;
            }
        }
        self.refresh(out);
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
            self.refresh_ended = true;
        }
        self.refresh(out);
        self.merge(client);
    }
    fn refresh(&mut self, out: &mut Queue<Request>) {
        if self.refresh_ended && !self.reading_ended {
            self.refresh_ended = false;
            self.pages = 0;
            self.read_ended(None, out);
        }
    }
    fn read_ended(&mut self, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        self.reading_ended = true;
        out.push(Request::Client(jig_client::Event::Read {
            handle: self.ended,
            read: Readable::Jig(c::Read::Tasks {
                query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), ..self.query() },
                live: None,
                ended,
            }),
        }));
    }
    pub fn told(&mut self, told: &Told, limits: &Limits, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if *handle == self.live => {
                self.refresh_ended = true;
                self.refresh(out);
                true
            }
            Told::Page { handle, page } if *handle == self.task_read => {
                self.document_at = self.reading_document.take();
                self.document = document(page, self.task);
                self.facts = match &self.document {
                    Some(task) => Some(facts::facts(task)),
                    None => None,
                };
                true
            }
            Told::Page { handle, page } if *handle == self.totals_read => {
                self.totals_at = self.reading_totals.take();
                self.totals = match forge::milestones::answer(page, &limits.forge) {
                    Some(w::MilestoneResult::Totals(totals)) if totals.task() == self.task => Some(totals),
                    Some(w::MilestoneResult::Totals(_) | w::MilestoneResult::NotMilestone) | None => None,
                };
                true
            }
            Told::Page { handle, page } if *handle == self.ended => {
                self.reading_ended = false;
                if self.pages == 0 {
                    self.ended_rows = Map::uncapped();
                    self.ended_more = false;
                }
                self.pages = self.pages.checked_add(1).expect("bounded ended pages");
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page
                    && let c::EndedGroup::Rows { rows, after, cut } = &page.ended
                {
                    for row in rows {
                        self.ended_rows.insert(row.chip.task, row.clone()).expect("bounded delivered rows");
                    }
                    self.ended_more |= *cut;
                    if let Some(after) = after {
                        if self.pages < PAGES && !self.refresh_ended {
                            self.read_ended(Some(after.clone()), out);
                        } else {
                            self.ended_more = true;
                        }
                    }
                }
                self.refresh(out);
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
        self.goals.clear();
        for (_, row) in &rows {
            self.goals.push(row.clone()).expect("bounded delivered rows");
        }
    }
    pub fn leave(&self, out: &mut Queue<Request>) {
        for handle in [self.task_read, self.live, self.ended, self.totals_read] {
            unwatch(handle, out);
        }
    }
}

#[derive(Debug)]
struct Requester {
    task: u64,
    handle: Handle,
    document: Option<Box<c::Task>>,
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "list handles are public; retained reads belong to the list")]
pub struct Milestones {
    pub live: Handle,
    pub ended: Handle,
    query: Option<c::TaskQuery>,
    answered: bool,
    ended_rows: List<c::TaskLine>,
    lines: List<c::TaskLine>,
    drawn: List<Milestone>,
    requesters: List<Requester>,
    sources: Map<u64, c::TaskLine>,
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
            lines: List::uncapped(),
            drawn: List::with_capacity(DRAWN),
            requesters: List::with_capacity(HELD),
            sources: Map::uncapped(),
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the shared read takes the client's values, clock and allocator explicitly"
    )]
    pub fn configure(
        &mut self,
        me: Option<&c::Me>,
        project: u32,
        client: &Client,
        limits: &Limits,
        wall: Wall,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let query = match me {
            Some(me) => query(me, project),
            None => None,
        };
        if self.query != query {
            self.leave(out);
            self.query = query;
            self.answered = false;
            self.lines.clear();
            self.ended_rows.clear();
            self.drawn.clear();
            self.requesters.clear();
            self.sources = Map::uncapped();
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
        let mut old = core::mem::replace(&mut self.drawn, List::with_capacity(0));
        for ordered in [&live, &ended] {
            for (_, line) in ordered {
                if drawn.room() == 0 {
                    break;
                }
                let mut found = None;
                for index in 0..old.len() {
                    if old.get(index).expect("drawn index").task == line.chip.task {
                        found = old.swap_remove(index);
                        break;
                    }
                }
                let mut read = match found {
                    Some(read) => read,
                    None => Milestone::new(project, line.chip.task, next),
                };
                read.configure(line, client, limits, out);
                drawn.push(read).expect("drawn bound checked");
            }
        }
        for read in &old {
            read.leave(out);
        }
        self.drawn = drawn;
    }
    fn read_ended(&self, out: &mut Queue<Request>) {
        if let Some(query) = self.query {
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.ended,
                read: Readable::Jig(c::Read::Tasks {
                    query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                    live: None,
                    ended: None,
                }),
            }));
        }
    }
    /// Called after each page step, including deliveries of the shared goals and drawn goals.
    pub fn requesters(&mut self, goals: Option<&Goals>, client: &Client, next: &mut u64, out: &mut Queue<Request>) {
        if self.query.is_none() {
            return;
        }
        let mut sources = Map::uncapped();
        if let Some(goals) = goals {
            if let Some(live) = goals.live(client) {
                for line in &live.rows {
                    sources.insert(line.chip.task, line.clone()).expect("held goals fit");
                }
            }
            for line in goals.ended() {
                if sources.get(&line.chip.task).is_none() {
                    sources.insert(line.chip.task, line.clone()).expect("held goals fit");
                }
            }
        }
        for milestone in &self.drawn {
            for line in &milestone.goals {
                sources.insert(line.chip.task, line.clone()).expect("held goals fit");
            }
        }
        for (task, line) in &sources {
            if self.sources.get(task) == Some(line) {
                continue;
            }
            let c::Requester::Task(requester) = line.requester else { continue };
            let mut known = false;
            for listed in &self.lines {
                known |= listed.chip.task == requester;
            }
            for held in &self.requesters {
                known |= held.task == requester;
            }
            if known {
                continue;
            }
            if self.requesters.room() == 0 {
                // Retain insertion order: the first held requester is the oldest.
                let mut kept = List::with_capacity(HELD);
                for (index, held) in core::mem::replace(&mut self.requesters, List::with_capacity(0))
                    .into_boxed()
                    .into_iter()
                    .enumerate()
                {
                    if index == 0 {
                        unwatch(held.handle, out);
                    } else {
                        kept.push(held).expect("held bound");
                    }
                }
                self.requesters = kept;
            }
            let handle = allocate(next);
            read_task(handle, requester, out);
            self.requesters.push(Requester { task: requester, handle, document: None }).expect("held bound checked");
        }
        self.sources = sources;
    }
    pub fn told(&mut self, told: &Told, limits: &Limits, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if *handle == self.live => {
                self.read_ended(out);
                return true;
            }
            Told::Page { handle, page } if *handle == self.ended => {
                self.answered = true;
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                    self.ended_rows = match &page.ended {
                        c::EndedGroup::Rows { rows, .. } => rows.clone(),
                        c::EndedGroup::None => List::uncapped(),
                    };
                }
                return true;
            }
            Told::Page { handle, page } => {
                for held in self.requesters.iter_mut() {
                    if held.handle == *handle {
                        held.document = document(page, held.task);
                        return true;
                    }
                }
            }
            Told::Ended { handle, .. } if *handle == self.live => return true,
            Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. } => {}
        }
        for read in self.drawn.iter_mut() {
            if read.told(told, limits, out) {
                return true;
            }
        }
        false
    }
    pub fn leave(&self, out: &mut Queue<Request>) {
        if self.query.is_some() {
            unwatch(self.live, out);
            unwatch(self.ended, out);
        }
        for read in &self.drawn {
            read.leave(out);
        }
        for held in &self.requesters {
            unwatch(held.handle, out);
        }
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
        if let Some(doc) = client.document(self.live)
            && let Some(Latest::Jig(c::Delivery::Tasks(_))) = doc.latest
        {
            return true;
        }
        false
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
    pub fn of(&self, line: &c::TaskLine) -> Option<u64> {
        let c::Requester::Task(task) = line.requester else { return None };
        for line in &self.lines {
            if line.chip.task == task {
                return Some(task);
            }
        }
        for held in &self.requesters {
            if held.task == task
                && let Some(document) = &held.document
                && facts::is(&document.chip)
            {
                return Some(task);
            }
        }
        None
    }
}
