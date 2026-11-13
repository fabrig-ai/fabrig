//! Bounded channel records and the version-seven kind table. This crate keeps no
//! runtime state and knows no domain, credentials or host policy. Generated
//! constructors, encoders and decoders handle bodies; [`schema`] advertises
//! their largest permitted encodings. Contracts: `protocol/channel.md`,
//! sections 2-4 and 6-8.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

#[rustfmt::skip]
mod generated {
    pub mod v7;
}

pub use generated::v7;

mod frame;
pub mod kind;
mod kinds;
pub mod reason;

pub use frame::{Body, frame};

pub use kinds::{KindRule, Overflow, V7, peer_terms_gap, schema};
pub use v7::{
    Cap, CompletionLimit, CompletionPhase, LimitFailure, LimitFailureParts, PathCap, PathCapParts, PoolLimit,
    PoolLimitParts, ResponseEvidence, ResponseEvidenceParts, TimeoutFailure, TimeoutFailureParts,
};

pub use v7::{
    Accepted, AcceptedParts, AccountRefused, AccountRefusedParts, Acknowledge, AcknowledgeParts, Admitted,
    AdmittedParts, Answer, AnswerParts, AnsweredCall, AnsweredCallParts, Ask, BudgetCeiling, BudgetCeilingValue,
    BudgetCeilingValueParts, BudgetChecks, BudgetChecksParts, BudgetCompletion, BudgetCompletionParts, BudgetFailure,
    BudgetFailureValue, BudgetFailureValueParts, BudgetPart, BudgetPartValue, BudgetPartValueParts, BudgetRefusal,
    BudgetRefusalValue, BudgetRefusalValueParts, BudgetSpend, BudgetSpendParts, BudgetTime, BudgetTimeParts,
    BudgetTurns, BudgetTurnsParts, CEILINGS, Call, CallName, CallNameParts, CallParts, Cancel, CancelParts,
    CompletionEvidence, CompletionFailure, CompletionFault, CompletionFaultParts, CredentialValue,
    CredentialValueParts, DeliverAsk, DeliverAskParts, Delivered, DeliveredParts, Delivery, DeliveryFailure,
    DeliveryFailureParts, DeliveryReason, DeliveryRefusal, DeliveryRefusalParts, DeliveryReply, DeliveryReplyParts,
    Directory, DirectoryParts, Drain, DrainParts, Effect, Fact, FactKind, FactParts, Failed, FailedParts, Field,
    FieldParts, Grant, GrantParts, GrantRefresh, GrantRefreshParts, GrantValue, HostAnswer, HostAnswerParts, HostAsk,
    HostAskParts, HostReply, HostReplyParts, InvalidStart, InvalidStartValue, InvalidStartValueParts, Limited,
    LimitedFault, LimitedFaultParts, LimitedParts, Limits, Live, LiveAsk, LiveAskParts, LiveCall, LiveCallParts,
    LiveKind, LiveParts, Long, LongDone, LongDoneParts, LongParts, Marker, MarkerParts, Message, MessageParts,
    MessageRefusal, MessageRefused, MessageRefusedParts, ModelFault, ModelFaultValue, ModelFaultValueParts, Operation,
    Overflow as BudgetOverflow, OverflowValue, OverflowValueParts, PolicyFailure, PolicyFailureParts, PriceRate,
    PriceRateValue, PriceRateValueParts, ProgramFault, ProgramFaultParts, ProgramValue, ProgramValueParts, Receipt,
    ReceiptParts, ReceivingLimit, ReceivingLimitValue, ReceivingLimitValueParts, Refused, RefusedFault,
    RefusedFaultParts, RefusedParts, Reply, RunFailure, RunResult, SavedReply, Scheme, Start, StartParts, StartRefusal,
    TranscriptRefusal, TranscriptRefusalValue, TranscriptRefusalValueParts, Turn, TurnParts, Unauthorized,
    UnauthorizedFault, UnauthorizedFaultParts, UnauthorizedParts, Waiting, WaitingParts, Window, WindowBound,
    WindowBoundParts, WindowParts, Withdraw, WithdrawParts, Workspace, WorkspaceParts,
};

pub use v7::{
    ListAnswer, ListAsk, ListAskParts, ListFailed, ListFailure, ListFailureParts, ListRefused, ListRefusedParts,
    ListRow, ListRowParts, ListRows, ListRowsParts,
};

/// This crate's spoken family and layout (temper/docs/design/protocol/wires.md, 3.1).
pub mod wire {
    /// The family named by the current schema's header.
    pub const FAMILY: &str = "smith-channel";

    /// The version named by the current schema's header.
    pub const VERSION: u16 = 7;

    #[expect(clippy::disallowed_macros, reason = "the wire identity embeds its current schema")]
    const TEXT: &str = include_str!("../schema/channel-v7.schema");

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
