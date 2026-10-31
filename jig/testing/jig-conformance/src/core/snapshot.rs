//! Stateless projection of the core's durable rows for every application adapter.
use crate::referee as r;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use std::collections::BTreeMap;

/// Translate the core and child rows without consulting live domain state.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one outside translation covers the durable row vocabulary")]
pub fn snapshot(records: &BTreeMap<core::Key, &core::Record>) -> r::Snapshot {
    let mut snapshot = r::Snapshot::default();
    for (key, row) in records {
        if let Some(name) = super::naming::naming(key, row) {
            if let core::Record::Accounts(jig_domain_accounts::Stored::Account { uses: Some(uses), .. }) = row {
                for payer in uses {
                    if let jig_domain_accounts::Use::Project(project) = payer {
                        snapshot.naming.push(r::Naming { project: Some(*project), ..name.clone() });
                    }
                }
            } else {
                snapshot.naming.push(name);
            }
        }
        if let core::Record::People(people::Stored::Deletion { project, .. }) = row {
            snapshot.deleting.insert(*project);
        }
    }
    let mut decisions = Vec::new();
    let mut authority_acceptances = Vec::new();
    for record in records.values().copied() {
        match record {
            core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Revision {
                history: false,
                revision,
            })) => {
                snapshot.newest = snapshot.newest.max(revision.head.number);
            }
            core::Record::Tasks(tasks::Stored::PlacedHold(row)) => {
                snapshot.placed_holds.insert(
                    row.number,
                    r::PlacedHold {
                        root: match row.scope {
                            tasks::Scope::Subtree { task } => Some(task),
                            tasks::Scope::Project => None,
                        },
                        lifted: row.lifted,
                    },
                );
            }
            core::Record::Tasks(tasks::Stored::Pool(row)) => {
                snapshot.pools.insert(super::name(&row.pool), row.slots);
            }
            core::Record::Tasks(tasks::Stored::History(row)) => {
                if let Some(accepted) = super::acceptance::task(row) {
                    authority_acceptances.push(accepted);
                }
            }
            core::Record::Tasks(tasks::Stored::PersonProposal(row))
                if matches!(row.state, tasks::PersonProposalState::Pending { .. }) =>
            {
                snapshot.party_proposals.insert(row.number, row.proposer);
            }
            core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)) => {
                let mut task = super::task(row);
                task.tracked |= records
                    .contains_key(&core::Key::Core(core::CoreKey::Projection(core::ProjectionKey::Goal(row.number))));
                snapshot.tasks.insert(row.number, task);
                if let tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), entry, .. } =
                    row.escalation
                {
                    snapshot.receipts.insert(r::Receipt::Word(holder, entry));
                }
                if let Some(proposal) = &row.proposal
                    && let tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(holder), .. } =
                        proposal.state
                {
                    snapshot.receipts.insert(r::Receipt::Word(holder, proposal.number));
                }
                snapshot.receipts.insert(r::Receipt::Task(row.number));
                if matches!(row.phase, tasks::Phase::Ended(_)) {
                    snapshot.receipts.insert(r::Receipt::Ended(row.number));
                }
                for word in &row.inbox {
                    snapshot.receipts.insert(r::Receipt::Word(row.number, word.number));
                }
            }
            core::Record::Tasks(tasks::Stored::Writer(row)) => {
                let (task, attempt) = match row.writer {
                    tasks::Writer::Run { task, attempt } => (task, attempt),
                    tasks::Writer::Effect { entry } => (0, entry),
                };
                snapshot.writers.push((
                    r::Name {
                        connector: row.resource.connector,
                        path: row.resource.path.iter().map(|x| x.to_vec()).collect(),
                    },
                    task,
                    attempt,
                ));
            }
            core::Record::Tasks(tasks::Stored::Ledger(row) | tasks::Stored::ClosedLedger(row)) => {
                snapshot.balances.push((
                    row.numbers.budget,
                    row.numbers.spent,
                    row.numbers.spent_below,
                    row.numbers.reserved,
                ));
                if let tasks::Funder::Pool { person, .. } = row.funder {
                    *snapshot.person_spend.entry(person).or_default() += row.numbers.spent;
                }
            }
            core::Record::People(
                people::Stored::ReadConversation { party, task, through }
                | people::Stored::EndedRead { party, task, through },
            ) => {
                snapshot.read_positions.insert((*party, *task), *through);
            }
            core::Record::People(people::Stored::SignIn { number, .. }) => {
                snapshot.receipts.insert(r::Receipt::SignIn(*number));
            }
            core::Record::People(people::Stored::Answer { key, .. }) => {
                snapshot.receipts.insert(r::Receipt::Party(key.person, key.key));
            }
            core::Record::Core(core::CoreRecord::RunProof(row)) => {
                snapshot.receipts.insert(r::Receipt::Claim(row.task, row.attempt));
            }
            core::Record::Core(core::CoreRecord::Turn(row)) => {
                snapshot.receipts.insert(r::Receipt::Turn(row.task, row.attempt, row.turn));
            }
            core::Record::Core(core::CoreRecord::Terminal(row)) => {
                snapshot.receipts.insert(r::Receipt::Terminal(row.task, row.attempt));
            }
            core::Record::Core(core::CoreRecord::Call(row)) => {
                snapshot.receipts.insert(r::Receipt::Call(
                    row.key.task,
                    row.key.attempt,
                    row.key.completion,
                    row.key.position,
                ));
                if let core::CallPart::Delegated(members) = &row.part {
                    snapshot.batches.push(members.iter().map(|member| member.task).collect());
                }
            }
            core::Record::Core(core::CoreRecord::ProposalDecision(row))
                if row.choice == people::ProposalChoice::Accepted =>
            {
                decisions.push(*row);
            }
            core::Record::Core(_)
            | core::Record::Tasks(_)
            | core::Record::People(_)
            | core::Record::Notes(_)
            | core::Record::Accounts(_) => {}
        }
    }
    for accepted in decisions {
        let key = core::Key::Tasks(tasks::Key::DecidedProposal(accepted.proposal));
        if let Some(core::Record::Tasks(tasks::Stored::DecidedProposal(proposal))) = records.get(&key)
            && let tasks::PersonAction::Amend { task, .. } = proposal.action
        {
            authority_acceptances.push((
                accepted.at,
                task,
                r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
            ));
        }
        if let Some((first, last)) = accepted.created {
            for task in first..=last {
                if snapshot.tasks.get(&task).is_some_and(|row| {
                    row.source
                        == match accepted.proposer {
                            people::Proposer::Task(task) => r::Source::Task(task),
                            people::Proposer::Party(person) => r::Source::Person(person),
                        }
                }) {
                    snapshot.accepted_tasks.insert(
                        task,
                        r::Acceptance { proposal: accepted.proposal, by: r::Source::Person(accepted.by) },
                    );
                }
            }
        }
    }
    authority_acceptances.sort_by_key(|(at, ..)| *at);
    for (_, task, accepted) in authority_acceptances {
        snapshot.accepted_tasks.insert(task, accepted);
    }
    snapshot
}

/// The committed proposal decisions used to attribute connector-owned effects.
#[must_use]
pub fn proposal_decisions(records: &BTreeMap<core::Key, &core::Record>) -> Vec<core::ProposalDecisionRecord> {
    records
        .values()
        .filter_map(|row| {
            if let core::Record::Core(core::CoreRecord::ProposalDecision(value)) = row
                && value.choice == people::ProposalChoice::Accepted
            {
                Some(*value)
            } else {
                None
            }
        })
        .collect()
}
