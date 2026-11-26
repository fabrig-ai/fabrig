//! A project's waits are read on entry or by a person's act, never watched (temper/docs/design/client/pages/waits.md, 1).
use super::Request;
use crate::{WaitsFilter, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Handle, Readable, Readout, Told, contract as c};
use skein_lib::{List, Queue};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    Busy,
    Refused,
    Unavailable,
}

#[derive(Debug)]
pub struct Waits {
    project: u32,
    filter: WaitsFilter,
    reader: Option<u64>,
    pending: Option<Handle>,
    page: Option<Box<c::WaitPage>>,
    problem: Option<Problem>,
}

impl Waits {
    pub(super) const fn new(project: u32, filter: WaitsFilter) -> Self {
        Self { project, filter, reader: None, pending: None, page: None, problem: None }
    }

    #[must_use]
    pub fn page(&self) -> Option<&c::WaitPage> {
        self.page.as_deref()
    }

    #[must_use]
    pub const fn problem(&self) -> Option<Problem> {
        self.problem
    }

    #[must_use]
    pub const fn reading(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn configure(&mut self, reader: u64, next: &mut u64, out: &mut Queue<Request>) {
        if self.reader != Some(reader) {
            self.reset(out);
            self.reader = Some(reader);
            self.read(reader, None, next, out);
        }
    }

    pub(super) fn reset(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.pending.take() {
            // Unwatch releases a handle's read too; there is no watch on this handle.
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        *self = Self::new(self.project, self.filter);
    }

    pub(super) fn more(&mut self, next: &mut u64, out: &mut Queue<Request>) {
        if !self.reading()
            && let Some(reader) = self.reader
            && let Some(page) = &self.page
            && let Some(after) = &page.after
        {
            self.read(reader, Some(after.clone()), next, out);
        }
    }

    fn read(&mut self, reader: u64, after: Option<c::Cursor>, next: &mut u64, out: &mut Queue<Request>) {
        let handle = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        self.pending = Some(handle);
        self.problem = None;
        let due = match self.filter.due {
            Some(seconds) => Some(u64::from(seconds).checked_mul(1_000_000_000).expect("u32 seconds fit nanoseconds")),
            None => None,
        };
        out.push(Request::Client(jig_client::Event::Read {
            handle,
            read: Readable::Jig(c::Read::Waits {
                project: self.project,
                kind: self.filter.kind,
                party: self.filter.mine.then_some(reader),
                due,
                after,
            }),
        }));
    }

    pub(super) fn told(&mut self, told: &Told) -> bool {
        let Told::Page { handle, page } = told else { return false };
        if self.pending != Some(*handle) {
            return false;
        }
        self.pending = None;
        self.problem = match page {
            Readout::Jig(c::ReadResult::Value(c::ReadValue::Waits(answer))) => {
                self.append(answer);
                None
            }
            Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)) => Some(Problem::Busy),
            Readout::Jig(c::ReadResult::None(c::ReadNone::Refused | c::ReadNone::Unknown)) => Some(Problem::Refused),
            Readout::Limit | Readout::Jig(_) | Readout::NotServed { .. } | Readout::Family { .. } => {
                Some(Problem::Unavailable)
            }
        };
        true
    }

    fn append(&mut self, answer: &c::WaitPage) {
        let page =
            self.page.get_or_insert(Box::new(c::WaitPage { now: answer.now, rows: List::uncapped(), after: None }));
        for row in &answer.rows {
            page.rows.push(row.clone()).expect("read wait rows fit");
        }
        page.after.clone_from(&answer.after);
    }
}
