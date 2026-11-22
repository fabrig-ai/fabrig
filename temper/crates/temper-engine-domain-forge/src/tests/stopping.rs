use super::fixtures::{answer, next_call};
use super::*;

fn sent() -> (Domain, Token) {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);

    outputs(&mut d, Event::Enqueue { entry: status(1, 7) });
    outputs(&mut d, Event::Committed { entry: 1 });
    let (call, _) = next_call(&mut d);
    answer(
        &mut d,
        call,
        Ok(skein_forge::api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
    );
    let (call, op) = next_call(&mut d);
    assert_eq!(op, skein_forge::api::Op::Write(status(1, 7).effect.write));
    (d, call)
}

#[test]
fn settle_with_no_write_sent_settles_at_once() {
    let mut d = domain();
    assert_eq!(outputs(&mut d, Event::Settle).as_ref(), &[Request::Settled { unfinished: Box::new([]) }]);
}

#[test]
fn settle_waits_for_a_sent_write_and_starts_no_other() {
    let (mut d, call) = sent();
    outputs(&mut d, Event::Enqueue { entry: status(2, 8) });
    outputs(&mut d, Event::Committed { entry: 2 });
    assert!(outputs(&mut d, Event::Settle).is_empty());
    assert!(ready(&mut d).is_empty());
    let requests = outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(skein_forge::api::Answer::Done),
        }),
    );
    assert!(requests.contains(&Request::Erase { key: Key::Entry(1) }));
    assert_eq!(requests.last(), Some(&Request::Settled { unfinished: Box::new([]) }));
    assert!(ready(&mut d).is_empty());
}

#[test]
fn settle_names_a_write_without_its_outcome_at_its_bound() {
    let (mut d, _call) = sent();
    outputs(&mut d, Event::Settle);
    let mut env = env();
    let until = env.now.saturating_add(skein_forge_client::stop_bound(&CLIENT));
    assert_eq!(d.next_deadline(), Some(until));
    let mut out = Queue::uncapped();
    env.now = Time::from_nanos(until.as_nanos().checked_sub(1).unwrap());
    fire(&mut d, &env, &mut out);
    assert!(out.is_empty());
    env.now = until;
    fire(&mut d, &env, &mut out);
    assert_eq!(out.pop(), Some(Request::Settled { unfinished: Box::new([Unfinished { entry: 1, task: 7 }]) }));
}

#[test]
fn settling_during_read_afresh_drops_its_late_report_and_keeps_the_read_facts() {
    use skein_forge::api::{Answer, Op, Read};
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Subscription(Subscriber {
                task: 7,
                number: 1,
                topic: Topic::Landings { repository: REPO, branch: b"main".as_slice().into() },
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            }),
        },
    );
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    let (call, op) = next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    assert_eq!(outputs(&mut d, Event::Settle).as_ref(), &[Request::Settled { unfinished: Box::new([]) }]);
    assert!(ready(&mut d).is_empty(), "settling sends nothing new");
    let facts = answer(&mut d, call, Ok(Answer::Commit([9; 32])));
    assert!(!facts.is_empty(), "the late read still commits facts");
    for fact in facts {
        assert!(matches!(fact, Request::Save { .. } | Request::Erase { .. }), "no step or restart report: {fact:?}");
    }
    assert!(ready(&mut d).is_empty());
}

fn uncertain() -> skein_forge_client::Entry {
    use skein_forge_client::{Attempt, Position};
    let mut entry = status(1, 7);
    entry.effect.write = skein_forge::api::Write::OpenPull {
        title: b"stop".as_slice().into(),
        body: Box::new([]),
        head: b"topic".as_slice().into(),
        base: b"main".as_slice().into(),
    };
    entry.start = Some(Position { at: Time::ZERO, comment: 0, review: 0, review_page: 1 });
    entry.attempt = Some(Attempt {
        sent: Time::ZERO,
        deadline: Time::from_nanos(100),
        wall: Wall::EPOCH,
        expires: Wall::from_nanos(100),
    });
    entry
}

#[test]
fn settling_during_outbox_recovery_drops_its_late_report_and_keeps_the_find_facts() {
    use skein_forge::api::{Answer, Op, Read};
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: uncertain(), held: false } });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    let (call, op) = next_call(&mut d);
    assert_eq!(op, Op::Read(Read::PullFor { head: b"topic".as_slice().into(), base: b"main".as_slice().into() }));
    assert_eq!(outputs(&mut d, Event::Settle).as_ref(), &[Request::Settled { unfinished: Box::new([]) }]);
    assert!(ready(&mut d).is_empty(), "the unanswered find does not hold the stop");
    // A report already on its way when Settle crossed the child boundary is dropped too.
    let mut report = Queue::uncapped();
    domain::restart_outbox_done(&mut d, &env(), &mut report);
    assert!(report.is_empty(), "a late outbox report steps nothing");
    let facts = answer(&mut d, call, Ok(Answer::Pull(fixtures::open_pull())));
    assert!(!facts.is_empty(), "the late find still commits facts");
    for fact in facts {
        assert!(matches!(fact, Request::Save { .. } | Request::Erase { .. }), "no step or restart report: {fact:?}");
    }
    assert!(ready(&mut d).is_empty());
}

#[test]
fn an_entry_committed_while_settling_is_not_made() {
    let (mut d, _call) = sent();
    outputs(&mut d, Event::Enqueue { entry: status(2, 8) });
    outputs(&mut d, Event::Settle);
    assert!(outputs(&mut d, Event::Committed { entry: 2 }).is_empty());
    assert!(ready(&mut d).is_empty());
}
