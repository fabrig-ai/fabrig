//! Every durable family names its owners (jig/docs/design/domain/people.md, 5.2).
use super::{connector, core, people, r, root, tasks};
use jig_domain_accounts as accounts;
use jig_domain_notes as notes;

/// Project and task names carried by a durable row, independently of live arenas.
#[must_use]
pub fn naming(key: &root::Key, record: &root::Record) -> Option<r::Naming> {
    let mut name = r::Naming { row: format!("{key:?}"), project: None, task: None, outlives: None };
    match key {
        root::Key::Core(core::Key::Core(key)) => match key {
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
        root::Key::Core(core::Key::Tasks(key)) => match key {
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
        root::Key::Core(core::Key::People(key)) => match key {
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
        root::Key::Core(core::Key::Notes(key)) => match key {
            notes::Key::Entry { .. } => {}
            notes::Key::Line { scope, .. } => note_scope(scope, &mut name),
        },
        root::Key::Core(core::Key::Accounts(key)) => match key {
            accounts::Key::Next
            | accounts::Key::Issued { .. }
            | accounts::Key::Account { .. }
            | accounts::Key::Source { .. } => {}
        },
        root::Key::Connector { key, .. } => match key {
            connector::RecordKey::Adoption { project, .. } => name.project = Some(*project),
            connector::RecordKey::Procedure(task)
            | connector::RecordKey::ProcedureDelegateOutcome(task)
            | connector::RecordKey::Result(task)
            | connector::RecordKey::Task(task)
            | connector::RecordKey::Subscription { task, .. } => name.task = Some(*task),
            connector::RecordKey::Outbox(key) | connector::RecordKey::Made(key) => name.task = Some(key.task),
            connector::RecordKey::ApplicationObject(_)
            | connector::RecordKey::ApplicationAnswer(_)
            | connector::RecordKey::Pool(_)
            | connector::RecordKey::Proposal(_) => {}
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

fn record_names(record: &root::Record, name: &mut r::Naming) {
    match record {
        root::Record::Core(core::Record::Core(record)) => match record {
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
        root::Record::Core(core::Record::Tasks(record)) => match record {
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
        root::Record::Core(core::Record::People(record)) => match record {
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
        root::Record::Core(core::Record::Notes(record)) => match record {
            // Entries are globally named; project ownership is carried by index lines.
            notes::Record::Entry(_) => {}
            notes::Record::Line(row) => note_scope(&row.scope, name),
        },
        root::Record::Core(core::Record::Accounts(record)) => match record {
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
        root::Record::Connector { record, .. } => match record {
            connector::Record::Proposal { project, task, .. } | connector::Record::Task { project, task, .. } => {
                name.project = Some(*project);
                name.task = Some(*task);
            }
            connector::Record::Procedure(row) => name.project = Some(row.project),
            connector::Record::ApplicationObject { .. }
            | connector::Record::ApplicationAnswer { .. }
            | connector::Record::ProcedureDelegateOutcome { .. }
            | connector::Record::Result { .. }
            | connector::Record::Adoption { .. }
            | connector::Record::Subscription { .. }
            | connector::Record::Pool { .. }
            | connector::Record::Outbox(_)
            | connector::Record::Made { .. } => {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Token, Wall};

    #[test]
    #[expect(clippy::too_many_lines, reason = "one table covers every key kind independently of record ownership")]
    fn naming_names_every_family() {
        use jig_domain_core::{CoreKey as C, Key as K};
        use tasks::Key as T;
        let task = 17;
        let project = 9;
        let proposer = people::Proposer::Task(task);
        let call = core::CallKey { task, attempt: 1, completion: 1, position: 0 };
        let effect = connector::Key {
            deployment: [1; 16],
            task,
            origin: connector::Purpose::Call { attempt: 1, completion: 1, position: 0 },
            purpose: 1,
        };
        let request = people::RequestKey { person: 1, key: [1; 16] };
        let acted = people::ActKey { connector: 1, key: [1; 16] };
        // A global record supplies no ownership: these rows test each key's own names.
        let record =
            root::Record::Core(core::Record::Core(core::CoreRecord::Settings(core::settings::Record::Extent {
                oldest: 1,
            })));
        let mut cases = Vec::new();
        for (key, expected) in [
            (C::Deployment, None),
            (C::RunProof(task), Some(task)),
            (C::Call(call), Some(task)),
            (C::Turn { task, attempt: 1, turn: 1 }, Some(task)),
            (C::Terminal { task, attempt: 1 }, Some(task)),
            (C::EscalationDecision { task, revision: 1 }, Some(task)),
            (C::ProposalDecision { proposer, proposal: 1 }, Some(task)),
            (C::Projection(core::ProjectionKey::Goal(task)), Some(task)),
            (C::Projection(core::ProjectionKey::Task { goal: 11, task }), Some(task)),
            (C::Projection(core::ProjectionKey::Lifecycle { goal: 11, task, position: 1 }), Some(task)),
            (C::Projection(core::ProjectionKey::Revision { goal: 11, task, revision: 1 }), Some(task)),
        ] {
            cases.push((root::Key::Core(K::Core(key)), None, expected, None));
        }
        for (key, owner, task) in [
            (core::settings::Key::Default(people::Setting::Policy(project)), None, None),
            (core::settings::Key::Changed(people::Setting::Policy(project)), None, None),
            (core::settings::Key::Held(1), None, None),
            (core::settings::Key::Charter { charter: 1, revision: 1 }, None, None),
            (core::settings::Key::Proposal { proposer, proposal: 1 }, None, Some(task)),
            (core::settings::Key::Extent, None, None),
            (core::settings::Key::History(1), None, None),
        ] {
            cases.push((root::Key::Core(K::Core(C::Settings(key))), owner, task, None));
        }
        for (key, owner, task, outlives) in [
            (T::Live(task), None, Some(task), None),
            (T::Ended(task), None, Some(task), None),
            (T::Stub(task), None, Some(task), None),
            (T::Ledger(tasks::Funder::Period { project, period: 1 }), Some(project), None, None),
            (
                T::ClosedLedger(tasks::Funder::Period { project, period: 1 }),
                Some(project),
                None,
                Some(r::Outlives::Spend),
            ),
            (T::Writer(1), None, None, None),
            (T::Pool(1), None, None, None),
            (T::PersonProposal(1), None, None, None),
            (T::DecidedProposal(1), None, None, None),
            (T::History { task, revision: 1 }, None, Some(task), None),
            (T::Milestone { task, position: 1 }, None, Some(task), None),
            (T::Activity { task, position: 1 }, None, Some(task), None),
            (
                T::Update {
                    party: 1,
                    at: tasks::Newest::of(1),
                    task: tasks::Newest::of(task),
                    object: tasks::UpdateObject::Result,
                },
                None,
                Some(task),
                None,
            ),
            (T::Result { party: 1, position: 1 }, None, None, None),
            (T::EndedDelegate { requester: 11, task }, None, Some(task), None),
            (T::Delegate { requester: 11, task }, None, Some(task), None),
            (T::EndedByParty { party: 1, label: 0, ended_at: tasks::Newest::of(1), task }, None, Some(task), None),
            (
                T::EndedByProject { project, label: 0, ended_at: tasks::Newest::of(1), task },
                Some(project),
                Some(task),
                None,
            ),
            (T::PlacedHold(1), None, None, None),
        ] {
            cases.push((root::Key::Core(K::Tasks(key)), owner, task, outlives));
        }
        for (key, owner, task) in [
            (people::Key::Person(1), None, None),
            (people::Key::SignIn(1), None, None),
            (people::Key::ReadResults { party: 1 }, None, None),
            (people::Key::ReadConversation { party: 1, task }, None, Some(task)),
            (people::Key::EndedRead { party: 1, task }, None, Some(task)),
            (people::Key::Roles(project), Some(project), None),
            (people::Key::Policy(project), Some(project), None),
            (people::Key::Deletion(project), Some(project), None),
            (people::Key::Answer(request), None, None),
            (people::Key::Acted(acted), None, None),
        ] {
            cases.push((root::Key::Core(K::People(key)), owner, task, None));
        }
        for (key, owner) in [
            (notes::Key::Entry { name: 1 }, None),
            (notes::Key::Line { scope: notes::Scope::Project { project }, name: 1 }, Some(project)),
        ] {
            cases.push((root::Key::Core(K::Notes(key)), owner, None, None));
        }
        for key in [
            accounts::Key::Next,
            accounts::Key::Account { number: 1 },
            accounts::Key::Source { account: 1, number: 2 },
            accounts::Key::Issued { number: 1 },
        ] {
            cases.push((root::Key::Core(K::Accounts(key)), None, None, None));
        }
        for (key, owner, task) in [
            (connector::RecordKey::ApplicationObject(1), None, None),
            (connector::RecordKey::ApplicationAnswer(Token::new(1)), None, None),
            (connector::RecordKey::Procedure(task), None, Some(task)),
            (connector::RecordKey::ProcedureDelegateOutcome(task), None, Some(task)),
            (connector::RecordKey::Result(task), None, Some(task)),
            (connector::RecordKey::Task(task), None, Some(task)),
            (
                connector::RecordKey::Adoption { project, path: crate::effects::World::effect().resources[0].clone() },
                Some(project),
                None,
            ),
            (connector::RecordKey::Subscription { topic: 1, task }, None, Some(task)),
            (connector::RecordKey::Pool(crate::effects::World::effect().resources[0].clone()), None, None),
            (connector::RecordKey::Proposal(1), None, None),
            (connector::RecordKey::Outbox(effect), None, Some(task)),
            (connector::RecordKey::Made(effect), None, Some(task)),
        ] {
            cases.push((root::Key::Connector { number: 1, key }, owner, task, None));
        }
        assert_eq!(cases.len(), 65, "all store key kinds");
        for (key, project, task, outlives) in cases {
            let expected = (project.is_some() || task.is_some()).then(|| r::Naming {
                row: format!("{key:?}"),
                project,
                task,
                outlives,
            });
            assert_eq!(naming(&key, &record), expected, "{key:?}");
        }
        let account = |uses: Box<[accounts::Use]>| {
            root::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                number: 1,
                provider: 1,
                holder: accounts::Holder::Deployment,
                place: 0,
                uses: Some(uses),
                subject: Box::new([]),
                display: Box::new([]),
            }))
        };
        for (uses, outlives) in [
            (Box::new([accounts::Use::Project(project)]) as Box<[_]>, Some(r::Outlives::SoleUse)),
            (Box::new([accounts::Use::Party(1), accounts::Use::Project(project)]), None),
        ] {
            let record = account(uses);
            let got = naming(&record.key(), &record).expect("account project from its uses");
            assert_eq!((got.project, got.outlives), (Some(project), outlives));
        }
        for record in [
            root::Record::Core(core::Record::People(people::Stored::Answer {
                key: request,
                ask: Box::new(people::Ask::Say { project, task, words: Box::new([]) }),
                models: Box::new([]),
                outcome: people::Outcome::Said { task, message: 1 },
                at: Wall::EPOCH,
            })),
            root::Record::Core(core::Record::People(people::Stored::Acted {
                key: acted,
                act: people::Act::RemoveProject { project },
                outcome: people::Outcome::ProjectRemoved { project },
                at: Wall::EPOCH,
            })),
            root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
                scope: notes::Scope::Project { project },
                name: 1,
                description: Box::new([]),
                revision: 1,
            }))),
        ] {
            assert_eq!(naming(&record.key(), &record).expect("record carries its project").project, Some(project));
        }
    }
}
