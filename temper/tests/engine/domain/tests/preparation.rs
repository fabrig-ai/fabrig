//! A run's preparation and its answer at awkward moments: a store read that
//! lands after the preparation it served was dropped, and an answer whose
//! parts exceed their bounds.
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Time, Token, Wall};
use temper_engine_domain as engine;
use temper_engine_domain::{CoreRange, Range};
use temper_engine_domain_world::direct::{Driver, say_ask};
use temper_engine_domain_world::protocol;

/// Step until the root has asked for `task`'s transcript, leaving the
/// store's answer queued; the answer's owner.
fn transcript_in_flight(driver: &mut Driver, task: u64) -> Token {
    for _ in 0..50 {
        driver.advance(true);
        let asked = driver.loads.iter().find_map(|(owner, range)| match range {
            Range::Core(CoreRange::TaskTranscript { task: read, .. }) if *read == task => Some(*owner),
            Range::Core(_) | Range::Census(_) | Range::Forge => None,
        });
        if let Some(owner) = asked {
            return owner;
        }
    }
    panic!("the parked chat's preparation read its transcript")
}

// A message can end a preparation while its store read is in flight; the
// read's answer then belongs to none (jig's `examples/ops/domain/src/restart.rs`).
#[test]
fn a_transcript_read_landing_after_a_message_ended_its_preparation_is_dropped() {
    let (mut driver, first) = Driver::chat();
    driver.send(protocol::turn(Token::new(7), first.task, first.attempt, 1, 1, None, b"first"));
    driver.settle();
    driver.answer(Token::new(7), first.task, first.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.send(say_ask(driver.session(), first.task, 50));
    transcript_in_flight(&mut driver, first.task);
    driver.send(say_ask(driver.session(), first.task, 51));
    driver.settle();
    // The message ended that preparation; the task backs off and prepares again.
    driver.env.now = Time::from_nanos(Duration::from_secs(1).as_nanos());
    driver.env.wall = Wall::from_nanos(Duration::from_secs(1).as_nanos());
    engine::fire(&mut driver.root, &driver.env);
    driver.settle();
    let next = driver.last_assignment();
    assert_eq!((next.task, next.attempt), (first.task, 2));
    assert_eq!(next.transcript.len(), 1);
    assert_eq!(next.messages.iter().map(|message| message.words.as_ref()).collect::<Vec<_>>(), [[50], [51]]);
}

// Parts beyond their bounds make the answer invalid; the run fails as invalid.
#[test]
fn an_answer_pushing_more_heads_than_a_task_may_hold_fails_its_run_as_invalid() {
    let (mut driver, run) = Driver::chat();
    let pushed = (0..5).map(|tag| temper_engine_domain_forge::Pushed { tag, commit: [7; 32] }).collect();
    let parts = engine::Parts { saved: None, pushed, preparation: None };
    let end = tasks::End::Finished {
        result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
        cancel_delegates: false,
    };
    driver.answer(Token::new(7), run.task, run.attempt, 1, end, parts);
    driver.settle();
    let task = driver.task(run.task).expect("the chat lives on");
    assert_eq!(task.tries.invalid, 1);
}

fn failed_preparation(run: &engine::Assignment, preparation: engine::Preparation) -> engine::Event {
    let ending = host::Ending::Failed {
        failure: host::Failure::Unprepared(host::Preparation::Permanent { resource: None }),
        detail: Box::new([]),
        work: host::Work { left: None, saved: None },
    };
    let answer = host::Answer { read: None, refused: Box::new([]), turns: 0, spent: 1, ending };
    let parts = engine::Parts { saved: None, pushed: Box::new([]), preparation: Some(preparation) };
    let up = host::LinkUp::Answer { run: Token::new(run.task), attempt: Token::new(run.attempt), answer };
    protocol::up(Token::new(7), up, parts)
}

// A permanent preparation failure names its repository beside the answer;
// the root reports it as a fact of the attempt (domain/root.md, 7.4).
#[test]
fn a_permanent_preparation_failure_is_reported_as_a_fact_of_its_attempt() {
    let (mut driver, run) = Driver::chat();
    let preparation = engine::Preparation::Missing { tag: 3, missing: engine::Missing::Branch };
    driver.send(failed_preparation(&run, preparation));
    driver.settle();
    let fact = engine::Fact::PreparationFailed { task: run.task, attempt: run.attempt, preparation };
    assert_eq!(driver.root.pop_fact(), Some(fact));
    assert_eq!(driver.root.pop_fact(), None);
}

#[test]
fn an_answer_without_a_preparation_reports_no_fact() {
    let (mut driver, run) = Driver::chat();
    driver.answer(Token::new(7), run.task, run.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    assert_eq!(driver.root.pop_fact(), None);
}

#[test]
fn a_failed_preparation_retries_by_its_answers_class() {
    let (mut driver, run) = Driver::chat();
    let preparation = engine::Preparation::Refused { tag: 3 };
    driver.send(failed_preparation(&run, preparation));
    driver.settle();
    assert_eq!(driver.task(run.task).expect("the chat lives on").tries.permanent, 1);
}
