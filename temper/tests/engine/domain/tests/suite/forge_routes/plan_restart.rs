//! The plan face's landing tail below the process boundary (temper/docs/design/testing/process-tier.md).
use super::*;
use crate::accounts_restart::{connect_plan, plan_provider, plan_sign_in, plan_source};
use jig_domain_core::{documents, reads};
use jig_domain_link::{Form, Scheme};
use temper_engine_domain_world::tier_runs::{LANDING_TICKS, restart_cut, task, until};

const CHANNEL: Token = Token::new(7);
const LANDED: &[u8] = b"The change landed.";
const REOPENED: &[u8] = b"The change is still on main.";
const CLOSED: &[u8] = b"The chat is now closed.";

#[test]
fn a_landed_chats_reply_is_committed_once_and_nothing_reruns_after_a_restart() {
    let mut world = World::configured(true, true).with_personal_plan(plan_provider());
    world.adopt();
    let source = connect(&mut world);
    let (chat, change, producer) = land(&mut world, source);
    let reply = reviews::next_assignment(&mut world, 2);
    assert_eq!(reply.task, chat);
    assert_plan_grant(&reply, source);
    assert!(reply.messages.iter().any(|word| word.from == tasks::Party::Task(change)
        && matches!(word.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))));
    let read = reply.messages.last().map(|word| word.number);
    world.send(protocol::turn(CHANNEL, chat, reply.attempt, 1, 0, read, LANDED));
    park_after_turn(&mut world, &reply, read, 1);
    world.oracle_settle();
    assert_eq!(task(&world, chat).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_replies(&mut world, chat, &[LANDED]);
    assert_delegates(&mut world, chat, change, producer);
    let before = world.assigned.len();
    let writes = world.writes.len();
    assert_eq!(before, 3, "chat, producer and resumed chat only");
    assert!(restart_cut(&world));
    world.restart_same();
    world.signed_in = None;
    world.sign_in();
    assert_delegates(&mut world, chat, change, producer);
    assert_replies(&mut world, chat, &[LANDED]);
    assert_eq!(world.assigned.len(), before, "idle chat and ended delegates do not rerun");

    let continued = continue_chat(&mut world, chat, before, source);
    close_chat(&mut world, &continued);
    assert_replies(&mut world, chat, &[LANDED, REOPENED, CLOSED]);
    assert_delegates(&mut world, chat, change, producer);
    assert_eq!(world.assigned.len(), before + 1, "no producer or reviewer assigned after restart");
    assert_eq!(world.writes.len(), writes, "restart did not repeat the landing");
    assert_closed(&mut world, chat, change, producer);
    world.close();
}

fn connect(world: &mut World) -> u32 {
    world.send(connect_plan(world.signed_in.expect("owner's session")));
    world.oracle_settle();
    let source = plan_source(&world.protocol);
    world.protocol.sign_in_sources.insert(source);
    for event in plan_sign_in(source) {
        world.send(event);
        world.oracle_settle();
    }
    source
}

fn land(world: &mut World, source: u32) -> (u64, u64, u64) {
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Update the file") else { panic!("chat ask") };
    batch[0].flags.ended_by = people::EndedBy::Requester;
    let chat = assigned_chat(world, 92, people::Ask::Create { project, batch });
    assert_plan_grant(&chat, source);
    let engine::CallAnswer::Delegated(delegates) = change_tool::change(world, &chat, 93, None, b"Update the file")
    else {
        panic!("change delegated")
    };
    assert_eq!(delegates.len(), 1);
    let change = delegates[0].task;
    let producer = reviews::next_assignment(world, 1);
    assert_eq!(task(world, producer.task).requester, tasks::Party::Task(change));
    assert_plan_grant(&producer, source);
    park_after_turn(world, &chat, None, 0);
    let branch = producer.workspace.repositories[0].push.as_deref().expect("producer's branch");
    change_tool::deliver_change(world, &producer, branch, b"Small fix");
    until(world, LANDING_TICKS, "change ended with a landing", |world| {
        matches!(task(world, change).phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. })))
    });
    assert_eq!(world.merges, 1);
    (chat.task, change, producer.task)
}

fn assert_plan_grant(run: &engine::Assignment, source: u32) {
    let credentials = run.grants.iter().filter(|grant| matches!(grant.form, Form::Credential(_)));
    assert_eq!(
        credentials.map(|grant| (grant.account, grant.form)).collect::<Vec<_>>(),
        [(source, Form::Credential(Scheme::Bearer))],
        "the party's plan is the only model grant"
    );
}

fn continue_chat(world: &mut World, task: u64, index: usize, source: u32) -> engine::Assignment {
    ask(world, 94, people::Ask::Say { project: 1, task, words: b"Did it stay landed?".as_slice().into() });
    let run = reviews::next_assignment(world, index);
    assert_eq!(run.task, task);
    assert_plan_grant(&run, source);
    let read = run.messages.last().map(|word| word.number);
    world.send(protocol::turn(CHANNEL, task, run.attempt, 1, 0, read, REOPENED));
    world.oracle_settle();
    assert_replies(world, task, &[LANDED, REOPENED]);
    run
}

fn close_chat(world: &mut World, run: &engine::Assignment) {
    ask(
        world,
        95,
        people::Ask::Cancel {
            project: 1,
            task: run.task,
            reason: b"Close chat".as_slice().into(),
            grace: Some(Duration::from_secs(5)),
        },
    );
    until(world, LANDING_TICKS, "close asks the run to finish", |world| {
        matches!(task(world, run.task).phase, tasks::Phase::Closing(_))
    });
    let read = run.messages.last().map(|word| word.number);
    world.send(protocol::turn(CHANNEL, run.task, run.attempt, 2, 0, read, CLOSED));
    park_after_turn(world, run, read, 2);
    until(world, LANDING_TICKS, "chat ended", |world| {
        matches!(&task(world, run.task).phase,
            tasks::Phase::Ended(tasks::Ending::Cancelled { reason, result: None, .. }) if reason.as_ref() == b"Close chat")
    });
}

fn ask(world: &mut World, key: u8, ask: people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask,
    }));
}

fn assert_replies(world: &mut World, chat: u64, expected: &[&[u8]]) {
    let mut turns = world
        .store
        .rows
        .values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) = row
                && turn.task == chat
            {
                Some((turn.attempt, turn.turn))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    turns.sort_unstable();
    assert_eq!(turns.len(), expected.len(), "each reply was committed once");
    for ((attempt, turn), words) in turns.into_iter().zip(expected) {
        let reads::Answer::Ended(past) = world.party_read(reads::Read::Turn { task: chat, attempt, turn }) else {
            panic!("the party reads the committed turn {attempt}/{turn}")
        };
        let reads::EndedValue::Turn(turn) = past.value else { panic!("committed turn") };
        assert_eq!(turn.transcript.as_ref(), *words);
    }
}

fn assert_delegates(world: &mut World, chat: u64, change: u64, producer: u64) {
    for (parent, child) in [(chat, change), (change, producer)] {
        let reads::Answer::Page(page) = world.party_read(reads::Read::EndedDelegates { task: parent, from: None })
        else {
            panic!("ended delegates")
        };
        assert!(!page.cut && page.next.is_none());
        assert_eq!(page.rows.len(), 1);
        let Some(reads::PageRow::EndedDelegate(row)) = page.rows.get(0) else { panic!("ended delegate row") };
        assert_eq!(row.chip.number, child);
    }
}

fn assert_closed(world: &mut World, chat: u64, change: u64, producer: u64) {
    let query = documents::TaskQuery { project: Some(1), requester: None, ..jig_test_profile::fixtures::own_tasks() };
    let reads::Answer::Tasks(page) = world.party_read(reads::Read::Tasks { query, live: None, ended: None }) else {
        panic!("project task query")
    };
    assert!(page.live.is_empty() && page.live_next.is_none(), "no task left live");
    let mut ended = world
        .store
        .rows
        .keys()
        .filter_map(|key| {
            if let Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))) = key { Some(*task) } else { None }
        })
        .collect::<Vec<_>>();
    ended.sort_unstable();
    assert_eq!(ended, [chat, change, producer], "exact task tree, no reviewer or duplicate delegate");
    assert!(matches!(
        world.party_read(reads::Read::Task { task: producer + 1 }),
        reads::Answer::None(reads::Why::Unknown)
    ));
}
