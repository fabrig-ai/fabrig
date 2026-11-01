//! Whole forge removal on the real root, core and store
//! (temper/docs/design/draft/playground-reset.md, 4 and 8).
use jig_domain_core as core;
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Key, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{cuts, direct::Driver, fixtures, forge_routes, forges, protocol};

const SEEDS: [u64; 2] = [71, 9762];
const FORGE: u16 = 1;
const REPOSITORY: skein_forge::api::Repository = skein_forge::api::Repository { forge: FORGE, repository: 2 };
const ENTRY: u64 = 100;
const REMOVE: Token = Token::new(220);

struct Story {
    driver: Driver,
    chat: u64,
    change: u64,
    producer: u64,
    seed: u64,
}

fn world(seed: u64) -> forge_routes::World {
    forge_routes::World::configured_limits(
        true,
        true,
        false,
        1000,
        forge_routes::Gate::default(),
        None,
        forge_routes::StoryLimits { cold_config: true, seed: Some(seed), ..forge_routes::StoryLimits::default() },
    )
}

fn config(seed: u64, granted: bool) -> engine::Config {
    let mut config = world(seed).cold_config.expect("independent cold configuration");
    config.under_box = true;
    // This forge is configured by its stored record, with no document identity to keep it connected.
    config.identities = Box::new([]);
    config.forge.writers = Box::new([]);
    config.core.grants = Box::new([people::ActGrant {
        connector: config.core.connectors[0],
        may: if granted { Box::new([people::ActKind::Cancel]) } else { Box::new([]) },
    }]);
    config
}

fn live(seed: u64, granted: bool) -> Story {
    let mut world = world(seed);
    world.store.seed(forges::rows(&[forges::record(FORGE, forge::Kind::Forgejo, forge::Mode::Owned)]), Box::new([]));
    let (mut world, chat, producer, _) = forge_routes::change_world_in(world, forge_routes::Change::default());
    // Finish the chat's run, keeping its delegated change live for removal.
    let read = chat.messages.last().map(|word| word.number);
    world.send(protocol::turn(Token::new(7), chat.task, chat.attempt, 1, 0, read, b"Change delegated"));
    world.until(forge_routes::Until::Ready);
    let mut event =
        world.protocol.answer(Token::new(7), chat.task, chat.attempt, 0, tasks::End::Parked, engine::Parts::none());
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: jig_domain_host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        unreachable!("worker answer")
    };
    answer.read = read.map(Token::new);
    answer.turns = 1;
    world.send(event);
    world.until(forge_routes::Until::Ready);
    // Drain the assignment's commit before taking the store into the cut driver.
    for _ in 0..20 {
        world.tick();
    }
    assert!(world.store.pending.is_empty() && world.pending.is_empty());
    let change = match live_task(&world.store, producer.task).requester {
        tasks::Party::Task(task) => task,
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => panic!("producer delegates from a change"),
    };
    assert_eq!(live_task(&world.store, change).requester, tasks::Party::Task(chat.task));
    let mut limits = world.env.limits;
    limits.people.act_grants = 1;
    world.store.seed(
        Box::new([Record::Forge {
            row: Box::new(forge::Stored::Writer(skein_forge_client::Writer { forge: FORGE, author: 1 })),
        }]),
        Box::new([]),
    );
    let mut driver = cuts::cold(world.store, config(seed, granted), &limits, seed, world.env.wall);
    driver.send(engine::Event::Forge(Box::new(forge::Event::Enqueue {
        entry: skein_forge_client::Entry {
            number: ENTRY,
            task: producer.task,
            repository: REPOSITORY,
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::Status {
                    commit: temper_engine_forge_world::translate::commit(1),
                    context: b"removal".as_slice().into(),
                    check: skein_forge::api::Check::Passed,
                    description: Box::new([]),
                    link: Box::new([]),
                },
                condition: skein_forge_client::Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    })));
    for _ in 0..200 {
        if driver.store.rows.contains_key(&Key::Forge(forge::Key::Flight { task: producer.task, entry: ENTRY })) {
            break;
        }
        driver.advance(true);
    }
    assert!(
        driver.store.rows.contains_key(&Key::Forge(forge::Key::Flight { task: producer.task, entry: ENTRY })),
        "live flight before removal"
    );
    assert_eq!(driver.root.forges().rows.len(), 1);
    Story { driver, chat: chat.task, change, producer: producer.task, seed }
}

fn live_task(store: &temper_engine_domain_world::store::Store, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) =
        store.rows.get(&Key::Core(core::Key::Tasks(tasks::Key::Live(task))))
    else {
        panic!("live task {task}")
    };
    row
}

fn session(d: &Driver) -> u64 {
    d.store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) = row {
                Some(*number)
            } else {
                None
            }
        })
        .expect("restored sign-in")
}

fn application(d: &mut Driver, request: forge::Ask, token: Token, key: u8) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        scope: if matches!(&request, forge::Ask::Adopt(_)) {
            people::ApplicationScope::Project(1)
        } else {
            people::ApplicationScope::Deployment
        },
        reply_to: ReplyTo::new(token),
        sign_in: session(d),
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: engine::Application::Forge(request),
    }));
}

fn remove(d: &mut Driver) {
    application(d, forge::Ask::RemoveForge { forge: FORGE }, REMOVE, 220);
    // Immediate outputs leave the journal one at a time, without a commit terminal.
    for _ in 0..200 {
        if d.immediate.iter().any(|request| matches!(request, engine::Request::Unregister { forge: FORGE })) {
            // Serve ends the unregistered forge's open calls. The stand has no connection layer.
            d.events.extend(d.forge.as_mut().expect("fake forge").replies());
            for event in &mut d.events {
                if let engine::Event::Forge(event) = event
                    && let forge::Event::Client(skein_forge_client::Event::Answered { result, .. }) = event.as_mut()
                {
                    *result = Err(skein_forge::api::Error::NotConnected);
                }
            }
            return;
        }
        engine::release(&mut d.root, &d.env, &mut d.out);
        d.collect();
    }
    panic!("Unregister must be immediate");
}

fn answer(d: &mut Driver, act: forge::KeptActKind) {
    let event = match act {
        forge::KeptActKind::RemoveGroup => forge::Event::GroupRemoved { forge: FORGE },
        forge::KeptActKind::EraseSecrets => forge::Event::SecretsErased { forge: FORGE },
        forge::KeptActKind::Cancel { .. } => panic!("the core supplies Cancel's answer"),
    };
    d.send(engine::Event::Forge(Box::new(event)));
}

fn finish(d: &mut Driver) {
    d.pump();
    for act in [forge::KeptActKind::RemoveGroup, forge::KeptActKind::EraseSecrets] {
        answer(d, act);
        d.pump();
    }
    assert!(cuts::kept_acts(&d.store).is_empty());
    let keys: Vec<_> = d
        .store
        .rows
        .keys()
        .filter_map(|key| match key {
            Key::Forge(key) => Some(key),
            Key::Core(_) | Key::Wires | Key::Transcript(_) => None,
        })
        .collect();
    assert!(
        !d.store.rows.contains_key(&Key::Forge(forge::Key::Forge(FORGE))),
        "forge rows after removal answers: {keys:?}"
    );
}

fn assert_removing(d: &Driver) {
    assert!(d.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::Application { to: reply_to, outcome: Some(engine::ApplicationAnswer::RemoveForge(forge::RemoveAnswer::Removing)), .. }
            if *reply_to == ReplyTo::new(REMOVE))), "RemoveForge must answer Removing: {:?}", d.delivered);
}

fn assert_cancelled(story: &Story) {
    let d = &story.driver;
    let answers = cuts::acted(&d.store);
    let [(key, people::Act::Cancel { project: 1, task, reason }, people::Outcome::Compound { operations: 2, .. })] =
        answers.as_slice()
    else {
        panic!("one core compound cancel: {answers:?}")
    };
    assert_eq!(*task, story.change);
    assert_eq!(reason.as_ref(), b"Forge removed");
    assert!(d.transactions.iter().flatten().any(|write| matches!(write,
        Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::KeptAct(row) if row.key == *key))));
    for task in [story.producer, story.change] {
        let phase = if let Some(row) = d.task(task) {
            &row.phase
        } else {
            let Some(Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) =
                d.store.rows.get(&Key::Core(core::Key::Tasks(tasks::Key::Ended(task))))
            else {
                panic!("cancelled task {task}")
            };
            &row.phase
        };
        let ending = match phase {
            tasks::Phase::Ended(ending) => ending,
            tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => &closing.ending,
            tasks::Phase::Closing(closing)
                if task == story.change
                    && closing.stage == tasks::Stage::Delegates
                    && matches!(
                        d.task(story.producer).map(|row| &row.phase),
                        Some(tasks::Phase::Held { was: tasks::Was::Closing(_), .. })
                    ) =>
            {
                &closing.ending
            }
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                panic!(
                    "cancelled task {task}: {phase:?}; producer phase: {:?}",
                    d.task(story.producer).map(|row| &row.phase)
                )
            }
        };
        assert!(
            matches!(ending,
            tasks::Ending::Cancelled { by: tasks::Party::Deployment { project: 1 }, reason, .. }
                if reason.as_ref() == b"Forge removed"),
            "task {task}: {phase:?}"
        );
    }
    let chat = d.task(story.chat).expect("chat stays live");
    assert!(matches!(chat.phase, tasks::Phase::Active(_)) && chat.grace.is_none());
}

#[test]
fn removing_a_forge_cancels_its_changes_and_ends_whole() {
    removing_a_forge_cancels_its_changes_and_ends_whole_at(SEEDS[0]);
}

#[test]
fn removing_a_forge_cancels_its_changes_and_ends_whole_at_second_seed() {
    removing_a_forge_cancels_its_changes_and_ends_whole_at(SEEDS[1]);
}

fn removing_a_forge_cancels_its_changes_and_ends_whole_at(seed: u64) {
    let mut story = live(seed, true);
    let d = &mut story.driver;
    assert!(cuts::forge_rows(&d.store).iter().any(|row| matches!(row, forge::Stored::Writer(_))));
    assert!(cuts::forge_rows(&d.store).iter().any(|row| matches!(row, forge::Stored::Flight { .. })));
    remove(d);
    let number = cuts::commit(d);
    let acts = cuts::kept_acts(&d.store);
    let mut saved = cuts::saved_acts(d);
    saved.sort_by_key(|row| row.key);
    assert_eq!(saved, acts, "all kept acts saved in this decision");
    assert_eq!(acts.len(), 3, "Cancel, RemoveGroup and EraseSecrets kept in the decision; seed {seed}");
    assert!(acts.iter().any(|row| row.act == forge::KeptActKind::Cancel { project: 1, task: story.change }));
    assert!(matches!(d.store.rows.get(&Key::Forge(forge::Key::Forge(FORGE))),
            Some(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Forge(row) if row.state == forge::ForgeState::Removing)));
    assert!(d.immediate.iter().any(|request| matches!(request, engine::Request::Unregister { forge: FORGE })));
    assert!(
        !cuts::forge_rows(&d.store).iter().any(|row| matches!(row, forge::Stored::Writer(_))),
        "writer erased in the removal decision"
    );
    d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
    d.pump();
    assert_removing(d);
    assert!(
        !d.store.rows.contains_key(&Key::Forge(forge::Key::Flight { task: story.producer, entry: ENTRY })),
        "the open call's terminal erases its flight before Cancel's answer is checked"
    );
    assert_eq!(cuts::kept_acts(&d.store).len(), 2, "the core answered Cancel");
    assert_cancelled(&story);
    let d = &mut story.driver;
    answer(d, forge::KeptActKind::RemoveGroup);
    d.pump();
    assert_eq!(cuts::kept_acts(&d.store).len(), 1);
    assert!(d.store.rows.contains_key(&Key::Forge(forge::Key::Forge(FORGE))));
    answer(d, forge::KeptActKind::EraseSecrets);
    d.pump();
    assert!(cuts::kept_acts(&d.store).is_empty());
    assert!(!d.store.rows.contains_key(&Key::Forge(forge::Key::Forge(FORGE))));
    for row in acts {
        assert!(d.transactions.iter().flatten().any(|write| matches!(write,
                Write::Erase(Key::Forge(forge::Key::KeptAct(key))) if *key == row.key)));
    }
}

fn restart(story: Story, granted: bool) -> Story {
    let limits = story.driver.env.limits;
    let wall = story.driver.env.wall;
    let driver = cuts::cold(story.driver.store, config(story.seed, granted), &limits, story.seed, wall);
    Story { driver, ..story }
}

fn comparable(d: &Driver) -> std::collections::BTreeMap<Key, Record> {
    let mut rows = d.store.rows.clone();
    let Some(Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))) =
        rows.get_mut(&Key::Core(core::Key::Core(core::CoreKey::Deployment)))
    else {
        panic!("deployment row")
    };
    // The journal commit counter counts the additional start mark and retained-answer replay.
    header.commits = 0;
    rows
}

fn commit_erasure(d: &mut Driver, key: [u8; 16]) -> u64 {
    for _ in 0..200 {
        let number = cuts::commit(d);
        if !d.store.rows.contains_key(&Key::Forge(forge::Key::KeptAct(key))) {
            return number;
        }
        d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
    }
    panic!("kept act answer did not commit");
}

#[test]
fn a_removal_cut_after_each_kept_act_resumes_to_the_same_end() {
    a_removal_cut_after_each_kept_act_resumes_to_the_same_end_at(SEEDS[0]);
}

#[test]
fn a_removal_cut_after_each_kept_act_resumes_to_the_same_end_at_second_seed() {
    a_removal_cut_after_each_kept_act_resumes_to_the_same_end_at(SEEDS[1]);
}

fn a_removal_cut_after_each_kept_act_resumes_to_the_same_end_at(seed: u64) {
    let mut uncut = live(seed, true);
    remove(&mut uncut.driver);
    finish(&mut uncut.driver);
    assert_removing(&uncut.driver);
    let expected = comparable(&uncut.driver);
    // Decision; Cancel's answer; RemoveGroup's answer; EraseSecrets' answer.
    for cut in 0..=3 {
        let mut story = live(seed, true);
        let d = &mut story.driver;
        remove(d);
        let number = cuts::commit(d);
        let original = cuts::kept_acts(&d.store);
        assert_eq!(original.len(), 3);
        if cut > 0 {
            d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
            // Commit the connector's erasure of Cancel, losing that decision's terminal.
            let cancel =
                original.iter().find(|row| matches!(row.act, forge::KeptActKind::Cancel { .. })).expect("kept Cancel");
            let number = commit_erasure(d, cancel.key);
            assert!(!cuts::kept_acts(&d.store).iter().any(|row| matches!(row.act, forge::KeptActKind::Cancel { .. })));
            if cut > 1 {
                d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
                d.pump();
                answer(d, forge::KeptActKind::RemoveGroup);
                let number = cuts::commit(d);
                if cut > 2 {
                    d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
                    d.pump();
                    answer(d, forge::KeptActKind::EraseSecrets);
                    cuts::commit(d);
                }
            }
        }
        let unanswered = cuts::kept_acts(&story.driver.store);
        let answers = cuts::acted(&story.driver.store);
        assert_eq!(answers.len(), 1, "the core committed its Cancel answer before the cut");
        assert!(
            original.iter().any(|row| row.key == answers[0].0
                && row.act == forge::KeptActKind::Cancel { project: 1, task: story.change })
        );
        story = restart(story, true);
        assert_eq!(cuts::acted(&story.driver.store), answers, "retained Cancel answer; seed {seed}, cut {cut}");
        for row in &unanswered {
            assert!(original.contains(row), "resend retains the key");
            if !matches!(row.act, forge::KeptActKind::Cancel { .. }) {
                assert!(cuts::kept_acts(&story.driver.store).contains(row), "unanswered serve act retains its key");
            }
            match row.act {
                forge::KeptActKind::Cancel { .. } => assert!(!cuts::kept_acts(&story.driver.store).contains(row)),
                forge::KeptActKind::RemoveGroup => assert!(
                    story
                        .driver
                        .immediate
                        .iter()
                        .any(|request| matches!(request, engine::Request::RemoveGroup { forge: FORGE }))
                ),
                forge::KeptActKind::EraseSecrets => assert!(
                    story
                        .driver
                        .immediate
                        .iter()
                        .any(|request| matches!(request, engine::Request::EraseSecrets { forge: FORGE }))
                ),
            }
        }
        finish(&mut story.driver);
        let resumed = comparable(&story.driver);
        assert_eq!(resumed.keys().collect::<Vec<_>>(), expected.keys().collect::<Vec<_>>(), "seed {seed}, cut {cut}");
        for (key, row) in resumed {
            assert_eq!(Some(&row), expected.get(&key), "seed {seed}, cut {cut}, row {key:?}");
        }
    }
}

fn assert_absent(story: &mut Story, key: u8) {
    let d = &mut story.driver;
    assert!(d.root.forge_ended(FORGE));
    assert!(d.root.forges().rows.is_empty());
    assert!(d.root.adopted_ids(FORGE).is_empty());
    let rows = cuts::forge_rows(&d.store);
    let repositories: Vec<_> =
        rows.iter().filter_map(|row| if let forge::Stored::Repository(row) = row { Some(row) } else { None }).collect();
    assert_eq!(repositories.len(), 1, "task-named repository kept");
    assert_eq!(repositories[0].provider, REPOSITORY);
    let tag = forge::tag(FORGE, 2);
    let historical = rows
        .iter()
        .find_map(|row| {
            if let forge::Stored::Repository(row) = row
                && forge::tag(row.provider.forge, row.provider.repository) == tag
            {
                Some(row)
            } else {
                None
            }
        })
        .expect("repository readable by its retained tag");
    assert_eq!(historical.owner.as_ref(), b"org");
    assert_eq!(historical.name.as_ref(), b"repo");
    assert!(
        !rows.iter().any(|row| matches!(
            row,
            forge::Stored::Forge(_)
                | forge::Stored::Writer(_)
                | forge::Stored::Flight { .. }
                | forge::Stored::KeptAct(_)
                | forge::Stored::Subscription(_)
                | forge::Stored::Entry { .. }
                | forge::Stored::Client(_)
        )),
        "remaining forge rows: {rows:?}"
    );
    assert_repository_rows(&rows);
    d.delivered.clear();
    d.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: Token::new(u64::from(key)),
        sign_in: session(d),
        subject: Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(story.change),
        },
        window: Window { rows: 1, after: None },
    }));
    d.pump();
    assert!(
        !d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Panel { .. })),
        "removed forge's change panel is still delivered: {:?}",
        d.delivered
    );
    assert!(d.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WatchEnded { watcher, why: jig_domain_views::End::Gone } if *watcher == Token::new(u64::from(key)))));
    d.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key) + 1)),
        sign_in: session(d),
        key: [key + 1; 16],
        models: Box::new([]),
        ask: people::Ask::Say { project: 1, task: story.chat, words: b"What remains?".as_slice().into() },
    }));
    d.pump();
    assert!(
        d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Assigned { .. })),
        "chat resumes at {key}: {:?}; proof {:?}; deliveries {:?}",
        d.task(story.chat).map(|row| &row.phase),
        d.store.rows.get(&Key::Core(core::Key::Core(core::CoreKey::RunProof(story.chat)))),
        d.delivered
    );
    let chat = d.last_assignment();
    assert_eq!(chat.task, story.chat, "the chat stays open for its next run");
    assert!(chat.workspace.repositories.is_empty(), "the project's repository list is empty");
    d.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        u64::from(key),
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::ReadNamed {
                repository: None,
                read: skein_forge::api::Read::Branch { branch: b"main".as_slice().into() },
            },
        },
    );
    d.pump();
    assert!(
        matches!(d.answered(u64::from(key)), Some(engine::CallAnswer::ReadRepositoryRefused { repositories, .. }) if repositories.is_empty()),
        "no project repository is offered to the chat"
    );
    application(d, forge::Ask::Adopt(fixtures::adoption(1)), Token::new(221), key + 2);
    d.pump();
    assert!(d.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::Application { to: reply_to, outcome: Some(engine::ApplicationAnswer::Adopt(forge::ApplicationOutcome::ForgeRefused(_))), .. }
            if *reply_to == ReplyTo::new(Token::new(221)))));
}

fn assert_repository(name: &forge::Name) {
    assert_eq!((name.forge, name.repository), (REPOSITORY.forge, REPOSITORY.repository));
}

fn assert_repository_rows(rows: &[forge::Stored]) {
    for row in rows {
        match row {
            forge::Stored::Hold(row) => assert_repository(&row.name),
            forge::Stored::Names { resources, .. } => {
                for name in resources {
                    assert_repository(name);
                }
            }
            forge::Stored::Change(row) => assert_eq!(row.repository, REPOSITORY),
            forge::Stored::Application { outcome: forge::ApplicationOutcome::Adopted(row), .. } => {
                assert_eq!(row.repository, REPOSITORY);
            }
            forge::Stored::BranchHead(row) => assert_repository(&row.name),
            forge::Stored::PullState(row) => assert_repository(&row.name),
            forge::Stored::Ci(row) => assert_eq!(row.repository, REPOSITORY),
            forge::Stored::Call { .. }
            | forge::Stored::ProposedEffect(_)
            | forge::Stored::Repository(_)
            | forge::Stored::Subscription(_)
            | forge::Stored::Landed { .. }
            | forge::Stored::Entry { .. }
            | forge::Stored::Client(_)
            | forge::Stored::Issue(_)
            | forge::Stored::Release(_)
            | forge::Stored::Application { .. }
            | forge::Stored::Writer(_)
            | forge::Stored::Demo(_)
            | forge::Stored::Binding { .. }
            | forge::Stored::DemoAct(_)
            | forge::Stored::Flight { .. }
            | forge::Stored::Landing(_)
            | forge::Stored::Receipt { .. }
            | forge::Stored::Forge(_)
            | forge::Stored::ForgeNumbers { .. }
            | forge::Stored::Playground(_)
            | forge::Stored::KeptAct(_) => {}
        }
    }
}

#[test]
fn nothing_of_a_removed_forge_is_read_but_its_repositories() {
    nothing_of_a_removed_forge_is_read_but_its_repositories_at(SEEDS[0]);
}

#[test]
fn nothing_of_a_removed_forge_is_read_but_its_repositories_at_second_seed() {
    nothing_of_a_removed_forge_is_read_but_its_repositories_at(SEEDS[1]);
}

fn nothing_of_a_removed_forge_is_read_but_its_repositories_at(seed: u64) {
    let mut story = live(seed, true);
    remove(&mut story.driver);
    finish(&mut story.driver);
    assert_absent(&mut story, 222);
    story = restart(story, true);
    assert_absent(&mut story, 232);
}

#[test]
fn a_removal_without_the_cancel_grant_keeps_its_acts_until_a_granted_restart() {
    a_removal_without_the_cancel_grant_keeps_its_acts_until_a_granted_restart_at(SEEDS[0]);
}

#[test]
fn a_removal_without_the_cancel_grant_keeps_its_acts_until_a_granted_restart_at_second_seed() {
    a_removal_without_the_cancel_grant_keeps_its_acts_until_a_granted_restart_at(SEEDS[1]);
}

fn a_removal_without_the_cancel_grant_keeps_its_acts_until_a_granted_restart_at(seed: u64) {
    let mut story = live(seed, false);
    remove(&mut story.driver);
    story.driver.pump();
    assert_removing(&story.driver);
    let kept = cuts::kept_acts(&story.driver.store);
    assert_eq!(kept.len(), 3);
    assert!(cuts::acted(&story.driver.store).is_empty());
    assert!(story.driver.delivered.iter().any(|delivery| matches!(delivery,
            Delivery::DemoNote { line } if line.starts_with(b"forge removal:")
                && line.windows(b"engine.acts".len()).any(|part| part == b"engine.acts")
                && line.windows(b"cancel".len()).any(|part| part == b"cancel"))));
    story = restart(story, false);
    assert_eq!(cuts::kept_acts(&story.driver.store), kept);
    assert!(cuts::acted(&story.driver.store).is_empty());
    story = restart(story, true);
    assert_eq!(cuts::kept_acts(&story.driver.store).len(), 2);
    finish(&mut story.driver);
    assert!(cuts::kept_acts(&story.driver.store).is_empty());
    let answers = cuts::acted(&story.driver.store);
    assert!(
        matches!(answers.as_slice(), [(_, people::Act::Cancel { task, reason, .. }, people::Outcome::Compound { operations: 2, .. })]
            if *task == story.change && reason.as_ref() == b"Forge removed")
    );
}
