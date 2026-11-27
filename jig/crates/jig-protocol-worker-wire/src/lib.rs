//! Worker channel records and their sizes, and a resumed start's
//! committed calls fitted to a room, without domain or payload policy.
//! Contracts: protocol/channel/README.md, sections 3 and 4.1, and protocol/channel/owners.md, section 1.2.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/golden.md, section 1).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const SCHEMA: &str = include_str!("../schema/worker-wire-v22.schema");

use alloc::boxed::Box;
use skein_lib::Token;

#[rustfmt::skip]
#[path = "generated/worker_wire.rs"]
pub mod v22;
mod application;
pub mod convert;
mod kinds;
pub mod layouts;
mod spoken;
pub use application::{Application, Carried, Root};
pub use spoken::{Spoken, spoken};
mod loss;
pub use loss::Loss;
pub mod sizes;

pub mod fit;
pub mod parts;
pub use v22::{KeptPart, PartBusy, PartKept, PartKind};

#[cfg(test)]
mod tests;

pub use v22::Record;

pub use v22::{ModelEvidence, ModelFailure, ModelFailureKind};

pub use kinds::{KINDS, Kind, KindRule, Overflow, schema, schema_and_spoken, terms_gap};
pub use v22::Draining;
pub use v22::Restarting;
pub use v22::{
    AccountRefused, AgentFailure, AgentFailureValue, Answer, Bound, Call, CancelReason, CancelReasonValue, Count,
    CountKind, Ended, Ending, Exceeded, Fact, FactValue, Failed, Failure, Invalid, InvalidStart, InvalidValue, Limited,
    LimitedAccount, Long, MessageRefusal, Mismatch, Operation, Parked, PhaseValue, Preparation, PreparationValue,
    Refusal, RefusalValue, Refused, RefusedAccount, Resource, RunFailure, RunFailureValue, StartCause, Turn,
    Unauthorized, Work,
};
pub use v22::{
    Acknowledge, AnsweredCall, ApplicationValue, Assignment, Cancel, CredentialValue, DeliveryAnswer, DeliveryOutcome,
    Grant, GrantValue, Granted, HostAnswer, Inbound, Message, ProgramValue, RelayAnswer, RelayText, Relayed, Scheme,
    SettledAnswer, TurnBusy, TurnKept, Workspace,
};
pub use v22::{Bounds, CEILINGS, Credential, EndpointBound, Hello, Hosting, Phase, Wire};
pub use v22::{Pause, PauseMode, PauseState, Paused};

/// An application's opaque bytes under the token its vocabulary names.
#[derive(PartialEq, Eq, Debug)]
pub struct Part {
    pub token: Token,
    pub bytes: Box<[u8]>,
}

/// The magic and current version (protocol/channel/wires.md, 2).
pub const MAGIC: [u8; 4] = *b"jigw";
pub const VERSION: u16 = wire::VERSION;

/// Jig's refusal reasons, above skein's reserved range (protocol/channel/README.md, 4.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reason {
    Unauthorized,
    Wires,
    Busy,
    Replaced,
    NotAdmitted,
    Stopping,
    Starting,
}

impl Reason {
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::Unauthorized => 256,
            Self::Wires => 257,
            Self::Busy => 258,
            Self::Replaced => 259,
            Self::NotAdmitted => 260,
            Self::Stopping => 261,
            Self::Starting => 262,
        }
    }

    #[must_use]
    pub const fn from_code(code: u16) -> Option<Self> {
        match code {
            256 => Some(Self::Unauthorized),
            257 => Some(Self::Wires),
            258 => Some(Self::Busy),
            259 => Some(Self::Replaced),
            260 => Some(Self::NotAdmitted),
            261 => Some(Self::Stopping),
            262 => Some(Self::Starting),
            _ => None,
        }
    }
}

pub use v22::{
    Check, Login, LoginOtherwise, LoginState, Logins, ProgramSource, SignOut, SignOutResult, SignedOut, SourceFailed,
    SourceFailure, Sources,
};

pub use v22::{InputCall, LiveAsk, LiveKind, Piece};

// Jobs use the worker channel's own records (jig/docs/design/protocol/channel/records.md, 4).
pub use v22::{
    DeliveryOutcomeValue, Job, JobAnswer, JobBound, JobEnding, JobFailed, JobFailure, JobFinished, JobInvalid,
    JobInvalidStart, JobInvalidValue, JobRefusal, JobRefusalValue,
};

pub use v22::{ListFailed, ListFailureValue, ListOutcome, ListRead, ListRefused, ListRow, ListedRows, ReadList};

/// This crate's spoken family and layout (protocol/channel/wires.md, 2).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "jig-worker-wire";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = crate::v22::VERSION;

    /// The embedded current schema text.
    pub const TEXT: &str = crate::SCHEMA;

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
        use super::{FAMILY, TEXT, VERSION, layout};

        #[test]
        fn identity_tracks_schema() {
            let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("current schema");
            assert_eq!(FAMILY.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(VERSION, schema.version());
            assert_eq!(VERSION, 22);
            assert_eq!(layout(), schema.fingerprints().family().digest());

            let changed = TEXT.replacen("spec: u32", "spec: u64", 1);
            assert_ne!(changed, TEXT, "the mutation changes a field's type");
            let changed = skein_codec::schema::parse(changed.as_bytes()).expect("changed schema");
            assert_ne!(layout(), changed.fingerprints().family().digest());
        }
    }
}

pub use v22::{Ask, AskAnswer, AskBytes, Asked};

pub use v22::{AgentEndpoint, Agents, EndpointLimits, Figures, Header, Limit, Opening, Oversized, Tls, Transport};

pub use v22::{ReadUsage, Usage, UsageOutcome, UsageRead, UsageWindow, UsageWindows};
