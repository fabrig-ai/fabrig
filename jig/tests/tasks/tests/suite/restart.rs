use jig_domain_tasks::{Active, End, Event, Key, Party, Phase, Refusal, Stored, TaskResult};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::Wall;

const BEFORE_CLAIM_ACCOUNT: u32 = 71;
const DURABLE_CLAIM_ACCOUNT: u32 = 72;

#[test]
fn make_and_claim_have_independent_before_and_after_durable_cuts() {
    let mut w = World::new(20, LIMITS);
    let reply_to = w.to();
    w.stage(Event::Make { reply_to, creator: Party::Person(1), batch: Box::new([task(1, &[])]) });
    assert_eq!(w.live(), 0);
    assert!(w.activations.is_empty());
    w.restart();
    assert_eq!(w.live(), 0);
    let reply_to = w.to();
    w.stage(Event::Make { reply_to, creator: Party::Person(1), batch: Box::new([task(2, &[])]) });
    w.durable();
    assert!(w.activations.is_empty());
    w.restart();
    assert!(w.activations.contains(&2));
    let reply_to = w.to();
    w.send(Event::Prepare { reply_to, task: 2 });
    let reply_to = w.to();
    w.stage(Event::Claim {
        account: Some(BEFORE_CLAIM_ACCOUNT),
        reply_to,
        task: 2,
        attempt: 10,
        resumed: false,
        budget: 100,
        writes: Box::new([]),
    });
    assert!(w.runs.is_empty());
    w.restart();
    assert_eq!(w.record(2).attempt, 0);
    assert_eq!(w.record(2).kept_account, None);
    assert!(w.activations.contains(&2));
    let reply_to = w.to();
    w.send(Event::Prepare { reply_to, task: 2 });
    let reply_to = w.to();
    w.stage(Event::Claim {
        account: Some(DURABLE_CLAIM_ACCOUNT),
        reply_to,
        task: 2,
        attempt: 11,
        resumed: false,
        budget: 100,
        writes: Box::new([]),
    });
    w.durable();
    assert!(w.runs.is_empty());
    w.restart();
    assert_eq!(w.runs.get(&2), Some(&11));
    assert_eq!(w.record(2).kept_account, Some(DURABLE_CLAIM_ACCOUNT));
    assert_eq!(w.record(2).phase, Phase::Active(Active::Claimed { attempt: 11 }));
    w.finish(2);
    w.settle(2);
}

#[test]
fn stale_claims_and_replayed_terminal_are_and_do_not_mutate() {
    let mut w = World::new(22, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim(1, 4);
    let reply_to = w.to();
    w.send(Event::Claim {
        account: None,
        reply_to,
        task: 1,
        attempt: 4,
        resumed: false,
        budget: 100,
        writes: Box::new([]),
    });
    assert!(
        matches!(w.replies.last_key_value().expect("reply").1, Reply::Refused(problem) if problem.why == Refusal::Attempt)
    );
    w.terminal(1, End::Refused);
    let record = w.record(1).clone();
    let reply_to = w.to();
    w.send(Event::Activation {
        saved: None,
        reply_to,
        task: 1,
        attempt: 4,
        end: End::Failed(jig_domain_tasks::Class::Lost),
        cause: jig_domain_tasks::Cause::Unpriced,
    });
    assert_eq!(w.record(1), &record);
    assert!(matches!(
        w.replies.last_key_value().expect("reply").1,
        Reply::Acknowledged(jig_domain_tasks::Accepted::Already)
    ));
}

#[test]
fn a_crash_before_finished_decision_resumes_current_attempt() {
    let mut w = World::new(24, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim(1, 1);
    let started = w.record(1).run_since;
    assert!(started.is_some());
    let reply_to = w.to();
    w.stage(Event::Activation {
        saved: None,
        reply_to,
        task: 1,
        attempt: 1,
        end: End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false },
        cause: jig_domain_tasks::Cause::Unpriced,
    });
    assert!(w.results.is_empty());
    assert!(w.closing.is_empty());
    w.restart();
    assert_eq!(w.runs.get(&1), Some(&1));
    assert_eq!(w.record(1).run_since, started);
    w.finish(1);
    w.settle(1);
}

#[test]
fn restore_rejects_corrupt_links_cycles_and_contracts_without_panicking() {
    use jig_domain_tasks::{Request, step};
    use skein_lib::{Env, Queue, Time, Wall};
    let mut source = World::new(25, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[])]);
    for corrupt in 0..13 {
        let mut domain = jig_tasks_world::domain(&LIMITS, 25);
        let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
        let mut out = Queue::uncapped();
        for number in [1, 2] {
            let mut record = source.record(number).clone();
            match corrupt {
                0 => record.root = 99,
                1 => {
                    record.phase = Phase::Waiting;
                    record.dependencies = Box::new([3 - number]);
                    record.waiting_on = Box::new([3 - number]);
                }
                2 => record.contract = jig_domain_tasks::Contract::Report { words: 33 },
                3 => {
                    if number == 1 {
                        record.delegates = Box::new([2]);
                    } else {
                        record.requester = Party::Task(1);
                        record.root = 1;
                        record.depth = 1;
                    }
                }
                4 => record.result_followups = Box::new([3]),
                5..=7 => {
                    record.contract = jig_domain_tasks::Contract::Verdict {
                        choices: Box::new([jig_domain_tasks::Verdict {
                            fields: Box::new([]),
                            code: 7,
                            words: 32,
                            followups: 2,
                        }]),
                    };
                    record.phase = Phase::Closing(jig_domain_tasks::Closing {
                        stage: jig_domain_tasks::Stage::Delegates,
                        ending: jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Verdict {
                            fields: Box::new([]),
                            code: 7,
                            words: Box::new([]),
                        }),
                    });
                    record.result_followups = match corrupt {
                        5 => Box::new([3, 3]),
                        6 => Box::new([3, 4, 5]),
                        7 => Box::new([0, number]),
                        _ => unreachable!(),
                    };
                }
                8 => record.run_since = Some(Wall::EPOCH),
                9 => {
                    record.attempt = 1;
                    record.phase = Phase::Active(Active::Running { attempt: 1 });
                }
                10 => record.phase = Phase::Active(Active::BackingOff { until: Wall::EPOCH }),
                11 => {
                    record.phase = Phase::Active(Active::BackingOff { until: Wall::EPOCH });
                    record.retry_notice = Some(jig_domain_tasks::RetryNotice::Refused { count: 1 });
                }
                12 => {
                    record.attempt = 1;
                    record.phase = Phase::Active(Active::Claimed { attempt: 1 });
                    record.run_since = Some(Wall::EPOCH);
                }
                _ => unreachable!(),
            }
            step(&mut domain, &env, Event::Restore { record: Stored::Live(Box::new(record)) }, &mut out);
        }
        step(&mut domain, &env, Event::Check, &mut out);
        let mut rejected = false;
        while let Some(request) = out.pop() {
            if matches!(request, Request::RestoreRefused { .. }) {
                rejected = true;
            }
        }
        assert!(rejected);
    }
}

#[test]
fn terminal_and_settlement_each_have_a_durable_cut_before_delivery() {
    let mut w = World::new(26, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim(1, 1);
    let reply_to = w.to();
    w.stage(Event::Activation {
        saved: None,
        reply_to,
        task: 1,
        attempt: 1,
        end: End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false },
        cause: jig_domain_tasks::Cause::Unpriced,
    });
    w.durable();
    assert!(w.closing.is_empty());
    assert!(w.results.is_empty());
    w.restart();
    assert!(w.closing.contains(&1));
    assert!(!w.runs.contains_key(&1));
    w.stage(Event::EffectsSettled { task: 1 });
    w.durable();
    assert!(w.results.is_empty());
    w.restart();
    w.observe(jig_tasks_world::referee::Seen::Settled { task: 1 });
    w.stage(Event::Settled { task: 1 });
    w.durable();
    assert_eq!(w.results.len(), 1);
    assert!(!w.records.contains_key(&Key::Live(1)));
    w.restart();
    assert_eq!(w.results.len(), 1);
    assert!(w.closing.is_empty());
}

#[test]
fn wall_correction_does_not_move_a_live_backoff_but_restore_reprojects_it() {
    use skein_lib::{Duration, Wall};
    let mut w = World::new(28, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim(1, 1);
    w.terminal(1, End::Refused);
    w.env.wall = Wall::from_nanos(Duration::from_secs(100).as_nanos());
    w.advance();
    assert!(w.activations.contains(&1));
    w.claim(1, 2);
    w.terminal(1, End::Refused);
    w.env.wall = Wall::from_nanos(Duration::from_secs(200).as_nanos());
    w.restart();
    w.advance();
    assert!(w.activations.contains(&1));
}

#[test]
fn delegate_cancel_cut_after_durability_stops_adopted_runs_then_closes_deepest() {
    let mut w = World::new(27, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[])]);
    w.claim(1, 1);
    w.make(Party::Task(1), vec![task(2, &[])]);
    w.claim(2, 2);
    w.make(Party::Task(2), vec![task(3, &[])]);
    w.claim(3, 3);
    w.observe(jig_tasks_world::referee::Seen::Cancelled { task: 2 });
    let reply_to = w.to();
    w.stage(Event::Activation {
        saved: None,
        reply_to,
        task: 1,
        attempt: 1,
        end: End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: true },
        cause: jig_domain_tasks::Cause::Unpriced,
    });
    w.durable();
    assert!(w.stops.is_empty());
    w.restart();
    assert_eq!(w.stops.len(), 2);
    w.complete_cancel();
    assert_eq!(w.results.len(), 3);
}

#[test]
fn dependency_progress_survives_restore_without_loading_historical_ends() {
    let mut w = World::new(29, LIMITS);
    w.make(Party::Person(1), vec![task(1, &[]), task(2, &[]), task(3, &[1, 2])]);
    w.claim(1, 1);
    w.finish(1);
    w.settle(1);
    assert_eq!(w.record(3).waiting_on.as_ref(), [2]);
    let before = w.records.clone();
    w.restart();
    assert_eq!(w.records, before);
    assert!(!w.activations.contains(&3));
    w.claim(2, 2);
    w.finish(2);
    w.settle(2);
    assert!(w.record(3).waiting_on.is_empty());
    w.restart();
    w.claim(3, 3);
    w.finish(3);
    w.settle(3);
    assert_eq!(w.results.len(), 3);
}

#[test]
fn restore_refuses_unrepresentable_eventual_funding_postings() {
    use jig_domain_tasks::{Cause, Funder, Request, step};
    use skein_lib::Queue;
    let mut source = World::new(30, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    source.claim(1, 1);
    source.terminal_cause(
        1,
        End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false },
        Cause::Priced { cumulative: 1 },
    );
    let mut rows = source.records.values().cloned().collect::<Vec<_>>();
    for row in &mut rows {
        if let Stored::Ledger(ledger) = row
            && ledger.funder == (Funder::Period { project: 1, period: 0 })
        {
            // Keep the row individually representable; only the eventual
            // aggregate with the live task overflows at Check.
            ledger.numbers.budget = u64::MAX;
            ledger.numbers.reserved = 0;
            ledger.numbers.spent_below = u64::MAX;
        }
    }
    let mut domain = jig_tasks_world::domain(&LIMITS, 30);
    let mut out = Queue::uncapped();
    for record in rows {
        step(&mut domain, &source.env, Event::Restore { record }, &mut out);
    }
    step(&mut domain, &source.env, Event::Check, &mut out);
    assert!(std::iter::from_fn(|| out.pop()).any(|request| matches!(request, Request::RestoreRefused { .. })));
}

#[test]
fn malformed_unfinished_dependencies_refuse_at_restore_entrance() {
    use jig_domain_tasks::{Request, step};
    use skein_lib::Queue;
    let mut source = World::new(31, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    for waiting in [
        Box::new([99_u64]) as Box<[u64]>,
        (1..=u64::from(LIMITS.dependencies) + 1).collect::<Vec<_>>().into_boxed_slice(),
    ] {
        let mut record = source.record(1).clone();
        record.waiting_on = waiting;
        let mut domain = jig_tasks_world::domain(&LIMITS, 31);
        let mut out = Queue::uncapped();
        step(&mut domain, &source.env, Event::Restore { record: Stored::Live(Box::new(record)) }, &mut out);
        assert!(
            matches!(out.pop(), Some(Request::RestoreRefused { .. })),
            "oversize/non-subset refused before Restored"
        );
        assert!(out.pop().is_none());
    }
}

#[test]
fn restore_refuses_a_missing_unfinished_live_dependency_before_activation() {
    use jig_domain_tasks::{Request, step};
    use skein_lib::Queue;
    let mut source = World::new(32, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[1])]);
    for phase in [Phase::Waiting, Phase::Active(Active::Due), Phase::Active(Active::Preparing)] {
        let mut rows = source.records.values().cloned().collect::<Vec<_>>();
        for row in &mut rows {
            if let Stored::Live(task) = row
                && task.number == 2
            {
                task.waiting_on = Box::new([]);
                task.phase = phase.clone();
            }
        }
        let mut domain = jig_tasks_world::domain(&LIMITS, 32);
        let mut out = Queue::uncapped();
        for record in rows {
            step(&mut domain, &source.env, Event::Restore { record }, &mut out);
        }
        step(&mut domain, &source.env, Event::Check, &mut out);
        let mut refused = false;
        while let Some(request) = out.pop() {
            assert!(!matches!(request, Request::Activate { .. } | Request::Adopt { .. }));
            refused |= matches!(request, Request::RestoreRefused { .. });
        }
        assert!(refused, "corrupt unfinished subset refuses before activation");
    }
}

#[test]
fn impossible_settled_live_and_inconsistent_ledger_states_refuse_at_restore_entrance() {
    use jig_domain_tasks::{Funder, Request, Stage, step};
    use skein_lib::Queue;
    let mut source = World::new(33, LIMITS);
    source.open_period(1, 300);
    source.carve_pool(1, 200);
    let mut new = task(1, &[]);
    new.funder = Funder::Period { project: 1, period: 1 };
    source.make(Party::Person(1), vec![new]);
    source.claim(1, 1);
    source.finish(1);
    let mut task = source.record(1).clone();
    if let Phase::Closing(closing) = &mut task.phase {
        closing.stage = Stage::Settled;
    } else {
        panic!("closing fixture");
    }
    let mut invalid = vec![Stored::Live(Box::new(task))];
    for row in source.records.values() {
        if let Stored::Ledger(ledger) = row {
            let mut closed = *ledger;
            closed.closed = true;
            closed.numbers.reserved = closed.numbers.reserved.saturating_add(1);
            invalid.push(Stored::Ledger(closed));
            let mut spent = *ledger;
            spent.numbers.spent = 1;
            invalid.push(Stored::Ledger(spent));
        }
    }
    for record in invalid {
        let mut domain = jig_tasks_world::domain(&LIMITS, 33);
        let mut out = Queue::uncapped();
        step(&mut domain, &source.env, Event::Restore { record }, &mut out);
        assert!(matches!(out.pop(), Some(Request::RestoreRefused { .. })));
        assert!(out.pop().is_none());
    }
}

#[test]
fn restore_refuses_task_funding_outside_its_requester_ancestry() {
    use jig_domain_tasks::{Funder, Request, step};
    use skein_lib::Queue;
    let mut source = World::new(34, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[])]);
    let mut rows = source.records.values().cloned().collect::<Vec<_>>();
    for row in &mut rows {
        match row {
            Stored::Live(task) if task.number == 1 => task.numbers.reserved = 100,
            Stored::Live(task) if task.number == 2 => task.funder = Funder::Task(1),
            Stored::Ledger(ledger) => ledger.numbers.reserved = 100,
            Stored::Live(_)
            | Stored::Ended(_)
            | Stored::Writer(_)
            | Stored::Result(_)
            | Stored::EndedByParty(_)
            | Stored::EndedByProject(_)
            | Stored::EndedDelegate(_)
            | Stored::Delegate(_)
            | Stored::Update(_)
            | Stored::Pool(_)
            | Stored::ClosedLedger(_)
            | Stored::DecidedProposal(_)
            | Stored::Stub(_)
            | Stored::Activity(_)
            | Stored::Milestone(_)
            | Stored::History(_)
            | Stored::PlacedHold(_)
            | Stored::PersonProposal(_) => {}
        }
    }
    let mut domain = jig_tasks_world::domain(&LIMITS, 34);
    let mut out = Queue::uncapped();
    for record in rows {
        step(&mut domain, &source.env, Event::Restore { record }, &mut out);
    }
    step(&mut domain, &source.env, Event::Check, &mut out);
    assert!(std::iter::from_fn(|| out.pop()).any(|request| matches!(request, Request::RestoreRefused { .. })));
}

#[test]
fn unresolved_or_inconsistent_escalation_rows_refuse_before_retention() {
    use jig_domain_tasks::{Escalation, EscalationHolder, Hold, Request, Was, step};
    use skein_lib::Queue;
    let mut source = World::new(36, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    for corrupt in 0..6 {
        let mut task = source.record(1).clone();
        task.phase = Phase::Held { was: Was::Active(Active::Due), why: Hold::Deadline, source: None };
        task.escalation = match corrupt {
            0 => Escalation::Routing { revision: 1 },
            1 => Escalation::Unheld { revision: 0 },
            2 => Escalation::Waiting { revision: 0, holder: EscalationHolder::Person(1), entry: 1, since: Wall::EPOCH },
            3 => Escalation::Waiting { revision: 1, holder: EscalationHolder::Person(2), entry: 1, since: Wall::EPOCH },
            4 => Escalation::Waiting {
                revision: 1,
                holder: EscalationHolder::Role { project: 2, role: 0 },
                entry: 1,
                since: Wall::EPOCH,
            },
            5 => Escalation::Rejected {
                revision: 1,
                by: 1,
                reason: vec![b'x'; usize::try_from(LIMITS.result_bytes).expect("configured bounded task result") + 1]
                    .into_boxed_slice(),
            },
            _ => unreachable!(),
        };
        let mut domain = jig_tasks_world::domain(&LIMITS, 36);
        let mut out = Queue::uncapped();
        step(&mut domain, &source.env, Event::Restore { record: Stored::Live(Box::new(task)) }, &mut out);
        assert!(matches!(out.pop(), Some(Request::RestoreRefused { .. })));
        assert!(out.pop().is_none());
    }
}

#[test]
fn waiting_restore_requests_recheck_once_and_identical_recipient_changes_nothing() {
    use jig_domain_tasks::{Escalation, EscalationHolder, Hold, Request, Was, step};
    use skein_lib::Queue;
    let mut source = World::new(37, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    for rejected in [false, true] {
        let mut domain = jig_tasks_world::domain(&LIMITS, 37);
        let mut out = Queue::uncapped();
        for record in source.records.values() {
            if matches!(record, Stored::Milestone(_) | Stored::Activity(_) | Stored::History(_) | Stored::Ended(_)) {
                continue;
            }
            let mut record = record.clone();
            if let Stored::Live(task) = &mut record {
                task.phase = Phase::Held { was: Was::Active(Active::Due), why: Hold::Deadline, source: None };
                task.revisions.hold = 4;
                task.escalation = if rejected {
                    Escalation::Rejected { revision: 4, by: 1, reason: b"no".as_slice().into() }
                } else {
                    Escalation::Waiting {
                        revision: 4,
                        holder: EscalationHolder::Person(1),
                        entry: 1,
                        since: source.env.wall,
                    }
                };
            }
            step(&mut domain, &source.env, Event::Restore { record }, &mut out);
        }
        assert!(out.is_empty());
        step(&mut domain, &source.env, Event::Check, &mut out);
        step(&mut domain, &source.env, Event::Restored, &mut out);
        let requests = std::iter::from_fn(|| out.pop()).collect::<Vec<_>>();
        assert_eq!(
            requests.iter().filter(|row| matches!(row, Request::EscalationNeeded { .. })).count(),
            usize::from(!rejected)
        );
        assert!(!requests.iter().any(|row| matches!(row, Request::Save { .. } | Request::Activate { .. })));
        let before = format!("{domain:?}");
        step(
            &mut domain,
            &source.env,
            Event::RoutedEscalation { task: 1, revision: 4, holder: EscalationHolder::Person(1), entry: 1 },
            &mut out,
        );
        assert!(out.is_empty(), "same selected recipient emits no save");
        assert_eq!(format!("{domain:?}"), before);
    }
}

#[test]
fn waiting_restore_keeps_saved_recipient_entry_and_time_without_routing() {
    use jig_domain_tasks::{Escalation, EscalationHolder, Hold, MessageKind, Request, Was, fire, step};
    use skein_lib::Queue;
    let mut source = World::new_start_restore(37, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[])]);
    let since = Wall::from_nanos(7);
    let entry = 41;
    let states = [
        Escalation::Waiting { revision: 4, holder: EscalationHolder::Task(2), entry, since },
        Escalation::Waiting { revision: 4, holder: EscalationHolder::Person(1), entry, since },
        Escalation::Waiting { revision: 4, holder: EscalationHolder::Role { project: 1, role: 0 }, entry, since },
        Escalation::Rejected { revision: 4, by: 1, reason: b"no".as_slice().into() },
    ];
    for state in states {
        for elapsed in [false, true] {
            let mut env = skein_lib::Env { limits: LIMITS, now: source.env.now, wall: source.env.wall };
            env.wall =
                Wall::from_nanos(since.as_nanos() + if elapsed { LIMITS.escalation_stall.as_nanos() } else { 1 });
            let mut domain = jig_tasks_world::domain(&LIMITS, 37);
            domain.start_restore();
            let mut out = Queue::uncapped();
            for record in source.records.values().filter(|row| row.key().live()) {
                let mut record = record.clone();
                if let Stored::Live(task) = &mut record {
                    if task.number == 1 {
                        task.phase = Phase::Held { was: Was::Active(Active::Due), why: Hold::Deadline, source: None };
                        task.revisions.hold = 4;
                        task.escalation = state.clone();
                    } else {
                        task.phase = Phase::Active(Active::Idle);
                    }
                }
                step(&mut domain, &env, Event::Restore { record }, &mut out);
            }
            assert!(out.is_empty());
            step(&mut domain, &env, Event::Check, &mut out);
            assert!(domain.checked());
            step(&mut domain, &env, Event::Restored, &mut out);
            let requests = std::iter::from_fn(|| out.pop()).collect::<Vec<_>>();
            assert!(
                !requests.iter().any(|row| matches!(
                    row,
                    Request::EscalationNeeded { .. }
                        | Request::EscalationStalled { .. }
                        | Request::RestoreRefused { .. }
                )),
                "restart routes nothing: {requests:?}"
            );
            assert_eq!(domain.task(1).expect("restored held task").escalation, state);
            if matches!(state, Escalation::Waiting { holder: EscalationHolder::Task(2), .. }) {
                let context = requests
                    .iter()
                    .find_map(|request| {
                        if let Request::Activate { context } = request {
                            (context.task == 2).then_some(context)
                        } else {
                            None
                        }
                    })
                    .expect("saved task holder wakes");
                let word = context
                    .inbox
                    .iter()
                    .find(|word| matches!(word.kind, MessageKind::Escalation { task: 1, revision: 4 }))
                    .expect("original virtual escalation");
                assert_eq!(word.number, entry);
                assert_eq!(word.at, since);
                assert_eq!(domain.run_message(2), Some(entry));
            }
            fire(&mut domain, &env, &mut out);
            let stalled = std::iter::from_fn(|| out.pop())
                .filter(|row| matches!(row, Request::EscalationStalled { task: 1, revision: 4, .. }))
                .count();
            let timed = matches!(
                state,
                Escalation::Waiting { holder: EscalationHolder::Task(_) | EscalationHolder::Person(_), .. }
            );
            assert_eq!(stalled, usize::from(elapsed && timed));
            assert_eq!(domain.task(1).expect("held after timer").escalation, state);
        }
    }
}

fn load_and_link(rows: Vec<Stored>, env: &skein_lib::Env<jig_domain_tasks::Limits>) -> jig_domain_tasks::Domain {
    let mut domain = jig_tasks_world::domain(&env.limits, 38);
    domain.start_restore();
    let mut out = skein_lib::Queue::uncapped();
    for record in rows.into_iter().filter(|row| row.key().live()) {
        jig_domain_tasks::step(&mut domain, env, Event::Restore { record }, &mut out);
    }
    jig_domain_tasks::step(&mut domain, env, Event::Check, &mut out);
    assert!(out.is_empty(), "loading and linking must not act: {out:?}");
    assert!(domain.checked());
    domain
}

#[test]
#[should_panic(expected = "restore Live(1): missing task 99")]
fn missing_root_panics_with_record_and_target_after_load() {
    let mut source = World::new_start_restore(38, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    if let Some(Stored::Live(row)) = source.records.get_mut(&Key::Live(1)) {
        row.root = 99;
    }
    load_and_link(source.records.values().cloned().collect(), &source.env);
}

#[test]
#[should_panic(expected = "restore Live(2): missing task 1")]
fn missing_waiting_dependency_panics_with_record_and_target_after_load() {
    let mut source = World::new_start_restore(39, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[1])]);
    source.records.remove(&Key::Live(1));
    load_and_link(source.records.values().cloned().collect(), &source.env);
}

#[test]
#[should_panic(expected = "restore Live(3): missing task or stub 2")]
fn missing_ended_stub_panics_with_record_and_target_after_load() {
    let mut source = World::new_start_restore(40, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    source.make(Party::Task(1), vec![task(2, &[]), task(3, &[])]);
    let reply_to = source.to();
    source.send(Event::Introduce { reply_to, by: 1, left: 2, right: 3 });
    source.claim(2, 2);
    source.finish(2);
    source.settle(2);
    assert!(source.record(3).references.contains(&2));
    assert!(source.records.remove(&Key::Stub(2)).is_some());
    load_and_link(source.records.values().cloned().collect(), &source.env);
}

#[test]
#[should_panic(expected = "restore Live(1): missing funder Period { project: 1, period: 0 }")]
fn missing_funder_panics_with_record_and_target_after_load() {
    let mut source = World::new_start_restore(41, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    source.records.remove(&Key::Ledger(jig_domain_tasks::Funder::Period { project: 1, period: 0 }));
    load_and_link(source.records.values().cloned().collect(), &source.env);
}

#[test]
fn restore_fits_payload_and_collection_bounds_and_keeps_failure_sticky() {
    use jig_domain_tasks::{Request, step};
    let mut source = World::new_start_restore(42, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    for oversize in 0..5 {
        let mut row = source.record(1).clone();
        match oversize {
            0 => row.waiting_on = vec![1; usize::try_from(LIMITS.dependencies + 1).expect("count")].into_boxed_slice(),
            1 => row.spec.words = vec![b'x'; usize::try_from(LIMITS.spec_bytes + 1).expect("bytes")].into_boxed_slice(),
            2 => {
                row.escalation = jig_domain_tasks::Escalation::Rejected {
                    revision: 1,
                    by: 1,
                    reason: vec![b'x'; usize::try_from(LIMITS.result_bytes + 1).expect("bytes")].into_boxed_slice(),
                }
            }
            3 => row.contract = jig_domain_tasks::Contract::Report { words: LIMITS.result_bytes + 1 },
            4 => {
                row.numbers =
                    jig_domain_tasks::Numbers { budget: u64::MAX, spent: u64::MAX, spent_below: 1, reserved: 0 }
            }
            _ => unreachable!(),
        }
        let mut domain = jig_tasks_world::domain(&LIMITS, 42);
        domain.start_restore();
        let mut out = skein_lib::Queue::uncapped();
        step(&mut domain, &source.env, Event::Restore { record: Stored::Live(Box::new(row)) }, &mut out);
        assert!(matches!(out.pop(), Some(Request::RestoreRefused { .. })));
        assert!(out.is_empty());
        step(&mut domain, &source.env, Event::Check, &mut out);
        step(&mut domain, &source.env, Event::Restored, &mut out);
        assert!(!domain.checked());
        assert!(out.is_empty());
    }
}

#[test]
fn restore_does_not_rejudge_shape_phase_authorization_or_funding_conservation() {
    use jig_domain_tasks::{Escalation, EscalationHolder, Funder, Hold, Was};
    let mut source = World::new_start_restore(43, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[])]);
    for change in 0..9 {
        let mut rows = source.records.values().cloned().collect::<Vec<_>>();
        for record in &mut rows {
            if let Stored::Live(row) = record
                && row.number == 1
            {
                match change {
                    0 => {
                        row.created_at = Wall::from_nanos(1);
                        row.last_activity = Wall::EPOCH;
                    }
                    1 => row.run_since = Some(Wall::EPOCH),
                    2 => row.phase = Phase::Active(Active::BackingOff { until: Wall::EPOCH }),
                    3 => {
                        row.phase = Phase::Active(Active::Running { attempt: 99 });
                        row.attempt = 1;
                    }
                    4 => row.escalation = Escalation::Routing { revision: 1 },
                    5 => {
                        row.escalation = Escalation::Waiting {
                            revision: 0,
                            holder: EscalationHolder::Person(2),
                            entry: 0,
                            since: Wall::EPOCH,
                        }
                    }
                    6 => row.phase = Phase::Held { was: Was::Active(Active::Due), why: Hold::Deadline, source: None },
                    7 => row.funder = Funder::Task(2),
                    8 => row.numbers.reserved = 1,
                    _ => unreachable!(),
                }
            }
        }
        load_and_link(rows, &source.env);
    }
}

#[test]
fn live_writer_refuses_invalid_open_flags_and_funding_before_committing() {
    let mut source = World::new_start_restore(44, LIMITS);
    for executor in [
        jig_domain_tasks::Executor::Person(jig_domain_tasks::PersonAddress::Person(1)),
        jig_domain_tasks::Executor::Procedure { connector: 1, code: 1 },
    ] {
        let mut member = task(1, &[]);
        member.executor = executor;
        member.flags.conversation = true;
        member.flags.ended_by = jig_domain_tasks::EndedBy::Requester;
        assert!(matches!(source.make(Party::Person(1), vec![member]), Reply::Refused(_)));
        assert_eq!(source.live(), 0);
    }
    source.make(Party::Person(1), vec![task(1, &[])]);
    let before = source.records.clone();
    let mut unrelated = task(2, &[]);
    unrelated.funder = jig_domain_tasks::Funder::Task(1);
    assert!(matches!(source.make(Party::Person(1), vec![unrelated]), Reply::Refused(_)));
    assert_eq!(source.records, before);
}

#[test]
#[should_panic(expected = "restore Writer(7): missing writer task 99")]
fn missing_writer_task_panics_with_record_and_target_after_load() {
    let mut source = World::new_start_restore(45, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    let mut rows = source.records.values().cloned().collect::<Vec<_>>();
    rows.push(Stored::Writer(jig_domain_tasks::WriterSlot {
        number: 7,
        resource: jig_domain_tasks::Name { connector: 1, path: Box::new([b"r".as_slice().into()]) },
        writer: jig_domain_tasks::Writer::Run { task: 99, attempt: 1 },
        lost: false,
    }));
    load_and_link(rows, &source.env);
}

#[test]
#[should_panic(expected = "restore PlacedHold(7): missing hold owner task 99")]
fn missing_hold_owner_panics_with_record_and_target_after_load() {
    let mut source = World::new_start_restore(46, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    let mut rows = source.records.values().cloned().collect::<Vec<_>>();
    rows.push(Stored::PlacedHold(Box::new(jig_domain_tasks::PlacedHold {
        number: 7,
        project: 1,
        owner: jig_domain_tasks::Owner::Task { task: 99 },
        mode: jig_domain_tasks::Mode::Stop,
        scope: jig_domain_tasks::Scope::Subtree { task: 1 },
        reason: Box::new([]),
        placed_at: 0,
        revision: 1,
        exempt: Box::new([]),
        lifted: false,
    })));
    load_and_link(rows, &source.env);
}

#[test]
fn writer_referee_catches_phase_observations_and_positions_that_restore_does_not_judge() {
    let mut source = World::new_start_restore(47, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    for change in 0..3 {
        let mut rows = source.records.clone();
        let Some(Stored::Live(row)) = rows.get_mut(&Key::Live(1)) else { panic!("live fixture") };
        match change {
            0 => row.run_since = Some(Wall::EPOCH),
            1 => row.question_lines = Box::new([jig_domain_tasks::QuestionLine { message: 0, line: 0, answer: None }]),
            2 => {
                row.phase = Phase::Closing(jig_domain_tasks::Closing {
                    stage: jig_domain_tasks::Stage::Settled,
                    ending: jig_domain_tasks::Ending::Failed { reason: Box::new([]) },
                });
            }
            _ => unreachable!(),
        }
        let mut judge = jig_tasks_world::accounting_referee::Accounting::default();
        judge.reset(&source.records);
        assert!(judge.committed(&rows, &[]).is_err());
        load_and_link(rows.values().cloned().collect(), &source.env);
    }
}

#[test]
fn lifetime_made_counters_do_not_size_retained_live_containers() {
    let mut source = World::new_start_restore(48, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    let mut rows = source.records.values().cloned().collect::<Vec<_>>();
    for row in &mut rows {
        if let Stored::Live(row) = row {
            row.made = u32::MAX;
        }
        if let Stored::Ledger(row) = row {
            row.made = u32::MAX;
        }
    }
    let restored = load_and_link(rows, &source.env);
    assert_eq!(restored.task(1).expect("linked task").made, u32::MAX);
    assert_eq!(
        restored.funding(jig_domain_tasks::Funder::Period { project: 1, period: 0 }).expect("linked period").made,
        u32::MAX
    );
}

#[test]
fn retained_llm_choices_must_fit_the_configured_executor_models() {
    use jig_domain_tasks::{LlmChoice, ModelEffort, Request, step};
    let mut source = World::new_start_restore(49, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[])]);
    for main in [
        ModelEffort { model: 0, effort: Some(1) },
        ModelEffort { model: 3, effort: None },
        ModelEffort { model: 0, effort: Some(2) },
        ModelEffort { model: u8::MAX, effort: None },
    ] {
        let mut rows = source.records.values().filter(|row| row.key().live()).cloned().collect::<Vec<_>>();
        for row in &mut rows {
            if let Stored::Live(row) = row {
                row.llm = Some(LlmChoice { account: None, main, subagents: Box::new([]) });
            }
        }
        let mut domain = jig_tasks_world::domain(&LIMITS, 49);
        domain.start_restore();
        let mut out = skein_lib::Queue::uncapped();
        for record in rows {
            step(&mut domain, &source.env, Event::Restore { record }, &mut out);
        }
        assert!(out.is_empty());
        step(&mut domain, &source.env, Event::Check, &mut out);
        let fits = main == (ModelEffort { model: 0, effort: Some(1) });
        assert_eq!(domain.checked(), fits);
        let refusals =
            std::iter::from_fn(|| out.pop()).filter(|row| matches!(row, Request::RestoreRefused { .. })).count();
        assert_eq!(refusals, usize::from(!fits));
    }
}

#[test]
fn a_completed_dependency_loaded_for_amendment_restores_without_an_optional_stub() {
    let mut source = World::new_start_restore(50, LIMITS);
    source.make(Party::Person(1), vec![task(1, &[]), task(2, &[])]);
    source.claim(1, 1);
    source.finish(1);
    source.settle(1);
    assert!(!source.records.contains_key(&Key::Stub(1)));
    let Some(Stored::Ended(ended)) = source.records.get(&Key::Ended(1)) else { panic!("ended dependency") };
    let place = jig_domain_tasks::DependencyPlace {
        number: 1,
        project: ended.project,
        requester: ended.requester,
        root: ended.root,
        depth: ended.depth,
        status: jig_domain_tasks::Status::Done,
    };
    let reply_to = source.to();
    let call = reply_to.into_token().raw();
    source.send(Event::Amend {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(call)),
        places: Box::new([place]),
        by: Party::Person(1),
        task: 2,
        message: 1,
        stop_run: false,
        funder: None,
        amendment: jig_domain_tasks::Amendment {
            tracked: None,
            addressed: None,
            instructions: None,
            release: None,
            spec: None,
            wake: None,
            dependencies: Some(Box::new([1])),
            authority: None,
            llm: None,
            reason: Box::new([]),
        },
    });
    assert_eq!(source.replies[&call], Reply::Done);
    assert_eq!(source.record(2).dependencies.as_ref(), [1]);
    assert!(source.record(2).waiting_on.is_empty());
    assert!(!source.records.contains_key(&Key::Stub(1)));
    let before = source.record(2).clone();
    source.restart();
    assert_eq!(source.record(2), &before);
}

#[test]
fn start_restore_selects_once_before_rows_check_or_activation() {
    let mut source = World::new(51, LIMITS);
    source.make(Party::Person(9), vec![task(1, &[])]);
    let row = source.records[&Key::Live(1)].clone();
    for explicit in [false, true] {
        for phase in 0..5 {
            let mut domain = jig_tasks_world::domain(&LIMITS, 51);
            if explicit {
                domain.start_restore();
            }
            let mut out = skein_lib::Queue::uncapped();
            match phase {
                0 => tasks_step_row(&mut domain, &source.env, row.clone(), &mut out),
                1 => jig_domain_tasks::step(&mut domain, &source.env, Event::Check, &mut out),
                2 => jig_domain_tasks::step(&mut domain, &source.env, Event::Restored, &mut out),
                3 => {
                    for record in source.records.values().filter(|record| record.key().live()) {
                        tasks_step_row(&mut domain, &source.env, record.clone(), &mut out);
                    }
                    jig_domain_tasks::step(&mut domain, &source.env, Event::Check, &mut out);
                    jig_domain_tasks::step(&mut domain, &source.env, Event::Restored, &mut out);
                    assert!(
                        std::iter::from_fn(|| out.pop())
                            .any(|request| matches!(request, jig_domain_tasks::Request::Activate { .. }))
                    );
                }
                4 => jig_domain_tasks::step(
                    &mut domain,
                    &source.env,
                    Event::Activation {
                        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
                        task: 1,
                        attempt: 1,
                        end: jig_domain_tasks::End::Parked,
                        saved: None,
                        cause: jig_domain_tasks::Cause::Unpriced,
                    },
                    &mut out,
                ),
                _ => unreachable!(),
            }
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| domain.start_restore())).is_err());
        }
    }
    let mut domain = jig_tasks_world::domain(&LIMITS, 51);
    domain.start_restore();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| domain.start_restore())).is_err());
}

fn tasks_step_row(
    domain: &mut jig_domain_tasks::Domain,
    env: &skein_lib::Env<jig_domain_tasks::Limits>,
    record: Stored,
    out: &mut skein_lib::Queue<jig_domain_tasks::Request>,
) {
    jig_domain_tasks::step(domain, env, Event::Restore { record }, out);
}
