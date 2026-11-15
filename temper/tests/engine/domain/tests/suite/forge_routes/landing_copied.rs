//! Copied project ceilings on shipped capacities (domain/landing.md, 4.2, 5.1,
//! 7.3, 13 and 16; domain/landing-search.md, 3–5).
use super::*;
use jig_domain_core::settings as s;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_world::forge_routes::box_limits;

// Four dispatched proofs include the shipped client's fallback polling intervals.
const STEPS: u32 = 6000;
const ROOT_ENTRIES: u32 = 20;
const CUE: &[u8] = b"gate-state";

#[derive(Clone, Copy)]
enum Policy {
    StockCopy,
    WithoutDeleteBranch,
}

fn copied_authority(value: &mut authority::Authority, policy: Policy) {
    value.delegation.kinds = value
        .delegation
        .kinds
        .iter()
        .copied()
        .filter(|kind| match kind {
            authority::Executor::Charter(number) => [1, 2, 4, 5, 9, 11, 12, 13, 14, 15, 16].contains(number),
            authority::Executor::Procedure(number) => [2, 3, 4, 6].contains(number),
            authority::Executor::Job(number) => [3, 4].contains(number),
            authority::Executor::Role(number) => [0, 1, 2].contains(number),
        })
        .collect();
    value.grants = value.grants.iter().filter(|grant| copied_grant(grant, policy)).cloned().collect();
}

fn copied_grant(grant: &authority::Grant, policy: Policy) -> bool {
    [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17].contains(&grant.kind)
        && (matches!(policy, Policy::StockCopy) || grant.kind != 15)
}

fn copied_policy(defaults: &s::Defaults, policy: Policy) -> jig_domain_core::ConfiguredPolicy {
    let mut copy = defaults.policies.get(&1).expect("stock policy").clone();
    copied_authority(&mut copy.policy.ceiling, policy);
    copy.policy.projections =
        copy.policy.projections.iter().filter(|grant| copied_grant(grant, policy)).cloned().collect();
    for role in &mut copy.policy.roles {
        copied_authority(&mut role.authority, policy);
    }
    copy
}

fn repository(world: &mut World) {
    let mut tree = vec![raw::File { path: CUE.into(), content: b"green".as_slice().into() }];
    for entry in 1..ROOT_ENTRIES {
        tree.push(raw::File {
            path: format!("file{entry}").into_bytes().into_boxed_slice(),
            content: b"base".as_slice().into(),
        });
    }
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            id: 2,
            name: b"org/repo".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: tree.into_boxed_slice(),
            labels: Box::new([]),
            checks: raw::Checks {
                contexts: Box::new([]),
                cues: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            workflows: Box::new([raw::Workflow {
                file: b"gate.yml".as_slice().into(),
                jobs: Box::new([b"test".as_slice().into()]),
                requirements: Box::new([]),
                fails: Box::new([]),
                flaky: Box::new([]),
                report: None,
                select: None,
                latency_min: Duration::from_secs(1),
                latency_max: Duration::from_secs(1),
                silent: 0,
                passes: 1000,
                cue: Some(raw::Cue::File { path: CUE.into(), green: b"green".as_slice().into() }),
            }]),
            protection: None,
            hooked: false,
        },
    );
    for (user, permission) in [(1, raw::Permission::Admin), (2, raw::Permission::Write), (7, raw::Permission::Read)] {
        fake::grant(&mut world.fake, b"org/repo", user, permission);
    }
    fake::manual_merges(
        &mut world.fake,
        b"org/repo",
        raw::ManualMerges { allow_manual_merge: true, autodetect_manual_merge: true },
    );
    let main = world.fake.branch(b"org/repo", b"main").expect("main");
    assert_eq!(world.fake.object(main).expect("root").tree.len(), ROOT_ENTRIES);
}

fn setup(policy: Policy, batch: u16) -> (World, engine::Assignment) {
    let defaults = goal_branch::stock_defaults();
    let mut copy = copied_policy(&defaults, policy);
    let mut limits = box_limits();
    if batch > 2 {
        limits.forge.landing_members = u32::from(batch);
        limits.forge.resources_per_task = u32::from(batch) + 1;
    }
    let mut world = World::configured_stock_turn_with(
        &defaults,
        &stock::box_model(),
        &limits,
        StoryLimits {
            batch: Some(batch),
            forge_body_bytes: Some(limits.forge.client.op_bytes),
            ..StoryLimits::default()
        },
    )
    .with_project_policy(copy.clone())
    .with_settings_owner(&forge_top::user_key(1, 7));
    world.fake_env.limits.limits.files = ROOT_ENTRIES;
    repository(&mut world);
    world.adopt();
    let before = temper_engine_domain_world::kinds::revision(&world.store.rows);
    let mut kinds = copy.policy.ceiling.delegation.kinds.into_vec();
    kinds.push(authority::Executor::Job(1));
    copy.policy.ceiling.delegation.kinds = kinds.into_boxed_slice();
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        kinds: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner"),
        key: [95; 16],
        digest: [95; 32],
        change: Box::new(s::Change {
            against: before,
            running: s::Running::Stays,
            items: Box::new([s::Item {
                setting: s::Setting::ProjectPolicy { project: 1 },
                edit: s::Edit::Set(s::Value::Policy(Box::new(copy.clone()))),
            }]),
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(
        temper_engine_domain_world::kinds::revision(&world.store.rows),
        before + 1,
        "owner revision: {:?}",
        world.web_replies
    );
    let Some(Record::Core(jig_domain_core::Record::People(people::Stored::Policy { value, .. }))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Policy(1))))
    else {
        panic!("copied project policy")
    };
    assert_eq!(value.ceiling.delegation.kinds.len(), copy.policy.ceiling.delegation.kinds.len());
    assert_eq!(value.ceiling.grants.len(), copy.policy.ceiling.grants.len());
    let mut hello = hello_prepare(4);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds = jig_test_profile::fixtures::bounds();
    worker.bounds.turns = 800;
    worker.bounds.time = Duration::from_secs(28_800);
    let model = stock::box_model();
    worker.bounds.endpoints = Box::new([jig_domain_link::EndpointBound {
        endpoint: model.endpoint,
        window: model.window,
        output: model.output,
    }]);
    world.send(hello);
    let ask = world.chat(1, b"Land the copied project's changes");
    let chat = assigned_chat(&mut world, 92, ask);
    (world, chat)
}

fn push(world: &mut World, producer: &engine::Assignment, file: &[u8], content: &[u8]) -> api::Commit {
    let branch = producer.workspace.repositories[0].push.as_ref().expect("push branch").clone();
    let main = world.fake.branch(b"org/repo", b"main").expect("main");
    if world.fake.branch(b"org/repo", &branch).is_none() {
        assert_eq!(
            world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: main })),
            raw::Answer::Branch(raw::Created::Created)
        );
    }
    translate::commit(
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, file, content, 1).expect("push"),
    )
}

fn finish(world: &mut World, producer: &engine::Assignment) {
    goal_landing::finish_run(
        world,
        producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
            connector: 1,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
}

fn approve(world: &mut World, changes: &[u64]) {
    for number in changes {
        if let Some((task, change::Delegate::Gate { .. })) = goal_landing::row(world, *number).delegate
            && world.root.tasks().task(task).is_some_and(|task| !matches!(task.phase, tasks::Phase::Ended(_)))
            && let Some(run) = world.assigned.iter().find(|run| run.task == task).cloned()
        {
            reviews::finish_review(
                world,
                &run,
                tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
            );
        }
    }
}

fn reviewed(world: &World, number: u64) -> bool {
    let row = goal_landing::row(world, number);
    row.delegate.is_none()
        && row
            .verdicts
            .iter()
            .any(|verdict| verdict.status == change::Status::Passed && Some(verdict.head) == row.change.last_head)
}

fn no_hold(world: &World) {
    for record in world.store.rows.values() {
        if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task))) =
            record
        {
            assert!(
                !matches!(task.phase, tasks::Phase::Held { .. }),
                "task {}: {:?}; last write: {:?}",
                task.number,
                task.phase,
                world.writes.last()
            );
        }
    }
}

fn await_landing_end(world: &mut World, changes: &[u64], landing: Option<u64>) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        no_hold(world);
        assert!(
            !world
                .writes
                .iter()
                .any(|write| matches!(write, api::Write::Merge { how, .. } if *how != api::MergeHow::FastForwardOnly)),
            "the turn only fast-forwards"
        );
        approve(world, changes);
        if let Some(row) =
            landing_rows(world).into_iter().find(|row| landing.is_none_or(|task| row.landing.task == task))
            && ended(world, row.landing.task)
        {
            return row.landing.task;
        }
    }
    panic!(
        "landing did not end: {:?}; changes: {:?}",
        landing_rows(world),
        changes.iter().map(|change| goal_landing::row(world, *change)).collect::<Vec<_>>()
    );
}

fn dispatches(world: &World) -> Vec<(&[u8], &[api::Input])> {
    world
        .writes
        .iter()
        .filter_map(|write| match write {
            api::Write::Dispatch { branch, inputs, .. } => Some((branch.as_ref(), inputs.as_ref())),
            _ => None,
        })
        .collect()
}

fn merged_pull(world: &mut World, number: u64) {
    let row = goal_landing::row(world, number);
    let change::State::Landed { merge, .. } = row.change.state else { panic!("change landed: {:?}", row.change.state) };
    let raw::Answer::Pull(pull) =
        world.external(1, raw::Op::Read(raw::Read::Pull { number: row.pull.expect("member pull") }))
    else {
        panic!("pull")
    };
    assert_eq!(pull.merged.map(translate::commit), Some(merge));
}

#[test]
fn a_deployment_sized_repository_root_lands_by_turn() {
    let (mut world, chat) = setup(Policy::StockCopy, 1);
    let (change, producer) = make_change(&mut world, &chat, 100, b"org/repo");
    push(&mut world, &producer, b"file1", b"fixed");
    finish(&mut world, &producer);
    let landing = await_landing_end(&mut world, &[change], None);
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(row.landing.preparation, 1);
    assert_eq!(dispatches(&world).len(), 1);
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }))
            .count(),
        1
    );
    let prepares = world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(task)))
        if task.requester == tasks::Party::Task(landing) && task.executor == tasks::Executor::Job { kind: 1 })
        })
        .count();
    assert_eq!(prepares, 1);
    merged_pull(&mut world, change);
}

#[test]
fn a_copied_projects_red_member_repairs_and_lands_later() {
    let (mut world, chat) = setup(Policy::WithoutDeleteBranch, 1);
    let base = world.fake.branch(b"org/repo", b"main");
    let (change, producer) = make_change(&mut world, &chat, 100, b"org/repo");
    let red_head = push(&mut world, &producer, CUE, b"red");
    finish(&mut world, &producer);
    let first = await_landing_end(&mut world, &[change], None);
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == first).expect("first landing");
    assert!(matches!(row.landing.members[0].left, Some(forge_top::LandingLeave::Failed { .. })));
    assert!(row.landing.members.iter().all(|member| member.landed.is_none()));
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(world.fake.branch(b"org/repo", b"main"), base);
    let task = task_row(&world, first).expect("ended landing");
    assert!(!task.authority.grants.iter().any(|grant| grant.kind == 15));
    for member in &row.landing.members {
        assert_eq!(world.fake.branch(b"org/repo", &member.branch), None);
    }
    assert_eq!(task.holdings.len(), 1, "the landing's branch was held");
    for holding in &task.holdings {
        let tasks::Holding::Write { resource, .. } = holding else { panic!("branch holding") };
        assert_eq!(world.root.tasks().holder(resource), None);
    }
    assert!(matches!(goal_landing::row(&world, change).change.state, change::State::Repairing { .. }));
    let first_key = dispatches(&world)[0]
        .1
        .iter()
        .find(|input| input.name.as_ref() == b"temper_key")
        .expect("first key")
        .value
        .clone();
    let repair = (0..STEPS)
        .find_map(|_| {
            world.tick();
            no_hold(&world);
            let (task, change::Delegate::Repair(_)) = goal_landing::row(&world, change).delegate? else { return None };
            world.assigned.iter().find(|run| run.task == task).cloned()
        })
        .expect("repair assigned");
    let repaired = push(&mut world, &repair, CUE, b"green repaired");
    assert_ne!(repaired, red_head);
    finish(&mut world, &repair);
    // Only the successor round's opening answer is delayed, after its member review.
    let successor = (0..STEPS)
        .find_map(|_| {
            world.tick();
            no_hold(&world);
            approve(&mut world, &[change]);
            landing_rows(&world).iter().find(|row| row.landing.task != first).map(|row| row.landing.task)
        })
        .expect("successor formed");
    world.hold_write_answers = Some(HoldWrite::All);
    for _ in 0..STEPS {
        world.tick();
        no_hold(&world);
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert_eq!(world.held_write_answers.len(), 1, "one opening answer held");
    let branch = &landing_rows(&world)
        .into_iter()
        .find(|row| row.landing.task == successor)
        .expect("successor")
        .landing
        .round
        .branch;
    assert!(
        matches!(world.writes.last(), Some(api::Write::OpenPull { head, .. }) if head == branch),
        "successor opening answer: {:?}",
        world.writes.last()
    );
    world.release_write_answers();
    assert_eq!(await_landing_end(&mut world, &[change], Some(successor)), successor);
    assert_eq!(dispatches(&world).len(), 2);
    let second_key = &dispatches(&world)[1]
        .1
        .iter()
        .find(|input| input.name.as_ref() == b"temper_key")
        .expect("successor key")
        .value;
    assert_ne!(&first_key, second_key);
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == successor).expect("successor");
    assert_eq!(row.landing.members[0].head, Some(repaired));
    assert!(row.landing.members[0].landed.is_some());
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::OpenPull { head, .. } if *head == row.landing.round.branch))
            .count(),
        1,
        "held opening is found once"
    );
    merged_pull(&mut world, change);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one copied batch follows shared readiness through its search and both round turns"
)]
fn a_batch_of_four_narrows_its_breaking_member_with_deployment_limits() {
    let (mut world, chat) = setup(Policy::StockCopy, 4);
    // One dependency keeps every reviewed change out of the forming decision.
    let mut latch = temper_engine_domain_world::fixtures::report_delegate(b"Ready together", Box::new([]));
    latch.authority = live_task(&world, chat.task).authority.clone();
    latch.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    latch.authority.budget.spend = goal_branch::stock_defaults()
        .charters
        .iter()
        .find(|charter| charter.number == 1)
        .expect("stock chat charter")
        .spend;
    let engine::CallAnswer::Delegated(made) =
        world.scripted_call(&chat, 110, engine::Tool::Delegate { batch: Box::new([latch]) })
    else {
        panic!("latch delegated")
    };
    let latch = made[0].task;
    let mut changes = Vec::new();
    for number in 1..=4 {
        let (change, producer) = make_change(&mut world, &chat, 100 + number, b"org/repo");
        push(&mut world, &producer, format!("file{number}").as_bytes(), b"fixed");
        if number == 2 {
            push(&mut world, &producer, CUE, b"red");
        }
        changes.push(change);
        let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"Ready together");
        amendment.dependencies = Some(Box::new([latch]));
        let revision = live_task(&world, change).revisions.spec;
        let answer = world.scripted_call(
            &chat,
            120 + number,
            engine::Tool::Amend { keep_parameters: false, target: change, revision, amendment },
        );
        assert!(matches!(answer, engine::CallAnswer::Amended { .. }), "dependency: {answer:?}");
        finish(&mut world, &producer);
        for _ in 0..STEPS {
            world.tick();
            no_hold(&world);
            approve(&mut world, &changes);
            if reviewed(&world, change) {
                break;
            }
        }
    }
    assert!(landing_rows(&world).is_empty(), "no member forms alone");
    assert!(
        changes.iter().all(|number| reviewed(&world, *number)),
        "every review ended before readiness is released: {:?}",
        changes.iter().map(|number| (number, goal_landing::row(&world, *number).delegate)).collect::<Vec<_>>()
    );
    let latch_run = world.assigned.iter().find(|run| run.task == latch).expect("latch run").clone();
    goal_landing::finish_run(
        &mut world,
        &latch_run,
        tasks::TaskResult::Report { words: b"Ready together".as_slice().into() },
    );
    let landing = await_landing_end(&mut world, &changes, None);
    let rows = landing_rows(&world);
    assert_eq!(rows.len(), 1);
    let row = rows[0];
    assert_eq!(row.landing.task, landing);
    assert_eq!(row.landing.members.len(), changes.len());
    assert!(row.landing.members.iter().all(|member| changes.contains(&member.change)));
    assert_eq!(row.landing.members[1].change, changes[1], "the second member is red");
    assert_eq!(row.landing.preparation, 2);
    assert!(matches!(row.landing.members[1].left, Some(forge_top::LandingLeave::Blamed { .. })));
    assert!(row.landing.members[1].landed.is_none());
    assert!(matches!(goal_landing::row(&world, changes[1]).change.state, change::State::Repairing { .. }));
    let dispatches = dispatches(&world);
    assert_eq!(dispatches.len(), 4);
    assert_eq!(
        dispatches.iter().map(|(branch, _)| *branch).collect::<Vec<_>>(),
        [
            row.landing.members[3].branch.as_ref(),
            row.landing.members[1].branch.as_ref(),
            row.landing.members[0].branch.as_ref(),
            row.landing.members[3].branch.as_ref()
        ],
        "top, probe 2 red, probe 1 green, restacked top"
    );
    assert_eq!(row.landing.members[2].base, row.landing.members[0].landed);
    assert_eq!(row.landing.members[3].base, row.landing.members[0].landed);
    assert_eq!(row.landing.members[0].round, Some(1));
    assert_eq!(row.landing.members[2].round, Some(2));
    assert_eq!(row.landing.members[3].round, Some(2));
    let third = row.landing.members[2].landed.expect("third landed");
    let fourth = row.landing.members[3].landed.expect("fourth landed");
    assert_eq!(
        world
            .fake
            .object(fake::commit_count(&fourth).expect("fourth commit"))
            .expect("restacked top")
            .parent
            .map(translate::commit),
        Some(third)
    );
    let merges: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            api::Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } => Some(*head),
            _ => None,
        })
        .collect();
    assert_eq!(
        merges,
        [row.landing.members[0].landed.expect("round 1"), row.landing.members[3].landed.expect("round 2")]
    );
    for number in [changes[0], changes[2], changes[3]] {
        merged_pull(&mut world, number);
    }
}
