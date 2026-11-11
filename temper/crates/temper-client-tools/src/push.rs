//! Push subscriptions (temper/docs/design/client/ctl/tools.md, 2, 3).
use crate::{jig::{self, call::{Value, number, value}, render::{self, Text}, runner::{Called, Out}, table::{Keyed, Kind, Occurs::{Optional, Required}, Tool, flag, positional}}, root::Limits};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Ask, Client, Event, Family, Handle, Key, Latest, Outcome, State, Told, Watchable, contract as c};
use skein_lib::{Env, Queue, Reader, base64::{self, Alphabet}};
use temper_client_domain::{forge, owner::Owner};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub const LIST: Tool = Tool {
    name: b"push list", line: b"List your push devices", prints: b"Each device's number, label, added time and push service origin; never its whole endpoint or keys", result: b"devices",
    forms: &[&[]], keyed: Keyed::Never, area: 6, notes: b"Only your own devices are shown.\n",
};
pub const ADD: Tool = Tool {
    name: b"push add", line: b"Register a browser's push subscription", prints: b"`device NUMBER: added`, or a refusal (exit 2)", result: b"device-added",
    forms: &[&[positional(b"file", b"FILE", Kind::File, Required, b"Browser subscription JSON, at most 4096 bytes"), flag(b"label", b"LABEL", Kind::Literal, Optional, b"1 to 60 UTF-8 bytes; temper ctl when absent")]],
    keyed: Keyed::Optional, area: 6, notes: b"The subscription's keys are base64url without padding. Other JSON members are ignored.\n",
};
pub const REMOVE: Tool = Tool {
    name: b"push remove", line: b"Remove one of your push devices", prints: b"`device NUMBER: removed`, or `No device NUMBER.` (exit 2)", result: b"device-removed",
    forms: &[&[positional(b"number", b"NUMBER", Kind::NumberRange { min: 1, max: 4_294_967_295 }, Required, b"Your device's number")]],
    keyed: Keyed::Optional, area: 6, notes: b"Removing a device stops pushes to that subscription.\n",
};
/// Validate the host's subscription file before opening any connection.
#[must_use]
pub fn subscription(bytes: &[u8], label: &[u8]) -> Option<w::AddDevice> {
    if bytes.is_empty() || bytes.len() > 4096 || !valid_label(label) { return None; }
    let doc = jig::json::document_depth(bytes, 4096).ok()?;
    let endpoint = string(&doc, doc.member(0, b"endpoint")?)?;
    let keys = doc.member(0, b"keys")?;
    let p256dh = base64::decode(Alphabet::Url, string(&doc, doc.member(keys, b"p256dh")?)?, 65).ok()?;
    let auth = base64::decode(Alphabet::Url, string(&doc, doc.member(keys, b"auth")?)?, 16).ok()?;
    if endpoint.is_empty() { return None; }
    w::AddDevice::new(&w::CEILINGS, w::AddDeviceParts { endpoint: endpoint.into(), p256dh: p256dh.as_ref().try_into().ok()?, auth: auth.as_ref().try_into().ok()?, label: label.into() }).ok()
}
fn string(doc: &skein_json::Document, at: u32) -> Option<&[u8]> {
    let token = doc.token(at)?;
    if token.kind != skein_json::document::Kind::String { return None; }
    doc.text(token)
}
#[must_use]
pub fn label(call: &jig::call::Call) -> &[u8] {
    match value(call, 1) { Some(Value::Text(label)) => label, _ => b"temper ctl" }
}
#[derive(Debug)]
pub(crate) struct Devices { waiting: BTreeMap<u32, Called>, sent: BTreeMap<Key, Pending>, watching: BTreeMap<Handle, Called>, json: bool }
#[derive(Debug)]
struct Pending { called: Option<Called>, number: u32 }
impl Devices {
    pub(crate) fn new() -> Self { Self { waiting: BTreeMap::new(), sent: BTreeMap::new(), watching: BTreeMap::new(), json: false } }
    pub(crate) fn calls(&self) -> usize { self.waiting.len().saturating_add(self.sent.len()).saturating_add(self.watching.len()) }
    pub(crate) fn kept_json(&mut self, json: bool) { self.json = json; }
}
pub(crate) fn owns(call: &jig::call::Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name;
    name == LIST.name || name == ADD.name || name == REMOVE.name
}
pub(crate) fn call(state: &mut Devices, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        finish(called, render::failure(b"invalid push call", false, 1), out); return;
    }
    state.waiting.insert(called.call_number, called);
    ready(state, client, env, out);
}
pub(crate) fn ready(state: &mut Devices, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    if crate::forge::me(client).is_none() { return; }
    for _ in 0..state.waiting.len() {
        let Some((_, called)) = state.waiting.pop_first() else { break };
        let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
        let name = crate::table::TOOLS.get(usize::from(called.call.tool)).expect("parsed tool").name;
        let serial = (u64::from(called.call_number) << 32_u32) | 0x28;
        if name == LIST.name {
            let handle = Handle { asker: Owner::Forge.asker(), number: serial };
            let watching = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Devices }).expect("devices watch");
            state.watching.insert(handle, called);
            out.push(Out::Client { event: Event::Watch { handle, subject: Watchable::Family { scope: c::Scope::Own, window: c::Window { rows: 0, after: None }, record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) } } } });
        } else {
            let number = if name == REMOVE.name { u32::try_from(number(&called.call, 0).expect("checked number")).expect("checked u32") } else { 0 };
            let request = if name == ADD.name {
                let Some(file) = &called.value else { finish(called, render::failure(b"Not a push subscription's JSON.", false, 2), out); continue; };
                let Some(record) = subscription(file.expose(), label(&called.call)) else { let json = called.json; finish(called, render::failure(b"Not a push subscription's JSON.", json, 2), out); continue; };
                w::Request::AddDevice(record)
            } else { w::Request::RemoveDevice(w::RemoveDevice::new(&bounds, w::RemoveDeviceParts { number }).expect("number")) };
            let record = w::Requesting::new(&bounds, w::RequestingParts { request }).expect("checked subscription");
            let request = Ask::Family { scope: c::Scope::Own, record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) } };
            let key = Key { asker: Owner::Forge.asker(), number: serial };
            let name = match &called.call.key { Some(name) => Some(crate::key::engine(name)), None => None };
            state.sent.insert(key, Pending { called: Some(called), number });
            out.push(Out::Client { event: Event::Send { key, name, request } });
        }
    }
}
pub(crate) fn told(state: &mut Devices, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    match told {
        Told::Kept { key, request: Ask::Family { scope: c::Scope::Own, record } } if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION => {
            let Some(bounds) = forge::wire_limits(&env.limits.domain.forge) else { return; };
            let Some(record) = wire::decode_requesting(&record.bytes, &bounds) else { return; };
            let number = match record.request() {
                w::Request::AddDevice(_) => 0,
                w::Request::RemoveDevice(row) => row.number(),
                w::Request::TryAccount(_) | w::Request::Adopt(_) | w::Request::SetUp(_) | w::Request::Connect(_) | w::Request::Install(_) | w::Request::Reachable(_) | w::Request::Accept(_) | w::Request::RemoveForge(_) | w::Request::StartPlayground | w::Request::EndPlayground | w::Request::Replace(_) | w::Request::Dismiss(_) | w::Request::MakeRepository(_) | w::Request::DismissMaking(_) | w::Request::Looked(_) | w::Request::SetAway(_) | w::Request::Overturn(_) => return,
            };
            state.sent.entry(*key).or_insert(Pending { called: None, number });
        }
        Told::Answered { key, outcome } => {
            let Some(pending) = state.sent.remove(key) else { return; };
            let json = match &pending.called { Some(called) => called.json, None => state.json };
            let printed = answer(outcome, pending.number, json, &env.limits);
            match pending.called { Some(called) => finish(called, printed, out), None => out.push(Out::Kept { line: if printed.status == 0 { printed.stdout } else { printed.stderr } }) }
        }
        Told::Changed { handle } => {
            if !state.watching.contains_key(handle) { return; }
            let Some(document) = client.document(*handle) else { return; };
            if (document.state == State::Opening || document.state == State::Behind) && document.latest.is_none() { return; }
            let called = state.watching.remove(handle).expect("watched devices");
            let printed = match document.latest {
                Some(Latest::Family(record)) if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION => {
                    let bounds = forge::wire_limits(&env.limits.domain.forge).expect("checked limits");
                    match w::Devices::decode(&bounds, &mut Reader::new(&record.bytes)) {
                        Ok(rows) => print_rows(&rows, called.json, &env.limits),
                        Err(_) => render::failure(b"invalid devices document", called.json, 1),
                    }
                }
                _ => render::failure(b"invalid devices document", called.json, 1),
            };
            out.push(Out::Client { event: Event::Unwatch { handle: *handle } });
            finish(called, printed, out);
        }
        Told::Ended { handle, .. } => {
            if let Some(called) = state.watching.remove(handle) { let json = called.json; finish(called, render::failure(b"devices watch ended", json, 2), out); }
        }
        Told::Link { .. } | Told::Dropped { .. } | Told::Kept { .. } | Told::Pending { .. } | Told::Text { .. } | Told::Page { .. } => {}
    }
}
fn answer(outcome: &Outcome, number: u32, json: bool, limits: &Limits) -> jig::Printed {
    let answer = match outcome {
        Outcome::Answered(jig_client::Answered::Family(record)) if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION => {
            let bounds = forge::wire_limits(&limits.domain.forge).expect("checked limits");
            w::Answer::decode(&bounds, &mut Reader::new(&record.bytes)).ok()
        }
        Outcome::Full { held, room_at } => return render::answers_full(crate::root::DIAGNOSTIC_PREFIX, *held, *room_at),
        Outcome::Answered(_) | Outcome::Unknown | Outcome::Held | Outcome::Limit | Outcome::Unconfirmed | Outcome::NotKept | Outcome::Unsent | Outcome::Lost => None,
    };
    let Some(answer) = answer else { return render::failure(b"push request not answered", json, 1); };
    let (number, word) = match answer.outcome() {
        w::Outcome::DeviceAdded(row) => (row.number(), b"added".as_slice()),
        w::Outcome::DeviceRemoved => (number, b"removed".as_slice()),
        w::Outcome::DeviceRefused(row) => {
            let why = match row.reason() {
                w::DeviceRefusal::Person => b"Only a person is pushed to.".as_slice(),
                w::DeviceRefusal::Off => b"This deployment doesn't push.",
                w::DeviceRefusal::Service => b"That push service isn't one this deployment sends to.",
                w::DeviceRefusal::Malformed => b"That subscription's keys aren't a push subscription's.",
                w::DeviceRefusal::TooMany => b"You have 8 devices already: remove one first.",
                w::DeviceRefusal::Unknown => {
                    let mut m = Text::measure(); missing(number, &mut m); let mut text = Text::writing_exact(m); missing(number, &mut text); return render::failure(&text.finish(), json, 2);
                }
            };
            return render::failure(why, json, 2);
        }
        w::Outcome::Adopted(_) | w::Outcome::CoreRefused(_) | w::Outcome::ForgeRefused(_) | w::Outcome::Unknown | w::Outcome::SettingUp | w::Outcome::SetUpRefused(_) | w::Outcome::Running | w::Outcome::StartRefused(_) | w::Outcome::StartFailed(_) | w::Outcome::Connecting | w::Outcome::Link(_) | w::Outcome::ConnectRefused(_) | w::Outcome::Page(_) | w::Outcome::Accepted(_) | w::Outcome::Removed | w::Outcome::RemoveRefused(_) | w::Outcome::Reset | w::Outcome::EndRefused(_) | w::Outcome::Dismissed | w::Outcome::MakeKept | w::Outcome::MakeRefused(_) | w::Outcome::Looked | w::Outcome::Away | w::Outcome::Overturned | w::Outcome::OverturnRefused(_) | w::Outcome::Tried | w::Outcome::NotTheirs => return render::failure(b"unknown push outcome", json, 1),
    };
    let bytes = if json {
        let mut m = skein_json::writer::Encoder::measure(&skein_json::writer::Limits { depth: 2, length: 128 }); encode_added(number, word, &mut m);
        let mut e = skein_json::writer::Encoder::write(m.measured().expect("short device result"), &skein_json::writer::Limits { depth: 2, length: 128 }); encode_added(number, word, &mut e); e.finish()
    } else {
        let mut m = Text::measure(); result(number, word, &mut m); let mut text = Text::writing_exact(m); result(number, word, &mut text); text.finish()
    };
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout: bytes, stderr: Box::default() }
}
fn missing(number: u32, text: &mut Text) { text.put(b"No device "); text.number(u64::from(number)); text.put(b"."); }
fn result(number: u32, word: &[u8], text: &mut Text) { text.put(b"device "); text.number(u64::from(number)); text.put(b": "); text.put(word); text.put(b"\n"); }
fn encode_added(number: u32, word: &[u8], e: &mut skein_json::writer::Encoder) { e.object_start(); e.key(b"number"); e.unsigned(u64::from(number)); e.key(b"outcome"); e.string(word); e.object_end(); }
fn print_rows(rows: &w::Devices, json: bool, limits: &Limits) -> jig::Printed {
    let bytes = if json {
        let bounds = skein_json::writer::Limits { depth: 4, length: limits.tools.text };
        let mut m = skein_json::writer::Encoder::measure(&bounds); encode_rows(rows, &mut m);
        let Ok(size) = m.measured() else { return render::failure(b"device result limit", json, 1); };
        let mut e = skein_json::writer::Encoder::write(size, &bounds); encode_rows(rows, &mut e); e.finish()
    } else { let mut m = Text::measure(); rows_text(rows, &mut m); let mut text = Text::writing_exact(m); rows_text(rows, &mut text); text.finish() };
    jig::Printed { file: jig::FileEnd::Keep, status: 0, stdout: bytes, stderr: Box::default() }
}
fn encode_rows(rows: &w::Devices, e: &mut skein_json::writer::Encoder) {
    e.object_start(); e.key(b"devices"); e.array_start();
    for row in rows.rows() {
        e.object_start(); e.key(b"number"); e.unsigned(u64::from(row.number())); e.key(b"label"); e.string(row.label()); e.key(b"at"); e.unsigned(row.at());
        e.key(b"service"); match skein_push::Endpoint::parse(row.endpoint()) { Some(endpoint) => e.string(endpoint.origin()), None => e.null() } e.object_end();
    }
    e.array_end(); e.object_end();
}
fn rows_text(rows: &w::Devices, text: &mut Text) {
    for row in rows.rows() {
        text.put(b"device "); text.number(u64::from(row.number())); text.put(b": "); text.put(row.label()); text.put(b" "); text.number(row.at()); text.put(b" ");
        if let Some(endpoint) = skein_push::Endpoint::parse(row.endpoint()) { text.put(endpoint.origin()); }
        text.put(b"\n");
    }
}
fn finish(called: Called, printed: jig::Printed, out: &mut Queue<Out>) { out.push(Out::Printed { number: called.number, printed }); }

/// Label validation at the ctl boundary (attention.md, 7.1).
#[must_use]
#[expect(clippy::disallowed_methods, reason = "ctl validates the device label before sending it")]
pub fn valid_label(label: &[u8]) -> bool { !label.is_empty() && label.len() <= 60 && core::str::from_utf8(label).is_ok() }
