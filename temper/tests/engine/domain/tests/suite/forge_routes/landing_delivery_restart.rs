//! Saved delivery beside proof across link loss and restart (temper/docs/design/domain/landing.md, 15–16).
use super::*;
use temper_engine_domain_forge_change::State;

const REPOSITORY: &[u8] = b"org/lib";

fn next_attempt(world: &mut World, producer: &engine::Assignment) -> engine::Assignment {
    for _ in 0..600 {
        world.tick();
        if let Some(run) = world.assigned.iter().find(|run| run.task == producer.task && run.attempt > producer.attempt)
        {
            return run.clone();
        }
    }
    panic!("saved producer retries: {:?}", task_row(world, producer.task));
}

fn change_row(world: &World, change: u64) -> &forge_top::ChangeHot {
    world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::ChangeHot(row) if row.task == change => Some(row.as_ref()),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("change")
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one cut follows a saved producer and an independent proving landing through recovery"
)]
fn a_saved_delivery_survives_worker_loss_and_engine_restart_beside_proof() {
    // The library's workflow latency keeps its first proof in flight through the cut.
    // Both changes belong to this chat and target the same main, with batch one.
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            batch: Some(1),
            across: true,
            library_gate_latency: Some(Duration::from_secs(20)),
            forge_branches: Some(8),
            ..StoryLimits::default()
        },
    );
    world.adopt();
    adopt_library(&mut world);
    world.send(hello_prepare(3));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { unreachable!() };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let (first, producer) = make_change(&mut world, &chat, 95, REPOSITORY);
    push_change(&mut world, &producer, REPOSITORY);
    finish_producer(&mut world, &producer, LIB);
    for _ in 0..600 {
        world.tick();
        if landing_rows(&world).iter().any(|row| {
            row.landing.phase == forge_top::LandingPhase::Proving
                && row.entries.iter().any(|entry| entry.run.is_some_and(|run| !run.ended))
        }) {
            break;
        }
    }
    let first_landing = landing_rows(&world)[0].landing.task;
    let first_run = landing_rows(&world)[0].entries.iter().find_map(|entry| entry.run).expect("proof in flight");
    assert!(!first_run.ended);
    assert_eq!(landing_rows(&world)[0].landing.phase, forge_top::LandingPhase::Proving);

    let (second, producer) = make_change(&mut world, &chat, 96, REPOSITORY);
    let repository = &producer.workspace.repositories[0];
    let tag = repository.tag;
    let saved = repository.save.clone().expect("producer can save");
    let branch = repository.push.clone().expect("delivery branch");
    assert_eq!(repository.landing.as_deref(), Some(b"main".as_slice()));
    let base = world.fake.branch(REPOSITORY, b"main").expect("main before either landing");
    assert_eq!(
        world.external_at(REPOSITORY, 1, raw::Op::Git(raw::Git::Create { branch: saved.clone(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, REPOSITORY, &saved, b"second", b"saved edit", 1)
        .expect("producer saves its edit");
    let answer = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        1,
        tasks::End::Failed(tasks::Class::Run),
        engine::Parts { saved: Some(Box::new([tag])), pushed: Box::new([]), preparation: None },
    );
    world.send(answer);
    let retry = next_attempt(&mut world, &producer);
    assert_eq!(retry.saved.as_ref(), [tag]);
    assert!(matches!(&retry.workspace.repositories[0].start, engine::ForgeStart::Saved(name) if *name == saved));
    assert_eq!(
        world.external_at(REPOSITORY, 1, raw::Op::Git(raw::Git::PushNew { branch: branch.clone(), commit: head })),
        raw::Answer::PushNew { existing: None }
    );

    // The push is on the forge; neither the retry's terminal nor its Parts::pushed has reached the engine.
    world.send(protocol::lost(Token::new(7)));
    world.drain_cut();
    assert_eq!(landing_rows(&world)[0].landing.phase, forge_top::LandingPhase::Proving);
    assert!(!ended(&world, retry.task));
    world.restart_same();
    // restart_same normally relinks every hosted assignment. This worker lost its producer,
    // so only the chat is still hosted when it links again.
    world.events.retain(|event| {
        !matches!(event, engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { .. }, .. }))
    });
    world.forge_reads = Some(Vec::new());
    world.send(protocol::hello_slots(
        Token::new(7),
        3,
        Box::new([protocol::hosting(chat.task, chat.attempt, jig_domain_link::Phase::Active)]),
    ));
    let resumed = next_attempt(&mut world, &retry);
    assert_eq!(resumed.saved.as_ref(), [tag]);
    assert!(matches!(&resumed.workspace.repositories[0].start, engine::ForgeStart::Saved(name) if *name == saved));
    assert_eq!(resumed.workspace.repositories[0].push.as_deref(), Some(branch.as_ref()));
    assert_eq!(world.fake.branch(REPOSITORY, &branch), Some(head));
    assert_eq!(world.fake.branch(REPOSITORY, &saved), Some(head));
    let recovered =
        landing_rows(&world).into_iter().find(|row| row.landing.task == first_landing).expect("restored landing");
    assert_eq!(recovered.landing.phase, forge_top::LandingPhase::Proving);
    assert!(recovered.entries.iter().any(|entry| entry.run == Some(first_run)), "same in-flight run found");
    assert!(
        world.forge_reads.as_ref().expect("reads").iter().any(|(_, read)| matches!(read,
        api::Read::Run { run, .. } if *run == first_run.id)),
        "restart rereads the proving run"
    );
    assert_eq!(world.merges, 0);

    let answer = world.protocol.answer(
        Token::new(7),
        resumed.task,
        resumed.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Saved edit".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: tag,
                words: b"saved edit delivered".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts {
            saved: None,
            pushed: Box::new([forge_top::Pushed { tag, commit: translate::commit(head), checked: true }]),
            preparation: None,
        },
    );
    world.send(answer);
    let mut ready = false;
    for _ in 0..PAIR_TICKS {
        world.tick();
        let second_row = change_row(&world, second);
        ready |= matches!(second_row.change.state, State::Ready { head: delivered, .. } if delivered == translate::commit(head));
        assert!(
            !matches!(second_row.change.state, State::Held { .. }),
            "producer delivery is not drift: {second_row:?}"
        );
        for task in [first, second, first_landing] {
            assert!(!matches!(task_row(&world, task).expect("task").phase, tasks::Phase::Held { .. }));
        }
        if !ended(&world, first) {
            assert_eq!(landing_rows(&world).len(), 1, "second waits for the first landing");
        }
        if ended(&world, first)
            && ended(&world, second)
            && landing_rows(&world).iter().all(|row| ended(&world, row.landing.task))
        {
            break;
        }
    }
    assert!(ready, "resumed head becomes ready");
    assert!(ended(&world, first) && ended(&world, second));
    assert!(ended(&world, resumed.task), "the resumed producer's terminal is committed");
    let rows = landing_rows(&world);
    assert_eq!(rows.len(), 2, "one landing for each change");
    for change in [first, second] {
        let row = rows
            .iter()
            .find(|row| row.landing.members.iter().any(|member| member.change == change))
            .expect("own landing");
        assert_eq!(row.landing.members.len(), 1, "batch one");
        assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
        assert_eq!(row.landing.members[0].left, None);
        assert!(ended(&world, row.landing.task));
        let candidate = row.landing.members[0].candidate.expect("candidate");
        assert_eq!(
            world
                .writes
                .iter()
                .filter(|write| matches!(write,
            api::Write::Dispatch { branch, .. } if *branch == row.landing.members[0].branch))
                .count(),
            1
        );
        assert_eq!(
            world
                .writes
                .iter()
                .filter(|write| matches!(write,
            api::Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } if *head == candidate))
                .count(),
            1
        );
    }
    let first_member =
        &rows.iter().find(|row| row.landing.task == first_landing).expect("first landing").landing.members[0];
    let second_member =
        &rows.iter().find(|row| row.landing.members[0].change == second).expect("second landing").landing.members[0];
    assert_eq!(second_member.base, first_member.landed, "second prepares above the first landed prefix");
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 2);
    assert_eq!(world.merges, 2);
    for (index, write) in world.repository_writes.iter().enumerate() {
        assert!(!world.repository_writes[..index].contains(write), "no forge write made twice: {write:?}");
    }
    let main = world.fake.branch(REPOSITORY, b"main").expect("landed main");
    let tree = &world.fake.object(main).expect("main object").tree;
    assert_eq!(tree.get(b"file".as_slice()).map(AsRef::as_ref), Some(b"fixed".as_slice()));
    assert_eq!(tree.get(b"second".as_slice()).map(AsRef::as_ref), Some(b"saved edit".as_slice()));
    assert_eq!(
        rows.iter().find(|row| row.landing.members[0].change == second).expect("second landing").landing.members[0]
            .head,
        Some(translate::commit(head)),
        "retry lands the saved delivery without another edit"
    );
}
