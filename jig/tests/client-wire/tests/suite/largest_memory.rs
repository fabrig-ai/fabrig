//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use skein_codec::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_client_protocol_wire::CLIENT_SCHEMA,
        jig_client_protocol_wire::v45,
        roots [
            Keyed,
            KeyedAnswer,
            LinkSignIn,
            SignInAnswer,
            SignOut,
            SignOutAnswer,
            ApprovalAsk,
            ApprovalAskAnswer,
            LinkAsk,
            LinkAskAnswer,
            ApprovalRead,
            ApprovalState,
            ApprovalDecide,
            ApprovalDecided,
            ApprovalSignIn,
            ApprovalSignInAnswer,
            TakenBy,
            ResourceLine,
            Fact,
            ChipPhaseFacts,
            ChipDetails,
            ResourceScope,
            Operation,
            NewTask,
            FamilyRecord,
            FamilyRequest,
            ReadAnswer,
            DecodedActivityLine,
            Hold,
            ProposedTask,
            HistoryRow,
            Note,
            ObjectPage,
            EndedDelegate,
            InboxSelection,
            InboxEntry,
            ProviderOffer,
            LiveDelegate,
            ReadPosition,
            OptionLine,
            PersonContract,
            ShownResult,
            Recurring,
            ProjectResource,
            ProjectPolicyRole,
            ProjectSpend,
            GoalHolding,
            TreeRow,
            Subject,
            HostRun,
            ProgramLogin,
            ProgramHost,
            Host,
            Authorize,
            AuthorizeAnswer,
            OpenAnswer,
            CloseAnswer,
            Batch,
            BatchReply,
            Gone,
            StreamEvent,
            PayerReadiness,
            StreamsFull,
            AccountSource,
            AccountLine,
            AccountSignIn,
            Account,
            ProviderNumber,
            Kept,
            TableEntry,
            SettingItem,
            RevisionItem,
            Final,
            Read,
            Offered,
            MeProject,
            EntranceRefusal,
            Ask,
            ReadValue,
            DeliveryValue,
            ReviewItem,
        ]
    );
    family!(
        CAP,
        jig_client_protocol_wire::PUBLIC_SCHEMA,
        jig_client_protocol_wire::public_v1,
        roots [
            PublicProjects,
            PublicBoard,
            PublicCard,
            PublicApplication,
        ]
    );
}

#[test]
fn test_client_largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_client_test_protocol_wire::SCHEMA,
        jig_client_test_protocol_wire::v2,
        roots [
            ClientSubject,
            Request,
            Outcome,
            Document,
            ReadAnswer,
            Place,
            Subject,
            Read,
            FamilyDocumentRecord,
            FamilyAnswerRecord,
        ]
    );
}
