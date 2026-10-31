//! Task timers survive fresh runs (temper/docs/design/domain/testing.md, 2.2).
use super::*;

#[test]
fn an_armed_timer_survives_a_fresh_director_run_and_its_helpers_cancellation() {
    let (mut configuration, bounds) = batch_settings(2, 1, 500, false, false, false);
    for config in &mut configuration {
        config.core.defaults.charters[0].policy.resume = false;
    }
    let mut driver = Driver::configured(Store::new(), configuration, &bounds);
    driver.send(hello_with(Token::new(7), 2, Duration::from_secs(1), Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let director = assigned(&driver);
    let at = Wall::from_nanos(Duration::from_secs(30).as_nanos());
    let kind = tasks::SubscriptionKind::Timer { at, period: None };
    tool_call(&mut driver, &director, 940, engine::Tool::Subscribe { kind });
    let Some(engine::CallAnswer::Subscribed { subscription }) = driver.answered(940) else {
        panic!("director's timer admitted")
    };
    let expected = [tasks::Subscription { number: *subscription, kind }];
    let mut helper = report_delegate(b"helper", Box::new([]));
    helper.tracked = Some(1);
    let helper = call_batch(&mut driver, &director, 941, Box::new([helper]))[0];
    let helper_run = assigned_task(&driver, helper);
    driver.send(protocol::turn(
        Token::new(7),
        director.task,
        director.attempt,
        1,
        1,
        None,
        b"timer armed; helper working",
    ));
    driver.settle();
    driver.answer(Token::new(7), director.task, director.attempt, 1, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    assert_eq!(live(&driver, director.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(live(&driver, director.task).subscriptions.as_ref(), expected);
    let correction = driver.say(director.task, 82);
    let fresh = driver.last_assignment();
    assert_eq!(fresh.task, director.task);
    assert!(fresh.attempt > director.attempt);
    assert!(!fresh.run.policy.resume);
    assert!(fresh.transcript.is_empty());
    assert_eq!(fresh.messages.iter().filter(|word| word.number == correction).count(), 1);
    assert_eq!(live(&driver, director.task).subscriptions.as_ref(), expected);
    tool_call(
        &mut driver,
        &fresh,
        942,
        engine::Tool::Cancel { target: helper, reason: b"priorities corrected".as_slice().into() },
    );
    assert_eq!(driver.answered(942), Some(&engine::CallAnswer::Controlled));
    assert!(cancelled(&driver).contains(&(helper, helper_run.attempt)));
    driver.answer(Token::new(7), helper, helper_run.attempt, 0, tasks::End::Parked, engine::Parts::none());
    driver.settle();
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) =
        driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(helper))))
    else {
        panic!("helper cancellation settled before the timer")
    };
    assert!(matches!(&ended.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { by, reason, .. })
        if *by == tasks::Party::Task(director.task) && reason.as_ref() == b"priorities corrected"));
    assert!(driver.env.wall < at);
    assert_eq!(live(&driver, director.task).subscriptions.as_ref(), expected);
    assert!(!cancelled(&driver).contains(&(fresh.task, fresh.attempt)));
}
