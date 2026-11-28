use crate::settings::{self as s, Current, Edit, Failure, Fault, Item, Overlay, Setting, Value};
use crate::tests::*;

pub(super) fn with_model_speeds() -> (Config, Limits) {
    let mut config = config(1);
    let mut limits = limits();
    limits.run_bytes = 4096;
    speed_example(&mut config.settings.models[0]);
    config.defaults.models = s::catalogue::defaults(&config.settings);
    config.defaults.charters[0].spend = 1000;
    (config, limits)
}

fn charter_item(value: Charter) -> Item {
    Item { setting: Setting::Charter(value.number), edit: Edit::Set(Value::Charter(Box::new(value))) }
}

#[test]
fn charter_resolution_takes_each_named_speed_or_the_models_default() {
    let (mut config, limits) = with_model_speeds();
    let first = &mut config.defaults.models[0];
    first.model.speed = Some(b"fast".as_slice().into());
    let mut rest = first.clone();
    rest.number = 2;
    rest.model.name = b"other".as_slice().into();
    rest.model.speed = Some(b"ultrafast".as_slice().into());
    let catalogue = [first.clone(), rest];
    let mut charter = config.defaults.charters[0].clone();
    charter.rest = true;
    for (choice, resolved) in [
        (Some(b"ultrafast".as_slice()), Some(b"ultrafast".as_slice())),
        (Some(b"fast".as_slice()), Some(b"fast".as_slice())),
        (Some(tasks::STANDARD_SPEED), None),
        (None, Some(b"fast".as_slice())),
    ] {
        charter.policy.model.speed = match choice {
            Some(name) => Some(Box::from(name)),
            None => None,
        };
        let value = charters::resolved(&charter, &catalogue, &limits).expect("resolves");
        assert_eq!(value.policy.model.speed.as_deref(), resolved, "{choice:?}");
        assert_eq!(value.policy.alternatives[0].speed.as_deref(), Some(b"ultrafast".as_slice()));
    }
    charter.policy.model.speed = Some(tasks::STANDARD_SPEED.into());
    charter.policy.alternatives = Box::new([catalogue[1].model.clone()]);
    charter.policy.alternatives[0].speed = Some(b"fast".as_slice().into());
    let value = charters::resolved(&charter, &catalogue, &limits).expect("resolves named alternative");
    assert_eq!(value.policy.alternatives[0].speed.as_deref(), Some(b"fast".as_slice()));
}

#[test]
fn a_charter_cannot_keep_a_speed_its_catalogue_model_does_not_declare() {
    let (config, limits) = with_model_speeds();
    let mut charter = config.defaults.charters[0].clone();
    let mut model = config.defaults.models[0].clone();
    let current = Current::new(config.defaults, &limits, &[]);
    let check = |items: &[Item]| s::check_change(&current, items, &config.settings, &config.connectors, &limits);
    let failure = Err(Failure { setting: Setting::Charter(1), fault: Fault::Reference, detail: None });
    // The fixture declares its speeds only in the catalogue.
    for (speed, expected) in [
        (Some(b"fast".as_slice()), Ok(())),
        (Some(tasks::STANDARD_SPEED), Ok(())),
        (None, Ok(())),
        (Some(b"priority".as_slice()), failure),
        (Some(b"turbo".as_slice()), failure),
    ] {
        charter.policy.model.speed = match speed {
            Some(name) => Some(Box::from(name)),
            None => None,
        };
        assert_eq!(check(&[charter_item(charter.clone())]), expected, "charter {speed:?}");
        let overlay = Overlay { model: Some(charter.policy.model.clone()), ..Overlay::empty() };
        let item = Item { setting: Setting::Charter(1), edit: Edit::Set(Value::Overlay(Box::new(overlay))) };
        assert_eq!(check(&[item]), expected, "overlay {speed:?}");
    }
    // Leave a charter naming Fast, then edit only the model's declarations.
    charter.policy.model.speed = Some(b"fast".as_slice().into());
    let mut current = current;
    drop(s::apply(&mut current, &[charter_item(charter.clone())], 2));
    model.model.speed = None;
    model.model.speeds = Box::new([]);
    let item = Item { setting: Setting::Model(1), edit: Edit::Set(Value::Model(Box::new(model))) };
    let check = |items: &[Item]| s::check_change(&current, items, &config.settings, &config.connectors, &limits);
    assert_eq!(check(core::slice::from_ref(&item)), failure);
    for speed in [Some(tasks::STANDARD_SPEED), None] {
        charter.policy.model.speed = match speed {
            Some(name) => Some(Box::from(name)),
            None => None,
        };
        assert_eq!(check(&[item.clone(), charter_item(charter.clone())]), Ok(()));
    }
}

#[test]
fn a_charters_budget_must_hold_a_completion_at_its_named_speed() {
    // The fixture's standard prices round (8192 × 2500 + 1024 × 10000) / 1_000_000 up to 31.
    const STANDARD_COMPLETION: u64 = 31;
    let (mut config, limits) = with_model_speeds();
    config.defaults.models[0].model.speed = None;
    let mut charter = config.defaults.charters[0].clone();
    charter.policy.reserve_turns = 0;
    charter.spend = STANDARD_COMPLETION;
    let current = Current::new(config.defaults, &limits, &[]);
    let check =
        |charter| s::check_change(&current, &[charter_item(charter)], &config.settings, &config.connectors, &limits);
    charter.policy.model.speed = Some(tasks::STANDARD_SPEED.into());
    assert_eq!(check(charter.clone()), Ok(()));
    charter.policy.model.speed = Some(b"ultrafast".as_slice().into());
    assert_eq!(check(charter), Err(Failure { setting: Setting::Charter(1), fault: Fault::Budget, detail: None }));
}

#[test]
fn a_charter_whose_default_spend_misses_its_floor_is_refused() {
    let mut config = config(1);
    config.defaults.charters[0].spend = 1;
    let current = Current::new(config.defaults, &limits(), &[]);
    assert_eq!(
        s::check(&current, &config.settings, &config.connectors, &limits()),
        Err(Failure { setting: Setting::Charter(1), fault: Fault::Budget, detail: None })
    );
}

pub(in crate::tests) fn context(task: u64, charter: u32) -> Box<tasks::RunContext> {
    Box::new(tasks::RunContext {
        session_wake: false,
        session_kinds: tasks::Reasons::EMPTY,
        reasons: jig_domain_tasks::Reasons::FIRST,
        ended_by: tasks::EndedBy::Result,
        added: Box::new([]),
        pending: Box::new([]),
        llm: None,
        payer: tasks::Payer::Party(1),
        task,
        last_message: 0,
        inbox: Box::new([]),
        delegates: Box::new([]),
        dependencies: Box::new([]),
        saved: Box::new([]),
        previous_attempt: 0,
        ever_turned: false,
        tries: tasks::Tries::NONE,
        invalid_result: None,
        project: 1,
        executor: tasks::Executor::Agent { charter },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"task".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
        requester: tasks::Party::Person(1),
        authority: task_authority(&config(1).seed_projects[0].policy.policy.roles[0].authority),
        numbers: tasks::Numbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
    })
}

#[test]
fn a_full_store_work_lane_does_not_delay_a_procedure() {
    let (mut core, limits) = core();
    core.opened = true;
    make_task(&mut core, &limits, 1);
    let mut procedure = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.llm = None;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([procedure]));
    for n in 0..limits.load_slots {
        fixture_window(&mut core, 100 + u64::from(n), (1, 1));
    }
    assert!(!core.loads.room(&limits));
    core.due.push(core.tasks.run_context(2).unwrap());
    let context = core.take_due().expect("a procedure has no window to load");
    let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true });
    for request in out {
        if let Request::Ask { connector: 1, ask: Ask::StartProcedure { context, step: 1 } } = request {
            assert_eq!(context.task, 2);
            return;
        }
    }
    panic!("ready procedure asks its connector");
}

#[test]
fn take_due_skips_an_unusable_account_and_starts_a_procedure() {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut procedure = new_row(&core, 2, tasks::Funder::Period { project: 1, period: 1 }, None);
    procedure.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
    procedure.llm = None;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([procedure]));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::uncapped();
    accounts::step(&mut core.accounts, &env, accounts::Event::Want, &mut out);
    while out.pop().is_some() {}
    accounts::step(&mut core.accounts, &env, accounts::Event::Limited { account: 1, reset: None }, &mut out);
    core.due.push(core.tasks.run_context(1).unwrap());
    core.due.push(core.tasks.run_context(2).unwrap());
    assert!(core.due_ready());
    assert_eq!(core.take_due().expect("procedure ready").task, 2);
    assert!(!core.due_ready());
    assert_eq!(core.due.pop().expect("account wait retains order").task, 1);
}

#[test]
fn startup_needs_own_models_and_preserve_charter_requirements() {
    let mut config = config(1);
    let mut limits = limits();
    limits.tasks.charters = 2;
    limits.run_bytes = 1024;
    let mut alternative = scripted_model();
    alternative.name = b"alternative".as_slice().into();
    alternative.window = 4096;
    alternative.output = 512;
    config.settings.models = Box::new([scripted_model(), alternative.clone()]);
    config.defaults.charters[0].policy.alternatives = Box::new([alternative]);
    let mut engine = config.defaults.charters[0].clone();
    engine.number = 2;
    engine.hosts = fleet::Kinds::Engine;
    engine.policy.modify = false;
    config.defaults.charters = Box::new([config.defaults.charters[0].clone(), engine]);
    charters::assert_catalogue(&config.settings);

    let needs = fleet_needs::configured(&config, &limits);
    assert_eq!(needs.spend, config.defaults.rules.maximum_run_spend);
    assert_eq!(needs.message_bytes, limits.tasks.message_bytes);
    assert_eq!(needs.start_messages, limits.tasks.run_inbox);
    assert_eq!(needs.unacknowledged, config.hub_turns);
    drop(config);

    assert_eq!(needs.charters.len(), 2);
    let worker = &needs.charters[0];
    assert_eq!(worker.version, fleet::Version { charter: 1, revision: 1 });
    assert_eq!(worker.kinds, fleet::Kinds::Workers);
    assert_eq!(worker.turns, 8);
    assert_eq!(worker.time, Duration::from_secs(60));
    assert_eq!(worker.waiting, Duration::from_secs(1));
    assert_eq!(worker.change_reserve, Some(Duration::from_secs(10)));
    assert_eq!(
        worker.models.as_ref(),
        &[
            fleet::ModelNeed { endpoint: 1, name: b"scripted".as_slice().into(), window: 8192, output: 1024 },
            fleet::ModelNeed { endpoint: 1, name: b"alternative".as_slice().into(), window: 4096, output: 512 },
        ]
    );
    let engine = &needs.charters[1];
    assert_eq!(engine.version, fleet::Version { charter: 2, revision: 1 });
    assert_eq!(engine.kinds, fleet::Kinds::Engine);
    assert_eq!(engine.change_reserve, None);
}
