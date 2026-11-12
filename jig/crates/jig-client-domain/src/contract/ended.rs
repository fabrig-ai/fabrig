//! Whole ended object facts (client/domain/contract/README.md, 4;
//! client/domain/client/README.md, 5). These passive values supply no current offers.
//! Raw person goals are admitted with recurrence absent at both entry and restore;
//! their boundary translator must refuse an unadmitted recurrence before copying.
use super::{
    Activity, ActivityLine, Authority, Bytes, Executor, Funder, MessageKind, MessageProposalKind, MilestoneLine,
    Numbers, ObjectKey, PersonTask, ProposalChoice, ProposalHistory, ProposalHistoryState, ProposalKey, Proposer,
    Question, Requester, Result, Specification, Task, TaskContractDocument, Text, WakePolicy,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// A raw goal holding retains kind and literal segment boundaries.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GoalHolding {
    Write { connector: u16, path: List<Bytes>, kind: u16 },
    Slot { connector: u16, path: List<Bytes>, kind: u16 },
}
/// Whole admitted New facts, not a clipped future-task document projection.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonProposalGoal {
    pub number: u64,
    pub project: u32,
    pub executor: Executor,
    pub spec: Specification,
    pub contract: TaskContractDocument,
    pub authority: Authority,
    pub numbers: Numbers,
    pub funder: Funder,
    pub dependencies: List<u64>,
    pub holdings: List<GoalHolding>,
    pub wake: WakePolicy,
    pub tracked: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonProposal {
    pub number: u64,
    pub proposer: u64,
    pub project: u32,
    pub goal: PersonProposalGoal,
    pub state: PersonProposalState,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PersonProposalState {
    Pending { since: Wall },
    Accepted { by: Requester },
    Rejected { by: Requester, reason: Text, message: u64, at: Wall },
}
/// The independently retained decision, including the inclusive created pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProposalDecision {
    pub project: u32,
    pub proposer: Proposer,
    pub proposal: u64,
    pub kind: MessageProposalKind,
    pub by: u64,
    pub at: Wall,
    pub choice: ProposalChoice,
    pub created: Option<(u64, u64)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EscalationDecision {
    Release,
    Reject { reason: Text },
    Pass,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EscalationDecisionRecord {
    pub project: u32,
    pub requester: u64,
    pub task: u64,
    pub revision: u64,
    pub by: u64,
    pub at: Wall,
    pub decision: EscalationDecision,
}
/// Each case keeps its complete sources independently, without invented context.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndedValue {
    Task { document: Box<Task>, result: Box<Result> },
    Proposal(ProposalHistory),
    PersonProposal { proposal: Box<PersonProposal>, decision: Option<ProposalDecision> },
    Escalation { task: Box<Task>, decision: EscalationDecisionRecord, hold: Option<MilestoneLine> },
    Question { question: Question, answer: Option<ActivityLine>, ending: Option<Box<Result>> },
    PersonTask { document: Box<PersonTask>, result: Box<Result> },
}
/// Key is the original native query identity, validated against the nested facts.
/// Outer clock and sender are independent of all inner clocks and senders.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ended {
    pub key: ObjectKey,
    pub value: EndedValue,
    pub at: Option<Wall>,
    pub by: Option<Requester>,
}
impl Ended {
    /// Check original query, envelope and nested identities before accepting a read.
    /// Answer order is authenticated by the source translator: Question does not
    /// supply the original activity position, so the receiver cannot reconstruct it.
    #[must_use]
    pub fn matches(&self, original: ObjectKey) -> bool {
        if original != self.key {
            return false;
        }
        match &self.value {
            EndedValue::Task { document, result } => {
                self.key == ObjectKey::Task(document.chip.task) && result.task.task == document.chip.task
            }
            EndedValue::Proposal(proposal) => {
                let proposer = match proposal.action {
                    super::RetainedAction::Settings(_) => {
                        return self.key
                            == ObjectKey::Proposal(ProposalKey {
                                proposer: Proposer::Party(proposal.proposer),
                                number: proposal.number,
                            })
                            || self.key
                                == ObjectKey::Proposal(ProposalKey {
                                    proposer: Proposer::Task(proposal.proposer),
                                    number: proposal.number,
                                });
                    }
                    super::RetainedAction::Batch(_)
                    | super::RetainedAction::Effect { .. }
                    | super::RetainedAction::ProcedureEffect { .. }
                    | super::RetainedAction::Amend { .. }
                    | super::RetainedAction::Widen { .. }
                    | super::RetainedAction::Release { .. } => Proposer::Task(proposal.proposer),
                };
                self.key == ObjectKey::Proposal(ProposalKey { proposer, number: proposal.number })
            }
            EndedValue::PersonProposal { proposal, decision } => {
                let key = ProposalKey { proposer: Proposer::Party(proposal.proposer), number: proposal.number };
                let record_matches = match decision {
                    Some(record) => {
                        record.project == proposal.project
                            && record.proposer == key.proposer
                            && record.proposal == key.number
                    }
                    None => match &proposal.state {
                        PersonProposalState::Accepted { .. } => false,
                        PersonProposalState::Pending { .. } | PersonProposalState::Rejected { .. } => true,
                    },
                };
                self.key == ObjectKey::Proposal(key) && proposal.goal.project == proposal.project && record_matches
            }
            EndedValue::Escalation { task, decision, hold } => {
                let hold_matches = match hold {
                    Some(hold) => hold.task == task.chip.task,
                    None => true,
                };
                self.key == ObjectKey::Escalation(task.chip.task)
                    && decision.task == task.chip.task
                    && decision.project == task.project
                    && hold_matches
            }
            EndedValue::Question { question, answer, ending } => {
                let answer_matches = match answer {
                    Some(answer) => answer.task == question.task.task && answers_question(answer, question.message),
                    None => true,
                };
                let ending_matches = match ending {
                    Some(ending) => ending.task.task == question.task.task,
                    None => true,
                };
                self.key == ObjectKey::Question { task: question.task.task, message: question.message }
                    && answer_matches
                    && ending_matches
            }
            EndedValue::PersonTask { document, result } => {
                self.key == ObjectKey::PersonTask(document.task.task) && result.task.task == document.task.task
            }
        }
    }
    /// Classification of supplied facts, not a retention duration or current act.
    /// Pending source-less objects still follow ordinary prune after read receipt.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        match &self.value {
            EndedValue::Task { .. } | EndedValue::PersonTask { .. } | EndedValue::Escalation { .. } => true,
            EndedValue::Proposal(proposal) => match &proposal.state {
                ProposalHistoryState::Pending { .. } => false,
                ProposalHistoryState::Accepted { .. }
                | ProposalHistoryState::Rejected { .. }
                | ProposalHistoryState::Withdrawn => true,
            },
            EndedValue::PersonProposal { proposal, .. } => match &proposal.state {
                PersonProposalState::Pending { .. } => false,
                PersonProposalState::Accepted { .. } | PersonProposalState::Rejected { .. } => true,
            },
            EndedValue::Question { answer, ending, .. } => answer.is_some() || ending.is_some(),
        }
    }
}
fn answers_question(answer: &ActivityLine, message: u64) -> bool {
    match &answer.line {
        Activity::Words { kind, .. } => match kind {
            MessageKind::Answer { question } => *question == message,
            MessageKind::Escalation { .. }
            | MessageKind::Proposal { .. }
            | MessageKind::PersonStalled { .. }
            | MessageKind::SettingsDecision { .. }
            | MessageKind::ProposalDecision { .. }
            | MessageKind::Words
            | MessageKind::EffectNotMade { .. }
            | MessageKind::Met { .. }
            | MessageKind::QuestionAnswered { .. }
            | MessageKind::QuestionOwed
            | MessageKind::NothingAwaited
            | MessageKind::QuestionEnded { .. }
            | MessageKind::Amendment { .. }
            | MessageKind::Question
            | MessageKind::Notice { .. }
            | MessageKind::Timer { .. }
            | MessageKind::News { .. }
            | MessageKind::Result(_) => false,
        },
        Activity::Heard(_)
        | Activity::Notified { .. }
        | Activity::RunBegan { .. }
        | Activity::RunEnded { .. }
        | Activity::Turn { .. }
        | Activity::Decided { .. }
        | Activity::NowWritten { .. }
        | Activity::Stood { .. }
        | Activity::TalkOpened { .. }
        | Activity::TalkEnded { .. }
        | Activity::SessionEnded { .. }
        | Activity::Kept { .. }
        | Activity::Milestone { .. } => false,
    }
}
