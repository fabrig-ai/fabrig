//! What reaches a person (temper/docs/design/domain/forge.md, 5.10).
#![no_std]
#![forbid(unsafe_code)]
#![expect(clippy::disallowed_macros, reason = "the contract gives the empty classes table a Default")]

extern crate alloc;
use alloc::boxed::Box;

mod judgement;
mod narrowing;
mod pattern;
mod record;
mod table;
pub use judgement::judge;
pub use narrowing::{narrowed, offers};
pub use pattern::{matches, well_formed};
pub use record::{classes_from, classes_record, largest, reach_from, reach_record, subject_from, subject_record};
#[cfg(test)]
mod tests;

pub const ITEMS: usize = 20; // items in a table
pub const WORDS_BYTES: usize = 120; // an item's words
pub const SETS: usize = 6; // path sets in a table, numbered 1 to SETS
pub const NAME_BYTES: usize = 40; // a set's name
pub const PATTERNS: usize = 6; // patterns in a set
pub const PATTERN_BYTES: usize = 80; // a pattern
pub const PATHS: usize = 16; // paths in one call
pub const PATH_BYTES: usize = 256; // a path

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Wording,
    Retry,
    Implementation,
    Tests,
    Dependency,
    Departure,
    Money,
    Irreversible,
    Rules,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    Decide,
    Ask,
    AskAway,
    Wait,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Subject {
    Kind(Kind),
    Set(u8),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
    pub subject: Subject,
    pub words: Box<[u8]>,
    pub class: Class,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PathSet {
    pub number: u8,
    pub name: Box<[u8]>,
    pub patterns: Box<[Box<[u8]>]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Classes {
    pub items: Box<[Item]>,
    pub sets: Box<[PathSet]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Way {
    YouApprove,
    AgentsReview,
    DraftFirst,
}

/// The first fault `check` finds; `item` and `set` are an item's place from 1 and a set's place from 1.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    Items,                            // more than ITEMS
    Sets,                             // more than SETS
    Fixed { item: u8 },               // an item of a fixed kind
    Wait { item: u8 },                // an item of class Wait
    Words { item: u8 },               // words past WORDS_BYTES
    NoSet { item: u8 },               // names a set the table lacks
    Twice { item: u8 },               // a second item without words of its subject
    Number { set: u8 },               // a number outside 1 to SETS, or twice
    Name { set: u8 },                 // empty or past NAME_BYTES
    Patterns { set: u8 },             // none, or more than PATTERNS
    Pattern { set: u8, pattern: u8 }, // empty, past PATTERN_BYTES or not well formed (pattern from 1)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ask<'a> {
    pub kind: Kind,
    pub paths: &'a [Box<[u8]>],
    pub under: u8,
    pub away: bool,
    pub always: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Reason {
    Kind,
    Set(u8),
    Always,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reach {
    pub kind: Kind,
    pub class: Class,
    pub reason: Reason,
    pub under: u8,
    pub sets: u8,
    pub matched: u8,
    pub away: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Offer {
    Decide(Subject),
    Words(u8),
    Split(u8),
}
