//! The current catalogue and revision-held resolutions (domain/engine/settings.md, 11).
use crate::settings_support::*;
use jig_core_world::{
    client_peer::ClientPeer,
    fixtures, peers,
    world::{Boot, Setup, World},
};
use jig_domain_core::{self as core, settings as s};
use jig_domain_fleet as fleet;
use jig_domain_link::Hello;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Queue, ReplyTo, Token};

fn configured(count: u32) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = configuration();
    limits.core.settings.alternatives = 7;
    config.core.defaults.charters[0].rest = true;
    let first = config.core.settings.models[0].clone();
    config.core.defaults.models = (1..=count)
        .map(|number| {
            let mut model = first.clone();
            if number != 1 {
                model.name = format!("stock-{number}").into_bytes().into_boxed_slice();
            }
            s::CatalogueModel {
                first_read: None,
                number,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: s::ModelSource::Stock,
                estimated: false,
            }
        })
        .collect();
    config.core.settings.models = config.core.defaults.models.iter().map(|entry| entry.model.clone()).collect();
    peers::journal(&mut limits);
    (config, limits)
}
fn start_with(config: root::Config, limits: &root::Limits) -> (World, u64) {
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits: *limits,
        boot: Boot::Empty,
        ..Setup::default()
    });
    assert!(world.domain.ready(), "catalogue start refused: {:?}", world.domain.start_failure());
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    (world, signed)
}
fn setup(count: u32) -> (World, u64) {
    let (config, limits) = configured(count);
    start_with(config, &limits)
}
fn entry(world: &World, number: u32) -> s::CatalogueModel {
    let Some(s::Value::Model(value)) = world.domain.settings().value(s::Setting::Model(number)) else {
        panic!("catalogue row")
    };
    value.as_ref().clone()
}
fn added(world: &World, number: u32) -> s::CatalogueModel {
    let mut value = entry(world, 1);
    value.number = number;
    value.model.name = b"person-model".as_slice().into();
    value.source = s::ModelSource::Person;
    value
}
fn set(value: s::CatalogueModel) -> s::Item {
    s::Item { setting: s::Setting::Model(value.number), edit: s::Edit::Set(s::Value::Model(Box::new(value))) }
}
fn apply(world: &mut World, signed: u64, item: s::Item) {
    let before = world.domain.settings().newest();
    changed(
        &change(world, signed, u8::try_from(before + 20).expect("story key"), before, Box::new([item])),
        before + 1,
    );
}
fn make(world: &mut World, signed: u64, model: Option<u8>) -> u64 {
    let mut member = jig_test_profile::fixtures::people_member(b"catalogue task", 20);
    member.llm = model.map(|model| people::LlmChoice {
        later: None,
        account: None,
        main: people::ModelEffort { model, effort: None },
        subagents: Box::new([]),
    });
    let reply = ask(world, signed, 90, people::Ask::Create { project: 1, batch: Box::new([member]) });
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = reply else { panic!("created: {reply:?}") };
    first
}
pub(super) fn host(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}
fn versions(world: &World) -> usize {
    world
        .store
        .rows
        .keys()
        .filter(|key| matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Settings(s::Key::Charter { .. })))))
        .count()
}

#[test]
fn a_set_model_is_in_the_next_tasks_charter_with_no_charter_change() {
    let (mut world, signed) = setup(1);
    let original = world.domain.settings().charter(1).expect("set charter").clone();
    let value = added(&world, 9);
    apply(&mut world, signed, set(value.clone()));
    assert_eq!(world.domain.settings().charter(1), Some(&original));
    assert_eq!(world.domain.core_mut().charter(1).expect("resolution").policy.alternatives.as_ref(), &[value.model]);
    host(&mut world);
    let me = frame(&mut world, signed);
    assert!(me.projects.as_slice()[0].offer.as_slice().iter().any(|row| row.model == 8));
    let task = make(&mut world, signed, Some(8));
    assert_eq!(fixtures::live_row(&world, task).settings.task, 2);
    assert_eq!(world.assigned.as_ref().expect("new model run").run.policy.model.name.as_ref(), b"person-model");
    fixtures::finish_chat(&mut world, 0, b"Done");
}

#[test]
fn a_task_keeps_the_model_it_has() {
    let (mut world, signed) = setup(2);
    let task = make(&mut world, signed, Some(0));
    host(&mut world);
    let original = world.assigned.as_ref().expect("first run").run.policy.model.clone();
    world.end(tasks::End::Failed(tasks::Class::Transient), 0);
    let mut replacement = entry(&world, 1);
    replacement.model.input_price = 0;
    replacement.model.window -= 1024;
    apply(&mut world, signed, set(replacement));
    apply(&mut world, signed, s::Item { setting: s::Setting::Model(1), edit: s::Edit::Removed });
    assert_eq!(world.domain.core_mut().charter_of(task).policy.model, original);
    world.retry_due();
    assert_eq!(world.assigned.as_ref().expect("next run").run.policy.model, original);
    assert_eq!(world.assigned.as_ref().expect("next run").task, task);
    assert_eq!(versions(&world), 1, "only the revision the task holds survives");
    fixtures::finish_chat(&mut world, 0, b"Done");
}

#[test]
fn a_model_a_person_turned_off_stays_off_across_a_release() {
    let (mut world, signed) = setup(1);
    let mut off = entry(&world, 1);
    off.picker = false;
    apply(&mut world, signed, set(off.clone()));
    let (mut next, _) = configured(1);
    next.core.defaults.models[0].description = Some(b"new release".as_slice().into());
    restart(&mut world, next);
    assert_eq!(entry(&world, 1), off);
    assert_eq!(world.domain.settings().newest(), 2, "the person's overlay masks the default");
}

#[test]
fn a_release_adds_its_new_model_at_the_next_start() {
    let (mut world, signed) = setup(1);
    let task = make(&mut world, signed, None);
    let (next, _) = configured(2);
    restart(&mut world, next);
    assert_eq!(revision(&world, 2, false).head.by, s::Decider::Release);
    assert_eq!(entry(&world, 2).model.name.as_ref(), b"stock-2");
    assert_eq!(versions(&world), 1);
    assert!(world.domain.core_mut().charter_of(task).policy.alternatives.is_empty());
    assert_eq!(world.domain.core_mut().charter(1).expect("release resolution").policy.alternatives.len(), 1);
    let signed = sign_in(&mut world);
    host(&mut world);
    assert!(frame(&mut world, signed).projects.as_slice()[0].offer.as_slice().iter().any(|row| row.model == 1));
    fixtures::finish_chat(&mut world, 0, b"Done");
}

#[test]
fn a_pickers_flip_makes_no_charter_version() {
    let (mut world, signed) = setup(1);
    make(&mut world, signed, None);
    let mut value = entry(&world, 1);
    value.picker = false;
    apply(&mut world, signed, set(value));
    assert_eq!(versions(&world), 0);
    assert_eq!(world.domain.settings().lookup(s::Setting::Charter(1)).1, Some((false, 1)));
}

#[test]
fn a_prices_change_makes_a_charter_version() {
    let (config, mut limits) = configured(1);
    limits.core.settings.versions = 0;
    let (mut full, signed) = start_with(config, &limits);
    let task = make(&mut full, signed, None);
    let mut value = entry(&full, 1);
    value.model.input_price = 0;
    let reply = change(&mut full, signed, 21, 1, Box::new([set(value)]));
    assert!(
        matches!(reply, people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Full {
        revision: 1, tasks, more: 0,
    })) if tasks.as_ref() == [task]),
        "a model edit prices the old resolution as a held version"
    );
    assert_eq!(full.domain.settings().newest(), 1);
    let (mut world, signed) = setup(1);
    let task = make(&mut world, signed, None);
    let original = world.domain.core_mut().charter_of(task).clone();
    let mut value = entry(&world, 1);
    value.model.input_price = 0;
    apply(&mut world, signed, set(value.clone()));
    assert_eq!(versions(&world), 1);
    assert_eq!(world.domain.settings().lookup(s::Setting::Charter(1)).1, Some((false, 2)));
    assert_eq!(world.domain.core_mut().charter_of(task), &original);
    assert_eq!(world.domain.core_mut().charter(1).expect("current resolution").policy.model, value.model);
}

#[test]
fn a_withdrawn_model_leaves_the_offer_and_the_resolution() {
    for (source, withdrawn, expected) in [
        (s::ModelSource::Stock, false, s::ModelSource::Stock),
        (s::ModelSource::List, false, s::ModelSource::Person),
        (s::ModelSource::List, true, s::ModelSource::Person),
        (s::ModelSource::Person, false, s::ModelSource::Person),
    ] {
        let (mut config, limits) = configured(2);
        config.core.defaults.charters[0].policy.alternatives = Box::new([config.core.defaults.models[1].model.clone()]);
        config.core.defaults.models[0].source = source;
        config.core.defaults.models[0].withdrawn = withdrawn;
        let (mut world, signed) = start_with(config, &limits);
        apply(&mut world, signed, s::Item { setting: s::Setting::Model(1), edit: s::Edit::Removed });
        assert!(entry(&world, 1).withdrawn);
        assert_eq!(entry(&world, 1).source, expected);
        assert_eq!(
            world.domain.core_mut().charter(1).expect("next named model").policy.model.name.as_ref(),
            b"stock-2"
        );
        assert!(!frame(&mut world, signed).projects.as_slice()[0].offer.as_slice().iter().any(|row| row.model == 0));
    }
}

#[test]
fn a_charter_left_with_no_model_is_refused_as_a_reference() {
    let (mut world, signed) = setup(1);
    let before = settings_rows(&world);
    let reply = change(
        &mut world,
        signed,
        21,
        1,
        Box::new([s::Item { setting: s::Setting::Model(1), edit: s::Edit::Removed }]),
    );
    assert!(
        matches!(
            reply,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
                why: people::Malformed::Settings { setting: s::Setting::Charter(1), fault: s::Fault::Reference, .. },
                ..
            }))
        ),
        "{reply:?}"
    );
    assert_eq!(settings_rows(&world), before);
}

#[test]
fn a_number_no_setting_has_is_a_hole() {
    let (mut world, signed) = setup(2);
    let value = added(&world, 64);
    apply(&mut world, signed, set(value));
    assert_eq!(world.domain.core_mut().settings.models.len(), 64);
    assert!(world.domain.core_mut().settings.models[2..63].iter().all(core::Model::is_hole));
    host(&mut world);
    let me = frame(&mut world, signed);
    assert_eq!(
        me.projects.as_slice()[0].offer.as_slice().iter().map(|row| row.model).collect::<Vec<_>>(),
        vec![0, 1, 63]
    );
    let mut member = jig_test_profile::fixtures::people_member(b"hole", 20);
    member.llm = Some(people::LlmChoice {
        later: None,
        account: None,
        main: people::ModelEffort { model: 31, effort: None },
        subagents: Box::new([]),
    });
    let reply = ask(&mut world, signed, 90, people::Ask::Create { project: 1, batch: Box::new([member]) });
    assert!(
        matches!(
            reply,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Llm {
                model: 31,
                why: people::LlmWhy::Unconfigured | people::LlmWhy::Charter,
            }))
        ),
        "{reply:?}"
    );
    assert_eq!(world.assignments.len(), 0);
}

#[test]
fn past_its_bound_a_charter_leaves_models_out() {
    let (config, mut limits) = configured(5);
    limits.core.settings.alternatives = 2;
    let (mut world, signed) = start_with(config, &limits);
    assert_eq!(world.domain.core_mut().charter(1).expect("bounded resolution").policy.alternatives.len(), 2);
    assert_eq!(frame(&mut world, signed).projects.as_slice()[0].left_out, 2);
    let bytes = core::policy_owned_bytes(&world.domain.core_mut().charter(1).expect("bounded resolution").policy)
        .expect("run bytes");
    for (room, alternatives, omitted) in [(bytes, 2, 2), (bytes - 1, 1, 3)] {
        let (mut config, mut limits) = configured(5);
        limits.core.run_bytes = u32::try_from(room).expect("fixture run bound");
        limits.core.tasks.charters = 2;
        let mut chat = config.core.defaults.charters[0].clone();
        chat.number = 2;
        config.core.defaults.charters[0].rest = false;
        config.core.defaults.charters = Box::new([config.core.defaults.charters[0].clone(), chat]);
        config.core.seed_projects[0].policy.templates = Box::new([people::TemplateOverride {
            template: 1,
            executor: Some(people::TemplateExecutor::Charter(2)),
            budget: None,
            deadline: None,
            priority: None,
        }]);
        let (mut bounded, owner) = start_with(config, &limits);
        assert_eq!(
            bounded.domain.core_mut().charter(2).expect("chat resolution").policy.alternatives.len(),
            alternatives
        );
        assert_eq!(frame(&mut bounded, owner).projects.as_slice()[0].left_out, omitted);
    }
}

#[test]
fn a_change_asks_the_root_to_tell_the_catalogue_before_its_commit() {
    let (mut world, signed) = setup(1);
    let value = added(&world, 9);
    let env = world.env().with(world.env().limits.core);
    let mut out = core::step(
        world.domain.core_mut(),
        &env,
        core::Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(SEED)),
            sign_in: signed,
            key: [21; 16],
            digest: [21; 32],
            change: Box::new(s::Change {
                against: 1,
                items: Box::new([set(value.clone())]),
                running: s::Running::Stays,
            }),
        },
    );
    let mut told = 0;
    let mut committed = false;
    while let Some(request) = out.pop() {
        match request {
            core::Request::Ask { ask: core::Ask::Catalogue { models }, .. } => {
                assert!(!committed, "catalogue precedes the revision's commit writes");
                assert!(models.contains(&value));
                told += 1;
            }
            core::Request::Write(core::Write::Save(core::Record::Core(core::CoreRecord::Settings(
                s::Record::Revision { .. },
            )))) => committed = true,
            _ => {}
        }
    }
    assert_eq!(told, 1);
    assert!(committed);
    world.broken(); // Inspecting the core's output bypasses the root's journal in this story.
}

#[test]
fn a_starts_restore_asks_once_with_a_persons_model() {
    let (mut world, signed) = setup(1);
    let value = added(&world, 64);
    apply(&mut world, signed, set(value.clone()));
    let (config, _) = configured(1);
    let env = world.env().with(world.env().limits.core);
    let mut restored = core::Core::new(config.core, &env.limits).expect("restart core");
    fixtures::phase(&restored.start_begin(false, env.wall), core::start::Phase::LoadCore);
    for range in core::RestoreRange::LIVE {
        for (key, row) in &world.store.rows {
            if let root::Key::Core(key) = key
                && range.admits(key)
            {
                let root::Record::Core(row) = row else { panic!("core row") };
                assert_ne!(restored.restore(&env, row.clone()), core::Restore::Refused);
            }
        }
    }
    let mut out = Queue::uncapped();
    let _action = fixtures::link(&mut restored, &env, &mut out);
    let mut told = 0;
    while let Some(request) = out.pop() {
        if let core::Request::Ask { ask: core::Ask::Catalogue { models }, .. } = request {
            assert!(models.contains(&value));
            told += 1;
        }
    }
    assert_eq!(told, 1);
    assert_eq!(restored.settings.models[63], value.model);
}
