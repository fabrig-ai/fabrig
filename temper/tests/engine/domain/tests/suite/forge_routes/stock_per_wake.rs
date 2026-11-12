//! The stock's parts selected by jig from a durable wake snapshot (domain/tasks/messages.md, 6).
use super::*;

fn worker(model: &engine::Model, event: &mut engine::Event) {
    if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = event {
        hello.bounds = jig_test_profile::fixtures::bounds();
        hello.bounds.time = Duration::from_secs(28_800);
        hello.bounds.endpoints = Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    }
}
fn stock_goal() -> (World, engine::Assignment, jig_domain_core::Charter) {
    let mut model = stock::box_model();
    model.input_price = 1; model.cached_price = 1; model.cache_write_price = None;
    model.output_price = 1; model.price_unit = 1000;
    let limits = temper_engine_domain_world::forge_routes::box_limits();
    let defaults = defaults(&model, limits.tasks.result_bytes);
    let charter = defaults.charters.iter().find(|charter| charter.number == CHARTER).unwrap().clone();
    let template = defaults.templates.iter().find(|template| template.number == TEMPLATE).unwrap().clone();
    let mut world = World::configured_stock_limits(&defaults, &model, &limits);
    world.adopt();
    let mut hello = protocol::hello_slots(CHANNEL, 3, Box::new([]));
    worker(&model, &mut hello); world.send(hello);
    let ask = request(&world, &template);
    let first = assigned_chat(&mut world, GOAL_REQUEST, ask);
    (world, first, charter)
}
fn selected(charter: &jig_domain_core::Charter, indices: &[usize]) -> Vec<u8> {
    let mut text = charter.policy.instructions.to_vec();
    for index in indices { text.extend_from_slice(b"\n\n"); text.extend_from_slice(&charter.parts[*index].instructions); }
    text
}
fn wake(world: &mut World, previous: &engine::Assignment, kinds: &[tasks::MessageKind]) -> engine::Assignment {
    // A live delegate keeps the closer from ending the parked selection fixture.
    let _delegate = delegate(world, previous);
    world.scripted_read_inbox(previous);
    let turns = live_task(world, previous.task).turn;
    park_after_turn(world, previous, None, turns);
    world.until(Until::Ready);
    world.drain_cut();
    let mut row = live_task(world, previous.task).clone();
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(row.reasons, tasks::Reasons::EMPTY);
    let mut header = world.store.header();
    row.inbox = kinds.iter().map(|kind| {
        header.messages += 1;
        tasks::Word { number: header.messages, from: tasks::Party::Task(previous.task), kind: kind.clone(), words: b"wake".as_slice().into(), at: world.env.wall, hits: 1, eligible: true }
    }).collect();
    row.last_message = header.messages;
    row.phase = tasks::Phase::Active(tasks::Active::Due);
    header.commits += 1;
    let writes = Box::new([
        engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(row))))),
        engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header)))),
    ]);
    world.store.pending.push_back((header.commits, writes)); world.store.apply();
    world.restart_same();
    for event in &mut world.events { worker(&previous.run.policy.model, event); }
    for _ in 0..1000 {
        world.tick();
        if let Some(run) = world.assigned.iter().rev().find(|run| run.task == previous.task && run.attempt > previous.attempt) { return run.clone(); }
    }
    panic!("stock goal not assigned: {:?}", live_task(world, previous.task));
}
#[test]
fn the_stock_goal_reads_only_the_parts_for_its_wake() {
    for (name, kinds, indices) in [
        ("question", vec![tasks::MessageKind::Question], vec![1]),
        ("red news", vec![tasks::MessageKind::News { subscription: 1, class: tasks::NewsClass::Wakes }], vec![3]),
        ("question and result", vec![tasks::MessageKind::Question, tasks::MessageKind::Result(tasks::ResultKind::Failed)], vec![1, 2]),
        ("answer", vec![tasks::MessageKind::Answer { question: 1 }], vec![]),
    ] {
        let (mut world, first, charter) = stock_goal();
        assert!(first.run.policy.instructions.as_ref() == selected(&charter, &[0, 1, 2]), "designer reads common, goal, designer, questions, results");
        let next = wake(&mut world, &first, &kinds);
        assert!(next.run.policy.instructions.as_ref() == selected(&charter, &indices), "{name}: the selected whole instructions differ");
        assert_eq!(next.messages.iter().map(|word| word.kind.clone()).collect::<Vec<_>>(), kinds, "{name}: messages reach the run");
        world.close();
    }
}
