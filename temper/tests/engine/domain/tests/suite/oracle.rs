//! Oracle controls (jig/docs/design/domain/engine/world.md, 1).
use temper_engine_domain::Key;
use temper_engine_domain_world::{direct::Driver, store::Store};

#[test]
fn a_declared_rebuilt_start_mark_compares_equal() {
    let mut driver = Driver::new(Store::new());
    driver.settle();
    let before = driver.store.header();
    driver.close();
    assert!(driver.root.ready());
    assert!(driver.store.header().commits > before.commits);
}

#[test]
fn an_erased_forge_row_names_its_key() {
    let limits = temper_engine_domain_world::fixtures::forge_limits();
    let mut store =
        temper_engine_domain_world::fixtures::restored_store(temper_engine_domain_world::walking::config(71), &limits);
    store.seed(
        Box::new([temper_engine_domain::Record::Forge {
            row: Box::new(temper_engine_domain_forge::Stored::Binding {
                address: Box::new([42]),
                subject: Box::new([43]),
            }),
        }]),
        Box::new([]),
    );
    let mut driver = Driver::configured(
        store,
        [temper_engine_domain_world::walking::config(71), temper_engine_domain_world::walking::config(71)],
        &limits,
    );
    driver.settle();
    let key = Key::Forge(temper_engine_domain_forge::Key::Binding(Box::new([42])));
    assert!(driver.store.rows.contains_key(&key));
    let finding = driver.close_erasing(Some(key)).expect_err("erased writer row");
    assert!(finding.contains("restart oracle row Forge(Binding([42]))"), "{finding}");
    let rows = driver.store.rows.clone();
    let commits = driver.store.applied;
    driver.close();
    assert_eq!(driver.store.applied, commits, "a finding ends the oracle attempt");
    assert_eq!(driver.store.rows, rows);
}

#[test]
fn an_erased_forge_row_has_the_same_finding_without_kept_reads() {
    use temper_engine_domain_world::{fixtures, store, walking::config};

    let run = |keep| {
        let store = if keep { Store::new() } else { Store::with_read_keeping(&store::widest(), false) };
        let limits = fixtures::forge_limits();
        let mut store = fixtures::restored_over(store, config(71), &limits);
        store.seed(
            Box::new([temper_engine_domain::Record::Forge {
                row: Box::new(temper_engine_domain_forge::Stored::Binding {
                    address: Box::new([42]),
                    subject: Box::new([43]),
                }),
            }]),
            Box::new([]),
        );
        let mut driver = Driver::configured(store, [config(71), config(71)], &limits);
        driver.settle();
        let key = Key::Forge(temper_engine_domain_forge::Key::Binding(Box::new([42])));
        assert!(driver.store.rows.contains_key(&key));
        let finding = driver.close_erasing(Some(key)).expect_err("erased writer row");
        assert!(finding.contains("restart oracle row Forge(Binding([42]))"), "{finding}");
        let rows = driver.store.rows.clone();
        let commits = driver.store.applied;
        driver.close();
        assert_eq!(driver.store.applied, commits, "a finding ends the oracle attempt");
        assert_eq!(driver.store.rows, rows);
        finding
    };
    assert_eq!(run(true), run(false));
}

#[test]
fn closing_readback_is_identical_with_and_without_kept_reads() {
    use jig_domain_core::reads;
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;
    use skein_lib::{ReplyTo, Token};
    use temper_engine_domain as engine;
    use temper_engine_domain_world::{
        fixtures, forge_stand::ForgeStand, oracle::CloseObservation, protocol, store, walking::config,
    };

    let run = |keep| {
        let store = if keep { Store::new() } else { Store::with_read_keeping(&store::widest(), false) };
        let limits = fixtures::forge_limits();
        let mut driver = Driver::configured(store, [config(71), config(71)], &limits);
        driver.forge = Some(ForgeStand::new());
        driver.connect_forges();
        let channel = Token::new(7);
        driver.hello(channel);
        driver.settle();
        for _ in 0..2 {
            driver.sign_in();
            driver.settle();
        }
        let sign_in = driver.session();
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1001)),
            sign_in,
            key: [31; 16],
            ask: driver.chat_ask(1, b"What did you find?"),
        }));
        driver.settle();
        let engine::Assignment { task, attempt, .. } = driver.last_assignment();
        let revision = driver.task(task).expect("live chat").revisions.spec;
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1002)),
            sign_in,
            key: [32; 16],
            ask: people::Ask::Amend {
                project: 1,
                task,
                revision,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: Some(b"Keep the report short.".as_slice().into()),
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: None,
                    llm: None,
                    reason: b"Prefer a summary.".as_slice().into(),
                },
            },
        }));
        driver.settle();
        driver.send(protocol::turn(channel, task, attempt, 1, 1, None, b"checked the work"));
        driver.settle();
        driver.answer(
            channel,
            task,
            attempt,
            2,
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: b"All done.".as_slice().into() },
                cancel_delegates: false,
            },
            engine::Parts::none(),
        );
        driver.settle();
        assert_retained_logs(&mut driver, task, attempt, sign_in);
        driver.close_observation = Some(CloseObservation::default());
        driver.close();
        let observation = driver.close_observation.as_ref().expect("closing readback");
        assert_eq!(observation.queries.iter().filter(|(_, read)| *read == reads::Read::Frame).count(), 2);
        assert!(observation.forge_before.iter().any(Option::is_some));
        assert!(observation.forge_after.iter().any(Option::is_some));
        driver
    };
    let on = run(true);
    let off = run(false);
    assert_eq!(on.close_observation, off.close_observation);
    assert_eq!(on.transactions, off.transactions);
    assert_eq!(on.store.applied, off.store.applied);
    assert_eq!(on.store.rows, off.store.rows);
}

fn assert_retained_logs(driver: &mut Driver, task: u64, attempt: u64, sign_in: u64) {
    use jig_domain_core::{self as core, reads};
    use jig_domain_people as people;
    use jig_domain_tasks as tasks;
    use temper_engine_domain as engine;

    assert!(driver.store.rows.contains_key(&Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    let turns = engine::Range::Core(engine::CoreRange::Turns { task, attempt });
    let (rows, _) = driver.store.page(&turns, None, 1);
    assert_eq!(rows.len(), 1, "the ended task still has its turns log");
    let sign_in_key = Key::Core(core::Key::People(people::Key::SignIn(sign_in)));
    let Some(engine::Record::Core(core::Record::People(people::Stored::SignIn { person, .. }))) =
        driver.store.rows.get(&sign_in_key)
    else {
        panic!("the requester is signed in");
    };
    let history = driver.store.read(Box::new(reads::Range {
        party: *person,
        scope: reads::Scope::History { task, subtree: false },
        after: None,
        backwards: false,
        rows: 4,
        bytes: driver.env.limits.loads.read_bytes,
    }));
    let reads::Loaded::Page { rows, .. } = history else { panic!("ended history: {history:?}") };
    assert!(
        rows.iter().any(|row| matches!(&row.value,
            core::Record::Tasks(tasks::Stored::History(line)) if line.task == task && line.change == tasks::Change::Amended
        )),
        "the ended task still has its amendment in the history log"
    );
}

#[test]
fn a_deliberately_broken_fixture_skips_even_an_unsettled_root() {
    let mut driver = Driver::new(Store::new());
    let rows = driver.store.rows.clone();
    driver.broken("deliberately incomplete startup fixture exercises the opt-out");
    driver.close();
    assert_eq!(driver.store.rows, rows);
    assert!(!driver.root.ready());
}

#[test]
fn walking_answers_survive_a_cold_restart() {
    let mut world =
        temper_engine_domain_world::walking::World::new(temper_engine_domain_world::walking::Settings::calm(61));
    world.run();
    world.close();
}

#[test]
fn escalation_answers_survive_a_cold_restart() {
    use temper_engine_domain_world::escalation::{Settings, World};
    use temper_engine_domain_world::escalation_referee::Story;
    let mut world = World::new(Settings::calm(9100, Story::Release));
    world.run();
    world.close();
}

#[test]
fn role_answers_survive_a_cold_restart() {
    use temper_engine_domain_world::roles::{Base, Settings, World};
    let mut world = World::new(Settings::calm(9300, Base::Requester));
    world.run();
    world.close();
}

#[test]
fn forge_documents_survive_a_cold_restart() {
    let mut world = temper_engine_domain_world::forge_routes::World::new();
    world.sign_in();
    world.adopt();
    world.close();
}

#[test]
fn live_ci_is_rebuilt_but_terminal_ci_and_task_identity_are_durable() {
    use temper_engine_domain_forge::{Document, Panel, PanelStep, PanelUsed};
    use temper_engine_domain_forge_change::{Status, Way};
    use temper_engine_domain_world::oracle::durable;
    let panel = Panel {
        landing_held: false,
        source: None,
        way: Way::Queue,
        in_landing: None,
        task: 2,
        forge: 1,
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
        branch: b"work".as_slice().into(),
        landing: b"main".as_slice().into(),
        head: None,
        pull: None,
        step: PanelStep::Checking,
        hold: None,
        wait: None,
        runs: None,
        ci_required: true,
        ci: Status::Pending,
        gates: Box::new([]),
        approvals: Box::new([]),
        unmapped_reviews: Box::new([]),
        repairs: PanelUsed { used: 0, limit: 3 },
        resolutions: PanelUsed { used: 0, limit: 3 },
        updates: PanelUsed { used: 0, limit: 3 },
        questions: temper_engine_domain_forge::PanelQuestions {
            route: temper_engine_domain_forge_change::Route::None,
            taken: temper_engine_domain_forge::PanelUsed { used: 0, limit: 4 },
            open: Box::new([]),
        },
    };
    let mut changed = panel.clone();
    changed.ci = Status::Unknown;
    assert_eq!(
        durable::forge(Some(Document::Change(panel.clone()))),
        durable::forge(Some(Document::Change(changed.clone())))
    );
    changed.task += 1;
    assert_ne!(durable::forge(Some(Document::Change(panel.clone()))), durable::forge(Some(Document::Change(changed))));
    let mut ended = panel;
    ended.step = PanelStep::Ended;
    let mut changed = ended.clone();
    changed.ci = Status::Unknown;
    assert_ne!(durable::forge(Some(Document::Change(ended))), durable::forge(Some(Document::Change(changed))));
}
