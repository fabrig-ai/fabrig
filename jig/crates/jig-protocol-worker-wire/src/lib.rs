//! Worker channel records and their sizes, and a resumed start's
//! committed calls fitted to a room, without domain or payload policy.
//! Contracts: protocol/channel/README.md, sections 3, 4.1 and 5.2.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/README.md, section 6).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const SCHEMA: &str = include_str!("../schema/worker-wire-v17.schema");

use alloc::boxed::Box;
use skein_lib::Token;

#[rustfmt::skip]
#[path = "generated/worker_wire.rs"]
pub mod v17;
mod application;
mod kinds;
mod spoken;
pub use application::{Application, Carried, Kept, Root};
pub mod translation;
pub use spoken::{Agreed, Spoken, spoken};
mod loss;
#[path = "../patches/mod.rs"]
pub mod patches;
pub use loss::Loss;
pub mod sizes;

pub mod fit;

#[cfg(test)]
mod tests;

pub use v17::{ModelEvidence, ModelFailure, ModelFailureKind, ModelFailureParts};

pub use kinds::{
    KINDS, Kind, KindRule, Overflow, schema, schema_and_spoken, schema_for, schema_for_prepared, terms_gap,
};
pub use v17::{
    AccountRefused, AccountRefusedParts, AgentFailure, AgentFailureValue, AgentFailureValueParts, Answer, AnswerParts,
    Bound, Call, CallParts, CancelReason, CancelReasonValue, CancelReasonValueParts, Count, CountKind, CountParts,
    Ended, EndedParts, Ending, Exceeded, ExceededParts, Fact, FactParts, FactValue, Failed, FailedParts, Failure,
    Invalid, InvalidStart, InvalidStartParts, InvalidValue, InvalidValueParts, Limited, LimitedAccount,
    LimitedAccountParts, LimitedParts, Long, LongParts, MessageRefusal, Mismatch, MismatchParts, Operation, Parked,
    ParkedParts, PhaseValue, PhaseValueParts, Preparation, PreparationValue, PreparationValueParts, Refusal,
    RefusalValue, RefusalValueParts, Refused, RefusedAccount, RefusedAccountParts, RefusedParts, Resource,
    ResourceParts, RunFailure, RunFailureValue, RunFailureValueParts, StartCause, Turn, TurnParts, Unauthorized,
    UnauthorizedParts, Work, WorkParts,
};
pub use v17::{
    Acknowledge, AcknowledgeParts, AnsweredCall, AnsweredCallParts, ApplicationValue, ApplicationValueParts,
    Assignment, AssignmentParts, Cancel, CancelParts, CredentialValue, CredentialValueParts, DeliveryAnswer,
    DeliveryAnswerParts, DeliveryOutcome, Grant, GrantParts, GrantValue, Granted, GrantedParts, HostAnswer,
    HostAnswerParts, Inbound, InboundParts, Message, MessageParts, ProgramValue, ProgramValueParts, RelayAnswer,
    RelayText, RelayTextParts, Relayed, RelayedParts, Scheme, SettledAnswer, TurnBusy, TurnBusyParts, TurnKept,
    TurnKeptParts, Workspace, WorkspaceParts,
};
pub use v17::{AgreedWire, AgreedWireParts, Draining, DrainingParts};
pub use v17::{
    Bounds, BoundsParts, CEILINGS, Credential, CredentialParts, EndpointBound, EndpointBoundParts, Hello, HelloParts,
    Hosting, HostingParts, Phase, Wire, WireParts,
};
pub use v17::{Restarting, RestartingParts};

/// An application's opaque bytes under the token its vocabulary names.
#[derive(PartialEq, Eq, Debug)]
pub struct Part {
    pub token: Token,
    pub bytes: Box<[u8]>,
}

/// The magic and current version (protocol/channel/bridges.md, 2).
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

pub use v17::{
    Check, CheckParts, Login, LoginOtherwise, LoginOtherwiseParts, LoginParts, LoginState, Logins, LoginsParts,
    ProgramSource, ProgramSourceParts, SignOut, SignOutParts, SignOutResult, SignedOut, SignedOutParts, SourceFailed,
    SourceFailedParts, SourceFailure, Sources, SourcesParts,
};

pub use v17::{InputCall, InputCallParts, LiveAsk, LiveAskParts, LiveKind, Piece, PieceParts};

// Jobs use the worker channel's own records (jig/docs/design/protocol/channel/records.md, 4).
pub use v17::{
    DeliveryOutcomeValue, DeliveryOutcomeValueParts, Job, JobAnswer, JobAnswerParts, JobBound, JobBoundParts,
    JobEnding, JobFailed, JobFailedParts, JobFailure, JobFinished, JobFinishedParts, JobInvalid, JobInvalidStart,
    JobInvalidStartParts, JobInvalidValue, JobInvalidValueParts, JobParts, JobRefusal, JobRefusalValue,
    JobRefusalValueParts,
};

pub use v17::{
    ListFailed, ListFailureValue, ListFailureValueParts, ListOutcome, ListRead, ListReadParts, ListRefused,
    ListRefusedParts, ListRow, ListRowParts, ListedRows, ListedRowsParts, ReadList, ReadListParts,
};

/// This crate's spoken family and layout (protocol/channel/wires.md, 2).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "jig-worker-wire";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = crate::v17::VERSION;

    /// The lowest version this build speaks.
    pub const FLOOR: u16 = crate::patches::kept::jig_worker_wire::FLOOR;

    #[cfg(test)]
    const TEXT: &str = crate::SCHEMA;

    /// The current schema's family fingerprint.
    #[must_use]
    pub fn layout() -> [u8; 32] {
        *crate::patches::kept::jig_worker_wire::LAYOUTS.last().expect("the kept wire has its current layout")
    }

    #[cfg(test)]
    mod tests {
        use super::{FAMILY, TEXT, VERSION, layout};

        #[test]
        fn identity_tracks_schema() {
            let schema = skein_codec::schema::parse(TEXT.as_bytes()).expect("current schema");
            assert_eq!(FAMILY.as_bytes(), schema.spelled(schema.family()));
            assert_eq!(VERSION, schema.version());
            assert_eq!(layout(), schema.fingerprints().family().digest());

            let changed = TEXT.replacen("spec: u32", "spec: u64", 1);
            assert_ne!(changed, TEXT, "the mutation changes a field's type");
            let changed = skein_codec::schema::parse(changed.as_bytes()).expect("changed schema");
            assert_ne!(layout(), changed.fingerprints().family().digest());
        }
    }
}

pub use v17::{Ask, AskAnswer, AskBytes, AskBytesParts, AskParts, Asked, AskedParts};

pub use v17::{
    AgentEndpoint, AgentEndpointParts, Agents, AgentsParts, EndpointLimits, EndpointLimitsParts, Figures, FiguresParts,
    Header, HeaderParts, Limit, LimitParts, Opening, Oversized, Tls, TlsParts, Transport,
};
