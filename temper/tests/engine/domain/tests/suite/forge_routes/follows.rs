//! Follow changes keep source landings, route questions up and wait for dependencies (domain/forge/follows.md, 4).
use super::*;
use temper_engine_domain_forge_change::{Early, Route, State};

const TICKS: u32 = 1000;
const QUIET_TICKS: u32 = 40; // Four simulated seconds settle the pushed-head reads.
const PRODUCER_SPEND: u64 = 60; // One producer's run reserve in this world.
const CHANNEL: Token = Token::new(7);
const FOLLOWS: &[u8] = b"3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

// Code builds the delegate; neither a model nor the change call supplies parameter 18.
fn make(world: &mut World, requester: &engine::Assignment, template: u64, follows: bool) -> u64 {
    let source = live_task(world, template);
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(
        b"Port the source landing",
        Box::new([engine::Dependency::Existing(template)]),
    );
    delegate.executor = source.executor;
    delegate.spec.parameters.clone_from(&source.spec.parameters);
    if follows {
        let mut parameters = delegate.spec.parameters.into_vec();
        parameters.push(tasks::Parameter::Bytes { name: 18, value: FOLLOWS.into() });
        delegate.spec.parameters = parameters.into_boxed_slice();
    }
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: world.env.limits.tasks.result_bytes };
    delegate.authority = source.authority.clone();
    delegate.authority.budget.spend = PRODUCER_SPEND;
    delegate.authority.delegation.tasks = 1;
    delegate.authority.delegation.depth = 1;
    questions::call(world, requester, 110, engine::Tool::Delegate { batch: Box::new([delegate]) });
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else {
        panic!("follow delegate admitted: {:?} {:?}", world.answers, world.call_errors)
    };
    made[0].task
}

fn next_producer(world: &mut World, change: u64) -> engine::Assignment {
    for _ in 0..TICKS {
        if let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change)))
            && let forge_top::Stored::Change(row) = row.as_ref()
            && let Some((child, _)) = row.delegate
            && let Some(run) = world.assigned.iter().rev().find(|run| run.task == child)
        {
            return run.clone();
        }
        world.tick();
    }
    panic!("producer assigned for {change}: {:?}", questions::row(world, change));
}

fn wait_without_producer(world: &mut World, task: u64, dependency: u64) {
    for _ in 0..QUIET_TICKS {
        world.tick();
    }
    let row = questions::row(world, task);
    assert_eq!(row.delegate, None, "a follow makes no producer while its dependency lives");
    assert_eq!(row.change.early, Early::None);
    assert_eq!(live_task(world, task).waiting_on.as_ref(), &[dependency]);
    assert!(live_task(world, task).started_from.is_empty());
    assert!(!ended(world, dependency));
}

fn ended(world: &World, task: u64) -> bool {
    world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
}

fn finish(world: &mut World, producer: &engine::Assignment) {
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Add prerequisite".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"Done: pushed.".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

#[test]
fn follow_changes_wait_for_landing_or_an_amendment_but_ordinary_changes_start_early() {
    // Landing, dropping the dependency, and the unchanged ordinary early start.
    for (follows, amend) in [(true, false), (true, true), (false, false)] {
        let (mut world, requester, branch, base) = early_start::goal_world();
        let first = early_start::make(&mut world, &requester, b"Add prerequisite", &[]);
        let upstream = next_producer(&mut world, first);
        let second = make(&mut world, &requester, first, follows);
        if follows {
            wait_without_producer(&mut world, second, first);
        }
        let head = early_start::push(&mut world, &upstream, base, b"prerequisite");
        early_start::observe_push(&mut world, &upstream);
        if follows {
            wait_without_producer(&mut world, second, first);
            let mut sha1 = [0_u8; 32];
            sha1[..20].fill(0xaa);
            let mut next = [0_u8; 32];
            next[..20].fill(0xbb);
            let expected = Some(forge_top::ChangeFollows { follow: 3, commits: Box::new([sha1, next]) });
            assert_eq!(questions::row(&world, second).follows, expected);
            world.drain_cut();
            world.restart_same();
            world.until(Until::Ready);
            assert_eq!(questions::row(&world, second).follows, expected, "restart keeps parameter 18");
            wait_without_producer(&mut world, second, first);
            if amend {
                let revision = spec_revision(&world, second);
                let mut amendment =
                    temper_engine_domain_world::fixtures::task_amendment(None, None, b"Drop prerequisite");
                amendment.dependencies = Some(Box::new([]));
                questions::call(
                    &mut world,
                    &requester,
                    111,
                    engine::Tool::Amend { keep_parameters: false, target: second, revision, amendment },
                );
                assert!(
                    matches!(world.answers.last(), Some(engine::CallAnswer::Amended { task, .. }) if *task == second)
                );
                assert!(live_task(&world, second).dependencies.is_empty());
                assert!(!ended(&world, first));
            } else {
                finish(&mut world, &upstream);
                for _ in 0..TICKS {
                    world.tick();
                    if ended(&world, first) {
                        break;
                    }
                    assert_eq!(questions::row(&world, second).delegate, None);
                    assert!(live_task(&world, second).started_from.is_empty());
                }
                assert!(ended(&world, first));
                assert!(matches!(questions::row(&world, first).change.state, State::Landed { .. }));
            }
        }
        let producer = next_producer(&mut world, second);
        if follows {
            assert!(live_task(&world, second).started_from.is_empty());
            assert_eq!(questions::row(&world, second).change.early, Early::None);
            assert_eq!(producer.workspace.repositories[0].start, engine::ForgeStart::Base(branch));
        } else {
            assert!(!ended(&world, first));
            assert_eq!(live_task(&world, second).started_from.as_ref(), &[first]);
            assert_eq!(
                producer.workspace.repositories[0].start,
                engine::ForgeStart::Merged { branch, heads: Box::new([translate::commit(head)]) }
            );
        }
        early_start::stop(&mut world, &requester);
    }
}

#[test]
fn a_follow_producers_question_reaches_its_requester_without_parameter_14() {
    let (mut world, requester) = questions::plan_world(None);
    let first = questions::make_change(&mut world, &requester, 109, engine::ChangeQuestions::None);
    next_producer(&mut world, first);
    let second = make(&mut world, &requester, first, true);
    let revision = spec_revision(&world, second);
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"Independent port");
    amendment.dependencies = Some(Box::new([]));
    questions::call(
        &mut world,
        &requester,
        112,
        engine::Tool::Amend { keep_parameters: false, target: second, revision, amendment },
    );
    assert!(matches!(world.answers.last(), Some(engine::CallAnswer::Amended { task, .. }) if *task == second));
    let producer = next_producer(&mut world, second);
    assert_eq!(questions::row(&world, second).questions.route, Route::Up);
    assert!(
        !live_task(&world, second)
            .spec
            .parameters
            .iter()
            .any(|p| matches!(p, tasks::Parameter::Number { name: 14, .. }))
    );
    let words = b"Question: which source behavior should be ported?";
    world.producer_question(&producer, 113, words);
    for _ in 0..TICKS {
        world.tick();
        if !questions::replies(&world, requester.task, second).is_empty() {
            break;
        }
    }
    let replies = questions::replies(&world, requester.task, second);
    assert_eq!(replies.len(), 1, "relay: {:?}", questions::row(&world, second).questions);
    assert_eq!(replies[0].kind, tasks::MessageKind::Question);
    assert!(replies[0].words.starts_with(words));
    world.close();
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one external procedure batch drives its follow change through dispatch and landing"
)]
fn a_procedure_batchs_follow_change_dispatches_its_own_gate_before_queue_or_turn_landing() {
    use temper_engine_domain_forge::settings as s;
    use temper_engine_domain_world::{kinds, tier_runs};
    for turn in [false, true] {
        let mut world = World::configured_limits(
            true,
            true,
            false,
            1000,
            Gate::default(),
            None,
            StoryLimits {
                turn,
                tier: Some(tier_runs::Tier::default()),
                procedure_batch: true,
                kinds: true,
                tasks: Some(12),
                slots: Some(2),
                ..StoryLimits::default()
            },
        );
        world.adopt();
        fake::repository(
            &mut world.fake,
            &world.fake_env.limits,
            raw::Setup {
                id: 3,
                name: b"org/source".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([]),
                labels: Box::new([]),
                workflows: Box::new([]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        fake::grant(&mut world.fake, b"org/source", 1, raw::Permission::Admin);
        let mut adoption = temper_engine_domain_world::fixtures::adoption(1);
        adoption.name = b"source".as_slice().into();
        adoption.home = false;
        adoption.role = forge_top::Role::Context;
        world.send(engine::Event::Party(engine::PartyInput::Application {
            scope: people::ApplicationScope::Project(1),
            reply_to: ReplyTo::new(Token::new(94)),
            sign_in: world.signed_in.expect("the follow fixture is admitted"),
            key: [94; 16],
            digest: [94; 32],
            carried: None,
            request: engine::Application::Forge(forge_top::Ask::Adopt(adoption)),
        }));
        for _ in 0..200 {
            world.tick();
            if world.adopted.len() == 2 {
                break;
            }
        }
        assert!(matches!(world.adopted.last(), Some(forge_top::ApplicationOutcome::Adopted(_))));
        world.until(Until::Ready);
        let workflow: &[u8] = if turn { b"gate.yml" } else { b"slow.yml" };
        let setting = s::Follow {
            source: s::FollowSource {
                repository: skein_forge::api::Repository { forge: 1, repository: 3 },
                branch: b"main".as_slice().into(),
                paths: Box::new([]),
            },
            target: s::FollowTarget { repository: forge_world::REPO, branch: b"main".as_slice().into() },
            owner: s::FollowOwner::Person(1),
            start: s::FollowStart::Code,
            producer: None,
            check: s::FollowCheck {
                workflow: workflow.into(),
                artifact: b"report".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Junit,
            },
        };
        let revision = kinds::revision(&world.store.rows);
        world.send(kinds::change(
            world.signed_in.expect("the follow fixture is admitted"),
            revision,
            97,
            Box::new([kinds::item(12, 1, 1, &s::Value::Follow(setting))]),
        ));
        world.until(Until::Ready);
        assert_eq!(kinds::revision(&world.store.rows), revision + 1, "follow setting: {:?}", world.web_replies);
        world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
        let ask = tier_runs::goal(&world);
        let chat = assigned_chat(&mut world, 95, ask);
        let mut parent = temper_engine_domain_world::fixtures::report_delegate(b"Carry a follow change", Box::new([]));
        parent.executor = tasks::Executor::Procedure { connector: 1, code: 1 };
        parent.authority = live_task(&world, chat.task).authority.clone();
        parent.authority.budget.spend = 200;
        parent.authority.delegation.tasks = 4;
        parent.authority.delegation.depth = 2;
        questions::call(&mut world, &chat, 96, engine::Tool::Delegate { batch: Box::new([parent]) });
        let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else {
            panic!("parent procedure: {:?}", world.call_errors)
        };
        let parent = made[0].task;
        let mut child = temper_engine_domain_world::fixtures::report_delegate(b"Port the source landing", Box::new([]));
        child.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
        child.authority = live_task(&world, parent).authority.clone();
        child.authority.budget.spend = 100;
        child.authority.delegation.tasks = 1;
        child.authority.delegation.depth = 1;
        child.contract = tasks::Contract::Change { connector: 0, kind: 1, words: world.env.limits.tasks.result_bytes };
        child.spec.parameters = Box::new([
            tasks::Parameter::Resource { name: 1, connector: 0, resource: forge_top::tag(1, 2) },
            tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
            tasks::Parameter::Bytes {
                name: 18,
                value: b"1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .as_slice()
                    .into(),
            },
        ]);
        world.send(engine::Event::Party(engine::PartyInput::ProcedureStep {
            task: parent,
            step: 1,
            connector: 1,
            code: 1,
            decision: engine::ProcedureStep {
                batch: Box::new([child]),
                ..engine::ProcedureStep::new(engine::StepEnd::Wait)
            },
        }));
        let mut change = None;
        for _ in 0..TICKS {
            world.tick();
            change = world.store.rows.values().find_map(|record| match record {
                Record::Forge { row } => match row.as_ref() {
                    forge_top::Stored::Change(row) if row.follows.is_some() => Some(row.task),
                    _ => None,
                },
                Record::Core(_) => None,
            });
            if change.is_some() {
                break;
            }
        }
        let change = change.expect("procedure batch committed its follow change");
        let producer = next_producer(&mut world, change);
        let head = early_start::push(&mut world, &producer, 1, b"ported");
        early_start::observe_push(&mut world, &producer);
        finish(&mut world, &producer);
        let mut dispatched = false;
        for _ in 0..tier_runs::RUN_TICKS * 2 {
            world.tick();
            if world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Dispatch { inputs, .. } if inputs.iter().any(|input| input.name.as_ref() == b"temper_follows"))) {
                dispatched = true;
            }
            if ended(&world, change) {
                break;
            }
        }
        assert!(
            dispatched,
            "the change dispatched its workflow: {:?}; task={:?}",
            questions::row(&world, change).change.state,
            live_task(&world, change).phase
        );
        assert!(ended(&world, change), "verification ends before the cold oracle");
        let row = questions::row(&world, change);
        assert!(matches!(row.change.state, State::Landed { .. } | State::Cleared { .. }));
        let calls: Vec<_> = world
            .writes
            .iter()
            .filter_map(|write| match write {
                skein_forge::api::Write::Dispatch { inputs, .. }
                    if inputs.iter().any(|input| input.name.as_ref() == b"temper_follows") =>
                {
                    Some(inputs)
                }
                _ => None,
            })
            .collect();
        assert_eq!(calls.len(), 1);
        assert!(calls[0].iter().any(|input| input.name.as_ref() == b"temper_commit"
            && input.value == forge_top::commit_text(&translate::commit(head))));
        let Some(Record::Forge { row }) =
            world.store.rows.get(&Key::Forge(forge_top::Key::Follow { project: 1, follow: 1 }))
        else {
            panic!("parity record")
        };
        let forge_top::Stored::Follow(row) = row.as_ref() else { panic!("follow") };
        assert_eq!(
            row.parity.as_ref().expect("the follow fixture is admitted").verdict,
            forge_top::FollowVerdict::Passed
        );
        assert!(world.writes.iter().all(|write| !matches!(write, skein_forge::api::Write::Status { context, .. } if context.starts_with(b"temper/gate/follow."))));
        world.close();
    }
}
