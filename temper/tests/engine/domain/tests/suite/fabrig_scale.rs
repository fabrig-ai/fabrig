//! Stock funding and lifetime capacity under init's effective limits
//! (`temper/docs/design/protocol/configuration/stock.md`;
//! `temper/docs/design/testing/README.md`, 1).
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_link::EndpointBound;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, List, ReplyTo, Token};
use temper_engine_domain::{self as engine, Key, Record};
use temper_engine_domain_world::{
    forge_routes::{Until, World, box_limits},
    oracle, protocol,
};

const PROJECT: u32 = 1;
const CHAT: u32 = 1;
const GOAL: u32 = 2;
const STEPS: u32 = 200; // The forge world's durable round-trip bound.
const CHATS: u8 = 80; // protocol/configuration/stock.md: eighty simultaneous 24K allotments.
const K: u64 = 8_720; // Sol: C = ceil((1,048,576 × 2,000 + 8,192 × 10,000) / 1,000,000), K = 4C.
const CHAT_SPEND: u64 = 24 * K;
const POOL_SPEND: u64 = CHATS as u64 * CHAT_SPEND;
const WORKER: Token = Token::new(7);
const TREE_TASKS: u32 = 384; // protocol/configuration/stock.md: lifetime tasks, including the goal.
const LEAVES: [u32; 4] = [95, 95, 95, 94]; // Goal + four coordinators + 379 reports = 384.

fn stock_world() -> (World, core::settings::Defaults) {
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        providers: Box::new([]),
        models: (Box::new([super::stock::box_model()]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),
    };
    let limits = box_limits();
    let stock = temper_engine_service_stock::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
    let mut world = World::configured_stock_limits(&stock, &llm.models[0].model, &limits);
    world.sign_in();
    (world, stock)
}

fn from_template(world: &World, stock: &core::settings::Defaults, number: u32) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Work at fabrig scale") else {
        panic!("one root task")
    };
    let template = stock.templates.iter().find(|template| template.number == number).expect("stock template");
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent {
        charter: match template.executor {
            people::TemplateExecutor::Charter(number) => number,
            people::TemplateExecutor::Chosen => CHAT,
        },
    };
    let tasks::Contract::Report { words } = template.contract else { panic!("stock report contract") };
    member.contract = people::Contract::Report { words };
    member.budget = match template.budget {
        people::TemplateBudget::AsAllowed { cap } => people::Budget::AsAllowed { cap },
        people::TemplateBudget::Given => {
            member.authority.spend = CHAT_SPEND;
            people::Budget::Named
        }
        people::TemplateBudget::Amount(_) => panic!("stock chat or goal budget"),
    };
    member.deadline = people::Deadline::Named;
    member.flags = people::Flags {
        conversation: template.flags.conversation,
        listed: template.flags.listed,
        retention: people::Retention::Kept,
        ended_by: match template.flags.ended_by {
            tasks::EndedBy::Requester => people::EndedBy::Requester,
            tasks::EndedBy::Standing => people::EndedBy::Standing,
            tasks::EndedBy::Closer(closer) => {
                people::EndedBy::Closer(people::Closer { connector: closer.connector, code: closer.code })
            }
            tasks::EndedBy::Result => people::EndedBy::Result,
        },
    };
    member.label = template.label;
    member.tracked = template.tracked;
    member.llm = None;
    people::Ask::Create { project, batch }
}

fn ask(world: &mut World, key: u8, ask: people::Ask) -> people::Outcome {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask,
    }));
    for _ in 0..STEPS {
        world.tick();
        if let Some(outcome) = world.store.rows.values().find_map(|row| match row {
            Record::Core(core::Record::People(people::Stored::Answer { key: request, outcome, .. }))
                if request.key == [key; 16] =>
            {
                Some(outcome.clone())
            }
            Record::Core(_) | Record::Forge { .. } => None,
        }) {
            return outcome;
        }
    }
    panic!("owner request {key} did not answer")
}

fn live(world: &World, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(core::Key::Tasks(tasks::Key::Live(task))))
    else {
        panic!("live task {task}")
    };
    row
}

fn pool(world: &World, funder: tasks::Funder) -> &tasks::FundingRecord {
    let Some(Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))) =
        world.store.rows.get(&Key::Core(core::Key::Tasks(tasks::Key::Ledger(funder))))
    else {
        panic!("owner's pool ledger")
    };
    row
}

fn ended(world: &mut World, task: u64) {
    let key = Key::Core(core::Key::Tasks(tasks::Key::Ended(task)));
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.contains_key(&key) && world.store.pending.is_empty() {
            return;
        }
    }
    panic!("task {task} ended")
}

fn close(world: &mut World) {
    world.until(Until::Ready);
    let mut snapshot = oracle::Snapshot::new(&world.store.rows, world.env.wall);
    for (_, read) in &snapshot.queries {
        snapshot.answers.push(world.party_read(read.clone()));
    }
    world.until(Until::Ready);
    let forges = world.root.forges();
    world.store.crash(0);
    snapshot.compare_rows(&world.store.rows);
    // Restore performs work per retained row. The small world's fixed 200 steps
    // cannot scan a scale story; give each row the existing round-trip bound.
    let steps = STEPS * u32::try_from(world.store.rows.len()).expect("bounded retained rows");
    world.assigned.clear(); // Every hosted task has ended before the restart.
    world.restart_same();
    for _ in 0..steps {
        world.tick();
        if world.root.quiescent() && world.events.is_empty() && world.store.idle() {
            break;
        }
    }
    assert!(world.root.quiescent(), "scale restart drains its retained rows");
    snapshot.compare_rows(&world.store.rows);
    oracle::compare_forges(forges, world.root.forges());
    for (index, (_, read)) in snapshot.queries.iter().enumerate() {
        snapshot.compare_answer(index, world.party_read(read.clone()));
    }
    world.until(Until::Ready);
}

fn worker(world: &mut World, stock: &core::settings::Defaults) {
    let model = super::stock::box_model();
    let mut bounds = protocol::bounds();
    bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    bounds.turns = 800;
    bounds.time = Duration::from_secs(28_800);
    bounds.spend = stock.rules.maximum_run_spend;
    bounds.message_bytes = world.env.limits.tasks.message_bytes;
    bounds.inbox = world.env.limits.tasks.run_inbox;
    world.send(engine::Event::Worker(engine::WorkerInput::Link {
        channel: WORKER,
        link: engine::Link::Hello {
            worker: 0,
            hello: jig_domain_link::Hello {
                jobs: Box::new([]),
                slots: world.env.limits.fleet.slots,
                stop_bound: Duration::from_millis(100),
                workstreams: Box::new([]),
                hosting: Box::new([]),
                bounds,
            },
        },
    }));
}

fn assignment(world: &mut World, task: u64, after: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        if let Some(run) = world.assigned.iter().rev().find(|run| run.task == task && run.attempt > after) {
            return run.clone();
        }
        world.tick();
    }
    panic!("task {task} assigned: {:?}", live(world, task).phase)
}

fn finish(world: &mut World, run: &engine::Assignment) {
    let event = world.protocol.answer(
        WORKER,
        run.task,
        run.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    ended(world, run.task);
}

fn park(world: &mut World, run: &engine::Assignment) {
    let event = world.protocol.answer(WORKER, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if live(world, run.task).phase == tasks::Phase::Active(tasks::Active::Idle) && world.store.pending.is_empty() {
            return;
        }
    }
    panic!("goal parks while its coordinator runs")
}

fn delegate(world: &World, goal: u64) -> engine::Delegate {
    let mut member = temper_engine_domain_world::fixtures::report_delegate(b"One tree member", Box::new([]));
    member.authority.tools = tasks::Tools(0);
    member.authority.models = live(world, goal).authority.models;
    member.authority.budget.spend = K;
    member
}

#[test]
fn eighty_chats_are_funded_at_once() {
    let (mut world, stock) = stock_world();
    let mut chats = Vec::new();
    for key in 1..=CHATS {
        let request = from_template(&world, &stock, CHAT);
        let outcome = ask(&mut world, key, request);
        let people::Outcome::Created { first, last, given } = outcome else {
            panic!("chat {key} admitted: {outcome:?}")
        };
        assert_eq!(first, last);
        assert_eq!(given, Some(people::Given { task: first, amount: CHAT_SPEND }));
        chats.push(first);
    }
    let funder = live(&world, chats[0]).funder;
    assert!(matches!(funder, tasks::Funder::Pool { project: PROJECT, .. }));
    for task in &chats {
        let row = live(&world, *task);
        assert_eq!(row.project, PROJECT);
        assert_eq!(row.executor, tasks::Executor::Agent { charter: CHAT });
        assert_eq!(row.funder, funder);
        assert_eq!(row.numbers.budget, CHAT_SPEND);
        assert_eq!(row.numbers.spent, 0);
    }
    assert_eq!(pool(&world, funder).numbers.budget, POOL_SPEND);
    assert_eq!(pool(&world, funder).numbers.reserved, POOL_SPEND);
    let request = from_template(&world, &stock, CHAT);
    let refused = ask(&mut world, CHATS + 1, request);
    assert!(
        matches!(
            refused,
            people::Outcome::Refused(people::Refusal::Authority { lacking: people::Lack::Budget { left: 0, .. }, .. })
        ),
        "the eighty-first chat exceeds its owner's pool: {refused:?}"
    );
    assert_eq!(pool(&world, funder).numbers.reserved, POOL_SPEND);
    for task in &chats {
        let _ = live(&world, *task);
    }
    // End the proved live set before the world's bounded restart oracle.
    let mut operations = List::with_capacity(u32::from(CHATS));
    for task in &chats {
        operations
            .push(people::Operation::Cancel { task: *task, reason: b"Story complete".as_slice().into(), grace: None })
            .expect("eighty cleanup operations");
    }
    let _ = ask(&mut world, CHATS + 2, people::Ask::Compound { project: PROJECT, operations });
    for task in chats {
        ended(&mut world, task);
    }
    close(&mut world);
}

#[test]
fn a_goal_tree_takes_384_tasks() {
    let (mut world, stock) = stock_world();
    worker(&mut world, &stock);
    let request = from_template(&world, &stock, GOAL);
    let outcome = ask(&mut world, 1, request);
    let people::Outcome::Created { first: goal, last, .. } = outcome else {
        panic!("stock goal admitted: {outcome:?}")
    };
    assert_eq!(goal, last);
    let mut run = assignment(&mut world, goal, 0);
    // Each coordinator is funded for batches of four, independent of the profile's batch bound.
    let batch_size =
        world.env.limits.authority.batch.min(world.env.limits.tasks.batch).min(world.env.limits.tasks.delegates).min(4);
    let mut call = 1000;
    for leaves in LEAVES {
        let mut member = delegate(&world, goal);
        member.authority.tools = tasks::Tools(core::ToolFamilies::JIG.delegate.0);
        member.authority.delegation =
            tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(CHAT)]), tasks: leaves, depth: 1 };
        // One K run plus four K reports fund a batch of four.
        member.authority.budget.spend = 5 * K;
        let answer = world.scripted_call(&run, call, engine::Tool::Delegate { batch: Box::new([member]) });
        let engine::CallAnswer::Delegated(coordinators) = answer else {
            panic!("goal admits its coordinator: {answer:?}")
        };
        call += 1;
        let coordinator = assignment(&mut world, coordinators[0].task, 0);
        park(&mut world, &run); // The other configured slot runs the coordinator's delegates.
        let mut left = leaves;
        while left != 0 {
            let count = batch_size.min(left);
            let batch = vec![delegate(&world, coordinator.task); usize::try_from(count).expect("bounded batch")].into();
            let answer = world.scripted_call(&coordinator, call, engine::Tool::Delegate { batch });
            let engine::CallAnswer::Delegated(children) = answer else {
                panic!("coordinator with {left} reports left admits {count}: {answer:?}")
            };
            assert_eq!(children.len(), usize::try_from(count).expect("bounded batch"));
            assert_eq!(live(&world, coordinator.task).delegates.len(), children.len());
            for child in &children {
                assert_eq!(live(&world, child.task).root, goal);
                let child_run = assignment(&mut world, child.task, 0);
                finish(&mut world, &child_run);
            }
            assert!(live(&world, coordinator.task).delegates.is_empty());
            left -= count;
            assert_eq!(live(&world, coordinator.task).made, 1 + leaves - left);
            call += 1;
        }
        finish(&mut world, &coordinator);
        run = assignment(&mut world, goal, run.attempt);
        assert!(live(&world, goal).delegates.is_empty());
    }
    let ended_tasks = world
        .store
        .rows
        .values()
        .filter(|row| {
            matches!(row,
                Record::Core(core::Record::Tasks(tasks::Stored::Ended(task))) if task.root == goal
            )
        })
        .count();
    assert_eq!(ended_tasks, usize::try_from(TREE_TASKS - 1).expect("tree descendants"));
    assert_eq!(live(&world, goal).made, TREE_TASKS);
    let batch = Box::new([delegate(&world, goal)]);
    let refused = world.scripted_call(&run, call, engine::Tool::Delegate { batch });
    let engine::CallAnswer::DelegationDenied { answer, findings } = refused else {
        panic!("one past the tree's delegation capacity: {refused:?}")
    };
    assert_eq!(answer, authority::Answer::Propose);
    assert_eq!(findings.as_ref(), &[authority::Finding::Tasks { source: authority::Source::Task }]);
    assert_eq!(live(&world, goal).made, TREE_TASKS);
    finish(&mut world, &run);
    close(&mut world);
}
