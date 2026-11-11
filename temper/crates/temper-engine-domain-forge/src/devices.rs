//! A person's push devices (temper/docs/design/domain/forge/attention.md, 7.1).
use crate::{Answered, Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Token, Wall};

pub const DEVICES_SUBJECT: u64 = 1 << 59;
pub const DEVICES: u32 = 8;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Device {
    pub party: u64,
    pub number: u32,
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
    pub label: Box<[u8]>,
    pub at: Wall,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AddDevice {
    pub endpoint: Box<[u8]>,
    pub p256dh: [u8; 65],
    pub auth: [u8; 16],
    pub label: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceRefusal { Person, Off, Service, Malformed, TooMany, Unknown }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeviceAnswer { Added { number: u32 }, Removed, Refused(DeviceRefusal) }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DeviceView {
    pub number: u32,
    pub label: Box<[u8]>,
    pub at: Wall,
    pub endpoint: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DevicesView { pub rows: Box<[DeviceView]> }

pub(crate) fn add(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, party: u64, call: AddDevice, out: &mut Queue<Request>) {
    let answer = add_row(d, env.wall, party, call, out);
    d.connect.answer(request, token, Answered::Device(answer), out);
}
#[expect(clippy::disallowed_methods, reason = "the connector judges the device label as UTF-8 (attention.md, 7.1)")]
fn add_row(d: &mut Domain, wall: Wall, party: u64, call: AddDevice, out: &mut Queue<Request>) -> DeviceAnswer {
    let Some(services) = &d.push else { return DeviceAnswer::Refused(DeviceRefusal::Off) };
    let Some(endpoint) = skein_push::Endpoint::parse(&call.endpoint) else { return DeviceAnswer::Refused(DeviceRefusal::Service); };
    let mut allowed = false;
    for service in services {
        if service.as_ref() == endpoint.origin() { allowed = true; break; }
    }
    if !allowed { return DeviceAnswer::Refused(DeviceRefusal::Service); }
    if call.p256dh.first() != Some(&4) || call.label.is_empty() || call.label.len() > 60 || core::str::from_utf8(&call.label).is_err() {
        return DeviceAnswer::Refused(DeviceRefusal::Malformed);
    }
    let mut count = 0_u32;
    let mut highest = 0_u32;
    let mut own = None;
    for ((_, number), row) in d.devices_of(party) {
        count = count.saturating_add(1);
        highest = *number;
        if row.endpoint == call.endpoint { own = Some(row); }
    }
    if let Some(row) = own {
        let number = row.number;
        if row.p256dh == call.p256dh && row.auth == call.auth && row.label == call.label { return DeviceAnswer::Added { number }; }
        save(d, Device { party, number, endpoint: call.endpoint, p256dh: call.p256dh, auth: call.auth, label: call.label, at: wall }, out);
        return DeviceAnswer::Added { number };
    }
    // A refused decision changes nothing, including another party's row.
    let mut previous = None;
    for (key, row) in &d.devices {
        if row.endpoint == call.endpoint { previous = Some(*key); break; }
    }
    if count >= DEVICES || (previous.is_none() && d.devices.len() == d.devices.capacity()) { return DeviceAnswer::Refused(DeviceRefusal::TooMany); }
    let Some(number) = highest.checked_add(1) else { return DeviceAnswer::Refused(DeviceRefusal::TooMany) };
    if let Some((party, number)) = previous {
        d.devices.remove(&(party, number));
        out.push(Request::Erase { key: Key::Device { party, number } });
    }
    save(d, Device { party, number, endpoint: call.endpoint, p256dh: call.p256dh, auth: call.auth, label: call.label, at: wall }, out);
    DeviceAnswer::Added { number }
}
fn save(d: &mut Domain, row: Device, out: &mut Queue<Request>) {
    d.devices.insert((row.party, row.number), row.clone()).expect("device room checked");
    out.push(Request::Save { record: Stored::Device(row), release: None, read_afresh: None });
    out.push(Request::Marked { tasks: Box::new([DEVICES_SUBJECT]) });
}
pub(crate) fn remove(d: &mut Domain, request: Token, token: Token, party: u64, number: u32, out: &mut Queue<Request>) {
    let answer = if d.devices.remove(&(party, number)).is_some() {
        out.push(Request::Erase { key: Key::Device { party, number } });
        out.push(Request::Marked { tasks: Box::new([DEVICES_SUBJECT]) });
        DeviceAnswer::Removed
    } else { DeviceAnswer::Refused(DeviceRefusal::Unknown) };
    d.connect.answer(request, token, Answered::Device(answer), out);
}
impl Domain {
    /// This party's subscriptions in number order, including keys for the push decision.
    pub fn devices_of(&self, party: u64) -> impl Iterator<Item = (&(u64, u32), &Device)> {
        self.devices.range((party, 0)..=(party, u32::MAX))
    }
    pub(crate) fn devices_view(&self, party: u64) -> DevicesView {
        let mut rows = List::with_capacity(DEVICES);
        for (_, row) in self.devices_of(party) {
            rows.push(DeviceView { number: row.number, label: row.label.clone(), at: row.at, endpoint: row.endpoint.clone() }).expect("bounded party devices");
        }
        DevicesView { rows: rows.into_boxed() }
    }
}
