//! Conditional answers, bounded oldest out (skein/docs/design/forge/connection.md, 4.6).
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api::Validator;
use skein_http::client as http;
use skein_lib::{List, Secret, len_u64};

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Identity {
    Token(Secret),
    Installation(u64),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Key {
    pub forge: u16,
    pub origin: Option<u32>,
    pub identity: Identity,
    pub target: Box<[u8]>,
    pub range: Option<Box<[u8]>>,
}
#[derive(Debug)]
pub(crate) struct Validators {
    etag: Option<Validator>,
    modified: Option<Validator>,
}
impl Validators {
    pub fn of(head: &http::Response) -> Option<Self> {
        let etag = validator(head.header(b"ETag"));
        let modified = validator(head.header(b"Last-Modified"));
        if etag.is_none() && modified.is_none() { None } else { Some(Self { etag, modified }) }
    }
    pub fn headers(&self, headers: &mut List<skein_http::Header>) {
        for (name, value) in [(b"If-None-Match".as_slice(), &self.etag), (b"If-Modified-Since", &self.modified)] {
            if let Some(value) = value {
                headers.push(crate::connection::field(name, value.as_bytes())).expect("conditional header room");
            }
        }
    }
    fn bytes(&self) -> u64 {
        let mut bytes = 0_u64;
        for validator in [&self.etag, &self.modified].into_iter().flatten() {
            bytes = bytes.saturating_add(len_u64(validator.as_bytes().len()));
        }
        bytes
    }
}
fn validator(value: Option<&[u8]>) -> Option<Validator> {
    Validator::new(value?)
}

#[derive(Debug)]
struct Entry {
    id: u64,
    used: u64,
    pins: u32,
    current: bool,
    key: Key,
    validators: Validators,
    head: http::Response,
    body: Box<[u8]>,
}
impl Entry {
    fn bytes(&self) -> u64 {
        Self::overhead(&self.key, &self.head, &self.validators).saturating_add(len_u64(self.body.len()))
    }
    fn overhead(key: &Key, head: &http::Response, validators: &Validators) -> u64 {
        let mut bytes = len_u64(size_of::<Self>()).saturating_add(len_u64(key.target.len()));
        match &key.identity {
            Identity::Token(token) => bytes = bytes.saturating_add(len_u64(token.len())),
            Identity::Installation(_) => {}
        }
        if let Some(range) = &key.range {
            bytes = bytes.saturating_add(len_u64(range.len()));
        }
        for field in &head.headers {
            bytes = bytes
                .saturating_add(len_u64(size_of::<skein_http::Header>()))
                .saturating_add(len_u64(field.name.len()))
                .saturating_add(len_u64(field.value.len()));
        }
        bytes.saturating_add(validators.bytes())
    }
}
/// Entries have exact-length storage; a pinned superseded answer lives until its last reader ends.
#[derive(Debug)]
pub(crate) struct Store {
    entries: Box<[Entry]>,
    bytes: u64,
    reserved: u64,
    limit: u64,
    clock: u64,
}
impl Store {
    pub fn new(limit: u64) -> Self {
        Self { entries: Box::from([]), bytes: 0, reserved: 0, limit, clock: 0 }
    }
    pub fn pin(&mut self, key: &Key, headers: &mut List<skein_http::Header>) -> Option<u64> {
        self.clock = self.clock.checked_add(1).expect("conditional read sequence");
        for entry in &mut self.entries {
            if entry.current && entry.key == *key {
                entry.used = self.clock;
                entry.pins = entry.pins.checked_add(1).expect("bounded concurrent requests");
                entry.validators.headers(headers);
                return Some(entry.id);
            }
        }
        None
    }
    pub fn reset_active(&mut self) {
        self.reserved = 0;
        for entry in &mut self.entries {
            entry.pins = 0;
        }
    }
    pub fn repin(&mut self, id: u64) {
        for entry in &mut self.entries {
            if entry.id == id {
                entry.pins = entry.pins.checked_add(1).expect("bounded concurrent requests");
            }
        }
    }
    pub fn release(&mut self, id: u64) {
        for entry in &mut self.entries {
            if entry.id == id {
                entry.pins = entry.pins.saturating_sub(1);
            }
        }
        self.prune();
    }
    pub fn prune(&mut self) {
        let mut expired = false;
        for entry in &self.entries {
            if !entry.current && entry.pins == 0 {
                expired = true;
                break;
            }
        }
        if !expired {
            return;
        }
        let entries = List::from_boxed(core::mem::replace(&mut self.entries, Box::from([])));
        let mut kept = List::with_capacity(entries.len());
        for entry in entries.into_boxed() {
            if entry.current || entry.pins > 0 {
                kept.push(entry).expect("no more entries than before");
            } else {
                self.bytes = self.bytes.checked_sub(entry.bytes()).expect("accounted entry");
            }
        }
        self.entries = kept.into_boxed();
    }
    pub fn forget(&mut self, forge: u16) {
        for entry in &mut self.entries {
            if entry.key.forge == forge {
                entry.current = false;
                entry.pins = 0;
            }
        }
        self.prune();
    }
    pub fn drop_key(&mut self, key: &Key) {
        for entry in &mut self.entries {
            if entry.key == *key {
                entry.current = false;
            }
        }
        self.prune();
    }
    pub fn body(&self, id: u64) -> Option<&[u8]> {
        for entry in &self.entries {
            if entry.id == id {
                return Some(&entry.body);
            }
        }
        None
    }
    pub fn replay(&self, id: u64, fresh: &http::Response) -> Option<http::Response> {
        for entry in &self.entries {
            if entry.id != id {
                continue;
            }
            let mut fields = List::with_capacity(
                u32::try_from(entry.head.headers.len()).ok()?.checked_add(u32::try_from(fresh.headers.len()).ok()?)?,
            );
            // Date and rate belong to the 304, even when it omits them.
            for field in &entry.head.headers {
                if !fresh_field(&field.name) && !field.is(b"Connection") {
                    fields.push(field.clone()).expect("kept head fields fit");
                }
            }
            for field in &fresh.headers {
                if fresh_field(&field.name) || field.is(b"Connection") {
                    fields.push(field.clone()).expect("fresh head fields fit");
                }
            }
            return Some(http::Response {
                version: entry.head.version,
                status: entry.head.status,
                headers: fields.into_boxed(),
                framing: http::Framing::Length(len_u64(entry.body.len())),
            });
        }
        None
    }
    pub fn held_room(&mut self, bytes: u64) {
        self.reserved = self.reserved.checked_add(bytes).expect("bounded pending copies");
    }
    pub fn reserve(&mut self, bytes: u64) -> bool {
        if !self.make_room(bytes) {
            return false;
        }
        self.held_room(bytes);
        true
    }
    pub fn unreserve(&mut self, bytes: u64) {
        self.reserved = self.reserved.checked_sub(bytes).expect("accounted pending copy");
    }
    fn make_room(&mut self, bytes: u64) -> bool {
        // An incomplete answer and every sent validator keep their room.
        let mut pinned = bytes.saturating_add(self.reserved);
        for held in &self.entries {
            if held.pins > 0 {
                pinned = pinned.saturating_add(held.bytes());
            }
        }
        if pinned > self.limit {
            return false;
        }
        for _ in 0..self.entries.len() {
            if self.bytes.saturating_add(self.reserved).saturating_add(bytes) <= self.limit {
                break;
            }
            let mut oldest = None;
            for held in &self.entries {
                let earlier = match oldest {
                    Some((_, used)) => held.used < used,
                    None => true,
                };
                if held.pins == 0 && earlier {
                    oldest = Some((held.id, held.used));
                }
            }
            let (id, _) = oldest.expect("unpinned room was measured");
            for held in &mut self.entries {
                if held.id == id {
                    held.current = false;
                }
            }
            self.prune();
        }
        true
    }
    pub fn overhead(key: &Key, head: &http::Response, validators: &Validators) -> u64 {
        Entry::overhead(key, head, validators)
    }
    pub fn body_room(key: &Key, head: &http::Response, validators: &Validators, limit: u64) -> Option<u32> {
        Some(u32::try_from(limit.checked_sub(Entry::overhead(key, head, validators))?).unwrap_or(u32::MAX))
    }
    pub fn keep(&mut self, key: Key, head: http::Response, validators: Validators, body: Box<[u8]>) {
        self.drop_key(&key);
        self.clock = self.clock.checked_add(1).expect("conditional read sequence");
        let entry = Entry { id: self.clock, used: self.clock, pins: 0, current: true, key, validators, head, body };
        let bytes = entry.bytes();
        if bytes > self.limit {
            return;
        }
        if !self.make_room(bytes) {
            return;
        }
        let mut entries = List::with_capacity(
            u32::try_from(self.entries.len()).expect("bounded entries").checked_add(1).expect("bounded entries"),
        );
        for held in core::mem::replace(&mut self.entries, Box::from([])) {
            entries.push(held).expect("entry room");
        }
        entries.push(entry).expect("new entry room");
        self.entries = entries.into_boxed();
        self.bytes = self.bytes.checked_add(bytes).expect("bounded kept bytes");
    }
}
fn fresh_field(name: &[u8]) -> bool {
    name.eq_ignore_ascii_case(b"Date")
        || name.eq_ignore_ascii_case(b"x-ratelimit-limit")
        || name.eq_ignore_ascii_case(b"x-ratelimit-remaining")
        || name.eq_ignore_ascii_case(b"x-ratelimit-reset")
        || name.eq_ignore_ascii_case(b"x-ratelimit-resource")
}

#[cfg(test)]
mod tests;
