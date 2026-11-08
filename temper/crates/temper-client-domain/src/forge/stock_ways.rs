//! Ask temper's stock ways and immediate acts (temper/docs/design/client/README.md, 4.6 and 5.13).
use super::{Limits, wire_limits};
use crate::{owner::Owner, pages::Request};
use alloc::boxed::Box;
use jig_client::{Client, Family, Handle, Key, Latest, Told, Watchable, contract as c};
use skein_lib::{List, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    Loading,
    Shown(Box<w::WaysPanel>),
    Ended,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayAct {
    Use,
    Keep,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Tab {
    pub index: Option<u32>,
    pub name: Box<[u8]>,
    pub line: Box<[u8]>,
    pub path: w::BranchPath,
    pub current: bool,
}
#[derive(Debug)]
pub struct StockWays {
    project: u32,
    handle: Option<Handle>,
    state: State,
    configured: bool,
    pending: Option<Pending>,
    refused: Option<c::Refusal>,
}
#[derive(Debug)]
struct Pending {
    key: Key,
    items: List<c::SettingItem>,
    retried: bool,
}
#[must_use]
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let subject = w::WaysSubject::new(&bounds, w::WaysSubjectParts { project }).ok()?;
    let watching = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Ways(subject) }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}
impl StockWays {
    pub(crate) fn new(project: u32, next: &mut u64, limits: Limits, out: &mut Queue<Request>) -> Self {
        let mut page =
            Self { project, handle: None, state: State::Ended, configured: false, pending: None, refused: None };
        if let Some(subject) = watch(project, &limits) {
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            page.handle = Some(handle);
            page.state = State::Loading;
            out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
        }
        page
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }
    pub(crate) const fn configure(&mut self, configured: bool) {
        self.configured = configured;
    }
    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }
    #[must_use]
    pub const fn refused(&self) -> Option<&c::Refusal> {
        self.refused.as_ref()
    }
    fn panel(&self) -> Option<&w::WaysPanel> {
        match &self.state {
            State::Shown(panel) => Some(panel),
            State::Loading | State::Ended => None,
        }
    }
    #[must_use]
    pub fn tabs(&self) -> List<Tab> {
        let mut tabs = List::uncapped();
        if let Some(panel) = self.panel() {
            for (index, way) in panel.ways().iter().enumerate() {
                let index = u32::try_from(index).expect("wire bounds ways");
                tabs.push(Tab {
                    index: Some(index),
                    name: way.name().into(),
                    line: way.line().into(),
                    path: way.path().clone(),
                    current: *panel.current() == Some(index),
                })
                .expect("wire bounds tabs");
            }
            if panel.current().is_none() {
                tabs.push(Tab {
                    index: None,
                    name: b"Yours".as_slice().into(),
                    line: Box::default(),
                    path: panel.own().clone(),
                    current: true,
                })
                .expect("one own tab");
            }
        }
        tabs
    }
    #[must_use]
    pub fn act(&self, index: u32) -> Option<WayAct> {
        let panel = self.panel()?;
        panel.ways().get(index)?;
        if !panel.may_change() || self.pending.is_some() {
            return None;
        }
        if *panel.current() != Some(index) {
            return Some(WayAct::Use);
        }
        if self.configured { None } else { Some(WayAct::Keep) }
    }
    pub(crate) fn send_act(
        &mut self,
        project: u32,
        index: Option<u32>,
        act: WayAct,
        client: &Client,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if project != self.project {
            return;
        }
        let Some(panel) = self.panel() else { return };
        let Some(index) = index.or(*panel.current()) else { return };
        if self.act(index) != Some(act) {
            return;
        }
        let Some(document) = client.document(super::ways::handle()) else { return };
        if document.state == jig_client::State::Ended {
            return;
        }
        let Some(Latest::Jig(c::Delivery::Settings(settings))) = document.latest else { return };
        let Some(way) = panel.ways().get(index) else { return };
        let mut items = List::uncapped();
        for item in way.items() {
            let edit = match item.edit() {
                w::SettingEdit::Set(value) => {
                    c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(value.value().into()) })
                }
                w::SettingEdit::Removed => c::SettingEdit::Removed,
                w::SettingEdit::Reset => return,
            };
            items
                .push(c::SettingItem {
                    setting: c::Setting::Application { kind: item.kind(), scope: item.scope(), number: item.number() },
                    edit,
                })
                .expect("wire bounds way items");
        }
        self.refused = None;
        self.send(
            Pending { key: Key { asker: Owner::Pages.asker(), number: 0 }, items, retried: false },
            settings.newest,
            serial,
            out,
        );
    }
    fn send(&mut self, mut pending: Pending, against: u32, serial: &mut u64, out: &mut Queue<Request>) {
        *serial = serial.checked_add(1).expect("page requests fit");
        pending.key = Key { asker: Owner::Pages.asker(), number: *serial };
        out.push(Request::Client(jig_client::Event::Send {
            key: pending.key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(c::Ask::ChangeSettings(c::ChangeSettings {
                against,
                items: pending.items.clone(),
                running: c::Running::Stays,
            }))),
        }));
        self.pending = Some(pending);
    }
    pub(crate) fn told(
        &mut self,
        told: &Told,
        client: &Client,
        limits: Limits,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) -> bool {
        match told {
            Told::Changed { handle } if Some(*handle) == self.handle => {
                let panel = match client.document(*handle) {
                    Some(document) => match document.state {
                        jig_client::State::Ended => None,
                        jig_client::State::Opening | jig_client::State::Behind => return true,
                        jig_client::State::Live => match document.latest {
                            Some(Latest::Family(record)) => panel(record, limits),
                            Some(Latest::Jig(_)) => None,
                            None => return true,
                        },
                    },
                    None => return true,
                };
                match panel {
                    Some(panel) if panel.project() == self.project => {
                        self.refused = None;
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
            Told::Answered { key, outcome }
                if match &self.pending {
                    Some(pending) => pending.key == *key,
                    None => false,
                } =>
            {
                self.answered(outcome, serial, out);
                true
            }
            Told::Link { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Text { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. } => false,
        }
    }
    fn answered(&mut self, outcome: &jig_client::Outcome, serial: &mut u64, out: &mut Queue<Request>) {
        let mut pending = self.pending.take().expect("matched pending act");
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Stale(stale),
            ..
        })) = outcome
            && !pending.retried
        {
            pending.retried = true;
            self.send(pending, stale.newest, serial, out);
        } else {
            self.refused = match outcome {
                jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused { refusal, .. })) => {
                    Some(refusal.clone())
                }
                jig_client::Outcome::Answered(
                    jig_client::Answered::Jig(c::Answer::Done(_)) | jig_client::Answered::Family(_),
                )
                | jig_client::Outcome::NotKept
                | jig_client::Outcome::Unknown
                | jig_client::Outcome::Limit
                | jig_client::Outcome::Unconfirmed
                | jig_client::Outcome::Held
                | jig_client::Outcome::Unsent
                | jig_client::Outcome::Full { .. }
                | jig_client::Outcome::Lost => None,
            };
        }
    }
    fn end(&mut self, out: &mut Queue<Request>) {
        self.leave(out);
        self.handle = None;
        self.state = State::Ended;
    }
}

fn panel(record: &Family, limits: Limits) -> Option<w::WaysPanel> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_ways_panel(&record.bytes, &wire_limits(&limits)?)
}
