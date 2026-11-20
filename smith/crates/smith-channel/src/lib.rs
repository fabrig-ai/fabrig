//! Bounded channel records and the version-eleven kind table. This crate keeps no
//! runtime state and knows no domain, credentials or host policy. Generated
//! records, encoders and decoders handle bodies; [`schema`] advertises
//! their largest permitted encodings. Contracts: `protocol/channel.md`,
//! sections 2-4 and 6-8.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

/// The current family schema text (skein/docs/design/data/codec/README.md, section 6).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const CHANNEL_SCHEMA: &str = include_str!("../schema/channel-v11.schema");

/// The current family schema text (skein/docs/design/data/codec/README.md, section 6).
#[expect(clippy::disallowed_macros, reason = "the owner embeds its current schema text once")]
pub const PIECES_SCHEMA: &str = include_str!("../schema/pieces-v1.schema");

#[rustfmt::skip]
mod generated {
    pub mod v11 {
        include!("generated/channel.rs");
    }
}

pub use generated::v11;

mod frame;
pub mod kind;
mod kinds;
pub mod pieces;
pub mod reason;

pub use frame::{Body, frame};

pub use kinds::{KindRule, Overflow, V11, peer_terms_gap, schema};
pub use v11::{
    Cap, CompletionLimit, CompletionPhase, LimitFailure, PathCap, PoolLimit, Record, Renewals, ResponseEvidence,
    TimeoutFailure,
};

pub use v11::{
    Accepted, AccountRefused, Acknowledge, Admitted, Answer, AnsweredCall, Ask, BudgetCeiling, BudgetCeilingValue,
    BudgetChecks, BudgetCompletion, BudgetFailure, BudgetFailureValue, BudgetPart, BudgetPartValue, BudgetRefusal,
    BudgetRefusalValue, BudgetSpend, BudgetTime, BudgetTurns, CEILINGS, Call, CallName, Cancel, CompletionEvidence,
    CompletionFailure, CompletionFault, CredentialValue, DeliverAsk, Delivered, Delivery, DeliveryFailure,
    DeliveryReason, DeliveryRefusal, DeliveryReply, Directory, Drain, Effect, Fact, FactKind, Failed, Field, Grant,
    GrantRefresh, GrantValue, HostAnswer, HostAsk, HostReply, InvalidStart, InvalidStartValue, Limited, LimitedFault,
    Limits, Live, LiveAsk, LiveCall, LiveKind, Long, LongDone, Marker, Message, MessageRefusal, MessageRefused,
    ModelFault, ModelFaultValue, Operation, Overflow as BudgetOverflow, OverflowValue, PolicyFailure, PriceRate,
    PriceRateValue, ProgramFault, ProgramValue, Receipt, ReceivingLimit, ReceivingLimitValue, Refused, RefusedFault,
    Reply, RunFailure, RunResult, SavedReply, Scheme, Start, StartRefusal, TranscriptRefusal, TranscriptRefusalValue,
    Turn, Unauthorized, UnauthorizedFault, Waiting, Window, WindowBound, Withdraw, Workspace,
};

pub use v11::{ListAnswer, ListAsk, ListFailed, ListFailure, ListRefused, ListRow, ListRows};

/// This crate's spoken family and layout (temper/docs/design/protocol/wires.md, 3.1).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "smith-channel";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = crate::v11::VERSION;

    /// The lowest version this build speaks.
    pub const FLOOR: u16 = crate::patches::kept::smith_channel::FLOOR;

    pub(crate) const TEXT: &str = crate::CHANNEL_SCHEMA;

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
            assert_eq!(FLOOR, crate::patches::kept::smith_channel::FLOOR);
            assert_eq!(FLOOR, VERSION);
            assert_eq!(layout(), schema.fingerprints().family().digest());

            let changed = TEXT.replacen("completion: u32", "completion: u64", 1);
            assert_ne!(changed, TEXT, "the mutation changes a field's type");
            let changed = skein_codec::schema::parse(changed.as_bytes()).expect("changed schema");
            assert_ne!(layout(), changed.fingerprints().family().digest());
        }
    }
}

#[path = "../patches/mod.rs"]
pub mod patches;
