//! Views read when shown and at a newer revision (temper/docs/design/client/pages/chat/conversation.md, 3).
use super::{Chat, Request, handle, unwatch};
use crate::{
    Limits, forge,
    views::{Part, parts},
};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, contract as c};
use skein_lib::{Duration, List, Map, Queue, Time};
use temper_client_protocol_wire::{self as wire, v1 as w};

/// A view's last answer, kept while a newer revision is read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    Asking,
    Drawn(Box<w::View>),
    Failed,
}

#[derive(Debug)]
struct Reading {
    address: Box<[u8]>,
    project: Option<u32>,
    handle: Handle,
    pending: bool,
    again: bool,
    retry_at: Option<Time>,
    wait: Duration,
}

#[derive(Debug)]
pub(super) struct Views {
    shown: Map<Box<[u8]>, Shown>,
    reads: List<Reading>,
    limits: forge::Limits,
    revision: Option<u32>,
}
impl Views {
    pub(super) fn new(limits: forge::Limits) -> Self {
        Self {
            shown: Map::with_capacity(Limits::chat_views()),
            reads: List::with_capacity(Limits::chat_views()),
            limits,
            revision: None,
        }
    }
    fn sync(
        &mut self,
        wanted: Map<Box<[u8]>, ()>,
        project: Option<u32>,
        client: &Client,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        for index in (0..self.reads.len()).rev() {
            let read = self.reads.get(index).expect("bounded view read");
            if !wanted.contains_key(read.address.as_ref()) || read.project != project {
                let read = self.reads.swap_remove(index).expect("bounded view read");
                self.shown.remove(read.address.as_ref()).expect("read has shown state");
                unwatch(read.handle, out);
            }
        }
        let revision = match client.document(forge::ways::handle()) {
            Some(document) => match document.latest {
                Some(Latest::Jig(c::Delivery::Settings(settings))) => Some(settings.newest),
                Some(Latest::Jig(_) | Latest::Family(_)) | None => None,
            },
            None => None,
        };
        if let Some(newest) = revision {
            let newer = match self.revision {
                Some(old) => newest > old,
                None => true,
            };
            if newer {
                self.revision = Some(newest);
                for read in self.reads.iter_mut() {
                    match self.shown.get(read.address.as_ref()).expect("read has shown state") {
                        Shown::Drawn(_) => read.again = true,
                        Shown::Asking | Shown::Failed => {}
                    }
                }
            }
        }
        for (address, ()) in &wanted {
            if !self.shown.contains_key(address.as_ref()) {
                let handle = handle(*next);
                *next = next.checked_add(1).expect("chat handles fit");
                self.shown.insert(address.clone(), Shown::Asking).expect("bounded views");
                self.reads
                    .push(Reading {
                        address: address.clone(),
                        project,
                        handle,
                        pending: false,
                        again: true,
                        retry_at: None,
                        wait: Duration::from_secs(1),
                    })
                    .expect("one read per view");
            }
        }
        self.fire(Time::ZERO, out);
    }
    pub(super) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        for read in self.reads.iter_mut() {
            if read.pending || !read.again {
                continue;
            }
            if let Some(at) = read.retry_at
                && at > now
            {
                continue;
            }
            read.again = false;
            read.retry_at = None;
            if let Some(record) = reading(read, self.limits) {
                read.pending = true;
                out.push(Request::Client(jig_client::Event::Read { handle: read.handle, read: record }));
            } else {
                *self.shown.get_mut(read.address.as_ref()).expect("read has shown state") = Shown::Failed;
            }
        }
    }
    pub(super) fn page(&mut self, now: Time, client: &Client, handle: Handle, page: &Readout) -> bool {
        for read in self.reads.iter_mut() {
            if read.handle != handle || !read.pending {
                continue;
            }
            read.pending = false;
            let shown = self.shown.get_mut(read.address.as_ref()).expect("read has shown state");
            if super::super::activity::read_end(client, page).transient() {
                read.again = true;
                read.retry_at = Some(now.saturating_add(read.wait));
                read.wait = read.wait.saturating_mul(2).min(Duration::from_secs(30));
                match shown {
                    Shown::Drawn(_) => {}
                    Shown::Asking | Shown::Failed => *shown = Shown::Failed,
                }
            } else {
                *shown = match answer(page, self.limits) {
                    Some(view) => Shown::Drawn(Box::new(view)),
                    None => Shown::Failed,
                };
                read.wait = Duration::from_secs(1);
                read.retry_at = if read.again { Some(now) } else { None };
            }
            return true;
        }
        false
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        let mut at = None;
        for read in &self.reads {
            at = super::super::activity::earliest(at, read.retry_at);
        }
        at
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        for read in &self.reads {
            unwatch(read.handle, out);
        }
    }
}
impl Chat {
    #[must_use]
    pub fn view(&self, address: &[u8]) -> Option<&Shown> {
        self.views.shown.get(address)
    }
    pub(super) fn sync_views(&mut self, client: &Client, out: &mut Queue<Request>) {
        let mut wanted = Map::with_capacity(Limits::chat_views());
        if let Some(draft) = &self.draft {
            gather(&draft.body, &mut wanted);
        }
        for line in self.thread.lines().iter().rev() {
            if let jig_client::thread::Line::Turn { turn, .. } = line {
                for message in turn.messages.iter().rev() {
                    for block in message.blocks.iter().rev() {
                        if let c::Block::Text(text) = block {
                            gather(text.bytes(), &mut wanted);
                        }
                    }
                }
            }
        }
        let project = match self.task(client) {
            Some(task) if task.project != 0 => Some(task.project),
            Some(_) | None => None,
        };
        self.views.sync(wanted, project, client, &mut self.next, out);
    }
}
#[expect(clippy::disallowed_methods, reason = "the view splitter borrows the protocol-validated text bytes as UTF-8")]
fn gather(text: &[u8], wanted: &mut Map<Box<[u8]>, ()>) {
    let Ok(text) = core::str::from_utf8(text) else { return };
    for part in parts(text).iter().rev() {
        match part {
            Part::View(address) if wanted.len() < wanted.capacity() => {
                wanted.insert(Box::from(*address), ()).expect("room for view");
            }
            Part::View(_) | Part::Text(_) => {}
        }
    }
}
fn reading(read: &Reading, limits: forge::Limits) -> Option<Readable> {
    let project = read.project?;
    let bounds = forge::wire_limits(&limits)?;
    let record = w::Reading::new(
        &bounds,
        w::ReadingParts {
            read: w::Read::View(
                w::ViewRead::new(&bounds, w::ViewReadParts { project, address: read.address.clone() }).ok()?,
            ),
        },
    )
    .ok()?;
    Some(Readable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::reading(&record),
        },
    })
}
fn answer(page: &Readout, limits: forge::Limits) -> Option<w::View> {
    let Readout::Family { family: wire::FORGE, record: Some(bytes) } = page else { return None };
    Some(wire::decode_view_answer(bytes, &forge::wire_limits(&limits)?)?.into_parts().view)
}
