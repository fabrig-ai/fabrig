//! Goals and their reports (temper/docs/design/client/pages/README.md, Goals).
use super::{Request, report};
use crate::{GoalsFilter, Limits, owner::Owner};
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Wall};

/// The open project's goal fact across pages (temper/docs/design/client/view.md, 3).
#[derive(Debug)]
pub(super) struct HasGoals {
    query: Option<c::TaskQuery>,
    ended: bool,
}

impl HasGoals {
    pub(super) fn new() -> Self {
        Self { query: None, ended: false }
    }
    fn live() -> Handle {
        handle(u64::MAX.checked_sub(1).expect("reserved goal watch handle"))
    }
    fn read_handle() -> Handle {
        handle(u64::MAX.checked_sub(2).expect("reserved goal read handle"))
    }
    pub(super) fn leave(&mut self, out: &mut Queue<Request>) {
        if self.query.take().is_some() {
            for handle in [Self::live(), Self::read_handle()] {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
        }
        self.ended = false;
    }
    pub(super) fn configure(&mut self, me: Option<&c::Me>, project: Option<u32>, out: &mut Queue<Request>) {
        let next = if let (Some(me), Some(project)) = (me, project) { query(me, project) } else { None };
        if self.query == next {
            return;
        }
        self.leave(out);
        self.query = next;
        if let Some(query) = next {
            out.push(Request::Client(jig_client::Event::Watch {
                handle: Self::live(),
                subject: Watchable::Jig(c::Subject::Tasks { query, window: 1 }),
            }));
            out.push(Request::Client(jig_client::Event::Read {
                handle: Self::read_handle(),
                read: Readable::Jig(c::Read::Tasks {
                    query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                    live: None,
                    ended: None,
                }),
            }));
        }
    }
    pub(super) fn told(&mut self, told: &Told) -> bool {
        match told {
            Told::Page { handle, page } if *handle == Self::read_handle() => {
                if self.query.is_some() {
                    self.ended = match page {
                        Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) => match &page.ended {
                            c::EndedGroup::Rows { rows, .. } => !rows.is_empty(),
                            c::EndedGroup::None => false,
                        },
                        Readout::Jig(c::ReadResult::Value(_) | c::ReadResult::None(_))
                        | Readout::Family { .. }
                        | Readout::NotServed { .. }
                        | Readout::Limit => self.ended,
                    };
                }
                true
            }
            Told::Changed { handle } | Told::Ended { handle, .. } => *handle == Self::live(),
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Page { .. } => false,
        }
    }
    pub(super) fn has(&self, client: &Client, project: u32) -> bool {
        let Some(query) = self.query else { return false };
        if query.project != Some(project) {
            return false;
        }
        if self.ended {
            return true;
        }
        if let Some(document) = client.document(Self::live())
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
    activity: Wall,
    read_at: Option<Wall>,
    report: Option<report::Report>,
}

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "handles and filter are public; read bookkeeping belongs to the page")]
pub struct Goals {
    pub filter: GoalsFilter,
    pub live: Handle,
    pub ended: Handle,
    pub needs: Handle,
    pub report: Handle,
    query: Option<c::TaskQuery>,
    page: Option<c::TaskPage>,
    first: bool,
    reading_page: bool,
    page_answered: bool,
    count: u32,
    needs_total: Option<u32>,
    selection: Option<List<c::InboxSelection>>,
    rows: List<Row>,
    reading_report: Option<(u64, Wall)>,
}

impl Goals {
    pub(crate) fn new(first: u64) -> Self {
        Self {
            filter: GoalsFilter::All,
            live: handle(first),
            ended: handle(first.checked_add(1).expect("page handles fit")),
            needs: handle(first.checked_add(2).expect("page handles fit")),
            report: handle(first.checked_add(3).expect("page handles fit")),
            query: None,
            page: None,
            first: true,
            reading_page: false,
            page_answered: false,
            count: 0,
            needs_total: None,
            selection: None,
            rows: List::with_capacity(0),
            reading_report: None,
        }
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [self.live, self.ended, self.needs, self.report] {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        self.report.number.checked_add(1).expect("page handles fit")
    }
    pub(crate) fn configure(
        &mut self,
        me: Option<&c::Me>,
        project: u32,
        client: &Client,
        limits: &Limits,
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
            self.query = next;
            self.page = None;
            self.page_answered = false;
            self.reading_page = false;
            self.first = true;
            self.count = 0;
            self.needs_total = None;
            self.selection = None;
            self.reading_report = None;
            self.rows = List::with_capacity(limits.client.domain.rows.checked_mul(2).expect("goal rows fit"));
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
            && self.needs_total != Some(me.needs_total)
        {
            self.needs_total = Some(me.needs_total);
            self.read_needs(out);
        }
        self.reconcile(client, out);
    }
    fn read(&mut self, ended: Option<c::Cursor>, out: &mut Queue<Request>) {
        if let Some(query) = self.query {
            self.reading_page = true;
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.ended,
                read: Readable::Jig(c::Read::Tasks {
                    query: c::TaskQuery { phase: Some(c::PhaseClass::Ended), order: c::QueryOrder::Ended, ..query },
                    live: None,
                    ended,
                }),
            }));
        }
    }
    fn read_needs(&self, out: &mut Queue<Request>) {
        if let Some(query) = self.query {
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
        if !self.reading_page
            && let Some(page) = &self.page
            && let c::EndedGroup::Rows { after: Some(after), .. } = &page.ended
        {
            let after = after.clone();
            self.first = false;
            self.read(Some(after), out);
        }
    }
    pub(crate) fn told(&mut self, told: Told, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Changed { handle } if handle == self.live => {
                if self.first {
                    self.read(None, out);
                }
                self.read_needs(out);
                true
            }
            Told::Page { handle, page } if handle == self.ended => {
                self.reading_page = false;
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                    self.page = Some(*page);
                    self.page_answered = true;
                    let read = u32::try_from(self.ended().len()).expect("goal page fits");
                    self.count =
                        if self.first { read } else { self.count.checked_add(read).expect("goals count fits") };
                }
                true
            }
            Told::Page { handle, page } if handle == self.needs => {
                self.selection = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(page))) => page.selection,
                    Readout::Jig(c::ReadResult::Value(_) | c::ReadResult::None(_))
                    | Readout::Family { .. }
                    | Readout::NotServed { .. }
                    | Readout::Limit => None,
                };
                true
            }
            Told::Page { handle, page } if handle == self.report => {
                if let Some((task, at)) = self.reading_report.take() {
                    for row in self.rows.iter_mut() {
                        if row.task == task {
                            row.read_at = Some(at);
                            row.report = match &page {
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
                        }
                    }
                }
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
    fn reconcile(&mut self, client: &Client, out: &mut Queue<Request>) {
        let mut rows = List::with_capacity(self.rows.capacity());
        let mut previous = core::mem::replace(&mut self.rows, List::with_capacity(0));
        let live = match self.live(client) {
            Some(live) => live.rows.as_slice(),
            None => &[],
        };
        for lines in [live, self.ended()] {
            for line in lines {
                if rows.len() == rows.capacity() {
                    break;
                }
                let mut row = Row { task: line.chip.task, activity: line.last_activity, read_at: None, report: None };
                for index in 0..previous.len() {
                    if previous.get(index).expect("retained goal row").task == row.task {
                        row = previous.swap_remove(index).expect("retained goal row");
                        row.activity = line.last_activity;
                        break;
                    }
                }
                rows.push(row).expect("live and ended goal rows fit");
            }
        }
        self.rows = rows;
        if self.reading_report.is_some() {
            return;
        }
        for row in &self.rows {
            if row.read_at != Some(row.activity) {
                self.reading_report = Some((row.task, row.activity));
                out.push(Request::Client(jig_client::Event::Read {
                    handle: self.report,
                    read: Readable::Jig(report::read(
                        self.query.expect("configured goals").project.expect("project query"),
                        row.task,
                    )),
                }));
                break;
            }
        }
    }
    /// The live watch has delivered and the ended read has answered with its value, for this query.
    #[must_use]
    pub fn known(&self, client: &Client) -> bool {
        self.page_answered && self.live(client).is_some()
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
    Some(c::TaskQuery {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Needs { funds: bool },
    Paused,
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
        c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Active(_)
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_) => {}
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
            State::Paused | State::Planning | State::Running | State::Landed | State::Closed | State::Failed => false,
        },
        GoalsFilter::Live => match state {
            State::Paused | State::Planning | State::Running => true,
            State::Needs { .. } | State::Landed | State::Closed | State::Failed => false,
        },
        GoalsFilter::Ended => match state {
            State::Landed | State::Closed | State::Failed => true,
            State::Needs { .. } | State::Paused | State::Planning | State::Running => false,
        },
    }
}
