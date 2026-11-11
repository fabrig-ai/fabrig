//! Unpushed changes read only on what can push them (keeping-up.md, 2).
use super::*;
use skein_forge::api::{Change, Error, Hint, Op, Read};
use temper_engine_domain_forge_change as change;

fn unpushed(e: &Env<Limits>) -> Domain {
    let mut d = domain_with_limits(&e.limits);
    outputs(
        &mut d,
        Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } },
    );
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.delegate = Some((71, change::Delegate::Produce));
    row.delegate_status = change::Status::Unknown;
    row.change.last_head = None;
    row.change.state = change::State::Producing { requested: false };
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d.offered_change(fixtures::CHANGE);
    d
}

fn waiting() -> Event {
    Event::StepChange {
        task: fixtures::CHANGE,
        entry: 100,
        heard: change::Heard {
            delegate_running: false,
            delegate_live: false,
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Unknown,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        pending: Box::new([change::Pending { task: 40, kind: change::PendingKind::Other }]),
        kept: false,
        merge_job: false,
    }
}

fn requests(d: &mut Domain, e: &Env<Limits>, event: Event) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    step(d, e, event, &mut out);
    for _ in 0_u32..16 {
        resume(d, e, &mut out);
    }
    let mut seen = skein_lib::List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        seen.push(request).expect("sized from outputs");
    }
    seen.into_boxed()
}

fn absent(d: &mut Domain, e: &Env<Limits>, seen: &[Request]) {
    let mut calls = skein_lib::List::with_capacity(2);
    for request in seen {
        if let Request::Call { call, op, .. } = request {
            calls.push((*call, op.clone())).expect("only one branch read");
        }
    }
    let [(call, op)] = calls.as_slice() else { panic!("one branch read: {seen:?}") };
    assert_eq!(*op, Op::Read(Read::Branch { branch: b"temper/51".as_slice().into() }));
    let answered = requests(
        d,
        e,
        Event::Client(skein_forge_client::Event::Answered {
            call: *call,
            cost: 1,
            free: 0,
            rate: None,
            result: Err(Error::Missing),
        }),
    );
    assert!(!has_call(&answered), "no base, PullFor or CI: {answered:?}");
    let mut waiting = false;
    for request in &answered {
        if let Request::ChangeDecision { decision: change::Decision::Wait { until: None }, .. } = request {
            waiting = true;
        }
    }
    assert!(waiting, "the task dependency stays pending: {answered:?}");
}

fn has_call(seen: &[Request]) -> bool {
    for request in seen {
        if let Request::Call { .. } = request {
            return true;
        }
    }
    false
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one due-cause table shares the first absent read and checks each wake's reads"
)]
fn an_unpushed_changes_branch_is_read_only_when_due() {
    for cause in [
        "dependency",
        "before fallback",
        "fallback",
        "named push",
        "unnamed push",
        "base",
        "commit",
        "item",
        "producer",
        "standing result",
        "lost writer",
        "restart",
        "zero fallback",
    ] {
        let mut e = env();
        e.limits.client.fallback = Duration::from_secs(300);
        let mut d = unpushed(&e);
        let first = requests(&mut d, &e, waiting());
        absent(&mut d, &e, &first);
        let expected = match cause {
            "dependency" => false,
            "before fallback" => {
                e.now = e.now.saturating_add(Duration::from_secs(299));
                false
            }
            "fallback" => {
                e.now = e.now.saturating_add(Duration::from_secs(300));
                true
            }
            "named push" | "unnamed push" | "base" | "commit" | "item" => {
                let change = match cause {
                    "named push" => Change::Branch(b"temper/51".as_slice().into()),
                    "unnamed push" => Change::Branch(Box::new([])),
                    "base" => Change::Branch(b"main".as_slice().into()),
                    "commit" => Change::Commit([99; 32]),
                    "item" => Change::Item(99),
                    _ => unreachable!(),
                };
                let seen = requests(&mut d, &e, Event::Hint { hint: Hint { repository: REPO, change, key: None } });
                let mut wakes = false;
                for request in &seen {
                    if let Request::WakeProcedure { task: fixtures::CHANGE } = request {
                        wakes = true;
                    }
                }
                assert_eq!(wakes, cause == "named push" || cause == "unnamed push", "{cause}: {seen:?}");
                wakes
            }
            "producer" => {
                requests(
                    &mut d,
                    &e,
                    Event::DelegateResult {
                        task: fixtures::CHANGE,
                        child: 71,
                        status: change::Status::Unknown,
                        no_change: false,
                        failed: false,
                        cancelled: false,
                        words: Box::new([]),
                    },
                );
                true
            }
            "standing result" => true,
            "lost writer" => {
                let resource = run_branch(b"51");
                requests(
                    &mut d,
                    &e,
                    Event::Hold { entry: 0, task: fixtures::CHANGE, resource: resource.clone(), from: None },
                );
                requests(
                    &mut d,
                    &e,
                    Event::Claim {
                        task: 71,
                        attempt: 1,
                        writes: Box::new([resource]),
                        designs: Box::new([]),
                        holders: Box::new([fixtures::CHANGE]),
                    },
                );
                let lost = requests(&mut d, &e, Event::Lost { task: 71, attempt: 1 });
                let mut sent = None;
                for request in &lost {
                    if let Request::Call { call, op, .. } = request {
                        assert_eq!(*op, Op::Read(Read::Branch { branch: b"temper/51".as_slice().into() }));
                        assert!(sent.replace(*call).is_none(), "one lost-writer reread");
                    }
                }
                requests(
                    &mut d,
                    &e,
                    Event::Client(skein_forge_client::Event::Answered {
                        call: sent.expect("lost writer rereads even before fallback"),
                        cost: 1,
                        free: 0,
                        rate: None,
                        result: Err(Error::Missing),
                    }),
                );
                true
            }
            "restart" => {
                d = unpushed(&e);
                true
            }
            "zero fallback" => {
                e.limits.client.fallback = Duration::ZERO;
                true
            }
            _ => unreachable!(),
        };
        let mut event = waiting();
        if cause == "standing result"
            && let Event::StepChange { heard, .. } = &mut event
        {
            heard.delegate = change::Status::Passed;
        }
        let seen = requests(&mut d, &e, event);
        assert_eq!(has_call(&seen), expected, "{cause}: {seen:?}");
        if expected {
            absent(&mut d, &e, &seen);
        }
        if cause == "standing result" {
            let mut event = waiting();
            if let Event::StepChange { heard, .. } = &mut event {
                heard.delegate = change::Status::Passed;
            }
            let seen = requests(&mut d, &e, event);
            assert!(!has_call(&seen), "the same result is not another push: {seen:?}");
        }
    }
}

#[test]
fn an_unpushed_changes_undated_wait_uses_fallback() {
    for (head, fallback, deadline, seconds) in [
        (false, 300, None, 300),
        (false, 0, None, 30),
        (true, 300, None, 30),
        (false, 300, Some(Wall::from_nanos(107_000_000_000)), 7),
    ] {
        let mut e = env();
        e.limits.client.fallback = Duration::from_secs(fallback);
        let mut d = unpushed(&e);
        if head {
            d.changes.get_mut(&fixtures::CHANGE).unwrap().change.last_head = Some(fixtures::HEAD);
        }
        d.wait_change(&e, fixtures::CHANGE, deadline);
        assert_eq!(
            d.next_wall_deadline(),
            Some(Wall::from_nanos(e.wall.as_nanos() + Duration::from_secs(seconds).as_nanos()))
        );
    }
}
