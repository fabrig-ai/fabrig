//! Calls answered at decode, as errors or successes, leave no commit and
//! no record (domain/root.md, 5.2; protocol/documentation.md, 5).
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::Token;
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_world::direct::Driver;

#[test]
fn an_undecodable_call_is_answered_as_an_error_with_no_commit() {
    let (mut driver, run) = Driver::chat();
    let (applied, rows) = (driver.store.applied, driver.store.rows.clone());
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    assert_eq!(driver.store.applied, applied);
    assert_eq!(driver.store.rows, rows);
    let relayed: Vec<&host::Relayed> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Link { down: host::LinkDown::Relayed { call, answer, .. }, .. } = delivery
                && call.as_ref() == 31_u64.to_be_bytes()
            {
                Some(answer)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(relayed, [&host::Relayed::Answer { error: true, text: Box::from(&b"unknown tool"[..]) }]);
}

#[test]
fn an_undecodable_call_leaves_no_record_for_the_resumed_run() {
    let (mut driver, run) = Driver::chat();
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    driver.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(run.task, 93);
    let resumed = driver.last_assignment();
    assert_ne!(resumed.attempt, run.attempt);
    assert!(resumed.settled.is_empty());
    assert!(resumed.answered.is_empty());
}

#[test]
fn a_corrected_call_under_a_new_name_is_decided_once() {
    let (mut driver, run) = Driver::chat();
    let event = driver.protocol.undecodable(Token::new(7), run.task, run.attempt, 31);
    driver.send(event);
    driver.settle();
    driver.call(
        Token::new(7),
        run.task,
        run.attempt,
        32,
        engine::Call { completion: 2, position: 0, tool: engine::Tool::Unavailable },
    );
    driver.settle();
    assert_eq!(driver.answered(32), Some(&engine::CallAnswer::Unavailable));
    let name = 32_u64.to_be_bytes();
    assert_eq!(driver.protocol.answers.iter().filter(|(named, _)| named.as_ref() == name).count(), 1);
}

#[test]
fn a_call_answered_at_decode_is_relayed_as_a_success_with_no_commit() {
    let (mut driver, run) = Driver::chat();
    let (applied, rows) = (driver.store.applied, driver.store.rows.clone());
    let event = driver.protocol.answered_at_decode(Token::new(7), run.task, run.attempt, 31, b"answer made at decode");
    driver.send(event);
    driver.settle();
    assert_eq!(driver.store.applied, applied);
    assert_eq!(driver.store.rows, rows);
    let relayed: Vec<&host::Relayed> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Link { down: host::LinkDown::Relayed { call, answer, .. }, .. } = delivery
                && call.as_ref() == 31_u64.to_be_bytes()
            {
                Some(answer)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(relayed, [&host::Relayed::Answer { error: false, text: Box::from(&b"answer made at decode"[..]) }]);
}

#[test]
fn a_call_answered_at_decode_leaves_no_record_for_the_resumed_run() {
    let (mut driver, run) = Driver::chat();
    let event = driver.protocol.answered_at_decode(Token::new(7), run.task, run.attempt, 31, b"answer made at decode");
    driver.send(event);
    driver.settle();
    driver.answer(Token::new(7), run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    driver.say(run.task, 93);
    let resumed = driver.last_assignment();
    assert_ne!(resumed.attempt, run.attempt);
    assert!(resumed.settled.is_empty());
    assert!(resumed.answered.is_empty());
}
