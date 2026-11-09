//! Independent M1 checks of temper's transcript-load translation.
//! domain/root.md 7.1, 8 and 9; jig's domain/engine.md 7.2.
//! The public driver supplies already rendered text, not smith's renderer.
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{
    direct::{Driver, say_ask},
    protocol,
    store::Store,
    walking::{config, limits},
};

const CHANNEL: Token = Token::new(7);
const FIRST_TURN: u32 = 1;
const SPENT: u64 = 1;
const OLD_TEXT: &[u8] = b"text from the retired conversation";
// say_ask uses each byte as both an opaque key and UTF-8 words.
const FIRST_WAKE: u8 = b'A';
// The direct driver's own public pump/settle bound is 200 iterations.
const STEPS: usize = 200;

fn proof_key(task: u64) -> engine::Key {
    engine::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)))
}

fn pending_reply(driver: &mut Driver) -> engine::Event {
    for _ in 0..STEPS {
        let found = driver.events.iter().position(|event| {
            if let engine::Event::Store(engine::StoreInput::Loaded { owner, .. }) = event {
                driver.loads.iter().any(|(issued, range)| issued == owner && transcript_task(range).is_some())
            } else {
                false
            }
        });
        if let Some(index) = found {
            return driver.events.remove(index).expect("selected issued transcript reply");
        }
        driver.advance(true);
    }
    panic!("the waking task never issued its transcript load");
}

fn waiting(closed: bool) -> (Driver, engine::Assignment, u64, engine::Event) {
    let (mut driver, first) = Driver::open_chat();
    let session = driver.session();
    driver.send(protocol::turn(CHANNEL, first.task, first.attempt, FIRST_TURN, SPENT, None, OLD_TEXT));
    driver.settle();
    driver.answer(CHANNEL, first.task, first.attempt, SPENT, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    // Seed the store-visible result of closing, using the public typed seam.
    // Jig's separate store-world cases exercise the real closing walk.
    let mut row = driver.store.rows.get(&proof_key(first.task)).expect("parked proof").clone();
    let engine::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) = &mut row else {
        panic!("typed run proof")
    };
    proof.window.as_mut().expect("committed first turn opens a window").closed = closed;
    driver.store.seed(Box::new([row]), Box::new([]));
    let mut driver = Driver::new(driver.take_store());
    driver.hello(CHANNEL);
    driver.settle();
    driver.send(say_ask(session, first.task, FIRST_WAKE));
    let reply = pending_reply(&mut driver);
    (driver, first, session, reply)
}

fn fresh(assignment: &engine::Assignment) {
    assert!(!assignment.run.policy.resume);
    assert!(assignment.transcript.is_empty());
    assert_eq!(assignment.conversation, assignment.attempt);
}

fn no_failure_charged(driver: &Driver, task: u64) {
    let row = driver.task(task).expect("live chat");
    let tries = &row.tries;
    assert_eq!(
        (tries.transient, tries.permanent, tries.run, tries.agent, tries.lost, tries.invalid),
        (0, 0, 0, 0, 0, 0)
    );
    assert_eq!(row.refusals, 0);
}

fn tail(assignment: &engine::Assignment) -> Option<&[u8]> {
    assignment.sections.iter().find_map(|section| {
        if section.kind == engine::BriefKind::Core(jig_domain_brief::Core::TranscriptTail) {
            if let engine::BriefBody::Text(text) = &section.body {
                Some(text.as_ref())
            } else {
                panic!("a transcript tail is text")
            }
        } else {
            None
        }
    })
}

#[test]
fn a_closed_window_supplies_text_to_its_new_conversation() {
    let (mut driver, first, _, reply) = waiting(true);
    driver.send(reply);
    driver.settle();
    let assignment = driver.last_assignment();
    assert_eq!(assignment.task, first.task);
    assert_eq!(assignment.attempt, first.attempt + 1);
    fresh(&assignment);
    assert_eq!(tail(&assignment), Some(OLD_TEXT));
    no_failure_charged(&driver, first.task);
}

#[test]
fn omitted_rendered_text_leaves_a_closed_tail_empty() {
    let (mut driver, first, _, mut reply) = waiting(true);
    let engine::Event::Store(engine::StoreInput::Loaded { texts, .. }) = &mut reply else {
        panic!("selected transcript reply")
    };
    // The renderer may omit a turn when its cut marker cannot fit. This
    // checks the root/core handling of that result, not the renderer itself.
    *texts = Some(Box::new([engine::TurnText::Rendered(Box::new([]))]));
    driver.send(reply);
    driver.settle();
    let assignment = driver.last_assignment();
    fresh(&assignment);
    // Jig's domain/engine.md 7.2 leaves empty-tail section encoding unspecified.
    assert_eq!(tail(&assignment), Some(b"".as_slice()));
    no_failure_charged(&driver, first.task);
}

// Two reads are the root's smallest valid table; a third chat must wait.
const CHATS: u32 = 3;
const READS: u32 = 2;

fn parked_chats() -> (Driver, Vec<engine::Assignment>) {
    let mut bounds = limits();
    bounds.tasks.tasks = CHATS;
    bounds.tasks.project_tasks = CHATS;
    bounds.forge.tasks = CHATS;
    bounds.people.inbox_tasks = CHATS * 2;
    bounds.people.requests = CHATS * 2; // One create and one wake per chat.
    bounds.fleet.slots = CHATS;
    bounds.fleet.attempts = CHATS;
    bounds.fleet.grants = CHATS;
    bounds.fleet.turns = CHATS;
    bounds.loads.loads = READS;
    let mut driver = Driver::configured(Store::new(), [config(91), config(91)], &bounds);
    driver.send(protocol::hello_slots(CHANNEL, CHATS, Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut chats = Vec::new();
    for key in 1..=u8::try_from(CHATS).expect("three chats") {
        let people::Ask::Create { project, mut batch } = driver.chat_ask(1, b"read room conversation") else {
            panic!("chat creation")
        };
        batch[0].flags.ended_by = people::EndedBy::Requester;
        batch[0].budget = people::Budget::Named;
        batch[0].authority.spend = 100;
        driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1000 + u64::from(key))),
            sign_in: driver.session(),
            key: [key; 16],
            ask: people::Ask::Create { project, batch },
        }));
        driver.settle();
        let chat = driver.last_assignment();
        assert!(chats.iter().all(|previous: &engine::Assignment| previous.task != chat.task));
        driver.send(protocol::turn(CHANNEL, chat.task, chat.attempt, FIRST_TURN, SPENT, None, OLD_TEXT));
        driver.settle();
        driver.answer(CHANNEL, chat.task, chat.attempt, SPENT, tasks::End::Parked, engine::Parts::none());
        driver.settle();
        chats.push(chat);
    }
    (driver, chats)
}

fn transcript_task(range: &engine::Range) -> Option<u64> {
    if let engine::Range::Core(engine::CoreRange::TaskTranscript { task, .. }) = range { Some(*task) } else { None }
}

// Retain transcript replies in the driver's input queue, letting commits and other work drain.
fn hold_replies(driver: &mut Driver) {
    for _ in 0..STEPS {
        let mut replies = Vec::new();
        let mut other = std::collections::VecDeque::new();
        while let Some(event) = driver.events.pop_front() {
            if matches!(&event, engine::Event::Store(engine::StoreInput::Loaded { owner, .. })
                if driver.loads.iter().any(|(issued, range)| issued == owner && transcript_task(range).is_some()))
            {
                replies.push(event);
            } else {
                other.push_back(event);
            }
        }
        driver.events = other;
        driver.advance(true);
        driver.events.extend(replies);
    }
}

fn transcript_tasks(driver: &Driver) -> Vec<u64> {
    driver.loads.iter().filter_map(|(_, range)| transcript_task(range)).collect()
}

fn waiting_for_room(driver: &Driver, chats: &[engine::Assignment]) {
    assert_eq!(transcript_tasks(driver), [chats[0].task, chats[1].task]);
    assert_eq!(driver.task(chats[2].task).expect("waking chat").phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(driver.task(chats[0].task).expect("first reader").phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert_eq!(
        driver.task(chats[1].task).expect("second reader").phase,
        tasks::Phase::Active(tasks::Active::Preparing)
    );
}

fn retire_read_and_prepare_waiter(driver: &mut Driver, chats: &[engine::Assignment]) {
    let reply = pending_reply(driver);
    driver.send(reply);
    // Complete the read terminal's queued core continuation before owner progress.
    driver.send(engine::Event::Resume);
    let before = driver.transactions.len();
    driver.send(engine::Event::Resume);
    assert!(
        driver.transactions[before..].iter().flatten().any(|write| {
            matches!(write, engine::Write::Save(engine::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))))
            if row.number == chats[2].task && row.phase == tasks::Phase::Active(tasks::Active::Preparing))
        }),
        "the next Resume prepares the chat that was waiting for room"
    );
    hold_replies(driver);
    assert_eq!(transcript_tasks(driver), [chats[0].task, chats[1].task, chats[2].task]);
    driver.settle();
}

#[test]
fn a_full_read_table_keeps_a_waking_chat_due_until_a_read_retires() {
    let (mut driver, chats) = parked_chats();
    for (index, chat) in chats.iter().enumerate() {
        driver.send(say_ask(driver.session(), chat.task, FIRST_WAKE + u8::try_from(index).expect("three chats")));
    }
    hold_replies(&mut driver);
    waiting_for_room(&driver, &chats);
    retire_read_and_prepare_waiter(&mut driver, &chats);
    for chat in chats {
        no_failure_charged(&driver, chat.task);
    }
}

#[test]
fn two_activations_in_one_decision_share_the_last_read_slot() {
    let (mut driver, chats) = parked_chats();
    driver.send(say_ask(driver.session(), chats[0].task, FIRST_WAKE));
    hold_replies(&mut driver);
    assert_eq!(transcript_tasks(&driver), [chats[0].task]);
    let mut operations = List::with_capacity(READS);
    for chat in &chats[1..] {
        operations
            .push(people::Operation::Message { task: chat.task, words: Box::from(b"wake".as_slice()) })
            .expect("two messages in one compound");
    }
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(3000)),
        sign_in: driver.session(),
        key: [99; 16],
        ask: people::Ask::Compound { project: 1, operations },
    }));
    hold_replies(&mut driver);
    waiting_for_room(&driver, &chats);
    retire_read_and_prepare_waiter(&mut driver, &chats);
}
