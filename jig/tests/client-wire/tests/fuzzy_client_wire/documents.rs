//! Decoder cases added with documents and stream exchanges.
use super::*;

fn records_1(bytes: &[u8]) {
    check!(PoolLedger, bytes);
    check!(MeParty, bytes);
    check!(Needs, bytes);
    check!(MeProject, bytes);
    check!(MeDefault, bytes);
    check!(Me, bytes);
    check!(ImpactCounts, bytes);
    check!(Impact, bytes);
    check!(LiveDelegate, bytes);
    check!(TaskContext, bytes);
    check!(TaskResultDocument, bytes);
    check!(TaskResultEnding, bytes);
    check!(TaskResult, bytes);
    check!(SubscriptionKindTask, bytes);
    check!(SubscriptionKindTimer, bytes);
    check!(SubscriptionKindTopic, bytes);
    check!(SubscriptionKind, bytes);
    check!(Subscription, bytes);
    check!(Subscriptions, bytes);
    check!(Task, bytes);
    check!(ReadPosition, bytes);
    check!(ExecutorKind, bytes);
    check!(TaskQuery, bytes);
    check!(WaitsOn, bytes);
    check!(ConversationRead, bytes);
    check!(TaskLine, bytes);
    check!(Tasks, bytes);
}

fn records_2(bytes: &[u8]) {
    check!(Call, bytes);
    check!(CurrentCall, bytes);
    check!(LiveCallDecoded, bytes);
    check!(LiveCallRaw, bytes);
    check!(LiveCall, bytes);
    check!(ProvisionalTurnName, bytes);
    check!(RunText, bytes);
    check!(RunState, bytes);
    check!(ProposalActionBatch, bytes);
    check!(ProposalActionEffect, bytes);
    check!(ProposalActionProcedureEffect, bytes);
    check!(ProposalActionAmend, bytes);
    check!(ProposalActionWiden, bytes);
    check!(ProposalActionRelease, bytes);
    check!(ProposalAction, bytes);
    check!(ProposalWaiting, bytes);
    check!(FundingOffer, bytes);
    check!(Proposal, bytes);
    check!(Escalation, bytes);
    check!(Question, bytes);
    check!(OptionLine, bytes);
    check!(PersonContractChoice, bytes);
    check!(PersonContract, bytes);
    check!(ShownResult, bytes);
    check!(PersonTask, bytes);
    check!(RecurringOverlap, bytes);
    check!(Recurring, bytes);
}

fn records_3(bytes: &[u8]) {
    check!(ProjectResourceRole, bytes);
    check!(ProjectResource, bytes);
    check!(ProjectPolicyRole, bytes);
    check!(ProjectPolicy, bytes);
    check!(ProjectMember, bytes);
    check!(ProjectSpend, bytes);
    check!(Project, bytes);
    check!(EndedValueTask, bytes);
    check!(EndedValueProposal, bytes);
    check!(GoalHoldingWrite, bytes);
    check!(GoalHoldingSlot, bytes);
    check!(GoalHolding, bytes);
    check!(PersonProposalGoal, bytes);
    check!(PersonProposalStatePending, bytes);
    check!(PersonProposalStateAccepted, bytes);
    check!(PersonProposalStateRejected, bytes);
    check!(PersonProposalState, bytes);
    check!(PersonProposal, bytes);
    check!(ProposalDecisionCreated, bytes);
    check!(ProposalDecision, bytes);
    check!(EndedValuePersonProposal, bytes);
    check!(EscalationDecisionReject, bytes);
    check!(EscalationDecision, bytes);
    check!(EscalationDecisionRecord, bytes);
    check!(EndedValueEscalation, bytes);
    check!(EndedValueQuestion, bytes);
    check!(EndedValuePersonTask, bytes);
    check!(EndedValue, bytes);
    check!(Ended, bytes);
    check!(TreeRow, bytes);
    check!(TreeFold, bytes);
    check!(Tree, bytes);
    check!(EntranceRefusal, bytes);
    check!(SubjectTask, bytes);
    check!(SubjectTree, bytes);
    check!(SubjectChips, bytes);
    check!(SubjectInbox, bytes);
    check!(SubjectTasks, bytes);
    check!(SubjectRun, bytes);
}

fn records_4(bytes: &[u8]) {
    check!(Subject, bytes);
    check!(StoreFill, bytes);
    check!(StoreCensus, bytes);
    check!(CensusKeyspace, bytes);
    check!(CensusHead, bytes);
    check!(Capacity, bytes);
    check!(ProjectCapacity, bytes);
    check!(HostKind, bytes);
    check!(AttemptState, bytes);
    check!(RunLocationConnected, bytes);
    check!(RunLocationAdrift, bytes);
    check!(RunLocation, bytes);
    check!(HostRun, bytes);
    check!(HostPerson, bytes);
    check!(HostRule, bytes);
    check!(Host, bytes);
    check!(AgentShortfallModel, bytes);
    check!(AgentShortfallWindow, bytes);
    check!(AgentShortfallOutput, bytes);
    check!(AgentShortfall, bytes);
    check!(HelloRefusalAgent, bytes);
    check!(HelloRefusalWorkers, bytes);
    check!(HelloRefusalPosition, bytes);
    check!(HelloRefusalStopBound, bytes);
    check!(HelloRefusal, bytes);
    check!(HelloRefusalLine, bytes);
    check!(RequestKey, bytes);
    check!(PersonCapacityActionApplication, bytes);
    check!(PersonCapacityActionNote, bytes);
    check!(PersonCapacityActionPool, bytes);
    check!(PersonCapacityActionEscalation, bytes);
    check!(PersonCapacityActionProposal, bytes);
    check!(PersonCapacityActionWords, bytes);
    check!(PersonCapacityActionQuestion, bytes);
    check!(PersonCapacityActionAmend, bytes);
    check!(PersonCapacityActionMove, bytes);
    check!(PersonCapacityActionTakePerson, bytes);
    check!(PersonCapacityActionHandBackPerson, bytes);
    check!(PersonCapacityActionAnswerPerson, bytes);
    check!(PersonCapacityActionStop, bytes);
    check!(PersonCapacityActionCancel, bytes);
    check!(PersonCapacityActionRelease, bytes);
}

fn records_5(bytes: &[u8]) {
    check!(PersonCapacityActionMarkSeen, bytes);
    check!(PersonCapacityAction, bytes);
    check!(PersonCapacityCauseRequests, bytes);
    check!(PersonCapacityCausePending, bytes);
    check!(PersonCapacityCauseWaiters, bytes);
    check!(PersonCapacityCauseLimit, bytes);
    check!(PersonCapacityCause, bytes);
    check!(PersonCapacityRefusal, bytes);
    check!(TaskCapacityActionBatch, bytes);
    check!(TaskCapacityActionProposal, bytes);
    check!(TaskCapacityActionPersonProposal, bytes);
    check!(TaskCapacityActionMessage, bytes);
    check!(TaskCapacityActionSource, bytes);
    check!(TaskCapacityActionIntroduce, bytes);
    check!(TaskCapacityActionSubscription, bytes);
    check!(TaskCapacityActionEffect, bytes);
    check!(TaskCapacityActionClaim, bytes);
    check!(TaskCapacityAction, bytes);
    check!(TaskCapacityCauseTasks, bytes);
    check!(TaskCapacityCauseCreationFlights, bytes);
    check!(TaskCapacityCauseResourceReports, bytes);
    check!(TaskCapacityCauseProjectTasks, bytes);
    check!(TaskCapacityCauseTree, bytes);
    check!(TaskCapacityCauseDelegates, bytes);
    check!(TaskCapacityCauseInbox, bytes);
    check!(TaskCapacityCauseHoldings, bytes);
    check!(TaskCapacityCauseHoldWaiters, bytes);
    check!(TaskCapacityCauseProposals, bytes);
    check!(TaskCapacityCauseReferences, bytes);
    check!(TaskCapacityCauseSubscriptions, bytes);
    check!(TaskCapacityCauseFunders, bytes);
    check!(TaskCapacityCauseWriters, bytes);
    check!(TaskCapacityCause, bytes);
    check!(TaskCapacityRequest, bytes);
    check!(TaskCapacityRefusal, bytes);
    check!(WatchEnd, bytes);
    check!(WatchRefusal, bytes);
    check!(Open, bytes);
    check!(OpenAnswerRefused, bytes);
    check!(OpenAnswer, bytes);
    check!(WatchClose, bytes);
}

fn records_6(bytes: &[u8]) {
    check!(CloseAnswer, bytes);
    check!(Gone, bytes);
    check!(Served, bytes);
    check!(Hello, bytes);
    check!(Delivery, bytes);
    check!(Ended, bytes);
    check!(StreamEvent, bytes);
}

/// Valid reports exercise the bounds that arbitrary bytes almost never reach.
fn generated_census(rng: &mut Rng, rows: u32) -> StoreCensus {
    let mut keyspaces = skein_lib::List::with_capacity(rows);
    for _ in 0..rows {
        let full = rows == 129;
        let prefix_len = if full { 8 } else { usize::try_from(rng.below(9)).expect("prefix bound") };
        let name_len = if full { 48 } else { usize::try_from(rng.below(49)).expect("name bound") };
        let mut prefix = vec![0; prefix_len];
        for byte in &mut prefix {
            *byte = u8::try_from(rng.below(256)).expect("byte");
        }
        let name = vec![b'n'; name_len].into_boxed_slice();
        let head_count = if full { 8 } else { u32::try_from(rng.below(9)).expect("head bound") };
        let mut heads = skein_lib::List::with_capacity(head_count);
        for h in 0..head_count {
            heads.push(CensusHead::new(&CEILINGS, CensusHeadParts {
                head: if h == 0 { None } else { Some(u16::try_from(h).expect("head")) },
                rows: rng.next_u64(),
            }).expect("bounded head")).expect("head capacity");
        }
        keyspaces.push(CensusKeyspace::new(&CEILINGS, CensusKeyspaceParts {
            prefix: prefix.into_boxed_slice(), name,
            rows: rng.next_u64(), keys: rng.next_u64(), values: rng.next_u64(), heads,
            puts: rng.next_u64(), erases: rng.next_u64(), written: rng.next_u64(),
        }).expect("bounded keyspace")).expect("row capacity");
    }
    StoreCensus::new(&CEILINGS, StoreCensusParts {
        commit: rng.next_u64(), since: rng.next_u64(), keyspaces,
    }).expect("bounded report")
}

#[test]
fn generated_deployment_censuses_round_trip_through_all_row_counts_and_full_fields() {
    let mut rng = Rng::new(0xc11e_ce22);
    let mut empty = jig_client_test_records::deployment_rows_record().into_parts();
    empty.census = None;
    let empty = Deployment::new(&CEILINGS, empty).expect("absent report");
    let mut writer = Writer::new(usize::try_from(empty.measure()).expect("measure"));
    empty.encode(&mut writer).expect("measured room");
    let bytes = writer.finish();
    assert_eq!(Deployment::decode(&CEILINGS, &mut Reader::new(&bytes)), Ok(empty.clone()));
    for rows in 0..=129 {
        let census = generated_census(&mut rng, rows);
        let mut writer = Writer::new(usize::try_from(census.measure()).expect("measure"));
        census.encode(&mut writer).expect("measured room");
        let bytes = writer.finish();
        assert_eq!(StoreCensus::decode(&CEILINGS, &mut Reader::new(&bytes)), Ok(census.clone()));
        let mut parts = empty.clone().into_parts();
        parts.census = Some(census);
        let deployment = Deployment::new(&CEILINGS, parts).expect("present report");
        let mut writer = Writer::new(usize::try_from(deployment.measure()).expect("measure"));
        deployment.encode(&mut writer).expect("measured room");
        let bytes = writer.finish();
        assert_eq!(Deployment::decode(&CEILINGS, &mut Reader::new(&bytes)), Ok(deployment));
    }
}

pub fn decoders(bytes: &[u8]) {
    account_records(bytes);
    records_1(bytes);
    records_2(bytes);
    records_3(bytes);
    records_4(bytes);
    records_5(bytes);
    records_6(bytes);
}

fn account_records(bytes: &[u8]) {
    check!(AccountStateLater, bytes);
    check!(AccountPlace, bytes);
    check!(AccountUnread, bytes);
    check!(AccountStateUnloaded, bytes);
    check!(AccountState, bytes);
    check!(Account, bytes);
    check!(AccountUseParty, bytes);
    check!(AccountUseProject, bytes);
    check!(AccountUse, bytes);
    check!(AccountHolderParty, bytes);
    check!(AccountHolder, bytes);
    check!(AccountKind, bytes);
    check!(AccountBound, bytes);
    check!(AccountKeyFailure, bytes);
    check!(AccountRefusalNotKept, bytes);
    check!(AccountRefusalLimit, bytes);
    check!(AccountRefusalHeld, bytes);
    check!(AccountRefusalProvider, bytes);
    check!(AccountRefusalKind, bytes);
    check!(AccountRefusal, bytes);
    check!(AccountFailureLater, bytes);
    check!(AccountFailureMissingScope, bytes);
    check!(AccountFailureUnloaded, bytes);
    check!(AccountFailure, bytes);
    check!(AccountEndMade, bytes);
    check!(AccountEndRenewed, bytes);
    check!(AccountEndRefused, bytes);
    check!(AccountEndFailed, bytes);
    check!(AccountEnd, bytes);
    check!(AccountSource, bytes);
    check!(AccountLine, bytes);
    check!(AccountSignIn, bytes);
    check!(AccountLastEnd, bytes);
    check!(Accounts, bytes);

    check!(Deployment, bytes);
}
