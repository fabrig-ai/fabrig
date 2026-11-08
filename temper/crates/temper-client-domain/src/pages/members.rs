//! Deployment's project members (temper/docs/design/client/README.md, 3.5, Members).
use super::Request;
use crate::owner::Owner;
use alloc::boxed::Box;
use jig_client::{Handle, Readable, Readout, Told, contract as c};
use skein_lib::{List, Queue};

#[derive(Debug)]
pub enum Read {
    Unread,
    Refused,
    Project(Box<c::Project>),
}

#[derive(Debug)]
pub struct Members {
    projects: List<c::MeProject>,
    reads: List<Read>,
    pending: Option<Handle>,
    pending_cursor: Option<u32>,
    cursor: u32,
}

impl Members {
    pub(super) fn new() -> Self {
        Self { projects: List::uncapped(), reads: List::uncapped(), pending: None, pending_cursor: None, cursor: 0 }
    }

    pub(super) fn configure(&mut self, me: Option<&c::Me>, next: &mut u64, out: &mut Queue<Request>) {
        let Some(me) = me else { return };
        if self.projects != me.projects {
            self.projects = me.projects.clone();
            self.reads.clear();
            for _project in &self.projects {
                self.reads.push(Read::Unread).expect("uncapped project outcomes");
            }
            self.pending_cursor = None;
            self.cursor = 0;
        }
        if self.pending.is_some() {
            return;
        }
        let Some(project) = self.projects.get(self.cursor) else { return };
        let handle = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        self.pending = Some(handle);
        self.pending_cursor = Some(self.cursor);
        out.push(Request::Client(jig_client::Event::Read {
            handle,
            read: Readable::Jig(c::Read::Project { project: project.project }),
        }));
    }

    pub(super) fn told(&mut self, told: &Told) -> bool {
        let Told::Page { handle, page } = told else { return false };
        if self.pending != Some(*handle) {
            return false;
        }
        if let Some(cursor) = self.pending_cursor.take() {
            let read = self.reads.get_mut(cursor).expect("pending project's outcome");
            if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(project))) = page {
                *read = Read::Project(project.clone());
            } else if let Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) = page {
                *read = Read::Refused;
            }
            self.cursor = self.cursor.checked_add(1).expect("project rows fit");
        }
        self.pending = None;
        true
    }

    #[must_use]
    pub fn read(&self, project: u32) -> &Read {
        for (index, row) in self.projects.iter().enumerate() {
            if row.project == project {
                return self.reads.get(u32::try_from(index).expect("project rows fit")).expect("project outcome");
            }
        }
        &Read::Unread
    }
}
