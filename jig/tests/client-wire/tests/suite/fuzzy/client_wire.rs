//! Arbitrary and mutated client bytes (skein's codec.md, section 6).
//! Job arms follow jig/docs/design/client/domain/contract.md, 5.7.
use jig_client_protocol_wire::v19::*;
use skein_lib::{Reader, Rng, Writer};
use std::{fs, path::Path};

macro_rules! check {
    ($record:ident, $bytes:expr) => {
        if let Ok(record) = $record::decode(&CEILINGS, &mut Reader::new($bytes)) {
            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
            record.encode(&mut writer).expect("measured room");
            assert_eq!(writer.finish().as_ref(), $bytes, stringify!($record));
        }
    };
}

#[path = "../../fuzzy_client_wire/documents.rs"]
mod documents;

fn records_1(bytes: &[u8]) {
    check!(CharterDocument, bytes);
    check!(Charter, bytes);
    check!(ReadValue, bytes);
    check!(DeliveryValue, bytes);
    check!(StreamText, bytes);
    check!(Chips, bytes);
    check!(Inbox, bytes);
    check!(Kept, bytes);
    check!(TableEntry, bytes);
    check!(Family, bytes);
    check!(Owner, bytes);
    check!(Keyed, bytes);
    check!(Owned, bytes);
    check!(KeyedAnswer, bytes);
    check!(LinkSignIn, bytes);
    check!(SignedIn, bytes);
    check!(SignInAnswer, bytes);
    check!(SignOut, bytes);
    check!(SignOutAnswer, bytes);
    check!(Authorize, bytes);
    check!(AuthorizeGo, bytes);
    check!(AuthorizeWhy, bytes);
    check!(AuthorizeRefused, bytes);
    check!(AuthorizeAnswer, bytes);
    check!(DocumentText, bytes);
    check!(ProposeSettings, bytes);
    check!(OriginRevision, bytes);
    check!(ReviewItem, bytes);
    check!(SettingsReview, bytes);
    check!(ProposalActionSettings, bytes);
    check!(RetainedActionSettings, bytes);
    check!(SettingOrigin, bytes);
    check!(ReviewValue, bytes);

    check!(ChipExecutorAgent, bytes);
    check!(ChipExecutorProcedure, bytes);
    check!(ExecutorParty, bytes);
    check!(ExecutorRole, bytes);
    check!(ChipExecutorJob, bytes);
    check!(ChipExecutor, bytes);
    check!(Phase, bytes);
    check!(RunStageClaimed, bytes);
    check!(RunStageRunning, bytes);
    check!(RunStageBackingOff, bytes);
    check!(RunStage, bytes);
    check!(PersonAddressParty, bytes);
    check!(PersonAddressRole, bytes);
    check!(PersonAddress, bytes);
    check!(ClosingStageRun, bytes);
    check!(ClosingStage, bytes);
    check!(HoldFailure, bytes);
    check!(HoldCategoryFailures, bytes);
    check!(HoldCategoryDependency, bytes);
    check!(HoldCategoryPlaced, bytes);
    check!(HoldCategoryUncertain, bytes);
    check!(HoldCategoryPoolLost, bytes);
    check!(HoldCategory, bytes);
    check!(ChipPhaseWaiting, bytes);
    check!(ChipPhaseActive, bytes);
    check!(ChipPhasePerson, bytes);
    check!(ChipPhaseClosing, bytes);
    check!(ChipPhaseHeld, bytes);
    check!(ChipPhase, bytes);
    check!(FailureClass, bytes);
}

fn records_2(bytes: &[u8]) {
    check!(RetryNoticeOwnerFailed, bytes);
    check!(RetryNoticeFailed, bytes);
    check!(RetryNoticeRefused, bytes);
    check!(RetryNotice, bytes);
    check!(CallIdentity, bytes);
    check!(RunWorkTool, bytes);
    check!(RunWork, bytes);
    check!(RunObservation, bytes);
    check!(ActiveRun, bytes);
    check!(ProposerTask, bytes);
    check!(ProposerParty, bytes);
    check!(Proposer, bytes);
    check!(ProposalKey, bytes);
    check!(RecipientTask, bytes);
    check!(RecipientParty, bytes);
    check!(RecipientRole, bytes);
    check!(RecipientRoles, bytes);
    check!(Recipient, bytes);
    check!(TakenBy, bytes);
    check!(Revision, bytes);
    check!(ActorParty, bytes);
    check!(ActorTask, bytes);
    check!(ActorProjectDeployment, bytes);
    check!(Actor, bytes);
    check!(PartyKind, bytes);
}

fn records_3(bytes: &[u8]) {
    check!(Party, bytes);
    check!(DecisionRejected, bytes);
    check!(DecisionReleased, bytes);
    check!(DecisionLeftHeld, bytes);
    check!(Decision, bytes);
    check!(AttemptFailure, bytes);
    check!(ExecutorAgent, bytes);
    check!(ExecutorProcedure, bytes);
    check!(ExecutorPerson, bytes);
    check!(ExecutorJob, bytes);
    check!(Executor, bytes);
    check!(WakeRuleBatch, bytes);
    check!(WakeRule, bytes);
    check!(ResultsWake, bytes);
    check!(WakePolicy, bytes);
    check!(PersonStallNotice, bytes);
    check!(PersonWait, bytes);
    check!(PartKind, bytes);
    check!(PartKeyPath, bytes);
    check!(PartKeyNumber, bytes);
    check!(PartKey, bytes);
    check!(Part, bytes);
    check!(PriorPhaseActive, bytes);
    check!(PriorPhaseClosing, bytes);
    check!(PriorPhase, bytes);
    check!(EscalationHolderTask, bytes);
    check!(EscalationHolderPerson, bytes);
    check!(EscalationHolderRole, bytes);
    check!(EscalationHolder, bytes);
    check!(HoldEscalationWaiting, bytes);
    check!(HoldEscalationRejected, bytes);
    check!(HoldEscalation, bytes);
    check!(FailureTries, bytes);
    check!(ChipHold, bytes);
    check!(Numbers, bytes);
    check!(FunderTask, bytes);
    check!(FunderPool, bytes);
    check!(FunderRecurring, bytes);
    check!(FunderPeriod, bytes);
    check!(Funder, bytes);
    check!(Budget, bytes);
}

fn records_4(bytes: &[u8]) {
    check!(ResourceWriterRun, bytes);
    check!(ResourceWriterEffect, bytes);
    check!(ResourceWriter, bytes);
    check!(ResourceLine, bytes);
    check!(ChipWorkTool, bytes);
    check!(ChipWork, bytes);
    check!(ChipObservation, bytes);
    check!(RequesterTask, bytes);
    check!(RequesterPerson, bytes);
    check!(RequesterDeployment, bytes);
    check!(Requester, bytes);
    check!(ResultField, bytes);
    check!(ChipFields, bytes);
    check!(ChipRow, bytes);
    check!(ChipPhaseOwnerFields, bytes);
    check!(ChipPhaseRow, bytes);
    check!(ClosingFields, bytes);
    check!(ClosingRow, bytes);
    check!(EndingTaskFields, bytes);
    check!(EndingFailedFields, bytes);
    check!(EndingCancelledFields, bytes);
    check!(EndingWithdrawnFields, bytes);
    check!(EndingDecidedFields, bytes);
    check!(EndingFields, bytes);
    check!(EndingRow, bytes);
    check!(HeldRuntimeActiveFields, bytes);
    check!(HeldRuntimeClosingFields, bytes);
    check!(HeldRuntimeFields, bytes);
}

fn records_5(bytes: &[u8]) {
    check!(HeldRuntimeRow, bytes);
    check!(HoldReasonTriesFields, bytes);
    check!(HoldReasonStalledFields, bytes);
    check!(HoldReasonDependencyFields, bytes);
    check!(HoldReasonBudgetFields, bytes);
    check!(HoldReasonDeadlineFields, bytes);
    check!(HoldReasonStoppedFields, bytes);
    check!(HoldReasonFields, bytes);
    check!(HoldReasonRow, bytes);
    check!(ChipDetailsFields, bytes);
    check!(ChipDetailsRow, bytes);
    check!(ChipEndingDoneFields, bytes);
    check!(ChipEndingFailedFields, bytes);
    check!(ChipEndingCancelledFields, bytes);
    check!(ChipEndingFields, bytes);
    check!(ChipEndingRow, bytes);
    check!(ResultValueReportFields, bytes);
    check!(ResultValueOwnerVerdictFields, bytes);
    check!(ResultValueVerdictFields, bytes);
    check!(ResultValueChangeFields, bytes);
    check!(ResultValueChoiceFields, bytes);
    check!(ResultValueApprovedFields, bytes);
    check!(ResultValueFailureFields, bytes);
    check!(ResultValueWordsFields, bytes);
    check!(ResultValueFields, bytes);
    check!(ResultValueRow, bytes);
    check!(Fact, bytes);
    check!(Chip, bytes);
    check!(ChipPhaseFacts, bytes);
    check!(Closing, bytes);
    check!(Ending, bytes);
    check!(HeldRuntime, bytes);
    check!(HoldReason, bytes);
}

fn records_6(bytes: &[u8]) {
    check!(ChipDetails, bytes);
    check!(ChipEnding, bytes);
    check!(ResultValue, bytes);
    check!(PersonAnswerChoice, bytes);
    check!(PersonAnswerApproved, bytes);
    check!(PersonAnswerWords, bytes);
    check!(PersonAnswerCannot, bytes);
    check!(PersonAnswer, bytes);
    check!(AddressChange, bytes);
    check!(ParameterNumber, bytes);
    check!(ParameterBytes, bytes);
    check!(ParameterResource, bytes);
    check!(Parameter, bytes);
    check!(Specification, bytes);
    check!(PatternEndExact, bytes);
    check!(PatternEndOpen, bytes);
    check!(PatternEnd, bytes);
    check!(Pattern, bytes);
    check!(Grant, bytes);
    check!(AuthorityExecutorCharter, bytes);
    check!(AuthorityExecutorProcedure, bytes);
    check!(AuthorityExecutorRole, bytes);
    check!(AuthorityExecutorJob, bytes);
    check!(AuthorityExecutor, bytes);
    check!(Delegation, bytes);
    check!(ResourceScope, bytes);
    check!(Authority, bytes);
    check!(Amendment, bytes);
    check!(SeenResults, bytes);
    check!(SeenConversation, bytes);
    check!(SeenThrough, bytes);
    check!(Seen, bytes);
    check!(NoteScopeProject, bytes);
    check!(NoteScopeResource, bytes);
    check!(NoteScopeTask, bytes);
}

fn records_7(bytes: &[u8]) {
    check!(NoteScope, bytes);
    check!(PolicyEditRole, bytes);
    check!(PolicyJudge, bytes);
    check!(PolicyGuardObserved, bytes);
    check!(PolicyGuard, bytes);
    check!(PolicyRequirement, bytes);
    check!(PolicyPermission, bytes);
    check!(PolicyEditProjectSpend, bytes);
    check!(PolicyEditRoleValue, bytes);
    check!(PolicyEditRequirements, bytes);
    check!(PolicyEditProjections, bytes);
    check!(PolicyEditPermissions, bytes);
    check!(PolicyEdit, bytes);
    check!(HeldRolePolicy, bytes);
    check!(HeldRole, bytes);
    check!(Holding, bytes);
    check!(Message, bytes);
    check!(AnswerQuestion, bytes);
    check!(AnswerPerson, bytes);
    check!(Take, bytes);
    check!(HandBack, bytes);
    check!(DecideProposal, bytes);
    check!(DecideEscalation, bytes);
    check!(Release, bytes);
    check!(Amend, bytes);
    check!(Stop, bytes);
    check!(Cancel, bytes);
    check!(Move, bytes);
    check!(Decide, bytes);
    check!(DecideOn, bytes);
    check!(Answer, bytes);
    check!(AnswerOn, bytes);
    check!(Tracking, bytes);
    check!(TrackingTracked, bytes);
    check!(TemplateCharter, bytes);
    check!(TemplateExecutor, bytes);
    check!(TemplateAmount, bytes);
    check!(TemplateAsAllowed, bytes);
    check!(TemplateBudget, bytes);
    check!(TemplateSpan, bytes);
    check!(TemplateDeadline, bytes);
    check!(TemplateOverride, bytes);
    check!(PolicyEditTemplates, bytes);
    check!(EditNote, bytes);
    check!(DeleteNote, bytes);
    check!(ChangePolicy, bytes);
    check!(SetRoles, bytes);
    check!(MakeService, bytes);
    check!(Rename, bytes);
    check!(MakeProject, bytes);
    check!(SetPool, bytes);
    check!(TaskRefMember, bytes);
    check!(TaskRefTask, bytes);
    check!(ContractReport, bytes);
    check!(ContractVerdict, bytes);
    check!(ContractChange, bytes);
    check!(BudgetAsAllowed, bytes);
    check!(Create, bytes);
    check!(ProposeBatch, bytes);
    check!(ProposeAmendment, bytes);
    check!(ProposeAction, bytes);
    check!(Propose, bytes);
    check!(Withdraw, bytes);
    check!(NewTask, bytes);
    check!(TaskRef, bytes);
    check!(NewContract, bytes);
    check!(NewBudget, bytes);
    check!(NewDeadline, bytes);
    check!(DeadlineSpan, bytes);
    check!(TaskFlags, bytes);
    check!(Retention, bytes);
    check!(CreatedGiven, bytes);
    check!(OutcomeCreated, bytes);
    check!(AskAccountHolder, bytes);
    check!(AskSignIn, bytes);
    check!(AskSignOut, bytes);
    check!(Ask, bytes);
}

fn records_8(bytes: &[u8]) {
    check!(ProposalChoice, bytes);
    check!(EscalationChoice, bytes);
    check!(OutcomeMarkedSeen, bytes);
    check!(OutcomeNoteEdited, bytes);
    check!(OutcomeServiceMade, bytes);
    check!(OutcomeAdopted, bytes);
    check!(OutcomeStopped, bytes);
    check!(OutcomeCancelled, bytes);
    check!(OutcomeReleased, bytes);
    check!(OutcomePersonTaken, bytes);
    check!(OutcomePersonHandedBack, bytes);
    check!(OutcomePersonAnswered, bytes);
    check!(OutcomeMoved, bytes);
    check!(OutcomeProposalDecided, bytes);
    check!(OutcomeSaid, bytes);
    check!(OutcomeQuestionAnswered, bytes);
    check!(OutcomeAmended, bytes);
    check!(OutcomeProposed, bytes);
    check!(OutcomeWithdrawn, bytes);
    check!(OutcomeRolesSet, bytes);
    check!(OutcomePolicyChanged, bytes);
    check!(OutcomeAccountSignInStarted, bytes);
    check!(OutcomeAccountSignedOut, bytes);
    check!(OutcomeRenamed, bytes);
    check!(OutcomeProjectMade, bytes);
    check!(NameRule, bytes);
    check!(MalformedName, bytes);
    check!(OutcomePoolSet, bytes);
    check!(OutcomeEscalationDecided, bytes);
    check!(DocumentOutcome, bytes);
    check!(MadeTasks, bytes);
    check!(MadeService, bytes);
    check!(MadePersonResult, bytes);
    check!(MadeNote, bytes);
    check!(Made, bytes);
    check!(OutcomeDocument, bytes);
    check!(OutcomeMade, bytes);
}

fn records_9(bytes: &[u8]) {
    check!(Outcome, bytes);
    check!(LackingBudget, bytes);
    check!(LackingGrantKind, bytes);
    check!(LackingAgain, bytes);
    check!(LackingRequired, bytes);
    check!(LackingFailed, bytes);
    check!(LackingUnguarded, bytes);
    check!(LackingTaskKind, bytes);
    check!(LackingDepth, bytes);
    check!(LackingDeadline, bytes);
    check!(LackingTasks, bytes);
    check!(Lacking, bytes);
    check!(AdmissionLimit, bytes);
    check!(LimitAdmission, bytes);
    check!(LimitLiveTasks, bytes);
    check!(LimitInbox, bytes);
    check!(LimitWords, bytes);
    check!(Limit, bytes);
    check!(CurrentHoldDocument, bytes);
    check!(CurrentHoldReason, bytes);
    check!(CurrentHold, bytes);
    check!(MovedHold, bytes);
    check!(Moved, bytes);
    check!(RefusalRole, bytes);
    check!(RefusalTaken, bytes);
    check!(RefusalAuthority, bytes);
    check!(RefusalLimit, bytes);
    check!(RefusalMoved, bytes);
    check!(RefusalEnded, bytes);
    check!(Refusal, bytes);
    check!(Done, bytes);
    check!(Refused, bytes);
    check!(Final, bytes);
}

fn records_10(bytes: &[u8]) {
    check!(Cursor, bytes);
    check!(InboxReadGroup, bytes);
    check!(InboxGroup, bytes);
    check!(ReadTask, bytes);
    check!(ReadProposal, bytes);
    check!(ReadEscalation, bytes);
    check!(ReadQuestion, bytes);
    check!(ReadPersonTask, bytes);
    check!(ReadResult, bytes);
    check!(ReadEnded, bytes);
    check!(ReadHistory, bytes);
    check!(ReadActivity, bytes);
    check!(ReadInbox, bytes);
    check!(ReadSpend, bytes);
    check!(ReadProject, bytes);
    check!(ReadNotes, bytes);
    check!(Read, bytes);
    check!(ReadNone, bytes);
    check!(ReadAnswerNone, bytes);
    check!(ReadAnswer, bytes);
    check!(PeriodTimes, bytes);
    check!(FundingRecord, bytes);
    check!(SpendPage, bytes);
}

fn records_11(bytes: &[u8]) {
    check!(MessageProposalKind, bytes);
    check!(NoticeState, bytes);
    check!(NewsClass, bytes);
    check!(WordResultApproved, bytes);
    check!(WordResultVerdict, bytes);
    check!(WordResultChange, bytes);
    check!(WordResult, bytes);
    check!(MessageKindEscalation, bytes);
    check!(MessageKindProposal, bytes);
    check!(MessageKindPersonStalled, bytes);
    check!(MessageKindProposalDecision, bytes);
    check!(MessageKindEffectNotMade, bytes);
    check!(MessageKindAmendment, bytes);
    check!(MessageKindAnswer, bytes);
    check!(MessageKindNotice, bytes);
    check!(MessageKindTimer, bytes);
    check!(MessageKindNews, bytes);
    check!(MessageKindResult, bytes);
    check!(MessageKind, bytes);
    check!(Word, bytes);
    check!(RunWhyMessage, bytes);
    check!(RunWhyRetry, bytes);
    check!(RunWhy, bytes);
    check!(RunEndFailed, bytes);
    check!(RunEnd, bytes);
    check!(ActivityWords, bytes);
    check!(ActivityHeard, bytes);
    check!(ActivityNotified, bytes);
    check!(ActivityRunBegan, bytes);
    check!(ActivityRunEnded, bytes);
    check!(ActivityTurn, bytes);
    check!(ActivityDecided, bytes);
    check!(ActivityMilestone, bytes);
    check!(Activity, bytes);
    check!(ActivityLine, bytes);
    check!(TurnName, bytes);
    check!(TurnCarried, bytes);
    check!(TurnTooLarge, bytes);
    check!(TurnBytes, bytes);
    check!(ArchivedTurn, bytes);
}

fn records_12(bytes: &[u8]) {
    check!(FieldRule, bytes);
    check!(VerdictChoice, bytes);
    check!(TaskContractDocumentApproval, bytes);
    check!(TaskContractDocumentReport, bytes);
    check!(TaskContractDocumentVerdict, bytes);
    check!(TaskContractDocumentChange, bytes);
    check!(TaskContractDocumentJob, bytes);
    check!(TaskContractDocument, bytes);
    check!(TaskStatus, bytes);
    check!(TaskLinkLive, bytes);
    check!(TaskLinkRetained, bytes);
    check!(TaskLinkStored, bytes);
    check!(TaskLink, bytes);
    check!(ResultDetails, bytes);
    check!(ActKind, bytes);
    check!(CreateProcedure, bytes);
    check!(KindImplication, bytes);
    check!(AuthorityDocument, bytes);
    check!(BoundsCreate, bytes);
    check!(BoundsAmend, bytes);
    check!(Bounds, bytes);
    check!(BeyondProposed, bytes);
    check!(BeyondRefused, bytes);
    check!(Beyond, bytes);
    check!(AdmissionProposed, bytes);
    check!(AdmissionBounded, bytes);
    check!(AdmissionRefused, bytes);
    check!(Admission, bytes);
    check!(Offered, bytes);
    check!(ResultDocument, bytes);
    check!(ObjectKeyTask, bytes);
    check!(ObjectKeyProposal, bytes);
    check!(ObjectKeyEscalation, bytes);
    check!(ObjectKeyQuestion, bytes);
    check!(ObjectKeyPersonTask, bytes);
    check!(ObjectKeyResult, bytes);
}

fn records_13(bytes: &[u8]) {
    check!(ObjectKeyNote, bytes);
    check!(ObjectKey, bytes);
    check!(DecodedActivityLine, bytes);
    check!(ActivityPage, bytes);
    check!(SpendLine, bytes);
    check!(Hold, bytes);
    check!(HistoryChange, bytes);
    check!(ParameterDocumentNumber, bytes);
    check!(ParameterDocumentBytes, bytes);
    check!(ParameterDocumentResource, bytes);
    check!(ParameterDocument, bytes);
    check!(DocumentSpecification, bytes);
    check!(SpecificationDocument, bytes);
    check!(ProposedTask, bytes);
    check!(RetainedAmendment, bytes);
    check!(RetainedActionBatch, bytes);
    check!(RetainedActionEffect, bytes);
    check!(RetainedActionProcedureEffect, bytes);
    check!(RetainedActionAmend, bytes);
    check!(RetainedActionWiden, bytes);
    check!(RetainedActionRelease, bytes);
}

fn records_14(bytes: &[u8]) {
    check!(RetainedAction, bytes);
    check!(HistoryHolderTask, bytes);
    check!(HistoryHolderPerson, bytes);
    check!(HistoryHolderPolicy, bytes);
    check!(HistoryHolder, bytes);
    check!(ProposalHistoryStatePending, bytes);
    check!(ProposalHistoryStateAccepted, bytes);
    check!(ProposalHistoryStateRejected, bytes);
    check!(ProposalHistoryState, bytes);
    check!(ProposalHistory, bytes);
    check!(AmendmentState, bytes);
    check!(HistoryAmendment, bytes);
    check!(HistoryLine, bytes);
    check!(MilestonePhaseActive, bytes);
    check!(MilestonePhaseClosing, bytes);
    check!(MilestonePhaseHeld, bytes);
    check!(MilestonePhaseEnded, bytes);
    check!(MilestonePhase, bytes);
    check!(MilestoneLine, bytes);
    check!(HistoryRowChange, bytes);
    check!(HistoryRowMilestone, bytes);
    check!(HistoryRow, bytes);
    check!(HistoryPage, bytes);
    check!(RuntimeActive, bytes);
    check!(RuntimeRunning, bytes);
    check!(RuntimeWaitingForWords, bytes);
    check!(RuntimeBackingOff, bytes);
    check!(RuntimePerson, bytes);
    check!(RuntimeClosing, bytes);
    check!(RuntimeHeld, bytes);
    check!(Runtime, bytes);
    check!(Count, bytes);
}

fn records_15(bytes: &[u8]) {
    check!(PhaseCounts, bytes);
    check!(NoteAuthorParty, bytes);
    check!(NoteAuthorTask, bytes);
    check!(NoteAuthor, bytes);
    check!(Note, bytes);
    check!(NotePage, bytes);
    check!(ObjectPage, bytes);
    check!(EndedDelegate, bytes);
    check!(EndedPage, bytes);
    check!(InboxObjectQuestion, bytes);
    check!(InboxObjectProposal, bytes);
    check!(InboxObjectEscalation, bytes);
    check!(InboxObjectTurn, bytes);
    check!(InboxObjectResult, bytes);
    check!(InboxObject, bytes);
    check!(InboxCursorNumber, bytes);
    check!(InboxCursorInbox, bytes);
    check!(InboxCursor, bytes);
    check!(InboxSelection, bytes);
    check!(InboxEntry, bytes);
    check!(InboxPage, bytes);
}

fn decoders(bytes: &[u8]) {
    documents::decoders(bytes);
    records_1(bytes);
    records_2(bytes);
    records_3(bytes);
    records_4(bytes);
    records_5(bytes);
    records_6(bytes);
    records_7(bytes);
    records_8(bytes);
    records_9(bytes);
    records_10(bytes);
    records_11(bytes);
    records_12(bytes);
    records_13(bytes);
    records_14(bytes);
    records_15(bytes);
}

fn arbitrary(rng: &mut Rng) {
    let mut buffer = [0; 256];
    for _ in 0..2000 {
        let length = usize::try_from(rng.below(257)).expect("bounded length");
        for byte in buffer.get_mut(..length).expect("within buffer") {
            *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
        }
        decoders(buffer.get(..length).expect("within buffer"));
    }
}

fn mutation_seed(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with("_full.bin")
        && [
            "record_charter_",
            "record_keyed_",
            "record_owned_",
            "record_refused_",
            "enum_ask_",
            "record_rename_",
            "record_make_project_",
            "record_outcome_renamed_",
            "record_outcome_project_made_",
            "record_malformed_name_",
            "enum_document_outcome_renamed_",
            "enum_document_outcome_project_made_",
            "enum_refusal_name_taken_",
            "enum_malformed_name_",
            "enum_name_rule_",
            "enum_admission_limit_projects_",
            "enum_act_kind_rename_",
            "enum_final_",
            "enum_fact_",
            "record_hold_reason_",
            "enum_read_",
            "enum_read_answer_",
            "enum_read_value_",
            "enum_delivery_value_",
            "record_chips_",
            "record_inbox_",
            "record_stream_text_",
            "record_spend_page_",
            "record_activity_page_",
            "record_history_page_",
            "record_inbox_page_",
            "record_ended_",
            "enum_goal_holding_",
            "record_person_proposal_goal_",
            "record_project_",
            "record_deployment_",
            "record_account_",
            "record_deployment_",
            "record_open_",
            "record_authorize_",
            "enum_authorize_",
            "record_hello_",
            "enum_stream_event_",
            "record_run_state_",
            "record_task_",
        ]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn mutated(rng: &mut Rng) {
    let root = skein_scratch::paths::tree().join("crates/jig-client-protocol-wire/golden");
    let mut paths: Vec<_> =
        fs::read_dir(root).expect("golden directory").map(|entry| entry.expect("golden entry").path()).collect();
    paths.sort();
    for path in paths {
        if !mutation_seed(&path) {
            continue;
        }
        let golden = fs::read(path).expect("golden bytes");
        decoders(&golden);
        for _ in 0..1 {
            let mut bytes = golden.clone();
            if !bytes.is_empty() {
                let index =
                    usize::try_from(rng.below(u64::try_from(bytes.len()).expect("length fits"))).expect("index fits");
                *bytes.get_mut(index).expect("chosen byte") = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            }
            decoders(&bytes);
        }
    }
}

#[test]
fn client_wire_decoders_never_panic_and_reencode_the_same() {
    let mut rng = Rng::new(0xc11e_0030);
    arbitrary(&mut rng);
    mutated(&mut rng);
}
