//! Changes follow their landing branch's rules (temper/docs/design/domain/forge.md, 8.2 and 8.7).
use super::*;
use temper_engine_domain_forge_change as change_step;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 200;

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn pushed_change(conflict: bool) -> (World, u64, u64, u64) {
    let mut world = World::configured_ci(true, true, true).with_landing_deployment(Box::new([engine::LandingRule {
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: authority::Last::Open(Box::new([])),
        },
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) = change_tool::change(&mut world, &chat, 93, None, b"Small fix") else {
        panic!("change delegated")
    };
    let task = made[0].task;
    let producer = super::reviews::next_assignment(&mut world, 1);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(task));
    let branch = change_row(&world, task).branch.clone();
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"producer", 2)
        .expect("producer pushed");
    let path: &[u8] = if conflict { b"file" } else { b"other" };
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", path, b"base moved", 1)
        .expect("main advanced after the producer pushed");
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state: raw::Check::Pending,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    (world, task, head, base)
}

fn assert_no_update_or_delegate(world: &World, task: u64) {
    let row = change_row(world, task);
    assert_eq!((row.change.updates, row.change.repairs, row.change.resolutions), (0, 0, 0));
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    let delegates = world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
        if row.requester == tasks::Party::Task(task))
        })
        .count();
    assert_eq!(delegates, 1, "only the producer, with no repair or resolution delegate");
}

fn assert_pending_and_stale(world: &mut World, head: u64, base: u64) -> raw::Pull {
    let raw::Answer::Comparison { contains_base, .. } =
        world.external(1, raw::Op::Read(raw::Read::Compare { base, head, page: 1, limit: 10 }))
    else {
        panic!("fake compares the producer's head with the advanced base")
    };
    assert!(!contains_base, "the producer's head never took main's new tip");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: 1 })) else {
        panic!("fake pull")
    };
    assert_eq!(pull.commit, head, "the pull keeps exactly the producer's head");
    assert!(
        pull.statuses.iter().any(|status| status.context.as_ref() == b"build" && status.state == raw::Check::Pending)
    );
    pull
}

#[test]
fn a_change_under_rules_asking_neither_merges_at_its_head() {
    let (mut world, task, head, base) = pushed_change(false);
    for _ in 0..STEPS {
        world.tick();
        if matches!(change_row(&world, task).change.state, change_step::State::Landed { .. }) {
            break;
        }
    }
    assert!(matches!(change_row(&world, task).change.state, change_step::State::Landed { .. }));
    assert_eq!(world.merges, 1);
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { head: merged, .. } if *merged == translate::commit(head))));
    assert_no_update_or_delegate(&world, task);
    let pull = assert_pending_and_stale(&mut world, head, base);
    assert!(pull.merged.is_some());
    world.close();
}

#[test]
fn a_conflict_under_rules_asking_neither_delegates_resolution() {
    let (mut world, task, head, base) = pushed_change(true);
    // A persistent conflict is decided after the shipped mergeability window,
    // with the world's twenty-second durable round trip left for resolution.
    let deadline =
        world.env.now.saturating_add(world.env.limits.forge.client.settle).saturating_add(Duration::from_secs(20));
    while world.env.now <= deadline {
        world.tick();
        if matches!(change_row(&world, task).delegate, Some((_, change_step::Delegate::Resolve { .. }))) {
            break;
        }
    }
    let resolution = super::delegates::fixtures::next_delegate(&mut world);
    let row = change_row(&world, task);
    let expected_base = translate::commit(base);
    assert_eq!(row.change.state, change_step::State::Resolving { requested: true, base: expected_base });
    assert_eq!(row.delegate, Some((resolution.task, change_step::Delegate::Resolve { base: expected_base })));
    assert_eq!(live_task(&world, resolution.task).requester, tasks::Party::Task(task));
    assert_eq!((row.change.updates, row.change.repairs, row.change.resolutions), (0, 0, 1));
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    assert_eq!(world.merges, 0);
    let pull = assert_pending_and_stale(&mut world, head, base);
    assert!(!pull.checking, "the forge has settled mergeability");
    assert!(!pull.mergeable);
    assert!(pull.merged.is_none());
    world.close();
}
