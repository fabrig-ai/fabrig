//! Stop at each suspended port exchange, before or after the Wires continuation.
use super::{resume::at_phase, resume_fixture as fixture};
use crate::{Domain, Event, Request, StoreInput, StoreRequest, Write};
use jig_domain_core::start::Phase;
use skein_lib::{Env, Queue};
use temper_engine_domain_forge as forge;

fn seeded() -> fixture::StartStore {
    let limits = fixture::limits();
    let mut domain = Domain::new(fixture::config(7), &limits).unwrap();
    let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut store = fixture::StartStore::new();
    crate::step(&mut domain, &env, Event::Restart { pending: false });
    store.run(&mut domain, &env, false);
    store
}

fn forge_input(domain: &mut Domain, env: &Env<crate::Limits>, event: forge::Event) {
    let mut out = Queue::uncapped();
    forge::step(&mut domain.forge, &crate::translate::environment_forge(env), event, &mut out);
    // Restore's report is the completed Link work supplied by at_phase.
    for request in &out {
        let (forge::Request::Up(jig_domain_contract::Up::Lifecycle(jig_domain_contract::lifecycle::Up::Done {
            ..
        }))
        | forge::Request::Save { .. }
        | forge::Request::MarkedForges
        | forge::Request::Marked { .. }) = request
        else {
            panic!("forge fixture: {request:?}")
        };
    }
}

fn prepare_forge(domain: &mut Domain, env: &Env<crate::Limits>, outbox: bool) {
    use skein_forge::api::Repository;
    let repository = Repository { forge: 1, repository: 1 };
    domain.forge.restore_begin(env.wall);
    forge_input(domain, env, forge::Event::Restore { record: forge::Stored::Repository(super::resume::repository()) });
    if !outbox {
        forge_input(
            domain,
            env,
            forge::Event::Restore {
                record: forge::Stored::Subscription(forge::Subscriber {
                    task: 1,
                    number: 1,
                    topic: forge::Topic::Landings { repository, branch: b"main".as_slice().into() },
                    own_change: None,
                    goal_tasks: None,
                    paths: Box::new([]),
                }),
            },
        );
    }
    if outbox {
        let mut second = super::resume::repository();
        second.provider.repository = 2;
        forge_input(domain, env, forge::Event::Restore { record: forge::Stored::Repository(second.clone()) });
        for entry in [uncertain(repository), status(second.provider)] {
            forge_input(domain, env, forge::Event::Restore { record: forge::Stored::Entry { entry, held: false } });
        }
    }
    forge_input(domain, env, forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    forge_input(
        domain,
        env,
        forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"writer".as_slice().into() },
        },
    );
}

use temper_limits_fixtures::start_forge::{status, uncertain};

fn answer(domain: &mut Domain, env: &Env<crate::Limits>, call: skein_lib::Token, result: skein_forge::api::Answer) {
    crate::step(
        domain,
        env,
        Event::Forge(Box::new(forge::Event::Client(skein_forge_client::Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(result),
        }))),
    );
}

/// Answer only work asked before the stop and the drain's commits. Anything outward is a failure.
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "every unlisted output, including a future one, fails the stop's negative contract"
)]
fn drain(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    store: &mut fixture::StartStore,
    mut out: Queue<Request>,
) -> u32 {
    let mut words = 0_u32;
    let mut drained = 0_u32;
    for _ in 0_u32..1024 {
        crate::release(domain, env, &mut out);
        crate::release(domain, env, &mut out);
        if let Some(request) = out.pop() {
            match request {
                Request::ToChild(released) => crate::step(domain, env, Event::Released(released)),
                Request::Store(StoreRequest::Commit { number, writes }) => {
                    assert!(!writes.contains(&Write::Wires), "a stop commits no further mark");
                    store.apply(&writes);
                    crate::step(domain, env, Event::Store(StoreInput::Committed { number, fill: 0 }));
                }
                Request::Restarting { wait } => {
                    assert_eq!(wait, env.limits.fleet.restart_wait);
                    words = words.checked_add(1).unwrap();
                }
                Request::Drained => drained = drained.checked_add(1).unwrap(),
                other => panic!("stopping starts nothing outward: {other:?}"),
            }
        } else if drained == 1 {
            break;
        } else {
            crate::try_step(domain, env, Event::Resume).unwrap();
        }
        domain.reclaim();
    }
    assert_eq!(
        drained,
        1,
        "the start drains without waiting for a read or page: {:?}",
        (domain.start_phase(), &domain.journal, domain.engine.core().stopping())
    );
    crate::try_step(domain, env, Event::Resume).unwrap();
    crate::release(domain, env, &mut out);
    assert!(out.is_empty(), "the drain is said once");
    words
}

fn held_work(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    store: &mut fixture::StartStore,
    phase: Option<Phase>,
    mut owed: Queue<Request>,
    before_mark: bool,
    expected: u32,
) -> Queue<Request> {
    let mut held = Queue::uncapped();
    for _ in 0_u32..128 {
        crate::release(domain, env, &mut owed);
        crate::release(domain, env, &mut owed);
        match owed.pop() {
            Some(Request::ToChild(released)) => crate::step(domain, env, Event::Released(released)),
            Some(Request::Store(StoreRequest::Commit { number, writes })) if !before_mark => {
                store.apply(&writes);
                crate::step(domain, env, Event::Store(StoreInput::Committed { number, fill: 0 }));
            }
            Some(Request::Forge {
                call, op: skein_forge::api::Op::Read(skein_forge::api::Read::Items { .. }), ..
            }) => {
                answer(
                    domain,
                    env,
                    call,
                    skein_forge::api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env.now },
                );
            }
            Some(
                Request::Store(StoreRequest::Catalogue { .. }) | Request::Party(crate::PartyRequest::Catalogue { .. }),
            ) => {}
            Some(request) => held.push(request),
            None if before_mark || phase == Some(Phase::AdoptRuns) || expected == 0 => break,
            None => {
                if held.len() >= expected {
                    break;
                }
                crate::try_step(domain, env, Event::Resume).unwrap();
            }
        }
    }
    assert_eq!(domain.start_phase(), phase, "the row holds its step");
    assert_eq!(held.len(), expected, "each row holds its page, commit, read or find and sent write: {phase:?}");
    held
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "every unlisted held request fails the phase contract")]
#[expect(clippy::too_many_lines, reason = "one table follows each suspended start through the stop and cold reopen")]
fn a_stop_at_each_start_step_drains_and_the_next_start_opens() {
    for (phase, before_mark, held_count, catching_up) in [
        (Some(Phase::LoadCore), true, 1, None),
        (Some(Phase::LoadInbox), true, 1, None),
        (Some(Phase::RestoreConnector { connector: 0 }), true, 1, None),
        (Some(Phase::Link), true, 1, None),
        (Some(Phase::Sweep), true, 1, None),
        (Some(Phase::Switch), true, 1, None),
        (Some(Phase::Wires), true, 1, None),
        // Adoption and opening run in one decision. Hold the durable Wires
        // continuation to stop at its entry, rather than inventing a pause inside it.
        (Some(Phase::Wires), true, 0, None),
        (None, false, 1, Some(false)),
        (None, false, 2, Some(true)),
        (None, false, 0, None),
    ] {
        let mut store = seeded();
        let (mut domain, env) = match phase {
            Some(Phase::Link) => {
                // Declared application defaults make the first Link a writing decision.
                let limits = fixture::limits();
                let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
                let mut domain = Domain::new(super::kinds::connector_restore_config(&limits, true), &limits).unwrap();
                store = fixture::StartStore::new();
                crate::step(&mut domain, &env, Event::Restart { pending: true });
                store.advance(&mut domain, &env, fixture::Until::Phase(Phase::Link));
                (domain, env)
            }
            Some(Phase::Wires) if held_count == 0 => {
                let (mut domain, env) = at_phase(false, Phase::Wires);
                store.run(&mut domain, &env, true);
                (domain, env)
            }
            Some(phase) => at_phase(true, phase),
            None => fixture::root(),
        };
        assert_eq!(domain.start_runtime(), !before_mark, "the root owns the mark boundary");
        let owed = Queue::uncapped();
        if let Some(outbox) = catching_up {
            assert!(domain.ready(), "catching up follows the opening");
            prepare_forge(&mut domain, &env, outbox);
            let row = crate::Record::Forge { row: Box::new(forge::Stored::Repository(super::resume::repository())) };
            store.rows.insert(row.key(), row);
            if outbox {
                let mut repository = super::resume::repository();
                repository.provider.repository = 2;
                let row = crate::Record::Forge { row: Box::new(forge::Stored::Repository(repository)) };
                store.rows.insert(row.key(), row);
            }
            crate::step(&mut domain, &env, Event::Forge(Box::new(forge::Event::CatchUp)));
        }
        // Do not feed the durable Wires continuation into adoption for this row.
        let held = if phase == Some(Phase::Wires) && held_count == 0 {
            assert!(domain.start_phase() == Some(Phase::Wires) && !domain.ready());
            owed
        } else {
            held_work(&mut domain, &env, &mut store, phase, owed, before_mark, held_count)
        };
        let mut owed = held;
        let steps = domain.start_phases.len();
        crate::try_step(&mut domain, &env, Event::Stop).unwrap();
        assert!(domain.engine.core().stopping() || domain.settling.active(), "stop heard at {phase:?}");
        assert_eq!(domain.engine.core().settling(), !before_mark, "settling follows only the mark at {phase:?}");
        let mut out = Queue::uncapped();
        let wires = store.wires;
        for _ in 0..owed.len() {
            match owed.pop().unwrap() {
                Request::Store(StoreRequest::Commit { number, writes }) => {
                    assert_eq!(writes.contains(&Write::Wires), phase == Some(Phase::Wires));
                    store.apply(&writes);
                    crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 }));
                }
                Request::Store(StoreRequest::Load { .. } | StoreRequest::Switch) => {
                    assert!(before_mark, "page still owed");
                    // The stopped start owes no page and asks no following phase.
                }
                Request::Forge { op: skein_forge::api::Op::Read(_), .. } => {
                    assert!(!before_mark, "no forge read before the mark");
                }
                Request::Forge { call, op: skein_forge::api::Op::Write(_), .. } => {
                    assert_eq!(catching_up, Some(true));
                    crate::release(&mut domain, &env, &mut out);
                    assert!(!domain.engine.core().stopping(), "the sent write holds the settled stop");
                    for request in &out {
                        if let Request::Drained = request {
                            panic!("the sent write is still owed");
                        }
                    }
                    answer(&mut domain, &env, call, skein_forge::api::Answer::Done);
                }
                other => panic!("unexpected held start work at {phase:?}: {other:?}"),
            }
        }
        crate::release(&mut domain, &env, &mut out);
        let words = drain(&mut domain, &env, &mut store, out);
        assert_eq!(words, u32::from(!before_mark), "restart word at {phase:?}");
        assert_eq!(domain.start_phases.len(), steps, "no further phase at {phase:?}");
        assert_eq!(store.wires, wires.checked_add(u32::from(phase == Some(Phase::Wires) && held_count != 0)).unwrap());
        assert_eq!(domain.engine.core().counters.deployment().restart_wait_ms.is_some(), !before_mark);
        assert!(domain.engine.core().stopping());
        assert_eq!(domain.start_phase(), phase, "a stopped start is no refused start");
        assert!(domain.start_stop().is_none());
        assert!(domain.work.is_empty());
        let limits = env.limits;
        let config = if phase == Some(Phase::Link) {
            super::kinds::connector_restore_config(&limits, true)
        } else {
            fixture::config(7)
        };
        let mut next = Domain::new(config, &limits).unwrap();
        crate::step(&mut next, &env, Event::Restart { pending: true });
        store.run(&mut next, &env, false);
        assert!(next.ready(), "the cold next start opens after {phase:?}");
    }
}
