//! Owner reads and task-query observations at ops' store boundary.
use super::{
    Action, ReplyTo, Store, Token, Violation, World, chat, check, ended, finish, people, root, state, state_mut, tasks,
};
use jig_domain_core::engine::store as engine_store;
use jig_domain_core::{self as core, documents, reads as transport};
fn query() -> documents::TaskQuery {
    jig_test_profile::fixtures::own_tasks()
}
pub(super) fn action(world: &mut World, action: Action) -> Result<(), Violation> {
    match action {
        Action::WatchQuery => {
            if state(world).children.is_empty() {
                for key in [110, 111] {
                    chat(world, key, b"query task");
                    world.drain()?;
                }
                let children = world.peers.assignments.iter().map(|(task, _)| *task).collect();
                state_mut(world).children = children;
            }
            world.send(root::Event::Core(core::Event::Watch {
                watcher: Token::new(7090),
                sign_in: world.peers.sign_in.expect("signed in"),
                subject: documents::Watched::Query { query: query() },
                window: documents::Window { rows: 4, after: None },
            }));
        }
        Action::EndTask => {
            let task = *state(world).children.iter().find(|&&task| !ended(world, task)).expect("live query task");
            finish(world, task)?;
        }
        Action::ReadEnded => {
            state_mut(world).found.clear();
            let mut cursor =
                Some(transport::Cursor::Store { scope: transport::Scope::Tasks { query: query() }, after: None });
            loop {
                world.send(root::Event::Core(core::Event::Read {
                    reply: ReplyTo::new(Token::new(7091)),
                    sign_in: world.peers.sign_in.expect("signed in"),
                    read: transport::Read::Tasks { query: query(), live: None, ended: cursor },
                }));
                world.drain()?;
                cursor = state_mut(world).next.take();
                if cursor.is_none() {
                    break;
                }
            }
            let indexed: Vec<_> = world
                .store
                .rows
                .keys()
                .filter_map(|key| match key {
                    root::Key::Core(core::Key::Tasks(tasks::Key::EndedByParty { party: 1, task, .. })) => Some(*task),
                    root::Key::Core(_) | root::Key::Observability(_) | root::Key::Infrastructure(_) => None,
                })
                .collect();
            check(
                state(world).found == indexed,
                &format!("ended query differs from its durable party index: {:?} != {indexed:?}", state(world).found),
            )?;
            state_mut(world).query_checks += 1;
        }
        Action::Compound
        | Action::CompoundAgain
        | Action::CancelWithGrace
        | Action::Finish
        | Action::Ready
        | Action::Late
        | Action::Effect(_)
        | Action::DeliverLate
        | Action::OtherHand
        | Action::HoldWrites
        | Action::ReleaseWrites
        | Action::CheckDeadline
        | Action::ChangeJudge
        | Action::FillPool
        | Action::ShrinkPool
        | Action::FinishHolder(_)
        | Action::Propose
        | Action::NarrowPolicy
        | Action::AcceptProposal
        | Action::StartStanding
        | Action::Period(_)
        | Action::Resume
        | Action::PartyAmend
        | Action::RunAmend
        | Action::Conversation { .. }
        | Action::Turn
        | Action::MarkTurns
        | Action::EndConversations
        | Action::MarkResults => unreachable!("query actions"),
    }
    Ok(())
}

pub(crate) fn point(store: &Store, range: &engine_store::Range) -> engine_store::Answer {
    let key = match range {
        engine_store::Range::Record(key) => Some(key.clone()),
        engine_store::Range::Proposal { task, proposal } => {
            match store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Live(*task)))) {
                Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) => {
                    row.proposal_rows.iter().find(|pointer| pointer.proposal == *proposal).map(|pointer| {
                        core::Key::Tasks(tasks::Key::History { task: *task, revision: pointer.revision })
                    })
                }
                _ => None,
            }
        }
        engine_store::Range::Read(_) | engine_store::Range::Window { .. } => unreachable!("not a point load"),
    };
    let record = key.and_then(|key| match store.rows.get(&root::Key::Core(key)) {
        Some(root::Record::Core(row)) => Some(row.clone()),
        _ => None,
    });
    let records: Box<[core::Record]> = match record {
        Some(row) => Box::new([row]),
        None => Box::new([]),
    };
    engine_store::Answer::Rows(engine_store::Rows::Records { records, more: false })
}

pub(crate) fn load(store: &Store, output: &root::Output) -> Option<root::Event> {
    match output {
        root::Output::Store(engine_store::Out::Load { load, range: engine_store::Range::Read(range) }) => {
            Some(page(store, *load, range))
        }

        root::Output::Store(engine_store::Out::Load {
            load,
            range: range @ (engine_store::Range::Record(_) | engine_store::Range::Proposal { .. }),
        }) => Some(root::Event::Store(root::Store::Engine(engine_store::In::Loaded {
            load: *load,
            result: point(store, range),
        }))),
        root::Output::ProjectMade { .. }
        | root::Output::ApplicationDocument { .. }
        | root::Output::ApplicationRead { .. }
        | root::Output::IncidentAssociationRefused { .. }
        | root::Output::Application { .. }
        | root::Output::Commit { .. }
        | root::Output::Switch
        | root::Output::Load { .. }
        | root::Output::CoreLoad(_)
        | root::Output::Core(_)
        | root::Output::Now(_)
        | root::Output::Observability(_)
        | root::Output::Infrastructure(_)
        | root::Output::ReadResult { .. }
        | root::Output::Read { .. }
        | root::Output::Agent(_)
        | root::Output::ToChild(_)
        | root::Output::Store(engine_store::Out::Load { range: engine_store::Range::Window { .. }, .. })
        | root::Output::Stop => None,
    }
}

fn page(store: &Store, load: Token, range: &transport::Range) -> root::Event {
    let mut rows = Vec::new();
    for record in store.rows.values() {
        let root::Record::Core(record) = record else { continue };
        if !match &range.scope {
            transport::Scope::Tasks { query } => transport::ended::position(query, &record.key()),
            transport::Scope::EndedTasks
            | transport::Scope::TaskSummaries
            | transport::Scope::Retention { .. }
            | transport::Scope::ErasureKeys { .. }
            | transport::Scope::ProposalRemains { .. }
            | transport::Scope::TaskOwnRows { .. }
            | transport::Scope::TaskRows { .. }
            | transport::Scope::SettingsHistory { .. }
            | transport::Scope::TaskText { .. }
            | transport::Scope::Task { .. }
            | transport::Scope::Proposal { .. }
            | transport::Scope::Escalation { .. }
            | transport::Scope::Question { .. }
            | transport::Scope::PersonTask { .. }
            | transport::Scope::Result { .. }
            | transport::Scope::Reply { .. }
            | transport::Scope::Turn { .. }
            | transport::Scope::EndedDelegates { .. }
            | transport::Scope::History { .. }
            | transport::Scope::Activity { .. }
            | transport::Scope::Inbox { .. }
            | transport::Scope::Notes { .. }
            | transport::Scope::ScopedNotes { .. }
            | transport::Scope::Spend { .. } => transport::admits(range.party, &range.scope, &record.key()),
        } {
            continue;
        }
        let number = match (&range.scope, record) {
            (
                transport::Scope::Tasks { .. },
                core::Record::Tasks(
                    tasks::Stored::EndedByParty(tasks::EndedByParty { task, .. })
                    | tasks::Stored::EndedByProject(tasks::EndedByProject { task, .. }),
                ),
            ) => *task,
            (
                transport::Scope::TaskText { task, .. } | transport::Scope::Task { task },
                core::Record::Tasks(tasks::Stored::Ended(row)),
            ) if row.number == *task => *task,
            _ => continue,
        };
        let owner = store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(number))));
        let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(owner)))) = owner else {
            continue;
        };
        let value = transport::Row {
            value: record.clone(),
            owner: Some(owner.clone()),
            summary: None,
            ancestors: Box::new([]),
            related: None,
            unjoined: None,
            ended_read: Some(
                store
                    .rows
                    .get(&root::Key::Core(core::Key::People(people::Key::EndedRead {
                        party: range.party,
                        task: number,
                    })))
                    .map_or(0, |record| match record {
                        root::Record::Core(core::Record::People(people::Stored::EndedRead { through, .. })) => *through,
                        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => 0,
                    }),
            ),
            linked: Box::new([]),
            inbox: None,
        };
        let key = transport::row_key(&value).expect("owner key");
        if range.after.as_ref().is_none_or(|after| if range.backwards { key < *after } else { key > *after }) {
            rows.push(value);
        }
    }
    rows.sort_by_key(|row| transport::row_key(row).expect("owner key"));
    if range.backwards {
        rows.reverse();
    }
    let more = rows.len() > skein_lib::length::usize_of(range.rows);
    rows.truncate(skein_lib::length::usize_of(range.rows));
    let next = if more { rows.last().and_then(transport::row_key) } else { None };
    root::Event::Store(root::Store::Engine(engine_store::In::Loaded {
        load,
        result: engine_store::Answer::Rows(engine_store::Rows::Read { rows: rows.into_boxed_slice(), next }),
    }))
}
