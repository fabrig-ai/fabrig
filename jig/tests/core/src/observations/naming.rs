//! Independent row attribution for project erasure
//! (temper/docs/design/draft/playground-reset.md, 5 and 8).
use std::collections::{BTreeMap, BTreeSet};

use crate::Store;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

/// Count each durable row once for each project it names. Task ownership is
/// learned before attribution, and retained even after the owner row is erased.
#[must_use]
pub fn naming(store: &Store, tasks: &mut BTreeMap<u64, u32>) -> BTreeMap<u32, u32> {
    for row in store.rows.values() {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))) = row {
            tasks.insert(task.number, task.project);
        }
    }
    let mut counts = BTreeMap::<u32, u32>::new();
    for row in store.rows.values() {
        let mut projects = BTreeSet::new();
        if let root::Record::Core(row) = row {
            record(row, tasks, &mut projects);
        }
        for project in projects {
            let count = counts.entry(project).or_default();
            *count = count.checked_add(1).expect("project row count fits u32");
        }
    }
    for place in store.transcript_places() {
        let task = match place {
            jig_protocol_store::keys::Own::Transcript { task }
            | jig_protocol_store::keys::Own::Attempt { task, .. } => task,
            jig_protocol_store::keys::Own::Format
            | jig_protocol_store::keys::Own::Layouts
            | jig_protocol_store::keys::Own::Wires => continue,
        };
        if let Some(project) = tasks.get(&task) {
            let count = counts.entry(*project).or_default();
            *count = count.checked_add(1).expect("project row count fits u32");
        }
    }
    counts
}

fn task(number: u64, tasks: &BTreeMap<u64, u32>, projects: &mut BTreeSet<u32>) {
    projects.extend(tasks.get(&number));
}

fn proposer(who: people::Proposer, tasks: &BTreeMap<u64, u32>, projects: &mut BTreeSet<u32>) {
    match who {
        people::Proposer::Task(number) => task(number, tasks, projects),
        people::Proposer::Party(_) => {}
    }
}

fn record(row: &core::Record, tasks: &BTreeMap<u64, u32>, projects: &mut BTreeSet<u32>) {
    match row {
        core::Record::Tasks(row) => task_row(row, tasks, projects),
        core::Record::Core(row) => core_row(row, tasks, projects),
        core::Record::People(row) => match row {
            people::Stored::Roles { project, .. } | people::Stored::Policy { project, .. } => {
                projects.insert(*project);
            }
            people::Stored::ReadConversation { task: number, .. } | people::Stored::EndedRead { task: number, .. } => {
                task(*number, tasks, projects);
            }
            people::Stored::Answer { .. }
            | people::Stored::Acted { .. }
            | people::Stored::Person { .. }
            | people::Stored::ReadResults { .. }
            | people::Stored::SignIn { .. } => {}
        },
        core::Record::Notes(row) => match row {
            notes::Record::Line(row) => scope(&row.scope, projects),
            notes::Record::Entry(row) => {
                scope(&row.scope, projects);
                if let notes::Author::Task { task: number, .. } = row.author {
                    task(number, tasks, projects);
                }
            }
        },
        core::Record::Accounts(row) => match row {
            accounts::Stored::Account { uses, .. } => {
                for payer in uses.iter().flatten() {
                    match payer {
                        accounts::Use::Project(project) => {
                            projects.insert(*project);
                        }
                        accounts::Use::Party(_) => {}
                    }
                }
            }
            accounts::Stored::Next { .. } | accounts::Stored::Issued { .. } | accounts::Stored::Source { .. } => {}
        },
    }
}

fn task_row(row: &tasks::Stored, tasks: &BTreeMap<u64, u32>, projects: &mut BTreeSet<u32>) {
    match row {
        tasks::Stored::Live(row) | tasks::Stored::Ended(row) => {
            projects.insert(row.project);
        }
        tasks::Stored::PlacedHold(row) => {
            projects.insert(row.project);
        }
        tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row) => {
            projects.insert(row.project);
        }
        tasks::Stored::EndedByProject(row) => {
            projects.insert(row.project);
            task(row.task, tasks, projects);
        }
        tasks::Stored::History(row) => task(row.task, tasks, projects),
        tasks::Stored::Milestone(row) => task(row.task, tasks, projects),
        tasks::Stored::Activity(row) => task(row.task, tasks, projects),
        tasks::Stored::Stub(row) => {
            task(row.task, tasks, projects);
            task(row.result.raw(), tasks, projects);
        }
        tasks::Stored::Update(row) => task(row.task, tasks, projects),
        tasks::Stored::Result(row) => task(row.task, tasks, projects),
        tasks::Stored::EndedByParty(row) => task(row.task, tasks, projects),
        tasks::Stored::EndedDelegate(row) | tasks::Stored::Delegate(row) => {
            task(row.task, tasks, projects);
            task(row.requester, tasks, projects);
        }
        tasks::Stored::Writer(row) => match row.writer {
            tasks::Writer::Run { task: number, .. } => task(number, tasks, projects),
            tasks::Writer::Effect { .. } => {}
        },
        tasks::Stored::Ledger(_) | tasks::Stored::ClosedLedger(_) | tasks::Stored::Pool(_) => {}
    }
}

fn core_row(row: &core::CoreRecord, tasks: &BTreeMap<u64, u32>, projects: &mut BTreeSet<u32>) {
    match row {
        core::CoreRecord::Projection(row) => match row {
            core::ProjectionRecord::Goal { goal, .. } => {
                projects.insert(goal.project);
                task(goal.number, tasks, projects);
            }
            core::ProjectionRecord::Task { goal, task: row } => {
                task(*goal, tasks, projects);
                task(row.number, tasks, projects);
            }
            core::ProjectionRecord::Milestone { goal, milestone } => {
                task(*goal, tasks, projects);
                let (core::MilestoneId::Lifecycle { task: number, .. }
                | core::MilestoneId::Revision { task: number, .. }) = milestone.identity;
                task(number, tasks, projects);
            }
        },
        core::CoreRecord::RunProof(row) => task(row.task, tasks, projects),
        core::CoreRecord::Call(row) => task(row.key.task, tasks, projects),
        core::CoreRecord::Turn(row) => task(row.task, tasks, projects),
        core::CoreRecord::Terminal(row) => task(row.task, tasks, projects),
        core::CoreRecord::EscalationDecision(row) => {
            projects.insert(row.project);
            task(row.task, tasks, projects);
        }
        core::CoreRecord::ProposalDecision(row) => {
            projects.insert(row.project);
            proposer(row.proposer, tasks, projects);
        }
        core::CoreRecord::Settings(row) => match row {
            core::settings::Record::Proposal { proposer: who, .. } => proposer(*who, tasks, projects),
            core::settings::Record::Revision { revision, .. } => {
                if let Some((who, _)) = revision.head.proposal {
                    proposer(who, tasks, projects);
                }
            }
            core::settings::Record::Default { .. }
            | core::settings::Record::Changed { .. }
            | core::settings::Record::Extent { .. }
            | core::settings::Record::Charter { .. } => {}
        },
        core::CoreRecord::Deployment(_) => {}
    }
}

fn scope(scope: &notes::Scope, projects: &mut BTreeSet<u32>) {
    match scope {
        notes::Scope::Project { project }
        | notes::Scope::Goal { project, .. }
        | notes::Scope::Resources { project, .. } => {
            projects.insert(*project);
        }
        notes::Scope::Deployment => {}
    }
}
