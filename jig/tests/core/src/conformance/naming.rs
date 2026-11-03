//! The testing connector's durable ownership and the shared core projection.
use super::{connector, r, root};

/// Project and task ownership in the testing root's durable rows.
#[must_use]
pub fn naming(key: &root::Key, record: &root::Record) -> Option<r::Naming> {
    let row = format!("{key:?}");
    let root::Key::Connector { key, .. } = key else {
        let (root::Key::Core(key), root::Record::Core(record)) = (key, record) else { return None };
        let mut name = jig_conformance::core::naming::naming(key, record);
        if let Some(name) = &mut name {
            name.row = row;
        }
        return name;
    };
    let mut name = r::Naming { row, project: None, task: None, outlives: None };
    match key {
        connector::RecordKey::SettingDefault { scope, .. }
        | connector::RecordKey::SettingChange { scope, .. }
        | connector::RecordKey::SettingProposal { scope, .. } => name.project = (*scope != 0).then_some(*scope),
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
    }
    if let root::Record::Connector { record, .. } = record {
        match record {
            connector::Record::SettingDefault { scope, .. }
            | connector::Record::SettingChange { scope, .. }
            | connector::Record::SettingProposal { scope, .. } => name.project = (*scope != 0).then_some(*scope),
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
        }
    }
    (name.project.is_some() || name.task.is_some()).then_some(name)
}
#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_accounts as accounts;
    use jig_domain_core as core;
    use jig_domain_notes as notes;
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;
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
            (connector::RecordKey::SettingDefault { kind: 1, scope: 0, number: 1 }, None, None),
            (connector::RecordKey::SettingChange { kind: 1, scope: 0, number: 1 }, None, None),
            (connector::RecordKey::SettingProposal { proposal: 1, kind: 1, scope: 0, number: 1 }, None, None),
            (connector::RecordKey::SettingDefault { kind: 1, scope: project, number: 1 }, Some(project), None),
            (connector::RecordKey::SettingChange { kind: 1, scope: project, number: 1 }, Some(project), None),
            (
                connector::RecordKey::SettingProposal { proposal: 1, kind: 1, scope: project, number: 1 },
                Some(project),
                None,
            ),
            (connector::RecordKey::ApplicationAnswer(Token::new(1)), None, None),
            (connector::RecordKey::Procedure(task), None, Some(task)),
            (connector::RecordKey::ProcedureDelegateOutcome(task), None, Some(task)),
            (connector::RecordKey::Result(task), None, Some(task)),
            (connector::RecordKey::Task(task), None, Some(task)),
            (
                connector::RecordKey::Adoption { project, path: crate::world::World::effect().resources[0].clone() },
                Some(project),
                None,
            ),
            (connector::RecordKey::Subscription { topic: 1, task }, None, Some(task)),
            (connector::RecordKey::Pool(crate::world::World::effect().resources[0].clone()), None, None),
            (connector::RecordKey::Proposal(1), None, None),
            (connector::RecordKey::Outbox(effect), None, Some(task)),
            (connector::RecordKey::Made(effect), None, Some(task)),
        ] {
            cases.push((root::Key::Connector { number: 1, key }, owner, task, None));
        }
        assert_eq!(cases.len(), 71, "all store key kinds, including both application scopes");
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
                act: people::Act::RemoveProject { project, grace: None },
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
