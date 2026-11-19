//! A raised entry limit reaches both the outbox and its deadlines.
use super::{LIMITS, configured, fixtures};
use crate::{Event, Outcome, RecoveryClock, Request};
use skein_lib::{Env, Queue, Time, Wall};

#[test]
fn a_raise_takes_the_ninth_entry_and_its_alarm_without_rebuilding() {
    let mut limits = LIMITS;
    limits.entries = 8;
    let mut domain = configured(&limits);
    let mut env = Env { limits, now: Time::ZERO, wall: Wall::EPOCH };
    let mut out = Queue::uncapped();
    crate::step(&mut domain, &env, Event::Restored { clock: RecoveryClock::Monotonic }, &mut out);
    for number in 1..=8 {
        let mut entry = fixtures::entry(skein_forge::api::Write::Close { number });
        entry.number = number;
        crate::step(&mut domain, &env, Event::Make { entry }, &mut out);
    }
    assert!(out.is_empty());
    let mut ninth = fixtures::entry(skein_forge::api::Write::Close { number: 9 });
    ninth.number = 9;
    crate::step(&mut domain, &env, Event::Make { entry: ninth.clone() }, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Outcome { entry: 9, task: 7, outcome: Outcome::Failed(skein_forge::api::Error::Busy) })
    );
    assert!(out.is_empty());
    env.limits.entries = 16;
    domain.raise(&env.limits);
    crate::step(&mut domain, &env, Event::Make { entry: ninth }, &mut out);
    assert!(out.is_empty(), "the ninth entry is retained, not refused");
    let alarms = env.limits.pending + env.limits.entries + env.limits.resources + env.limits.repositories + 2;
    for number in 0..alarms {
        domain.alarms.arm(crate::domain::Alarm::Entry(u64::from(number)), Time::ZERO).expect("raised alarm room");
    }
    assert_eq!(domain.alarms.len(), alarms);
    // Existing entries survive the raise and can still be withdrawn.
    for number in 1..=9 {
        crate::step(&mut domain, &env, Event::Withdraw { entry: number }, &mut out);
        assert_eq!(out.pop(), Some(Request::Outcome { entry: number, task: 7, outcome: Outcome::Withdrawn }));
    }
}
