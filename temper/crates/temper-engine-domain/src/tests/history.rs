use crate::{Key, Range};
use temper_engine_domain_forge::{HistoryRange as H, Key as F};

#[test]
fn history_ranges_admit_only_their_own_family_project_and_subject() {
    let change = Range::History(H::Change { project: 3, change: 9 });
    let numbers = Range::History(H::Numbers { project: 3, numbers: Box::new([4, 7]) });
    let cases = [
        (F::HistoryChange { project: 3, change: 9, number: 4 }, true, false),
        (F::HistoryChange { project: 3, change: 8, number: 4 }, false, false),
        (F::HistoryChange { project: 2, change: 9, number: 4 }, false, false),
        (F::HistoryEvent { project: 3, number: 4 }, false, true),
        (F::HistoryEvent { project: 3, number: 7 }, false, true),
        (F::HistoryEvent { project: 3, number: 5 }, false, false),
        (F::HistoryEvent { project: 2, number: 4 }, false, false),
        (F::HistoryLanding { project: 3, landing: 9, number: 4 }, false, false),
        (F::HistoryLast { project: 3, change: 9, number: 4 }, false, false),
    ];
    for (key, in_change, in_numbers) in cases {
        let key = Key::Forge(key);
        assert_eq!(change.contains(&key), in_change, "{key:?}");
        assert_eq!(numbers.contains(&key), in_numbers, "{key:?}");
        assert!(!Range::Forge.contains(&key));
    }
    for (range, yes, no) in [
        (
            H::Events { project: 3 },
            F::HistoryEvent { project: 3, number: 4 },
            F::HistoryEvent { project: 2, number: 4 },
        ),
        (
            H::Landing { project: 3, landing: 9 },
            F::HistoryLanding { project: 3, landing: 9, number: 4 },
            F::HistoryLanding { project: 3, landing: 8, number: 4 },
        ),
        (
            H::Last { project: 3 },
            F::HistoryLast { project: 3, number: 4, change: 9 },
            F::HistoryLast { project: 2, number: 4, change: 9 },
        ),
    ] {
        assert!(Range::History(range.clone()).contains(&Key::Forge(yes)));
        assert!(!Range::History(range).contains(&Key::Forge(no)));
    }
    assert!(!change.contains(&Key::Wires));
    let invalid = Range::History(H::Numbers { project: 3, numbers: Box::new([4; 33]) });
    assert!(!invalid.contains(&Key::Forge(F::HistoryEvent { project: 3, number: 4 })));
    assert!(Range::Forge.contains(&Key::Forge(F::HistoryProject { project: 3 })));
}

use crate::{Decision, Delivery, Domain, Event, Limits, Record, Request, StoreInput, StoreRequest, Work};
use skein_lib::{Env, Queue, Token};
use temper_engine_domain_forge as forge;

fn issue(domain: &mut Domain, env: &Env<Limits>, load: u64, most: u32, after: Option<F>) -> Option<Token> {
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut requests = Queue::with_capacity(1);
    requests.push(forge::Request::LoadHistory { load, range: H::Change { project: 3, change: 9 }, after, most });
    crate::route::forge_route::outputs(domain, env, &mut decision, &mut requests);
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
    while let Some(output) = crate::decision::commit(&mut domain.journal, &env.limits.journal) {
        let crate::decision::JournalOutput::Commit { number, .. } = output else { panic!("commit") };
        crate::decision::committed(&mut domain.journal, number);
    }
    let mut released = Queue::with_capacity(1);
    crate::decision::resume(&mut domain.journal, &mut released);
    let Some(crate::decision::JournalOutput::Deliver(Delivery::Load { waiter, range, after })) = released.pop() else {
        return None;
    };
    let mut out = Queue::with_capacity(1);
    crate::restart::request_load(domain, waiter, range.clone(), after.clone(), &mut out);
    let Some(Request::Store(StoreRequest::Load { owner, range: found, after: cursor, most: rows, .. })) = out.pop()
    else {
        return None;
    };
    assert_eq!(found, range);
    assert_eq!(cursor, after);
    assert_eq!(rows, most.min(env.limits.loads.rows));
    Some(owner)
}
fn index(number: u64) -> forge::Stored {
    forge::Stored::HistoryChange(forge::history::ChangeIndex { project: 3, change: 9, number, landing: None })
}
fn terminal(domain: &mut Domain) -> forge::Event {
    let Some(Work::Forge(event)) = domain.work.pop() else { panic!("forge load terminal") };
    event
}
#[test]
fn forge_history_loads_route_ordered_stored_rows_and_the_continuation_back_to_the_waiter() {
    let (mut domain, env) = super::resume_fixture::root();
    let after = F::HistoryChange { project: 3, change: 9, number: 4 };
    let owner = issue(&mut domain, &env, 77, 2, Some(after)).unwrap();
    let next = F::HistoryChange { project: 3, change: 9, number: 6 };
    let mut out = Queue::with_capacity(4);
    crate::route::step_routed(
        &mut domain,
        &env,
        Event::Store(StoreInput::Loaded {
            owner,
            rows: Box::new([Record::Forge { row: Box::new(index(5)) }, Record::Forge { row: Box::new(index(6)) }]),
            next: Some(Key::Forge(next.clone())),
            texts: None,
        }),
        &mut out,
    );
    let forge::Event::HistoryLoaded { load, rows, next: cursor } = terminal(&mut domain) else {
        panic!("loaded history")
    };
    assert_eq!(load, 77);
    assert_eq!(rows.as_ref(), [index(5), index(6)]);
    assert_eq!(cursor, Some(next));
    crate::route::step_routed(&mut domain, &env, Event::Store(StoreInput::Unloaded { owner }), &mut out);
    assert!(domain.work.is_empty(), "a late root load answer is dropped");
}
#[test]
fn refused_failed_and_saturated_history_loads_reach_the_forge_as_unloaded() {
    let (mut domain, env) = super::resume_fixture::root();
    assert!(issue(&mut domain, &env, 71, 0, None).is_none());
    assert_eq!(terminal(&mut domain), forge::Event::HistoryUnloaded { load: 71 });
    domain.reclaim();
    let owner = issue(&mut domain, &env, 72, 1, None).unwrap();
    crate::route::step_routed(
        &mut domain,
        &env,
        Event::Store(StoreInput::Unloaded { owner }),
        &mut Queue::with_capacity(4),
    );
    assert_eq!(terminal(&mut domain), forge::Event::HistoryUnloaded { load: 72 });
    let (mut domain, env) = super::resume_fixture::root();
    for n in 1..=env.limits.loads.loads {
        crate::loads::begin(
            &mut domain.loads,
            Token::new(u64::from(n)),
            Range::Forge,
            None,
            1,
            &mut Queue::with_capacity(1),
        )
        .unwrap();
    }
    assert!(issue(&mut domain, &env, 73, 1, None).is_none());
    assert_eq!(terminal(&mut domain), forge::Event::HistoryUnloaded { load: 73 });
}
