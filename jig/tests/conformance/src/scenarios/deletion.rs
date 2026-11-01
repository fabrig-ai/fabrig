//! Project deletion at every durable family boundary
//! (jig/docs/design/domain/testing.md, 5).
mod inventory;
use super::{World, ask, check};
use crate::Neighbours;
use jig_conformance::{referee::Violation, scenarios::Action};
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token, Wall};
use std::collections::{BTreeMap, BTreeSet};

const PROJECT: u32 = 2;
const CONTROL: u32 = 1;
const PARTY: u64 = 1;
const CONNECTOR: u16 = 1;
const NAME: &str = "Deletion populated";
const CHAT: &[u8] = b"deletion conversation";
const LIVE: &[u8] = b"deletion live conversation";
const HELD: &[u8] = b"deletion held conversation";
const CHILD_CALL: u32 = 1;
const EFFECT_CALL: u32 = 2;
const ADOPTION_WAIT: u64 = 6_000_000_000;
const RETRY_WAIT: u64 = 1_000_000_000;
const DELETE_KEY: u8 = 140;
const REUSE_KEY: u8 = 141;
// Accounts 0 and 1 name the fixture's existing account and source.
const ACCOUNT: u32 = 2;
const SOURCE: u32 = 3;
const PERIOD: u64 = 1;
// The fixture's project policy supplies 1,000 deployment units per period.
const PERIOD_BUDGET: u64 = 1000;
const SETTLED_SPEND: u64 = 1;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    tasks: BTreeSet<u64>,
    parent: u64,
    child: u64,
    live: u64,
    held: u64,
    control: BTreeMap<root::Key, root::Record>,
    stages: BTreeMap<people::Stage, u64>,
    removed: bool,
    reused: bool,
}

pub(crate) fn configure(config: &mut root::Config, limits: &mut root::Limits) {
    limits.core.authority.projects = 3;
    limits.core.people.projects = 4;
    limits.core.people.initial_owners = 3;
    limits.core.fleet.slots = 3;
    limits.core.accounts.accounts = 2;
    limits.core.tasks.inbox_messages = 8;
    limits.core.tasks.inbox_bytes = 1024;
    limits.core.people.inbox_recipients = 32;
    limits.core.people.requests = 64;
    limits.core.people.amendment_bytes = 1024;
    config.core.defaults.rules = authority::Domain::new(config.core.defaults.rules.clone(), limits.core.authority)
        .expect("deletion project capacity")
        .rules()
        .clone();
    let seed = &config.core.seed_projects[0];
    let first = core::SeedProject { number: seed.number, name: seed.name.clone(), policy: seed.policy.clone() };
    config.core.defaults.policies = skein_lib::Map::with_capacity(1);
    config.core.defaults.policies.insert(1, seed.policy.clone()).expect("replacement policy");
    let second = core::SeedProject { number: PROJECT, name: "Deletion".into(), policy: seed.policy.clone() };
    config.core.seed_projects = Box::new([first, second]);
    let identity = config.core.owners[0].identity.clone();
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        people::InitialOwner { project: PROJECT, identity: identity.clone() },
        people::InitialOwner { project: people::DEPLOYMENT, identity },
    ]);
    config.core.grants = Box::new([people::ActGrant {
        connector: CONNECTOR,
        may: Box::new([people::ActKind::Close, people::ActKind::RemoveProject, people::ActKind::MakeProject]),
    }]);
}

// Seed retained notes, settled period spend and a sourced project-only account through
// the same first journal commit as the world's existing account fixture.
pub(crate) fn fixture(rows: &mut Box<[root::Record]>) {
    let mut entries = rows.clone().into_vec();
    entries.push(root::Record::Core(core::Record::Accounts(accounts::Stored::Account {
        number: ACCOUNT,
        provider: 1,
        holder: accounts::Holder::Deployment,
        place: 1,
        uses: Some(Box::new([accounts::Use::Project(PROJECT)])),
        subject: Box::new([]),
        display: Box::new([]),
    })));
    entries.push(root::Record::Core(core::Record::Accounts(accounts::Stored::Source {
        account: ACCOUNT,
        number: SOURCE,
        place: 0,
        kind: accounts::Kind::Key,
        name: Box::new([]),
        held: None,
    })));
    for row in &mut entries {
        if let root::Record::Core(core::Record::Accounts(accounts::Stored::Next { number })) = row {
            *number = SOURCE + 1;
        }
    }
    entries.push(root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(tasks::FundingRecord {
        funder: tasks::Funder::Period { project: PROJECT, period: PERIOD },
        parent: None,
        period: Some(tasks::PeriodTimes { began: Wall::EPOCH, next: None }),
        numbers: tasks::Numbers { budget: PERIOD_BUDGET, spent: 0, spent_below: SETTLED_SPEND, reserved: 0 },
        made: 0,
        closed: false,
    }))));
    for project in [CONTROL, PROJECT] {
        for (index, scope) in [
            notes::Scope::Project { project },
            notes::Scope::Goal { project, goal: u64::from(project) },
            notes::Scope::Resources {
                project,
                connector: CONNECTOR,
                pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
            },
        ]
        .into_iter()
        .enumerate()
        {
            if project == CONTROL && matches!(scope, notes::Scope::Goal { .. }) {
                continue;
            }
            let name = u64::from(project) * 10 + u64::try_from(index).expect("three note scopes");
            entries.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(notes::Entry {
                name,
                scope: scope.clone(),
                description: b"guidance".as_slice().into(),
                body: b"project guidance".as_slice().into(),
                references: List::with_capacity(0),
                author: notes::Author::Party { party: PARTY },
                revision: 1,
                changed: Wall::EPOCH,
            }))));
            entries.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
                name,
                scope,
                description: b"guidance".as_slice().into(),
                revision: 1,
            }))));
        }
    }
    *rows = entries.into_boxed_slice();
}

fn act(world: &mut World, key: u8, act: people::Act) {
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(10_000 + u64::from(key))),
        connector: CONNECTOR,
        key: [key; 16],
        act: Box::new(act),
        carried: None,
    }));
}
fn create(world: &mut World, key: u8, words: &[u8]) {
    let mut ask_value = jig_core_world::fixtures::create_chat(
        world.peers.peers.creating.as_ref().expect("creation vocabulary"),
        PROJECT,
        0,
        words.into(),
    );
    if let people::Ask::Create { batch, .. } = &mut ask_value {
        batch[0].flags.listed = true;
        if words == LIVE {
            batch[0].parameters =
                Box::new([people::Parameter::Resource { name: 1, connector: CONNECTOR, resource: 1 }]);
        }
    }
    ask(world, key, ask_value);
}
fn task(world: &World, number: u64) -> &tasks::TaskRecord {
    [tasks::Key::Live(number), tasks::Key::Ended(number)]
        .into_iter()
        .find_map(|key| {
            if let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) =
                world.store.rows.get(&root::Key::Core(core::Key::Tasks(key)))
            {
                Some(&**row)
            } else {
                None
            }
        })
        .expect("story task retained")
}
fn named(world: &World, words: &[u8]) -> u64 {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.spec.words.as_ref() == words =>
            {
                Some(row.number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .expect("named deletion task")
}
fn running(world: &World, number: u64) -> (u64, u64) {
    *world
        .peers
        .assignments
        .iter()
        .rev()
        .find(|(task, attempt)| {
            *task == number
                && matches!(self::task(world, number).phase,
            tasks::Phase::Active(tasks::Active::Running { attempt: current }) if *attempt == current)
        })
        .unwrap_or_else(|| {
            panic!(
                "deletion task {number} not running: {:?}; assignments {:?}",
                self::task(world, number).phase,
                world.peers.assignments
            )
        })
}
fn message(world: &mut World, number: u64) {
    super::message(world, number);
}
fn propose(world: &mut World, key: u8) {
    let number = if world.peers.deletion.live == 0 { world.peers.deletion.parent } else { world.peers.deletion.live };
    ask(
        world,
        key,
        people::Ask::Propose {
            project: PROJECT,
            action: people::Action::Amend {
                task: number,
                revision: task(world, number).revisions.spec,
                amendment: Box::new(people::Amendment {
                    spec: Some(people::Spec {
                        asks: false,
                        words: CHAT.into(),
                        parameters: Box::new([]),
                        inputs: Box::new([]),
                    }),
                    ..jig_core_world::fixtures::person_unchanged(b"proposed deletion history")
                }),
            },
            reason: b"review before deleting".as_slice().into(),
        },
    );
}

#[expect(
    clippy::too_many_lines,
    reason = "one replayable fill continuation retains each outside action before draining"
)]
pub(super) fn advance(world: &mut World, action: Action, phase: &mut u8) -> Result<bool, Violation> {
    if action == Action::RemoveProject {
        match *phase {
            0 => act(world, DELETE_KEY, people::Act::RemoveProject { project: PROJECT }),
            1 => world.advance(ADOPTION_WAIT),
            2 => {
                check_deleted(world)?;
                act(world, DELETE_KEY, people::Act::RemoveProject { project: PROJECT });
            }
            3 | 4 => act(world, REUSE_KEY, people::Act::MakeProject { maker: PARTY, name: NAME.into(), policy: 1 }),
            _ => return Ok(false),
        }
        *phase += 1;
        return Ok(true);
    }
    match *phase {
        0 => world.send(root::Event::Core(core::Event::Period {
            project: PROJECT,
            period: PERIOD,
            budget: PERIOD_BUDGET,
            next: None,
        })),
        1 => create(world, 110, CHAT),
        2 | 5 | 21 => world.advance(ADOPTION_WAIT),
        3 | 6 | 22 => world.advance(RETRY_WAIT),
        4 => {
            let parent = named(world, CHAT);
            world.peers.deletion.parent = parent;
            world.peers.deletion.tasks.insert(parent);
            let (task, attempt) = running(world, parent);
            let mut authority = self::task(world, parent).authority.clone();
            authority.budget.spend = 10;
            authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
            world.send(root::Event::Core(core::Event::DelegateValidated {
                to: ReplyTo::new(Token::new(9100)),
                key: core::CallKey { task, attempt, completion: CHILD_CALL, position: 0 },
                batch: Box::new([core::Delegate {
                    deadline: tasks::Deadline::Named,
                    label: 0,
                    flags: tasks::Flags::PLAIN,
                    tracked: None,
                    budget: people::Budget::Named,
                    executor: tasks::Executor::Agent { charter: 1 },
                    spec: tasks::Spec {
                        asks: false,
                        words: b"deletion delegate".as_slice().into(),
                        parameters: Box::new([]),
                        inputs: Box::new([]),
                    },
                    contract: tasks::Contract::Report { words: 128 },
                    authority,
                    llm: None,
                    symbolic_grants: Box::new([]),
                    dependencies: Box::new([]),
                    wake: tasks::WakePolicy::DEFAULT,
                }]),
                stubs: Box::new([]),
            }));
        }
        7 => {
            let child = named(world, b"deletion delegate");
            world.peers.deletion.child = child;
            world.peers.deletion.tasks.insert(child);
            running(world, child);
            message(world, world.peers.deletion.parent);
            message(world, child);
        }
        8 => ask(
            world,
            111,
            people::Ask::MarkSeen {
                seen: people::Seen::Conversation { task: world.peers.deletion.parent, through: 1 },
            },
        ),
        9 => {
            let task = world.peers.deletion.parent;
            ask(
                world,
                113,
                people::Ask::Amend {
                    project: PROJECT,
                    task,
                    revision: self::task(world, task).revisions.spec,
                    release: None,
                    amendment: people::Amendment {
                        tracked: Some(people::Tracking::Tracked { priority: 1 }),
                        spec: Some(people::Spec {
                            asks: false,
                            words: b"deletion amended".as_slice().into(),
                            parameters: Box::new([]),
                            inputs: Box::new([]),
                        }),
                        ..jig_core_world::fixtures::person_unchanged(b"history before deletion")
                    },
                },
            );
        }
        10 => propose(world, 114),
        11 => {
            let proposal = world
                .store
                .rows
                .values()
                .find_map(|row| match row {
                    root::Record::Core(core::Record::Tasks(tasks::Stored::PersonProposal(row)))
                        if row.project == PROJECT =>
                    {
                        Some(row.number)
                    }
                    root::Record::Core(_) | root::Record::Connector { .. } => None,
                })
                .expect("person proposal before deletion");
            ask(
                world,
                115,
                people::Ask::DecideProposal {
                    project: PROJECT,
                    proposer: people::Proposer::Party(PARTY),
                    proposal,
                    decision: people::ProposalDecision::Reject { reason: b"keep current".as_slice().into() },
                    running: None,
                },
            );
        }
        12 => {}
        13 => {
            if !super::ended(world, world.peers.deletion.child) {
                message(world, world.peers.deletion.child);
                return Ok(true);
            }
        }
        14 => act(
            world,
            118,
            people::Act::Close {
                project: PROJECT,
                task: world.peers.deletion.child,
                grace: None,
                reason: b"already ended".as_slice().into(),
            },
        ),
        15 => ask(world, 117, people::Ask::Rename { project: PROJECT, name: NAME.into() }),
        16 => {
            world.send(root::Event::Connector {
                number: CONNECTOR,
                event: connector::Event::Adopt {
                    project: PROJECT,
                    resource: jig_test_connector_world::path(1, 1),
                    role: connector::ResourceRole::Owned,
                },
            });
        }
        17 => create(world, 122, HELD),
        18 => {
            if !super::ended(world, world.peers.deletion.parent) {
                message(world, world.peers.deletion.parent);
                return Ok(true);
            }
        }
        19 => {
            check(super::ended(world, world.peers.deletion.parent), "populated conversation did not end")?;
            check(super::ended(world, world.peers.deletion.child), "populated delegate did not end")?;
        }
        20 => create(world, 119, LIVE),
        23 => {
            let number = named(world, LIVE);
            world.peers.deletion.live = number;
            world.peers.deletion.tasks.insert(number);
            let held = named(world, HELD);
            world.peers.deletion.held = held;
            world.peers.deletion.tasks.insert(held);
            running(world, held);
            message(world, held);
            running(world, number);
            world.send(root::Event::Connector {
                number: CONNECTOR,
                event: connector::Event::Names {
                    project: PROJECT,
                    task: number,
                    resources: Box::new([jig_test_connector_world::path(1, 1)]),
                },
            });
            message(world, number);
        }
        24 => ask(
            world,
            120,
            people::Ask::MarkSeen { seen: people::Seen::Conversation { task: world.peers.deletion.live, through: 1 } },
        ),
        25 => {
            let (task, attempt) = running(world, world.peers.deletion.live);
            world.send(root::Event::EffectCall {
                to: ReplyTo::new(Token::new(9101)),
                key: core::CallKey { task, attempt, completion: EFFECT_CALL, position: 0 },
                number: CONNECTOR,
                effect: jig_core_world::effects::World::effect(),
                deadline: Wall::from_nanos(world.clock().now + RETRY_WAIT),
                proposal: Some(b"connector proposal".as_slice().into()),
            });
        }
        26 => ask(world, 121, people::Ask::Stop { project: PROJECT, task: world.peers.deletion.held }),
        27 => propose(world, 116),
        _ => {
            inventory::filled(world)?;
            let control_tasks = inventory::tasks(world, CONTROL);
            world.peers.deletion.control = world
                .store
                .rows
                .iter()
                .filter(|(_, row)| inventory::family(row, CONTROL, &control_tasks).is_some())
                .map(|(key, row)| (key.clone(), row.clone()))
                .collect();
            return Ok(false);
        }
    }
    *phase += 1;
    Ok(true)
}

pub(crate) fn delivered(peers: &mut Neighbours, delivery: &root::Delivery) {
    if let root::Delivery::Core(core::Held::Acted { outcome, .. }) = delivery {
        if matches!(outcome, people::Outcome::ProjectRemoved { project: PROJECT }) {
            peers.deletion.removed = true;
        }
        if matches!(outcome, people::Outcome::ProjectMade { project: 3 }) {
            peers.deletion.reused = true;
        }
    }
}
fn check_deleted(world: &World) -> Result<(), Violation> {
    for row in world.store.rows.values() {
        if let Some(family) = inventory::family(row, PROJECT, &world.peers.deletion.tasks) {
            check(
                matches!(family, "Acted" | "Answer" | "ClosedLedger"),
                &format!("deleted project remains in {family}: {row:?}"),
            )?;
        }
    }
    let readback = world.store.fork().readback();
    for row in readback.values() {
        if let Some(family) = inventory::family(row, PROJECT, &world.peers.deletion.tasks) {
            check(
                matches!(family, "Acted" | "Answer" | "ClosedLedger"),
                &format!("store readback retains deleted {family}: {row:?}"),
            )?;
        }
    }
    for (key, row) in &world.peers.deletion.control {
        check(world.store.rows.get(key) == Some(row), &format!("control row changed: {key:?}"))?;
    }
    check(!world.referee.policy.projects.contains_key(&PROJECT), "referee retains deleted project")
}
pub(super) fn check_ending(world: &World) -> Result<(), Violation> {
    check_deleted(world)?;
    check(
        world.peers.deletion.removed && world.peers.deletion.reused,
        "deletion answer or name reuse was not observed",
    )?;
    let account = world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Accounts(accounts::Stored::Account { uses: Some(uses), .. })) if uses.is_empty()));
    check(account, "project-only account did not retain Some([])")
}

pub(crate) fn submitted(peers: &mut Neighbours, store: &jig_core_world::Store) {
    for pending in &store.pending {
        for write in &pending.writes {
            if let root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Deletion {
                project: PROJECT,
                stage,
            }))) = write
            {
                peers.deletion.stages.entry(*stage).or_insert(pending.number);
            }
        }
    }
}

pub(super) fn stage_cuts(world: &World) -> [u64; 3] {
    [people::Stage::Ending, people::Stage::Erasing, people::Stage::Releasing]
        .map(|stage| world.peers.deletion.stages[&stage])
}
