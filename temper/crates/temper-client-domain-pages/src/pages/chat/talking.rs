//! A role's talk beside its chat thread (temper/docs/design/client/pages/talk.md, 1–2).
use super::{Chat, handle, unwatch};
use crate::pages::{Request, team};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, contract as c, thread::Line};
use skein_lib::{Queue, Wall};

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the address is public; read facts are exposed by accessors")]
pub struct Talking {
    pub project: u32,
    pub task: u64,
    name: Option<Box<[u8]>>,
    open: Option<c::Talk>,
    from: Option<u64>,
    cost: Option<u64>,
    spend: Option<(u64, u64)>,
    now: Option<team::Now>,
    reading: Option<Handle>,
    refresh: bool,
    last_activity: Option<Wall>,
}
impl Talking {
    #[must_use]
    pub fn name(&self) -> Option<&[u8]> {
        self.name.as_deref()
    }
    #[must_use]
    pub fn open(&self) -> Option<&c::Talk> {
        self.open.as_ref()
    }
    #[must_use]
    pub fn from(&self) -> Option<u64> {
        self.from
    }
    #[must_use]
    pub fn cost(&self) -> Option<u64> {
        self.cost
    }
    #[must_use]
    pub fn spend(&self) -> Option<(u64, u64)> {
        self.spend
    }
    #[must_use]
    pub fn now(&self) -> Option<&team::Now> {
        self.now.as_ref()
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        if let Some(handle) = self.reading {
            unwatch(handle, out);
        }
    }
    pub(super) fn told(&mut self, told: &Told, out: &mut Queue<Request>) -> bool {
        match told {
            Told::Page { handle, page } if self.reading == Some(*handle) => {
                self.now = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(page))) => team::now(page),
                    Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => None,
                };
            }
            Told::Ended { handle, .. } if self.reading == Some(*handle) => self.now = None,
            Told::Page { .. }
            | Told::Ended { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Link { .. } => return false,
        }
        if let Some(handle) = self.reading.take() {
            unwatch(handle, out);
        }
        true
    }
}
impl Chat {
    pub(crate) fn leave_talk(&self, client: &Client, serial: &mut u64, out: &mut Queue<Request>) {
        let Some(talk) = &self.talking else { return };
        let Some(document) = self.task(client) else { return };
        let Some(open) = &document.talk else { return };
        let Some(Latest::Jig(c::Delivery::Me(me))) = super::latest(client, crate::pages::me_handle()) else { return };
        if open.party != me.party.number || !super::allowed(document, c::ActKind::Leave) {
            return;
        }
        *serial = serial.checked_add(1).expect("pages keys fit");
        out.push(Request::Client(jig_client::Event::Send {
            key: jig_client::Key { asker: crate::owner::Owner::Pages.asker(), number: *serial },
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::Leave { project: talk.project, task: talk.task })),
        }));
    }
    pub(crate) fn in_talk(&mut self, project: u32, client: &Client, out: &mut Queue<Request>) {
        self.opening_mode = None;
        self.talking = Some(Talking {
            project,
            task: self.task,
            name: None,
            open: None,
            from: None,
            cost: None,
            spend: None,
            now: None,
            reading: None,
            refresh: true,
            last_activity: None,
        });
        self.refresh_talk(client, out);
    }
    pub(crate) fn refresh_talk(&mut self, client: &Client, out: &mut Queue<Request>) {
        let Some(mut talk) = self.talking.take() else { return };
        let me = match super::latest(client, crate::pages::me_handle()) {
            Some(Latest::Jig(c::Delivery::Me(me))) => Some(me.party.number),
            Some(Latest::Jig(_) | Latest::Family(_)) | None => None,
        };
        talk.name = None;
        talk.open = None;
        talk.spend = None;
        if let Some(document) = self.task(client) {
            talk.name = Some(document.chip.title.bytes().into());
            if let Some(open) = &document.talk
                && Some(open.party) == me
            {
                talk.open = Some(*open);
            }
            talk.spend = Some((document.budget.numbers.spent, document.budget.numbers.budget));
            if talk.last_activity.is_some() && talk.last_activity != Some(document.last_activity) {
                talk.refresh = true;
            }
            talk.last_activity = Some(document.last_activity);
        }
        talk.from = None;
        talk.cost = None;
        if let Some(open) = &talk.open {
            let lines = self.thread.lines();
            for line in &lines {
                if let Line::Activity { line } = line
                    && let c::Activity::TalkOpened { party } = line.source.line
                    && party == open.party
                {
                    talk.from = Some(talk.from.unwrap_or(0).max(line.source.position));
                }
            }
            if let Some(from) = talk.from {
                let mut cost = 0_u64;
                for line in &lines {
                    if let Line::Activity { line } = line
                        && line.source.position > from
                        && let c::Activity::RunEnded { spent, .. } = line.source.line
                    {
                        cost = cost.saturating_add(spent);
                    }
                }
                talk.cost = Some(cost);
            }
        }
        if talk.refresh && talk.reading.is_none() {
            let handle = handle(self.next);
            self.next = self.next.checked_add(1).expect("talk handles fit");
            talk.reading = Some(handle);
            talk.refresh = false;
            out.push(Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(c::Read::Notes {
                    scope: c::NoteScope::Task { project: talk.project, task: talk.task },
                    description: Some(c::Text::new(b"now".as_slice().into(), 3).expect("now fits")),
                    after: None,
                }),
            }));
        }
        self.talking = Some(talk);
    }
}
