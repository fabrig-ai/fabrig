//! Format and links for the two TOML files (temper/docs/design/protocol/temper-dir.md, 3.1–3.2, 5 and 7).
use crate::judge::{BAD_FORMAT, Report, name_words, newer};
use crate::reading::{Kept, Reading};
use crate::{FAULTS, FILE_BYTES, WAYS};
use alloc::boxed::Box;
use jig_domain_people::settings::is_name;
use skein_lib::{Decimal, List, Set, bytes};
use skein_toml::{Array, Document, Limit, Limits, Table, Value, ValueKind, Why};

const LIMITS: Limits = Limits { input_bytes: FILE_BYTES, depth: FILE_BYTES, entries: FILE_BYTES };
const OFFER_TYPE: &[u8] = b"`offer`: an array of way names";

pub(crate) fn check(tree: &Reading, path: &[u8], body: &[u8], offers: bool, report: &mut Report) {
    let doc = match skein_toml::parse(body, &LIMITS) {
        Ok(doc) => doc,
        Err(refusal) => {
            report.add(path, line(refusal.line), bytes::copy_of(toml_words(refusal.why)));
            return;
        }
    };
    if let Some(entry) = doc.get(doc.root(), b"format")
        && let Err(why) = check_format(&doc, entry.value)
    {
        report.add(path, line(entry.line), why);
        return;
    }
    if offers {
        let mut faults = FileFaults { first: List::with_capacity(FAULTS), more: 0 };
        let offered = check_offer(tree, &doc, &mut faults);
        check_keys(&doc, &offered, &mut faults);
        for fault in faults.first.into_boxed() {
            report.add(path, fault.line, fault.why);
        }
        report.more = report.more.saturating_add(faults.more);
    }
}

fn check_format(doc: &Document, value: Value) -> Result<(), Box<[u8]>> {
    match value.kind {
        ValueKind::Integer(n) if n > 0 && doc.integer_radix(doc.root(), b"format") == Some(10) => {
            if n == 1 {
                return Ok(());
            }
            let number = u64::try_from(n).expect("format is positive");
            Err(newer(Decimal::of(number).as_bytes()))
        }
        ValueKind::Integer(_)
        | ValueKind::String(_)
        | ValueKind::Float(_)
        | ValueKind::Boolean(_)
        | ValueKind::DateTime { .. }
        | ValueKind::Array(_)
        | ValueKind::Table(_)
        | ValueKind::ArrayOfTables(_) => Err(bytes::copy_of(BAD_FORMAT)),
    }
}

fn check_offer<'a>(tree: &Reading, doc: &'a Document, faults: &mut FileFaults) -> Set<&'a [u8]> {
    let Some(entry) = doc.get(doc.root(), b"offer") else {
        faults.add(0, bytes::copy_of(b"missing key `offer`"));
        return Set::with_capacity(0);
    };
    let array = match entry.value.kind {
        ValueKind::Array(array) => array,
        ValueKind::Integer(_)
        | ValueKind::String(_)
        | ValueKind::Float(_)
        | ValueKind::Boolean(_)
        | ValueKind::DateTime { .. }
        | ValueKind::Table(_)
        | ValueKind::ArrayOfTables(_) => {
            faults.add(line(entry.line), bytes::copy_of(OFFER_TYPE));
            return Set::with_capacity(0);
        }
    };
    let count = doc.array_len(array).expect("the array belongs to this document");
    if count == 0 {
        faults.add(line(entry.line), bytes::copy_of(b"`offer`: at least one way name"));
    }
    let mut offered = Set::with_capacity(count);
    for index in 0..count {
        check_offer_element(tree, doc, array, index, &mut offered, faults);
    }
    offered
}

fn check_offer_element<'a>(
    tree: &Reading,
    doc: &'a Document,
    array: Array,
    index: u32,
    offered: &mut Set<&'a [u8]>,
    faults: &mut FileFaults,
) {
    let value = doc.element(array, index).expect("the index is in this array");
    let at = line(value.line);
    let Some(name) = string(doc, value) else {
        faults.add(at, bytes::copy_of(OFFER_TYPE));
        return;
    };
    if !is_name(name) {
        faults.add(at, name_words(name));
        return;
    }
    if !offered.insert(name).expect("the set has a place per offer element") {
        faults.add(at, bytes::concat(&[b"`offer` names `", name, b"` twice"]));
        return;
    }
    let path = bytes::concat(&[WAYS, b"/", name, b".toml"]);
    if way_missing(tree, &path) {
        faults.add(at, bytes::concat(&[b"`offer` names `", name, b"`, and `", &path, b"` is missing"]));
    }
}

fn way_missing(tree: &Reading, path: &[u8]) -> bool {
    // A present but ill-shaped entry already has its own fault. A cut
    // listing doesn't prove that an entry it omitted is missing.
    if tree.entries.contains_key(path) {
        return false;
    }
    match tree.entries.get(WAYS) {
        Some(Kept::Directory { listed, cut }) => *listed && !*cut,
        Some(Kept::Fault(_) | Kept::File { .. }) => true,
        None => !tree.root_cut,
    }
}

fn check_keys(doc: &Document, offered: &Set<&[u8]>, faults: &mut FileFaults) {
    let root = doc.root();
    let count = doc.table_len(root).expect("the root is a table");
    for index in 0..count {
        let entry = doc.entry(root, index).expect("the index is in the root table");
        let key = doc.text(entry.key).expect("the key belongs to this document");
        let at = line(entry.line);
        match key {
            b"format" | b"offer" => {}
            b"default" => match entry.value.kind {
                ValueKind::Table(table) => check_defaults(doc, table, offered, faults),
                ValueKind::Integer(_)
                | ValueKind::String(_)
                | ValueKind::Float(_)
                | ValueKind::Boolean(_)
                | ValueKind::DateTime { .. }
                | ValueKind::Array(_)
                | ValueKind::ArrayOfTables(_) => {
                    faults.add(at, bytes::copy_of(b"`default`: a table of start commands"));
                }
            },
            _ => faults.add(at, bytes::concat(&[b"unknown key `", key, b"`"])),
        }
    }
}

fn check_defaults(doc: &Document, table: Table, offered: &Set<&[u8]>, faults: &mut FileFaults) {
    let count = doc.table_len(table).expect("the table belongs to this document");
    for index in 0..count {
        let entry = doc.entry(table, index).expect("the index is in this table");
        let key = doc.text(entry.key).expect("the key belongs to this document");
        let at = line(entry.line);
        match key {
            b"chat" | b"change" | b"goal" | b"milestone" => {
                let Some(name) = string(doc, entry.value) else {
                    faults.add(at, bytes::concat(&[b"`default.", key, b"`: a way name"]));
                    continue;
                };
                if !is_name(name) {
                    faults.add(at, name_words(name));
                } else if !offered.contains(name) {
                    faults
                        .add(at, bytes::concat(&[b"`default.", key, b"` names `", name, b"`, which `offer` doesn't"]));
                }
            }
            _ => faults.add(at, bytes::concat(&[b"unknown key `default.", key, b"`"])),
        }
    }
}

fn string(doc: &Document, value: Value) -> Option<&[u8]> {
    match value.kind {
        ValueKind::String(text) => doc.text(text),
        ValueKind::Integer(_)
        | ValueKind::Float(_)
        | ValueKind::Boolean(_)
        | ValueKind::DateTime { .. }
        | ValueKind::Table(_)
        | ValueKind::Array(_)
        | ValueKind::ArrayOfTables(_) => None,
    }
}

fn line(line: u64) -> u32 {
    u32::try_from(line).expect("a TOML file has at most 65536 bytes")
}

fn toml_words(why: Why) -> &'static [u8] {
    match why {
        Why::InvalidUtf8 => b"not UTF-8",
        Why::Grammar => b"does not parse as TOML",
        Why::KeyRedefined => b"a TOML key is defined twice",
        Why::TableRedefined => b"a TOML table is defined twice",
        Why::Range => b"a TOML value is outside its range",
        Why::Limit(Limit::InputBytes) => b"past what temper reads, 65,536 bytes",
        Why::Limit(Limit::Depth) => b"past what temper reads: TOML nesting depth",
        Why::Limit(Limit::Entries) => b"past what temper reads: TOML entries",
    }
}

struct LineFault {
    line: u32,
    why: Box<[u8]>,
}

// Offer and default checks use the same set of names, regardless of where
// their keys appear. Keep only the earliest faults of their two walks.
struct FileFaults {
    first: List<LineFault>,
    more: u32,
}

impl FileFaults {
    fn add(&mut self, line: u32, why: Box<[u8]>) {
        let mut fault = LineFault { line, why };
        for index in 0..self.first.len() {
            let current = self.first.get_mut(index).expect("the index is in the list");
            if fault.line < current.line {
                core::mem::swap(current, &mut fault);
            }
        }
        if self.first.push(fault).is_err() {
            self.more = self.more.saturating_add(1);
        }
    }
}
