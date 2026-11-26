//! Cold note indexes and transcript windows share the engine’s work lane.
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{
    direct::{Driver, say_ask},
    protocol,
    store::Store,
};

const CHANNEL: Token = Token::new(7); // The direct world's one worker.
const PROJECT: u32 = 1; // The walking fixture's configured project.
const REQUEST: Token = Token::new(1001); // The direct world's creation reply.
const WHOSE: Token = Token::new(1002); // A separate reply for the post-pressure read.
const FIRST_TASK: u64 = 1; // The first chat consumes the first task number.
const SECOND_TASK: u64 = 2; // The second chat consumes the next task number.
const TRANSCRIPTS: usize = 2; // Both chats fill walking::limits().loads.loads.
const CHAT_SPEND: u64 = 100; // The walking policy's maximum run spend; two shares fit its period.
// The direct world's settlement horizon; this checks progress, never elapsed time.
const STEPS: u32 = 200;

fn park(driver: &mut Driver, run: &engine::Assignment) {
    let read = run.messages.last().map(|message| message.number);
    driver.send(protocol::turn(CHANNEL, run.task, run.attempt, 1, 1, read, b"kept turn"));
    driver.pump();
    driver.answer(CHANNEL, run.task, run.attempt, 1, jig_domain_tasks::End::Parked, engine::Parts::none());
    driver.pump();
}

fn signed_in(store: Store) -> Driver {
    let mut driver = Driver::new(store);
    driver.hello(CHANNEL);
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}

fn restarted_transcripts() -> (Driver, Vec<engine::Event>) {
    let mut driver = signed_in(Store::new());
    let first = chat(&mut driver, 31, b"first chat");
    park(&mut driver, &first);
    let second = chat(&mut driver, 32, b"second chat");
    park(&mut driver, &second);
    let mut driver = signed_in(driver.take_store());
    driver.send(say_ask(driver.session(), first.task, 50));
    driver.send(say_ask(driver.session(), second.task, 51));
    hold_transcripts(driver)
}

fn chat(driver: &mut Driver, key: u8, words: &[u8]) -> engine::Assignment {
    let people::Ask::Create { project, mut batch } = driver.chat_ask(PROJECT, words) else {
        panic!("chat template creates a task")
    };
    // A named share leaves the fixture's remaining period funds for the
    // other chat; AsAllowed would give the first chat all of them.
    batch[0].flags.ended_by = people::EndedBy::Requester;
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = CHAT_SPEND;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(REQUEST),
        sign_in: driver.session(),
        key: [key; 16],
        ask: people::Ask::Create { project, batch },
    }));
    driver.settle();
    driver.last_assignment()
}

fn hold_transcripts(mut driver: Driver) -> (Driver, Vec<engine::Event>) {
    // Both live chats have a transcript. Hold their issued store answers,
    // allowing the root's ordinary owner continuations to prepare both.
    let mut held = Vec::new();
    for _ in 0..STEPS {
        driver.advance(true);
        let mut remaining = std::collections::VecDeque::new();
        while let Some(event) = driver.events.pop_front() {
            let transcript = temper_engine_domain_world::direct::answer_token(&event)
                .is_some_and(|load| driver.windows.iter().any(|(issued, _)| *issued == load));
            if transcript {
                held.push(event);
            } else {
                remaining.push_back(event);
            }
        }
        driver.events = remaining;
        if held.len() == TRANSCRIPTS {
            return (driver, held);
        }
    }
    panic!("both transcript loads must be issued before their answers: {:?}", driver.root);
}

#[test]
fn restart_notes_fit_after_a_window_finishes() {
    let (mut driver, mut held) = restarted_transcripts();
    assert!(driver.note_loads.is_empty(), "both windows still hold their slots");
    driver.send(held.remove(0));
    driver.events.extend(held);
    driver.pump();
    assert_eq!(driver.last_assignment().task, FIRST_TASK);
}

#[test]
fn restart_notes_need_no_window_slot_reclaim() {
    let (mut driver, mut held) = restarted_transcripts();
    let before = driver.task(FIRST_TASK).expect("first transcript is preparing").clone();
    let loads = driver.note_loads.len();
    let decisions = driver.transactions.len();
    // Window answers pass through the engine; the notes read need not wait for root reclaim.
    engine::try_step(&mut driver.root, &driver.env, held.remove(0)).expect("store terminal admitted");
    super::preparation::preparation_commits(&mut driver);
    let preparing = driver.task(FIRST_TASK).expect("preparation stays held");
    assert_eq!(before.phase, jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Preparing));
    assert_eq!(preparing.phase, before.phase);
    assert_eq!(preparing.refusals, before.refusals);
    assert_eq!(preparing.retry_notice, before.retry_notice);
    assert_eq!(preparing.tries, before.tries);
    assert_eq!(preparing.attempt, before.attempt);
    assert_eq!(preparing.made, before.made);
    assert_eq!(preparing.numbers, before.numbers);
    assert!(
        driver.note_loads[loads..].iter().any(|(_, range)| matches!(range, jig_domain_notes::Range::Lines { .. })),
        "notes can load before reclaim"
    );
    driver.root.reclaim();
    driver.events.extend(held);
    driver.pump();
    // The worker has one slot. Return the first new assignment's slot so
    // the other live chat must also make progress after its room returns.
    let assigned = driver.last_assignment();
    park(&mut driver, &assigned);
    driver.pump();
    let mut tasks: Vec<u64> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let engine::Delivery::Assigned { assignment, .. } = delivery { Some(assignment.task) } else { None }
        })
        .collect();
    tasks.sort_unstable();
    assert_eq!(tasks, [FIRST_TASK, SECOND_TASK]);
    let phases: Vec<_> = driver.transactions[decisions..]
        .iter()
        .flatten()
        .filter_map(|write| {
            if let engine::Write::Save(engine::Record::Core(jig_domain_core::Record::Tasks(
                jig_domain_tasks::Stored::Live(row),
            ))) = write
                && row.number == FIRST_TASK
            {
                Some(row.phase.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(
        !phases.contains(&jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Due)),
        "the first task never had to reactivate for notes"
    );
    driver.send(engine::Event::Party(engine::PartyInput::Whose {
        reply_to: ReplyTo::new(WHOSE),
        sign_in: driver.session(),
    }));
    driver.pump();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(delivery,
            engine::Delivery::WebReply { to, reply: people::Reply::Signer(Some(_)) }
                if *to == ReplyTo::new(WHOSE)
        )),
        "load pressure leaves the root able to answer a person"
    );
}
