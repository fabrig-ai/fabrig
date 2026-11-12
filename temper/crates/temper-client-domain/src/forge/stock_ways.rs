//! Ask temper's stock ways and immediate acts (temper/docs/design/client/forge/records.md and temper/docs/design/client/pages/settings.md, 2).
use super::{Limits, wire_limits};
use crate::{
    owner::Owner,
    pages::{Request, needs::Toasted},
};
use alloc::boxed::Box;
use jig_client::{Client, Family, Handle, Key, Latest, Readable, Readout, Told, Watchable, contract as c};
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
    pub headline: Box<[u8]>,
    pub line: Box<[u8]>,
    pub path: w::BranchPath,
    pub current: bool,
}
#[derive(Debug)]
pub struct StockWays {
    project: u32,
    handle: Option<Handle>,
    policy: Handle,
    state: State,
    configured: bool,
    shown: Option<u32>,
    open: Option<u32>,
    toast: Option<Toasted>,
    pending: Option<Pending>,
    refused: Option<c::Refusal>,
}
#[derive(Debug)]
enum Pending {
    Policy { against: u32, items: List<c::SettingItem>, receipt: Receipt },
    Change(Change),
}
#[derive(Debug)]
struct Change {
    key: Key,
    items: List<c::SettingItem>,
    retried: bool,
    receipt: Receipt,
}
#[derive(Debug)]
struct Receipt {
    name: Box<[u8]>,
    kept: bool,
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
        let policy = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        let mut page = Self {
            project,
            handle: None,
            policy,
            state: State::Ended,
            configured: false,
            shown: None,
            open: None,
            toast: None,
            pending: None,
            refused: None,
        };
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
        out.push(Request::Client(jig_client::Event::Unwatch { handle: self.policy }));
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
    /// The index into tabs of the tab shown (client/pages/settings.md, 2).
    #[must_use]
    pub fn shown(&self) -> Option<u32> {
        let panel = self.panel()?;
        match self.shown {
            Some(index) => {
                panel.ways().get(index)?;
                Some(index)
            }
            None => {
                if panel.current().is_none() {
                    Some(panel.ways().len())
                } else {
                    None
                }
            }
        }
    }
    /// The stop of the shown tab's path whose card is open.
    #[must_use]
    pub fn open(&self) -> Option<u32> {
        self.open
    }
    /// Whether a way's act is out: the act shows, disabled.
    #[must_use]
    pub const fn sending(&self) -> bool {
        self.pending.is_some()
    }
    pub(crate) fn toasted(&mut self) -> Option<Toasted> {
        self.toast.take()
    }
    pub(crate) fn show_tab(&mut self, project: u32, index: u32) {
        if project == self.project
            && let Some(tab) = self.tabs().get(index)
        {
            self.shown = tab.index;
            self.open = None;
        }
    }
    pub(crate) fn show_stop(&mut self, project: u32, index: u32) {
        if project != self.project {
            return;
        }
        let Some(shown) = self.shown() else { return };
        let tabs = self.tabs();
        let Some(tab) = tabs.get(shown) else { return };
        let Some(stop) = tab.path.stops().get(index) else { return };
        match stop.actor() {
            w::Actor::Bar => {}
            w::Actor::Person | w::Actor::Agent | w::Actor::Code => {
                self.open = if self.open == Some(index) { None } else { Some(index) };
            }
        }
    }
    fn show(&mut self, panel: w::WaysPanel) {
        let old_tabs = self.tabs();
        let old = match self.shown() {
            Some(index) => old_tabs.get(index),
            None => None,
        };
        self.state = State::Shown(Box::new(panel));
        let tabs = self.tabs();
        let mut shown = None;
        for tab in &tabs {
            if tab.current {
                shown = Some(tab);
            }
        }
        if let Some(old) = old {
            for tab in &tabs {
                if tab.index == old.index {
                    shown = Some(tab);
                    break;
                }
            }
        }
        self.shown = match shown {
            Some(tab) => tab.index,
            None => None,
        };
        let same = match (old, shown) {
            (Some(old), Some(shown)) => same_stops(old, shown),
            (Some(_) | None, None) | (None, Some(_)) => false,
        };
        if !same {
            self.open = None;
        }
    }
    /// The running path (client/pages/settings.md, 2, *What follows a landing*).
    #[must_use]
    pub fn current_path(&self) -> Option<&w::BranchPath> {
        let panel = self.panel()?;
        match panel.current() {
            Some(index) => Some(panel.ways().get(*index)?.path()),
            None => Some(panel.own()),
        }
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
                    headline: way.headline().into(),
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
                    headline: Box::default(),
                    line: Box::default(),
                    path: panel.own().clone(),
                    current: true,
                })
                .expect("one own tab");
            }
        }
        tabs
    }
    /// The project's words the read carries, kind 10 (client/forge/records.md).
    #[must_use]
    pub fn words(&self) -> Option<&w::ProjectWords> {
        self.panel()?.words().as_ref()
    }
    #[must_use]
    pub fn act(&self, index: u32) -> Option<WayAct> {
        let panel = self.panel()?;
        panel.ways().get(index)?;
        if !panel.may_change() {
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
        if project != self.project || self.sending() {
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
        let receipt = Receipt { name: way.name().into(), kept: act == WayAct::Keep };
        let delegation = way.delegation();
        self.refused = None;
        if delegation {
            self.pending = Some(Pending::Policy { against: settings.newest, items, receipt });
            out.push(Request::Client(jig_client::Event::Read {
                handle: self.policy,
                read: Readable::Jig(c::Read::Setting(c::ReadSetting {
                    setting: c::Setting::ProjectPolicy { project: self.project },
                    revision: settings.newest,
                    proposal: None,
                })),
            }));
        } else {
            self.send_items(items, settings.newest, receipt, serial, out);
        }
    }
    fn send_items(
        &mut self,
        items: List<c::SettingItem>,
        against: u32,
        receipt: Receipt,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        self.send(
            Change { key: Key { asker: Owner::Pages.asker(), number: 0 }, items, retried: false, receipt },
            against,
            serial,
            out,
        );
    }
    fn send(&mut self, mut pending: Change, against: u32, serial: &mut u64, out: &mut Queue<Request>) {
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
        self.pending = Some(Pending::Change(pending));
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
                        self.show(panel);
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
                    Some(Pending::Change(pending)) => pending.key == *key,
                    Some(Pending::Policy { .. }) | None => false,
                } =>
            {
                self.answered(outcome, serial, out);
                true
            }
            Told::Page { handle, page } if *handle == self.policy => {
                self.policy_read(page, serial, out);
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
    fn policy_read(&mut self, page: &Readout, serial: &mut u64, out: &mut Queue<Request>) {
        let (against, mut items, receipt) = match self.pending.take() {
            Some(Pending::Policy { against, items, receipt }) => (against, items, receipt),
            pending @ (Some(Pending::Change(_)) | None) => {
                self.pending = pending;
                return;
            }
        };
        let setting = c::Setting::ProjectPolicy { project: self.project };
        if let Some(mut policy) = whole_policy(page, setting, against) {
            if add_delegation(&mut policy) {
                items
                    .push(c::SettingItem {
                        setting,
                        edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Policy(policy) }),
                    })
                    .expect("one policy joins the way items");
            }
            self.send_items(items, against, receipt, serial, out);
        } else {
            let why: &[u8] = match page {
                Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)) => b"The project policy read was refused.",
                Readout::Jig(_) | Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit => {
                    b"The project policy could not be read whole."
                }
            };
            self.refused = Some(c::Refusal::Judged {
                at: 0,
                setting,
                why: c::Text::new(why.into(), 128).expect("policy read refusal fits"),
            });
        }
    }
    fn answered(&mut self, outcome: &jig_client::Outcome, serial: &mut u64, out: &mut Queue<Request>) {
        let mut pending = match self.pending.take().expect("matched pending act") {
            Pending::Change(pending) => pending,
            Pending::Policy { .. } => unreachable!("answer matched a sent change"),
        };
        if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Stale(stale),
            ..
        })) = outcome
            && !pending.retried
        {
            pending.retried = true;
            self.send(pending, stale.newest, serial, out);
        } else {
            if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                c::Outcome::SettingsChanged(changed),
            ))) = outcome
            {
                self.toast = Some(Toasted::Way {
                    name: pending.receipt.name,
                    revision: changed.revision,
                    kept: pending.receipt.kept,
                });
            }
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
        self.open = None;
    }
}

fn whole_policy(page: &Readout, setting: c::Setting, against: u32) -> Option<Box<c::SettingPolicy>> {
    let document = match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(document))) => document,
        Readout::Jig(_) | Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit => return None,
    };
    if document.setting != setting || document.revision != against {
        return None;
    }
    match &document.value {
        Some(c::SettingValue::Policy(policy)) => Some(policy.clone()),
        Some(
            c::SettingValue::Charter(_)
            | c::SettingValue::Template(_)
            | c::SettingValue::Rules(_)
            | c::SettingValue::Model(_)
            | c::SettingValue::Application(_),
        )
        | None => None,
    }
}

fn add_delegation(policy: &mut c::SettingPolicy) -> bool {
    let mut changed = add_executors(&mut policy.ceiling.delegation);
    for role in policy.roles.iter_mut() {
        if role.number == 0 {
            changed |= add_executors(&mut role.authority.delegation);
        }
    }
    changed
}

fn add_executors(delegation: &mut c::Delegation) -> bool {
    let mut kinds = List::uncapped();
    for executor in &delegation.kinds {
        kinds.push(*executor).expect("held executors fit");
    }
    let mut changed = false;
    for executor in [
        c::AuthorityExecutor::Charter(5),
        c::AuthorityExecutor::Procedure(6),
        c::AuthorityExecutor::Role(0),
        c::AuthorityExecutor::Job(3),
        c::AuthorityExecutor::Job(4),
    ] {
        if !kinds.as_slice().contains(&executor) {
            kinds.push(executor).expect("five executors join the policy");
            changed = true;
        }
    }
    if changed {
        delegation.kinds = kinds;
    }
    changed
}

fn panel(record: &Family, limits: Limits) -> Option<w::WaysPanel> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    wire::decode_ways_panel(&record.bytes, &wire_limits(&limits)?)
}

fn same_stops(old: &Tab, shown: &Tab) -> bool {
    if old.index != shown.index || old.path.stops().len() != shown.path.stops().len() {
        return false;
    }
    for (old, shown) in old.path.stops().iter().zip(shown.path.stops()) {
        if old.actor() != shown.actor() || old.words() != shown.words() {
            return false;
        }
    }
    true
}
