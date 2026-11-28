//! A repository's `.temper/` at one commit (temper/docs/design/protocol/temper-dir.md, 2–8).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use skein_lib::{Decimal, Overflow, Writer};

mod judge;
mod reading;
mod ways;

pub use reading::{MethodFile, Reading, Unread};

const ROOT: &[u8] = b".temper";
const WAYS_TOML: &[u8] = b".temper/ways.toml";
const WAYS: &[u8] = b".temper/ways";
const CHARTERS: &[u8] = b".temper/charters";
const HOOKS: &[u8] = b".temper/hooks";
const DIRECTORIES: [&[u8]; 3] = [CHARTERS, HOOKS, WAYS];
const METHOD: [&[u8]; 3] = [CHARTERS, WAYS, WAYS_TOML];
const FILE_BYTES: u32 = 65_536;
const FILES: u32 = 64;
const TOTAL_BYTES: u32 = 524_288;
const FAULTS: u32 = 16;
const READER: &[u8] = b"temper-engine";

/// The repository's place in the project's method.
#[derive(Debug, PartialEq, Eq)]
pub enum Home {
    Here,
    Elsewhere { name: Box<[u8]> },
    None,
}

/// A tree's method and the first sixteen faults, with the count past them.
#[derive(Debug, PartialEq, Eq)]
pub struct Verdict {
    pub method: bool,
    pub faults: Box<[Fault]>,
    pub more: u32,
}

/// One fault, on a line from one or at zero for a whole path.
#[derive(Debug, PartialEq, Eq)]
pub struct Fault {
    pub path: Box<[u8]>,
    pub line: u32,
    pub why: Box<[u8]>,
}

impl Fault {
    /// Write `PATH:LINE: WHY` (or `PATH: WHY` at line zero), without a newline.
    /// # Errors
    /// The writer has insufficient room.
    pub fn write(&self, writer: &mut Writer) -> Result<(), Overflow> {
        writer.put(&self.path)?;
        if self.line != 0 {
            writer.put(b":")?;
            writer.put(Decimal::of(u64::from(self.line)).as_bytes())?;
        }
        writer.put(b": ")?;
        writer.put(&self.why)
    }
}

/// What the comparison shows of `.temper/`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    Method,
    Temper,
    None,
    Unknown,
}

/// Classify the paths a comparison names; `whole` means none were omitted.
#[must_use]
pub fn touched(paths: &[Box<[u8]>], whole: bool) -> Touch {
    let mut temper = false;
    for path in paths {
        if path.as_ref() == WAYS_TOML || path.starts_with(b".temper/ways/") || path.starts_with(b".temper/charters/") {
            return Touch::Method;
        }
        temper |= path.starts_with(b".temper/");
    }
    if temper {
        Touch::Temper
    } else if whole {
        Touch::None
    } else {
        Touch::Unknown
    }
}
