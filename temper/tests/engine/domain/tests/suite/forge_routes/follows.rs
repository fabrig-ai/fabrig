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
