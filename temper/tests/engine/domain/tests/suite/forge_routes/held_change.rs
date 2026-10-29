//! A held change's requester replies to its person before parking (temper/docs/design/domain/forge.md, 8.4).
use super::*;
use jig_domain_core::reads;

const CHANNEL: Token = Token::new(7);
// The chat and its producer were assigned before the hold wakes the chat.
const CHAT_AND_PRODUCER: usize = 2;
const STEPS: u32 = 200; // The world's durable-round-trip allowance.
const REASON: &[u8] = b"the repository has no search UI; no files changed";

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
    let reads::Answer::Historical(past) =
        world.party_read(reads::Read::Turn { task: chat.task, attempt: resumed.attempt, turn: 1 })
    else {
        panic!("person reads the reply after the chat waits")
    };
    let reads::Past::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply.as_bytes());
    assert!(turn.transcript.windows(REASON.len()).any(|bytes| bytes == REASON));
}
