//! A change amends its live producer (temper/docs/design/domain/forge/change/questions.md, 1).
use super::questions::{
    answer, ask, call, finish_producing, make_change, next_producer, plan_world_with_limits, replies, row, wait_answer,
};
use super::*;
use temper_engine_domain_forge_change::{Stands, State};
use temper_engine_domain_world::fixtures::task_amendment;

const TICKS: u32 = 200;
const CALL: u64 = 94;
const WORDS: &[u8] = b"Include the command line";
const REASON: &[u8] = b"Widen to the CLI";
const SCOPE: &[u8] = b"\n\nThe host pushes what you deliver; the change task opens the pull request, waits for CI and merges it. Finish with your change once it is checked. Report the branch and head you produced; only the change task reports landing after merge.";

fn plan_world() -> (World, engine::Assignment) {
    // A batch of two gives a 256-byte message bound, enough for the whole reason form.
    plan_world_with_limits(None, &StoryLimits { batch: Some(2), ..StoryLimits::default() }, false)
}

fn up_world() -> (World, engine::Assignment, u64, engine::Assignment) {
    let (mut world, requester) = plan_world();
    let change = make_change(&mut world, &requester, CALL - 1, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    world.until(Until::Ready);
    (world, requester, change, producer)
}

fn amend(
    world: &mut World,
    requester: &engine::Assignment,
    change: u64,
    number: u64,
    words: Option<&[u8]>,
    dependency: Option<u64>,
) {
    let mut amendment = task_amendment(words, None, REASON);
    amendment.dependencies = dependency.map(|task| Box::new([task]) as Box<[u64]>);
    let revision = live_task(world, change).revisions.spec;
    let parameters = live_task(world, change).spec.parameters.clone();
    call(world, requester, number, engine::Tool::Amend { keep_parameters: true, target: change, revision, amendment });
    assert!(
        matches!(world.answers.last(), Some(engine::CallAnswer::Amended { .. })),
        "amended: {:?}; errors: {:?}",
        world.answers,
        world.call_errors
    );
    assert_eq!(live_task(world, change).spec.parameters, parameters);
    for _ in 0..TICKS {
        world.tick();
        if live_task(world, change).inbox.is_empty() && world.store.pending.is_empty() {
            return;
        }
    }
    panic!("the change takes its amendment: {:?}; {:?}", live_task(world, change), row(world, change));
}

fn amendment(world: &World, change: u64, producer: u64) -> tasks::Word {
    let words = replies(world, producer, change);
    let amendments: Vec<_> =
        words.into_iter().filter(|message| matches!(message.kind, tasks::MessageKind::Amendment { .. })).collect();
    let [message] = amendments.as_slice() else { panic!("one amendment: {amendments:?}") };
    assert_eq!(message.kind, tasks::MessageKind::Amendment { revision: live_task(world, producer).revisions.spec });
    assert_eq!(message.words.as_ref(), format!("Task {change}, your change, was amended to spec revision {}. Your spec now holds its amended words; work to them from here on. Its reason: Widen to the CLI", live_task(world, change).revisions.spec).as_bytes());
    (*message).clone()
}

fn next_attempt(world: &mut World, producer: &engine::Assignment) -> engine::Assignment {
    for _ in 0..TICKS {
        world.tick();
        if let Some(run) = world.assigned.iter().find(|run| run.task == producer.task && run.attempt > producer.attempt)
        {
            return run.clone();
        }
    }
    panic!("the amended producer starts its next attempt");
}

fn assert_spec(world: &World, producer: u64, original: &tasks::Spec, words: &[u8]) {
    let mut expected = original.clone();
    expected.words = [words, SCOPE].concat().into_boxed_slice();
    assert_eq!(live_task(world, producer).spec, expected, "only the producer's words change");
    assert_eq!(world.root.tasks().task(producer).expect("producer in the core").spec, expected);
}

fn assert_offered(run: &engine::Assignment, amendment: &tasks::Word, words: &[u8]) {
    assert!(run.messages.contains(amendment), "the next attempt is offered the amendment");
    assert!(task_brief(run).starts_with(&[words, SCOPE, b"\n"].concat()), "the next brief uses the amended spec");
}

#[test]
fn a_running_then_parked_producer_hears_and_works_to_its_amendments() {
    // Both routes amend, including the route with no question relay.
    for route in [engine::ChangeQuestions::None, engine::ChangeQuestions::Up] {
        let (mut world, requester) = plan_world();
        let change = make_change(&mut world, &requester, CALL, route);
        let producer = next_producer(&mut world, change);
        world.until(Until::Ready);
        let original = live_task(&world, producer.task).spec.clone();
        let revision = live_task(&world, producer.task).revisions.spec;
        assert!(matches!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
        amend(&mut world, &requester, change, CALL + 1, Some(WORDS), None);
        assert_spec(&world, producer.task, &original, WORDS);
        assert_eq!(live_task(&world, producer.task).revisions.spec, revision + 1);
        let first = amendment(&world, change, producer.task);
        for _ in 0..TICKS {
            world.tick();
            if world
                .delivered_messages
                .get(&(producer.task, producer.attempt))
                .is_some_and(|messages| messages.contains(&first.number))
            {
                break;
            }
        }
        assert!(
            world
                .delivered_messages
                .get(&(producer.task, producer.attempt))
                .is_some_and(|messages| messages.contains(&first.number)),
            "jig delivers the amendment to the live run"
        );
        // End a turn without consuming the amendment: the next attempt must receive it.
        world.send(protocol::turn(
            Token::new(7),
            producer.task,
            producer.attempt,
            1,
            0,
            None,
            b"Save the current work",
        ));
        world.until(Until::Ready);
        park_after_turn(&mut world, &producer, None, 1);
        let resumed = next_attempt(&mut world, &producer);
        assert_offered(&resumed, &first, WORDS);
        // Await a later timer, not a question, so jig can leave this result-ended task parked.
        let subscribed = world.scripted_call(
            &resumed,
            CALL + 3,
            engine::Tool::Subscribe {
                kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
            },
        );
        assert!(matches!(subscribed, engine::CallAnswer::Subscribed { .. }));
        let turns = live_task(&world, producer.task).turn;
        assert!(
            live_task(&world, producer.task).inbox.is_empty(),
            "amendment read before parking: {:?}",
            live_task(&world, producer.task)
        );
        park_after_turn(&mut world, &resumed, Some(first.number), turns);
        world.until(Until::Ready);
        assert!(live_task(&world, producer.task).questions.is_empty());
        assert_eq!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Idle));
        let revision = live_task(&world, producer.task).revisions.spec;
        let parked_words = b"Include the CLI and its help";
        amend(&mut world, &requester, change, CALL + 2, Some(parked_words), None);
        assert_spec(&world, producer.task, &original, parked_words);
        assert_eq!(live_task(&world, producer.task).revisions.spec, revision + 1);
        let second = amendment(&world, change, producer.task);
        let resumed = next_attempt(&mut world, &resumed);
        assert_offered(&resumed, &second, parked_words);
    }
}

#[test]
fn an_amendment_keeps_the_producers_question_open_until_its_answer() {
    let (mut world, requester, change, producer) = up_world();
    let question = ask(&mut world, &producer, change);
    park_after_turn(&mut world, &producer, None, 0);
    for _ in 0..TICKS {
        world.tick();
        if live_task(&world, producer.task).phase == tasks::Phase::Active(tasks::Active::Idle)
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_eq!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    let original = live_task(&world, producer.task).spec.clone();
    let open = live_task(&world, producer.task).questions.clone();
    amend(&mut world, &requester, change, CALL, Some(WORDS), None);
    assert_spec(&world, producer.task, &original, WORDS);
    let message = amendment(&world, change, producer.task);
    assert_eq!(row(&world, change).questions.open.len(), 1, "an amendment does not clear the question");
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(live_task(&world, producer.task).questions, open);
    for _ in 0..TICKS {
        world.tick();
    }
    assert!(
        !world.assigned.iter().any(|run| run.task == producer.task && run.attempt > producer.attempt),
        "an amendment never answers or lets a kept producer go on"
    );
    answer(&mut world, &requester, change, CALL + 1);
    wait_answer(&mut world, producer.task, change);
    let resumed = next_attempt(&mut world, &producer);
    assert_offered(&resumed, &message, WORDS);
    assert!(resumed.messages.iter().any(|message| message.kind == tasks::MessageKind::Answer { question }));
    assert!(live_task(&world, producer.task).questions.is_empty());
}

#[test]
fn dependencies_alone_leave_the_producer_unamended() {
    let (mut world, requester, change, producer) = up_world();
    let original = live_task(&world, producer.task).spec.clone();
    let revision = live_task(&world, producer.task).revisions.spec;
    let dependency = make_change(&mut world, &requester, CALL, engine::ChangeQuestions::None);
    amend(&mut world, &requester, change, CALL + 1, None, Some(dependency));
    assert_eq!(live_task(&world, change).waiting_on.as_ref(), &[dependency]);
    assert!(live_task(&world, change).inbox.is_empty(), "the amendment was taken");
    assert_eq!(live_task(&world, producer.task).revisions.spec, revision);
    assert_eq!(live_task(&world, producer.task).spec, original);
    assert!(
        !replies(&world, producer.task, change)
            .iter()
            .any(|message| matches!(message.kind, tasks::MessageKind::Amendment { .. }))
    );
}

#[test]
fn an_amendment_after_production_goes_on_without_amending_a_producer() {
    let (mut world, requester) =
        plan_world_with_limits(None, &StoryLimits { batch: Some(2), ..StoryLimits::default() }, true);
    let change = make_change(&mut world, &requester, CALL - 1, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    finish_producing(&mut world, change, &producer);
    for _ in 0..TICKS {
        world.tick();
        if matches!(row(&world, change).change.state, State::Checking { .. }) && world.store.pending.is_empty() {
            break;
        }
    }
    assert!(
        matches!(row(&world, change).change.state, State::Checking { .. }),
        "checking: {:?}",
        row(&world, change).change.state
    );
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(producer.task)));
    let ended = world.store.rows.get(&key).expect("producer ended").clone();
    amend(&mut world, &requester, change, CALL, Some(WORDS), None);
    assert_eq!(world.store.rows.get(&key), Some(&ended));
    assert!(live_task(&world, change).inbox.is_empty(), "the amendment step commits");
    assert!(matches!(live_task(&world, change).phase, tasks::Phase::Active(_)));
    assert!(matches!(row(&world, change).change.state, State::Checking { .. }));
    assert!(!world.assigned.iter().any(|run| run.task == producer.task && run.attempt > producer.attempt));
}
