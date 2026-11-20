//! Directing words survive a committed read (domain/tasks/messages.md, 2).
use jig_domain_tasks::{self as tasks, *};
use jig_tasks_world::{LIMITS, PERSON_PARTY, Reply, World, response, task};
use skein_lib::Wall;

const TASK: u64 = 12;
const WATCHED: u64 = 13;
const MESSAGE: u64 = 30;

fn goal() -> World {
    goal_with(task(TASK, &[]), &LIMITS, false)
}

fn goal_with(mut parent: New, limits: &Limits, resumes: bool) -> World {
    let mut world = World::new(9236, *limits);
    if resumes {
        let mut charters = jig_tasks_world::charters();
        charters[0].resumes = true;
        world.send(Event::Charters { newest: 1, charters, efforts: Box::new(jig_tasks_world::EFFORTS) });
    }
    parent.numbers.budget = 1_000;
    parent.authority.budget.spend = 1_000;
    assert_eq!(world.make(PERSON_PARTY, vec![parent]), Reply::Made(vec![TASK]));
    let mut child = task(WATCHED, &[]);
    child.funder = Funder::Task(TASK);
    assert_eq!(world.make(Party::Task(TASK), vec![child]), Reply::Made(vec![WATCHED]));
    world
}

fn say(world: &mut World, from: Party, target: u64, number: u64, kind: MessageKind, text: &[u8]) -> Reply {
    response(world, |reply_to| Event::AuthorizedMessage {
        reply_to,
        project: 1,
        operation: tasks::Message {
            task: target,
            word: Word { number, from, kind, words: text.into(), at: Wall::EPOCH, hits: 1, eligible: false },
        },
    })
}

fn words(world: &mut World, number: u64) {
    assert_eq!(say(world, Party::Person(7), TASK, number, MessageKind::Words, b"keep this request"), Reply::Done);
}

fn record_words(world: &mut World, messages: &[u64]) -> Reply {
    response(world, |reply_to| Event::RecordWords { reply_to, task: TASK, messages: messages.into() })
}

fn read_inbox(world: &mut World, attempt: u64) {
    let read = world.record(TASK).last_message;
    let outstanding = world.record(TASK).inbox.iter().map(|word| word.number).collect();
    assert_eq!(
        response(world, |reply_to| Event::Turn {
            reply_to,
            task: TASK,
            attempt,
            turn: 1,
            read: Some(read),
            offered: Some(read),
            outstanding,
            refused: Box::new([]),
            cumulative: 0,
        }),
        Reply::Turn(Accepted::New)
    );
}

#[test]
fn a_read_persons_words_wake_a_goal_without_spending_a_try() {
    let mut world = goal();
    world.claim(TASK, 1);
    let tries = world.record(TASK).tries;
    words(&mut world, MESSAGE);
    read_inbox(&mut world, 1);
    assert!(world.record(TASK).inbox.is_empty());
    assert_eq!(world.record(TASK).unrecorded.iter().map(|word| word.number).collect::<Vec<_>>(), [MESSAGE]);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(TASK).tries, tries);
    assert_eq!(world.record(TASK).words_woken, MESSAGE);
    assert!(world.run_context(TASK).reasons.contains(Reasons::WORDS));
    assert!(world.contexts[&TASK].reasons.contains(Reasons::WORDS));
    world.claim(TASK, 2);
}

#[test]
fn unrecorded_words_wake_once_then_only_newer_words_wake_again() {
    let mut world = goal();
    world.claim(TASK, 1);
    words(&mut world, MESSAGE);
    read_inbox(&mut world, 1);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Due));
    world.claim(TASK, 2);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Idle));
    assert_eq!(world.record(TASK).tries, Tries::NONE);
    assert_eq!(world.record(TASK).words_woken, MESSAGE);
    words(&mut world, MESSAGE + 1);
    world.claim(TASK, 3);
    read_inbox(&mut world, 3);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(TASK).words_woken, MESSAGE + 1);
    assert_eq!(world.record(TASK).tries, Tries::NONE);
}

#[test]
fn recording_words_removes_only_named_kept_messages_and_reports_unknown_numbers() {
    let mut world = goal();
    world.claim(TASK, 1);
    words(&mut world, MESSAGE);
    words(&mut world, MESSAGE + 1);
    read_inbox(&mut world, 1);
    assert_eq!(
        record_words(&mut world, &[MESSAGE, 99]),
        Reply::WordsRecorded { recorded: Box::new([MESSAGE]), unknown: Box::new([99]) }
    );
    assert_eq!(world.record(TASK).unrecorded.iter().map(|word| word.number).collect::<Vec<_>>(), [MESSAGE + 1]);
    assert_eq!(
        record_words(&mut world, &[MESSAGE, MESSAGE + 1]),
        Reply::WordsRecorded { recorded: Box::new([MESSAGE + 1]), unknown: Box::new([MESSAGE]) }
    );
    world.terminal(TASK, End::Parked);
    assert!(world.record(TASK).unrecorded.is_empty());
    assert_eq!(world.record(TASK).words_woken, 0);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Idle));
    assert!(!world.run_context(TASK).reasons.contains(Reasons::WORDS));
}

#[test]
fn only_person_and_requester_words_are_kept_and_questions_stay_credits() {
    const REQUESTER: u64 = 11;
    const PEER: u64 = 14;
    let mut world = World::new(9236, LIMITS);
    let mut requester = task(REQUESTER, &[]);
    requester.numbers.budget = 2_000;
    requester.authority.budget.spend = 2_000;
    world.make(PERSON_PARTY, vec![requester]);
    let mut peer = task(PEER, &[]);
    peer.funder = Funder::Task(REQUESTER);
    world.make(Party::Task(REQUESTER), vec![peer]);
    let mut member = task(TASK, &[]);
    member.numbers.budget = 1_000;
    member.authority.budget.spend = 1_000;
    member.funder = Funder::Task(REQUESTER);
    world.make(Party::Task(REQUESTER), vec![member]);
    let mut child = task(WATCHED, &[]);
    child.funder = Funder::Task(TASK);
    world.make(Party::Task(TASK), vec![child]);
    assert_eq!(
        response(&mut world, |reply_to| Event::Introduce { reply_to, by: REQUESTER, left: TASK, right: PEER }),
        Reply::Done
    );
    world.claim(TASK, 1);
    for (number, from, kind) in [
        (30, Party::Person(7), MessageKind::Words),
        (31, Party::Task(REQUESTER), MessageKind::Words),
        (32, Party::Task(WATCHED), MessageKind::Words),
        (33, Party::Task(PEER), MessageKind::Words),
        (34, Party::Task(TASK), MessageKind::Question),
    ] {
        let target = if number == 34 { WATCHED } else { TASK };
        assert_eq!(say(&mut world, from, target, number, kind, b"direct or news?"), Reply::Done);
    }
    // A delegate's question is not a kept word, but its question position remains open.
    assert_eq!(say(&mut world, Party::Task(WATCHED), TASK, 35, MessageKind::Question, b"answer?"), Reply::Done);
    read_inbox(&mut world, 1);
    assert_eq!(world.record(TASK).unrecorded.iter().map(|word| word.number).collect::<Vec<_>>(), [30, 31]);
    assert_eq!(world.record(TASK).questions[0].number, 34);
    assert_eq!(world.record(TASK).question_lines[0].message, 35);
    assert!(world.record(TASK).question_lines[0].answer.is_none());
    assert!(world.record(TASK).inbox.is_empty());
}

#[derive(Clone, Copy, Debug)]
enum Exempt {
    Conversation,
    Standing,
    RequesterEnded,
    Resuming,
}

#[test]
fn conversations_standing_requester_ended_and_resuming_tasks_keep_no_words() {
    for exemption in [Exempt::Conversation, Exempt::Standing, Exempt::RequesterEnded, Exempt::Resuming] {
        let mut member = task(TASK, &[]);
        match exemption {
            Exempt::Conversation => member.flags.conversation = true,
            Exempt::Standing => member.flags.ended_by = EndedBy::Standing,
            Exempt::RequesterEnded => {
                member.flags.conversation = true;
                member.flags.ended_by = EndedBy::Requester;
            }
            Exempt::Resuming => {}
        }
        let mut world = goal_with(member, &LIMITS, matches!(exemption, Exempt::Resuming));
        world.claim(TASK, 1);
        words(&mut world, MESSAGE);
        read_inbox(&mut world, 1);
        assert!(world.record(TASK).unrecorded.is_empty(), "{exemption:?}");
        world.terminal(TASK, End::Parked);
        assert_eq!(world.record(TASK).phase, Phase::Active(Active::Idle), "{exemption:?}");
        assert_eq!(world.record(TASK).words_woken, 0);
    }
}

#[test]
fn unrecorded_words_share_count_and_byte_room_but_leave_a_delegate_its_result() {
    for limits in [
        Limits { inbox_messages: 3, run_inbox: 3, ..LIMITS },
        Limits { inbox_bytes: LIMITS.result_bytes + 2 * 17, ..LIMITS },
    ] {
        let mut world = goal_with(task(TASK, &[]), &limits, false);
        world.claim(TASK, 1);
        words(&mut world, MESSAGE);
        words(&mut world, MESSAGE + 1);
        read_inbox(&mut world, 1);
        let reply = say(&mut world, PERSON_PARTY, TASK, MESSAGE + 2, MessageKind::Words, b"x");
        let Reply::Refused(problem) = reply else { panic!("full inbox refuses words: {reply:?}") };
        assert_eq!(problem.why, Refusal::Busy);
        assert!(problem.capacity.is_some());
        world.claim(WATCHED, 2);
        world.finish(WATCHED);
        world.settle(WATCHED);
        assert!(world.record(TASK).inbox.iter().any(|word| matches!(word.kind, MessageKind::Result(_))));
        assert_eq!(world.record(TASK).unrecorded.len(), 2);
        world.restart();
    }
}

#[test]
fn unrecorded_words_run_before_a_closer_then_the_next_park_makes_it() {
    let mut world = World::new(9236, LIMITS);
    let mut member = task(TASK, &[]);
    member.flags.ended_by = EndedBy::Closer(Closer { connector: 1, code: 6 });
    member.authority.delegation.kinds = Box::new([AuthorityExecutor::Procedure(6)]);
    world.make(PERSON_PARTY, vec![member]);
    world.claim(TASK, 1);
    words(&mut world, MESSAGE);
    read_inbox(&mut world, 1);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Due));
    assert_eq!(world.record(TASK).words_woken, MESSAGE);
    assert_eq!(world.record(TASK).closer, None);
    assert_eq!(world.record(TASK).last_message, MESSAGE);
    assert!(world.record(TASK).inbox.is_empty());
    world.claim(TASK, 2);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Idle));
    assert!(world.record(TASK).closer.is_some());
    assert!(world.record(TASK).inbox.iter().all(|word| word.kind != MessageKind::NothingAwaited));
    assert_eq!(world.record(TASK).tries, Tries::NONE);
}

#[test]
fn unrecorded_words_and_their_wake_watermark_survive_restart_and_the_tasks_codec() {
    use jig_protocol_store_records::codec::tasks as codec;
    let mut world = goal();
    world.claim(TASK, 1);
    words(&mut world, MESSAGE);
    read_inbox(&mut world, 1);
    world.terminal(TASK, End::Parked);
    let before = world.record(TASK).clone();
    assert_eq!(before.words_woken, MESSAGE);
    assert_eq!(before.unrecorded.len(), 1);
    world.restart();
    assert_eq!(world.record(TASK).unrecorded, before.unrecorded);
    assert_eq!(world.record(TASK).words_woken, before.words_woken);
    let limits = codec::limits(&LIMITS).expect("world limits");
    let cold = Stored::LiveCold {
        number: TASK,
        spec: Box::new(before.spec.clone()),
        contract: before.contract.clone(),
        authority: before.authority.clone(),
        instructions: before.instructions.clone(),
    };
    let cold = codec::encode(&cold, &limits, &[]).expect("cold record");
    let live = Stored::Live(Box::new(before.clone()));
    let hot = codec::encode(&live, &limits, &[]).expect("hot record");
    assert_eq!(codec::decode_live(&hot, &cold, &limits, &[]), Ok(live));
    // BeforeTalk is also used by retained records without a talk.
    let ended = Stored::Ended(Box::new(before));
    let bytes = codec::encode(&ended, &limits, &[]).expect("whole record");
    assert_eq!(codec::decode(&bytes, &limits, &[]), Ok(ended));
    world.claim(TASK, 2);
    world.terminal(TASK, End::Parked);
    assert_eq!(world.record(TASK).phase, Phase::Active(Active::Idle));
}
