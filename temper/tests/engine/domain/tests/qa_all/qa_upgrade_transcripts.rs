//! Independent M1 checks of temper's transcript-load translation.
//! domain/root.md 7.1, 8 and 9; jig's domain/engine.md 7.2.
//! The public driver supplies already rendered text, not smith's renderer.
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{
    direct::{Driver, say_ask},
    protocol,
};

const CHANNEL: Token = Token::new(7);
const FIRST_TURN: u32 = 1;
const SPENT: u64 = 1;
const OLD_TEXT: &[u8] = b"text from the retired conversation";
const NEW_TEXT: &[u8] = b"text from the new conversation";
// say_ask uses each byte as both an opaque key and UTF-8 words.
const FIRST_WAKE: u8 = b'A';
const NEXT_WAKE: u8 = b'B';
// The direct driver's own public pump/settle bound is 200 iterations.
const STEPS: usize = 200;

fn proof_key(task: u64) -> engine::Key {
    engine::Key::Core(core::Key::Core(core::CoreKey::RunProof(task)))
}

fn pending_reply(driver: &mut Driver) -> engine::Event {
    for _ in 0..STEPS {
        let found = driver.events.iter().position(|event| {
            if let engine::Event::Store(engine::StoreInput::Loaded { owner, .. }) = event {
                driver.loads.iter().any(|(issued, range)| {
                    issued == owner && matches!(range, engine::Range::Core(engine::CoreRange::TaskTranscript { .. }))
                })
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

fn load_owner(reply: &engine::Event) -> Token {
    let engine::Event::Store(engine::StoreInput::Loaded { owner, .. }) = reply else {
        panic!("pending transcript reply")
    };
    *owner
}

fn waiting(closed: bool) -> (Driver, engine::Assignment, u64, engine::Event) {
    let (mut driver, first) = Driver::chat();
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
    let mut driver = Driver::new(driver.store);
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
fn an_issued_closed_unread_starts_fresh_without_charges() {
    let (mut driver, first, _, reply) = waiting(true);
    driver.send(engine::Event::Store(engine::StoreInput::Unloaded { owner: load_owner(&reply) }));
    driver.settle();
    let assignment = driver.last_assignment();
    assert_eq!(assignment.attempt, first.attempt + 1);
    fresh(&assignment);
    assert_eq!(tail(&assignment), None);
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
    *texts = Some(Box::new([Some(Box::new([]))]));
    driver.send(reply);
    driver.settle();
    let assignment = driver.last_assignment();
    fresh(&assignment);
    // Jig's domain/engine.md 7.2 leaves empty-tail section encoding unspecified.
    assert_eq!(tail(&assignment), Some(b"".as_slice()));
    no_failure_charged(&driver, first.task);
}

#[test]
fn an_issued_open_unread_spends_one_transient_try() {
    let (mut driver, first, _, reply) = waiting(false);
    driver.send(engine::Event::Store(engine::StoreInput::Unloaded { owner: load_owner(&reply) }));
    driver.pump();
    let row = driver.task(first.task).expect("chat waits for its retry");
    assert_eq!(row.tries.transient, 1);
    assert_eq!(
        (row.tries.permanent, row.tries.run, row.tries.agent, row.tries.lost, row.tries.invalid),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(row.refusals, 0);
    // Walking's public retry fixture waits 10 ms, capped at one second.
    driver.fire_at(Duration::from_secs(1));
    driver.settle();
    let assignment = driver.last_assignment();
    // Preparation precedes the claim (jig's domain/engine.md 7.1; domain/tasks.md 5.2).
    assert_eq!(assignment.attempt, first.attempt + 1);
    fresh(&assignment);
    // The discarded open window leaves no text in the fresh brief's tail.
    assert_eq!(tail(&assignment), Some(b"".as_slice()));
}

#[test]
fn retired_terminals_cannot_poison_a_later_window_load() {
    let (mut driver, first, session, retired) = waiting(true);
    let retired_owner = load_owner(&retired);
    driver.send(engine::Event::Store(engine::StoreInput::Unloaded { owner: retired_owner }));
    driver.settle();
    let second = driver.last_assignment();
    // Take the offered messages so parking waits for the next explicit wake
    // (jig's domain/engine.md 7.4). Their numbers, not request keys, fence reads.
    let read = second.messages.last().map(|message| message.number);
    driver.send(protocol::turn(CHANNEL, second.task, second.attempt, FIRST_TURN, SPENT, read, NEW_TEXT));
    driver.settle();
    driver.answer(CHANNEL, second.task, second.attempt, SPENT, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.send(say_ask(session, first.task, NEXT_WAKE));
    let current = pending_reply(&mut driver);
    let assigned = |driver: &Driver| {
        driver.delivered.iter().filter(|delivery| matches!(delivery, engine::Delivery::Assigned { .. })).count()
    };
    let before = assigned(&driver);
    driver.send(retired);
    driver.send(engine::Event::Store(engine::StoreInput::Unloaded { owner: retired_owner }));
    assert_eq!(assigned(&driver), before, "retired terminals release no current assignment");
    driver.send(current);
    driver.settle();
    let assignment = driver.last_assignment();
    assert_eq!(assignment.attempt, first.attempt + 2);
    assert_eq!(assignment.conversation, second.attempt);
    assert!(assignment.run.policy.resume);
    assert_eq!(assignment.transcript.as_ref(), &[Box::<[u8]>::from(NEW_TEXT)]);
    no_failure_charged(&driver, first.task);
}
