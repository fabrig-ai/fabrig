//! Requester words reach the live producer without answering it
//! (temper/docs/design/domain/forge/change/questions.md, 1; draft/producer-reach.md, 3 and 5).
use super::questions::{
    answer, ask, call, finish_producing, make_change, next_producer, plan_world_with_limits, replies, row, up_world,
    wait_answer,
};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Stands, Waiting};

const TICKS: u32 = 200;
const CALL: u64 = 100;
const WORDS: &[u8] = b"Go on: 3541 landed";

fn send(world: &mut World, requester: &engine::Assignment, change: u64) {
    call(
        world,
        requester,
        CALL,
        engine::Tool::Message { target: change, form: engine::MessageForm::Words, words: WORDS.into() },
    );
}

fn next_attempt(world: &mut World, producer: &engine::Assignment) -> engine::Assignment {
    for _ in 0..TICKS {
        world.tick();
        if let Some(run) = world.assigned.iter().find(|run| run.task == producer.task && run.attempt > producer.attempt)
        {
            return run.clone();
        }
    }
    panic!("the same producer wakes: {:?}", live_task(world, producer.task));
}

fn passed(world: &World, requester: u64, change: u64, producer: u64) -> tasks::Word {
    let expected = format!(
        "From task {requester}, your change's requester:\n\n{}",
        core::str::from_utf8(WORDS).expect("story UTF-8")
    );
    let found: Vec<_> = replies(world, producer, change)
        .into_iter()
        .filter(|word| word.kind == tasks::MessageKind::Words && word.words.as_ref() == expected.as_bytes())
        .collect();
    assert_eq!(found.len(), 1, "words passed exactly once");
    found[0].clone()
}

fn told(world: &World, requester: u64, change: u64, producer: u64) -> tasks::Word {
    let expected = format!("Passed to task {producer}, this change's producer.");
    let found: Vec<_> = replies(world, requester, change)
        .into_iter()
        .filter(|word| word.kind == tasks::MessageKind::Words && word.words.starts_with(expected.as_bytes()))
        .collect();
    assert_eq!(found.len(), 1, "requester told exactly once");
    found[0].clone()
}

fn settle_words(world: &mut World, requester: u64, change: u64) {
    for _ in 0..TICKS {
        world.tick();
        if row(world, change).questions.waiting.is_empty()
            && replies(world, requester, change).iter().any(|word| word.kind == tasks::MessageKind::Words)
            && world.store.pending.is_empty()
        {
            return;
        }
    }
    panic!("words passed and requester told: {:?}", row(world, change).questions);
}

#[test]
fn words_wake_the_same_parked_producer_after_its_question_was_answered() {
    let (mut world, requester, change, producer) = up_world(None);
    ask(&mut world, &producer, change);
    park_after_turn(&mut world, &producer, None, 0);
    answer(&mut world, &requester, change, CALL + 1);
    wait_answer(&mut world, producer.task, change);
    let resumed = next_attempt(&mut world, &producer);
    let subscribed = world.scripted_call(
        &resumed,
        CALL + 2,
        engine::Tool::Subscribe {
            kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
        },
    );
    assert!(matches!(subscribed, engine::CallAnswer::Subscribed { .. }));
    assert!(live_task(&world, producer.task).inbox.is_empty());
    let turns = live_task(&world, producer.task).turn;
    park_after_turn(&mut world, &resumed, None, turns);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert!(live_task(&world, producer.task).questions.is_empty());
    send(&mut world, &requester, change);
    settle_words(&mut world, requester.task, change);
    let message = passed(&world, requester.task, change, producer.task);
    let reply = told(&world, requester.task, change, producer.task);
    assert_eq!(reply.words.as_ref(), format!("Passed to task {}, this change's producer.", producer.task).as_bytes());
    let resumed = next_attempt(&mut world, &resumed);
    assert!(resumed.messages.contains(&message));
    assert_eq!(row(&world, change).delegate, Some((producer.task, Delegate::Produce)));
}

#[test]
fn words_leave_the_question_asked_and_are_offered_with_its_answer() {
    let (mut world, requester) =
        plan_world_with_limits(None, &StoryLimits { batch: Some(2), ..StoryLimits::default() }, false);
    let change = make_change(&mut world, &requester, CALL - 1, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    let question = ask(&mut world, &producer, change);
    park_after_turn(&mut world, &producer, None, 0);
    let open = live_task(&world, producer.task).questions.clone();
    send(&mut world, &requester, change);
    settle_words(&mut world, requester.task, change);
    let message = passed(&world, requester.task, change, producer.task);
    let reply = told(&world, requester.task, change, producer.task);
    assert_eq!(reply.words.as_ref(), format!("Passed to task {}, this change's producer. Its question still waits, and words do not answer it: Which behavior should hold?. Answer it with message form=answer; read_task on this change gives the question's number.", producer.task).as_bytes());
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(live_task(&world, producer.task).questions, open);
    for _ in 0..TICKS {
        world.tick();
    }
    assert!(!world.assigned.iter().any(|run| run.task == producer.task && run.attempt > producer.attempt));
    answer(&mut world, &requester, change, CALL + 1);
    wait_answer(&mut world, producer.task, change);
    let resumed = next_attempt(&mut world, &producer);
    assert!(resumed.messages.contains(&message));
    assert!(resumed.messages.iter().any(|word| word.kind == tasks::MessageKind::Answer { question }));
}

#[test]
fn route_none_passes_its_requesters_words_too() {
    let (mut world, requester) = questions::plan_world(None);
    let change = make_change(&mut world, &requester, CALL - 1, engine::ChangeQuestions::None);
    let producer = next_producer(&mut world, change);
    send(&mut world, &requester, change);
    settle_words(&mut world, requester.task, change);
    passed(&world, requester.task, change, producer.task);
    told(&world, requester.task, change, producer.task);
}

#[test]
fn words_without_a_live_producer_keep_the_routes_fallback() {
    for route in [engine::ChangeQuestions::None, engine::ChangeQuestions::Up] {
        let (mut world, requester) = plan_world_with_limits(None, &StoryLimits::default(), true);
        let change = make_change(&mut world, &requester, CALL - 1, route.clone());
        let producer = next_producer(&mut world, change);
        finish_producing(&mut world, change, &producer);
        send(&mut world, &requester, change);
        world.until(Until::Ready);
        let found = replies(&world, requester.task, change);
        assert_eq!(found.len(), 1);
        let text = core::str::from_utf8(&found[0].words).expect("reply UTF-8");
        match route {
            engine::ChangeQuestions::None => assert_eq!(
                text,
                format!("A change takes no questions, answers or words; its requester is task {}.", requester.task)
            ),
            engine::ChangeQuestions::Up => assert!(text.starts_with("Relays questions"), "{text}"),
        }
        assert!(row(&world, change).questions.waiting.is_empty());
        assert_ne!(row(&world, change).delegate, Some((producer.task, Delegate::Produce)));
    }
}

#[test]
fn a_restart_passes_waiting_words_and_tells_each_once() {
    for already_passed in [false, true] {
        let (mut world, requester, change, producer) = up_world(None);
        ask(&mut world, &producer, change);
        park_after_turn(&mut world, &producer, None, 0);
        world.drain_cut();
        let mut saved = row(&world, change);
        if already_passed {
            // The first effect committed at a batch-one step; its reply has not.
            let mut target = Box::new(live_task(&world, producer.task).clone());
            let mut header = world.store.header();
            header.messages += 1;
            let mut inbox = target.inbox.into_vec();
            inbox.push(tasks::Word {
                number: header.messages,
                from: tasks::Party::Task(change),
                kind: tasks::MessageKind::Words,
                words: format!(
                    "From task {}, your change's requester:\n\n{}",
                    requester.task,
                    core::str::from_utf8(WORDS).expect("story UTF-8")
                )
                .as_bytes()
                .into(),
                at: world.env.wall,
                hits: 1,
                eligible: true,
            });
            target.inbox = inbox.into_boxed_slice();
            target.last_message = header.messages;
            world.store.seed(
                Box::new([
                    Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(target))),
                    Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))),
                ]),
                Box::new([]),
            );
        }
        saved.questions.waiting = Box::new([Waiting {
            from: requester.task,
            number: 1000,
            words: if already_passed { Box::new([]) } else { WORDS.into() },
            passed: already_passed.then_some(producer.task),
        }]);
        world.store.seed(
            Box::new([Record::Forge { row: Box::new(forge_top::Stored::Change(Box::new(saved))) }]),
            Box::new([]),
        );
        world.restart(true, true);
        settle_words(&mut world, requester.task, change);
        for _ in 0..TICKS {
            world.tick();
        }
        passed(&world, requester.task, change, producer.task);
        told(&world, requester.task, change, producer.task);
    }
}

#[test]
fn a_batch_of_one_keeps_words_taken_without_room_then_passes_before_telling() {
    let (mut world, requester, change, producer) = up_world(None);
    ask(&mut world, &producer, change);
    park_after_turn(&mut world, &producer, None, 0);
    world.drain_cut();
    let mut task = Box::new(live_task(&world, change).clone());
    let mut header = world.store.header();
    let mut inbox = task.inbox.into_vec();
    // A fallback reply spends this step's one message before passing begins.
    for from in [producer.task, requester.task] {
        header.messages += 1;
        inbox.push(tasks::Word {
            number: header.messages,
            from: tasks::Party::Task(from),
            kind: tasks::MessageKind::Words,
            words: WORDS.into(),
            at: world.env.wall,
            hits: 1,
            eligible: true,
        });
    }
    task.inbox = inbox.into_boxed_slice();
    task.last_message = header.messages;
    task.phase = tasks::Phase::Active(tasks::Active::Due);
    world.store.seed(
        Box::new([
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))),
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header))),
        ]),
        Box::new([]),
    );
    world.restart(true, true);
    let mut retained = false;
    let mut sent = false;
    for _ in 0..TICKS {
        world.tick();
        let waiting = row(&world, change).questions.waiting;
        if waiting.first().is_some_and(|word| word.passed.is_none() && word.words.as_ref() == WORDS) {
            retained = true;
        }
        if waiting.first().is_some_and(|word| word.passed == Some(producer.task) && word.words.is_empty()) {
            sent = true;
        }
        if retained && sent && waiting.is_empty() && world.store.pending.is_empty() {
            break;
        }
    }
    assert!(retained, "words taken without message room stay on the record");
    assert!(sent, "a batch-one step passes before the next tells");
    passed(&world, requester.task, change, producer.task);
    told(&world, requester.task, change, producer.task);
}

#[test]
fn a_full_producer_inbox_holds_words_until_release_then_passes_once() {
    let (mut world, requester, change, producer) = up_world(None);
    questions::fill_requester_inbox(&mut world, &producer);
    send(&mut world, &requester, change);
    for _ in 0..TICKS {
        world.tick();
    }
    assert!(
        matches!(live_task(&world, change).phase, tasks::Phase::Held { why: tasks::Hold::EffectFailed, .. }),
        "{:?}",
        live_task(&world, change).phase
    );
    let waiting = &row(&world, change).questions.waiting;
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].words.as_ref(), WORDS);
    assert_eq!(waiting[0].passed, None);
    assert_eq!(replies(&world, requester.task, change).len(), 0);
    let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(producer.task))))
    else {
        panic!("producer proof")
    };
    let read = proof.offered.expect("full inbox offered");
    let turn = live_task(&world, producer.task).turn + 1;
    world.send(protocol::turn(
        Token::new(7),
        producer.task,
        producer.attempt,
        turn,
        0,
        Some(read),
        b"Read pending instructions",
    ));
    world.until(Until::Ready);
    let revision = live_task(&world, change).revisions.hold;
    call(&mut world, &requester, CALL + 1, engine::Tool::Release { target: change, revision, again: None });
    settle_words(&mut world, requester.task, change);
    for _ in 0..TICKS {
        world.tick();
    }
    passed(&world, requester.task, change, producer.task);
    told(&world, requester.task, change, producer.task);
    assert_eq!(row(&world, change).delegate, Some((producer.task, Delegate::Produce)));
}
