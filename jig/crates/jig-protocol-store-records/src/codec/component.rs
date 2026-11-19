//! The component's own durable rows (protocol/store/records.md, section 3, and protocol/store/transcripts.md, section 1).
use crate::component_v3::{self as wire, Record};
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
impl Levels {
    fn encoded(self) -> wire::Levels {
        wire::Levels { skein: self.skein, smith: self.smith, jig: self.jig, application: self.application }
    }
    fn decoded(row: wire::Levels) -> Self {
        let wire::Levels { skein, smith, jig, application } = row;
        Self { skein, smith, jig, application }
    }
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
        let row = wire::Format {
            component: self.component,
            core: self.core,
            tasks: self.tasks,
            people: self.people,
            notes: self.notes,
            accounts: self.accounts,
            settings: self.settings,
            application: self.application,
            deployment: self.deployment,
            levels: self.levels.encoded(),
        };
        row.encode(&wire::CEILINGS)
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::Format { component, core, tasks, people, notes, accounts, settings, application, deployment, levels } =
            wire::Format::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self {
            component,
            core,
            tasks,
            people,
            notes,
            accounts,
            settings,
            application,
            deployment,
            levels: Levels::decoded(levels),
        })
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
        let row = wire::Manifest {
            generation: self.generation,
            from: self.from,
            before: self.before.encoded(),
            after: self.after.encoded(),
            counts: self.counts,
            store: self.store,
            secrets: self.secrets,
            written: self.written,
        };
        row.encode(&wire::CEILINGS).expect("manifest counts fit the checked sets")
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::Manifest { generation, from, before, after, counts, store, secrets, written } =
            wire::Manifest::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self {
            generation,
            from,
            before: Levels::decoded(before),
            after: Levels::decoded(after),
            counts,
            store,
            secrets,
            written,
        })
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
        let row = wire::Checkpoint {
            store: self.store,
            secrets: self.secrets,
            made: self.made,
            build: self.build,
            deployment: self.deployment,
            levels: self.levels.encoded(),
            files: self.files,
        };
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::Checkpoint { store, secrets, made, build, deployment, levels, files } =
            wire::Checkpoint::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self { store, secrets, made, build, deployment, levels: Levels::decoded(levels), files })
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
        let row = wire::Transcript { length: self.length, window: self.window };
        row.encode(&wire::CEILINGS)
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::Transcript { length, window } = wire::Transcript::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self { length, window })
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
        let row = wire::Attempt { first: self.first };
        row.encode(&wire::CEILINGS)
    }

    /// Decode one row, refusing another version or trailing bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::Attempt { first } = wire::Attempt::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self { first })
    }
}

/// A task log's place (protocol/store/logs.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HistoryLog {
    pub record: Option<u64>,
}
impl HistoryLog {
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = wire::HistoryLog { record: self.record };
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::HistoryLog { record } = wire::HistoryLog::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self { record })
    }
}

/// A task log's place (protocol/store/logs.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TurnLog {
    pub window: Option<u64>,
}
impl TurnLog {
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = wire::TurnLog { window: self.window };
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::TurnLog { window } = wire::TurnLog::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self { window })
    }
}

/// A task log's place (protocol/store/logs.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LogAttempt {
    pub first: u64,
}
impl LogAttempt {
    pub fn encode(&self) -> Result<Box<[u8]>, wire::Problem> {
        let row = wire::LogAttempt { first: self.first };
        row.encode(&wire::CEILINGS)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, wire::Problem> {
        let wire::LogAttempt { first } = wire::LogAttempt::decode(&wire::CEILINGS, &mut Reader::new(bytes))?;
        Ok(Self { first })
    }
}
