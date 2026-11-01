//! Removal across durable decisions (temper/docs/design/draft/playground-reset.md, 4 and 8;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, D3 and D4).
use super::*;
use temper_engine_domain_world::store::Store;

// The chat template grants five seconds; this horizon also drains task endings.
const END_TURNS: u32 = 400;
const FORGE: u16 = 1;

fn kept(store: &Store) -> Vec<forge_top::Kept> {
    let mut rows = Vec::new();
    for record in store.rows.values() {
        if let Record::Forge { row } = record
            && let forge_top::Stored::Kept(row) = row.as_ref()
        {
            rows.push(*row);
        }
    }
    rows
}

fn changes() -> World {
    let mut world = World::boxed_setup();
    world.sign_in();
    world.forge_ask(
        100,
        forge_top::Ask::SetUp {
            org: b"org".as_slice().into(),
            repository: b"repo".as_slice().into(),
            project: forge_top::SetUpProject::Existing(1),
            playground: false,
        },
    );
    world.until(Until::Ready);
    let request = world
        .setup_requests
        .iter()
        .find_map(|request| match request {
            engine::Request::SetUp { request, .. } => Some(*request),
            _ => None,
        })
        .expect("owned group asked");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::SetUpAnswered {
        forge: FORGE,
        request,
        answer: forge_top::SetUpAnswer::SettingUp,
    })));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Connection {
        forge: FORGE,
        state: forge_top::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    for key in [93, 94] {
        let answer = change_tool::change(&mut world, &chat, key, None, b"Small fix");
        assert!(matches!(answer, engine::CallAnswer::Delegated(_)), "{answer:?}");
    }
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let parked = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Parked,
        engine::Parts::none(),
    );
    world.send(parked);
    world.until(Until::Ready);
    let mut live = 0;
    for record in world.store.rows.values() {
        if let Record::Forge { row } = record
            && let forge_top::Stored::Change(change) = row.as_ref()
            && !matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
        {
            live += 1;
        }
    }
    assert_eq!(live, 2, "two admitted live changes before removal");
    world.hold_write_answers = Some(HoldWrite::All);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Enqueue {
        entry: skein_forge_client::Entry {
            number: 100,
            task: chat.task,
            repository: forge_world::REPO,
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::Status {
                    commit: translate::commit(1),
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
        world.tick();
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert!(!world.held_write_answers.is_empty(), "the fake applied the write but its answer is held");
    assert!(world.store.rows.contains_key(&Key::Forge(forge_top::Key::Writer(FORGE))));
    assert!(world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(100))));
    assert!(world.store.rows.contains_key(&Key::Forge(forge_top::Key::Flight { task: chat.task, entry: 100 })));
    world.setup_requests.clear();
    world
}

fn finish(world: &mut World) {
    world.box_removed(FORGE, true);
    for _ in 0..END_TURNS {
        world.tick();
    }
}

fn whole(world: &World) {
    assert!(
        !world.store.rows.contains_key(&Key::Forge(forge_top::Key::Forge(FORGE))),
        "forge erased; kept={:?}",
        kept(&world.store)
    );
    assert!(kept(&world.store).is_empty());
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Writer(FORGE))));
    let mut repositories = 0;
    for record in world.store.rows.values() {
        if let Record::Forge { row } = record {
            match row.as_ref() {
                forge_top::Stored::Entry { entry, .. } => assert_ne!(entry.repository.forge, FORGE),
                forge_top::Stored::Flight { .. } => panic!("removed forge retains a flight"),
                forge_top::Stored::Change(change) => {
                    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change.task)));
                    assert!(matches!(world.store.rows.get(&key),
                        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task))))
                            if matches!(task.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. }))));
                }
                forge_top::Stored::Repository(repository) => {
                    assert_eq!(repository.provider.forge, FORGE);
                    assert_eq!(repository.owner.as_ref(), b"org");
                    assert_eq!(repository.name.as_ref(), b"repo");
                    repositories += 1;
                }
                _ => {}
            }
        }
    }
    assert_eq!(repositories, 1, "historical repository stays");
    assert!(world.root.forges().rows.is_empty());
}

fn answer_writes(cuts: &[temper_engine_domain_world::forge_routes::CommitCut], key: people::ActKey) -> u32 {
    let mut count = 0;
    for cut in cuts {
        for write in &cut.writes {
            if let engine::Write::Save(Record::Core(jig_domain_core::Record::People(people::Stored::Acted {
                key: saved,
                ..
            }))) = write
                && *saved == key
            {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn a_removal_cut_after_each_kept_act_ends_whole() {
    let mut world = changes();
    world.commit_cuts = Some(Vec::new());
    world.forge_ask(101, forge_top::Ask::RemoveForge { forge: FORGE });
    // The decision and the core's immediately answered cancel commit together.
    for _ in 0..20 {
        world.tick();
    }
    let saved = kept(&world.store);
    assert_eq!(saved.len(), 2, "second Cancel and BoxRemove remain: {saved:?}");
    assert!(saved.iter().any(|row| matches!(row.act, forge_top::KeptAct::Cancel { .. })));
    assert!(saved.iter().any(|row| row.act == forge_top::KeptAct::BoxRemove { forge: FORGE }));
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Forge(FORGE))) else {
        panic!("the removal decision keeps its forge")
    };
    assert!(matches!(row.as_ref(), forge_top::Stored::Forge(forge) if forge.removing));
    world.restart_removal(world.store.fork());
    world.until(Until::Ready);
    finish(&mut world);
    whole(&world);
    let cuts = world.commit_cuts.take().expect("recorded removal commits");
    assert!(!cuts.is_empty());
    let mut pending_cut = false;
    for cut in cuts {
        let store = cut.store;
        let erased = !store.rows.contains_key(&Key::Forge(forge_top::Key::Forge(FORGE)));
        let saved = kept(&store);
        let mut answered = Vec::new();
        for record in store.rows.values() {
            if let Record::Core(jig_domain_core::Record::People(people::Stored::Acted {
                key,
                act: people::Act::Cancel { .. },
                ..
            })) = record
            {
                answered.push(*key);
            }
        }
        world.env.now = cut.now;
        world.env.wall = cut.wall;
        world.commit_cuts = Some(Vec::new());
        pending_cut |= !kept(&store).is_empty();
        let expected_box =
            kept(&store).iter().filter(|row| matches!(row.act, forge_top::KeptAct::BoxRemove { forge: FORGE })).count();
        world.setup_requests.clear();
        world.restart_removal(store);
        world.until(Until::Ready);
        let mut sent = 0;
        for request in &world.setup_requests {
            if let engine::Request::BoxRemove { forge: FORGE } = request {
                sent += 1;
            }
        }
        assert_eq!(sent, expected_box, "one resend per kept box act; writes={:?}", cut.writes);
        finish(&mut world);
        whole(&world);
        let resumed = world.commit_cuts.take().expect("recorded removal commits");
        for row in &saved {
            if let forge_top::KeptAct::Cancel { .. } = row.act {
                let key = people::ActKey { connector: 0, key: row.key };
                assert_eq!(answer_writes(&resumed, key), 1, "saved Cancel answered once after restart");
            }
        }
        for key in answered {
            assert_eq!(answer_writes(&resumed, key), 0, "a Cancel answered before the cut is not repeated");
        }
        if erased {
            break;
        }
    }
    assert!(pending_cut, "at least one crash cut must retain an unanswered act");
}

#[test]
fn a_failed_box_remove_is_sent_again_after_a_restart() {
    let mut world = changes();
    world.forge_ask(101, forge_top::Ask::RemoveForge { forge: FORGE });
    for _ in 0..20 {
        world.tick();
    }
    world.box_removed(FORGE, false);
    for _ in 0..END_TURNS {
        world.tick();
    }
    assert!(kept(&world.store).iter().any(|row| matches!(row.act, forge_top::KeptAct::BoxRemove { .. })));
    world.store = world.store.fork();
    world.setup_requests.clear();
    world.restart(true, true);
    world.until(Until::Ready);
    let mut sent = 0;
    for request in &world.setup_requests {
        if let engine::Request::BoxRemove { forge: FORGE } = request {
            sent += 1;
        }
    }
    assert_eq!(sent, 1);
    finish(&mut world);
    whole(&world);
}

#[test]
fn nothing_of_a_removed_forge_is_read_but_its_repositories() {
    let mut world = changes();
    world.forge_ask(101, forge_top::Ask::RemoveForge { forge: FORGE });
    for _ in 0..20 {
        world.tick();
    }
    world.restart_removal(world.store.fork());
    world.until(Until::Ready);
    finish(&mut world);
    whole(&world);
    project_has_no_forge(&mut world);
    world.restart_removal(world.store.fork());
    world.until(Until::Ready);
    whole(&world);
    project_has_no_forge(&mut world);
    let before = world.forge_calls;
    world.forge_ask(121, forge_top::Ask::Reachable { forge: FORGE, installation: None, page: 1 });
    world.until(Until::Ready);
    assert!(matches!(
        world.applications.last(),
        Some(Some(engine::ApplicationAnswer::Reachable(forge_top::ReachableAnswer::Refused(
            forge_top::ForgeRefusal::NoSuchForge
        ))))
    ));
    assert_eq!(world.forge_calls, before, "no request reaches the removed forge");
}

fn project_has_no_forge(world: &mut World) {
    world.signed_in = None;
    world.sign_in();
    let answer = world.party_read(jig_domain_core::reads::Read::Project { project: 1 });
    let jig_domain_core::reads::Answer::Document(document) = answer else { panic!("project read: {answer:?}") };
    let jig_domain_core::documents::Document::Project(project) = *document else { panic!("project document") };
    assert!(project.resources.is_empty(), "ended repositories leave the project's current resources");
}
