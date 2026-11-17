//! Workers as settings (temper/docs/design/client/pages/workers.md, 1).
use super::{Request, accounts};
use crate::owner::Owner;
use alloc::boxed::Box;
use jig_client::{Client, Handle, Key, Latest, Outcome, Readable, Readout, State, Told, contract as c};
use skein_lib::{Decimal, List, Queue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Places { number: u32, value: Box<[u8]> },
    Change(u32),
    Remove(u32),
    Name(Box<[u8]>),
    AddPlaces(Box<[u8]>),
    Digest(Box<[u8]>),
    Add,
}
#[derive(Debug)]
enum Pending {
    Reading { handle: Handle, item: c::SettingItem, row: Option<u32> },
    Sending { key: Key, row: Option<u32> },
}
impl Pending {
    fn row(&self) -> Option<u32> {
        match self {
            Self::Reading { row, .. } | Self::Sending { row, .. } => *row,
        }
    }
}
#[derive(Debug)]
pub struct Builders {
    handle: Option<Handle>,
    places: List<(u32, Box<[u8]>)>,
    name: Box<[u8]>,
    add_places: Box<[u8]>,
    digest: Box<[u8]>,
    pending: Option<Pending>,
    refused: Option<(Option<u32>, Outcome)>,
}
#[derive(Debug)]
pub enum Read<'a> {
    NoRole,
    Unknown,
    Hosts(&'a c::Deployment),
}
impl Default for Builders {
    fn default() -> Self {
        Self {
            handle: None,
            places: List::uncapped(),
            name: Box::default(),
            add_places: b"4".as_slice().into(),
            digest: Box::default(),
            pending: None,
            refused: None,
        }
    }
}
impl Builders {
    pub(crate) fn configure(&mut self, role: bool, next: &mut u64, out: &mut Queue<Request>) {
        if role && self.handle.is_none() {
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            self.handle = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: jig_client::Watchable::Jig(c::Subject::Deployment),
            }));
        } else if !role {
            self.leave(out);
        }
    }
    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        if let Some(Pending::Reading { handle, .. }) = self.pending.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        self.places.clear();
        self.name = Box::default();
        self.add_places = b"4".as_slice().into();
        self.digest = Box::default();
        self.refused = None;
    }
    pub(super) fn watches(&self, handle: Handle) -> bool {
        self.handle == Some(handle)
    }
    #[must_use]
    pub fn read<'a>(&self, client: &'a Client) -> Read<'a> {
        let Some(me) = accounts::current_me(client) else { return Read::Unknown };
        if me.deployment_role.is_none() {
            return Read::NoRole;
        }
        let Some(handle) = self.handle else { return Read::Unknown };
        let Some(doc) = client.document(handle) else { return Read::Unknown };
        if doc.state != State::Live {
            return Read::Unknown;
        }
        if let Some(Latest::Jig(c::Delivery::Deployment(deployment))) = doc.latest {
            Read::Hosts(deployment)
        } else {
            Read::Unknown
        }
    }
    #[must_use]
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }
    #[must_use]
    pub fn pending(&self, row: Option<u32>) -> bool {
        match &self.pending {
            Some(pending) => pending.row() == row,
            None => false,
        }
    }
    #[must_use]
    pub fn refused(&self, row: Option<u32>) -> Option<&Outcome> {
        let (at, outcome) = self.refused.as_ref()?;
        if *at == row { Some(outcome) } else { None }
    }
    #[must_use]
    pub fn places(&self, worker: &c::WorkerState) -> Box<[u8]> {
        for (number, value) in &self.places {
            if *number == worker.number {
                return value.clone();
            }
        }
        Decimal::of(u64::from(worker.places)).as_bytes().into()
    }
    #[must_use]
    pub fn can_change(&self, worker: &c::WorkerState) -> bool {
        !self.busy()
            && match count(&self.places(worker)) {
                Some(places) => places != worker.places,
                None => false,
            }
    }
    #[must_use]
    pub fn name(&self) -> &[u8] {
        &self.name
    }
    #[must_use]
    pub fn add_places(&self) -> &[u8] {
        &self.add_places
    }
    #[must_use]
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
    #[must_use]
    pub fn duplicate(&self, deployment: &c::Deployment) -> bool {
        match c::worker_number(deployment.worker_settings.as_slice(), &self.name) {
            c::WorkerNumber::Live(_) => true,
            c::WorkerNumber::Withdrawn(_) | c::WorkerNumber::Free(_) => false,
        }
    }
    #[must_use]
    pub fn can_add(&self, deployment: &c::Deployment) -> bool {
        !self.busy()
            && name_shape(&self.name)
            && count(&self.add_places).is_some()
            && digest(&self.digest).is_some()
            && !self.duplicate(deployment)
    }
    pub(super) fn act(&mut self, client: &Client, act: Act, next: &mut u64, out: &mut Queue<Request>) -> bool {
        let Some(me) = accounts::current_me(client) else { return false };
        if me.deployment_role != Some(c::HeldRole::Owner) || self.busy() {
            return false;
        }
        let Read::Hosts(deployment) = self.read(client) else { return false };
        let (item, row) = match act {
            Act::Name(value) => {
                self.name = super::composer::held(value, c::PROGRAM_HOST_NAME.saturating_add(1)).0;
                return true;
            }
            Act::AddPlaces(value) => {
                self.add_places = super::composer::held(value, 11).0;
                return true;
            }
            Act::Digest(value) => {
                self.digest = super::composer::held(value, 65).0;
                return true;
            }
            Act::Places { number, value } => {
                if worker(deployment, number).is_none() {
                    return false;
                }
                let value = super::composer::held(value, 11).0;
                for (at, old) in self.places.iter_mut() {
                    if *at == number {
                        *old = value;
                        return true;
                    }
                }
                self.places.push((number, value)).expect("one field per worker");
                return true;
            }
            Act::Change(number) => {
                let Some(worker) = worker(deployment, number) else { return false };
                if !self.can_change(worker) {
                    return false;
                }
                let value = c::WorkerSetting {
                    number,
                    name: worker.name.clone(),
                    places: count(&self.places(worker)).expect("checked count"),
                    digest: worker.digest,
                    withdrawn: false,
                };
                (set(value), Some(number))
            }
            Act::Remove(number) => {
                let Some(worker) = worker(deployment, number) else { return false };
                if worker.digest.is_none() {
                    return false;
                }
                (
                    c::SettingItem {
                        setting: c::Setting::Worker(c::SettingNumber { number }),
                        edit: c::SettingEdit::Removed,
                    },
                    Some(number),
                )
            }
            Act::Add => {
                if !self.can_add(deployment) {
                    return false;
                }
                let number = match c::worker_number(deployment.worker_settings.as_slice(), &self.name) {
                    c::WorkerNumber::Live(_) => return false,
                    c::WorkerNumber::Withdrawn(number) | c::WorkerNumber::Free(number) => number,
                };
                (
                    set(c::WorkerSetting {
                        number,
                        name: self.name.clone(),
                        places: count(&self.add_places).expect("checked count"),
                        digest: digest(&self.digest),
                        withdrawn: false,
                    }),
                    None,
                )
            }
        };
        let handle = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        self.pending = Some(Pending::Reading { handle, item, row });
        self.refused = None;
        out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Settings) }));
        true
    }
    pub(super) fn told(&mut self, client: &Client, told: &Told, serial: &mut u64, out: &mut Queue<Request>) -> bool {
        match (&self.pending, told) {
            (Some(Pending::Reading { handle, .. }), Told::Page { handle: at, page }) if handle == at => {
                let Some(Pending::Reading { item, row, .. }) = self.pending.take() else {
                    unreachable!("matched read")
                };
                let owner = match accounts::current_me(client) {
                    Some(me) => me.deployment_role == Some(c::HeldRole::Owner),
                    None => false,
                };
                if !owner {
                    return true;
                }
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(settings))) = page {
                    *serial = serial.checked_add(1).expect("page keys fit");
                    let key = Key { asker: Owner::Pages.asker(), number: *serial };
                    out.push(Request::Client(jig_client::Event::Send {
                        key,
                        name: None,
                        request: jig_client::Ask::Jig(Box::new(c::Ask::ChangeSettings(c::ChangeSettings {
                            against: settings.newest,
                            items: List::from_boxed(Box::new([item])),
                            running: c::Running::Stays,
                        }))),
                    }));
                    self.pending = Some(Pending::Sending { key, row });
                } else {
                    self.refused = Some((row, Outcome::Unknown));
                }
                true
            }
            (Some(Pending::Sending { key, row }), Told::Answered { key: at, outcome }) if key == at => {
                let row = *row;
                self.answered(row, outcome);
                true
            }
            (Some(Pending::Sending { key, row }), Told::Pending { key: at }) if key == at => {
                let unknown = match client.pending(*key) {
                    Some(request) => request.progress == jig_client::Progress::Unknown,
                    None => false,
                };
                if unknown {
                    let row = *row;
                    self.answered(row, &Outcome::Unknown);
                    true
                } else {
                    false
                }
            }
            (
                Some(Pending::Reading { .. } | Pending::Sending { .. }) | None,
                Told::Page { .. }
                | Told::Answered { .. }
                | Told::Pending { .. }
                | Told::Changed { .. }
                | Told::Ended { .. }
                | Told::Link { .. }
                | Told::Kept { .. }
                | Told::Dropped { .. }
                | Told::Text { .. },
            ) => false,
        }
    }
    fn answered(&mut self, row: Option<u32>, outcome: &Outcome) {
        if super::seen_answered(outcome) {
            if let Some(number) = row {
                let mut index = None;
                for (i, (at, _)) in self.places.iter().enumerate() {
                    if *at == number {
                        index = Some(u32::try_from(i).expect("worker fields fit"));
                    }
                }
                if let Some(index) = index {
                    self.places.swap_remove(index);
                }
            } else {
                self.name = Box::default();
                self.add_places = Box::default();
                self.digest = Box::default();
            }
        } else {
            self.refused = Some((row, outcome.clone()));
        }
        self.pending = None;
    }
}
#[expect(clippy::manual_find, reason = "step crates use explicit traversal, not predicate closures")]
fn worker(deployment: &c::Deployment, number: u32) -> Option<&c::WorkerState> {
    for worker in &deployment.worker_settings {
        if worker.number == number && !worker.withdrawn {
            return Some(worker);
        }
    }
    None
}
fn set(value: c::WorkerSetting) -> c::SettingItem {
    c::SettingItem {
        setting: c::Setting::Worker(c::SettingNumber { number: value.number }),
        edit: c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Worker(Box::new(value)) }),
    }
}
fn count(value: &[u8]) -> Option<u32> {
    if value.is_empty() {
        return None;
    }
    let mut count = 0_u32;
    for byte in value {
        if !byte.is_ascii_digit() {
            return None;
        }
        count = count.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
    }
    (count != 0).then_some(count)
}
fn name_shape(name: &[u8]) -> bool {
    let Some(first) = name.first() else { return false };
    if !first.is_ascii_alphanumeric() || name.len() > skein_lib::usize_of(c::PROGRAM_HOST_NAME) {
        return false;
    }
    for byte in name {
        if !(byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_' || *byte == b'.') {
            return false;
        }
    }
    true
}
fn digest(value: &[u8]) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut bytes = [0; 32];
    for (out, pair) in bytes.iter_mut().zip(value.as_chunks::<2>().0) {
        let mut byte = 0_u8;
        for digit in pair {
            let digit = match digit {
                b'0'..=b'9' => digit.checked_sub(b'0')?,
                b'a'..=b'f' => digit.checked_sub(b'a')?.checked_add(10)?,
                _ => return None,
            };
            byte = byte.checked_mul(16)?.checked_add(digit)?;
        }
        *out = byte;
    }
    Some(bytes)
}
