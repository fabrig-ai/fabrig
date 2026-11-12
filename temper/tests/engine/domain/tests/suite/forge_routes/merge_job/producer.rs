//! A conflict waits for the producer's terminal (domain/forge/change/repairs.md).
use super::*;

const BRANCH: &[u8] = b"temper/handed";

fn live_producer(world: World) -> (World, u64, engine::Assignment, u64, u64) {
    live_producer_with_route(world, false)
}

fn live_producer_with_route(mut world: World, questions_up: bool) -> (World, u64, engine::Assignment, u64, u64) {
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 3, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(b"Small fix", Box::new([]));
    delegate.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 128 };
    delegate.authority = live_task(&world, chat.task).authority.clone();
    delegate.authority.budget.spend = 40;
    delegate.authority.delegation.tasks = 6;
    delegate.authority.delegation.depth = 1;
    let mut parameters = vec![
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
        tasks::Parameter::Bytes { name: 3, value: BRANCH.into() },
        tasks::Parameter::Bytes { name: 10, value: b"Small fix".as_slice().into() },
    ];
    if questions_up {
        // This route leaves half the producer's funding for concurrent delegates.
        parameters.push(tasks::Parameter::Number { name: 14, value: 1 });
    }
    delegate.spec.parameters = parameters.into_boxed_slice();
    let event = world.protocol.call(
        CHANNEL,
        chat.task,
        chat.attempt,
        93,
        engine::Call { completion: 1, position: 1, tool: engine::Tool::Delegate { batch: Box::new([delegate]) } },
    );
    world.send(event);
    let producer = super::super::reviews::next_assignment(&mut world, 1);
    let tasks::Party::Task(task) = live_task(&world, producer.task).requester else { panic!("change's producer") };
    world.external(2, raw::Op::Git(raw::Git::Create { branch: BRANCH.into(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", BRANCH, b"file", b"producer", 2)
        .expect("producer pushes H1");
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base moved", 1)
        .expect("conflicting base");
    passed_ci(&mut world, head);
    passed_ci(&mut world, base);
    // A reported push frees the writer, but parking does not end the producer.
    let event = world.protocol.call(
        CHANNEL,
        producer.task,
        producer.attempt,
        94,
        engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::Subscribe {
                kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
            },
        },
    );
    world.send(event);
    world.until(Until::Ready);
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        0,
        tasks::End::Parked,
        engine::Parts {
            saved: None,
            pushed: Box::new([forge_top::Pushed {
                tag: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                commit: translate::commit(head),
            }]),
            preparation: None,
        },
    );
    world.send(event);
    world.until(Until::Ready);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    (world, task, producer, head, base)
}

fn wait_for_conflict(world: &mut World, task: u64, base: u64, assignments: usize) {
    let deadline = world
        .env
        .now
        .saturating_add(world.env.limits.forge.client.settle)
        .saturating_add(world.env.limits.forge.change_policy.stall)
        .saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline {
        world.tick();
    }
    let row = change_row(world, task);
    assert!(world.jobs.is_empty(), "no merge job while the producer lives");
    assert_eq!(world.assigned.len(), assignments, "no resolution while the producer lives");
    assert!(!matches!(row.change.state, change::State::Held { .. }), "{row:?}");
    assert_eq!(row.change.resolutions, 0);
    assert_eq!(
        row.change.state,
        change::State::WaitingProducer { base: translate::commit(base) },
        "{row:?}; phase {:?}; writes {:?}",
        live_task(world, task).phase,
        world.writes
    );
}

fn resume(world: &mut World, producer: &engine::Assignment) -> engine::Assignment {
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("person"),
        key: [95; 16],
        ask: people::Ask::Say { project: 1, task: producer.task, words: b"Finish".as_slice().into() },
    }));
    super::super::reviews::next_assignment(world, 2)
}

#[test]
fn a_conflict_waits_for_the_producers_last_push_and_terminal() {
    let (mut world, task, producer, _, base) = live_producer(world(false, true, true));
    wait_for_conflict(&mut world, task, base, 2);
    let producer = resume(&mut world, &producer);
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", BRANCH, b"file", b"producer H2", 2)
        .expect("producer pushes H2 after conflict");
    world.forge_reads = Some(Vec::new());
    finish_resolution(&mut world, &producer);
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline && world.assigned.len() < 4 {
        world.tick();
    }
    let resolution =
        world.assigned.get(3).unwrap_or_else(|| panic!("resolution: {:?}", change_row(&world, task))).clone();
    assert_eq!(change_row(&world, task).change.last_head, Some(translate::commit(head)));
    assert_preserving_resolution(&live_task(&world, resolution.task).spec.words);
    assert_eq!(
        resolution.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: BRANCH.into(), base: translate::commit(base) }
    );
    let pull = resolution
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull))
        .expect("pull section");
    let engine::BriefBody::Text(words) = &pull.body else { panic!("readable pull section") };
    let head_text = forge_top::commit_text(&translate::commit(head));
    assert!(
        words.windows(head_text.len()).any(|part| part == head_text.as_ref()),
        "{}",
        String::from_utf8_lossy(words)
    );
    let reads = world.forge_reads.as_ref().expect("recorded reads");
    assert_eq!(
        reads
            .iter()
            .filter(|(_, read)| matches!(read, skein_forge::api::Read::Item { number: 1, after: 0, .. }))
            .count(),
        1,
        "the brief was gathered once, not retried for a moved head"
    );
    assert_eq!(reads.iter().filter(|(_, read)| matches!(read, skein_forge::api::Read::PullFiles { head: pinned, page: 1, .. } if *pinned == translate::commit(head))).count(), 1);
    let merged = push_merge(&mut world, task, head, base);
    finish_resolution(&mut world, &resolution);
    landed(&mut world, task, merged);
    world.close();
}

#[test]
fn a_turn_preparation_waits_for_its_producers_final_head() {
    let configured = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, ..StoryLimits::default() },
    );
    let (mut world, task, producer, _, base) = live_producer(configured);
    wait_for_conflict(&mut world, task, base, 2);
    assert_eq!(change_row(&world, task).change.way, change::Way::Turn);
    let producer = resume(&mut world, &producer);
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", BRANCH, b"file", b"producer H2", 2)
        .expect("producer pushes H2");
    finish_resolution(&mut world, &producer);
    let resolution = super::super::reviews::next_assignment(&mut world, 3);
    assert!(
        matches!(change_row(&world, task).delegate, Some((child, change::Delegate::Resolve { .. })) if child == resolution.task)
    );
    assert_eq!(change_row(&world, task).change.last_head, Some(translate::commit(head)));
    assert_eq!(
        resolution.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: BRANCH.into(), base: translate::commit(base) }
    );
    assert_eq!(change_row(&world, task).change.resolutions, 1);
    let merged = push_merge(&mut world, task, head, base);
    passed_ci(&mut world, merged);
    finish_resolution(&mut world, &resolution);
    for _ in 0..STEPS {
        world.tick();
        if matches!(change_row(&world, task).change.state, change::State::Ready { .. }) {
            break;
        }
    }
    assert!(
        matches!(change_row(&world, task).change.state, change::State::Ready { head, .. } if head == translate::commit(merged))
    );
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline && !matches!(change_row(&world, task).change.state, change::State::Landed { .. }) {
        world.tick();
    }
    assert!(
        matches!(change_row(&world, task).change.state, change::State::Landed { .. }),
        "{:?}",
        change_row(&world, task)
    );
    world.close();
}

#[test]
fn a_merge_job_waits_for_the_producers_final_head() {
    let (mut world, task, producer, _, base) = live_producer(world(true, true, true).with_merge_job(COMMAND));
    wait_for_conflict(&mut world, task, base, 2);
    let producer = resume(&mut world, &producer);
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", BRANCH, b"file", b"producer H2", 2)
        .expect("producer pushes H2");
    finish_resolution(&mut world, &producer);
    let job = next_job(&mut world, task, false);
    check_job(&world, task, &job, head, base);
    assert_eq!(change_row(&world, task).change.last_head, Some(translate::commit(head)));
    let merged = push_merge(&mut world, task, head, base);
    passed_ci(&mut world, merged);
    finish_job(&mut world, &job, RESULT);
    landed(&mut world, task, merged);
    assert_eq!(change_row(&world, task).result.head, Some(translate::commit(merged)));
    world.close();
}

#[test]
fn a_producer_that_cleans_the_conflict_needs_no_resolution() {
    let (mut world, task, producer, first, base) = live_producer(world(true, true, true));
    wait_for_conflict(&mut world, task, base, 2);
    let producer = resume(&mut world, &producer);
    let head = push_merge(&mut world, task, first, base);
    finish_resolution(&mut world, &producer);
    for _ in 0..STEPS {
        world.tick();
    }
    let row = change_row(&world, task);
    assert_eq!(row.change.resolutions, 0);
    assert_eq!(row.change.last_head, Some(translate::commit(head)));
    assert_eq!(row.change.state, change::State::Checking { head: translate::commit(head) });
    assert_eq!(row.result.head, Some(translate::commit(head)), "the final push is checked afresh");
    assert!(world.jobs.is_empty());
    assert_eq!(world.assigned.len(), 3, "no resolution");
    passed_ci(&mut world, head);
    landed(&mut world, task, head);
    world.close();
}

#[test]
fn a_producer_displaced_by_a_gate_still_keeps_the_conflict_waiting() {
    let configured = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate { agent: true, ..Gate::default() },
        None,
        StoryLimits::default(),
    );
    let (mut world, task, producer, head, base) = live_producer_with_route(configured, true);
    let gate = super::super::reviews::next_assignment(&mut world, 2);
    assert!(
        matches!(change_row(&world, task).delegate, Some((child, change::Delegate::Gate { .. })) if child == gate.task)
    );
    super::super::reviews::finish_review(
        &mut world,
        &gate,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    wait_for_conflict(&mut world, task, base, 3);
    assert_eq!(change_row(&world, task).delegate, None, "the producer is no longer the current delegate");
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(951)),
        sign_in: world.signed_in.expect("person"),
        key: [96; 16],
        ask: people::Ask::Cancel {
            project: 1,
            task: producer.task,
            reason: b"finish pushing".as_slice().into(),
            grace: None,
        },
    }));
    let resolution = super::super::reviews::next_assignment(&mut world, 3);
    assert!(
        matches!(change_row(&world, task).delegate, Some((child, change::Delegate::Resolve { .. })) if child == resolution.task)
    );
    assert_eq!(change_row(&world, task).change.last_head, Some(translate::commit(head)));
    assert_eq!(change_row(&world, task).change.resolutions, 1);
    world.close();
}
