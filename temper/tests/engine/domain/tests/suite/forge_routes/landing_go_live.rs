//! Going live across a restart with running work kept on its revision
//! (temper/docs/design/domain/landing.md, 13, 15;
//! temper/docs/design/protocol/configuration/settings.md, 1).
use super::super::{goal_branch, goal_landing, reviews};
use super::*;
use jig_domain_core::settings as s;

fn narrow(authority: &mut authority::Authority) {
    authority.grants = authority.grants.iter().filter(|grant| grant.kind != 15).cloned().collect();
    authority.delegation.kinds =
        authority.delegation.kinds.iter().filter(|kind| **kind != authority::Executor::Job(1)).copied().collect();
}

fn narrow_policy(policy: &mut jig_domain_core::ConfiguredPolicy) {
    narrow(&mut policy.policy.ceiling);
    policy.policy.projections = policy.policy.projections.iter().filter(|grant| grant.kind != 15).cloned().collect();
    for role in &mut policy.policy.roles {
        narrow(&mut role.authority);
    }
}

fn project_policy(world: &World) -> &people::PolicyValue {
    let Some(Record::Core(jig_domain_core::Record::People(people::Stored::Policy { value, .. }))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Policy(1))))
    else {
        panic!("project policy")
    };
    value
}

fn landing(by: engine::Way) -> engine::ProjectLanding {
    engine::ProjectLanding {
        by,
        batch: 1,
        combine_main: engine::Combine::Merge,
        combine_other: engine::Combine::Squash,
        gate_retries: 2,
        gate_timeout: Duration::from_secs(3600),
        landing_attempts: engine::LANDING_ATTEMPTS,
        check_timeout: None,
        close_job: None,
        merge_job: None,
        after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
        repositories: Box::new([engine::RepositoryChecks {
            repository: forge_world::REPO,
            gates: Box::new([engine::GateCheck {
                name: b"gate".as_slice().into(),
                workflow: b"gate.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
            }]),
            tiers: Box::new([]),
        }]),
    }
}

fn worker_hello(hosting: Box<[jig_domain_link::Hosting]>) -> engine::Event {
    let mut hello = hello_prepare(3);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("hello")
    };
    worker.hosting = hosting;
    worker.bounds = jig_test_profile::fixtures::bounds();
    worker.bounds.turns = 800;
    worker.bounds.time = Duration::from_secs(28_800);
    let model = stock::box_model();
    worker.bounds.endpoints = Box::new([jig_domain_link::EndpointBound {
        endpoint: model.endpoint,
        window: model.window,
        output: model.output,
    }]);
    hello
}

#[test]
#[expect(clippy::too_many_lines, reason = "one owner revision takes a running producer across the go-live restart")]
fn a_project_with_its_own_roles_goes_live_by_turn_with_running_stays() {
    let broad = goal_branch::stock_defaults();
    let mut defaults = goal_branch::stock_defaults();
    narrow(&mut defaults.rules.ceiling);
    narrow_policy(defaults.policies.get_mut(&1).expect("policy 1"));
    let mut own = broad.policies.get(&1).expect("policy 1").clone();
    own.permissions[0].role = 2;
    own.policy.roles.iter_mut().find(|role| role.number == 1).expect("member role").requests = authority::Requests(185);
    assert_ne!(own, *broad.policies.get(&1).expect("stock policy"));
    let mut initial = own.clone();
    narrow_policy(&mut initial);
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    limits.documents.lines = 8;
    let mut world = World::configured_stock_turn(&defaults, &stock::box_model(), &limits)
        .with_project_policy(initial)
        .with_project_landing(1, landing(engine::Way::Queue))
        .with_settings_owner(&forge_top::user_key(1, 7));
    world.adopt();
    world.send(worker_hello(Box::new([])));
    let ask = world.chat(1, b"Make a small fix");
    let chat = assigned_chat(&mut world, 92, ask);
    let (change, producer) = make_change(&mut world, &chat, 93, b"org/repo");
    let prior = temper_engine_domain_world::kinds::revision(&world.store.rows);
    assert_eq!(live_task(&world, producer.task).settings.task, prior);
    assert_eq!(live_task(&world, producer.task).settings.run, Some(prior));
    assert!(matches!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    world.drain_cut();
    let policy_before = project_policy(&world).clone();
    assert_eq!(policy_before.permissions, own.permissions);
    assert_eq!(policy_before.roles.iter().find(|role| role.number == 1).expect("own member").requests, 185);
    world.restart_with_project_landing(1, landing(engine::Way::Turn));
    for _ in 0..PAIR_TICKS {
        world.tick();
        if world.root.ready() {
            break;
        }
    }
    assert!(world.root.ready(), "restart: {:?}; {:?}", world.root.start_phase(), world.root.start_stop());
    world.send(worker_hello(
        [&chat, &producer].map(|run| protocol::hosting(run.task, run.attempt, jig_domain_link::Phase::Active)).into(),
    ));
    assert_eq!(*project_policy(&world), policy_before, "restart keeps the project's own roles");
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        kinds: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner"),
        key: [95; 16],
        digest: [95; 32],
        change: Box::new(s::Change {
            against: prior,
            running: s::Running::Stays,
            items: Box::new([
                s::Item { setting: s::Setting::Rules, edit: s::Edit::Set(s::Value::Rules(Box::new(broad.rules))) },
                s::Item {
                    setting: s::Setting::Policy(1),
                    edit: s::Edit::Set(s::Value::Policy(Box::new(broad.policies.get(&1).expect("policy 1").clone()))),
                },
                s::Item {
                    setting: s::Setting::ProjectPolicy { project: 1 },
                    edit: s::Edit::Set(s::Value::Policy(Box::new(own.clone()))),
                },
            ]),
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(temper_engine_domain_world::kinds::revision(&world.store.rows), prior + 1, "{:?}", world.web_replies);
    assert_eq!(live_task(&world, producer.task).settings.task, prior, "running stays");
    assert_eq!(live_task(&world, producer.task).settings.run, Some(prior));
    assert!(matches!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    assert_eq!(project_policy(&world).permissions, own.permissions);
    assert_eq!(project_policy(&world).roles.iter().find(|role| role.number == 1).expect("own member").requests, 185);
    push_change(&mut world, &producer, b"org/repo");
    let tasks::Contract::Change { connector, .. } = live_task(&world, producer.task).contract else {
        panic!("producer's change contract")
    };
    goal_landing::finish_run(
        &mut world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
            connector,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    let mut reviewed = Vec::new();
    for _ in 0..PAIR_TICKS {
        world.tick();
        let row = goal_landing::row(&world, change);
        assert!(!matches!(row.change.state, temper_engine_domain_forge_change::State::Held { .. }), "{row:?}");
        assert_eq!(row.change.repairs, 0, "{row:?}");
        assert_eq!(row.change.resolutions, 0, "{row:?}");
        for row in landing_rows(&world) {
            assert!(!matches!(
                task_row(&world, row.landing.task).expect("landing task").phase,
                tasks::Phase::Held { .. }
            ));
        }
        if let Some((task, temper_engine_domain_forge_change::Delegate::Gate { .. })) = row.delegate
            && !reviewed.contains(&task)
            && let Some(run) = world.assigned.iter().find(|run| run.task == task).cloned()
        {
            reviewed.push(task);
            reviews::finish_review(
                &mut world,
                &run,
                tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
            );
        }
        if ended(&world, change) && landing_rows(&world).iter().all(|row| ended(&world, row.landing.task)) {
            break;
        }
    }
    let row = goal_landing::row(&world, change);
    assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. }), "{row:?}");
    let rows = landing_rows(&world);
    assert_eq!(rows.len(), 1, "one turn landing");
    let landing = &rows[0].landing;
    assert_eq!(landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(landing.members.len(), 1);
    assert_eq!(landing.members[0].change, change);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(), 1);
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }))
            .count(),
        1
    );
    assert!(ended(&world, change));
    assert!(ended(&world, landing.task));
    world.close();
}
