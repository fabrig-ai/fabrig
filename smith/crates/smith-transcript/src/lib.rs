//! Bounded turn records; this crate retains no runtime state and knows no
//! session, provider configuration or host policy. Generated constructors,
//! encoders and decoders are the entry points. Contract:
//! `protocol/transcript.md`, sections 2-7.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[rustfmt::skip]
mod generated {
    /// Archived version-6 codec, without supplied system context.
    pub mod v6;
    /// Current bounded transcript codec, written from transcript-v7.schema.
    pub mod v7;
}

pub use generated::{v6, v7};

pub use v7::{
    Affinity, AffinityParts, AfterFailedWrite, AfterFailedWriteParts, AllowedCalls, AllowedCallsParts, Ambiguous,
    AmbiguousParts, Backend, Block, ByteClamp, ByteClampParts, BytesClamp, CEILINGS, Call, CallName, CallNameParts,
    CallParts, CallProblem, CommandEnd, CommandEndParts, CommandRunning, CommandRunningParts, CutProblem,
    CutProblemParts, DurationClamp, DurationClampParts, EditFailed, EditFailedParts, Edited, EditedParts, Entry,
    EntryParts, Exit, ExitCode, ExitCodeParts, ExitSignal, ExitSignalParts, Failed, FailedParts, Fault, FieldProblem,
    FieldProblemParts, Found, FoundParts, Hit, HitParts, Invalid, InvalidParts, JobEnd, JobExit, JobExitParts, JobRead,
    JobReadParts, JobsFullProblem, JobsFullProblemParts, Kind, LineNumber, LineNumberParts, Listed, ListedParts,
    Message, MessageParts, Mismatch, Next, NotRun, NotRunParts, NotRunWhy, Opaque, OpaqueParts, Opening, OpeningParts,
    OtherPath, OtherPathParts, Outcome, OutsideChoice, OutsideChoiceParts, OutsidePath, OutsidePathParts,
    OversizeProblem, OversizeProblemParts, OwnedOutcome, OwnedOutcomeParts, Program, ProgramParts, Read, ReadCut,
    ReadParts, Replay, ReplayParts, Resolved, ResolvedFailure, ResolvedFailureParts, Returned, Role, Said, SaidParts,
    SizePart, Text, TextParts, TimeoutClamp, TooLarge, TooLargeParts, ToolResult, ToolResultParts, Turn, TurnKind,
    TurnParts, UnknownJobProblem, UnknownJobProblemParts, Usage, UsageParts, WorkingPath, WorkingPathParts, Written,
    WrittenParts,
};

impl v6::Turn {
    /// The wire format version this archived codec writes and reads.
    pub const FORMAT_VERSION: u64 = 6;
}

impl Turn {
    /// The wire format version this codec writes and reads.
    pub const FORMAT_VERSION: u64 = 7;
}

/// The owner of smith's patches, as their headers spell it (smith/docs/design/protocol/transcript.md, 12).
pub const OWNER: &str = "smith";

/// The turn's family, as its kept texts spell it.
pub const FAMILY: &str = "smith-transcript";

/// The declaration a kept turn is, where a host's machine starts.
pub const ROOT: &str = "Turn";

/// The mark of a smith patch across which a host closes each live window
/// and starts the next run afresh from its brief (smith/docs/design/protocol/transcript.md, 12).
pub const FRESH_WINDOWS: &str = "fresh-windows";

#[path = "../patches/mod.rs"]
pub mod patches;

/// Decode an archived version-6 or current turn; version 6 has no supplied system context.
/// Resuming readers use the current codec directly.
pub fn decode_archived(limits: &v7::Limits, bytes: &[u8]) -> Result<Turn, v7::Problem> {
    if bytes.get(..2) != Some(&[0, 6][..]) {
        return Turn::decode(limits, &mut skein_lib::Reader::new(bytes));
    }
    let length = bytes.len().checked_add(1).expect("a slice and one byte fit usize");
    let mut writer = skein_lib::Writer::new(length);
    writer.put(&7_u16.to_be_bytes()).expect("version within measured record");
    writer.put(bytes.get(2..).expect("version present")).expect("body within measured record");
    writer.put(&[0]).expect("unavailable system context suffix");
    Turn::decode(limits, &mut skein_lib::Reader::new(&writer.finish()))
}
