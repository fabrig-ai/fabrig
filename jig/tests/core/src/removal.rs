//! Independent store audit for whole project deletion (draft 5; store.md, 11.1).
use crate::effects::World;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;

/// Every stored row naming the project or its collected task numbers, except
/// the project's closed period and pool totals and the connector's records of
/// objects it made, which outlive deletion.
#[must_use]
pub fn rows_naming(world: &World, project: u32, tasks: &[u64]) -> Vec<root::Key> {
    world.store.rows.iter().filter(|(_, row)| names(row, project, tasks)).map(|(key, _)| key.clone()).collect()
}

fn funder(value: tasks::Funder, project: u32, tasks: &[u64]) -> bool {
    match value {
        tasks::Funder::Task(task) => tasks.contains(&task),
        tasks::Funder::Period { project: named, .. } | tasks::Funder::Pool { project: named, .. } => named == project,
        tasks::Funder::Recurring { project: named, task, .. } => named == project || tasks.contains(&task),
    }
}
fn party(value: tasks::Party, project: u32, tasks: &[u64]) -> bool {
    match value {
        tasks::Party::Task(task) => tasks.contains(&task),
        tasks::Party::Deployment { project: named } => named == project,
        tasks::Party::Person(_) => false,
    }
}
fn scope(value: &notes::Scope, project: u32, tasks: &[u64]) -> bool {
    match value {
        notes::Scope::Project { project: named } | notes::Scope::Resources { project: named, .. } => *named == project,
        notes::Scope::Goal { project: named, goal } => *named == project || tasks.contains(goal),
        notes::Scope::Deployment => false,
    }
}
fn proposer(value: people::Proposer, tasks: &[u64]) -> bool {
    match value {
        people::Proposer::Task(task) => tasks.contains(&task),
        people::Proposer::Party(_) => false,
    }
}
fn setting(value: people::Setting, project: u32) -> bool {
    match value {
        people::Setting::Policy(named) => named == project,
        people::Setting::Charter(_) | people::Setting::Template(_) | people::Setting::Rules => false,
    }
}
fn names(row: &root::Record, project: u32, ids: &[u64]) -> bool {
    match row {
        root::Record::Core(core::Record::Tasks(row)) => task_row(row, project, ids),
        root::Record::Core(core::Record::People(row)) => people_row(row, project, ids),
        root::Record::Core(core::Record::Accounts(row)) => match row {
            accounts::Stored::Account { uses, .. } => {
                uses.as_ref().is_some_and(|uses| uses.contains(&accounts::Use::Project(project)))
            }
            accounts::Stored::Next { .. } | accounts::Stored::Issued { .. } | accounts::Stored::Source { .. } => false,
        },
        root::Record::Core(core::Record::Notes(row)) => match row {
            notes::Record::Line(row) => scope(&row.scope, project, ids),
            notes::Record::Entry(row) => {
                scope(&row.scope, project, ids)
                    || row.references.iter().any(|task| ids.contains(task))
                    || match row.author {
                        notes::Author::Task { task, .. } => ids.contains(&task),
                        notes::Author::Party { .. } => false,
                    }
            }
        },
        root::Record::Core(core::Record::Core(row)) => core_row(row, project, ids),
        root::Record::Connector { record, .. } => match record {
            connector::Record::Task { task, project: named, .. }
            | connector::Record::Proposal { task, project: named, .. } => *named == project || ids.contains(task),
            connector::Record::Adoption { project: named, .. } => *named == project,
            connector::Record::Procedure(row) => {
                row.project == project
                    || ids.contains(&row.task)
                    || row.delegate.is_some_and(|task| ids.contains(&task))
            }
            connector::Record::ProcedureDelegateOutcome { task, child, revisions, .. } => {
                ids.contains(task)
                    || child.is_some_and(|task| ids.contains(&task))
                    || revisions.iter().any(|(task, _)| ids.contains(task))
            }
            connector::Record::Result { task, .. } | connector::Record::Subscription { task, .. } => ids.contains(task),
            connector::Record::Outbox(row) => ids.contains(&row.task) || ids.contains(&row.key.task),
            connector::Record::ApplicationObject { .. }
            | connector::Record::ApplicationAnswer { .. }
            | connector::Record::Pool { .. }
            | connector::Record::Made { .. } => false,
        },
    }
}
fn task_row(row: &tasks::Stored, project: u32, ids: &[u64]) -> bool {
    match row {
        tasks::Stored::Live(row) | tasks::Stored::Ended(row) => {
            row.project == project
                || ids.contains(&row.number)
                || ids.contains(&row.root)
                || party(row.requester, project, ids)
                || funder(row.funder, project, ids)
                || row
                    .dependencies
                    .iter()
                    .chain(row.delegates.iter())
                    .chain(row.references.iter())
                    .chain(row.result_followups.iter())
                    .any(|task| ids.contains(task))
        }
        tasks::Stored::Ledger(row) => {
            funder(row.funder, project, ids) || row.parent.is_some_and(|parent| funder(parent, project, ids))
        }
        tasks::Stored::ClosedLedger(row) => match row.funder {
            tasks::Funder::Period { project: named, .. } | tasks::Funder::Pool { project: named, .. }
                if named == project =>
            {
                false
            }
            value @ (tasks::Funder::Task(_)
            | tasks::Funder::Period { .. }
            | tasks::Funder::Pool { .. }
            | tasks::Funder::Recurring { .. }) => {
                funder(value, project, ids) || row.parent.is_some_and(|parent| funder(parent, project, ids))
            }
        },
        tasks::Stored::PlacedHold(row) => {
            row.project == project
                || match row.owner {
                    tasks::Owner::Task { task } => ids.contains(&task),
                    tasks::Owner::Party { .. } => false,
                }
                || match row.scope {
                    tasks::Scope::Subtree { task } => ids.contains(&task),
                    tasks::Scope::Project => false,
                }
                || row.exempt.iter().any(|task| ids.contains(task))
        }
        tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row) => {
            row.project == project
                || match &row.action {
                    tasks::PersonAction::Amend { task, .. } => ids.contains(task),
                    tasks::PersonAction::Batch(_) | tasks::PersonAction::Settings { .. } => false,
                }
        }
        tasks::Stored::Stub(row) => ids.contains(&row.task) || ids.contains(&row.result.raw()),
        tasks::Stored::History(row) => ids.contains(&row.task) || party(row.by, project, ids),
        tasks::Stored::Milestone(row) => ids.contains(&row.task),
        tasks::Stored::Activity(row) => ids.contains(&row.task),
        tasks::Stored::Result(row) => ids.contains(&row.task),
        tasks::Stored::Update(row) => ids.contains(&row.task),
        tasks::Stored::EndedByParty(row) => ids.contains(&row.task),
        tasks::Stored::EndedByProject(row) => row.project == project || ids.contains(&row.task),
        tasks::Stored::Delegate(row) | tasks::Stored::EndedDelegate(row) => {
            ids.contains(&row.task) || ids.contains(&row.requester)
        }
        tasks::Stored::Writer(row) => match row.writer {
            tasks::Writer::Run { task, .. } => ids.contains(&task),
            tasks::Writer::Effect { .. } => false,
        },
        tasks::Stored::Pool(_) => false,
    }
}
fn core_row(row: &core::CoreRecord, project: u32, ids: &[u64]) -> bool {
    match row {
        core::CoreRecord::Removal(row) => row.project == project,
        core::CoreRecord::Deployment(_) => false,
        core::CoreRecord::Call(row) => ids.contains(&row.key.task),
        core::CoreRecord::RunProof(row) => ids.contains(&row.task),
        core::CoreRecord::Turn(row) => ids.contains(&row.task),
        core::CoreRecord::Terminal(row) => ids.contains(&row.task),
        core::CoreRecord::ProposalDecision(row) => {
            row.project == project
                || proposer(row.proposer, ids)
                || row.created.is_some_and(|(first, last)| ids.iter().any(|task| first <= *task && *task <= last))
        }
        core::CoreRecord::EscalationDecision(row) => row.project == project || ids.contains(&row.task),
        core::CoreRecord::Projection(row) => match row {
            core::ProjectionRecord::Goal { goal, .. } => goal.project == project || ids.contains(&goal.number),
            core::ProjectionRecord::Task { goal, task } => {
                ids.contains(goal) || ids.contains(&task.number) || party(task.requester, project, ids)
            }
            core::ProjectionRecord::Milestone { goal, milestone } => {
                ids.contains(goal)
                    || match milestone.identity {
                        core::MilestoneId::Lifecycle { task, .. } | core::MilestoneId::Revision { task, .. } => {
                            ids.contains(&task)
                        }
                    }
            }
        },
        core::CoreRecord::Settings(row) => match row {
            core::settings::Record::Default { setting: named, .. }
            | core::settings::Record::Changed { setting: named, .. } => setting(*named, project),
            core::settings::Record::Revision { revision, .. } => {
                revision.items.iter().any(|item| setting(item.setting, project))
            }
            core::settings::Record::Proposal { proposer: owner, value, .. } => {
                proposer(*owner, ids) || value.items.iter().any(|item| setting(item.setting, project))
            }
            core::settings::Record::Extent { .. } | core::settings::Record::Charter { .. } => false,
        },
    }
}
fn people_row(row: &people::Stored, project: u32, ids: &[u64]) -> bool {
    match row {
        people::Stored::Roles { project: named, .. } | people::Stored::Policy { project: named, .. } => {
            *named == project
        }
        people::Stored::ReadConversation { task, .. } | people::Stored::EndedRead { task, .. } => ids.contains(task),
        people::Stored::Answer { ask, outcome, .. } => {
            ask_project(ask, project) || ask_tasks(ask, ids) || outcome_names(outcome, project, ids)
        }
        people::Stored::Acted { act, outcome, .. } => {
            (match act {
                people::Act::Close { project: named, .. }
                | people::Act::Cancel { project: named, .. }
                | people::Act::Adopt { project: named, .. }
                | people::Act::RemoveProject { project: named } => *named == project,
                people::Act::MakeProject { .. } | people::Act::AddKey { .. } | people::Act::RemoveAccount { .. } => {
                    false
                }
            }) || outcome_names(outcome, project, ids)
        }
        people::Stored::Person { .. } | people::Stored::SignIn { .. } | people::Stored::ReadResults { .. } => false,
    }
}
fn ask_project(ask: &people::Ask, project: u32) -> bool {
    match ask {
        people::Ask::Application { scope, .. } => match scope {
            people::ApplicationScope::Project(named) => *named == project,
            people::ApplicationScope::Own | people::ApplicationScope::Deployment => false,
        },
        people::Ask::AccountChange { change: people::AccountChange::SetUses { uses: Some(uses), .. }, .. } => {
            uses.iter().any(|value| matches!(value, people::Use::Project(named) if *named == project))
        }
        people::Ask::Lift { .. }
        | people::Ask::MarkSeen { .. }
        | people::Ask::AccountSignIn { .. }
        | people::Ask::AccountSignOut { .. }
        | people::Ask::AccountChange { .. }
        | people::Ask::MakeProject { .. }
        | people::Ask::ChangeSettings { .. }
        | people::Ask::MoveToNewest { .. } => false,
        people::Ask::Compound { project: named, .. }
        | people::Ask::Create { project: named, .. }
        | people::Ask::EditNote { project: named, .. }
        | people::Ask::MakeService { project: named, .. }
        | people::Ask::Adopt { project: named, .. }
        | people::Ask::Propose { project: named, .. }
        | people::Ask::Withdraw { project: named, .. }
        | people::Ask::Stop { project: named, .. }
        | people::Ask::Cancel { project: named, .. }
        | people::Ask::Release { project: named, .. }
        | people::Ask::TakePerson { project: named, .. }
        | people::Ask::HandBackPerson { project: named, .. }
        | people::Ask::AnswerPerson { project: named, .. }
        | people::Ask::Move { project: named, .. }
        | people::Ask::DecideProposal { project: named, .. }
        | people::Ask::Say { project: named, .. }
        | people::Ask::AnswerQuestion { project: named, .. }
        | people::Ask::Amend { project: named, .. }
        | people::Ask::SetRoles { project: named, .. }
        | people::Ask::ChangePolicy { project: named, .. }
        | people::Ask::Rename { project: named, .. }
        | people::Ask::RemoveProject { project: named, .. }
        | people::Ask::SetPool { project: named, .. }
        | people::Ask::DecideEscalation { project: named, .. } => *named == project,
    }
}

#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "task references in retained asks supplement their project identities"
)]
fn ask_tasks(ask: &people::Ask, ids: &[u64]) -> bool {
    match ask {
        people::Ask::MarkSeen { seen: people::Seen::Conversation { task, .. } }
        | people::Ask::Stop { task, .. }
        | people::Ask::Cancel { task, .. }
        | people::Ask::Release { task, .. }
        | people::Ask::TakePerson { task, .. }
        | people::Ask::HandBackPerson { task, .. }
        | people::Ask::AnswerPerson { task, .. }
        | people::Ask::Move { task, .. }
        | people::Ask::Say { task, .. }
        | people::Ask::AnswerQuestion { task, .. }
        | people::Ask::Amend { task, .. }
        | people::Ask::DecideEscalation { task, .. } => ids.contains(task),
        people::Ask::DecideProposal { proposer: owner, .. } => proposer(*owner, ids),
        _ => false,
    }
}

#[expect(clippy::wildcard_enum_match_arm, reason = "only outcomes with project or task identities name deletion rows")]
fn outcome_names(value: &people::Outcome, project: u32, ids: &[u64]) -> bool {
    match value {
        people::Outcome::Compound { project: named, .. }
        | people::Outcome::RolesSet { project: named }
        | people::Outcome::PolicyChanged { project: named }
        | people::Outcome::Renamed { project: named }
        | people::Outcome::ProjectMade { project: named }
        | people::Outcome::ProjectRemoved { project: named }
        | people::Outcome::Adopted { project: named }
        | people::Outcome::PoolSet { project: named, .. } => *named == project,
        people::Outcome::Created { first, last, .. } => ids.iter().any(|task| first <= task && task <= last),
        people::Outcome::Amended { task, .. }
        | people::Outcome::EscalationDecided { task, .. }
        | people::Outcome::Stopped { task }
        | people::Outcome::Cancelled { task }
        | people::Outcome::Released { task }
        | people::Outcome::PersonTaken { task }
        | people::Outcome::PersonHandedBack { task }
        | people::Outcome::PersonAnswered { task }
        | people::Outcome::Moved { task }
        | people::Outcome::Said { task, .. }
        | people::Outcome::QuestionAnswered { task, .. }
        | people::Outcome::MarkedSeen { seen: people::Seen::Conversation { task, .. } } => ids.contains(task),
        people::Outcome::ProposalDecided { proposer: owner, .. } => proposer(*owner, ids),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_audit_finds_references_held_only_in_retained_values() {
        let mut world = World::new(1880, false);
        let project = 99;
        let task = 999;
        for (ask, outcome) in [
            (
                people::Ask::MakeProject { name: Box::from("removed"), policy: 1 },
                people::Outcome::ProjectMade { project },
            ),
            (
                people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } },
                people::Outcome::Refused(people::Refusal::Unknown),
            ),
            (people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } }, people::Outcome::Stopped { task }),
        ] {
            let row = root::Record::Core(core::Record::People(people::Stored::Answer {
                key: people::RequestKey { person: 1, key: [99; 16] },
                ask: Box::new(ask),
                models: Box::new([]),
                outcome,
                at: skein_lib::Wall::EPOCH,
            }));
            let key = row.key();
            world.store.rows.insert(key.clone(), row);
            assert!(rows_naming(&world, project, &[task]).contains(&key));
            world.store.rows.remove(&key);
            assert!(rows_naming(&world, project, &[task]).is_empty());
        }
    }
}
