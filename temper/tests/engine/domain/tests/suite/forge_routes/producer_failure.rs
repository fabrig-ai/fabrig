//! Explicit no-change completion and production failures in the engine domain world.
use super::*;
use jig_domain_core::reads;
use temper_engine_domain_forge_change::{Hold, State};

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 200;
const REASON: &[u8] = b"the repository has no search UI: it is a CSV command-line program; no files changed";

pub(super) fn failed_production(reason: &[u8]) -> (World, engine::Assignment, u64) {
    production(tasks::TaskResult::Failure { reason: reason.into() })
}

fn production(result: tasks::TaskResult) -> (World, engine::Assignment, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { result_bytes: Some(tasks::HOLD_ACCOUNT_BYTES), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let ask = world.chat(1, b"Fix widget's empty search state");
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) =
        super::change_tool::change(&mut world, &chat, 93, None, b"Fix widget's empty search state")
    else {
        panic!("change delegated")
    };
    let producer = super::reviews::next_assignment(&mut world, 1);
    let event = world.protocol.answer(CHANNEL, chat.task, chat.attempt, 1, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
    let change = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change))))
            || matches!(live_task(&world, change).phase, tasks::Phase::Held { .. })
        {
            return (world, chat, change);
        }
    }
    panic!("production did not settle its change");
}

#[test]
fn an_inapplicable_change_finishes_without_a_hold() {
    let (mut world, chat, change) = production(tasks::TaskResult::NoChange { words: REASON.into() });
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change)));
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = world.store.rows.get(&key)
    else {
        panic!("no-change task ended")
    };
    assert!(matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { words }))
        if words.as_ref() == REASON));
    assert!(!world.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
        if matches!(task.escalation, tasks::Escalation::Waiting { .. }))));
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("durable change") };
    assert!(row.pull.is_none());
    assert_eq!(world.merges, 0);
    let resumed = super::reviews::next_assignment(&mut world, 2);
    assert_eq!(resumed.task, chat.task);
    let notice = resumed
        .messages
        .iter()
        .find(|word| {
            word.from == tasks::Party::Task(change)
                && word.kind == tasks::MessageKind::Result(tasks::ResultKind::NoChange)
        })
        .expect("requester hears the successful no-change result");
    assert_eq!(notice.words.as_ref(), REASON);
    let read = resumed.messages.iter().map(|word| word.number).max();
    let reply = b"No changes were made: the repository has no search UI.";
    world.send(protocol::turn(CHANNEL, resumed.task, resumed.attempt, 1, 1, read, reply));
    for _ in 0..STEPS {
        world.tick();
    }
    let reads::Answer::Ended(past) =
        world.party_read(reads::Read::Turn { task: chat.task, attempt: resumed.attempt, turn: 1 })
    else {
        panic!("person reads the reply")
    };
    let reads::EndedValue::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply);
}

fn account(world: &World, change: u64) -> &[u8] {
    let tasks::Phase::Held { why: tasks::Hold::EffectFailed, source: Some(source), .. } =
        &live_task(world, change).phase
    else {
        panic!("retained failure hold")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("producer's account") };
    words
}

#[test]
fn a_failed_producers_words_reach_its_hold_and_its_chats_reply() {
    let (mut world, chat, change) = failed_production(REASON);
    assert_eq!(account(&world, change), REASON);
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("durable change") };
    assert!(matches!(&row.change.state, State::Held { was, why: Hold::Failed }
        if matches!(was.as_ref(), State::Producing { .. })));
    assert!(row.pull.is_none());
    assert_eq!(world.merges, 0);
    let resumed = super::reviews::next_assignment(&mut world, 2);
    assert_eq!(resumed.task, chat.task);
    let notice = resumed
        .messages
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Words)
        .expect("chat hears its change's reason");
    assert_eq!(notice.words.as_ref(), REASON);
    // The scripted worker plays the chat's response to the words it received.
    let reply = format!("I could not make the change: {}", core::str::from_utf8(&notice.words).expect("explanation"));
    let read = resumed.messages.iter().map(|word| word.number).max();
    world.send(protocol::turn(CHANNEL, resumed.task, resumed.attempt, 1, 1, read, reply.as_bytes()));
    for _ in 0..STEPS {
        world.tick();
    }
    let reads::Answer::Ended(past) =
        world.party_read(reads::Read::Turn { task: resumed.task, attempt: resumed.attempt, turn: 1 })
    else {
        panic!("person reads the chat's reply")
    };
    let reads::EndedValue::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply.as_bytes());
    assert!(turn.transcript.windows(REASON.len()).any(|bytes| bytes == REASON));
}

#[test]
fn a_producer_failure_uses_the_hold_account_bound_even_with_a_smaller_message_bound() {
    let bound = usize::try_from(tasks::HOLD_ACCOUNT_BYTES).expect("hold bound");
    let mut reason = vec![b'x'; bound];
    reason[..REASON.len()].copy_from_slice(REASON);
    let (world, chat, change) = failed_production(&reason);
    assert_eq!(world.env.limits.tasks.message_bytes, 128);
    assert_eq!(account(&world, change), reason, "a whole hold account is never cut to the inbox bound");
    let notice = live_task(&world, chat.task)
        .inbox
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Words)
        .expect("requester notified");
    assert_eq!(notice.words.len(), usize::try_from(world.env.limits.tasks.message_bytes).expect("message bound"));
    assert!(notice.words.starts_with(REASON));
    assert!(notice.words.ends_with(b"..."));
}
