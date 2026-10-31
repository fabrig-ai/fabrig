//! Every durable family names its owners (jig/docs/design/domain/people.md, 5.2).
use super::{core, people, r, tasks};
use jig_domain_accounts as accounts;
use jig_domain_notes as notes;

/// Project and task names carried by a durable row, independently of live arenas.
#[must_use]
pub fn naming(key: &core::Key, record: &core::Record) -> Option<r::Naming> {
    let mut name = r::Naming { row: format!("{key:?}"), project: None, task: None, outlives: None };
    match key {
        core::Key::Core(key) => match key {
            core::CoreKey::Deployment => {}
            core::CoreKey::Projection(key) => match key {
                core::ProjectionKey::Goal(task)
                | core::ProjectionKey::Task { task, .. }
                | core::ProjectionKey::Lifecycle { task, .. }
                | core::ProjectionKey::Revision { task, .. } => name.task = Some(*task),
            },
            core::CoreKey::RunProof(task)
            | core::CoreKey::Turn { task, .. }
            | core::CoreKey::Terminal { task, .. }
            | core::CoreKey::EscalationDecision { task, .. } => name.task = Some(*task),
            core::CoreKey::Call(key) => name.task = Some(key.task),
            core::CoreKey::ProposalDecision { proposer, .. } => name.task = proposer_task(*proposer),
            core::CoreKey::Settings(key) => match key {
                core::settings::Key::Proposal { proposer, .. } => name.task = proposer_task(*proposer),
                // Policy settings are deployment templates copied when a project is made.
                core::settings::Key::Default(_)
                | core::settings::Key::Changed(_)
                | core::settings::Key::Held(_)
                | core::settings::Key::Charter { .. }
                | core::settings::Key::Extent
                | core::settings::Key::History(_) => {}
            },
        },
        core::Key::Tasks(key) => match key {
            tasks::Key::Live(task)
            | tasks::Key::Ended(task)
            | tasks::Key::Stub(task)
            | tasks::Key::History { task, .. }
            | tasks::Key::Milestone { task, .. }
            | tasks::Key::Activity { task, .. }
            | tasks::Key::Delegate { task, .. }
            | tasks::Key::EndedDelegate { task, .. }
            | tasks::Key::EndedByParty { task, .. } => name.task = Some(*task),
            tasks::Key::EndedByProject { project, task, .. } => {
                name.project = Some(*project);
                name.task = Some(*task);
            }
            tasks::Key::Update { task, .. } => name.task = Some(task.value()),
            tasks::Key::Ledger(funder) | tasks::Key::ClosedLedger(funder) => {
                match funder {
                    tasks::Funder::Task(task) => name.task = Some(*task),
                    tasks::Funder::Pool { project, .. }
                    | tasks::Funder::Period { project, .. }
                    | tasks::Funder::Recurring { project, .. } => name.project = Some(*project),
                }
                if let tasks::Key::ClosedLedger(_) = key {
                    name.outlives = Some(r::Outlives::Spend);
                }
            }
            tasks::Key::Result { .. }
            | tasks::Key::Writer(_)
            | tasks::Key::Pool(_)
            | tasks::Key::PersonProposal(_)
            | tasks::Key::DecidedProposal(_)
            | tasks::Key::PlacedHold(_) => {}
        },
        core::Key::People(key) => match key {
            people::Key::Roles(project) | people::Key::Policy(project) | people::Key::Deletion(project) => {
                name.project = Some(*project);
            }
            people::Key::ReadConversation { task, .. } | people::Key::EndedRead { task, .. } => name.task = Some(*task),
            people::Key::Person(_)
            | people::Key::SignIn(_)
            | people::Key::ReadResults { .. }
            | people::Key::Answer(_)
            | people::Key::Acted(_) => {}
        },
        core::Key::Notes(key) => match key {
            notes::Key::Entry { .. } => {}
            notes::Key::Line { scope, .. } => note_scope(scope, &mut name),
        },
        core::Key::Accounts(key) => match key {
            accounts::Key::Next
            | accounts::Key::Issued { .. }
            | accounts::Key::Account { .. }
            | accounts::Key::Source { .. } => {}
        },
    }
    record_names(record, &mut name);
    (name.project.is_some() || name.task.is_some()).then_some(name)
}

fn proposer_task(proposer: people::Proposer) -> Option<u64> {
    match proposer {
        people::Proposer::Task(task) => Some(task),
        people::Proposer::Party(_) => None,
    }
}
fn note_scope(scope: &notes::Scope, name: &mut r::Naming) {
    match scope {
        notes::Scope::Deployment => {}
        notes::Scope::Project { project } | notes::Scope::Resources { project, .. } => name.project = Some(*project),
        notes::Scope::Goal { project, goal } => {
            name.project = Some(*project);
            name.task = Some(*goal);
        }
    }
}

fn record_names(record: &core::Record, name: &mut r::Naming) {
    match record {
        core::Record::Core(record) => match record {
            core::CoreRecord::ProposalDecision(row) => name.project = Some(row.project),
            core::CoreRecord::Settings(_)
            | core::CoreRecord::Deployment(_)
            | core::CoreRecord::Projection(_)
            | core::CoreRecord::Call(_)
            | core::CoreRecord::RunProof(_)
            | core::CoreRecord::Turn(_)
            | core::CoreRecord::Terminal(_)
            | core::CoreRecord::EscalationDecision(_) => {}
        },
        core::Record::Tasks(record) => match record {
            tasks::Stored::Live(row) | tasks::Stored::Ended(row) => name.project = Some(row.project),
            tasks::Stored::PlacedHold(row) => name.project = Some(row.project),
            tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row) => {
                name.project = Some(row.project);
            }
            tasks::Stored::Result(row) => name.task = Some(row.task),
            tasks::Stored::Writer(row) => match row.writer {
                tasks::Writer::Run { task, .. } => name.task = Some(task),
                tasks::Writer::Effect { .. } => {}
            },
            tasks::Stored::Update(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedByProject(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::Activity(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Pool(_) => {}
        },
        core::Record::People(record) => match record {
            people::Stored::Answer { ask, .. } => name.project = ask_project(ask),
            people::Stored::Acted { act, .. } => match act {
                people::Act::Close { project, task, .. } => {
                    name.project = Some(*project);
                    name.task = Some(*task);
                }
                people::Act::Adopt { project, .. } | people::Act::RemoveProject { project } => {
                    name.project = Some(*project);
                }
                people::Act::MakeProject { .. } | people::Act::AddKey { .. } | people::Act::RemoveAccount { .. } => {}
            },
            people::Stored::Deletion { .. }
            | people::Stored::Person { .. }
            | people::Stored::ReadResults { .. }
            | people::Stored::ReadConversation { .. }
            | people::Stored::EndedRead { .. }
            | people::Stored::SignIn { .. }
            | people::Stored::Roles { .. }
            | people::Stored::Policy { .. } => {}
        },
        core::Record::Notes(record) => match record {
            // Entries are globally named; project ownership is carried by index lines.
            notes::Record::Entry(_) => {}
            notes::Record::Line(row) => note_scope(&row.scope, name),
        },
        core::Record::Accounts(record) => match record {
            accounts::Stored::Account { uses, .. } => {
                if let Some(uses) = uses {
                    for payer in uses {
                        match payer {
                            accounts::Use::Project(project) => {
                                name.project = Some(*project);
                                if uses.len() == 1 {
                                    name.outlives = Some(r::Outlives::SoleUse);
                                }
                                break;
                            }
                            accounts::Use::Party(_) => {}
                        }
                    }
                }
            }
            accounts::Stored::Next { .. } | accounts::Stored::Issued { .. } | accounts::Stored::Source { .. } => {}
        },
    }
}

fn ask_project(ask: &people::Ask) -> Option<u32> {
    match ask {
        people::Ask::Compound { project, .. }
        | people::Ask::EditNote { project, .. }
        | people::Ask::MakeService { project, .. }
        | people::Ask::Adopt { project, .. }
        | people::Ask::SetRoles { project, .. }
        | people::Ask::RemoveProject { project }
        | people::Ask::Rename { project, .. }
        | people::Ask::ChangePolicy { project, .. }
        | people::Ask::SetPool { project, .. }
        | people::Ask::DecideEscalation { project, .. }
        | people::Ask::DecideProposal { project, .. }
        | people::Ask::Say { project, .. }
        | people::Ask::AnswerQuestion { project, .. }
        | people::Ask::Amend { project, .. }
        | people::Ask::Move { project, .. }
        | people::Ask::TakePerson { project, .. }
        | people::Ask::HandBackPerson { project, .. }
        | people::Ask::AnswerPerson { project, .. }
        | people::Ask::Stop { project, .. }
        | people::Ask::Cancel { project, .. }
        | people::Ask::Release { project, .. }
        | people::Ask::Create { project, .. }
        | people::Ask::Propose { project, .. }
        | people::Ask::Withdraw { project, .. } => Some(*project),
        people::Ask::MakeProject { .. } | people::Ask::ChangeSettings { .. } | people::Ask::MoveToNewest { .. } => {
            Some(0)
        }
        people::Ask::Application { scope, .. } => match scope {
            people::ApplicationScope::Own => None,
            people::ApplicationScope::Project(project) => Some(*project),
            people::ApplicationScope::Deployment => Some(0),
        },
        people::Ask::Lift { .. }
        | people::Ask::MarkSeen { .. }
        | people::Ask::AccountSignIn { .. }
        | people::Ask::AccountSignOut { .. }
        | people::Ask::AccountChange { .. } => None,
    }
}
