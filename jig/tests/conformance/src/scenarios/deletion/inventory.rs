//! Exhaustive project/task ownership inventory for the deletion story
//! (temper/docs/plan/playground-reset-r6/notes/contracts.md, J4).
use super::{CONTROL, PROJECT, World, check};
use jig_conformance::referee::Violation;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use std::collections::BTreeSet;

pub(super) fn tasks(world: &World, project: u32) -> BTreeSet<u64> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.project == project =>
            {
                Some(row.number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect()
}
fn party(party: tasks::Party, project: u32, task_numbers: &BTreeSet<u64>) -> bool {
    match party {
        tasks::Party::Deployment { project: owner } => owner == project,
        tasks::Party::Task(task) => task_numbers.contains(&task),
        tasks::Party::Person(_) => false,
    }
}
fn funder(funder: tasks::Funder, project: u32, task_numbers: &BTreeSet<u64>) -> bool {
    match funder {
        tasks::Funder::Period { project: owner, .. } | tasks::Funder::Pool { project: owner, .. } => owner == project,
        tasks::Funder::Recurring { project: owner, task, .. } => owner == project || task_numbers.contains(&task),
        tasks::Funder::Task(task) => task_numbers.contains(&task),
    }
}
fn proposer(proposer: people::Proposer, task_numbers: &BTreeSet<u64>) -> bool {
    match proposer {
        people::Proposer::Task(task) => task_numbers.contains(&task),
        people::Proposer::Party(_) => false,
    }
}
fn scope(scope: &notes::Scope, project: u32, task_numbers: &BTreeSet<u64>) -> bool {
    match scope {
        notes::Scope::Project { project: owner } | notes::Scope::Resources { project: owner, .. } => *owner == project,
        notes::Scope::Goal { project: owner, goal } => *owner == project || task_numbers.contains(goal),
        notes::Scope::Deployment => false,
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "every root and child record is classified exhaustively so new families require an ownership decision"
)]
pub(super) fn family(row: &root::Record, project: u32, task_numbers: &BTreeSet<u64>) -> Option<&'static str> {
    let (names, family) = match row {
        root::Record::Core(row) => match row {
            core::Record::Tasks(row) => match row {
                tasks::Stored::Live(row) => (
                    row.project == project
                        || task_numbers.contains(&row.number)
                        || party(row.requester, project, task_numbers)
                        || funder(row.funder, project, task_numbers)
                        || row
                            .delegates
                            .iter()
                            .chain(row.references.iter())
                            .chain(row.spec.inputs.iter())
                            .chain(row.dependencies.iter())
                            .any(|task| task_numbers.contains(task)),
                    "Live",
                ),
                tasks::Stored::Ended(row) => (
                    row.project == project
                        || task_numbers.contains(&row.number)
                        || party(row.requester, project, task_numbers)
                        || funder(row.funder, project, task_numbers),
                    "Ended",
                ),
                tasks::Stored::History(row) => {
                    (task_numbers.contains(&row.task) || party(row.by, project, task_numbers), "History")
                }
                tasks::Stored::Activity(row) => (task_numbers.contains(&row.task), "Activity"),
                tasks::Stored::Milestone(row) => (task_numbers.contains(&row.task), "Milestone"),
                tasks::Stored::Update(row) => (task_numbers.contains(&row.task), "Update"),
                tasks::Stored::Result(row) => (task_numbers.contains(&row.task), "Result"),
                tasks::Stored::EndedByParty(row) => (task_numbers.contains(&row.task), "EndedByParty"),
                tasks::Stored::EndedByProject(row) => {
                    (row.project == project || task_numbers.contains(&row.task), "EndedByProject")
                }
                tasks::Stored::Delegate(row) => {
                    (task_numbers.contains(&row.task) || task_numbers.contains(&row.requester), "Delegate")
                }
                tasks::Stored::EndedDelegate(row) => {
                    (task_numbers.contains(&row.task) || task_numbers.contains(&row.requester), "EndedDelegate")
                }
                tasks::Stored::Stub(row) => {
                    (task_numbers.contains(&row.task) || task_numbers.contains(&row.result.raw()), "Stub")
                }
                tasks::Stored::Ledger(row) => (
                    funder(row.funder, project, task_numbers)
                        || row.parent.is_some_and(|parent| funder(parent, project, task_numbers)),
                    match row.funder {
                        tasks::Funder::Period { .. } => "PeriodLedger",
                        tasks::Funder::Pool { .. } => "PoolLedger",
                        tasks::Funder::Recurring { .. } | tasks::Funder::Task(_) => "Ledger",
                    },
                ),
                tasks::Stored::ClosedLedger(row) => (funder(row.funder, project, task_numbers), "ClosedLedger"),
                tasks::Stored::PersonProposal(row) => {
                    (row.project == project || task_numbers.contains(&row.first()), "PersonProposal")
                }
                tasks::Stored::DecidedProposal(row) => {
                    (row.project == project || task_numbers.contains(&row.first()), "DecidedProposal")
                }
                tasks::Stored::PlacedHold(row) => (
                    row.project == project
                        || match row.owner {
                            tasks::Owner::Party { .. } => false,
                            tasks::Owner::Task { task } => task_numbers.contains(&task),
                        }
                        || match row.scope {
                            tasks::Scope::Project => false,
                            tasks::Scope::Subtree { task } => task_numbers.contains(&task),
                        }
                        || row.exempt.iter().any(|task| task_numbers.contains(task)),
                    "PlacedHold",
                ),
                tasks::Stored::Writer(row) => (
                    match row.writer {
                        tasks::Writer::Run { task, .. } => task_numbers.contains(&task),
                        tasks::Writer::Effect { .. } => false,
                    },
                    "Writer",
                ),
                tasks::Stored::Pool(_) => (false, "Pool"),
            },
            core::Record::Core(row) => match row {
                core::CoreRecord::Turn(row) => (task_numbers.contains(&row.task), "Turn"),
                core::CoreRecord::Terminal(row) => (task_numbers.contains(&row.task), "Terminal"),
                core::CoreRecord::Call(row) => (task_numbers.contains(&row.key.task), "Call"),
                core::CoreRecord::RunProof(row) => (task_numbers.contains(&row.task), "RunProof"),
                core::CoreRecord::ProposalDecision(row) => (
                    row.project == project
                        || proposer(row.proposer, task_numbers)
                        || row.created.is_some_and(|(first, last)| task_numbers.range(first..=last).next().is_some()),
                    "ProposalDecision",
                ),
                core::CoreRecord::EscalationDecision(row) => {
                    (row.project == project || task_numbers.contains(&row.task), "EscalationDecision")
                }
                core::CoreRecord::Projection(row) => (
                    match row {
                        core::ProjectionRecord::Goal { goal, .. } => {
                            goal.project == project || task_numbers.contains(&goal.number)
                        }
                        core::ProjectionRecord::Task { goal, task } => {
                            task_numbers.contains(goal)
                                || task_numbers.contains(&task.number)
                                || party(task.requester, project, task_numbers)
                        }
                        core::ProjectionRecord::Milestone { goal, milestone } => {
                            task_numbers.contains(goal)
                                || match milestone.identity {
                                    core::MilestoneId::Lifecycle { task, .. }
                                    | core::MilestoneId::Revision { task, .. } => task_numbers.contains(&task),
                                }
                        }
                    },
                    "Projection",
                ),
                // Settings have deployment scope 0; project settings do not exist.
                core::CoreRecord::Settings(row) => (
                    match row {
                        core::settings::Record::Proposal { proposer: by, .. } => proposer(*by, task_numbers),
                        core::settings::Record::Default { .. }
                        | core::settings::Record::Changed { .. }
                        | core::settings::Record::Revision { .. }
                        | core::settings::Record::Extent { .. }
                        | core::settings::Record::Charter { .. } => false,
                    },
                    "Settings",
                ),
                core::CoreRecord::Deployment(_) => (false, "Deployment"),
            },
            core::Record::People(row) => match row {
                people::Stored::Deletion { project: owner, .. } => (*owner == project, "Deletion"),
                people::Stored::Roles { project: owner, .. } => (*owner == project, "Roles"),
                people::Stored::Policy { project: owner, .. } => (*owner == project, "Policy"),
                people::Stored::ReadConversation { task, .. } => (task_numbers.contains(task), "ReadConversation"),
                people::Stored::EndedRead { task, .. } => (task_numbers.contains(task), "EndedRead"),
                people::Stored::Acted { act, .. } => (
                    match act {
                        people::Act::Close { project: owner, task, .. }
                        | people::Act::Cancel { project: owner, task, .. } => {
                            *owner == project || task_numbers.contains(task)
                        }
                        people::Act::Adopt { project: owner, .. } | people::Act::RemoveProject { project: owner } => {
                            *owner == project
                        }
                        people::Act::AddKey { uses, .. } => uses.as_ref().is_some_and(|uses| {
                            uses.iter().any(|payer| match payer {
                                people::Use::Project(owner) => *owner == project,
                                people::Use::Party(_) => false,
                            })
                        }),
                        people::Act::MakeProject { .. } | people::Act::RemoveAccount { .. } => false,
                    },
                    "Acted",
                ),
                // Retained party answers may name any deleted task or project.
                people::Stored::Answer { .. } => (true, "Answer"),
                people::Stored::Person { .. } | people::Stored::SignIn { .. } | people::Stored::ReadResults { .. } => {
                    (false, "People")
                }
            },
            core::Record::Notes(row) => match row {
                notes::Record::Entry(row) => (
                    scope(&row.scope, project, task_numbers)
                        || row.references.iter().any(|task| task_numbers.contains(task))
                        || match row.author {
                            notes::Author::Task { task, .. } => task_numbers.contains(&task),
                            notes::Author::Party { .. } => false,
                        },
                    match row.scope {
                        notes::Scope::Project { .. } => "ProjectNote",
                        notes::Scope::Goal { .. } => "GoalNote",
                        notes::Scope::Resources { .. } => "ResourcesNote",
                        notes::Scope::Deployment => "DeploymentNote",
                    },
                ),
                notes::Record::Line(row) => (scope(&row.scope, project, task_numbers), "NoteLine"),
            },
            core::Record::Accounts(row) => match row {
                accounts::Stored::Account { uses, .. } => (
                    uses.as_ref().is_some_and(|uses| {
                        uses.iter().any(|payer| match payer {
                            accounts::Use::Project(owner) => *owner == project,
                            accounts::Use::Party(_) => false,
                        })
                    }),
                    "AccountUse",
                ),
                accounts::Stored::Next { .. } | accounts::Stored::Issued { .. } | accounts::Stored::Source { .. } => {
                    (false, "Accounts")
                }
            },
        },
        root::Record::Connector { record, .. } => match record {
            connector::Record::Adoption { project: owner, .. } => (*owner == project, "Adoption"),
            connector::Record::Task { project: owner, task, .. } => {
                (*owner == project || task_numbers.contains(task), "ConnectorTask")
            }
            connector::Record::Proposal { project: owner, task, .. } => {
                (*owner == project || task_numbers.contains(task), "ConnectorProposal")
            }
            connector::Record::Procedure(row) => (
                row.project == project
                    || task_numbers.contains(&row.task)
                    || row.delegate.is_some_and(|task| task_numbers.contains(&task)),
                "Procedure",
            ),
            connector::Record::ProcedureDelegateOutcome { task, child, revisions, .. } => (
                task_numbers.contains(task)
                    || child.is_some_and(|task| task_numbers.contains(&task))
                    || revisions.iter().any(|(task, _)| task_numbers.contains(task)),
                "ProcedureDelegateOutcome",
            ),
            connector::Record::Result { task, .. } | connector::Record::Subscription { task, .. } => {
                (task_numbers.contains(task), "ConnectorResultOrSubscription")
            }
            connector::Record::Outbox(row) => {
                (task_numbers.contains(&row.task) || task_numbers.contains(&row.key.task), "Outbox")
            }
            connector::Record::Made { key, .. } => (task_numbers.contains(&key.task), "Made"),
            connector::Record::ApplicationObject { .. }
            | connector::Record::ApplicationAnswer { .. }
            | connector::Record::Pool { .. } => (false, "ConnectorGlobal"),
        },
    };
    names.then_some(family)
}

pub(super) fn filled(world: &World) -> Result<(), Violation> {
    let task_numbers = &world.peers.deletion.tasks;
    check(super::task(world, world.peers.deletion.parent).tracked.is_some(), "goal scope must name a tracked goal")?;
    check(
        world.store.rows.values().any(|row| {
            matches!(row,
        root::Record::Core(core::Record::People(people::Stored::Answer {
            ask, outcome: people::Outcome::Renamed { project: PROJECT }, ..
        })) if matches!(&**ask, people::Ask::Rename { project: PROJECT, .. }))
        }),
        "party rename answer missing",
    )?;
    let readback = world.store.fork().readback();
    for task in task_numbers {
        check(
            readback.values().any(|row| {
                matches!(row,
            root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.task == *task)
            }),
            "populated task has no transcript in store readback",
        )?;
    }
    let families: BTreeSet<_> =
        world.store.rows.values().filter_map(|row| family(row, PROJECT, task_numbers)).collect();
    for family in [
        "Live",
        "PeriodLedger",
        "PoolLedger",
        "PersonProposal",
        "DecidedProposal",
        "Ended",
        "History",
        "Milestone",
        "Activity",
        "Update",
        "Result",
        "EndedDelegate",
        "Delegate",
        "EndedByParty",
        "EndedByProject",
        "PlacedHold",
        "Terminal",
        "ProposalDecision",
        "Turn",
        "Roles",
        "Policy",
        "ReadConversation",
        "EndedRead",
        "Answer",
        "Acted",
        "ProjectNote",
        "GoalNote",
        "ResourcesNote",
        "AccountUse",
        "Adoption",
        "ConnectorTask",
        "ConnectorProposal",
    ] {
        check(families.contains(family), &format!("deletion fill missing {family}; present: {families:?}"))?;
    }
    for row in world.store.rows.values() {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row))) = row
            && funder(row.funder, PROJECT, task_numbers)
        {
            check(row.numbers.spent + row.numbers.spent_below > 0, "project period and pool must have settled spend")?;
        }
    }
    check(!tasks(world, CONTROL).is_empty(), "control project has no task")
}
