//! Standing roles and the selected card (client/pages/team.md, 1).
use super::{Facts, ROLES, new_handle};
use crate::{
    Address,
    owner::Owner,
    pages::{Request, chat, needs::Toasted, task},
};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Key, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Wall};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Role {
    pub task: u64,
    pub name: Box<[u8]>,
    pub now: Option<Box<[u8]>>,
    pub spent: u64,
    pub budget: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Now {
    pub body: Box<[u8]>,
    pub changed: Option<Wall>,
    pub cut: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartedBy {
    You,
    Member(Box<[u8]>),
    Requester(c::Requester),
}
/// The card's read facts; sibling parts join this struct.
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "card facts keep independent control permissions and document cut")]
pub struct Card {
    pub task: u64,
    pub name: Box<[u8]>,
    pub made: Option<Wall>,
    pub requester: Option<StartedBy>,
    pub hold: Option<Box<c::ChipHold>>,
    pub paused: bool,
    pub pause: bool,
    pub resume: bool,
    pub now: Option<Now>,
    pub report: Option<Now>,
    pub sessions: List<super::Session>,
    pub rules: Option<super::Rules>,
    pub wakes: List<super::wakes::Row>,
    pub wakes_cut: bool,
}
#[derive(Debug)]
struct Listed {
    line: c::TaskLine,
    handle: Option<Handle>,
    now: Option<Now>,
    reading: bool,
    refresh: bool,
}
#[derive(Debug)]
struct Report {
    handle: Option<Handle>,
    body: Option<Now>,
    refresh: bool,
}
impl Report {
    const fn new() -> Self {
        Self { handle: None, body: None, refresh: false }
    }
}
#[derive(Debug)]
pub(super) struct Standing {
    watch: Option<Handle>,
    read: bool,
    rows: List<Listed>,
    more: bool,
    selected: Option<u64>,
    card: Option<(u64, Handle)>,
    pending: List<(Key, task::Sending)>,
    rules: super::rules::Reader,
    report: Report,
}
impl Default for Standing {
    fn default() -> Self {
        Self {
            watch: None,
            read: false,
            rows: List::with_capacity(ROLES),
            more: false,
            selected: None,
            card: None,
            pending: List::with_capacity(2),
            rules: super::rules::Reader::default(),
            report: Report::new(),
        }
    }
}
impl Standing {
    pub(super) fn count(&self) -> Option<u32> {
        if self.read { Some(self.rows.len()) } else { None }
    }
    pub(super) fn configure(&mut self, project: u32, address: &Address, next: &mut u64, out: &mut Queue<Request>) {
        if self.watch.is_none() {
            let handle = new_handle(next);
            self.watch = Some(handle);
            let query = c::TaskQuery {
                project: Some(project),
                label: Some(3),
                order: c::QueryOrder::Number,
                requester: None,
                root: None,
                parent: None,
                phase: None,
                executor: None,
                tracked: None,
            };
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Tasks { query, window: ROLES }),
            }));
        }
        let Address::Team { role, .. } = address else {
            self.close_card(out);
            for row in self.rows.iter_mut() {
                close_now(row, out);
            }
            return;
        };
        self.selected = *role;
        // One Now read in flight; its handle is released at its answer, so sixteen
        // roles coexist with the eight other people’s watches within the client’s bounds.
        let mut reading = false;
        for row in &self.rows {
            reading |= row.reading;
        }
        for row in self.rows.iter_mut() {
            if !reading && row.refresh && !row.reading {
                if let Some(handle) = row.handle.take() {
                    unwatch(handle, out);
                }
                let handle = new_handle(next);
                row.handle = Some(handle);
                row.reading = true;
                row.refresh = false;
                reading = true;
                out.push(Request::Client(jig_client::Event::Read {
                    handle,
                    read: Readable::Jig(c::Read::Notes {
                        scope: c::NoteScope::Task { project, task: row.line.chip.task },
                        description: Some(c::Text::new(b"now".as_slice().into(), 3).expect("now fits")),
                        after: None,
                    }),
                }));
            }
        }
        let wanted = self.chosen();
        if wanted
            != match self.card {
                Some((task, _)) => Some(task),
                None => None,
            }
        {
            self.close_card(out);
            if let Some(task) = wanted {
                let handle = new_handle(next);
                self.card = Some((task, handle));
                self.report.refresh = true;
                out.push(Request::Client(jig_client::Event::Watch {
                    handle,
                    subject: Watchable::Jig(c::Subject::Task { task }),
                }));
            }
        }
        // The card's report has its own in-flight slot so it never delays a Now.
        if self.report.refresh && self.report.handle.is_none() && let Some(task) = wanted {
            let handle = new_handle(next);
            self.report.handle = Some(handle);
            self.report.refresh = false;
            out.push(Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(c::Read::Notes {
                    scope: c::NoteScope::Task { project, task },
                    description: Some(c::Text::new(b"report".as_slice().into(), 6).expect("report fits")),
                    after: None,
                }),
            }));
        }
    }
    pub(super) fn configure_rules(
        &mut self,
        client: &Client,
        limits: crate::forge::Limits,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        // The card watch is closed on leaving Team, even while the roles watch stays open.
        let document = card_document(self.card, self.chosen(), self.read, client);
        self.rules.configure(document, client, limits, next, out);
    }
    pub(super) fn rules_told(&mut self, told: &Told, limits: crate::forge::Limits) -> bool {
        self.rules.told(told, limits)
    }
    fn chosen(&self) -> Option<u64> {
        for row in &self.rows {
            if Some(row.line.chip.task) == self.selected {
                return self.selected;
            }
        }
        if let Some(row) = self.rows.get(0) { Some(row.line.chip.task) } else { None }
    }
    fn close_card(&mut self, out: &mut Queue<Request>) {
        self.rules.leave(out);
        if let Some(handle) = self.report.handle.take() {
            unwatch(handle, out);
        }
        self.report = Report::new();
        if let Some((_, handle)) = self.card.take() {
            unwatch(handle, out);
        }
    }
    pub(super) fn leave(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.watch {
            unwatch(handle, out);
        }
        self.close_card(out);
        for row in self.rows.iter_mut() {
            close_now(row, out);
        }
    }
    pub(super) fn told(&mut self, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Page { handle, page } if self.report.handle == Some(*handle) => {
                self.report.body = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) => note_body(page, b"report"),
                    Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => None,
                };
                self.report.handle = None;
                unwatch(*handle, out);
                true
            }
            Told::Ended { handle, .. } if self.report.handle == Some(*handle) => {
                self.report.body = None;
                self.report.handle = None;
                unwatch(*handle, out);
                true
            }
            Told::Changed { handle } if self.watch == Some(*handle) => {
                let tasks = match latest(client, *handle) {
                    Some(Latest::Jig(c::Delivery::Tasks(tasks))) => Some(tasks.as_ref()),
                    Some(Latest::Jig(_) | Latest::Family(_)) | None => None,
                };
                self.read = tasks.is_some();
                self.more = false;
                if let Some(tasks) = tasks {
                    self.update(tasks, out);
                } else {
                    self.update(&c::Tasks { rows: List::with_capacity(0), more: false }, out);
                }
                true
            }
            Told::Ended { handle, .. } if self.watch == Some(*handle) => {
                self.read = false;
                self.more = false;
                for row in self.rows.iter_mut() {
                    close_now(row, out);
                }
                self.rows = List::with_capacity(ROLES);
                true
            }
            Told::Page { handle, page } => {
                for row in self.rows.iter_mut() {
                    if row.handle == Some(*handle) {
                        row.reading = false;
                        if let Some(handle) = row.handle.take() {
                            unwatch(handle, out);
                        }
                        row.now = match page {
                            Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) => note_body(page, b"now"),
                            Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => {
                                None
                            }
                        };
                        return true;
                    }
                }
                false
            }
            Told::Ended { handle, .. } => {
                for row in self.rows.iter_mut() {
                    if row.handle == Some(*handle) {
                        row.reading = false;
                        row.now = None;
                        if let Some(handle) = row.handle.take() {
                            unwatch(handle, out);
                        }
                        return true;
                    }
                }
                match self.card {
                    Some((_, watched)) => watched == *handle,
                    None => false,
                }
            }
            Told::Changed { handle } => match self.card {
                Some((_, watched)) => watched == *handle,
                None => false,
            },
            Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Link { .. }
            | Told::Text { .. } => false,
        }
    }
    fn update(&mut self, tasks: &c::Tasks, out: &mut Queue<Request>) {
        self.more = tasks.more;
        self.report.refresh = true;
        let mut rows: List<Listed> = List::with_capacity(ROLES);
        for line in &tasks.rows {
            if line.ended_at.is_some() || line.label != 3 || rows.len() == ROLES {
                continue;
            }
            let mut duplicate = false;
            for row in &rows {
                duplicate |= row.line.chip.task == line.chip.task;
            }
            if duplicate {
                continue;
            }
            let mut listed = Listed { line: line.clone(), handle: None, now: None, reading: false, refresh: true };
            for row in self.rows.iter_mut() {
                if row.line.chip.task == line.chip.task {
                    listed.handle = row.handle.take();
                    listed.now = row.now.take();
                    listed.reading = row.reading;
                    // A note write and a park do not move last_activity. Each
                    // delivered line invalidates its Now; an in-flight read coalesces it.
                    listed.refresh = true;
                    break;
                }
            }
            rows.push(listed).expect("at most ROLES unique rows");
        }
        for row in self.rows.iter_mut() {
            if let Some(handle) = row.handle.take() {
                unwatch(handle, out);
            }
        }
        self.rows = rows;
    }
    pub(super) fn gather(&self, client: &Client, project: Option<&c::Project>, you: u64, facts: &mut Facts) {
        facts.selected = self.selected;
        if !self.read {
            return;
        }
        facts.roles_cut = self.more;
        facts.roles = List::with_capacity(ROLES);
        for row in &self.rows {
            let first = if let Some(now) = &row.now {
                let end = skein_lib::bytes::find(&now.body, b"\n").unwrap_or(now.body.len());
                let line = now.body.get(..end).expect("first line fits Now");
                if line.is_empty() { None } else { Some(line.into()) }
            } else {
                None
            };
            facts
                .roles
                .push(Role {
                    task: row.line.chip.task,
                    name: row.line.chip.title.bytes().into(),
                    now: first,
                    spent: row.line.budget.numbers.spent,
                    budget: row.line.budget.numbers.budget,
                })
                .expect("roles fit watch");
        }
        for role in &facts.roles {
            if Some(role.task) == self.chosen() {
                let mut current = None;
                for row in &self.rows {
                    if row.line.chip.task == role.task {
                        current.clone_from(&row.now);
                    }
                }
                facts.card = Some(Box::new(Card {
                    task: role.task,
                    name: role.name.clone(),
                    made: None,
                    requester: None,
                    hold: None,
                    paused: false,
                    pause: false,
                    resume: false,
                    now: current,
                    report: self.report.body.clone(),
                    sessions: List::with_capacity(0),
                    rules: None,
                    wakes: List::with_capacity(0),
                    wakes_cut: false,
                }));
                break;
            }
        }
        if let Some(task) = self.document(client) {
            let mut requester = StartedBy::Requester(task.requester);
            if task.requester == c::Requester::Person(you) {
                requester = StartedBy::You;
            } else if let Some(project) = project {
                for member in &project.members {
                    if task.requester == c::Requester::Person(member.party) {
                        requester = StartedBy::Member(
                            if member.name.bytes().is_empty() { member.login.bytes() } else { member.name.bytes() }
                                .into(),
                        );
                        break;
                    }
                }
            }
            let mut current = None;
            for row in &self.rows {
                if row.line.chip.task == task.chip.task {
                    current.clone_from(&row.now);
                }
            }
            let paused = paused(task);
            facts.card = Some(Box::new(Card {
                task: task.chip.task,
                name: task.chip.title.bytes().into(),
                made: Some(task.made),
                requester: Some(requester),
                hold: task.hold.clone(),
                paused,
                pause: chat::allowed(task, c::ActKind::Stop),
                resume: chat::allowed(task, c::ActKind::Release),
                now: current,
                report: self.report.body.clone(),
                sessions: List::with_capacity(0),
                rules: self.rules.facts(),
                wakes: super::wakes::rows(&task.subscriptions, &task.wake),
                wakes_cut: task.cut,
            }));
        }
    }
    pub(super) fn activity(&self, client: &Client) -> Option<(u64, Option<u64>)> {
        let (task, _) = self.card?;
        let activity = if let Some(document) = self.document(client) { Some(document.activity) } else { None };
        Some((task, activity))
    }
    fn document<'a>(&self, client: &'a Client) -> Option<&'a c::Task> {
        card_document(self.card, self.chosen(), self.read, client)
    }

    pub(super) fn send(&mut self, client: &Client, kind: task::Sending, serial: &mut u64, out: &mut Queue<Request>) {
        for (_, pending) in &self.pending {
            if *pending == kind {
                return;
            }
        }
        let Some(document) = self.document(client) else { return };
        if kind == task::Sending::Stop && document.hold.is_some() {
            return;
        }
        let paused = paused(document);
        let Some(ask) = task::control(document, kind, paused) else { return };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.pending.push((key, kind)).expect("one request per control act");
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
    }
    pub(super) fn toasted(&mut self, told: &Told) -> Option<Toasted> {
        let Told::Answered { key, outcome } = told else { return None };
        let mut kind = None;
        let mut kept = List::with_capacity(2);
        for (pending, sending) in &self.pending {
            if pending == key {
                kind = Some(*sending);
            } else {
                kept.push((*pending, *sending)).expect("pending controls fit");
            }
        }
        self.pending = kept;
        let kind = kind?;
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_))) = outcome {
            match kind {
                task::Sending::Stop => Some(Toasted::Paused),
                task::Sending::Resume => Some(Toasted::Resumed),
                task::Sending::Message | task::Sending::Cancel | task::Sending::Amend => None,
            }
        } else {
            Some(Toasted::Refused { outcome: outcome.clone() })
        }
    }
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
fn close_now(row: &mut Listed, out: &mut Queue<Request>) {
    if let Some(handle) = row.handle.take() {
        unwatch(handle, out);
    }
    row.now = None;
    row.reading = false;
    row.refresh = true;
}
fn latest(client: &Client, handle: Handle) -> Option<&Latest> {
    let document = client.document(handle)?;
    if document.state == jig_client::State::Ended {
        return None;
    }
    document.latest
}
fn note_body(page: &c::NotePage, description: &[u8]) -> Option<Now> {
    let mut first: Option<&c::Note> = None;
    for note in &page.rows {
        if note.description.bytes() == description
            && match first {
                Some(previous) => note.number < previous.number,
                None => true,
            }
        {
            first = Some(note);
        }
    }
    let note = first?;
    Some(Now { body: note.body.bytes().into(), changed: note.changed, cut: note.body.cut })
}

fn paused(task: &c::Task) -> bool {
    let Some(hold) = &task.hold else { return false };
    match hold.reason {
        c::HoldCategory::Placed { .. } | c::HoldCategory::Stopped => {
            super::super::goals::state(&task.chip, false, false, 0) == super::super::goals::State::Paused
        }
        c::HoldCategory::Exceeded(_)
        | c::HoldCategory::Failures { .. }
        | c::HoldCategory::Stalled
        | c::HoldCategory::Dependency { .. }
        | c::HoldCategory::Drift
        | c::HoldCategory::Effects
        | c::HoldCategory::EffectFailed
        | c::HoldCategory::Uncertain { .. }
        | c::HoldCategory::Budget
        | c::HoldCategory::Deadline
        | c::HoldCategory::HoldsWaited
        | c::HoldCategory::Circle
        | c::HoldCategory::ResourceUnavailable
        | c::HoldCategory::ResourceAccess
        | c::HoldCategory::PoolLost { .. }
        | c::HoldCategory::Procedure => false,
    }
}

fn card_document(card: Option<(u64, Handle)>, chosen: Option<u64>, read: bool, client: &Client) -> Option<&c::Task> {
    let (task, handle) = card?;
    if Some(task) != chosen || !read {
        return None;
    }
    let Latest::Jig(c::Delivery::Task(document)) = latest(client, handle)? else { return None };
    if document.chip.task == task { Some(document) } else { None }
}
