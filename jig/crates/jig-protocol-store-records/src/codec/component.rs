//! The component's own durable rows (protocol/store/records.md, section 3, and protocol/store/transcripts.md, section 1).
use crate::component_v3::{self as wire, Record};
use crate::generated::component as maps;
use alloc::boxed::Box;
use skein_lib::{List, Reader};
pub use wire::{CheckpointFile, Count};

/// Each owner's applied patch level (protocol/store/records.md 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Levels {
    pub skein: Option<u32>,
    pub smith: Option<u32>,
    pub jig: Option<u32>,
    pub application: Option<u32>,
}
/// The component's format row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Format {
    pub component: u16,
    pub core: u16,
    pub tasks: u16,
    pub people: u16,
    pub notes: u16,
    pub accounts: u16,
    pub settings: u16,
    pub application: u32,
    pub deployment: [u8; 16],
    pub levels: Levels,
}

impl Format {
    /// Encode this row with the component family's version.
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::format_to_record(self, &())?;
        row.encode(&wire::CEILINGS)
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::Format::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::format_to_domain(row, &())
    }

    /// The component family's version a stored row begins with, read before
    /// the row decodes, so that opening can say which version it found.
    #[must_use]
    pub fn version(bytes: &[u8]) -> Option<u16> {
        Reader::new(bytes).u16()
    }
}

/// A generation's immutable manifest (protocol/store/README.md 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Manifest {
    pub generation: u64,
    pub from: Option<u64>,
    pub before: Levels,
    pub after: Levels,
    pub counts: List<Count>,
    pub store: u64,
    pub secrets: Option<u64>,
    pub written: u64,
}
impl Manifest {
    #[must_use]
    pub fn encode(self) -> Box<[u8]> {
        let row = maps::manifest_to_record(&self, &()).expect("manifest translation is infallible");
        row.encode(&wire::CEILINGS).expect("manifest counts fit the checked sets")
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::Manifest::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::manifest_to_domain(row, &())
    }
}

/// A checkpoint's immutable record (protocol/store/checkpoints.md, 2.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Checkpoint {
    pub store: u64,
    pub secrets: u64,
    pub made: u64,
    pub build: Box<[u8]>,
    pub deployment: [u8; 16],
    pub levels: Levels,
    pub files: List<CheckpointFile>,
}
impl Checkpoint {
    pub fn encode(self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::checkpoint_to_record(&self, &())?;
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::Checkpoint::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::checkpoint_to_domain(row, &())
    }
}

/// The component's transcript row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Transcript {
    pub length: u64,
    pub window: u64,
}

impl Transcript {
    /// Encode this row with the component family's version.
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::transcript_to_record(self, &())?;
        row.encode(&wire::CEILINGS)
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::Transcript::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::transcript_to_domain(row, &())
    }
}

/// The component's attempt row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Attempt {
    pub first: u64,
}

impl Attempt {
    /// Encode this row with the component family's version.
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::attempt_to_record(self, &())?;
        row.encode(&wire::CEILINGS)
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::Attempt::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::attempt_to_domain(row, &())
    }
}

/// A task log's place (protocol/store/logs.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HistoryLog {
    pub record: Option<u64>,
}
impl HistoryLog {
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::history_log_to_record(self, &())?;
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::HistoryLog::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::history_log_to_domain(row, &())
    }
}

/// A task log's place (protocol/store/logs.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TurnLog {
    pub window: Option<u64>,
}
impl TurnLog {
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::turn_log_to_record(self, &())?;
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::TurnLog::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::turn_log_to_domain(row, &())
    }
}

/// A task log's place (protocol/store/logs.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LogAttempt {
    pub first: u64,
}
impl LogAttempt {
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = maps::log_attempt_to_record(self, &())?;
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let row = wire::LogAttempt::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        maps::log_attempt_to_domain(row, &())
    }
}
