//! An independent project census over records written through the real codecs.
use jig_core_world::{Store, effects::World, fixtures::live_row, observations::naming::naming};
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, Token, Wall};
use std::collections::BTreeMap;

const PROJECT: u32 = 3;
const TASK: u64 = 1;
const ENDED: u64 = 2;

fn store() -> Store {
    let (config, limits) = jig_core_world::effects::fixture(301, false);
    let mut store = Store::new(&limits, &config.core.settings.models);
    let header = core::Deployment {
        id: [0; 16],
        tasks: ENDED,
        people: 1,
        sign_ins: 0,
        messages: 1,
        runs: 1,
        calls: 1,
        connector_rows: 0,
        projects: 4,
        applications: 0,
        commits: 1,
    };
    store.submit(
        1,
        Box::new([
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))),
            root::Write::Wires,
        ]),
        0,
    );
    assert_eq!(store.tick(false), Ok(Some(1)));
    store
}

fn header(store: &Store, number: u64) -> root::Write {
    let key = root::Key::Core(core::Key::Core(core::CoreKey::Deployment));
    let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(mut header))) = store.rows[&key] else {
        panic!("fixture header")
    };
    header.commits = number;
    root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))))
}

fn save(store: &mut Store, row: core::Record) {
    let number = store.applied + 1;
    store.submit(number, Box::new([header(store, number), root::Write::Save(root::Record::Core(row))]), 0);
    assert_eq!(store.tick(false), Ok(Some(number)), "codec-backed commit");
}

#[expect(clippy::too_many_lines, reason = "one inventory names each durable record kind and its concrete fields")]
fn rows(project: u32) -> Vec<(&'static str, core::Record)> {
    let world = World::new(301, false);
    let mut live = live_row(&world, TASK).clone();
    live.project = project;
    let mut ended = live.clone();
    ended.number = ENDED;
    ended.root = ENDED;
    ended.phase = tasks::Phase::Ended(tasks::Ending::Cancelled {
        by: tasks::Party::Person(1),
        at: Wall::EPOCH,
        reason: Box::new([]),
        result: None,
    });
    ended.ended_at = Some(Wall::EPOCH);
    let proof = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(row))) => Some(row.clone()),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("fixture claim");
    let policy = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::People(people::Stored::Policy { value, .. })) => Some(value.clone()),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("fixture policy");
    let proposal = tasks::PersonProposal {
        number: 1,
        proposer: 1,
        project,
        action: tasks::PersonAction::Settings { against: 1, items: Box::new([]) },
        reason: Box::new([]),
        state: tasks::PersonProposalState::Pending { since: Wall::EPOCH },
    };
    let scope = notes::Scope::Goal { project, goal: TASK };
    let task_rows = vec![
        ("Live", tasks::Stored::Live(Box::new(live))),
        ("Ended", tasks::Stored::Ended(Box::new(ended))),
        (
            "History",
            tasks::Stored::History(tasks::History {
                task: ENDED,
                revision: 1,
                by: tasks::Party::Person(1),
                change: tasks::Change::Cancelled,
                reason: Box::new([]),
                proposal: None,
                at: Wall::EPOCH,
                amendment: None,
            }),
        ),
        (
            "Milestone",
            tasks::Stored::Milestone(tasks::Milestone {
                task: TASK,
                position: 1,
                phase: tasks::Phase::Waiting,
                at: Wall::EPOCH,
            }),
        ),
        (
            "Activity",
            tasks::Stored::Activity(tasks::ActivityLine {
                task: TASK,
                position: 1,
                at: Wall::EPOCH,
                line: tasks::Line::Milestone { position: 1 },
            }),
        ),
        (
            "Stub",
            tasks::Stored::Stub(tasks::Stub { task: ENDED, phase: tasks::Status::Done, result: Token::new(ENDED) }),
        ),
        (
            "Update",
            tasks::Stored::Update(tasks::Update {
                party: 1,
                at: Wall::EPOCH,
                task: TASK,
                object: tasks::UpdateObject::Result,
            }),
        ),
        ("Result", tasks::Stored::Result(tasks::ResultRow { party: 1, position: 1, task: ENDED })),
        (
            "EndedByParty",
            tasks::Stored::EndedByParty(tasks::EndedByParty { party: 1, label: 0, ended_at: Wall::EPOCH, task: ENDED }),
        ),
        (
            "EndedByProject",
            tasks::Stored::EndedByProject(tasks::EndedByProject {
                project,
                label: 0,
                ended_at: Wall::EPOCH,
                task: ENDED,
            }),
        ),
        ("EndedDelegate", tasks::Stored::EndedDelegate(tasks::Edge { requester: TASK, task: ENDED })),
        ("Delegate", tasks::Stored::Delegate(tasks::Edge { requester: TASK, task: ENDED })),
        (
            "PlacedHold",
            tasks::Stored::PlacedHold(Box::new(tasks::PlacedHold {
                number: 1,
                project,
                owner: tasks::Owner::Task { task: TASK },
                mode: tasks::Mode::Stop,
                scope: tasks::Scope::Subtree { task: TASK },
                reason: Box::new([]),
                placed_at: 0,
                revision: 1,
                exempt: Box::new([]),
                lifted: false,
            })),
        ),
        ("PersonProposal", tasks::Stored::PersonProposal(Box::new(proposal.clone()))),
        (
            "DecidedProposal",
            tasks::Stored::DecidedProposal(Box::new(tasks::PersonProposal {
                state: tasks::PersonProposalState::Withdrawn { at: Wall::EPOCH },
                ..proposal
            })),
        ),
        (
            "Writer",
            tasks::Stored::Writer(tasks::WriterSlot {
                number: 1,
                resource: tasks::Name { connector: 1, path: Box::new([b"one".as_slice().into()]) },
                writer: tasks::Writer::Run { task: TASK, attempt: 1 },
                lost: false,
            }),
        ),
    ];
    let mut rows: Vec<_> = task_rows.into_iter().map(|(name, row)| (name, core::Record::Tasks(row))).collect();
    let core_rows = vec![
        (
            "ProjectionGoal",
            core::CoreRecord::Projection(core::ProjectionRecord::Goal {
                goal: core::ProjectionGoal {
                    number: TASK,
                    project,
                    priority: 1,
                    words: Box::new([]),
                    phase: tasks::Phase::Waiting,
                },
                pending: Box::new([]),
                changed: false,
                fed_at: None,
            }),
        ),
        (
            "ProjectionTask",
            core::CoreRecord::Projection(core::ProjectionRecord::Task {
                goal: TASK,
                task: core::ProjectionTask {
                    number: ENDED,
                    requester: tasks::Party::Task(TASK),
                    words: Box::new([]),
                    phase: tasks::Phase::Waiting,
                },
            }),
        ),
        (
            "ProjectionMilestone",
            core::CoreRecord::Projection(core::ProjectionRecord::Milestone {
                goal: TASK,
                milestone: core::ProjectionMilestone {
                    identity: core::MilestoneId::Lifecycle { task: ENDED, position: 1 },
                    phase: Some(tasks::Phase::Waiting),
                    change: None,
                    words: Box::new([]),
                },
            }),
        ),
        ("RunProof", core::CoreRecord::RunProof(proof)),
        (
            "Call",
            core::CoreRecord::Call(core::CallRecord {
                key: core::CallKey { task: TASK, attempt: 1, completion: 1, position: 0 },
                part: core::CallPart::Unavailable,
                settled: None,
            }),
        ),
        (
            "Terminal",
            core::CoreRecord::Terminal(core::TerminalRecord {
                task: TASK,
                attempt: 1,
                cumulative: 0,
                end: tasks::End::Parked,
            }),
        ),
        (
            "EscalationDecision",
            core::CoreRecord::EscalationDecision(core::EscalationDecisionRecord {
                project,
                requester: 1,
                task: TASK,
                revision: 1,
                by: 1,
                at: Wall::EPOCH,
                decision: people::EscalationDecision::Release,
            }),
        ),
        (
            "ProposalDecision",
            core::CoreRecord::ProposalDecision(core::ProposalDecisionRecord {
                project,
                proposer: people::Proposer::Task(TASK),
                proposal: 1,
                kind: tasks::ProposalKind::Settings,
                by: 1,
                at: Wall::EPOCH,
                choice: people::ProposalChoice::Withdrawn,
                created: None,
            }),
        ),
        (
            "SettingsProposal",
            core::CoreRecord::Settings(core::settings::Record::Proposal {
                proposer: people::Proposer::Task(TASK),
                proposal: 1,
                value: core::settings::Proposal { against: 1, items: Box::new([]) },
            }),
        ),
        (
            "Turn",
            core::CoreRecord::Turn(core::TurnRecord {
                opens_window: true,
                task: TASK,
                attempt: 1,
                turn: 1,
                spent: 0,
                read: None,
                at: Wall::EPOCH,
                transcript: b"turn".as_slice().into(),
            }),
        ),
    ];
    rows.extend(core_rows.into_iter().map(|(name, row)| (name, core::Record::Core(row))));
    rows.extend([
        ("Roles", core::Record::People(people::Stored::Roles { project, holdings: Box::new([]) })),
        ("Policy", core::Record::People(people::Stored::Policy { project, name: "Example".into(), value: policy })),
        (
            "ReadConversation",
            core::Record::People(people::Stored::ReadConversation { party: 1, task: TASK, through: 1 }),
        ),
        ("EndedRead", core::Record::People(people::Stored::EndedRead { party: 1, task: ENDED, through: 1 })),
        (
            "Line",
            core::Record::Notes(notes::Record::Line(notes::Line {
                scope: scope.clone(),
                name: 1,
                description: Box::new([]),
                revision: 1,
            })),
        ),
        (
            "Entry",
            core::Record::Notes(notes::Record::Entry(notes::Entry {
                name: 1,
                scope,
                description: Box::new([]),
                body: Box::new([]),
                references: List::with_capacity(0),
                author: notes::Author::Task { task: TASK, attempt: 1 },
                revision: 1,
                changed: Wall::EPOCH,
            })),
        ),
        (
            "Account",
            core::Record::Accounts(accounts::Stored::Account {
                number: 1,
                provider: 1,
                holder: accounts::Holder::Deployment,
                place: 0,
                uses: Some(Box::new([accounts::Use::Project(project)])),
                subject: Box::new([]),
                display: Box::new([]),
            }),
        ),
    ]);
    rows
}

fn ownership(rows: &[(&str, core::Record)], project: u32) -> BTreeMap<u64, u32> {
    let mut store = store();
    for (_, row) in rows.iter().take(2) {
        save(&mut store, row.clone());
    }
    let mut tasks = BTreeMap::new();
    assert_eq!(naming(&store, &mut tasks), [(project, 2)].into());
    assert_eq!(tasks, [(TASK, project), (ENDED, project)].into());
    tasks
}

#[test]
fn every_family_row_naming_a_project_is_counted() {
    let rows = rows(PROJECT);
    let mut tasks = ownership(&rows, PROJECT);
    for (name, row) in rows {
        let mut store = store();
        let key = root::Key::Core(row.key());
        save(&mut store, row);
        assert!(store.readback().contains_key(&key), "{name} passed the codecs");
        // A turn creates a transcript place and an attempt place as well as its frame.
        let expected = if name == "Turn" { 3 } else { 1 };
        assert_eq!(naming(&store, &mut tasks), [(PROJECT, expected)].into(), "{name}");
    }
}

#[test]
fn another_projects_rows_are_not_counted() {
    let rows = rows(4);
    let mut tasks = ownership(&rows, 4);
    for (name, row) in rows {
        let mut store = store();
        save(&mut store, row);
        let counts = naming(&store, &mut tasks);
        assert!(!counts.contains_key(&PROJECT), "{name}");
        assert_eq!(counts.get(&4), Some(&(if name == "Turn" { 3 } else { 1 })), "{name}");
    }
}

#[test]
fn ledgers_and_retained_answers_are_not_counted() {
    let rows = rows(PROJECT);
    let mut tasks = ownership(&rows, PROJECT);
    let mut store = store();
    let numbers = tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    let ledger = tasks::FundingRecord {
        period: None,
        funder: tasks::Funder::Pool { project: PROJECT, person: 1, period: 1 },
        parent: None,
        numbers,
        made: 0,
        closed: false,
    };
    for row in [
        core::Record::Tasks(tasks::Stored::Ledger(ledger)),
        core::Record::Tasks(tasks::Stored::ClosedLedger(tasks::FundingRecord { closed: true, ..ledger })),
        core::Record::People(people::Stored::Answer {
            key: people::RequestKey { person: 1, key: [1; 16] },
            ask: Box::new(people::Ask::Rename { project: PROJECT, name: "Example".into() }),
            models: Box::new([]),
            outcome: people::Outcome::ProjectRemoved { project: PROJECT },
            at: Wall::EPOCH,
        }),
        core::Record::People(people::Stored::Acted {
            key: people::ActKey { connector: 1, key: [2; 16] },
            act: people::Act::RemoveProject { project: PROJECT },
            outcome: people::Outcome::ProjectRemoved { project: PROJECT },
            at: Wall::EPOCH,
        }),
    ] {
        save(&mut store, row);
    }
    assert_eq!(store.readback().len(), 5, "excluded rows are durable");
    assert!(naming(&store, &mut tasks).is_empty());
}

#[test]
fn an_erased_owners_history_and_transcript_places_still_name_its_project() {
    let rows = rows(PROJECT);
    let mut tasks = BTreeMap::new();
    let mut store = store();
    for (_, row) in rows.iter().take(2) {
        save(&mut store, row.clone());
    }
    assert_eq!(naming(&store, &mut tasks), [(PROJECT, 2)].into());
    for name in ["History", "Turn"] {
        save(&mut store, rows.iter().find(|(kind, _)| *kind == name).expect("inventory kind").1.clone());
    }
    let number = store.applied + 1;
    store.submit(
        number,
        Box::new([
            header(&store, number),
            root::Write::Erase(root::Key::Core(core::Key::Tasks(tasks::Key::Live(TASK)))),
            root::Write::Erase(root::Key::Core(core::Key::Tasks(tasks::Key::Ended(ENDED)))),
        ]),
        0,
    );
    assert_eq!(store.tick(false), Ok(Some(number)));
    assert_eq!(naming(&store, &mut tasks), [(PROJECT, 4)].into(), "history, turn, transcript and attempt survive");
    store.crash(302);
    assert_eq!(naming(&store, &mut tasks), [(PROJECT, 4)].into(), "observer ownership survives the restart");
}

#[test]
fn one_account_row_is_counted_once_for_each_project_it_names() {
    let mut store = store();
    save(
        &mut store,
        core::Record::Accounts(accounts::Stored::Account {
            number: 1,
            provider: 1,
            holder: accounts::Holder::Deployment,
            place: 0,
            uses: Some(Box::new([accounts::Use::Project(PROJECT), accounts::Use::Project(4)])),
            subject: Box::new([]),
            display: Box::new([]),
        }),
    );
    assert_eq!(naming(&store, &mut BTreeMap::new()), [(PROJECT, 1), (4, 1)].into());
}
