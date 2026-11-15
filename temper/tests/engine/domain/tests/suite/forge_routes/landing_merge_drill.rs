//! A configured merge job settles a turn member (temper/docs/design/domain/landing.md, 3, 5.2).
use super::*;

fn hello() -> engine::Event {
    let mut event = hello_prepare(8);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.bounds.message_bytes = 256;
    hello.jobs = Box::new([hello.jobs[0].clone(), protocol::merge_bound()]);
    event
}

fn prepare_three(world: &mut World, job: &host::JobAssignment, conflicted: u64) {
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&job.spec),
    )
    .expect("prepare spec");
    assert_eq!(spec.command(), b"");
    let request: serde_json::Value = serde_json::from_slice(spec.request()).expect("prepare request");
    let landing = job.workspace.as_ref().expect("workspace").workstream;
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing").clone();
    assert_eq!(row.landing.members.len(), 4);
    let mut base = fake::commit_count(&row.landing.members[0].base.expect("base")).expect("fake base");
    let conflict =
        row.landing.members.iter().position(|member| member.change == conflicted).expect("conflicted member");
    let mut combined = Vec::new();
    for index in (0..4).filter(|index| *index != conflict) {
        let member = &row.landing.members[index];
        let head = fake::commit_count(&member.head.expect("head")).expect("fake head");
        let files = forge_world::landings::stack_files(&world.fake, base, head, world.fake_env.limits.limits.commits);
        let message = request["members"][index]["message"].as_str().expect("merge message");
        let candidate =
            fake::commit_tree(&mut world.fake, &world.fake_env.limits, base, Some(head), files, message.as_bytes())
                .expect("clean prefix");
        assert_eq!(
            world.external(1, raw::Op::Git(raw::Git::PushNew { branch: member.branch.clone(), commit: candidate })),
            raw::Answer::PushNew { existing: None }
        );
        base = candidate;
        combined.push(serde_json::json!({"member": index + 1, "by": "merge"}));
    }
    let result = serde_json::to_vec(&serde_json::json!({"combined": combined, "conflicts": [{"member": conflict + 1, "with": "branch", "files": ["file"], "cut": false}]})).expect("prepare result");
    let event = world.protocol.answer_job(
        Token::new(7),
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished { result: result.into(), work: host::Work { left: None, saved: None } },
    );
    world.send(event);
}

#[test]
fn drill_16_a_batch_four_conflict_is_settled_by_the_merge_job_without_an_agent() {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            batch: Some(4),
            landing_members: Some(0),
            slots: Some(8),
            tasks: Some(12),
            spec_bytes: Some(4096),
            inbox_messages: Some(16),
            merge_kind: true,
            merge_authority: true,
            ..StoryLimits::default()
        },
    )
    .with_merge_job(b".temper/merge")
    .with_forge_holds(12);
    world.adopt();
    world.send(hello());
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { panic!("chat") };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let mut changes = Vec::new();
    for number in 0..4 {
        let (change, producer) = make_change(&mut world, &chat, 100 + number, b"org/repo");
        let branch = producer.workspace.repositories[0].push.clone().expect("producer branch");
        world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
        let file = if number == 1 { b"file".to_vec() } else { format!("safe{number}").into_bytes() };
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, &file, b"member", 1)
            .expect("producer pushes");
        finish_producer(&mut world, &producer, forge_world::REPO);
        changes.push(change);
    }
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"branch", 2)
        .expect("conflicting branch push");
    for _ in 0..PAIR_TICKS {
        world.tick();
    }
    assert!(landing_rows(&world).is_empty());
    assert!(changes.iter().all(|change| matches!(
        super::super::goal_landing::row(&world, *change).change.state,
        temper_engine_domain_forge_change::State::Ready { .. }
    )));
    world.manual_prepare_jobs();
    world.drain_cut();
    world.restart_with_batch(4);
    world.send(hello());
    for _ in 0..PAIR_TICKS {
        world.tick();
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert!(
        !world.jobs.is_empty(),
        "prepare not assigned: landings {:?}; tasks {:?}",
        landing_rows(&world),
        world
            .store
            .rows
            .values()
            .filter_map(|record| {
                if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = record {
                    Some((row.number, &row.phase))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    );
    let prepare = world.jobs.remove(0);
    assert_eq!(prepare.kind, jig_domain_link::JobKind(1));
    let conflicted = changes[1];
    prepare_three(&mut world, &prepare, conflicted);
    world.automatic_prepare_jobs();
    for _ in 0..PAIR_TICKS {
        world.tick();
        if world.jobs.iter().any(|job| job.kind == jig_domain_link::JobKind(3)) {
            break;
        }
    }
    let index =
        world.jobs.iter().position(|job| job.kind == jig_domain_link::JobKind(3)).expect("merge job before resolution");
    let merge = world.jobs.remove(index);
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&merge.spec),
    )
    .expect("merge spec");
    assert_eq!(spec.command(), b"repo/.temper/merge");
    let row = super::super::goal_landing::row(&world, conflicted);
    let branch = row.branch.clone();
    let head = fake::commit_count(&row.change.last_head.expect("member head")).expect("head");
    let engine::ForgeStart::Merge { base, .. } =
        world.job_workspaces.get(&merge.run.raw()).expect("merge workspace").repositories[0].start
    else {
        panic!("merge start")
    };
    let base = fake::commit_count(&base).expect("base");
    let mut tree: std::collections::BTreeMap<_, _> = world
        .fake
        .object(base)
        .expect("base tree")
        .tree
        .iter()
        .map(|(path, content)| (path.clone(), content.clone()))
        .collect();
    tree.insert(b"file".as_slice().into(), b"settled".as_slice().into());
    let files = tree.into_iter().map(|(path, content)| raw::File { path, content }).collect();
    let resolved =
        fake::commit_tree(&mut world.fake, &world.fake_env.limits, head, Some(base), files, b"Settle the conflict")
            .expect("merge job resolves");
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Push { branch, commit: resolved, expected: Some(head) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let event = world.protocol.answer_job(
        Token::new(7),
        merge.run.raw(),
        merge.attempt.raw(),
        host::JobEnding::Finished {
            result: br#"{"settled":["file"],"left":[]}"#.as_slice().into(),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
    for _ in 0..PAIR_TICKS {
        world.tick();
        if changes.iter().all(|change| super::super::goal_landing::ended(&world, *change).is_some()) {
            break;
        }
    }
    assert!(
        changes.iter().all(|change| super::super::goal_landing::ended(&world, *change).is_some()),
        "{:?}",
        landing_rows(&world)
    );
    assert_eq!(
        world.assigned.iter().map(|run| run.task).collect::<std::collections::BTreeSet<_>>().len(),
        5,
        "only chat and four producers; no resolution agent"
    );
    let rows = landing_rows(&world);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].landing.members.len(), 4);
    assert_eq!(
        rows[0].landing.members.iter().find(|member| member.change == conflicted).expect("conflicted member").left,
        Some(forge_top::LandingLeave::Conflict)
    );
    assert_eq!(rows[1].landing.members.len(), 1);
    assert_eq!(rows[1].landing.members[0].change, conflicted);
    assert!(rows[1].landing.members[0].landed.is_some());
    world.close();
}
