//! The root keeps these orders whatever admits work (skein's draft
//! `dynamic-memory.md`, 4.6): store terminals in arrival order, and a kept
//! turn before its terminal acknowledgement, while admission is under load.

use super::admission_support;
use jig_core_world::world::World;
use jig_domain_core::{self as core, reads};
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const WORKER: Token = Token::new(7);
const READ_A: Token = Token::new(960);
const READ_B: Token = Token::new(961);
const TURN: u32 = 1;
const SPENT: u64 = 3;
const TURN_TEXT: &[u8] = b"kept turn";
const REPORT: &[u8] = b"finished work";
// Each story writes one turn, and the terminal story writes one final answer.
const ONCE: usize = 1;
const STORY_COMMITS: usize = 2;
// Same finite settlement budget as the direct world.
const STEPS: usize = 300;

/// O2: store answers taken while admission binds answer in arrival order.
#[test]
fn queued_read_answers_keep_their_arrival_order_under_load() {
    let mut driver = admission_support::chat(9302);
    let (task, attempt) = run(&driver);
    driver.offer(turn(task, attempt));
    driver.drain();
    let second = read_turn(&mut driver, READ_B);
    let first = read_turn(&mut driver, READ_A);
    driver.fill_to_watermark();
    let before_probe = driver.returned;
    driver.probe();
    assert_eq!(driver.returned, before_probe + 1, "new work is handed back under load");
    let returned = driver.returned;
    driver.offer(first);
    driver.offer(second);
    driver.collect();
    assert_eq!(driver.returned, returned, "both store terminals are taken under load");
    assert!(read_replies(&driver).is_empty(), "answers wait for preceding durability");
    driver.drain();
    let replies = read_replies(&driver);
    assert_eq!(replies.iter().map(|(reply, _)| *reply).collect::<Vec<_>>(), [READ_A, READ_B]);
    for (_, answer) in replies {
        let reads::Answer::Ended(past) = answer else { panic!("ended turn: {answer:?}") };
        let reads::EndedValue::Turn(row) = &past.value else { panic!("requested turn") };
        assert_eq!((row.task, row.attempt, row.turn), (task, attempt, TURN));
        assert_eq!(row.transcript.as_ref(), TURN_TEXT);
    }
    assert!(driver.domain.quiescent() && !driver.stopped);
    println!("O2 handed-back inputs: {}", driver.returned);
}

/// O3: a kept turn precedes its terminal acknowledgement under load.
#[test]
fn turn_kept_precedes_terminal_acknowledgement_under_load() {
    let mut driver = admission_support::chat(9303);
    let (task, attempt) = run(&driver);
    driver.offer(turn(task, attempt));
    driver.collect();
    driver.offer(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: WORKER,
        task,
        attempt,
        cumulative: SPENT,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: REPORT.into() },
            cancel_delegates: false,
        },
    });
    driver.collect();
    assert_eq!(driver.store.pending.len(), STORY_COMMITS, "turn and terminal commits are withheld");
    driver.fill_to_watermark();
    assert!(worker_notices(&driver).is_empty(), "acknowledgements wait for durability");
    // This root routes released callbacks inside release().
    // Refill before that release, so their decisions face real pressure too.
    for _ in 0..STORY_COMMITS {
        driver.commit_one();
        driver.fill_to_watermark();
    }
    driver.drain();
    println!("O3 handed-back inputs: {}", driver.returned);
    assert_eq!(
        worker_notices(&driver),
        [(task, attempt, Some(TURN)), (task, attempt, None)],
        "worker sees its kept turn before the terminal acknowledgement, each once"
    );
    let writes: Vec<_> = driver.commits.iter().flatten().collect();
    assert_eq!(
        writes
            .iter()
            .filter(|write| matches!(write,
                root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Turn(row))))
                    if row.task == task && row.attempt == attempt && row.turn == TURN
            ))
            .count(),
        ONCE,
        "turn is written once"
    );
    assert_eq!(
        writes
            .iter()
            .filter(|write| matches!(write,
                root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(row))))
                    if row.task == task && row.attempt == attempt
            ))
            .count(),
        ONCE,
        "terminal is written once"
    );
    assert_eq!(
        driver
            .released
            .iter()
            .filter(|request| matches!(request,
                root::Request::Deliver(root::Delivery::Core(core::Held::Result { task: named, words, .. }))
                    if *named == task && words.as_ref() == REPORT
            ))
            .count(),
        ONCE,
        "party receives the result once"
    );
    assert!(driver.domain.quiescent() && !driver.stopped);
}

fn run(driver: &World) -> (u64, u64) {
    let assignment = driver.assignment();
    (assignment.task, assignment.attempt)
}

fn turn(task: u64, attempt: u64) -> root::Event {
    root::Event::Turn {
        opens_window: true,
        refused: Box::new([]),
        channel: WORKER,
        task,
        attempt,
        turn: TURN,
        cumulative: SPENT,
        read: None,
        transcript: TURN_TEXT.into(),
    }
}

fn read_turn(driver: &mut World, reply: Token) -> root::Event {
    let (task, attempt) = run(driver);
    driver.offer(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(reply),
        sign_in: driver.session(),
        read: reads::Read::Turn { task, attempt, turn: TURN },
    }));
    for _ in 0..STEPS {
        driver.collect();
        if let Some(index) =
            driver.events.iter().position(|event| matches!(event, root::Event::Core(core::Event::ReadLoaded { .. })))
        {
            return driver.events.remove(index).expect("actual store terminal");
        }
        if let Some(event) = driver.events.pop_front() {
            driver.send_retaining(event);
        }
        if !driver.store.pending.is_empty() {
            driver.commit_one();
        }
    }
    panic!("ended read issues a store load within the step budget");
}

fn read_replies(driver: &World) -> Vec<(Token, &reads::Answer)> {
    driver
        .released
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Core(core::Held::Read { reply, answer })) = request {
                [READ_A, READ_B].into_iter().find(|token| *reply == ReplyTo::new(*token)).map(|token| (token, answer))
            } else {
                None
            }
        })
        .collect()
}

fn worker_notices(driver: &World) -> Vec<(u64, u64, Option<u32>)> {
    driver
        .released
        .iter()
        .filter_map(|request| {
            if let root::Request::Deliver(root::Delivery::Core(held)) = request {
                if let core::Held::AcknowledgeTurn { channel, run, attempt, turn } = held
                    && *channel == WORKER
                {
                    Some((run.raw(), attempt.raw(), Some(*turn)))
                } else if let core::Held::Acknowledge { channel, run, attempt } = held
                    && *channel == WORKER
                {
                    Some((run.raw(), attempt.raw(), None))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
