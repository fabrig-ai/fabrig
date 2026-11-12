use super::*;
use jig_domain_people::ApplicationScope;
use temper_engine_domain_forge_change as change;

fn panel(d: &Domain) -> Panel {
    d.panel(fixtures::CHANGE, ApplicationScope::Project(5), env().wall, &LIMITS).expect("known change")
}
fn with_row(row: ChangeRow) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    d
}
#[test]
fn each_step_and_held_step_comes_from_the_retained_record() {
    let head = fixtures::HEAD;
    let since = env().wall;
    let rows = [
        (change::State::Ready { head, ready_since: since }, PanelStep::Ready { landing: None }),
        (
            change::State::Preparing { from: head, main: fixtures::BASE, ready_since: since },
            PanelStep::Preparing { landing: None },
        ),
        (change::State::Producing { requested: true }, PanelStep::Producing),
        (change::State::WaitingProducer { base: fixtures::BASE }, PanelStep::Producing),
        (change::State::Opening { requested: false }, PanelStep::Opening),
        (change::State::Recreating { head }, PanelStep::Recreating),
        (change::State::Reopening { requested: true, retarget: true }, PanelStep::Reopening),
        (change::State::Checking { head }, PanelStep::Checking),
        (change::State::Gating { head, asked: Box::new([]) }, PanelStep::Gating),
        (change::State::Queued { head, ready_since: since }, PanelStep::Queued { place: 1, ahead: Box::new([]) }),
        (change::State::First { head, ready_since: since }, PanelStep::First),
        (change::State::Updating { from: head, base: fixtures::BASE, ready_since: since }, PanelStep::Updating),
        (change::State::Resolving { requested: true, base: fixtures::BASE }, PanelStep::Resolving),
        (
            change::State::Repairing { requested: true, why: change::Repair::Gate(7) },
            PanelStep::Repairing(change::Repair::Gate(7)),
        ),
        (change::State::Landing { head }, PanelStep::Landing),
        (
            change::State::Landed { merge: fixtures::BASE, since: Wall::from_nanos(17) },
            PanelStep::Landed { merge: fixtures::BASE },
        ),
    ];
    for (state, step) in rows {
        for hold in [None, Some(change::Hold::Stalled)] {
            let mut row = fixtures::opening_row();
            row.change.last_head = Some(head);
            row.change.ready_since = Some(since);
            row.change.state = match hold {
                Some(why) => change::State::Held { was: Box::new(state.clone()), why },
                None => state.clone(),
            };
            let d = with_row(row.clone());
            let got = panel(&d);
            assert_eq!(got.step, step);
            assert_eq!(got.way, row.change.way);
            assert_eq!(got.in_landing, row.change.landing);
            assert_eq!(got.hold, hold);
            assert_eq!(got.task, row.task);
            assert_eq!(got.forge, REPO.forge);
            assert_eq!(got.owner.as_ref(), b"org");
            assert_eq!(got.name.as_ref(), b"repo");
            assert_eq!(got.branch, row.branch);
            assert_eq!(got.landing, row.base);
            assert_eq!(got.pull, row.pull);
            assert_eq!(got.head, Some(head));
            assert_eq!(got.repairs, PanelUsed { used: row.change.repairs, limit: LIMITS.change_policy.repairs });
            assert!(got.within());
        }
    }
}
#[test]
fn unknown_subjects_and_other_scopes_have_no_panel() {
    let d = with_row(fixtures::opening_row());
    for (task, scope) in [
        (0, ApplicationScope::Project(5)),
        (999, ApplicationScope::Project(5)),
        (fixtures::CHANGE, ApplicationScope::Project(6)),
        (fixtures::CHANGE, ApplicationScope::Deployment),
        (fixtures::CHANGE, ApplicationScope::Own),
    ] {
        assert_eq!(d.panel(task, scope, env().wall, &LIMITS), None);
    }
}
#[test]
fn every_record_save_marks_its_change_after_the_save() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let row = fixtures::opening_row();
    assert_eq!(
        outputs(&mut d, Event::Change { row: row.clone() }).as_ref(),
        &[
            Request::Save { record: Stored::Change(Box::new(row.clone())), release: None, read_afresh: None },
            Request::Marked { tasks: Box::new([row.task]) },
        ]
    );
}

fn queued(task: u64, priority: i32, seconds: u64) -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.task = task;
    row.change.task = task;
    row.priority = priority;
    let since = Wall::from_nanos(seconds.checked_mul(1_000_000_000).expect("test time"));
    row.change.state = change::State::Queued { head: fixtures::HEAD, ready_since: since };
    row.change.ready_since = Some(since);
    row.pending = None;
    row
}
#[test]
fn queue_place_uses_priority_time_aging_and_the_base_repairs_precedence() {
    type QueueCase = ([(u64, i32, u64); 3], bool, [u64; 3]);
    let cases: [QueueCase; 3] = [
        ([(51, 9, 95), (52, 1, 96), (53, 9, 90)], false, [52, 53, 51]),
        ([(51, 9, 60), (52, 1, 96), (53, 9, 90)], false, [51, 52, 53]),
        ([(51, 9, 95), (52, 1, 96), (53, 9, 90)], true, [51, 52, 53]),
    ];
    for (rows, repair, expected) in cases {
        let mut d = domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        for (task, priority, seconds) in rows {
            let mut row = queued(task, priority, seconds);
            row.base_repair = repair && task == 51;
            outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
        }
        for (index, task) in expected.iter().enumerate() {
            let got = d.panel(*task, ApplicationScope::Project(5), env().wall, &LIMITS).expect("queued panel");
            let mut ahead = skein_lib::List::with_capacity(PANEL_AHEAD);
            for predecessor in expected.get(..index).expect("predecessors").iter().rev() {
                ahead.push(*predecessor).expect("ahead room");
            }
            assert_eq!(
                got.step,
                PanelStep::Queued { place: u32::try_from(index + 1).expect("place"), ahead: ahead.into_boxed() }
            );
        }
    }
}
#[test]
fn a_long_queue_counts_every_place_but_carries_only_the_nearest_ahead() {
    const CHANGES: u32 = PANEL_AHEAD + 3;
    let limits = Limits { changes: CHANGES, ..LIMITS };
    let mut d = Domain::new(
        &limits,
        4,
        Config { push: None, namespace: Box::from(&b"test"[..]), writers: Box::new([]) },
        &[],
        &configured(),
        false,
    )
    .expect("long queue bounds");
    let environment = Env { limits, ..env() };
    d.restore_begin(environment.wall);
    let mut out = Queue::uncapped();
    step(&mut d, &environment, Event::Restore { record: Stored::Repository(adopted()) }, &mut out);
    for task in 1..=u64::from(CHANGES) {
        step(&mut d, &environment, Event::Restore { record: Stored::Change(Box::new(queued(task, 0, 95))) }, &mut out);
    }
    let got = d.panel(u64::from(CHANGES), ApplicationScope::Project(5), env().wall, &limits).expect("last panel");
    let mut ahead = skein_lib::List::with_capacity(PANEL_AHEAD);
    for task in (u64::from(CHANGES - PANEL_AHEAD)..u64::from(CHANGES)).rev() {
        ahead.push(task).expect("ahead room");
    }
    assert_eq!(got.step, PanelStep::Queued { place: CHANGES, ahead: ahead.into_boxed() });
}
#[test]
fn queue_entry_and_departure_mark_the_nearest_followers_within_the_subject_bound() {
    let mut d = with_row(queued(52, 2, 90));
    let expected = queued(51, 1, 90);
    let got = outputs(&mut d, Event::Change { row: expected.clone() });
    assert_eq!(
        got.as_ref(),
        &[
            Request::Save { record: Stored::Change(Box::new(expected)), release: None, read_afresh: None },
            Request::Marked { tasks: Box::new([51, changes_subject(5), status_subject(5), APPROVALS_SUBJECT, 52]) },
        ]
    );
    // Refusing an outstanding delegate holds the queued change, taking it out.
    outputs(&mut d, Event::Delegated { task: 51, child: 90, kind: change::Delegate::Produce });
    let got = outputs(&mut d, Event::DelegateRefused { task: 51, child: 90 });
    let [Request::Save { .. }, Request::Marked { tasks }] = got.as_ref() else { panic!("save and mark: {got:?}") };
    assert_eq!(tasks.as_ref(), &[51, changes_subject(5), status_subject(5), APPROVALS_SUBJECT, 52]);
}
#[test]
fn gates_are_judged_at_the_head_with_failed_and_pending_verdicts_taking_precedence() {
    let mut row = fixtures::opening_row();
    row.change.gates = Box::new([
        change::Gate {
            number: 1,
            kind: change::GateKind::Agent,
            blocking: true,
            freshness: change::Freshness::Exact,
            eager: false,
        },
        change::Gate {
            number: 2,
            kind: change::GateKind::Person,
            blocking: false,
            freshness: change::Freshness::Clean,
            eager: false,
        },
        change::Gate {
            number: 3,
            kind: change::GateKind::Check,
            blocking: true,
            freshness: change::Freshness::Exact,
            eager: false,
        },
    ]);
    row.change.clean = Box::new([fixtures::BASE]);
    row.verdicts = Box::new([
        change::GateReport { number: 1, head: fixtures::BASE, status: change::Status::Passed },
        change::GateReport { number: 2, head: fixtures::BASE, status: change::Status::Passed },
        change::GateReport { number: 3, head: fixtures::HEAD, status: change::Status::Failed },
    ]);
    let got = panel(&with_row(row));
    assert_eq!(
        got.gates.as_ref(),
        &[
            PanelGate { number: 1, kind: change::GateKind::Agent, blocking: true, verdict: change::Status::Unknown },
            PanelGate { number: 2, kind: change::GateKind::Person, blocking: false, verdict: change::Status::Passed },
            PanelGate { number: 3, kind: change::GateKind::Check, blocking: true, verdict: change::Status::Failed },
        ]
    );
}
#[test]
fn a_gate_whose_review_runs_at_the_head_is_pending() {
    let gate = |number| change::Gate {
        number,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    };
    for (head, verdict) in [(fixtures::HEAD, change::Status::Pending), (fixtures::BASE, change::Status::Unknown)] {
        let mut row = fixtures::opening_row();
        row.change.gates = Box::new([gate(1), gate(2)]);
        row.change.state = change::State::Gating { head: fixtures::HEAD, asked: Box::new([1]) };
        row.delegate = Some((90, change::Delegate::Gate { number: 1, head }));
        let got = panel(&with_row(row));
        assert_eq!(got.runs, Some(90));
        assert_eq!(
            got.gates.as_ref(),
            &[
                PanelGate { number: 1, kind: change::GateKind::Agent, blocking: true, verdict },
                PanelGate {
                    number: 2,
                    kind: change::GateKind::Agent,
                    blocking: true,
                    verdict: change::Status::Unknown
                },
            ]
        );
    }
}
#[test]
fn a_fresh_ci_read_marks_a_changed_verdict_and_restart_forgets_it() {
    let (mut d, _) = fixtures::opening();
    fixtures::missing_pull(&mut d);
    let got = fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Pending)));
    let [Request::Save { record: Stored::Change(saved), .. }, Request::Marked { tasks }] = got.as_ref() else {
        panic!("observed contexts saved, panel marked: {got:?}")
    };
    assert_eq!(saved.result.checks.bytes.as_ref(), b"ci pending");
    assert_eq!(tasks.as_ref(), &[fixtures::CHANGE]);
    assert_eq!(panel(&d).ci, change::Status::Pending);
    let row = d.change(fixtures::CHANGE).expect("kept change").clone();
    let mut restarted = with_row(row);
    ready(&mut restarted);
    assert_eq!(panel(&restarted).ci, change::Status::Unknown);
}
#[test]
fn admission_accepts_large_internal_output_and_rejects_oversized_panel_names() {
    let limits = Limits { output: u32::MAX, ..LIMITS };
    assert!(valid(&limits));
    Domain::new(
        &limits,
        4,
        Config { push: None, namespace: Box::from(&b"test"[..]), writers: Box::new([]) },
        &[],
        &configured(),
        false,
    )
    .expect("large internal output does not limit admission");
    assert!(!valid(&Limits { name_bytes: PANEL_NAME_BYTES + 1, ..LIMITS }));
    assert!(valid(&LIMITS));
}

#[test]
fn priority_moves_mark_the_nearest_followers_and_stop_at_the_mark_bound() {
    const CHANGES: u32 = MARKED + 3;
    let limits = Limits { changes: CHANGES, ..LIMITS };
    let mut d = Domain::new(
        &limits,
        4,
        Config { push: None, namespace: Box::from(&b"test"[..]), writers: Box::new([]) },
        &[],
        &configured(),
        false,
    )
    .expect("queue bounds");
    let environment = Env { limits, ..env() };
    d.restore_begin(environment.wall);
    let mut out = Queue::uncapped();
    step(&mut d, &environment, Event::Restore { record: Stored::Repository(adopted()) }, &mut out);
    for task in 1..=u64::from(CHANGES) {
        step(&mut d, &environment, Event::Restore { record: Stored::Change(Box::new(queued(task, 0, 95))) }, &mut out);
    }
    d.priority(&environment, u64::from(CHANGES), -10, &mut out);
    let Some(Request::Save { record: Stored::Change(row), .. }) = out.pop() else { panic!("priority saved") };
    assert_eq!(row.priority, -10_i32);
    let Some(Request::Marked { tasks }) = out.pop() else { panic!("priority marked") };
    let mut expected = skein_lib::List::with_capacity(MARKED);
    expected.push(u64::from(CHANGES)).expect("own task");
    expected.push(changes_subject(5)).expect("project changes");
    expected.push(status_subject(5)).expect("project status");
    expected.push(APPROVALS_SUBJECT).expect("approvals");
    for task in 1..u64::from(MARKED - 3) {
        expected.push(task).expect("nearest followers");
    }
    assert_eq!(tasks, expected.into_boxed());
    assert!(out.is_empty());
    // Moving it behind the same followers also invalidates them in their new order.
    d.priority(&environment, u64::from(CHANGES), 10, &mut out);
    let Some(Request::Save { .. }) = out.pop() else { panic!("priority saved") };
    let Some(Request::Marked { tasks: moved }) = out.pop() else { panic!("move marked") };
    assert_eq!(moved, tasks);
    assert!(out.is_empty());
}
#[test]
fn ci_replacements_mark_once_and_repeated_reads_are_inert() {
    let mut d = with_row(fixtures::opening_row());
    let mut out = Queue::uncapped();
    for status in [change::Status::Pending, change::Status::Passed, change::Status::Failed] {
        let read = change::Ci { head: fixtures::HEAD, status };
        domain::keep_panel_ci(&mut d, fixtures::CHANGE, read, &mut out);
        assert_eq!(out.pop(), Some(Request::Marked { tasks: Box::new([fixtures::CHANGE]) }));
        assert_eq!(panel(&d).ci, status);
        domain::keep_panel_ci(&mut d, fixtures::CHANGE, read, &mut out);
        assert!(out.is_empty());
    }
    domain::keep_panel_ci(
        &mut d,
        fixtures::CHANGE,
        change::Ci { head: fixtures::BASE, status: change::Status::Passed },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Request::Marked { tasks: Box::new([fixtures::CHANGE]) }));
    assert_eq!(panel(&d).ci, change::Status::Unknown);
}

#[test]
fn terminal_panel_rejects_a_saved_verdict_at_another_head() {
    for state in [change::State::Landed { merge: fixtures::BASE, since: Wall::from_nanos(17) }, change::State::Ended] {
        let mut row = fixtures::opening_row();
        row.change.state = state;
        row.ci = Some(change::Ci { head: fixtures::BASE, status: change::Status::Passed });
        assert_eq!(panel(&with_row(row)).ci, change::Status::Unknown);
    }
}

#[test]
fn ending_a_change_saves_its_last_ci_without_an_extra_write() {
    let mut row = fixtures::opening_row();
    row.pending = None;
    let mut d = with_row(row);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let read = change::Ci { head: fixtures::HEAD, status: change::Status::Failed };
    let mut out = Queue::uncapped();
    domain::keep_panel_ci(&mut d, fixtures::CHANGE, read, &mut out);
    let requests = outputs(
        &mut d,
        Event::Release {
            task: fixtures::CHANGE,
            root: fixtures::CHANGE,
            ending: ReleaseEnding::Cancelled,
            reason: None,
            entry: fixtures::OPEN,
        },
    );
    let mut records = skein_lib::List::with_capacity(LIMITS.changes);
    for request in requests {
        if let Request::Save { record: Stored::Change(row), .. } = request {
            records.push(row).expect("fixture changes fit");
        }
    }
    let [saved] = records.as_slice() else { panic!("the finish's one change save") };
    assert_eq!(saved.change.state, change::State::Ended);
    assert_eq!(saved.ci, Some(read));
    assert_eq!(panel(&with_row(*saved.clone())).ci, change::Status::Failed);
}

/// The parent commits every save; replaying them in order restores each key's
/// last value without copying any of the connector's volatile working set.
fn commit_panel_records(records: &mut skein_lib::List<Stored>, requests: Box<[Request]>) {
    for request in requests {
        if let Request::Save { record, .. } = request {
            records.push(record).expect("fixture's durable records fit");
        }
    }
}

#[test]
fn landed_panel_keeps_passed_ci_across_a_restart() {
    use skein_forge::api::{Answer, State};
    use skein_forge_client::RecoveryClock;

    // Repository, initial change, branch and base heads, contexts, project facts, landed change, and settlement.
    const RESTART_RECORDS: u32 = 8;
    let repository = Stored::Repository(adopted());
    let mut records = skein_lib::List::with_capacity(RESTART_RECORDS);
    records.push(repository.clone()).expect("repository fits");
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: repository });
    outputs(&mut d, Event::Restored { clock: RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.pull = Some(fixtures::open_pull().number);
    row.change.state = change::State::Landing { head: fixtures::HEAD };
    commit_panel_records(&mut records, outputs(&mut d, Event::Change { row }));

    fixtures::step_change(&mut d);
    let mut merged = fixtures::open_pull();
    merged.state = State::Closed;
    merged.merged = Some(fixtures::BASE);
    for answer in [
        Answer::Commit(fixtures::HEAD),
        Answer::Commit(fixtures::BASE),
        Answer::Pull(merged),
        fixtures::checks(skein_forge::api::Check::Passed),
        Answer::Commits { commits: Box::new([]), more: false },
        Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
    ] {
        commit_panel_records(&mut records, fixtures::answer_next(&mut d, Ok(answer)));
    }
    let before = panel(&d);
    commit_panel_records(&mut records, outputs(&mut d, Event::Settle));
    drop(d);

    let mut restarted = domain();
    for record in records.into_boxed() {
        outputs(&mut restarted, Event::Restore { record });
    }
    outputs(&mut restarted, Event::Restored { clock: RecoveryClock::Monotonic });
    outputs(&mut restarted, Event::ReadAfresh);
    outputs(&mut restarted, Event::SettleOutbox);
    let after = panel(&restarted);
    assert_eq!(before.step, PanelStep::Landed { merge: fixtures::BASE });
    assert_eq!(after.step, PanelStep::Landed { merge: fixtures::BASE });
    assert_eq!(after.head, Some(fixtures::HEAD));
    assert_eq!((before.ci, after.ci), (change::Status::Passed, change::Status::Passed));
}

#[test]
fn a_turn_change_in_ready_has_a_panel_naming_its_landing() {
    for landing in [None, Some(61)] {
        let mut row = fixtures::opening_row();
        row.change.way = change::Way::Turn;
        row.change.landing = landing;
        row.change.state = change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
        let got = panel(&with_row(row));
        assert_eq!(got.step, PanelStep::Ready { landing });
        assert_eq!(got.way, change::Way::Turn);
        assert_eq!(got.in_landing, landing);
    }
}
#[test]
fn a_preparing_change_has_a_panel() {
    for held in [false, true] {
        let mut row = fixtures::opening_row();
        row.change.way = change::Way::Turn;
        row.change.landing = Some(61);
        let state = change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall };
        row.change.state =
            if held { change::State::Held { was: Box::new(state), why: change::Hold::Stalled } } else { state };
        let got = panel(&with_row(row));
        assert_eq!(got.step, PanelStep::Preparing { landing: Some(61) });
        assert_eq!(got.way, change::Way::Turn);
        assert_eq!(got.in_landing, Some(61));
        assert_eq!(got.hold, if held { Some(change::Hold::Stalled) } else { None });
    }
}

#[test]
fn after_landing_steps_keep_each_tiers_verdict_and_repairing_run() {
    use temper_engine_domain_forge_tiers::{Blame, Failures};
    let merge = fixtures::BASE;
    let lines: Box<[Box<[u8]>]> = Box::new([
        Box::from(b"fast run 2".as_slice()),
        Box::from(b"slow run 1 \xc2\xb7 suspected of many tests".as_slice()),
        Box::from(b"later waiting".as_slice()),
    ]);
    for (state, expected) in [
        (change::State::Landed { merge, since: env().wall }, PanelStep::Verifying { merge, tiers: lines.clone() }),
        (change::State::Red { merge, since: env().wall }, PanelStep::Red { merge, tiers: lines.clone() }),
        (change::State::Cleared { merge }, PanelStep::Cleared { merge, tiers: lines.clone(), uncleared: false }),
        (change::State::Uncleared { merge }, PanelStep::Cleared { merge, tiers: lines.clone(), uncleared: true }),
    ] {
        let mut row = fixtures::opening_row();
        row.change.state = state;
        let mut d = with_row(row);
        let verification = Verification {
            repository: REPO,
            commit: merge,
            changes: Box::new([fixtures::CHANGE]),
            tiers: Box::new([
                TierVerification {
                    tier: Box::from(b"fast".as_slice()),
                    verdict: Some(TierVerdict { run: 1, blame: Blame::Owns(Failures::Many) }),
                    repaired: Some(TierRepair { run: 2, tip: fixtures::HEAD, by: None }),
                },
                TierVerification {
                    tier: Box::from(b"slow".as_slice()),
                    verdict: Some(TierVerdict { run: 1, blame: Blame::Suspect(Failures::Many) }),
                    repaired: None,
                },
                TierVerification { tier: Box::from(b"later".as_slice()), verdict: None, repaired: None },
            ]),
        };
        d.verifications.insert((REPO, merge), verification).expect("verification room");
        assert_eq!(panel(&d).step, expected);
    }
}

#[test]
fn tier_lines_show_runs_in_flight_failed_ids_and_outright_clearance() {
    use temper_engine_domain_forge_tiers as p;
    let merge = fixtures::BASE;
    let mut row = fixtures::opening_row();
    row.change.state = change::State::Landed { merge, since: env().wall };
    let mut d = with_row(row);
    running_tier(&mut d, 7, merge, &[merge]);
    d.verifications
        .insert(
            (REPO, merge),
            Verification {
                repository: REPO,
                commit: merge,
                changes: Box::new([fixtures::CHANGE]),
                tiers: Box::new([
                    TierVerification { tier: Box::from(b"slow".as_slice()), verdict: None, repaired: None },
                    TierVerification {
                        tier: Box::from(b"red".as_slice()),
                        verdict: Some(TierVerdict {
                            run: 4,
                            blame: p::Blame::Owns(p::Failures::Tests(Box::new([
                                Box::from(b"one".as_slice()),
                                Box::from(b"two".as_slice()),
                            ]))),
                        }),
                        repaired: None,
                    },
                    TierVerification {
                        tier: Box::from(b"green".as_slice()),
                        verdict: Some(TierVerdict { run: 5, blame: p::Blame::Cleared }),
                        repaired: None,
                    },
                ]),
            },
        )
        .expect("verification room");
    assert_eq!(
        panel(&d).step,
        PanelStep::Verifying {
            merge,
            tiers: Box::new([
                Box::from(b"slow run 7 running".as_slice()),
                Box::from(b"red run 4 \xc2\xb7 owns 2 tests: one, two".as_slice()),
                Box::from(b"green run 5".as_slice()),
            ])
        }
    );
}

#[test]
fn a_landing_during_the_baseline_waits_for_the_next_run() {
    let mut d = waiting_changes(&[(fixtures::CHANGE, fixtures::HEAD)]);
    running_tier(&mut d, 1, fixtures::BASE, &[]);
    assert_tier_panel(&d, fixtures::CHANGE, fixtures::HEAD, b"slow waiting");
    running_tier(&mut d, 2, fixtures::HEAD, &[fixtures::HEAD]);
    assert_tier_panel(&d, fixtures::CHANGE, fixtures::HEAD, b"slow run 2 running");
}

#[test]
fn later_landings_wait_until_the_next_run_covers_them() {
    const NEXT_CHANGE: u64 = 52;
    const LAST_CHANGE: u64 = 53;
    const LAST_TIP: skein_forge::api::Commit = [6; 32];
    let later = [(NEXT_CHANGE, fixtures::HEAD), (LAST_CHANGE, LAST_TIP)];
    let mut d = waiting_changes(&[(fixtures::CHANGE, fixtures::BASE), later[0], later[1]]);
    running_tier(&mut d, 2, fixtures::BASE, &[fixtures::BASE]);
    assert_tier_panel(&d, fixtures::CHANGE, fixtures::BASE, b"slow run 2 running");
    for (task, merge) in later {
        assert_tier_panel(&d, task, merge, b"slow waiting");
    }
    running_tier(&mut d, 3, LAST_TIP, &[fixtures::HEAD, LAST_TIP]);
    for (task, merge) in later {
        assert_tier_panel(&d, task, merge, b"slow run 3 running");
    }
}

fn waiting_changes(changes: &[(u64, skein_forge::api::Commit)]) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for (task, merge) in changes {
        let mut row = fixtures::opening_row();
        row.task = *task;
        row.change.task = *task;
        row.change.state = change::State::Landed { merge: *merge, since: env().wall };
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
        d.verifications
            .insert(
                (REPO, *merge),
                Verification {
                    repository: REPO,
                    commit: *merge,
                    changes: Box::new([*task]),
                    tiers: Box::new([TierVerification {
                        tier: Box::from(b"slow".as_slice()),
                        verdict: None,
                        repaired: None,
                    }]),
                },
            )
            .expect("verification room");
    }
    d
}

fn assert_tier_panel(d: &Domain, task: u64, merge: skein_forge::api::Commit, line: &[u8]) {
    let shown = d.panel(task, ApplicationScope::Project(5), env().wall, &LIMITS).expect("known change");
    assert_eq!(shown.step, PanelStep::Verifying { merge, tiers: Box::new([Box::from(line)]) });
}

fn running_tier(d: &mut Domain, number: u64, tip: skein_forge::api::Commit, commits: &[skein_forge::api::Commit]) {
    use temper_engine_domain_forge_tiers as p;
    let tier: Box<[u8]> = Box::from(b"slow".as_slice());
    let mut landings = skein_lib::List::uncapped();
    for commit in commits {
        landings.push(p::Landing { commit: *commit, key: *commit }).expect("landing room");
    }
    d.tiers
        .insert(
            (REPO, tier.clone()),
            TierRow {
                observations: Box::new([]),
                repository: REPO,
                tier,
                task: Some(61),
                requested: false,
                cancelled: false,
                purpose: None,
                pending: Box::new([]),
                baseline: None,
                branch: TierBranch::Absent,
                probe: None,
                flight: None,
                held: None,
                result: None,
                received: false,
                goals: Box::new([]),
                goal: None,
                started: None,
                record: p::Record {
                    sequence: number,
                    judged: None,
                    failing: p::Failures::Tests(Box::new([])),
                    flakes: Box::new([]),
                    run: Some(p::Run {
                        number,
                        tip,
                        key: tip,
                        landings: landings.into_boxed(),
                        part: p::Part::Whole,
                        faults: 0,
                    }),
                },
            },
        )
        .expect("tier room");
}

#[test]
fn a_panel_shows_its_questions() {
    let mut row = fixtures::opening_row();
    row.questions = change::Questions {
        route: change::Route::Up,
        taken: 2,
        open: Box::new([
            change::Open {
                asker: 71,
                number: 1,
                line: Box::from(b"Which branch?".as_slice()),
                words: Box::from(b"Which branch?".as_slice()),
                stands: change::Stands::Asked,
            },
            change::Open {
                asker: 72,
                number: 2,
                line: Box::from(b"Which check?".as_slice()),
                words: Box::from(b"Which check?".as_slice()),
                stands: change::Stands::Taken,
            },
            change::Open {
                asker: 73,
                number: 3,
                line: Box::from(b"Which dependency?".as_slice()),
                words: Box::from(b"Which dependency?".as_slice()),
                stands: change::Stands::Held { answer: Box::from(b"answer words stay private".as_slice()) },
            },
        ]),
    };
    let got = panel(&with_row(row));
    assert_eq!(
        got.questions,
        PanelQuestions {
            route: change::Route::Up,
            taken: PanelUsed { used: 2, limit: LIMITS.change_policy.questions },
            open: Box::new([
                PanelQuestion {
                    asker: 71,
                    number: 1,
                    line: Box::from(b"Which branch?".as_slice()),
                    stands: PanelStands::Asked
                },
                PanelQuestion {
                    asker: 72,
                    number: 2,
                    line: Box::from(b"Which check?".as_slice()),
                    stands: PanelStands::Taken
                },
                PanelQuestion {
                    asker: 73,
                    number: 3,
                    line: Box::from(b"Which dependency?".as_slice()),
                    stands: PanelStands::Held
                },
            ]),
        }
    );
    assert!(got.within());
}

#[test]
fn a_panel_shows_at_most_eight_questions() {
    let mut row = fixtures::opening_row();
    let mut open = skein_lib::List::with_capacity(9);
    for number in 1..=9 {
        open.push(change::Open {
            asker: 71,
            number,
            line: Box::from(b"Question".as_slice()),
            words: Box::from(b"Question".as_slice()),
            stands: change::Stands::Taken,
        })
        .expect("nine questions fit the record");
    }
    row.questions.open = open.into_boxed();
    let got = panel(&with_row(row));
    assert_eq!(got.questions.open.len(), 8);
    for (index, question) in got.questions.open.iter().enumerate() {
        assert_eq!(
            question.number,
            u64::try_from(index.checked_add(1).expect("question index fits")).expect("question number")
        );
    }
}

#[test]
fn a_panel_bounds_question_lines_in_bytes_and_open_questions() {
    let mut got = panel(&with_row(fixtures::opening_row()));
    // Two-byte UTF-8 words make the bound a byte count rather than a character count.
    let line = "é".repeat(change::QUESTION_LINE / 2).into_bytes().into_boxed_slice();
    got.questions.open = Box::new([PanelQuestion { asker: 71, number: 1, line, stands: PanelStands::Asked }]);
    assert!(got.within());
    got.questions.open[0].line = Box::from("é".repeat(change::QUESTION_LINE / 2 + 1).as_bytes());
    assert!(!got.within());
    let mut open = skein_lib::List::with_capacity(9);
    for number in 1..=9 {
        open.push(PanelQuestion { asker: 71, number, line: Box::new([]), stands: PanelStands::Taken })
            .expect("fixture room");
    }
    got.questions.open = open.into_boxed();
    assert!(!got.within());
}

#[test]
fn red_panels_keep_all_sixteen_full_length_test_ids() {
    use temper_engine_domain_forge_tiers::{Blame, Failures};
    let merge = fixtures::BASE;
    let mut row = fixtures::opening_row();
    row.change.state = change::State::Red { merge, since: env().wall };
    let mut d = with_row(row);
    let mut ids = skein_lib::List::with_capacity(16);
    for byte in b'a'..=b'p' {
        ids.push(Box::<[u8]>::from([byte; 256])).unwrap();
    }
    d.verifications
        .insert(
            (REPO, merge),
            Verification {
                repository: REPO,
                commit: merge,
                changes: Box::new([fixtures::CHANGE]),
                tiers: Box::new([TierVerification {
                    tier: Box::from(b"slow".as_slice()),
                    verdict: Some(TierVerdict {
                        run: 3,
                        blame: Blame::Owns(Failures::Tests(ids.clone().into_boxed())),
                    }),
                    repaired: None,
                }]),
            },
        )
        .unwrap();
    let shown = panel(&d);
    assert!(shown.within());
    let PanelStep::Red { tiers, .. } = shown.step else { panic!("red panel") };
    assert!(tiers[0].starts_with(b"slow run 3 \xc2\xb7 owns 16 tests: "));
    for id in ids.as_slice() {
        assert!(skein_lib::bytes::find(&tiers[0], id).is_some());
    }
}
