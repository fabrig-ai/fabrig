//! Bounded turn records; this crate retains no runtime state and knows no
//! session, provider configuration or host policy. Generated records,
//! encoders and decoders are the entry points. Contract:
//! `protocol/transcript.md`, sections 2-7.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/golden.md, section 1).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const SCHEMA: &str = include_str!("../schema/transcript-v8.schema");

/// Texts a store registers for frames written before its registry: version 7, then `SCHEMA` at 8
/// (smith/docs/design/protocol/transcript.md, 12).
#[expect(clippy::disallowed_macros, reason = "the store seed embeds its one older schema text")]
pub const SEED: &[&str] = &[include_str!("../schema/seed/smith-transcript-v7.schema"), SCHEMA];

#[rustfmt::skip]
mod generated {
    /// Current bounded transcript codec, written from transcript-v8.schema.
    pub mod v8 {
        include!("generated/transcript.rs");
    }
}

pub use generated::v8;

pub use v8::{
    Affinity, AfterFailedWrite, AllowedCalls, Ambiguous, Backend, Block, ByteClamp, BytesClamp, CEILINGS, Call,
    CallName, CallProblem, CommandEnd, CommandRunning, CutProblem, DurationClamp, EditFailed, Edited, Entry, Exit,
    ExitCode, ExitSignal, Failed, Fault, FieldProblem, Found, Hit, Interrupted, Invalid, JobEnd, JobExit, JobRead,
    JobsFullProblem, Kind, LineNumber, Listed, Message, Mismatch, Next, NotRun, NotRunWhy, Opaque, Opening, OtherPath,
    Outcome, OutsideChoice, OutsidePath, OversizeProblem, OwnedOutcome, Problem, Program, Read, ReadCut, Record,
    Replay, Resolved, ResolvedFailure, Returned, Role, Said, SizePart, Text, TimeoutClamp, TooLarge, ToolResult, Turn,
    TurnKind, UnknownJobProblem, Usage, WorkingPath, Written, paths,
};

impl Turn {
    /// The wire format version this codec writes and reads.
    pub const FORMAT_VERSION: u64 = {
        let [high, low] = wire::VERSION.to_be_bytes();
        u64::from_be_bytes([0, 0, 0, 0, 0, 0, high, low])
    };
}

/// The owner of smith's patches, as their headers spell it (smith/docs/design/protocol/transcript.md, 12).
pub const OWNER: &str = "smith";

/// The turn's family, as its kept texts spell it.
pub const FAMILY: &str = wire::FAMILY;

/// The declaration a kept turn is, where a host's machine starts.
pub const ROOT: &str = "Turn";

/// The mark of a smith patch across which a host closes each live window
/// and starts the next run afresh from its brief (smith/docs/design/protocol/transcript.md, 12).
pub const FRESH_WINDOWS: &str = "fresh-windows";

#[path = "../patches/mod.rs"]
pub mod patches;

/// Decode an archived version-6 or current turn; version 6 has no supplied system context.
/// Resuming readers use the current codec directly.
pub fn decode_archived(limits: &v8::Limits, bytes: &[u8]) -> Result<Turn, Problem> {
    if bytes.get(..2) != Some(&[0, 6][..]) {
        return Turn::decode(limits, &mut skein_lib::Reader::new(bytes));
    }
    let length = bytes.len().checked_add(1).expect("a slice and one byte fit usize");
    let mut writer = skein_lib::Writer::new(length);
    writer.put(&wire::VERSION.to_be_bytes()).expect("version within measured record");
    writer.put(bytes.get(2..).expect("version present")).expect("body within measured record");
    writer.put(&[0]).expect("unavailable system context suffix");
    Turn::decode(limits, &mut skein_lib::Reader::new(&writer.finish()))
}

/// This crate's spoken family and layout (temper/docs/design/protocol/wires.md, 3.1).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "smith-transcript";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = crate::v8::VERSION;

    /// The lowest version this build speaks.
    pub const FLOOR: u16 = crate::patches::kept::smith_transcript::FLOOR;

    const TEXT: &str = crate::SCHEMA;

    /// The current schema's family fingerprint.
    #[must_use]
    pub fn layout() -> [u8; 32] {
        skein_codec::schema::parse(TEXT.as_bytes())
            .expect("the embedded current schema is valid")
            .fingerprints()
            .family()
            .digest()
    }

    #[cfg(test)]
    mod tests {
        use super::{FAMILY, FLOOR, TEXT, VERSION, layout};

        #[test]
        fn identity_tracks_schema() {
            let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("current schema");
            assert_eq!(FAMILY.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(VERSION, schema.version());
            assert_eq!(FLOOR, crate::patches::kept::smith_transcript::FLOOR);
            assert_eq!(FLOOR, 7);
            assert_eq!(layout(), schema.fingerprints().family().digest());

            let changed = TEXT.replacen("input: option u64", "input: option u32", 1);
            assert_ne!(changed, TEXT, "the mutation changes a field's type");
            let changed = skein_codec::schema::parse(changed.as_bytes()).expect("changed schema");
            assert_ne!(layout(), changed.fingerprints().family().digest());
        }
    }
}
