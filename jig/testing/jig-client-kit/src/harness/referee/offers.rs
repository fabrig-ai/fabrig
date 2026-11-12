//! Offers kept, from delivered documents, visible builds and keyed answers (testing.md, 6.3).
use super::{ClientId, JudgeOf};
use crate::harness::DraftOutcome;
use jig_client::{
    Key,
    contract::{
        Answer, Ask, Creating, DocumentOutcome, HistoryHolder, Lacking, MessageProposalKind, NewBudget, Outcome,
        ProposalKey, ProposeAction, Proposer, ReadValue, Recipient, Refusal,
    },
    shapes::{self, Fall, GoalDraft},
};
use skein_lib::Token;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Default)]
pub(crate) struct Offers {
    latest: BTreeMap<(ClientId, u32), Creating>,
    shown: BTreeMap<(ClientId, u32), Creating>,
    drafts: BTreeMap<(ClientId, u64), Fall>,
    as_allowed: BTreeSet<(ClientId, u64)>,
    recipients: BTreeMap<ProposalKey, Recipient>,
    proposed: BTreeMap<ProposalKey, Vec<Recipient>>,
    parties: BTreeMap<ClientId, u64>,
    holders: BTreeMap<ProposalKey, HistoryHolder>,
    roles: BTreeMap<u32, Vec<(u32, u8)>>,
}
impl Offers {
    pub(super) fn shown(&mut self, client: ClientId) {
        self.shown.retain(|(owner, _), _| *owner != client);
        for (key, value) in &self.latest {
            if key.0 == client {
                self.shown.insert(*key, value.clone());
            }
        }
    }
    pub(super) fn draft(&mut self, client: ClientId, key: Key, offer: Fall) {
        self.drafts.entry((client, key.number)).or_insert(offer);
    }
    pub(super) fn delivered(&mut self, client: ClientId, value: &ReadValue, judge: &mut JudgeOf) {
        match value {
            ReadValue::Me(me) => {
                self.parties.insert(client, me.party.number);
                for project in &me.projects {
                    if let Some(creating) = &project.creating {
                        self.latest.insert((client, project.project), creating.clone());
                    }
                }
            }
            ReadValue::Project(project) => {
                self.roles.insert(
                    project.number,
                    project.policy.roles.iter().map(|role| (role.number, role.decides)).collect(),
                );
                let waiting: Vec<_> = self.holders.iter().map(|(key, holder)| (*key, *holder)).collect();
                for (key, holder) in waiting {
                    self.owner_recipient(key, holder, judge);
                }
            }
            ReadValue::Proposal(proposal) => self.owner_recipient(proposal.key, proposal.waiting.holder, judge),
            ReadValue::Tasks(_)
            | ReadValue::EndedPage(_)
            | ReadValue::Task(_)
            | ReadValue::Escalation(_)
            | ReadValue::Question(_)
            | ReadValue::PersonTask(_)
            | ReadValue::Result(_)
            | ReadValue::Ended(_)
            | ReadValue::History(_)
            | ReadValue::Activity(_)
            | ReadValue::Turn(_)
            | ReadValue::Inbox(_)
            | ReadValue::Spend(_)
            | ReadValue::Notes(_)
            | ReadValue::Settings(_)
            | ReadValue::Setting(_)
            | ReadValue::Revision(_)
            | ReadValue::TaskText(_)
            | ReadValue::Revisions(_)
            | ReadValue::Deployment(_) => {}
        }
    }
    fn recipient(&mut self, key: ProposalKey, to: &Recipient, judge: &mut JudgeOf) {
        self.recipients.insert(key, to.clone());
        if let Some(wanted) = self.proposed.remove(&key) {
            for wanted in wanted {
                judge.check(wanted == *to, "offers kept: proposed to a different recipient");
            }
        }
    }
    fn owner_recipient(&mut self, key: ProposalKey, holder: HistoryHolder, judge: &mut JudgeOf) {
        self.holders.insert(key, holder);
        let to = match holder {
            HistoryHolder::Task(task) => Recipient::Task(task),
            HistoryHolder::Person(person) => Recipient::Party(person),
            HistoryHolder::Policy { project, kind } => {
                let Some(roles) = self.roles.get(&project) else {
                    return;
                };
                let bit = match kind {
                    MessageProposalKind::Settings => return,
                    MessageProposalKind::Batch => 1,
                    MessageProposalKind::Effect => 2,
                    MessageProposalKind::Widen => 4,
                    MessageProposalKind::Amend => 8,
                    MessageProposalKind::Release => 16,
                };
                Recipient::Roles {
                    project,
                    roles: crate::test::list(
                        roles.iter().filter(|(_, decides)| decides & bit != 0).map(|(number, _)| *number),
                    ),
                }
            }
        };
        self.recipient(key, &to, judge);
    }
    pub(super) fn sent(&mut self, client: ClientId, key: Key, ask: &Ask, judge: &mut JudgeOf) -> Option<Fall> {
        let (project, draft, deadline, budget) = match ask {
            Ask::Create { project, tasks } | Ask::Propose { project, action: ProposeAction::Batch { tasks }, .. }
                if tasks.len() == 1 =>
            {
                let task = &tasks.as_slice()[0];
                (
                    *project,
                    GoalDraft {
                        words: task.spec.words.clone(),
                        executor: task.executor,
                        budget: task.authority.spend,
                        deadline: task.authority.deadline,
                        priority: task.tracked,
                        llm: task.llm.clone(),
                    },
                    task.deadline,
                    &task.budget,
                )
            }
            Ask::Create { .. }
            | Ask::Propose { .. }
            | Ask::Withdraw { .. }
            | Ask::Talk { .. }
            | Ask::Leave { .. }
            | Ask::Message { .. }
            | Ask::Compound { .. }
            | Ask::Answer { .. }
            | Ask::Take { .. }
            | Ask::HandBack { .. }
            | Ask::Decide { .. }
            | Ask::Release { .. }
            | Ask::Amend { .. }
            | Ask::Stop { .. }
            | Ask::Cancel { .. }
            | Ask::Move { .. }
            | Ask::Seen { .. }
            | Ask::EditNote { .. }
            | Ask::SignIn { .. }
            | Ask::AccountChange { .. }
            | Ask::SignOut { .. }
            | Ask::DeleteNote { .. }
            | Ask::ChangePolicy { .. }
            | Ask::SetRoles { .. }
            | Ask::MakeService { .. }
            | Ask::RemoveProject { .. }
            | Ask::SetVisibility { .. }
            | Ask::Rename { .. }
            | Ask::Lift { .. }
            | Ask::ReadLists { .. }
            | Ask::ChangeModel(_)
            | Ask::ChangeSettings(_)
            | Ask::MoveToNewest(_)
            | Ask::Restore { .. }
            | Ask::MakeProject { .. }
            | Ask::SetPool { .. } => return None,
        };
        if let Some(creating) = self.shown.get(&(client, project)) {
            if let NewBudget::AsAllowed { .. } = budget {
                self.as_allowed.insert((client, key.number));
            }
            // A delivered change before sending explicitly permits the newer offer.
            let creating = self.latest.get(&(client, project)).unwrap_or(creating);
            let offer = shapes::fall(creating, &draft, deadline);
            judge
                .check(!matches!(offer, Fall::Refused(_) | Fall::NotOffered(_)), "offers kept: refused draft was sent");
            return Some(offer);
        }
        None
    }
    pub(super) fn requested(&self, client: ClientId, attempt: Token, judge: &mut JudgeOf) {
        if let Some(Fall::Refused(_) | Fall::NotOffered(_)) = self.drafts.get(&(client, attempt.raw())) {
            judge.check(false, "offers kept: refused draft was sent");
        }
    }
    pub(super) fn answered(&mut self, client: ClientId, attempt: Token, outcome: DraftOutcome, judge: &mut JudgeOf) {
        let outcome = match outcome {
            DraftOutcome::ProposedByParty { number, to } => {
                if let Some(Fall::Proposable(recipient)) = self.drafts.get(&(client, attempt.raw())) {
                    let matches = match recipient {
                        Recipient::Role { role, .. } => *role == to,
                        Recipient::Roles { roles, .. } => roles.iter().any(|role| *role == to),
                        Recipient::Task(_) | Recipient::Party(_) => false,
                    };
                    judge.check(matches, "offers kept: actual admitted role differs from the offer");
                }
                let party = self.parties.get(&client).expect("a draft's confirmed party was delivered");
                DraftOutcome::Proposed(ProposalKey { proposer: Proposer::Party(*party), number })
            }
            outcome @ (DraftOutcome::Made
            | DraftOutcome::Proposed(_)
            | DraftOutcome::NoBudget
            | DraftOutcome::Refused) => outcome,
        };
        let as_allowed = self.as_allowed.remove(&(client, attempt.raw()));
        let Some(offer) = self.drafts.remove(&(client, attempt.raw())) else {
            return;
        };
        match (offer, outcome) {
            (Fall::Within, DraftOutcome::Made) => {}
            // AsAllowed is resolved against the live pool, not its delivered snapshot
            // (jig/docs/design/client/domain/contract/requests.md, 2, and jig/docs/design/client/domain/contract/documents.md, 3).
            (Fall::Within, DraftOutcome::NoBudget) if as_allowed => {}
            (Fall::Proposable(to), DraftOutcome::Proposed(proposal)) => {
                if let Some(recipient) = self.recipients.get(&proposal) {
                    judge.check(*recipient == to, "offers kept: proposed to a different recipient");
                } else {
                    self.proposed.entry(proposal).or_default().push(to);
                }
            }
            _ => judge.check(false, "offers kept: draft answer differs from its offer"),
        }
    }
}
pub(crate) fn outcome(answer: &Answer) -> Option<DraftOutcome> {
    match answer {
        Answer::Done(Outcome::Made { .. } | Outcome::Document(DocumentOutcome::Created { .. })) => {
            Some(DraftOutcome::Made)
        }
        Answer::Done(Outcome::Document(DocumentOutcome::Proposed { proposal, to })) => {
            Some(DraftOutcome::ProposedByParty { number: *proposal, to: *to })
        }
        Answer::Refused {
            refusal: Refusal::Authority { lacking: Lacking::Budget { needed: 1, left: 0 }, proposable: None },
            ..
        } => Some(DraftOutcome::NoBudget),
        Answer::Refused { .. } => Some(DraftOutcome::Refused),
        Answer::Done(_) => None,
    }
}
