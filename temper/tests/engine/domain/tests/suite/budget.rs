//! Activation alone waits on the root's transcript and turn budget (domain/root.md, 4.1).
//! A goal's spend can be raised by its person's amendment.
use super::forge_routes::{assigned_chat, live_task};
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Record};
use temper_engine_domain_world::{
    direct::Driver,
    forge_routes::{Until, World},
    protocol,
    store::Store,
    walking::{self, config, limits},
};

const WORKER: Token = Token::new(7);
const READ: Token = Token::new(900);
const BODY: &[u8] = b"committed history";
const STEPS: usize = 200;
// The fixture's configured resume and text-tail allowances, per preparation.
const ALLOWANCE: u64 = 256 + 128;

fn bounds() -> engine::Limits {
    let mut bounds = limits();
    bounds.tasks.tasks = 4;
    bounds.forge.tasks = 4;
    bounds.tasks.project_tasks = 4;
    bounds.tasks.project_goals = 4;
    bounds.people.requests = 16;
    bounds.people.inbox_tasks = 8;
    bounds.people.inbox_recipients = 32;
    bounds.fleet.slots = 3;
    bounds.fleet.attempts = 4;
    bounds.fleet.grants = 4;
    bounds.fleet.turns = 4;
    bounds.loads.loads = 4;
    bounds.brief.briefs = 4;
    bounds.forge.brief_sections = 4 * bounds.brief.sections * 2;
    bounds
}

// Three real parked conversations with committed history, ready to wake again.
fn configuration(settings: bool) -> engine::Config {
    let mut configuration = config(91);
    if settings {
        configuration.core.kinds =
            temper_engine_service_stock::stock::kinds(&bounds(), configuration.core.connectors[0])
                .into_iter()
                .filter(|kind| kind.number == 22)
                .collect();
        configuration.kinds_stock =
            temper_engine_service_stock::stock::forge(&bounds()).into_iter().filter(|stock| stock.kind == 22).collect();
    }
    configuration
}

fn parked(settings: bool) -> (Store, Vec<engine::Assignment>) {
    let mut driver = Driver::configured(Store::new(), [config(91), configuration(settings)], &bounds());
    driver.send(protocol::hello_slots(WORKER, 3, Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut runs = Vec::new();
    for key in 1..=3 {
        let people::Ask::Create { project, mut batch } = driver.chat_ask(1, b"budget conversation") else {
            unreachable!()
        };
        batch[0].flags.ended_by = people::EndedBy::Requester;
        batch[0].budget = people::Budget::Named;
        batch[0].authority.spend = 100;
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1000 + u64::from(key))),
            sign_in: driver.session(),
            key: [key; 16],
            ask: people::Ask::Create { project, batch },
        }));
        driver.settle();
        let run = driver.last_assignment();
        assert!(
            runs.iter().all(|previous: &engine::Assignment| previous.task != run.task),
            "each ask created its own chat"
        );
        driver.send(protocol::turn(WORKER, run.task, run.attempt, 1, 1, None, BODY));
        driver.settle();
        driver.answer(WORKER, run.task, run.attempt, 1, tasks::End::Parked, engine::Parts::none());
        driver.settle();
        runs.push(run);
    }
    (driver.store.fork(), runs)
}

fn wake(driver: &mut Driver, runs: &[engine::Assignment]) {
    for (index, run) in runs.iter().enumerate() {
        let key = u8::try_from(index).expect("three tasks") + 20;
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(2000 + u64::from(key))),
            sign_in: driver.store.header().sign_ins,
            key: [key; 16],
            ask: people::Ask::Say { project: 1, task: run.task, words: b"wake".as_slice().into() },
        }));
    }
}

// Delay only preparation transcript pages; store commits, reads and fleet progress continue.
fn hold_transcripts(driver: &mut Driver, held: &mut Vec<engine::Event>) {
    for _ in 0..STEPS {
        let mut other = std::collections::VecDeque::new();
        while let Some(event) = driver.events.pop_front() {
            if matches!(event, engine::Event::Store(engine::StoreInput::Loaded { texts: Some(_), .. })) {
                held.push(event);
            } else {
                other.push_back(event);
            }
        }
        driver.events = other;
        driver.advance(true);
    }
}

fn assigned(driver: &Driver) -> usize {
    driver.delivered.iter().filter(|out| matches!(out, Delivery::Assigned { .. })).count()
}

#[test]
fn due_tasks_activate_within_the_byte_budget_as_earlier_runs_drain() {
    let (store, runs) = parked(false);
    let mut driver = Driver::with_budget(store, config(91), &bounds(), 2 * ALLOWANCE);
    driver.settle();
    driver.send(protocol::hello_slots(WORKER, 3, Box::new([])));
    driver.settle();
    wake(&mut driver, &runs);
    let mut held = Vec::new();
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 2, "only two preparations start while their full allowances are held");
    assert_eq!(driver.root.held_bytes(), 2 * ALLOWANCE);
    driver.send(held.remove(0));
    hold_transcripts(&mut driver, &mut held);
    assert!(driver.root.held_bytes() >= driver.env.limits.transcript_memory);
    assert!(assigned(&driver) >= 1, "fleet places a completed preparation while bytes bind");
    assert_eq!(held.len(), 2, "the third activation follows the first preparation's drain");
    for event in held {
        driver.send(event);
    }
    driver.settle();
    assert_eq!(assigned(&driver), 3);
    let runs: Vec<_> = driver
        .delivered
        .iter()
        .filter_map(|out| {
            if let Delivery::Assigned { assignment, .. } = out {
                Some((assignment.task, assignment.attempt))
            } else {
                None
            }
        })
        .collect();
    for (task, attempt) in runs {
        driver.send(protocol::turn(WORKER, task, attempt, 1, 1, None, BODY));
    }
    driver.settle();
    assert_eq!(driver.root.held_bytes(), 0);
}

#[test]
fn a_restart_with_three_due_tasks_activates_them_within_the_budget() {
    let (mut store, runs) = parked(false);
    let mut saves = Vec::new();
    for (index, run) in runs.iter().enumerate() {
        let key = engine::Key::Core(core::Key::Tasks(tasks::Key::Live(run.task)));
        let Record::Core(core::Record::Tasks(tasks::Stored::Live(mut row))) = store.rows[&key].clone() else {
            panic!("live parked conversation")
        };
        row.phase = tasks::Phase::Active(if index == 0 { tasks::Active::Preparing } else { tasks::Active::Due });
        saves.push(Record::Core(core::Record::Tasks(tasks::Stored::Live(row))));
    }
    store.seed(saves.into_boxed_slice(), Box::new([]));
    let mut driver = Driver::with_budget(store, config(91), &bounds(), 2 * ALLOWANCE);
    driver.send(protocol::hello_slots(WORKER, 3, Box::new([])));
    // Drive restart inputs without poll/Resume so Open's own behaviour is visible.
    for _ in 0..STEPS {
        if driver.root.ready() {
            break;
        }
        if let Some(event) = driver.events.pop_front() {
            driver.send(event);
        }
        if !driver.store.pending.is_empty() {
            let number = driver.store.apply();
            driver.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
        engine::release(&mut driver.root, &driver.env, &mut driver.out);
        driver.collect();
    }
    assert!(driver.root.ready());
    assert_eq!(driver.root.held_bytes(), 0, "Open starts no preparations even with a worker connected");
    let mut held = Vec::new();
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 2);
    driver.send(held.remove(0));
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 2);
    for event in held {
        driver.send(event);
    }
    driver.settle();
    assert_eq!(assigned(&driver), 3);
}

fn read_history(driver: &mut Driver, run: &engine::Assignment) {
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(READ),
        sign_in: driver.store.header().sign_ins,
        read: Box::new(reads::Read::Turn { task: run.task, attempt: run.attempt, turn: 1 }),
    }));
    for _ in 0..STEPS {
        driver.advance(true);
    }
    let answer = driver
        .delivered
        .iter()
        .find_map(|out| {
            if let Delivery::Read { reply, answer, .. } = out {
                (*reply == ReplyTo::new(READ)).then_some(answer)
            } else {
                None
            }
        })
        .expect("the person's read is answered while activation waits");
    let reads::Answer::Ended(past) = answer else { panic!("historical turn: {answer:?}") };
    let reads::EndedValue::Turn(turn) = &past.value else { panic!("requested turn") };
    assert_eq!(turn.transcript.as_ref(), BODY);
}

#[test]
fn a_durable_transcript_memory_raise_prepares_waiting_work() {
    let (store, runs) = parked(true);
    let mut driver = Driver::with_budget(store, configuration(true), &bounds(), ALLOWANCE);
    driver.settle();
    driver.send(protocol::hello_slots(WORKER, 3, Box::new([])));
    driver.settle();
    wake(&mut driver, &runs);
    let mut held = Vec::new();
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 1);
    assert_eq!(driver.root.held_bytes(), ALLOWANCE);
    let value = temper_engine_domain_forge::settings::Value::Limits(Box::new([core::settings::Override {
        name: b"transcript_memory".as_slice().into(),
        value: 4 * ALLOWANCE,
        reason: b"prepare waiting work".as_slice().into(),
    }]));
    let item = temper_engine_domain_world::kinds::item(22, 0, 1, &value);
    let revision = driver.root.settings_revision();
    driver.send(temper_engine_domain_world::kinds::change(
        driver.store.header().sign_ins,
        revision,
        51,
        Box::new([item]),
    ));
    assert!(!driver.immediate.iter().any(|request| matches!(request, engine::Request::LimitSettingsCommitted { .. })));
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 1, "the old budget remains until the service takes the durable word");
    assert!(driver.immediate.iter().any(|request| matches!(request, engine::Request::LimitSettingsCommitted { revision: durable, .. } if *durable == revision + 1)));
    driver.env.limits.transcript_memory = 4 * ALLOWANCE;
    driver.send(engine::Event::LimitsRaised);
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 3, "waiting work prepares without draining the previously held transcript");
    for event in held {
        driver.send(event);
    }
    driver.settle();
    assert_eq!(assigned(&driver), 3);
}

#[test]
fn a_persons_read_is_answered_while_the_budget_binds() {
    let (store, runs) = parked(false);
    let mut driver = Driver::with_budget(store, config(91), &bounds(), ALLOWANCE);
    driver.settle();
    driver.send(protocol::hello_slots(WORKER, 3, Box::new([])));
    driver.settle();
    wake(&mut driver, &runs);
    let mut held = Vec::new();
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 1, "one preparation fills the positive byte budget");
    assert_eq!(driver.root.held_bytes(), ALLOWANCE);
    read_history(&mut driver, &runs[0]);
    assert_eq!(assigned(&driver), 0);
    assert_eq!(driver.root.held_bytes(), ALLOWANCE, "the read leaves activation at the byte boundary");
    assert_eq!(driver.root.pending_assignments().len(), 0, "no assignment is ready while its transcript is held");
    driver.events.extend(held);
    driver.pump();
}

#[test]
fn with_no_worker_the_budget_stops_preparations_and_nothing_else() {
    let (store, runs) = parked(false);
    let mut driver = Driver::with_budget(store, config(91), &bounds(), 2 * ALLOWANCE);
    driver.settle();
    wake(&mut driver, &runs);
    read_history(&mut driver, &runs[0]);
    assert_eq!(driver.root.pending_assignments().len(), 0, "without a worker the tasks remain due");
    assert_eq!(driver.root.held_bytes(), 0);
    assert_eq!(assigned(&driver), 0);
    driver.send(protocol::hello_slots(WORKER, 3, Box::new([])));
    let mut held = Vec::new();
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 2, "on connection, the byte budget paces the previously due tasks");
    driver.send(held.remove(0));
    hold_transcripts(&mut driver, &mut held);
    assert_eq!(held.len(), 2);
    for event in held {
        driver.send(event);
    }
    driver.settle();
    assert_eq!(assigned(&driver), 3);
    assert_eq!(driver.root.held_bytes(), 0);
}

#[test]
fn a_goal_refused_funding_at_a_change_cuts_it_after_its_budget_is_raised() {
    // The run keeps half for delegates; an ordinary delegate below leaves only 2 unreserved.
    const SMALL: u64 = 62;
    // The owner's 500-unit pool can fund 200 and leave room for the change and its producer.
    const WIDER: u64 = 200;
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(WORKER, 2, Box::new([])));
    let ask = walking::goal_from(
        world.chat(1, b"Make a small fix"),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: SMALL, priority: 0, deadline: None },
    );
    let people::Ask::Create { project, batch } = ask else { panic!("goal batch") };
    let mut wider = batch[0].authority.clone();
    wider.spend = WIDER;
    let run = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let row = live_task(&world, run.task);
    let tasks::Party::Person(person) = row.requester else { panic!("person requested goal") };
    assert!(matches!(row.funder, tasks::Funder::Pool { project: funded_project, person: funder, .. }
        if funded_project == project && funder == person));
    assert_eq!(row.authority.budget.spend, SMALL);
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(b"Inspect the small fix", Box::new([]));
    delegate.authority.tools = row.authority.tools;
    delegate.authority.budget.spend =
        row.numbers.budget - row.numbers.reserved - row.numbers.spent - row.numbers.spent_below - 2;
    let answer = world.scripted_call(&run, 93, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let engine::CallAnswer::Delegated(ordinary) = answer else { panic!("ordinary delegate: {answer:?}") };
    assert_eq!(ordinary.len(), 1);
    let row = live_task(&world, run.task);
    assert_eq!(
        (row.numbers.budget - row.numbers.reserved - row.numbers.spent - row.numbers.spent_below) / 2,
        1,
        "the running goal leaves too little for a change"
    );
    let change =
        engine::Tool::Change { changes: Box::new([engine::ChangeCall::words(b"Small fix".as_slice().into(), None)]) };
    let refused = world.scripted_call(&run, 94, change.clone());
    assert!(
        matches!(&refused, engine::CallAnswer::DelegationRefused(problem) if problem.why == tasks::Refusal::Funding),
        "{refused:?}"
    );
    assert!(
        !world.store.rows.values().any(|record| matches!(record,
            Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
                if task.requester == tasks::Party::Task(run.task)
                    && task.executor == tasks::Executor::Procedure { connector: 0, code: 2 }
        )),
        "the refused change creates no change task"
    );
    let before = live_task(&world, run.task).clone();
    let replies = world.web_replies.len();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("goal's person"),
        key: [95; 16],
        ask: people::Ask::Amend {
            project,
            task: run.task,
            revision: before.revisions.spec,
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: Some(wider),
                llm: None,
                reason: b"Fund the goal's next change".as_slice().into(),
            },
        },
    }));
    world.until(Until::Ready);
    assert_eq!(
        &world.web_replies[replies..],
        &[people::Reply::Outcome(people::Outcome::Amended { task: run.task, released: false })]
    );
    let mut expected = before.authority;
    expected.budget.spend = WIDER;
    let raised = live_task(&world, run.task);
    assert_eq!(raised.authority, expected, "only spend changes");
    assert_eq!(raised.revisions.spec, before.revisions.spec + 1);
    assert_eq!(raised.numbers.budget, WIDER);
    let answer = world.scripted_call(&run, 96, change);
    let engine::CallAnswer::Delegated(made) = answer else { panic!("change after raise: {answer:?}") };
    assert_eq!(made.len(), 1);
    let task = live_task(&world, made[0].task);
    assert_eq!(task.requester, tasks::Party::Task(run.task));
    assert_eq!(task.executor, tasks::Executor::Procedure { connector: 0, code: 2 });
}
