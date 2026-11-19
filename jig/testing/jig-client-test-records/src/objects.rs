//! Independent wire literals for single-read documents (testing-strategy.md, 8).
use super::{List, Writer, fact_bytes, owner_waiting_chip_record, task_with_hold_record, w, wire_text};
#[must_use]
pub fn pending_proposal_record() -> w::Proposal {
    let mut funding = List::with_capacity(2);
    for (source, needed, left, after) in [
        (
            w::Funder::Task(w::FunderTask { value: 17 }),
            19,
            None,
            Some(23),
        ),
        (
            w::Funder::Period(
                w::FunderPeriod { project: 7, period: 29 },
            ),
            31,
            Some(0),
            None,
        ),
    ] {
        funding
            .push(
                w::FundingOffer { source, needed, left, after },
            )
            .expect("two offers");
    }
    w::Proposal {
            projects: List::with_capacity(0),
            key: w::ProposalKey {
                    proposer: w::Proposer::Party(
                        w::ProposerParty { value: 11 },
                    ),
                    number: 13,
                },
            project: 7,
            reason: wire_text(b"pending"),
            action: w::ProposalAction::Release(
                w::ProposalActionRelease {
                        task: w::TaskLink::Stored(
                            w::TaskLinkStored { number: 37, status: None },
                        ),
                        hold: None,
                    },
            ),
            as_holder: true,
            waiting: w::ProposalWaiting {
                    holder: w::HistoryHolder::Policy(
                        w::HistoryHolderPolicy { project: 7, kind: w::MessageProposalKind::Amend },
                    ),
                    since: 41,
                },
            funding,
            acts: Box::from([]),
        }
}
#[must_use]
pub fn failed_result_record() -> w::ResultDocument {
    let root = w::Fact::Ending(
        w::EndingRow {
                parent: None,
                value: w::EndingFields::Task(
                    w::EndingTaskFields { value: 1 },
                ),
            },
    );
    let child = w::Fact::ChipEnding(
        w::ChipEndingRow {
                parent: Some(0),
                value: w::ChipEndingFields::Failed(
                    w::ChipEndingFailedFields { value: wire_text(b"failed") },
                ),
            },
    );
    let first = fact_bytes(&root);
    let second = fact_bytes(&child);
    let mut rows = Writer::new(first.len().checked_add(second.len()).expect("literal ending size"));
    rows.put(&first).expect("first fact");
    rows.put(&second).expect("second fact");
    w::ResultDocument {
            task: owner_waiting_chip_record(),
            ending: w::Ending { rows: rows.finish() },
            value: None,
            at: Some(47),
            details: Some(
                w::ResultDetails {
                        contract: w::TaskContractDocument::Report(
                            w::TaskContractDocumentReport { words: 53 },
                        ),
                        requester: w::Requester::Person(
                            w::RequesterPerson { value: 59 },
                        ),
                        executor: w::Executor::Agent(
                            w::ExecutorAgent { charter: 7 },
                        ),
                        chosen_by: Some(61),
                        delegates: Box::from([]),
                        followups: Some(Box::from([])),
                        proposal: None,
                        cut: true,
                    },
            ),
            acts: Box::from([]),
        }
}
#[must_use]
pub fn ended_task_record() -> w::Ended {
    w::Ended {
            key: w::ObjectKey::Task(
                w::ObjectKeyTask { value: 7 },
            ),
            value: w::EndedValue::Task(
                w::EndedValueTask { document: task_with_hold_record(), result: failed_result_record() },
            ),
            at: Some(67),
            by: Some(w::Requester::Person(
                w::RequesterPerson { value: 71 },
            )),
        }
}
#[must_use]
pub fn decided_escalation_record(decision: w::EscalationDecision) -> w::Ended {
    w::Ended {
            key: w::ObjectKey::Escalation(
                w::ObjectKeyEscalation { value: 7 },
            ),
            value: w::EndedValue::Escalation(
                w::EndedValueEscalation {
                        task: task_with_hold_record(),
                        decision: w::EscalationDecisionRecord {
                                project: 7,
                                requester: 73,
                                task: 7,
                                revision: 79,
                                by: 83,
                                at: 89,
                                decision,
                            },
                        hold: None,
                    },
            ),
            at: Some(97),
            by: Some(w::Requester::Person(
                w::RequesterPerson { value: 101 },
            )),
        }
}
#[must_use]
pub fn owner_project_record() -> w::Project {
    w::Project {
            acts: Box::from([]),
            number: 7,
            name: w::DocumentText { bytes: Box::from(*b"Tally"), cut: false },
            resources: Box::from([]),
            policy: w::ProjectPolicy {
                    templates: List::with_capacity(0),
                    ceiling: w::AuthorityDocument {
                            cut: false,
                            authority: w::Authority {
                                    models: Some(w::ModelSet::All),
                                    tools: 7,
                                    grants: Box::from([]),
                                    delegation: w::Delegation { kinds: List::with_capacity(0), tasks: 7, depth: 7 },
                                    spend: 7,
                                    deadline: None,
                                    notes: 7,
                                    note_resources: Box::from([]),
                                },
                        },
                    period_spend: 7,
                    escalation_role: None,
                    projections: Box::from([]),
                    roles: Box::from([]),
                    requirements: Box::from([]),
                    permissions: None,
                    permissions_cut: false,
                    cut: false,
                },
            members: List::with_capacity(0),
            current_period: None,
            pools: List::with_capacity(0),
            configured_period: 7,
            configured_pools: List::with_capacity(0),
            configured_pools_cut: false,
            spend: Box::from([]),
            recurring: Box::from([]),
            cut: false,
        }
}
