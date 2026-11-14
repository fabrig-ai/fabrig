//! Store progress on standard error (hearth/docs/design/client/README.md, 3.2).
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Waiting {
    pub store: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Strays {
    pub store: Box<[u8]>,
    pub count: u64,
    pub first: Box<[u8]>,
}
impl Waiting {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"waiting for the store's lock: ");
        lines.put(&self.store);
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"waiting");
        field(json, b"store", &self.store);
    }
}
impl Strays {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"strays in the store ");
        lines.put(&self.store);
        lines.put(b": ");
        lines.decimal(self.count);
        lines.put(b", the first ");
        lines.put(&self.first);
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"strays");
        field(json, b"store", &self.store);
        json.key(b"count");
        json.unsigned(self.count);
        field(json, b"first", &self.first);
    }
}

/// A full store can reach its layer limit (hearth/docs/design/client/README.md, 3.2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StoreFull {
    pub store: Box<[u8]>,
    pub limit: StoreLimit,
    pub most: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum StoreLimit {
    Resolutions,
    Layers,
    CacheFiles,
}
impl StoreFull {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"the store ");
        lines.put(&self.store);
        lines.put(b" is full: ");
        lines.decimal(u64::from(self.most));
        lines.put(match self.limit {
            StoreLimit::Resolutions => b" resolutions",
            StoreLimit::Layers => b" layers",
            StoreLimit::CacheFiles => b" cache files",
        });
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        field(json, b"record", b"error");
        field(json, b"kind", b"store");
        field(json, b"store", &self.store);
        field(
            json,
            b"limit",
            match self.limit {
                StoreLimit::Resolutions => b"resolutions",
                StoreLimit::Layers => b"layers",
                StoreLimit::CacheFiles => b"cache_files",
            },
        );
        json.key(b"most");
        json.unsigned(u64::from(self.most));
        field(json, b"class", b"operational");
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolving {
    pub key: [u8; 32],
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Resolved {
    pub key: [u8; 32],
    pub took: skein_lib::Duration,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Unresolved {
    pub key: [u8; 32],
}
impl Resolving {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"resolving ");
        lines.hex12(&self.key);
        lines.put(b"\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        resolution(json, b"resolving", &self.key);
    }
}
impl Resolved {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"resolved ");
        lines.hex12(&self.key);
        lines.put(b" in ");
        lines.decimal(self.took.as_nanos() / 1_000_000);
        lines.put(b" ms\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        resolution(json, b"resolved", &self.key);
        json.key(b"took_ms");
        json.unsigned(self.took.as_nanos() / 1_000_000);
    }
}
impl Unresolved {
    pub(crate) fn text(&self, lines: &mut Lines) {
        lines.put(b"resolution ");
        lines.hex12(&self.key);
        lines.put(b" failed\n");
    }
    pub(crate) fn json(&self, json: &mut Encoder) {
        resolution(json, b"unresolved", &self.key);
    }
}
fn resolution(json: &mut Encoder, kind: &[u8], key: &[u8; 32]) {
    field(json, b"record", kind);
    json.key(b"key");
    crate::form::json::digest(json, key);
}
