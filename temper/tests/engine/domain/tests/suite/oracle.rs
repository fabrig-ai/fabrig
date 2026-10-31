//! Oracle controls (jig/docs/design/domain/engine.md, 15).
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
#[should_panic(expected = "restart oracle row Forge(Binding([42]))")]
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
    driver.close_erasing(Some(key));
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
        runs: None,
        ci: Status::Pending,
        gates: Box::new([]),
        repairs: PanelUsed { used: 0, limit: 3 },
        resolutions: PanelUsed { used: 0, limit: 3 },
        updates: PanelUsed { used: 0, limit: 3 },
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

#[test]
#[should_panic(expected = "story failure before startup")]
fn dropping_during_a_story_panic_preserves_that_failure() {
    let _driver = Driver::cold(
        Store::new(),
        temper_engine_domain_world::walking::config(71),
        &temper_engine_domain_world::walking::limits(),
    );
    panic!("story failure before startup");
}
