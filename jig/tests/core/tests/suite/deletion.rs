//! Project deletion on the real store (`temper/docs/design/draft/playground-reset.md`, 5 and 8).

use jig_core_world::{
    effects::{Cut, World},
    fixtures::{self, live_row},
    reads,
    stories::{delegates::delegate_pair, reads::request},
};
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core::{self as core, documents};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{Duration, List, Map, ReplyTo, Token};
use std::collections::BTreeMap;

const SEED: u64 = 1900;
const PROJECT: u32 = 1;
const UNKNOWN: u32 = 99;
const PARTY: u64 = 1;
const REMOVE: u8 = 91;
const SHARED: u32 = 0;
const SOLE: u32 = 2;

fn configuration(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_core_world::faults::plan_fixture(seed);
    limits.core.authority.projects = 3;
    limits.core.people.projects = 4;
    limits.core.people.initial_owners = 2;
    limits.core.accounts.accounts = 2;
    limits.core.accounts.holder_accounts = 2;
    limits.core.accounts.sources = 2;
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        people::InitialOwner { project: 0, identity: config.core.owners[0].identity.clone() },
    ]);
    config.core.grants = Box::new([people::ActGrant {
        connector: 1,
        may: Box::new([people::ActKind::RemoveProject, people::ActKind::Close, people::ActKind::Adopt]),
    }]);
    config.core.defaults.rules = authority::Domain::new(config.core.defaults.rules.clone(), limits.core.authority)
        .expect("three project identities")
        .rules()
        .clone();
    config.core.defaults.policies = Map::with_capacity(1);
    let mut policy = config.core.seed_projects[0].policy.clone();
    policy.permissions = Box::new([]);
    config.core.defaults.policies.insert(1, policy).expect("one policy");
    jig_core_world::peers::journal(&mut limits);
    (config, limits)
}

fn account_rows(config: &root::Config) -> Box<[root::Record]> {
    let mut rows = fixtures::account_rows(&config.core.settings.llm_providers).into_vec();
    for row in &mut rows {
        if let root::Record::Core(core::Record::Accounts(accounts::Stored::Account { uses, .. })) = row {
            *uses = Some(Box::new([accounts::Use::Project(PROJECT), accounts::Use::Party(PARTY)]));
        }
        if let root::Record::Core(core::Record::Accounts(accounts::Stored::Next { number })) = row {
            *number = 4;
        }
    }
    rows.extend(
        [
            accounts::Stored::Account {
                number: SOLE,
                provider: config.core.settings.llm_providers.providers[0].number,
                holder: accounts::Holder::Deployment,
                place: 1,
                uses: Some(Box::new([accounts::Use::Project(PROJECT)])),
                subject: Box::new([]),
                display: Box::new([]),
            },
            accounts::Stored::Source {
                account: SOLE,
                number: 3,
                place: 0,
                kind: accounts::Kind::Key,
                name: Box::new([]),
                held: None,
            },
        ]
        .into_iter()
        .map(|row| root::Record::Core(core::Record::Accounts(row))),
    );
    rows.into_boxed_slice()
}

fn deleted_world(seed: u64) -> World {
    let (config, limits) = configuration(seed);
    let initial = fixtures::create_chat(
        &fixtures::Creating::new(&config.core, limits.core.tasks.tree_tasks),
        PROJECT,
        0,
        b"retained conversation".as_slice().into(),
    );
    let rows = account_rows(&config);
    let mut world = World::configured_with_rows(seed, false, config, limits, initial, rows);
    assert!(
        world.assigned.is_some(),
        "fixture conversation assigned: {:?}; restore {:?}; restart {:?}; steps {:?}",
        world.people_answers,
        world.domain.restore_failure(),
        world.domain.restart_failure(),
        world.restart_steps
    );
    let ended = world.task();
    world.turn(1, 7, None, b"retained reply");
    request(&mut world, 60, people::Ask::MarkSeen { seen: people::Seen::Conversation { task: ended, through: 1 } });
    fixtures::finish_chat(&mut world, 7, b"retained result");
    let next = fixtures::chat(&world, PROJECT, b"live conversation".as_slice().into());
    world.assignments.clear();
    request(&mut world, 61, next);
    let chat = world.task();
    world.turn(1, 3, None, b"read reply");
    request(&mut world, 62, people::Ask::MarkSeen { seen: people::Seen::Conversation { task: chat, through: 1 } });
    world.turn(2, 5, None, b"unread reply");
    let (held, other) = delegate_pair(&mut world);
    request(&mut world, 63, people::Ask::Stop { project: PROJECT, task: held });
    assert!(live_row(&world, held).placed.is_some(), "fixture has a placed hold");
    let attempt = live_row(&world, held).attempt;
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: Token::new(7),
        task: held,
        attempt,
        cumulative: 0,
        end: tasks::End::Parked,
    });
    let mut rows = Vec::new();
    for (name, scope) in
        [(1, notes::Scope::Project { project: PROJECT }), (2, notes::Scope::Goal { project: PROJECT, goal: other })]
    {
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(notes::Entry {
            name,
            scope: scope.clone(),
            description: b"project knowledge".as_slice().into(),
            body: b"kept note body".as_slice().into(),
            references: List::with_capacity(0),
            author: notes::Author::Party { party: PARTY },
            revision: 1,
            changed: world.wall_time(),
        }))));
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            name,
            scope,
            description: b"project knowledge".as_slice().into(),
            revision: 1,
        }))));
    }
    world.seed(rows.into_boxed_slice());
    assert!(world.store.rows.values().any(|row| matches!(row,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) if row.number == ended)));
    world
}

fn act(world: &mut World, key: u8, act: people::Act) {
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        connector: 1,
        key: [key; 16],
        act: Box::new(act),
        carried: None,
    }));
}

fn remove(world: &mut World, key: u8) {
    act(world, key, people::Act::RemoveProject { project: PROJECT });
}

fn deletion() -> root::Key {
    root::Key::Core(core::Key::People(people::Key::Deletion(PROJECT)))
}

fn removed(world: &World) {
    assert_eq!(
        world.act_answers.last().expect("removal answered").1,
        people::Outcome::ProjectRemoved { project: PROJECT }
    );
}

fn finish(world: &mut World) {
    // A host which does not answer cancellation is retired at its advertised stop bound.
    world.advance(Duration::from_secs(6));
    assert!(!world.store.rows.contains_key(&deletion()), "deletion reached its last step");
    assert!(!world.stopped, "the store stayed writable");
}

fn scope_names(scope: &notes::Scope, project: u32) -> bool {
    match scope {
        notes::Scope::Deployment => false,
        notes::Scope::Project { project: owner }
        | notes::Scope::Goal { project: owner, .. }
        | notes::Scope::Resources { project: owner, .. } => *owner == project,
    }
}

fn ask_names(ask: &people::Ask, project: u32) -> bool {
    match ask {
        people::Ask::RemoveProject { project: owner }
        | people::Ask::Compound { project: owner, .. }
        | people::Ask::Create { project: owner, .. }
        | people::Ask::EditNote { project: owner, .. }
        | people::Ask::MakeService { project: owner, .. }
        | people::Ask::Adopt { project: owner, .. }
        | people::Ask::Propose { project: owner, .. }
        | people::Ask::Withdraw { project: owner, .. }
        | people::Ask::Stop { project: owner, .. }
        | people::Ask::Cancel { project: owner, .. }
        | people::Ask::Release { project: owner, .. }
        | people::Ask::TakePerson { project: owner, .. }
        | people::Ask::HandBackPerson { project: owner, .. }
        | people::Ask::AnswerPerson { project: owner, .. }
        | people::Ask::Move { project: owner, .. }
        | people::Ask::DecideProposal { project: owner, .. }
        | people::Ask::Say { project: owner, .. }
        | people::Ask::AnswerQuestion { project: owner, .. }
        | people::Ask::Amend { project: owner, .. }
        | people::Ask::SetRoles { project: owner, .. }
        | people::Ask::ChangePolicy { project: owner, .. }
        | people::Ask::Rename { project: owner, .. }
        | people::Ask::SetPool { project: owner, .. }
        | people::Ask::DecideEscalation { project: owner, .. }
        | people::Ask::Application { scope: people::ApplicationScope::Project(owner), .. } => *owner == project,
        people::Ask::Application {
            scope: people::ApplicationScope::Own | people::ApplicationScope::Deployment,
            ..
        }
        | people::Ask::ChangeSettings { .. }
        | people::Ask::MoveToNewest { .. }
        | people::Ask::MakeProject { .. }
        | people::Ask::MarkSeen { .. }
        | people::Ask::Lift { .. }
        | people::Ask::AccountSignIn { .. }
        | people::Ask::AccountSignOut { .. }
        | people::Ask::AccountChange { .. } => false,
    }
}

fn people_names(row: &people::Stored, project: u32) -> bool {
    match row {
        people::Stored::Roles { project: owner, .. }
        | people::Stored::Policy { project: owner, .. }
        | people::Stored::Deletion { project: owner, .. } => *owner == project,
        people::Stored::Acted { act, .. } => match act {
            people::Act::Close { project: owner, .. }
            | people::Act::Adopt { project: owner, .. }
            | people::Act::RemoveProject { project: owner } => *owner == project,
            people::Act::MakeProject { .. } | people::Act::AddKey { .. } | people::Act::RemoveAccount { .. } => false,
        },
        people::Stored::Answer { ask, .. } => ask_names(ask, project),
        // Every task and conversation in this fixture belongs to the removed project.
        people::Stored::ReadConversation { .. } | people::Stored::EndedRead { .. } => true,
        people::Stored::Person { .. } | people::Stored::ReadResults { .. } | people::Stored::SignIn { .. } => false,
    }
}

fn rows_naming(world: &World, project: u32) -> Vec<String> {
    world
        .store
        .rows
        .iter()
        .filter_map(|(key, row)| {
            let names = match row {
                root::Record::Core(core::Record::People(row)) => people_names(row, project),
                root::Record::Core(core::Record::Notes(row)) => match row {
                    notes::Record::Entry(row) => scope_names(&row.scope, project),
                    notes::Record::Line(row) => scope_names(&row.scope, project),
                },
                root::Record::Core(core::Record::Accounts(accounts::Stored::Account { number, uses, .. })) => {
                    *number != SOLE && uses.as_ref().is_some_and(|uses| uses.contains(&accounts::Use::Project(project)))
                }
                root::Record::Core(
                    core::Record::Accounts(
                        accounts::Stored::Next { .. }
                        | accounts::Stored::Issued { .. }
                        | accounts::Stored::Source { .. },
                    )
                    | core::Record::Tasks(tasks::Stored::ClosedLedger(_))
                    | core::Record::Core(core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_)),
                ) => false,
                // This fixture creates no task, funding source or run outside the removed project.
                root::Record::Core(core::Record::Tasks(_) | core::Record::Core(_)) => true,
                root::Record::Connector { record, .. } => match record {
                    connector::Record::Task { project: owner, .. }
                    | connector::Record::Adoption { project: owner, .. }
                    | connector::Record::Proposal { project: owner, .. } => *owner == project,
                    connector::Record::Procedure(_)
                    | connector::Record::ProcedureDelegateOutcome { .. }
                    | connector::Record::Result { .. }
                    | connector::Record::Subscription { .. }
                    | connector::Record::Outbox(_) => true,
                    connector::Record::ApplicationObject { .. }
                    | connector::Record::ApplicationAnswer { .. }
                    | connector::Record::Pool { .. }
                    | connector::Record::Made { .. } => false,
                },
            };
            names.then(|| format!("{key:?}: {row:?}"))
        })
        .collect()
}

fn final_rows(world: &World) -> BTreeMap<root::Key, root::Record> {
    let mut rows = world.store.rows.clone();
    for row in rows.values_mut() {
        if let root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = row {
            // A restart has its own durable decisions; all allocated identities still agree.
            header.commits = 0;
        }
    }
    rows
}

#[test]
fn a_deleted_project_leaves_only_its_spend() {
    let mut world = deleted_world(SEED);
    let number = world.store.applied + 1;
    world.cut = Some(Cut::Durable(number));
    remove(&mut world, REMOVE);
    assert_eq!(world.reached_cut, Some(Cut::Durable(number)));
    assert!(world.act_answers.is_empty(), "answer waits for the closing decision");
    assert!(world.commits.last().expect("closing decision").iter().any(|write| matches!(
        write,
        root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Acted {
            outcome: people::Outcome::ProjectRemoved { project: PROJECT },
            ..
        })))
    )));
    world.drain();
    removed(&world);
    finish(&mut world);
    assert_eq!(rows_naming(&world, PROJECT), Vec::<String>::new());
    for (number, expected) in
        [(SHARED, vec![accounts::Use::Party(PARTY)]), (SOLE, vec![accounts::Use::Project(PROJECT)])]
    {
        let row = &world.store.rows[&root::Key::Core(core::Key::Accounts(accounts::Key::Account { number }))];
        let root::Record::Core(core::Record::Accounts(accounts::Stored::Account { uses, .. })) = row else {
            panic!("account retained")
        };
        assert_eq!(uses.as_deref(), Some(expected.as_slice()));
    }
}

#[test]
fn a_deletion_cut_after_any_commit_ends_the_same() {
    let mut uncut = deleted_world(SEED);
    let first = uncut.store.applied + 1;
    remove(&mut uncut, REMOVE);
    finish(&mut uncut);
    let last = uncut.store.applied;
    let expected = final_rows(&uncut);
    assert!(last > first, "the sweep crosses deletion steps");
    for number in first..=last {
        for cut in [Cut::Durable(number), Cut::Submitted(number)] {
            let mut world = deleted_world(SEED);
            world.cut = Some(cut);
            remove(&mut world, REMOVE);
            if world.reached_cut != Some(cut) {
                world.advance(Duration::from_secs(6));
            }
            assert_eq!(world.reached_cut, Some(cut), "every deletion commit is cut");
            world.restart_with(configuration(SEED).0);
            // Only a submission lost before the closing decision needs the requester again.
            if cut == Cut::Submitted(first) {
                remove(&mut world, REMOVE);
            }
            finish(&mut world);
            assert!(world.domain.ready(), "restart finishes deletion before opening");
            assert_eq!(rows_naming(&world, PROJECT), Vec::<String>::new(), "{cut:?}");
            assert_eq!(final_rows(&world), expected, "{cut:?}");
        }
    }
}

fn party_answer(world: &mut World, key: u8, ask: people::Ask) -> people::Reply {
    let before = world.people_answers.len();
    request(world, key, ask);
    assert_eq!(world.people_answers.len(), before + 1, "one party terminal");
    world.people_answers.last().expect("party answer").clone()
}

fn adoption() -> people::Adoption {
    people::Adoption {
        resource: people::ResourceName { connector: 1, path: Box::new([b"own".as_slice().into()]) },
        role: people::ResourceRole::Owned,
        options: Box::new([]),
    }
}

fn party_entries(world: &World, project: u32, chat: u64) -> [people::Ask; 5] {
    [
        people::Ask::Say { project, task: chat, words: b"too late".as_slice().into() },
        fixtures::chat(world, project, b"another chat".as_slice().into()),
        people::Ask::Rename { project, name: "Another name".into() },
        people::Ask::SetRoles {
            project,
            holdings: Box::new([people::Holding { person: PARTY, role: people::Role::Owner }]),
        },
        people::Ask::Application { scope: people::ApplicationScope::Project(project), connector: 1, digest: [7; 32] },
    ]
}

fn watch_end(world: &mut World, project: u32, watcher: u64) -> jig_domain_views::End {
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(watcher),
        sign_in: world.sign_in.expect("signed in"),
        subject: documents::Watched::Query {
            query: documents::TaskQuery { project: Some(project), requester: None, ..fixtures::OWN_TASKS },
        },
        window: documents::Window { rows: 2, after: None },
    }));
    let core::Held::WatchEnded { watcher: ended, why } = world.stop_deliveries.last().expect("watch ended") else {
        panic!("unknown project's watch ends")
    };
    assert_eq!(*ended, Token::new(watcher));
    *why
}

#[test]
fn a_closed_project_refuses_every_entry_as_one_never_made() {
    let mut world = deleted_world(SEED);
    let chat = world.assignments[0].0;
    remove(&mut world, REMOVE);
    removed(&world);
    assert!(world.store.rows.contains_key(&deletion()), "hosts have not finished");
    let closed = party_entries(&world, PROJECT, chat);
    let unknown = party_entries(&world, UNKNOWN, chat);
    for (index, (closed, unknown)) in closed.into_iter().zip(unknown).enumerate() {
        let key = 100 + u8::try_from(index * 2).expect("five entrances");
        let expected = party_answer(&mut world, key, unknown);
        assert_eq!(party_answer(&mut world, key + 1, closed), expected, "party entrance {index}");
        assert_eq!(expected, people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)));
    }
    for (index, (closed, unknown)) in [
        (
            people::Act::Close { project: PROJECT, task: chat, grace: None, reason: Box::new([]) },
            people::Act::Close { project: UNKNOWN, task: chat, grace: None, reason: Box::new([]) },
        ),
        (
            people::Act::Adopt { project: PROJECT, adoption: adoption() },
            people::Act::Adopt { project: UNKNOWN, adoption: adoption() },
        ),
        (people::Act::RemoveProject { project: PROJECT }, people::Act::RemoveProject { project: UNKNOWN }),
    ]
    .into_iter()
    .enumerate()
    {
        let key = 120 + u8::try_from(index * 2).expect("three acts");
        act(&mut world, key, unknown);
        let expected = world.act_answers.last().expect("unknown act answer").1.clone();
        act(&mut world, key + 1, closed);
        assert_eq!(world.act_answers.last().expect("closed act answer").1, expected);
        assert_eq!(expected, people::Outcome::Refused(people::Refusal::Unknown));
    }
    let expected = watch_end(&mut world, UNKNOWN, 800);
    assert_eq!(watch_end(&mut world, PROJECT, 801), expected);
    world.reads = Some(reads::Parties::new(world.store.applied));
    reads::ask(&mut world, 802, core::reads::Read::Frame);
    let core::reads::Answer::Document(document) = reads::latest(&world) else { panic!("party frame") };
    let documents::Document::Frame(frame) = document.as_ref() else { panic!("frame") };
    assert!(frame.projects.iter().all(|row| row.project != PROJECT));
}

#[test]
fn a_resent_removal_answers_removed_then_unknown() {
    let mut world = deleted_world(SEED);
    remove(&mut world, REMOVE);
    removed(&world);
    assert!(world.store.rows.contains_key(&deletion()));
    let commits = world.commits.len();
    remove(&mut world, REMOVE);
    removed(&world);
    assert_eq!(world.commits.len(), commits, "same kept answer makes no decision");
    finish(&mut world);
    let rows = world.store.rows.clone();
    remove(&mut world, REMOVE);
    assert_eq!(world.act_answers.last().expect("resend answer").1, people::Outcome::Refused(people::Refusal::Unknown));
    assert_eq!(world.store.rows, rows, "unknown does not recreate the retained answer");
}

#[test]
fn a_deleted_projects_name_is_free_and_its_number_never_comes_back() {
    let mut world = deleted_world(SEED);
    remove(&mut world, REMOVE);
    removed(&world);
    assert!(world.store.rows.contains_key(&deletion()));
    let answer = party_answer(&mut world, 130, people::Ask::MakeProject { name: "Project 1".into(), policy: 1 });
    let people::Reply::Outcome(people::Outcome::ProjectMade { project: made }) = answer else {
        panic!("name free in the closing decision: {answer:?}")
    };
    assert!(made > PROJECT);
    world.restart_with(configuration(SEED).0);
    finish(&mut world);
    assert!(world.domain.ready(), "restart completes with the replacement project");
    let answer = party_answer(&mut world, 131, people::Ask::MakeProject { name: "Later project".into(), policy: 1 });
    let people::Reply::Outcome(people::Outcome::ProjectMade { project: later }) = answer else {
        panic!("later project made: {answer:?}")
    };
    assert!(later > made);
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Policy(PROJECT)))));
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Policy(made)))));
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Policy(later)))));
}

fn spend(world: &mut World) -> Vec<tasks::FundingRecord> {
    if world.reads.is_none() {
        world.reads = Some(reads::Parties::new(world.store.applied));
    }
    let mut from = None;
    let mut rows = Vec::new();
    for page in 0..32 {
        reads::ask(world, 900 + page, core::reads::Read::Spend { project: 0, from: from.take() });
        let core::reads::Answer::Page(answer) = reads::latest(world) else { panic!("deployment period spend") };
        for row in &answer.rows {
            let core::reads::PageRow::Spend(row) = row else { panic!("spend row") };
            rows.push(*row);
        }
        from.clone_from(&answer.next);
        if from.is_none() {
            return rows;
        }
    }
    panic!("bounded spend pages")
}

#[test]
fn a_deleted_projects_spend_stays_in_the_totals() {
    let mut world = deleted_world(SEED);
    let before = spend(&mut world);
    assert!(before.iter().any(|row| row.numbers.spent + row.numbers.spent_below > 0), "nonzero deployment spend");
    remove(&mut world, REMOVE);
    finish(&mut world);
    assert_eq!(spend(&mut world), before);
    world.restart_with(configuration(SEED).0);
    assert_eq!(spend(&mut world), before, "period spend survives a cold start");
}

#[test]
fn a_deleted_projects_tasks_leave_every_inbox() {
    let mut world = deleted_world(SEED);
    let signed = world.sign_in.expect("requester signed in");
    world.reads = Some(reads::Parties::new(world.store.applied));
    assert!(!fixtures::inbox_objects(&mut world, signed, core::reads::Group::Updates).is_empty());
    remove(&mut world, REMOVE);
    finish(&mut world);
    for restart in [false, true] {
        if restart {
            world.restart_with(configuration(SEED).0);
        }
        for (number, group) in [
            core::reads::Group::Needs,
            core::reads::Group::Updates,
            core::reads::Group::Proposals,
            core::reads::Group::Escalations,
            core::reads::Group::Questions,
            core::reads::Group::PersonTasks,
            core::reads::Group::Results,
            core::reads::Group::Replies,
        ]
        .into_iter()
        .enumerate()
        {
            reads::ask(
                &mut world,
                1000 + u64::try_from(number).expect("eight groups"),
                core::reads::Read::Inbox {
                    group,
                    filter: core::reads::Filter { project: None, goal: None },
                    from: None,
                },
            );
            let core::reads::Answer::Page(answer) = reads::latest(&world) else { panic!("requester inbox") };
            assert!(answer.rows.is_empty(), "{group:?} after restart={restart}");
            assert!(answer.next.is_none());
        }
        assert!(!world.store.rows.keys().any(|key| matches!(
            key,
            root::Key::Core(core::Key::People(people::Key::ReadConversation { .. } | people::Key::EndedRead { .. }))
        )));
    }
}

#[test]
fn each_connector_is_asked_until_it_releases() {
    let mut world = deleted_world(SEED);
    world.hold_project_releases.push(2);
    remove(&mut world, REMOVE);
    world.advance(Duration::from_secs(6));
    let mut releases = world.project_releases.clone();
    releases.sort_unstable();
    assert_eq!(releases, [(1, PROJECT), (2, PROJECT)]);
    assert!(world.store.rows.contains_key(&deletion()), "one connector still owes its answer");
    for _ in 0..3 {
        world.drain();
        world.advance(Duration::from_secs(1));
    }
    assert_eq!(world.project_releases.len(), 2, "once per connector in this start");
    world.restart_with(configuration(SEED).0);
    let mut releases = world.project_releases[2..].to_vec();
    releases.sort_unstable();
    assert_eq!(releases, [(1, PROJECT), (2, PROJECT)], "a new start asks every connector again");
    assert!(world.store.rows.contains_key(&deletion()));
    world.send(root::Event::Core(core::Event::ProjectReleased { connector: 2, project: PROJECT }));
    finish(&mut world);
    assert!(world.domain.ready());
    assert_eq!(rows_naming(&world, PROJECT), Vec::<String>::new());
}
