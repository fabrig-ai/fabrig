//! A held change's requester replies to its person before parking (temper/docs/design/domain/forge.md, 8.4).
use super::*;
use jig_domain_core::reads;

const CHANNEL: Token = Token::new(7);
// The chat and its producer were assigned before the hold wakes the chat.
const CHAT_AND_PRODUCER: usize = 2;
const STEPS: u32 = 200; // The world's durable-round-trip allowance.
const REASON: &[u8] = b"the repository has no search UI; no files changed";

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn delegate_count(world: &World, task: u64) -> usize {
    world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
            if row.requester == tasks::Party::Task(task))
        })
        .count()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one race follows a hold before reads through release and landing")]
fn a_change_held_for_effects_before_its_producer_asks_its_producer_once_released() {
    let mut world = World::configured(true, true);
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) = super::change_tool::change(&mut world, &chat, 93, None, b"Small fix")
    else {
        panic!("change delegated")
    };
    let task = made[0].task;
    let mut first_read = None;
    for _ in 0..STEPS {
        if let Some(index) = world.events.iter().position(|event| {
            matches!(event,
            engine::Event::Forge(event) if matches!(event.as_ref(),
                forge_top::Event::Client(skein_forge_client::Event::Answered { .. })))
        }) {
            first_read = world.events.remove(index);
            break;
        }
        world.tick();
    }
    let first_read = first_read.expect("first read has not returned to the change");
    assert!(matches!(
        change_row(&world, task).change.state,
        temper_engine_domain_forge_change::State::Producing { requested: false }
    ));
    assert_eq!(delegate_count(&world, task), 0);
    let tasks::Executor::Procedure { connector, code } = live_task(&world, task).executor else {
        panic!("change procedure")
    };
    world.send(engine::Event::Party(engine::PartyInput::ProcedureStep {
        task,
        step: live_task(&world, task).attempt + 1,
        connector,
        code,
        decision: engine::ProcedureStep::new(engine::StepEnd::Hold(tasks::Hold::Effects)),
    }));
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, task).phase, tasks::Phase::Held { why: tasks::Hold::Effects, .. }) {
            break;
        }
    }
    assert!(matches!(live_task(&world, task).phase, tasks::Phase::Held { why: tasks::Hold::Effects, .. }));
    assert!(change_row(&world, task).delegate.is_none());
    assert_eq!(delegate_count(&world, task), 0);
    world.send(first_read);
    for _ in 0..STEPS {
        world.tick();
    }
    assert!(matches!(live_task(&world, task).phase, tasks::Phase::Held { why: tasks::Hold::Effects, .. }));
    assert!(change_row(&world, task).delegate.is_none());
    assert_eq!(delegate_count(&world, task), 0, "returning reads cannot make a producer while held");
    let revision = live_task(&world, task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [94; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
    for _ in 0..STEPS {
        world.tick();
        if delegate_count(&world, task) != 0 {
            break;
        }
    }
    assert_eq!(delegate_count(&world, task), 1, "release asks exactly one producer");
    let producer = super::reviews::next_assignment(&mut world, 1);
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(task));
    assert_eq!(
        change_row(&world, task).delegate,
        Some((producer.task, temper_engine_domain_forge_change::Delegate::Produce))
    );
    let branch = change_row(&world, task).branch.clone();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if matches!(change_row(&world, task).change.state, temper_engine_domain_forge_change::State::Landed { .. }) {
            break;
        }
    }
    assert!(matches!(change_row(&world, task).change.state, temper_engine_domain_forge_change::State::Landed { .. }));
    assert_eq!(world.merges, 1);
    assert_eq!(delegate_count(&world, task), 1, "landing keeps the single producer");
    world.close();
}

#[test]
fn a_chat_tells_its_person_why_its_change_holds_before_waiting() {
    let (mut world, chat, change) = super::producer_failure::failed_production(REASON);
    let resumed = super::reviews::next_assignment(&mut world, CHAT_AND_PRODUCER);
    assert_eq!(resumed.task, chat.task);
    let notice = resumed
        .messages
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Words)
        .expect("the hold wakes its requester with its words");
    assert_eq!(notice.words.as_ref(), REASON);
    // The scripted worker chooses to report the hold before waiting.
    let reply = format!(
        "The change holds: {}. You decide: release it to go on, or cancel it.",
        core::str::from_utf8(&notice.words).expect("hold words")
    );
    let read = resumed.messages.iter().map(|word| word.number).max();
    world.send(protocol::turn(CHANNEL, resumed.task, resumed.attempt, 1, 0, read, reply.as_bytes()));
    park_after_turn(&mut world, &resumed, read, 1);
    for _ in 0..STEPS {
        world.tick();
    }
    assert!(matches!(live_task(&world, chat.task).phase, tasks::Phase::Active(tasks::Active::Idle)));
    let reads::Answer::Ended(past) =
        world.party_read(reads::Read::Turn { task: chat.task, attempt: resumed.attempt, turn: 1 })
    else {
        panic!("person reads the reply after the chat waits")
    };
    let reads::EndedValue::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply.as_bytes());
    assert!(turn.transcript.windows(REASON.len()).any(|bytes| bytes == REASON));
}
