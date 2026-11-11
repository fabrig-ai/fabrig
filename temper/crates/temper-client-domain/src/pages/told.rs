//! Telling you (temper/docs/design/client/pages/told.md; client/web/README.md, 2.7).
use super::Request;
use crate::{forge::{devices, Limits, wire_limits}, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Key, Latest, Outcome, Told as Tell};
use skein_lib::{Queue, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission { Ask, Granted, Denied }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Browser {
    pub capable: bool,
    pub permission: Permission,
    pub endpoint: Option<Box<[u8]>>,
    pub user_agent: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subscription {
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duty {
    Subscribe { token: u64, key: [u8; 65] },
    Drop { token: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Report {
    State(Browser),
    Subscribed { token: u64, subscription: Subscription },
    Refused { token: u64, why: Refusal },
    Dropped { token: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal { Off, Service, TooMany, Malformed, Person, Blocked, Failed, Unknown }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unknown,
    NotSetUp,
    Cannot { home_screen: bool },
    Blocked,
    Off,
    TurningOn,
    On { number: u32, at: Wall },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act { TurnOn, TurnOff, Remove { number: u32 } }
#[derive(Debug)]
enum Pending { Subscribe(u64), Add(Key), Remove { key: Key, drop: bool } }
#[derive(Debug)]
pub struct Told {
    browser: Option<Browser>,
    rows: Option<Box<[devices::Row]>>,
    handle: Option<Handle>,
    party: Option<u64>,
    pending: Option<Pending>,
    refusal: Option<Refusal>,
    dropping: Option<u64>,
    ended: bool,
}
impl Told {
    pub(super) const fn new() -> Self {
        Self { browser: None, rows: None, handle: None, party: None, pending: None, refusal: None, dropping: None, ended: false }
    }
    #[must_use]
    pub fn rows(&self) -> Option<&[devices::Row]> { self.rows.as_deref() }
    #[must_use]
    pub const fn browser(&self) -> Option<&Browser> { self.browser.as_ref() }
    #[must_use]
    pub const fn refusal(&self) -> Option<Refusal> { self.refusal }
    #[must_use]
    #[expect(clippy::manual_find, reason = "step code searches devices without a callback")]
    pub fn current(&self) -> Option<&devices::Row> {
        let endpoint = self.browser.as_ref()?.endpoint.as_ref()?;
        for row in self.rows.as_deref()? {
            if row.endpoint == *endpoint { return Some(row); }
        }
        None
    }
    #[must_use]
    pub fn state(&self, push: Option<Option<[u8; 65]>>) -> State {
        match push { None => return State::Unknown, Some(None) => return State::NotSetUp, Some(Some(_)) => {} }
        if self.refusal == Some(Refusal::Off) { return State::NotSetUp; }
        let Some(browser) = &self.browser else { return State::Unknown };
        if !browser.capable { return State::Cannot { home_screen: home_screen(&browser.user_agent) }; }
        if browser.permission == Permission::Denied || self.refusal == Some(Refusal::Blocked) { return State::Blocked; }
        if let Some(Pending::Subscribe(_) | Pending::Add(_)) = self.pending { return State::TurningOn; }
        if self.rows.is_none() { return State::Unknown; }
        match self.current() { Some(row) => State::On { number: row.number, at: row.at }, None => State::Off }
    }
    pub(super) fn configure(&mut self, open: bool, party: Option<u64>, next: &mut u64, limits: Limits, out: &mut Queue<Request>) {
        if self.party != party {
            self.close(out);
            self.party = party;
            self.rows = None;
            self.pending = None;
            self.refusal = None;
            self.dropping = None;
            self.ended = false;
        }
        if !open { self.close(out); self.ended = false; return; }
        if party.is_some() && !self.ended && self.handle.is_none() && let Some(subject) = devices::watch(limits) {
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            self.handle = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
        }
    }
    pub(super) fn close(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }
    pub(super) fn report(&mut self, report: Report, serial: &mut u64, limits: Limits, out: &mut Queue<Request>) {
        match report {
            Report::State(browser) => self.browser = Some(browser),
            Report::Subscribed { token, subscription } => {
                if !self.subscribing(token) { return; }
                self.pending = None;
                let Some(browser) = &mut self.browser else { return };
                browser.endpoint = Some(subscription.endpoint.clone());
                browser.permission = Permission::Granted;
                let Some(bounds) = wire_limits(&limits) else { return };
                let parts = w::AddDeviceParts { endpoint: subscription.endpoint, p256dh: subscription.p256dh, auth: subscription.auth, label: label(&browser.user_agent) };
                let Ok(value) = w::AddDevice::new(&bounds, parts) else { self.refusal = Some(Refusal::Malformed); return };
                if let Some(key) = send(w::Request::AddDevice(value), serial, limits, out) { self.pending = Some(Pending::Add(key)); }
            }
            Report::Refused { token, why } => {
                if self.subscribing(token) {
                    self.pending = None;
                    self.refusal = Some(why);
                } else if self.dropping == Some(token) {
                    self.dropping = None;
                    self.refusal = Some(why);
                }
            }
            Report::Dropped { token } => {
                if self.dropping == Some(token) {
                    self.dropping = None;
                    if let Some(browser) = &mut self.browser { browser.endpoint = None; }
                }
            }
        }
    }
    fn subscribing(&self, token: u64) -> bool {
        match self.pending { Some(Pending::Subscribe(pending)) => pending == token, Some(Pending::Add(_) | Pending::Remove { .. }) | None => false }
    }
    pub(super) fn act(&mut self, act: Act, push: Option<[u8; 65]>, serial: &mut u64, limits: Limits, out: &mut Queue<Request>) {
        if self.pending.is_some() || self.dropping.is_some() || self.party.is_none() { return; }
        self.refusal = None;
        let (number, drop) = match act {
            Act::TurnOn => {
                let Some(key) = push else { return };
                if self.state(Some(Some(key))) != State::Off { return; }
                *serial = serial.checked_add(1).expect("push tokens fit");
                self.pending = Some(Pending::Subscribe(*serial));
                out.push(Request::Push(Duty::Subscribe { token: *serial, key }));
                return;
            }
            Act::TurnOff => { let Some(row) = self.current() else { return }; (row.number, true) }
            Act::Remove { number } => {
                let Some(rows) = &self.rows else { return };
                let mut found = false;
                for row in rows { found |= row.number == number; }
                if !found { return; }
                (number, false)
            }
        };
        let Some(bounds) = wire_limits(&limits) else { return };
        let Ok(value) = w::RemoveDevice::new(&bounds, w::RemoveDeviceParts { number }) else { return };
        if let Some(key) = send(w::Request::RemoveDevice(value), serial, limits, out) { self.pending = Some(Pending::Remove { key, drop }); }
    }
    pub(super) fn told(&mut self, client: &Client, told: &Tell, limits: Limits, out: &mut Queue<Request>) -> bool {
        match told {
            Tell::Changed { handle } if self.handle == Some(*handle) => {
                if let Some(document) = client.document(*handle) && let Some(Latest::Family(record)) = document.latest && let Some(rows) = devices::rows(record, limits) { self.rows = Some(rows); }
                return true;
            }
            Tell::Ended { handle, .. } if self.handle == Some(*handle) => { self.close(out); self.ended = true; return true; }
            Tell::Answered { key, outcome } => {
                let (pending, drop) = match self.pending { Some(Pending::Add(key)) => (key, false), Some(Pending::Remove { key, drop }) => (key, drop), Some(Pending::Subscribe(_)) | None => return false };
                if pending != *key { return false; }
                self.pending = None;
                let Some(answer) = answer(outcome, limits) else { self.refusal = Some(Refusal::Failed); return true };
                match answer {
                    Answer::Added | Answer::Unknown => {}
                    Answer::Removed => if drop { self.dropping = Some(key.number); out.push(Request::Push(Duty::Drop { token: key.number })); },
                    Answer::Refused(why) => self.refusal = Some(why),
                }
                return true;
            }
            Tell::Link { .. } | Tell::Changed { .. } | Tell::Ended { .. } | Tell::Page { .. } | Tell::Text { .. } | Tell::Pending { .. } | Tell::Kept { .. } | Tell::Dropped { .. } => {}
        }
        false
    }
}
fn send(request: w::Request, serial: &mut u64, limits: Limits, out: &mut Queue<Request>) -> Option<Key> {
    let record = w::Requesting::new(&wire_limits(&limits)?, w::RequestingParts { request }).ok()?;
    *serial = serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: *serial };
    out.push(Request::Client(jig_client::Event::Send { key, name: None, request: jig_client::Ask::Family { scope: jig_client::contract::Scope::Own, record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) } } }));
    Some(key)
}
enum Answer { Added, Removed, Refused(Refusal), Unknown }
fn answer(outcome: &Outcome, limits: Limits) -> Option<Answer> {
    if let Outcome::Unknown = outcome { return Some(Answer::Unknown); }
    let Outcome::Answered(jig_client::Answered::Family(record)) = outcome else { return None };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION { return None; }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let outcome = w::Answer::decode(&wire_limits(&limits)?, &mut reader).ok()?.into_parts().outcome;
    if !reader.is_empty() { return None; }
    match outcome {
        w::Outcome::DeviceAdded(_) => Some(Answer::Added),
        w::Outcome::DeviceRemoved => Some(Answer::Removed),
        w::Outcome::DeviceRefused(value) => Some(Answer::Refused(match value.reason() { w::DeviceRefusal::Person => Refusal::Person, w::DeviceRefusal::Off => Refusal::Off, w::DeviceRefusal::Service => Refusal::Service, w::DeviceRefusal::Malformed => Refusal::Malformed, w::DeviceRefusal::TooMany => Refusal::TooMany, w::DeviceRefusal::Unknown => Refusal::Unknown })),
        w::Outcome::Unknown => Some(Answer::Unknown),
        w::Outcome::Looked | w::Outcome::Page(_) | w::Outcome::Accepted(_) | w::Outcome::Connecting | w::Outcome::Link(_) | w::Outcome::ConnectRefused(_) | w::Outcome::Adopted(_) | w::Outcome::CoreRefused(_) | w::Outcome::ForgeRefused(_) | w::Outcome::Running | w::Outcome::StartRefused(_) | w::Outcome::StartFailed(_) | w::Outcome::SettingUp | w::Outcome::SetUpRefused(_) | w::Outcome::Removed | w::Outcome::RemoveRefused(_) | w::Outcome::Reset | w::Outcome::Dismissed | w::Outcome::Tried | w::Outcome::NotTheirs | w::Outcome::Overturned | w::Outcome::OverturnRefused(_) | w::Outcome::EndRefused(_) | w::Outcome::MakeKept | w::Outcome::MakeRefused(_) | w::Outcome::Away => None,
    }
}
fn contains(ua: &[u8], part: &[u8]) -> bool {
    for window in ua.windows(part.len()) { if window == part { return true; } }
    false
}
#[must_use]
pub fn home_screen(ua: &[u8]) -> bool { contains(ua, b"iPhone") || contains(ua, b"iPad") }
/// BROWSER on SYSTEM, or This browser where either is unknown (client/pages/told.md).
#[must_use]
pub fn label(ua: &[u8]) -> Box<[u8]> {
    let browser: &[u8] = if contains(ua, b"Edg/") || contains(ua, b"EdgiOS/") || contains(ua, b"EdgA/") { b"Edge" }
    else if contains(ua, b"Firefox/") || contains(ua, b"FxiOS/") { b"Firefox" }
    else if contains(ua, b"Chrome/") || contains(ua, b"CriOS/") { b"Chrome" }
    else if contains(ua, b"Safari/") { b"Safari" } else { return b"This browser".as_slice().into() };
    let system: &[u8] = if contains(ua, b"iPhone") { b"iPhone" } else if contains(ua, b"iPad") { b"iPad" }
    else if contains(ua, b"Android") { b"Android" } else if contains(ua, b"Macintosh") || contains(ua, b"Mac OS") { b"Mac" }
    else if contains(ua, b"Windows") { b"Windows" } else if contains(ua, b"Linux") { b"Linux" } else { return b"This browser".as_slice().into() };
    let mut words = skein_lib::List::uncapped();
    for part in [browser, b" on ".as_slice(), system] {
        for byte in part { words.push(*byte).expect("uncapped label"); }
    }
    words.into_boxed()
}

#[cfg(test)]
#[path = "../tests/pages/told.rs"]
mod tests;
