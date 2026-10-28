//! The root keeps these orders whatever admits work (skein's draft
//! `dynamic-memory.md`, 4.3): pending callbacks before ordinary entrances,
//! store answers in arrival order, and older releases before later ones.

use jig_domain_core::reads;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Record};
use temper_engine_domain_world::{direct::Driver, protocol, roles, walking::config};

const HISTORY: Token = Token::new(950);
const ROSTER: Token = Token::new(951);
const READ_A: Token = Token::new(960);
const READ_B: Token = Token::new(961);
const WORKER: Token = Token::new(7);
const TURN_TEXT: &[u8] = b"kept turn";
// The direct world's journal has three outstanding commit slots.
const FULL_COMMITS: usize = 3;
// The direct fixture's finite settlement budget.
const STEPS: usize = 200;

fn fill_commits(driver: &mut Driver) {
    for _ in driver.store.pending.len()..FULL_COMMITS {
        driver.sign_in();
    }
}

/// O1: a load's callbacks, left in the root's work queue while the
/// journal's commits are full, run before a later party change.
#[test]
fn completed_history_callbacks_precede_a_later_party_change() {
    let seed = 9204;
    // This fixture runs escalation's completed history, then gives the saved
    // policy permission for the later roster change.
    let world = roles::World::new(roles::Settings::calm(seed, roles::Base::Rejected));
    let archive = world
        .store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::EscalationDecision(
                archive,
            ))) = row
            {
                Some(archive.clone())
            } else {
                None
            }
        })
        .expect("actual immutable escalation decision");
    let session = world.store.header().sign_ins;
    let mut driver = Driver::configured(world.store, [config(seed), config(seed)], &roles::limits());
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(HISTORY),
        sign_in: session,
        key: [9; 16],
        ask: people::Ask::DecideEscalation {
            project: archive.project,
            task: archive.task,
            revision: archive.revision,
            decision: people::EscalationDecision::Release,
        },
    }));
    let loaded = take_history_load(&mut driver);
    fill_commits(&mut driver);
    driver.send(loaded);
    let returned = driver.returned;
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(ROSTER),
        sign_in: session,
        key: [10; 16],
        ask: people::Ask::SetRoles {
            project: archive.project,
            holdings: Box::new([people::Holding { person: archive.by, role: people::Role::Owner }]),
        },
    }));
    assert_eq!(
        driver.store.pending.len(),
        FULL_COMMITS,
        "unanswered commits bind the room check with the history callback waiting"
    );
    assert_eq!(
        driver.returned,
        returned + 1,
        "the later party change is handed back while the callback waits for room"
    );
    assert!(keyed_replies(&driver).is_empty(), "neither request answers before callback durability");
    driver.settle();
    assert_eq!(
        keyed_replies(&driver),
        [
            (
                HISTORY,
                people::Reply::Outcome(people::Outcome::EscalationDecided {
                    task: archive.task,
                    revision: archive.revision,
                    by: archive.by,
                    at: archive.at,
                    choice: people::EscalationChoice::Rejected,
                })
            ),
            (ROSTER, people::Reply::Outcome(people::Outcome::RolesSet { project: archive.project })),
        ],
        "history callback answers before the later roster change, each exactly once"
    );
    assert!(driver.root.quiescent() && !driver.stopped);
}

fn take_history_load(driver: &mut Driver) -> engine::Event {
    for _ in 0..STEPS {
        if let Some(index) = driver
            .events
            .iter()
            .position(|event| matches!(event, engine::Event::Store(engine::StoreInput::Loaded { .. })))
        {
            return driver.events.remove(index).expect("historical load terminal");
        }
        driver.advance(true);
    }
    panic!("historical request issues its load within the direct world's budget");
}

fn keyed_replies(driver: &Driver) -> Vec<(Token, people::Reply)> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::WebReply { to, reply } = delivery {
                [HISTORY, ROSTER]
                    .into_iter()
                    .find(|token| *to == ReplyTo::new(*token))
                    .map(|token| (token, reply.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// O2: two read answers, queued while the journal is full, answer in
/// the order they arrived.
#[test]
fn queued_read_answers_keep_their_arrival_order_under_load() {
    let (mut driver, assignment) = Driver::chat();
    driver.send(protocol::turn(WORKER, assignment.task, assignment.attempt, 1, 3, None, TURN_TEXT));
    driver.settle();
    let second = read_turn(&mut driver, &assignment, READ_B);
    let first = read_turn(&mut driver, &assignment, READ_A);
    // Completion order deliberately differs from request order.
    fill_commits(&mut driver);
    let returned = driver.returned;
    driver.send(first);
    driver.send(second);
    assert_eq!(driver.returned, returned, "both read terminals are taken while commit room binds");
    assert_eq!(driver.store.pending.len(), FULL_COMMITS, "both ReadLoaded answers wait behind the full journal's room check");
    assert!(read_replies(&driver).is_empty(), "neither queued answer has bypassed the full journal");
    driver.settle();
    assert_eq!(read_replies(&driver), [READ_A, READ_B], "read replies follow terminal arrival order, each exactly once");
    for delivery in &driver.delivered {
        if let Delivery::Read { answer, .. } = delivery {
            let reads::Answer::Historical(past) = answer else { panic!("authenticated historical read: {answer:?}") };
            let reads::Past::Turn(turn) = &past.value else { panic!("the requested historical turn") };
            assert_eq!(turn.transcript.as_ref(), TURN_TEXT, "authenticated read returned the durable turn");
        }
    }
    assert!(driver.root.quiescent() && !driver.stopped);
}

fn read_turn(driver: &mut Driver, assignment: &engine::Assignment, reply: Token) -> engine::Event {
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(reply),
        sign_in: driver.session(),
        read: reads::Read::Turn { task: assignment.task, attempt: assignment.attempt, turn: 1 },
    }));
    for _ in 0..STEPS {
        if let Some(index) = driver.events.iter().position(|event| {
            matches!(event, engine::Event::Store(engine::StoreInput::ReadLoaded { .. }))
        }) {
            return driver.events.remove(index).expect("real read terminal");
        }
        driver.advance(true);
    }
    panic!("authenticated read issues its load within the direct world's budget");
}

fn read_replies(driver: &Driver) -> Vec<Token> {
    driver.delivered.iter().filter_map(|delivery| {
        if let Delivery::Read { reply, .. } = delivery {
            [READ_A, READ_B].into_iter().find(|token| *reply == ReplyTo::new(*token))
        } else {
            None
        }
    }).collect()
}
