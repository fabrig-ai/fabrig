//! The changes watch and one task read per shown row (temper/docs/design/client/pages/README.md, temper/docs/design/client/forge/records.md).
use super::Request;
use crate::{forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, contract as c};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::v1 as w;

#[derive(Debug)]
pub enum State {
    Loading,
    Shown(Box<w::ChangesPanel>),
    Ended,
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "project is the page's public identity; its handles are owned here")]
pub struct Changes {
    pub project: u32,
    handle: Option<Handle>,
    state: State,
    reads: List<Row>,
    next: u64,
}
#[derive(Debug)]
struct Row {
    task: u64,
    handle: Handle,
    document: Option<Box<c::Task>>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum From<'a> {
    Goal(&'a [u8]),
    Chat(u64),
}
impl Changes {
    pub(super) fn new(project: u32, serial: &mut u64, limits: forge::Limits, out: &mut Queue<Request>) -> Self {
        let handle = Handle { asker: Owner::Pages.asker(), number: *serial };
        *serial = serial.checked_add(1).expect("page handles fit");
        let mut page = Self { project, handle: None, state: State::Ended, reads: List::uncapped(), next: *serial };
        if let Some(subject) = forge::changes::watch(project, &limits) {
            page.handle = Some(handle);
            page.state = State::Loading;
            out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
        }
        page
    }
    pub(super) const fn next(&self) -> u64 {
        self.next
    }
    pub(super) fn reserve(&mut self, next: u64) {
        self.next = self.next.max(next);
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            unwatch(handle, out);
        }
        for row in &self.reads {
            unwatch(row.handle, out);
        }
    }
    pub(super) fn told(
        &mut self,
        told: &Told,
        client: &Client,
        limits: forge::Limits,
        out: &mut Queue<Request>,
    ) -> bool {
        match told {
            Told::Changed { handle } if Some(*handle) == self.handle => {
                let panel = match client.document(*handle) {
                    Some(document) if document.state == jig_client::State::Ended => None,
                    Some(document) => match document.latest {
                        Some(Latest::Family(record)) => forge::changes::panel(record, limits),
                        Some(Latest::Jig(_)) => None,
                        None => return true,
                    },
                    None => return true,
                };
                match panel {
                    Some(panel) if panel.project() == self.project => {
                        self.rows(&panel, out);
                        self.state = State::Shown(Box::new(panel));
                    }
                    Some(_) | None => self.end(out),
                }
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.handle => {
                self.end(out);
                true
            }
            Told::Page { handle, page } => {
                for row in self.reads.iter_mut() {
                    if row.handle == *handle {
                        row.document = match page {
                            Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(task)))
                                if task.chip.task == row.task =>
                            {
                                Some(task.clone())
                            }
                            Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(value)))
                                if value.matches(c::ObjectKey::Task(row.task)) =>
                            {
                                match &value.value {
                                    c::EndedValue::Task { document, .. } => Some(document.clone()),
                                    c::EndedValue::Proposal(_)
                                    | c::EndedValue::PersonProposal { .. }
                                    | c::EndedValue::Escalation { .. }
                                    | c::EndedValue::Question { .. }
                                    | c::EndedValue::PersonTask { .. } => None,
                                }
                            }
                            Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => {
                                None
                            }
                        };
                        return true;
                    }
                }
                false
            }
            Told::Link { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Text { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. } => false,
        }
    }
    fn end(&mut self, out: &mut Queue<Request>) {
        self.leave(out);
        self.handle = None;
        self.reads.clear();
        self.state = State::Ended;
    }
    fn rows(&mut self, panel: &w::ChangesPanel, out: &mut Queue<Request>) {
        let mut tasks = List::uncapped();
        for queue in panel.queues() {
            if let Some(flight) = queue.in_flight()
                && !tasks.as_slice().contains(&flight.member())
            {
                tasks.push(flight.member()).expect("wire bounds rows");
            }
            for row in queue.changes() {
                if !tasks.as_slice().contains(&row.task()) {
                    tasks.push(row.task()).expect("wire bounds rows");
                }
            }
        }
        for row in panel.landed() {
            if !tasks.as_slice().contains(&row.task()) {
                tasks.push(row.task()).expect("wire bounds rows");
            }
        }
        for row in panel.landed_default() {
            if !tasks.as_slice().contains(&row.task()) {
                tasks.push(row.task()).expect("wire bounds rows");
            }
        }
        let mut kept = List::uncapped();
        for row in core::mem::replace(&mut self.reads, List::uncapped()).into_boxed() {
            if tasks.as_slice().contains(&row.task) {
                kept.push(row).expect("wire bounds rows");
            } else {
                unwatch(row.handle, out);
            }
        }
        for task in tasks.into_boxed() {
            let mut found = false;
            for row in &kept {
                if row.task == task {
                    found = true;
                }
            }
            if !found {
                let handle = Handle { asker: Owner::Pages.asker(), number: self.next };
                self.next = self.next.checked_add(1).expect("page handles fit");
                out.push(Request::Client(jig_client::Event::Read {
                    handle,
                    read: Readable::Jig(c::Read::Task { task }),
                }));
                kept.push(Row { task, handle, document: None }).expect("wire bounds rows");
            }
        }
        self.reads = kept;
    }
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }
    /// The goal named by a listed task, once its read has arrived (client/pages/README.md).
    #[must_use]
    pub fn goal(&self, task: u64) -> Option<u64> {
        for row in &self.reads {
            if row.task == task {
                return row.document.as_ref()?.goal;
            }
        }
        None
    }
    /// Where a listed task came from, once its read has arrived.
    #[must_use]
    pub fn from<'a>(&'a self, client: &Client, task: u64) -> Option<From<'a>> {
        for row in &self.reads {
            if row.task == task {
                let task = row.document.as_deref()?;
                if let Some(goal) = task.goal {
                    for chip in &task.lineage_chips {
                        if chip.task == goal {
                            return Some(From::Goal(chip.title.bytes()));
                        }
                    }
                    return None;
                }
                let root = task.lineage_chips.get(0)?;
                let me = super::accounts::current_me(client)?;
                let template = jig_client::shapes::template(me, self.project, jig_client::shapes::CHAT)?;
                return match template.executor {
                    c::TemplateExecutor::Charter(charter) if root.executor == (c::ChipExecutor::Agent { charter }) => {
                        Some(From::Chat(root.task))
                    }
                    c::TemplateExecutor::Charter(_) | c::TemplateExecutor::Chosen => None,
                };
            }
        }
        None
    }
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
