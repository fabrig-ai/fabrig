//! A party's words reach the producer, or are refused (temper/docs/design/domain/forge/change/questions.md, 1).
use super::*;
use temper_engine_domain_forge_change::{Delegate, Repair};

const PROJECT: u32 = 1;
const OWNER: u64 = 1;
const MESSAGE_KEY: u8 = 130;
const STOP_KEY: u8 = 131;
const CHANGE_CALL: u64 = 93;
const WORDS: &[u8] = "Keep the café heading.\nLeave its examples unchanged.".as_bytes();
const FIRST_OPERATION: u32 = 0; // Compound positions are zero-based (jig's domain/people/acts.md, 1).
const REPAIR_TICKS: u32 = 1000; // The CI failure must reach the change and start its repair.

fn party(world: &mut World, key: u8, ask: people::Ask) -> people::Reply {
    let reply = world.web_replies.len();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(skein_lib::len_u64(reply))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask,
    }));
    world.until(Until::Ready);
    world.web_replies.get(reply).expect("party answered").clone()
}

fn say(world: &mut World, task: u64) -> people::Reply {
    party(world, MESSAGE_KEY, people::Ask::Say { project: PROJECT, task, words: WORDS.into() })
}

fn copies(world: &World) -> Vec<(u64, &tasks::Word)> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) => {
                Some(row)
            }
            Record::Core(_) | Record::Forge { .. } => None,
        })
        .flat_map(|row| row.inbox.iter().filter(|word| word.words.as_ref() == WORDS).map(|word| (row.number, word)))
        .collect()
}

fn delivered(world: &World, producer: u64, reply: &people::Reply) {
    let saved = copies(world);
    let [(task, message)] = saved.as_slice() else { panic!("one inbox copy of the whole words: {saved:?}") };
    assert_eq!(
        (*task, message.from, &message.kind),
        (producer, tasks::Party::Person(OWNER), &tasks::MessageKind::Words)
    );
    assert_eq!(reply, &people::Reply::Outcome(people::Outcome::Said { task: producer, message: message.number }));
}

pub(super) fn not_offered(world: &mut World, task: u64) {
    assert_eq!(say(world, task), people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotOffered)));
    assert_eq!(copies(world).as_slice(), &[]);
}

#[test]
fn party_words_reach_the_active_producer_as_person_words() {
    let (mut world, _, change, producer) = questions::up_world(None);
    let inbox = live_task(&world, change).inbox.clone();
    let reply = say(&mut world, change);
    delivered(&world, producer.task, &reply);
    assert_eq!(live_task(&world, change).inbox, inbox);
}

#[test]
fn party_words_reach_a_producer_held_while_active() {
    let (mut world, _, change, producer) = questions::up_world(None);
    let _stopped = party(
        &mut world,
        STOP_KEY,
        people::Ask::Stop { project: PROJECT, task: producer.task, reason: b"Parking the task".as_slice().into() },
    );
    let tasks::Phase::Held { was: tasks::Was::Active(_), .. } = live_task(&world, producer.task).phase else {
        panic!("producer held while active: {:?}", live_task(&world, producer.task).phase)
    };
    let reply = say(&mut world, change);
    delivered(&world, producer.task, &reply);
}

#[test]
fn a_replay_names_the_same_producer_and_message_once() {
    let (mut world, _, change, producer) = questions::up_world(None);
    let first = say(&mut world, change);
    let replay = say(&mut world, change);
    assert_eq!(replay, first);
    delivered(&world, producer.task, &replay);
}

#[test]
fn a_waiting_producer_does_not_take_party_words() {
    let (mut world, requester, change, producer) = questions::up_world(None);
    // Restore the producer as if it was created with a still-pending prerequisite.
    let mut waiting = live_task(&world, producer.task).clone();
    waiting.phase = tasks::Phase::Waiting;
    waiting.dependencies = Box::new([requester.task]);
    waiting.waiting_on = Box::new([requester.task]);
    world.store.seed(
        Box::new([Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(waiting))))]),
        Box::new([]),
    );
    world.restart_same();
    world.sign_in();
    let tasks::Phase::Waiting = live_task(&world, producer.task).phase else { panic!("producer waits") };
    not_offered(&mut world, change);
}

#[test]
fn a_change_refuses_party_words_after_its_producer_ends() {
    let (mut world, requester) = questions::plan_world_with_limits(None, &StoryLimits::default(), true);
    let change = questions::make_change(&mut world, &requester, CHANGE_CALL, engine::ChangeQuestions::Up);
    let producer = questions::next_producer(&mut world, change);
    questions::finish_producing(&mut world, change, &producer);
    not_offered(&mut world, change);
}

fn parent(world: &World, delegate: u64) -> u64 {
    let tasks::Party::Task(change) = live_task(world, delegate).requester else { panic!("change is its requester") };
    change
}

#[test]
fn a_review_does_not_take_its_changes_party_words() {
    let (mut world, reviewer, _) = reviews::review_world();
    let change = parent(&world, reviewer.task);
    let Some((_, Delegate::Gate { .. })) = questions::row(&world, change).delegate else { panic!("the review") };
    not_offered(&mut world, change);
}

#[test]
fn a_repair_does_not_take_its_changes_party_words() {
    let (mut world, _, producer, branch) = change_world(true, REPAIR_TICKS, None);
    let change = parent(&world, producer.task);
    reviews::produce(&mut world, &producer, &branch);
    let head = world.fake.branch(b"org/repo", &branch).expect("pushed head");
    let assigned = world.assigned.len();
    goal_landing::status(&mut world, head, raw::Check::Failed);
    reviews::next_assignment(&mut world, assigned);
    let Some((_, Delegate::Repair(Repair::Ci))) = questions::row(&world, change).delegate else { panic!("CI repair") };
    not_offered(&mut world, change);
}

#[test]
fn words_in_a_compound_request_are_not_rerouted() {
    let (mut world, _, change, _) = questions::up_world(None);
    let mut operations = skein_lib::List::uncapped();
    operations.push(people::Operation::Message { task: change, words: WORDS.into() }).expect("one operation");
    let reply = party(&mut world, MESSAGE_KEY, people::Ask::Compound { project: PROJECT, operations });
    assert_eq!(
        reply,
        people::Reply::Outcome(people::Outcome::CompoundRefused {
            at: FIRST_OPERATION,
            why: people::Refusal::NotOffered
        })
    );
    assert_eq!(copies(&world).as_slice(), &[]);
}
