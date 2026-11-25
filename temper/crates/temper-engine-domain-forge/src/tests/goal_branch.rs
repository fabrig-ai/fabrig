use super::*;
use skein_forge::api::{Answer, BranchCreation, Error, Op, Read, Write};

const GOAL: u64 = 7;
const OTHER: u64 = 8;
const TAKING: u64 = 21;
const CLOSING: u64 = 22;
const GOAL_BRANCH: &[u8] = b"temper/7/g7";
const TIP: skein_forge::api::Commit = [4; 32];

pub(super) fn name(branch: &[u8]) -> Name {
    let mut parts = skein_lib::List::with_capacity(LIMITS.authority_segments);
    let mut start = 0;
    for index in 0..=branch.len() {
        if index == branch.len() || branch.get(index) == Some(&b'/') {
            parts.push(Box::from(branch.get(start..index).unwrap())).unwrap();
            start = index.checked_add(1).unwrap();
        }
    }
    Name { forge: REPO.forge, repository: REPO.repository, what: What::Branch(parts.into_boxed()) }
}

fn entry() -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number: TAKING,
        task: GOAL,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP },
            condition: skein_forge_client::Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}
fn assert_kept_making(requests: &[Request]) {
    assert!(
        requests.contains(&Request::Save {
            release: Some(TAKING),
            read_afresh: None,
            record: Stored::Entry { entry: entry(), held: false }
        }),
        "{requests:?}"
    );
}
fn assert_no_making(requests: &[Request]) {
    for request in requests {
        if let Request::Save { record: Stored::Entry { .. }, .. } | Request::MakeFailed { .. } = request {
            panic!("no making expected: {request:?}");
        }
    }
}
fn made() -> Request {
    Request::Outcome {
        entry: TAKING,
        task: GOAL,
        account: None,
        outcome: skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Branch(TIP), found: false },
    }
}

fn fresh() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    restore_done(&mut d);
    d
}
fn restore_done(d: &mut Domain) {
    outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(d, Event::CatchUp);
}
fn take(d: &mut Domain) {
    outputs(d, Event::Hold { entry: TAKING, task: GOAL, resource: name(GOAL_BRANCH), from: None });
}
fn missing(d: &mut Domain) -> Box<[Request]> {
    take(d);
    fixtures::answer_next(d, Err(Error::Missing));
    fixtures::answer_next(d, Ok(Answer::Commit(TIP)))
}
fn making(d: &mut Domain) -> (Token, Op) {
    missing(d);
    outputs(d, Event::Committed { entry: TAKING });
    fixtures::answer_next(d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    fixtures::next_call(d)
}

#[test]
fn goal_branch_names_are_canonical_and_distinct_from_other_own_branches() {
    for (branch, expected) in [
        (b"temper/7/g7".as_slice(), Some((7, 7))),
        (b"temper/12/g34", Some((12, 34))),
        (b"temper/0/g0", Some((0, 0))),
        (b"temper/7/c7", None),
        (b"temper/7/r7", None),
        (b"temper/7/s7", None),
        (b"temper/7/l7", None),
        (b"temper/07/g7", None),
        (b"temper/7/g07", None),
        (b"temper/7/g", None),
        (b"temper//g7", None),
        (b"temper/7/g7/extra", None),
        (b"temper/7/g7x", None),
        (b"temper/7/g-7", None),
        (b"temper/+7/g7", None),
        (b"other/7/g7", None),
        (b"temper/18446744073709551616/g7", None),
        (b"temper/7/g18446744073709551616", None),
    ] {
        assert_eq!(items::goal_branch(&name(branch), b"temper/"), expected, "{branch:?}");
    }
    assert_eq!(items::goal_branch(&name(b"team/temper/12/g34"), b"team/temper/"), Some((12, 34)));
    assert_eq!(items::goal_branch(&Name { forge: 1, repository: 2, what: What::Repository }, b"temper/"), None);
}

fn run_watch(restored: bool, goal: bool) {
    let mut d = domain();
    let run = name(b"temper/7/r7");
    let resources: Box<[Name]> =
        if goal { Box::from([run.clone(), name(GOAL_BRANCH)]) } else { Box::from([run.clone()]) };
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    if restored {
        // Store key order can restore names before the goal's holding.
        outputs(&mut d, Event::Restore { record: Stored::Names { task: GOAL, resources: resources.clone() } });
    }
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: run.clone(),
                task: GOAL,
                writer: None,
                beside: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
    );
    if goal {
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Hold(Hold {
                    name: name(GOAL_BRANCH),
                    task: GOAL,
                    writer: None,
                    beside: None,
                    design_writers: Box::new([]),
                    drift: None,
                }),
            },
        );
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::BranchHead(BranchHead { name: name(GOAL_BRANCH), commit: TIP, owned: true }),
            },
        );
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let mut log = Queue::uncapped();
    step(&mut d, &env(), Event::CatchUp, &mut log);

    if !restored {
        step(&mut d, &env(), Event::Names { task: GOAL, resources }, &mut log);
    }
    let mut later = env();
    later.now = later.now.saturating_add(CLIENT.poll_max);
    fire(&mut d, &later, &mut log);
    let mut run_reads = 0_usize;
    let mut goal_reads = 0_usize;
    for _ in 0..CLIENT.calls {
        resume(&mut d, &later, &mut log);
        while let Some(request) = log.pop() {
            if let Request::Call { call, op: Op::Read(Read::Branch { branch }), .. } = request {
                run_reads = run_reads.saturating_add(usize::from(branch.as_ref() == b"temper/7/r7"));
                goal_reads = goal_reads.saturating_add(usize::from(branch.as_ref() == GOAL_BRANCH));
                step(
                    &mut d,
                    &later,
                    Event::Client(skein_forge_client::Event::Answered {
                        call,
                        cost: 1,
                        free: 0,
                        rate: None,
                        result: Ok(Answer::Commit(TIP)),
                    }),
                    &mut log,
                );
            }
        }
    }
    assert_eq!(run_reads, usize::from(!goal), "goal={goal}, restored={restored}");
    assert_eq!(d.names(GOAL).unwrap().contains(&run), !goal);
    if goal {
        assert!(goal_reads > 0, "g stays watched");
    }
}

#[test]
fn naming_a_goals_run_branch_watches_only_g_but_a_chat_still_watches_r() {
    run_watch(false, true);
    run_watch(false, false);
}

#[test]
fn restored_goal_names_drop_r_even_when_names_precede_holds() {
    run_watch(true, true);
    run_watch(true, false);
}

#[test]
fn a_goal_branch_hold_is_exclusive_and_refusing() {
    let mut d = fresh();
    take(&mut d);
    assert_eq!(d.resource_facts(&name(GOAL_BRANCH)), Some((Access::Owned, resources::GOAL_BRANCH_HOLD)));
    for branch in [b"temper/7/c7".as_slice(), b"temper/7/r7", b"temper/7/s7", b"temper/7/l7"] {
        assert_eq!(d.resource_facts(&name(branch)), Some((Access::Owned, resources::BRANCH_HOLD)));
    }
    assert_eq!(
        outputs(&mut d, Event::Hold { entry: CLOSING, task: OTHER, resource: name(GOAL_BRANCH), from: None }).as_ref(),
        &[Request::Taken { task: OTHER, resource: name(GOAL_BRANCH), by: GOAL }]
    );
}

#[test]
fn taking_a_missing_goal_branch_makes_it_at_the_default_branchs_tip() {
    let mut d = fresh();
    take(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    fixtures::answer(&mut d, call, Err(Error::Missing));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    let saved = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_kept_making(&saved);
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, Some(Writer::Entry(TAKING)));
    for request in ready(&mut d) {
        if let Request::Call { op: Op::Write(_), .. } = request {
            panic!("write before commit: {request:?}");
        }
    }
    outputs(&mut d, Event::Committed { entry: TAKING });
    fixtures::answer_next(&mut d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (_, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
}

#[test]
fn taking_a_missing_goal_branch_makes_it_at_the_adopted_landing_tip() {
    let mut d = fresh();
    d.repositories.get_mut(&REPO).unwrap().landing = b"staging".as_slice().into();
    take(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    fixtures::answer(&mut d, call, Err(Error::Missing));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"staging".as_slice().into() }));
    let saved = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_kept_making(&saved);
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, Some(Writer::Entry(TAKING)));
    for request in ready(&mut d) {
        if let Request::Call { op: Op::Write(_), .. } = request {
            panic!("write before commit: {request:?}");
        }
    }
    outputs(&mut d, Event::Committed { entry: TAKING });
    fixtures::answer_next(&mut d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (_, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
}

#[test]
fn a_room_bound_gives_up_a_making_before_its_write() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Err(Error::Missing));
    let (call, _) = fixtures::next_call(&mut d);
    for number in 1..=u64::from(LIMITS.entries) {
        outputs(&mut d, Event::Enqueue { entry: status(number, OTHER) });
    }
    let waiting = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_no_making(&waiting);
    assert!(d.making_waits(GOAL));
    let ended = outbox_room::later(&env(), Duration::from_secs(1));
    at_step(&mut d, &ended, Event::ChangeEnding { task: OTHER, ending: Ending::Cancelled });
    let due = outbox_room::later(&ended, ended.limits.waits.room);
    assert_eq!(d.next_wall_deadline(), Some(due.wall));
    let mut out = Queue::uncapped();
    fire(&mut d, &due, &mut out);
    let requests = collected(out);
    let reason = format!(
        "no progress at making for 30 minutes: waiting for room in the forge's outbox, held by entry 1 of ended task {OTHER}"
    );
    assert!(
        requests.contains(&Request::MakeFailed {
            task: GOAL,
            resource: name(GOAL_BRANCH),
            reason: reason.into_bytes().into_boxed_slice()
        }),
        "{requests:?}"
    );
    assert!(!d.making_waits(GOAL));
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
    assert_eq!(d.entries.len(), LIMITS.entries);
}

#[test]
fn a_goal_branch_making_waits_for_room_but_holds_for_each_unmendable_cause() {
    for (case, count, number, access, branch, reason) in [
        ("full", LIMITS.entries, 5, Some(Role::Owned), true, None),
        ("released before answer", LIMITS.entries, TAKING, None, true, None),
        (
            "read only",
            LIMITS.entries,
            TAKING,
            Some(Role::Context),
            true,
            Some(b"the repository is adopted to be read, not written".as_slice()),
        ),
        (
            "no branch writes",
            LIMITS.entries,
            TAKING,
            Some(Role::Owned),
            false,
            Some(b"the repository's adoption takes no branch writes".as_slice()),
        ),
        (
            "entry in use",
            LIMITS.entries,
            1,
            Some(Role::Owned),
            true,
            Some(b"its outbox entry 1 is already in use".as_slice()),
        ),
    ] {
        let mut d = fresh();
        outputs(&mut d, Event::Hold { entry: number, task: GOAL, resource: name(GOAL_BRANCH), from: None });
        fixtures::answer_next(&mut d, Err(Error::Missing));
        let (call, _) = fixtures::next_call(&mut d);
        let first = if case == "full" { number + 1 } else { 1 };
        for n in first..first + u64::from(count) {
            outputs(&mut d, Event::Enqueue { entry: status(n, OTHER) });
        }
        match access {
            Some(role) => {
                let repository = d.repositories.get_mut(&REPO).unwrap();
                repository.role = role;
                repository.kinds.branch = branch;
            }
            None => {
                d.repositories.remove(&REPO);
            }
        }
        let requests = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
        if access.is_none() {
            assert!(requests.is_empty(), "a released repository's answer makes no new work");
            assert_eq!(d.entries.len(), count);
            continue;
        }
        match reason {
            Some(reason) => assert_eq!(
                requests.as_ref(),
                &[Request::MakeFailed { task: GOAL, resource: name(GOAL_BRANCH), reason: reason.into() }],
                "{case}"
            ),
            None => {
                assert_no_making(&requests);
                assert!(!d.is_ready(), "{case}: no busy loop");
                let report = env().now.saturating_add(WAIT_REPORT);
                assert_eq!(d.next_deadline(), Some(report), "{case}: only a report, no polling");
                assert_no_making(&ready(&mut d));
                assert_eq!(
                    outputs(
                        &mut d,
                        Event::Claim {
                            designs: Box::new([]),
                            task: OTHER,
                            attempt: 1,
                            writes: Box::new([name(GOAL_BRANCH)]),
                            holders: Box::new([GOAL]),
                        }
                    )
                    .as_ref(),
                    &[Request::Taken { task: OTHER, resource: name(GOAL_BRANCH), by: GOAL }]
                );
            }
        }
        assert_eq!(d.entries.len(), count, "{case}: no entry replaced or added");
        for (n, row) in &d.entries {
            assert_eq!(row.entry, status(*n, OTHER), "{case}");
        }
        assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None, "{case}");
        for request in requests {
            if let Request::Call { .. } = request {
                panic!("{case}: no call: {request:?}");
            }
        }
        if case == "full" {
            outputs(&mut d, Event::Committed { entry: first });
            fixtures::answer_next(
                &mut d,
                Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
            );
            let (_, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Write(status(first, OTHER).effect.write), "a higher entry hands off");
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table follows each entry-freeing cause through a fresh-number creation")]
fn a_goal_branch_making_is_kept_and_sent_when_an_entry_frees() {
    for cause in ["made", "withdrawn", "not made"] {
        let mut environment = env();
        environment.limits.entries = 1;
        let mut d = domain_with_limits(&environment.limits);
        at_step(&mut d, &environment, Event::Restore { record: Stored::Repository(adopted()) });
        if cause == "not made" {
            at_step(
                &mut d,
                &environment,
                Event::Restore { record: Stored::Entry { entry: status(1, OTHER), held: true } },
            );
        }
        at_step(&mut d, &environment, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        at_step(&mut d, &environment, Event::CatchUp);

        if cause != "not made" {
            at_step(&mut d, &environment, Event::Enqueue { entry: status(1, OTHER) });
        }
        let completing = if cause == "made" {
            at_step(&mut d, &environment, Event::Committed { entry: 1 });
            at_answer(
                &mut d,
                &environment,
                Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: environment.now }),
            );
            let (call, op) = at_call(&mut d, &environment);
            assert_eq!(op, Op::Write(status(1, OTHER).effect.write));
            Some(call)
        } else {
            None
        };
        at_step(
            &mut d,
            &environment,
            Event::Hold { entry: TAKING, task: GOAL, resource: name(GOAL_BRANCH), from: None },
        );
        at_answer(&mut d, &environment, Err(Error::Missing));
        assert_no_making(&at_answer(&mut d, &environment, Ok(Answer::Commit(TIP))));
        let freed = if let Some(call) = completing {
            at_step(
                &mut d,
                &environment,
                Event::Client(skein_forge_client::Event::Answered {
                    free: 0,
                    rate: None,
                    call,
                    cost: 1,
                    result: Ok(Answer::Done),
                }),
            )
        } else if cause == "withdrawn" {
            at_step(&mut d, &environment, Event::Withdraw { entry: 1 })
        } else {
            at_step(&mut d, &environment, Event::Resolve { entry: 1, again: false })
        };
        assert!(freed.contains(&Request::Erase { key: Key::Entry(1) }), "{cause}: {freed:?}");
        assert!(
            freed.contains(&Request::ContinueGoalBranch { task: GOAL }),
            "{cause}: freed room asks without polling"
        );
        let mut resumed = Queue::uncapped();
        resume(&mut d, &environment, &mut resumed);
        let mut saved = skein_lib::List::with_capacity(u32::try_from(freed.len()).unwrap() + resumed.len());
        for request in freed {
            saved.push(request).unwrap();
        }
        while let Some(request) = resumed.pop() {
            saved.push(request).unwrap();
        }
        let saved = saved.into_boxed();
        let mut asks = 0_u32;
        for request in &saved {
            if let Request::ContinueGoalBranch { task } = request {
                assert_eq!(*task, GOAL, "{cause}");
                asks = asks.checked_add(1).unwrap();
            }
        }
        assert_eq!(asks, 1, "{cause}: fresh number asked once: {saved:?}");
        assert_no_making(&saved);
        at_step(&mut d, &environment, Event::ContinueGoalBranch { task: GOAL, entry: CLOSING });
        let tip = [5; 32];
        let saved = at_answer(&mut d, &environment, Ok(Answer::Commit(tip)));
        let mut expected = entry();
        expected.number = CLOSING;
        expected.effect.write = Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: tip };
        assert!(
            saved.contains(&Request::Save {
                read_afresh: None,
                release: Some(CLOSING),
                record: Stored::Entry { entry: expected.clone(), held: false },
            }),
            "{cause}: fresh number and fresh tip: {saved:?}"
        );
        assert_eq!(d.entries.get(&CLOSING).unwrap().entry, expected);
        assert!(!d.entries.contains_key(&TAKING), "{cause}: old number given up");
        at_step(&mut d, &environment, Event::Committed { entry: CLOSING });
        at_answer(
            &mut d,
            &environment,
            Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: environment.now }),
        );
        let (_, op) = at_call(&mut d, &environment);
        assert_eq!(op, Op::Write(expected.effect.write), "{cause}");
    }
}

fn assert_room_ask(requests: &[Request], expected: u64) {
    let mut asks = 0_u32;
    for request in requests {
        if let Request::ContinueGoalBranch { task } = request {
            assert_eq!(*task, expected, "room goes to the first place");
            asks = asks.checked_add(1).unwrap();
        }
    }
    assert_eq!(asks, 1, "one ask for the freed room");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table checks FIFO re-waits and inverted fresh-tip answers through handoff"
)]
fn goal_branch_room_waiters_ask_in_wait_order_and_keep_their_place() {
    for (case, entries) in [("reserved while rereading", 1), ("inverted tip answers", 2)] {
        let mut environment = env();
        environment.limits.entries = entries;
        let mut d = domain_with_limits(&environment.limits);
        at_step(&mut d, &environment, Event::Restore { record: Stored::Repository(adopted()) });
        at_step(&mut d, &environment, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        at_step(&mut d, &environment, Event::CatchUp);

        for number in 1..=u64::from(entries) {
            at_step(&mut d, &environment, Event::Enqueue { entry: status(number, 9) });
        }
        // G2 starts reading first, but G1 meets the full outbox first.
        let other_branch = b"temper/7/g8";
        at_step(
            &mut d,
            &environment,
            Event::Hold { entry: CLOSING, task: OTHER, resource: name(other_branch), from: None },
        );
        assert!(d.hold(&name(other_branch)).is_some(), "G2 held");
        at_answer(&mut d, &environment, Err(Error::Missing));
        let (later, _) = at_call(&mut d, &environment);
        // Distinct landing reads keep the client from coalescing the two answers.
        d.repositories.get_mut(&REPO).unwrap().landing = b"other-main".as_slice().into();
        at_step(
            &mut d,
            &environment,
            Event::Hold { entry: TAKING, task: GOAL, resource: name(GOAL_BRANCH), from: None },
        );
        assert!(d.hold(&name(GOAL_BRANCH)).is_some(), "G1 held");
        at_answer(&mut d, &environment, Err(Error::Missing));
        assert_no_making(&at_answer(&mut d, &environment, Ok(Answer::Commit(TIP))));
        assert_no_making(&at_step(
            &mut d,
            &environment,
            Event::Client(skein_forge_client::Event::Answered {
                call: later,
                cost: 1,
                free: 0,
                rate: None,
                result: Ok(Answer::Commit(TIP)),
            }),
        ));
        let asked = at_step(&mut d, &environment, Event::Withdraw { entry: 1 });
        assert_room_ask(&asked, GOAL);
        assert!(!d.is_ready(), "one freed entry asks only one waiter, even before the answer");
        let mut again = Queue::uncapped();
        resume(&mut d, &environment, &mut again);
        assert!(again.is_empty());
        at_step(&mut d, &environment, Event::ContinueGoalBranch { task: GOAL, entry: 30 });
        let (reread, _) = at_call(&mut d, &environment);
        if case == "inverted tip answers" {
            let asked = at_step(&mut d, &environment, Event::Withdraw { entry: 2 });
            assert_room_ask(&asked, OTHER);
            d.repositories.get_mut(&REPO).unwrap().landing = b"new-main".as_slice().into();
            at_step(&mut d, &environment, Event::ContinueGoalBranch { task: OTHER, entry: 31 });
            assert!(d.is_ready(), "before the younger reread");
            at_answer(&mut d, &environment, Ok(Answer::Commit(TIP)));
            at_step(&mut d, &environment, Event::Committed { entry: 31 });
            assert!(!d.is_ready(), "a higher fresh entry waits for the older tip read");
            at_step(
                &mut d,
                &environment,
                Event::Client(skein_forge_client::Event::Answered {
                    call: reread,
                    cost: 1,
                    free: 0,
                    rate: None,
                    result: Ok(Answer::Commit(TIP)),
                }),
            );
            let mut log = Queue::uncapped();
            let mut calls = Queue::uncapped();
            step(&mut d, &environment, Event::Committed { entry: 30 }, &mut log);
            for branch in [GOAL_BRANCH, other_branch.as_slice()] {
                let mut sent = false;
                for _ in 0_u8..4 {
                    resume(&mut d, &environment, &mut log);
                    while let Some(request) = log.pop() {
                        if let Request::Call { call, op, .. } = request {
                            calls.push((call, op));
                        }
                    }
                    let Some((call, op)) = calls.pop() else { continue };
                    match op {
                        Op::Read(Read::Items { .. }) => {
                            step(
                                &mut d,
                                &environment,
                                Event::Client(skein_forge_client::Event::Answered {
                                    call,
                                    cost: 1,
                                    free: 0,
                                    rate: None,
                                    result: Ok(Answer::Items {
                                        cut: false,
                                        items: Box::new([]),
                                        more: false,
                                        now: environment.now,
                                    }),
                                }),
                                &mut log,
                            );
                            d.reclaim();
                        }
                        Op::Write(Write::CreateBranch { branch: actual, commit }) => {
                            assert_eq!(actual.as_ref(), branch, "{case}: wait order is send order");
                            assert_eq!(commit, TIP);
                            step(
                                &mut d,
                                &environment,
                                Event::Client(skein_forge_client::Event::Answered {
                                    call,
                                    cost: 1,
                                    free: 0,
                                    rate: None,
                                    result: Ok(Answer::Branch(BranchCreation::Created)),
                                }),
                                &mut log,
                            );
                            d.reclaim();
                            sent = true;
                            break;
                        }
                        other @ (Op::Read(_) | Op::Write(_)) => panic!("{case}: unexpected call: {other:?}"),
                    }
                }
                assert!(sent, "{case}: {branch:?} branch sent");
            }
            continue;
        }
        // An asked making owns its room while it reads its fresh tip (forge/room.md, 2).
        let refused = at_step(&mut d, &environment, Event::Enqueue { entry: status(31, 9) });
        assert!(refused.contains(&Request::Outcome {
            entry: 31,
            task: 9,
            account: None,
            outcome: skein_forge_client::Outcome::Failed(Error::Busy),
        }));
        let kept = at_step(
            &mut d,
            &environment,
            Event::Client(skein_forge_client::Event::Answered {
                call: reread,
                cost: 1,
                free: 0,
                rate: None,
                result: Ok(Answer::Commit(TIP)),
            }),
        );
        let mut expected = entry();
        expected.number = 30;
        assert!(kept.contains(&Request::Save {
            record: Stored::Entry { entry: expected, held: false },
            release: Some(30),
            read_afresh: None,
        }));
        let asked = at_step(&mut d, &environment, Event::Withdraw { entry: 30 });
        assert_room_ask(&asked, OTHER);
    }
}

#[test]
fn taking_an_existing_goal_branch_makes_nothing() {
    let mut d = fresh();
    take(&mut d);
    let answered = fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    assert_no_making(&answered);
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(TIP));
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
    assert!(!d.is_ready());
}

#[test]
fn a_goal_branch_making_that_fails_for_good_says_why() {
    let mut d = fresh();
    let (call, _) = making(&mut d);
    let failed = fixtures::answer(&mut d, call, Err(Error::Forbidden));
    assert!(
        failed.contains(&Request::MakeFailed {
            task: GOAL,
            resource: name(GOAL_BRANCH),
            reason: b"forge refused permission".as_slice().into()
        }),
        "{failed:?}"
    );
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
}

#[test]
fn a_change_into_a_goal_branch_being_made_reads_it_once_made() {
    use temper_engine_domain_forge_change as change;
    let mut d = fresh();
    outputs(
        &mut d,
        Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } },
    );
    let (making_call, _) = making(&mut d);
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = change::EffectResult::None;
    outputs(&mut d, Event::Change { row });
    fixtures::step_change(&mut d);
    let waiting = fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    assert!(waiting.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::Wait {
            until: Some(Wall::from_nanos(env().wall.as_nanos().saturating_add(LIMITS.change_policy.stall.as_nanos())))
        },
        entry: None,
        evidence: None
    }));
    let made = fixtures::answer(&mut d, making_call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(made.contains(&self::made()), "{made:?}");
    let wake = ready(&mut d);
    assert!(wake.contains(&Request::WakeProcedure { task: fixtures::CHANGE }), "{wake:?}");
    fixtures::step_change(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    let (_, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
}

#[test]
fn a_change_waiting_for_its_base_to_be_made_is_bounded_by_its_stall() {
    use temper_engine_domain_forge_change as change;
    let mut d = fresh();
    fixtures::connect(&mut d);
    making(&mut d);
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = change::EffectResult::None;
    let since = row.change.since;
    outputs(&mut d, Event::Change { row });
    fixtures::step_change(&mut d);
    let waiting = fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    let until = Wall::from_nanos(since.as_nanos().saturating_add(LIMITS.change_policy.stall.as_nanos()));
    assert!(waiting.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::Wait { until: Some(until) },
        entry: None,
        evidence: None
    }));
    d.wait_change(&env(), fixtures::CHANGE, Some(until));
    let mut later = env();
    let wake = fixtures::advance(&mut d, &mut later, LIMITS.change_policy.stall);
    let mut signalled = false;
    let mut held = false;
    for request in &wake {
        signalled |= *request == Request::WakeProcedure { task: fixtures::CHANGE };
        if let Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Hold(change::Hold::Stalled),
            evidence: Some(_),
            ..
        } = request
        {
            held = true;
        }
    }
    assert!(signalled);
    assert!(held, "the read session expires without another forge answer: {wake:?}");
    assert!(matches!(
        d.change(fixtures::CHANGE).expect("change").change.state,
        change::State::Held { why: change::Hold::Stalled, .. }
    ));
}

#[test]
fn a_closing_goal_keeps_its_branch_when_no_change_landed_it() {
    for ending in [ReleaseEnding::Done, ReleaseEnding::Failed, ReleaseEnding::Cancelled] {
        let mut d = fresh();
        take(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
        let released =
            outputs(&mut d, Event::Release { task: GOAL, root: OTHER, ending, reason: None, entry: CLOSING });
        assert!(released.contains(&Request::Released { task: GOAL }), "{released:?}");
        assert_no_making(&released);
        for request in &released {
            if let Request::Retained { .. } = request {
                panic!("a goal keeps its branch without retaining a hold: {request:?}");
            }
        }
        assert_eq!(d.hold(&name(GOAL_BRANCH)), None);
        assert_eq!(d.branch_head(&name(GOAL_BRANCH)), None);
        assert!(released.contains(&Request::Erase { key: Key::BranchHead(name(GOAL_BRANCH)) }));
    }
}

fn landed_row(head: Option<skein_forge::api::Commit>) -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.branch = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = temper_engine_domain_forge_change::EffectResult::None;
    row.change.state = temper_engine_domain_forge_change::State::Landed { merge: MERGED, since: env().wall };
    row.result.head = head;
    row
}

fn held_goal_with_change(row: ChangeRow) -> Domain {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    d
}

fn release(d: &mut Domain, ending: ReleaseEnding) -> Box<[Request]> {
    outputs(d, Event::Release { task: GOAL, root: OTHER, ending, reason: None, entry: CLOSING })
}

fn assert_kept_goal(d: &mut Domain) {
    let released = release(d, ReleaseEnding::Failed);
    assert!(released.contains(&Request::Released { task: GOAL }), "{released:?}");
    assert!(released.contains(&Request::Erase { key: Key::Hold(name(GOAL_BRANCH)) }), "{released:?}");
    for request in &released {
        if let Request::Retained { .. }
        | Request::Save { record: Stored::Entry { .. }, .. }
        | Request::Call { op: Op::Write(Write::DeleteBranch { .. }), .. } = request
        {
            panic!("a kept goal branch has no delete or retained hold: {request:?}");
        }
    }
    assert_eq!(d.hold(&name(GOAL_BRANCH)), None);
    assert!(!d.is_ready());
}

fn deleting_goal(d: &mut Domain, ending: ReleaseEnding) -> Token {
    let requests = release(d, ending);
    assert!(!requests.contains(&Request::Released { task: GOAL }), "delete must settle first: {requests:?}");
    assert!(d.hold(&name(GOAL_BRANCH)).is_some());
    outputs(d, Event::Committed { entry: CLOSING });
    fixtures::answer_next(d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Write(Write::DeleteBranch { branch: GOAL_BRANCH.into() }));
    call
}

fn complete_goal_delete(d: &mut Domain, call: Token, result: Result<Answer, Error>) {
    let deleted = fixtures::answer(d, call, result);
    assert!(deleted.contains(&Request::ContinueRelease { task: GOAL }), "{deleted:?}");
    assert_eq!(d.hold(&name(GOAL_BRANCH)), None);
    let released = outputs(d, Event::ContinueRelease { task: GOAL, entry: CLOSING });
    assert!(released.contains(&Request::Released { task: GOAL }), "{released:?}");
}

#[test]
fn a_closing_goal_deletes_its_branch_once_a_change_landed_it_whatever_way_it_ended() {
    for ending in [ReleaseEnding::Done, ReleaseEnding::Failed, ReleaseEnding::Cancelled] {
        let mut d = held_goal_with_change(landed_row(Some(TIP)));
        let call = deleting_goal(&mut d, ending);
        complete_goal_delete(&mut d, call, Ok(Answer::Done));
    }
}

#[test]
fn a_goal_branch_landed_before_or_after_verification_is_deleted() {
    use temper_engine_domain_forge_change as change;
    for state in [
        change::State::Landed { merge: MERGED, since: env().wall },
        change::State::Red { merge: MERGED, since: env().wall },
        change::State::Cleared { merge: MERGED },
        change::State::Uncleared { merge: MERGED },
    ] {
        let mut row = landed_row(Some(TIP));
        row.change.state = state;
        let mut d = held_goal_with_change(row);
        let call = deleting_goal(&mut d, ReleaseEnding::Done);
        complete_goal_delete(&mut d, call, Ok(Answer::Done));
    }
}

#[test]
fn a_closing_goal_keeps_its_branch_moved_past_its_landing() {
    let mut d = held_goal_with_change(landed_row(Some(fixtures::HEAD)));
    assert_kept_goal(&mut d);
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), None);
}

#[test]
fn a_closing_goal_keeps_its_branch_when_its_tip_is_unknown() {
    for head in [Some(TIP), None] {
        let mut d = held_goal_with_change(landed_row(head));
        d.heads.remove(&name(GOAL_BRANCH));
        assert_kept_goal(&mut d);
    }
}

#[test]
fn a_closing_goal_keeps_its_branch_without_a_matching_landed_change() {
    use temper_engine_domain_forge_change as change;
    let matching = landed_row(Some(TIP));
    let mut other_forge = matching.clone();
    other_forge.repository.forge = 3;
    let mut other_repository = matching.clone();
    other_repository.repository.repository = 3;
    let mut other_branch = matching.clone();
    other_branch.branch = b"temper/7/c7".as_slice().into();
    let mut unlanded = matching.clone();
    unlanded.change.state = change::State::Landing { head: TIP };
    let mut unknown_head = matching;
    unknown_head.result.head = None;
    for row in [other_forge, other_repository, other_branch, unlanded, unknown_head] {
        let mut d = held_goal_with_change(row);
        assert_kept_goal(&mut d);
    }
}

#[test]
fn a_goal_branchs_refused_delete_fails_the_release() {
    let mut d = held_goal_with_change(landed_row(Some(TIP)));
    let call = deleting_goal(&mut d, ReleaseEnding::Done);
    let failed = fixtures::answer(&mut d, call, Err(Error::Protected));
    let mut release_failed = false;
    for request in &failed {
        if let Request::ReleaseFailed { task: GOAL, .. } = request {
            release_failed = true;
        }
    }
    assert!(release_failed, "{failed:?}");
    assert!(!failed.contains(&Request::Released { task: GOAL }), "{failed:?}");
    assert!(d.hold(&name(GOAL_BRANCH)).is_some());
}

#[test]
fn a_goal_branch_already_gone_completes_its_release() {
    let mut d = held_goal_with_change(landed_row(Some(TIP)));
    let call = deleting_goal(&mut d, ReleaseEnding::Done);
    complete_goal_delete(&mut d, call, Err(Error::Missing));
}

const ONE_READ: u32 = 1; // The first goal fills the client's read table.
const TWO_READS: u32 = 2; // Both goals' reads fit at once.
const READ_ATTEMPTS: u32 = 1; // The first transport refusal reaches the connector.
const FIRST_GOAL: u64 = 6;
const FIRST_GOAL_BRANCH: &[u8] = b"temper/6/g6";
const FIRST_REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: REPO.forge, repository: 1 };

fn restore_goal(d: &mut Domain, env: &Env<Limits>, task: u64, name: Name) {
    at_step(
        d,
        env,
        Event::Restore {
            record: Stored::Hold(Hold {
                name,
                task,
                writer: None,
                beside: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
    );
}

fn restart_makings(read_room: u32) -> (Domain, Env<Limits>) {
    let mut env = env();
    env.limits.client.pending = read_room;
    env.limits.client.calls = read_room;
    env.limits.client.read_attempts = READ_ATTEMPTS;
    let mut d = domain_with_limits(&env.limits);
    for repository in [adopted(), Repository { provider: FIRST_REPO, home: false, ..adopted() }] {
        at_step(&mut d, &env, Event::Restore { record: Stored::Repository(repository) });
    }
    restore_goal(&mut d, &env, FIRST_GOAL, Name { repository: FIRST_REPO.repository, ..name(FIRST_GOAL_BRANCH) });
    restore_goal(&mut d, &env, GOAL, name(GOAL_BRANCH));
    at_step(
        &mut d,
        &env,
        Event::Restore { record: Stored::BranchHead(BranchHead { name: name(GOAL_BRANCH), commit: TIP, owned: true }) },
    );
    fixtures::connect(&mut d);
    at_step(&mut d, &env, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    at_step(&mut d, &env, Event::CatchUp);
    (d, env)
}

#[test]
fn restart_finishes_while_a_making_waits_for_read_room() {
    let (mut d, env) = restart_makings(ONE_READ);
    let answered = at_answer(&mut d, &env, Ok(Answer::Commit(TIP)));
    assert!(answered.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }), "{answered:?}");
    assert!(d.repository_fresh(REPO));
    assert!(d.goal_branch_waiting(&name(GOAL_BRANCH)));

    let (call, op) = at_call(&mut d, &env);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let missing = at_answer_call(&mut d, &env, call, Err(Error::Missing));
    assert!(missing.contains(&Request::ContinueGoalBranch { task: GOAL }), "{missing:?}");
    let (call, op) = send_restarted_making(&mut d, &env);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
    let made = at_answer_call(&mut d, &env, call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(made.contains(&self::made()), "{made:?}");
}

fn missing_after_read_room_frees() -> (Domain, Env<Limits>) {
    let (mut d, env) = restart_makings(ONE_READ);
    at_answer(&mut d, &env, Ok(Answer::Commit(TIP)));
    at_answer(&mut d, &env, Err(Error::Missing));
    (d, env)
}

fn send_restarted_making(d: &mut Domain, env: &Env<Limits>) -> (Token, Op) {
    at_step(d, env, Event::ContinueGoalBranch { task: GOAL, entry: TAKING });
    at_answer(d, env, Ok(Answer::Commit(TIP)));
    at_step(d, env, Event::Committed { entry: TAKING });
    at_answer(d, env, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env.now }));
    at_call(d, env)
}

fn finish_restarted_making(d: &mut Domain, env: &Env<Limits>) -> Box<[Request]> {
    let (call, _) = send_restarted_making(d, env);
    at_answer_call(d, env, call, Ok(Answer::Branch(BranchCreation::Created)))
}

#[test]
fn repository_finishes_while_a_making_waits_for_read_room() {
    let (d, _) = restart_makings(ONE_READ);
    assert!(d.repository_fresh(REPO));
    assert!(!d.repository_fresh(FIRST_REPO));
    assert!(!d.fresh_reads_in());
}

#[test]
fn only_out_making_reads_appear_in_catch_up() {
    for (room, count, words) in [
        (ONE_READ, ONE_READ, b"1 goal branch (task 6)".as_slice()),
        (TWO_READS, TWO_READS, b"2 goal branches (task 6, task 7)".as_slice()),
    ] {
        let (d, _) = restart_makings(room);
        assert_eq!(d.catching_up(REPO.forge).expect("a making's read is out").goal_branches, count);
        assert_eq!(d.restart_waits(RestartStage::ReadAfresh).as_ref(), words);
    }
}

#[test]
fn restart_does_not_wait_for_a_makings_forge() {
    for error in [Error::Unavailable, Error::Timeout, Error::NotConnected] {
        let (mut d, env) = restart_makings(TWO_READS);
        let (first, _) = at_call(&mut d, &env);
        at_answer(&mut d, &env, Err(error));
        assert!(d.repository_fresh(REPO), "{error:?}");
        let answered = at_answer_call(&mut d, &env, first, Ok(Answer::Commit(TIP)));
        assert!(answered.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }), "{answered:?}");
        assert!(d.goal_branch_waiting(&name(GOAL_BRANCH)));
    }
}

#[test]
fn a_change_waits_for_making_after_read_afresh() {
    use temper_engine_domain_forge_change as change;
    let (mut d, env) = missing_after_read_room_frees();
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = change::EffectResult::None;
    at_step(&mut d, &env, Event::Change { row });
    at_step(&mut d, &env, fixtures::change_step());
    let waiting = at_answer(&mut d, &env, Ok(Answer::Commit(fixtures::HEAD)));
    let until = Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.change_policy.stall.as_nanos()));
    assert!(waiting.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::Wait { until: Some(until) },
        entry: None,
        evidence: None,
    }));

    finish_restarted_making(&mut d, &env);
    let mut woke = Queue::uncapped();
    resume(&mut d, &env, &mut woke);
    let mut change_woke = false;
    for request in &woke {
        change_woke |= *request == Request::WakeProcedure { task: fixtures::CHANGE };
    }
    assert!(change_woke);
}

const DESIGN_ATTEMPT: u64 = 1;

fn claim_design(d: &mut Domain, env: &Env<Limits>) -> Box<[Request]> {
    at_step(
        d,
        env,
        Event::Claim {
            task: GOAL,
            attempt: DESIGN_ATTEMPT,
            writes: Box::new([]),
            designs: Box::new([name(GOAL_BRANCH)]),
            holders: Box::new([]),
        },
    )
}

#[test]
fn a_design_claim_waits_for_making_after_read_afresh() {
    let (mut d, env) = missing_after_read_room_frees();
    let refused = claim_design(&mut d, &env);
    assert!(refused.contains(&Request::Refused { task: GOAL }), "{refused:?}");

    finish_restarted_making(&mut d, &env);
    claim_design(&mut d, &env);
    assert_eq!(
        d.hold(&name(GOAL_BRANCH)).unwrap().design_writers.as_ref(),
        &[DesignWriter { task: GOAL, attempt: DESIGN_ATTEMPT, rereading: false }]
    );
}

#[test]
fn a_goal_workspace_waits_for_making_after_read_afresh() {
    for (modify, start) in
        [(false, items::Start::Branch(GOAL_BRANCH.into())), (true, items::Start::Base(GOAL_BRANCH.into()))]
    {
        let (mut d, env) = missing_after_read_room_frees();
        let seed = items::Seed {
            project: adopted().project,
            modify,
            task: GOAL,
            parent: None,
            repository: REPO,
            run_branch: b"temper/7/r7".as_slice().into(),
            saved_branch: None,
        };
        assert!(d.goal_workspace_waiting(GOAL, modify));
        assert!(d.workspace_item(seed.clone()).is_none());

        finish_restarted_making(&mut d, &env);
        assert!(!d.goal_workspace_waiting(GOAL, modify));
        assert_eq!(d.workspace_item(seed).expect("goal workspace ready").start, start);
    }
}

#[test]
fn a_producer_workspace_waits_for_its_goal_branch() {
    use temper_engine_domain_forge_change as change;
    const SAVED: &[u8] = b"temper/8/s8";
    const CHANGE_BRANCH: &[u8] = b"temper/51";
    for (branch, base, early, saved_branch, start) in [
        (GOAL_BRANCH, b"main".as_slice(), change::Early::None, None, items::Start::Branch(GOAL_BRANCH.into())),
        (
            CHANGE_BRANCH,
            GOAL_BRANCH,
            change::Early::Started(Box::new([])),
            Some(Box::from(SAVED)),
            items::Start::Merge { branch: SAVED.into(), base: TIP },
        ),
    ] {
        let (mut d, env) = missing_after_read_room_frees();
        let mut row = fixtures::opening_row();
        row.pending = None;
        row.branch = branch.into();
        row.base = base.into();
        row.change.early = early;
        row.change.state = change::State::Producing { requested: true };
        row.delegate = Some((OTHER, change::Delegate::Produce));
        at_step(&mut d, &env, Event::Restore { record: Stored::Change(Box::new(row)) });
        let seed = items::Seed {
            project: adopted().project,
            modify: true,
            task: OTHER,
            parent: Some(fixtures::CHANGE),
            repository: REPO,
            run_branch: b"temper/8/r8".as_slice().into(),
            saved_branch,
        };
        assert!(d.producer_workspace_waiting(OTHER, fixtures::CHANGE));
        assert!(d.workspace_item(seed.clone()).is_none());
        finish_restarted_making(&mut d, &env);
        assert!(!d.producer_workspace_waiting(OTHER, fixtures::CHANGE));
        assert_eq!(d.workspace_item(seed).expect("producer workspace ready").start, start);
    }
}

#[test]
fn goal_release_waits_to_judge_its_branchs_head() {
    let (mut d, env) = restart_makings(ONE_READ);
    at_answer(&mut d, &env, Ok(Answer::Commit(TIP)));
    at_step(&mut d, &env, Event::Restore { record: Stored::Change(Box::new(landed_row(Some(TIP)))) });
    let released = release(&mut d, ReleaseEnding::Done);
    assert_eq!(d.releases.get(&GOAL).expect("release still pending").pending, None, "{released:?}");

    let found = at_answer(&mut d, &env, Ok(Answer::Commit(TIP)));
    assert!(found.contains(&Request::ContinueRelease { task: GOAL }), "{found:?}");
    at_step(&mut d, &env, Event::ContinueRelease { task: GOAL, entry: CLOSING });
    assert_eq!(d.releases.get(&GOAL).expect("delete kept").pending, Some((CLOSING, Some(name(GOAL_BRANCH)))));
}

#[test]
fn completing_a_making_wakes_the_goals_closer() {
    use jig_domain_core_vocabulary::waits::{AtBound, ReportedBound, WaitKind, WaitObject};
    const CLOSER: u64 = 9;
    let (mut d, env) = missing_after_read_room_frees();
    let row = closer::closer(CLOSER, GOAL, false);
    at_step(&mut d, &env, Event::Closer { row });
    let parked = at_step(&mut d, &env, Event::ParkCloser { task: CLOSER, wait: CloserPark::Making(name(GOAL_BRANCH)) });
    let [wait] = wait_report(&parked, CLOSER).expect("goal branch making wait") else { panic!("one closer wait") };
    assert_eq!(
        wait.object,
        WaitObject::Procedure {
            step: temper_client_protocol_wire::waits::CLOSER_MAKING,
            things: skein_lib::List::with_capacity(0),
        }
    );
    assert_eq!(wait.since, env.wall);
    assert_eq!(wait.kind, WaitKind::Step);
    assert_eq!(wait.bound, ReportedBound::Figure(AtBound::Hold));
    let made = finish_restarted_making(&mut d, &env);
    assert!(made.contains(&Request::WakeProcedure { task: CLOSER }), "{made:?}");
    assert_eq!(wait_report(&made, CLOSER), Some([].as_slice()));
}

#[test]
fn a_goal_branch_found_missing_after_restart_is_asked_as_its_read_ends() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                beside: None,
                design_writers: Box::new([]),
                name: name(GOAL_BRANCH),
                task: GOAL,
                writer: None,
                drift: None,
            }),
        },
    );
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let started = outputs(&mut d, Event::CatchUp);
    assert!(started.contains(&Request::RestartDone { stage: RestartStage::Settled }));
    let missing = fixtures::answer_next(&mut d, Err(Error::Missing));
    assert!(missing.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }), "{missing:?}");
    assert_no_making(&missing);
    assert!(missing.contains(&Request::ContinueGoalBranch { task: GOAL }), "{missing:?}");
    outputs(&mut d, Event::ContinueGoalBranch { task: GOAL, entry: TAKING });
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    assert_kept_making(&fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP))));
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, Some(Writer::Entry(TAKING)));
    // Restart twice after the entry committed but before its first attempt.
    for _ in 0_u8..2 {
        d = restored_making(entry(), false);
    }
    fixtures::answer_next(&mut d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
    assert!(fixtures::answer(&mut d, call, Ok(Answer::Branch(BranchCreation::Created))).contains(&made()));
}

fn restored_making(entry: skein_forge_client::Entry, held: bool) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: name(GOAL_BRANCH),
                task: GOAL,
                writer: Some(Writer::Entry(TAKING)),
                beside: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
    );
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held } });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    fixtures::answer_next(&mut d, Err(Error::Missing));

    d
}

#[test]
fn a_restored_goal_branch_not_found_past_its_deadline_is_made_again() {
    let mut attempted = entry();
    attempted.start = held_issue().start;
    attempted.attempt = held_issue().attempt;
    attempted.failures = 1;
    let mut d = restored_making(attempted.clone(), false);
    for restart in 0_u8..2 {
        if restart != 0 {
            d = restored_making(attempted.clone(), false);
        }
        let missing = fixtures::answer_next(&mut d, Err(Error::Missing));
        for request in &missing {
            if let Request::MakeFailed { .. } | Request::Outcome { outcome: skein_forge_client::Outcome::Held, .. } =
                request
            {
                panic!("a goal is not held for a missing branch: {request:?}");
            }
        }
        // The connector reads again before remaking. A second restart cuts
        // after this save and before the remade entry's first attempt.
        let remade = fixtures::answer_next(&mut d, Err(Error::Missing));
        assert!(
            remade.contains(&Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::Entry { entry: attempted.clone(), held: false },
            }),
            "{remade:?}"
        );
    }
    // Remake looks first too; it retains the old entry's commit.
    fixtures::answer_next(&mut d, Err(Error::Missing));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
    let made = fixtures::answer(&mut d, call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(made.contains(&self::made()), "{made:?}");
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
    // A crash can also retain the connector's held entry before its reread.
    let mut d = restored_making(attempted, true);
    fixtures::answer_next(&mut d, Err(Error::Missing));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
    assert!(fixtures::answer(&mut d, call, Ok(Answer::Branch(BranchCreation::Created))).contains(&self::made()));
}

#[test]
fn a_restored_goal_branch_found_at_its_commit_is_made_and_at_another_fails() {
    for after_held in [false, true] {
        for found in [TIP, fixtures::HEAD] {
            let mut attempted = entry();
            attempted.start = held_issue().start;
            attempted.attempt = held_issue().attempt;
            let mut d = restored_making(attempted, false);
            if after_held {
                fixtures::answer_next(&mut d, Err(Error::Missing));
            }
            let requests = fixtures::answer_next(&mut d, Ok(Answer::Commit(found)));
            if found == TIP {
                assert!(
                    requests.contains(&Request::Outcome {
                        entry: TAKING,
                        task: GOAL,
                        account: None,
                        outcome: skein_forge_client::Outcome::Made {
                            made: skein_forge_client::Made::Branch(TIP),
                            found: true
                        },
                    }),
                    "{requests:?}"
                );
                assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(TIP));
            } else {
                assert!(
                    requests.contains(&Request::MakeFailed {
                        task: GOAL,
                        resource: name(GOAL_BRANCH),
                        reason: b"target already exists".as_slice().into(),
                    }),
                    "{requests:?}"
                );
            }
            assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
        }
    }
}

fn at_step(d: &mut Domain, env: &Env<Limits>, event: Event) -> Box<[Request]> {
    if let Event::Restore { .. } = &event {
        d.restore_begin(Wall::EPOCH);
    }
    let mut out = Queue::uncapped();
    step(d, env, event, &mut out);
    let mut requests = skein_lib::List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).unwrap();
    }
    requests.into_boxed()
}
fn at_call(d: &mut Domain, env: &Env<Limits>) -> (Token, Op) {
    let mut out = Queue::uncapped();
    for _ in 0_u8..2 {
        resume(d, env, &mut out);
        while let Some(request) = out.pop() {
            if let Request::Call { call, op, .. } = request {
                return (call, op);
            }
        }
    }
    panic!("one goal branch call at {:?}, ready={}, due={:?}", env.now, d.is_ready(), d.next_deadline());
}
fn at_answer(d: &mut Domain, env: &Env<Limits>, result: Result<Answer, Error>) -> Box<[Request]> {
    let (call, _) = at_call(d, env);
    at_answer_call(d, env, call, result)
}
fn at_answer_call(d: &mut Domain, env: &Env<Limits>, call: Token, result: Result<Answer, Error>) -> Box<[Request]> {
    let requests = at_step(
        d,
        env,
        Event::Client(skein_forge_client::Event::Answered { free: 0, rate: None, call, cost: 1, result }),
    );
    d.reclaim();
    requests
}

#[test]
fn goal_branch_reads_wait_out_an_outage_with_capped_backoff_reset_by_an_answer() {
    for (error, waits) in [
        (Error::Unavailable, &[1, 2][..]),
        (Error::Unavailable, &[1, 2, 4, 8, 16, 32, 40, 40][..]),
        (Error::Timeout, &[1, 2, 4, 8, 16, 32, 40, 40][..]),
    ] {
        let mut env = env();
        env.limits.client.read_attempts = 1;
        let mut d = domain_with_limits(&env.limits);
        at_step(&mut d, &env, Event::Restore { record: Stored::Repository(adopted()) });
        at_step(&mut d, &env, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        at_step(&mut d, &env, Event::CatchUp);

        at_step(&mut d, &env, Event::Hold { entry: TAKING, task: GOAL, resource: name(GOAL_BRANCH), from: None });
        for seconds in waits {
            let requests = at_answer(&mut d, &env, Err(error));
            assert_no_making(&requests);
            assert!(!d.is_ready());
            let due = env.now.saturating_add(Duration::from_secs(*seconds));
            assert_eq!(d.next_deadline(), Some(due));
            env.now = due;
            let mut out = Queue::uncapped();
            fire(&mut d, &env, &mut out);
        }
        // Missing answers the branch read and resets the wait for the default tip.
        let requests = at_answer(&mut d, &env, Err(Error::Missing));
        assert_no_making(&requests);
        let requests = at_answer(&mut d, &env, Err(error));
        assert_no_making(&requests);
        assert_eq!(d.next_deadline(), Some(env.now.saturating_add(CLIENT.backoff)));
        env.now = env.now.saturating_add(CLIENT.backoff);
        let mut out = Queue::uncapped();
        fire(&mut d, &env, &mut out);
        assert_kept_making(&at_answer(&mut d, &env, Ok(Answer::Commit(TIP))));
    }
}

#[test]
fn a_goal_branch_making_reads_past_a_restored_default_tip() {
    const CURRENT: skein_forge::api::Commit = [5; 32];
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(
        &mut d,
        Event::Restore { record: Stored::BranchHead(BranchHead { name: name(b"main"), commit: TIP, owned: false }) },
    );
    restore_done(&mut d);
    take(&mut d);
    let missing = fixtures::answer_next(&mut d, Err(Error::Missing));
    assert_no_making(&missing);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"main".as_slice().into() }));
    let made = fixtures::answer(&mut d, call, Ok(Answer::Commit(CURRENT)));
    let mut expected = entry();
    expected.effect.write = Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: CURRENT };
    assert!(
        made.contains(&Request::Save {
            release: Some(TAKING),
            read_afresh: None,
            record: Stored::Entry { entry: expected, held: false }
        }),
        "{made:?}"
    );
    assert!(!d.is_ready(), "the effect still waits for its durable commit");
}

#[test]
fn a_goal_branch_read_failure_names_the_resource_and_forge_answer() {
    for read_default in [false, true] {
        let mut d = fresh();
        take(&mut d);
        if read_default {
            fixtures::answer_next(&mut d, Err(Error::Missing));
        }
        let failed = fixtures::answer_next(&mut d, Err(Error::Forbidden));
        assert!(
            failed.contains(&Request::MakeFailed {
                task: GOAL,
                resource: name(GOAL_BRANCH),
                reason: b"forge refused permission".as_slice().into()
            }),
            "{failed:?}"
        );
        assert!(d.hold(&name(GOAL_BRANCH)).is_some());
    }
}

#[test]
fn a_goal_branch_taking_blocks_a_writer_during_its_reads() {
    let mut d = fresh();
    take(&mut d);
    assert_eq!(
        outputs(
            &mut d,
            Event::Claim {
                designs: Box::new([]),
                task: OTHER,
                attempt: 1,
                writes: Box::new([name(GOAL_BRANCH)]),
                holders: Box::new([GOAL])
            }
        )
        .as_ref(),
        &[Request::Taken { task: OTHER, resource: name(GOAL_BRANCH), by: GOAL }]
    );
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(
        &mut d,
        Event::Claim {
            designs: Box::new([]),
            task: OTHER,
            attempt: 1,
            writes: Box::new([name(GOAL_BRANCH)]),
            holders: Box::new([GOAL]),
        },
    );
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, Some(Writer::Run { task: OTHER, attempt: 1 }));
}

#[test]
fn a_goal_branch_taking_waits_for_its_forge_to_connect() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Err(Error::NotConnected));
    assert!(!d.is_ready());
    outputs(
        &mut d,
        Event::Connection {
            forge: REPO.forge,
            state: Connection::Connected { id: GOAL, login: b"bot".as_slice().into() },
        },
    );
    ready(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let found = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_no_making(&found);
}

#[test]
fn a_goal_branch_making_restored_before_sending_keeps_its_taking_entry() {
    let mut before = fresh();
    let saved = missing(&mut before);
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for request in saved {
        if let Request::Save { record, .. } = request {
            outputs(&mut d, Event::Restore { record });
        }
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    let missing = fixtures::answer_next(&mut d, Err(Error::Missing));
    assert!(missing.contains(&Request::RestartDone { stage: RestartStage::ReadAfresh }));
    assert_no_making(&missing);

    fixtures::answer_next(&mut d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Write(Write::CreateBranch { branch: GOAL_BRANCH.into(), commit: TIP }));
    let made = fixtures::answer(&mut d, call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(made.contains(&self::made()), "{made:?}");
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
}

const MERGING: u64 = 23;
const MERGED: skein_forge::api::Commit = [9; 32];
const MERGE_CALLS: u32 = 8; // Inbox, pull and branch reads, merge, then its proof.

fn merge_into_goal(d: &mut Domain) -> Box<[Request]> {
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pull = Some(1);
    row.pending = Some(MERGING);
    row.change.state = temper_engine_domain_forge_change::State::Landing { head: fixtures::HEAD };
    d.changes.insert(row.task, row).expect("fixture change");
    let mut entry = entry();
    entry.number = MERGING;
    entry.task = fixtures::CHANGE;
    entry.effect = skein_forge_client::Effect {
        write: Write::Merge { number: 1, head: fixtures::HEAD, how: skein_forge::api::MergeHow::Style },
        condition: skein_forge_client::Condition::Merge { base: GOAL_BRANCH.into() },
    };
    outputs(d, Event::Enqueue { entry });
    outputs(d, Event::Committed { entry: MERGING });
    let mut merged = false;
    for _ in 0..MERGE_CALLS {
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Items { .. }) => {
                Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }
            }
            Op::Read(Read::Branch { branch }) if branch.as_ref() == GOAL_BRANCH => Answer::Commit(TIP),
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"temper/51" => Answer::Commit(fixtures::HEAD),
            Op::Read(Read::Pull { number: 1, ci: false }) => {
                let mut pull = fixtures::open_pull();
                pull.base = GOAL_BRANCH.into();
                if merged {
                    pull.state = skein_forge::api::State::Closed;
                    pull.merged = Some(MERGED);
                }
                Answer::Pull(pull)
            }
            Op::Write(Write::Merge { head, .. }) => {
                assert_eq!(head, fixtures::HEAD);
                merged = true;
                Answer::Merged(MERGED)
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected goal merge call: {other:?}"),
        };
        let requests = fixtures::answer(d, call, Ok(answer));
        if requests.contains(&Request::Outcome {
            entry: MERGING,
            task: fixtures::CHANGE,
            account: None,
            outcome: skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(MERGED), found: false },
        }) {
            assert!(merged, "outcome follows the merge write");
            return requests;
        }
    }
    panic!("goal merge did not settle")
}

#[test]
fn a_merge_into_a_held_goal_branch_saves_its_head_whatever_the_previous_record() {
    for owned in [Some(true), Some(false), None] {
        let mut d = fresh();
        take(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
        if let Some(owned) = owned {
            d.heads.get_mut(&name(GOAL_BRANCH)).expect("goal head").owned = owned;
        } else {
            d.heads.remove(&name(GOAL_BRANCH));
        }
        let requests = merge_into_goal(&mut d);
        let advanced = BranchHead { name: name(GOAL_BRANCH), commit: MERGED, owned: true };
        assert!(
            requests.contains(&Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::BranchHead(advanced.clone())
            }),
            "owned={owned:?}: {requests:?}"
        );
        assert_eq!(d.heads.get(&name(GOAL_BRANCH)), Some(&advanced), "owned={owned:?}");
        for request in &requests {
            if let Request::Save { record: Stored::BranchHead(head), .. } = request {
                assert_eq!(head, &advanced);
            }
        }
        let mut landing = fixtures::opening_row();
        landing.task = OTHER;
        landing.change.task = OTHER;
        landing.branch = GOAL_BRANCH.into();
        landing.pending = None;
        landing.effect = temper_engine_domain_forge_change::EffectResult::None;
        landing.change.state = temper_engine_domain_forge_change::State::Checking { head: TIP };
        landing.change.last_head = Some(TIP);
        outputs(&mut d, Event::Change { row: landing });
        fixtures::connect(&mut d);
        held_branch::step_at_head_for(&mut d, OTHER, MERGED);
        assert_eq!(
            d.change(OTHER).unwrap().change.state,
            temper_engine_domain_forge_change::State::Checking { head: MERGED }
        );
        assert_eq!(d.change(OTHER).unwrap().drift, None);
    }
}

#[test]
fn a_merge_into_a_watched_goal_branch_saves_the_clients_advanced_tip() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(&mut d, Event::Names { task: GOAL, resources: Box::new([name(GOAL_BRANCH)]) });
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let watched = fixtures::answer(&mut d, call, Ok(Answer::Commit(TIP)));
    assert_saved_goal_tip(&watched, TIP);
    let requests = merge_into_goal(&mut d);
    assert_saved_goal_tip(&requests, MERGED);
}

#[test]
fn a_merge_beside_a_design_writer_leaves_g_written_until_its_fresh_answer() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(&mut d, Event::Names { task: GOAL, resources: Box::new([name(GOAL_BRANCH)]) });
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(
        &mut d,
        Event::Claim {
            task: GOAL,
            attempt: 3,
            writes: Box::new([]),
            designs: Box::new([name(GOAL_BRANCH)]),
            holders: Box::new([]),
        },
    );
    merge_into_goal(&mut d);
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(MERGED));
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().writer, None);
    let tip = [8; 32];
    outputs(
        &mut d,
        Event::Hint {
            hint: skein_forge::api::Hint {
                repository: REPO,
                change: skein_forge::api::Change::BranchMoved { branch: GOAL_BRANCH.into(), head: tip, actor: 7 },
                key: None,
            },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(&mut d, &later, &mut Queue::uncapped());
    fixtures::answer_next(&mut d, Ok(Answer::Commit(tip)));
    assert_eq!(
        d.hold(&name(GOAL_BRANCH)).unwrap().drift,
        None,
        "ending the merge leaves the design writer's forward push owned"
    );
    fixtures::answer_next(&mut d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    outputs(&mut d, Event::Answered { task: GOAL, attempt: 3, pushed: Box::new([(name(GOAL_BRANCH), TIP, true)]) });
    fixtures::answer_next(&mut d, Ok(Answer::Commit(tip)));
    fixtures::answer_next(&mut d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    assert_eq!(
        d.branch_head(&name(GOAL_BRANCH)),
        Some(tip),
        "the fresh head holds the merge and the design, not the report"
    );
    assert!(d.hold(&name(GOAL_BRANCH)).unwrap().design_writers.is_empty());
}

#[test]
fn a_design_answer_read_started_before_a_merge_reads_g_again() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(
        &mut d,
        Event::Claim {
            task: GOAL,
            attempt: 3,
            writes: Box::new([]),
            designs: Box::new([name(GOAL_BRANCH)]),
            holders: Box::new([]),
        },
    );
    outputs(&mut d, Event::Answered { task: GOAL, attempt: 3, pushed: Box::new([]) });
    let (stale, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    merge_into_goal(&mut d);
    fixtures::answer(&mut d, stale, Ok(Answer::Commit(TIP)));
    let (fresh, op) = fixtures::next_call(&mut d);
    assert_eq!(
        op,
        Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }),
        "a read from before the merge cannot judge its head"
    );
    fixtures::answer(&mut d, fresh, Ok(Answer::Commit(MERGED)));
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(MERGED));
    let hold = d.hold(&name(GOAL_BRANCH)).unwrap();
    assert_eq!(hold.drift, None);
    assert!(hold.design_writers.is_empty());
}

fn assert_saved_goal_tip(requests: &[Request], tip: skein_forge::api::Commit) {
    let resource =
        skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Branch(GOAL_BRANCH.into()) };
    let mut saved = false;
    for request in requests {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request
            && row.watch.resource == resource
        {
            assert_eq!(row.cached.tip, Some(tip));
            saved = true;
        }
    }
    assert!(saved, "the client's advanced tip is durable: {requests:?}");
}

const DESIGN: skein_forge::api::Commit = [8; 32];

fn watched_design_goal() -> Domain {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(&mut d, Event::Names { task: GOAL, resources: Box::new([name(GOAL_BRANCH)]) });
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    outputs(
        &mut d,
        Event::Claim {
            task: GOAL,
            attempt: 3,
            writes: Box::new([]),
            designs: Box::new([name(GOAL_BRANCH)]),
            holders: Box::new([]),
        },
    );
    d
}

fn delayed_goal_merge(d: &mut Domain) -> Token {
    let mut row = fixtures::opening_row();
    row.base = GOAL_BRANCH.into();
    row.pull = Some(1);
    row.pending = Some(MERGING);
    row.change.state = temper_engine_domain_forge_change::State::Landing { head: fixtures::HEAD };
    d.changes.insert(row.task, row).expect("fixture change");
    let mut entry = entry();
    entry.number = MERGING;
    entry.task = fixtures::CHANGE;
    entry.effect = skein_forge_client::Effect {
        write: Write::Merge { number: 1, head: fixtures::HEAD, how: skein_forge::api::MergeHow::Style },
        condition: skein_forge_client::Condition::Merge { base: GOAL_BRANCH.into() },
    };
    outputs(d, Event::Enqueue { entry });
    outputs(d, Event::Committed { entry: MERGING });
    for _ in 0..MERGE_CALLS {
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Items { .. }) => {
                Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }
            }
            Op::Read(Read::Branch { branch }) if branch.as_ref() == GOAL_BRANCH => Answer::Commit(TIP),
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"temper/51" => Answer::Commit(fixtures::HEAD),
            Op::Read(Read::Pull { number: 1, ci: false }) => {
                let mut pull = fixtures::open_pull();
                pull.base = GOAL_BRANCH.into();
                Answer::Pull(pull)
            }
            Op::Write(Write::Merge { .. }) => return call,
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected delayed merge call: {other:?}"),
        };
        assert_no_goal_drift(&fixtures::answer(d, call, Ok(answer)));
    }
    panic!("merge was not handed")
}

fn finish_goal_merge(d: &mut Domain, call: Token) -> Box<[Request]> {
    let mut requests = fixtures::answer(d, call, Ok(Answer::Merged(MERGED)));
    if !requests.contains(&merge_outcome()) {
        let (call, op) = fixtures::next_call(d);
        assert_eq!(op, Op::Read(Read::Pull { number: 1, ci: false }));
        let mut pull = fixtures::open_pull();
        pull.base = GOAL_BRANCH.into();
        pull.state = skein_forge::api::State::Closed;
        pull.merged = Some(MERGED);
        requests = fixtures::answer(d, call, Ok(Answer::Pull(pull)));
    }
    assert!(requests.contains(&merge_outcome()), "{requests:?}");
    assert_no_goal_drift(&requests);
    requests
}

fn finish_design(d: &mut Domain, tip: skein_forge::api::Commit, baseline: skein_forge::api::Commit) -> Box<[Request]> {
    assert_no_goal_drift(&outputs(d, Event::Answered { task: GOAL, attempt: 3, pushed: Box::new([]) }));
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let requests = fixtures::answer(d, call, Ok(Answer::Commit(tip)));
    assert_no_goal_drift(&requests);
    if baseline == tip {
        return requests;
    }
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: tip, after: baseline, page: 1 }));
    let requests = fixtures::answer(d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    assert_no_goal_drift(&requests);
    requests
}

fn observe_goal(d: &mut Domain, tip: skein_forge::api::Commit, actor: u64) -> Box<[Request]> {
    outputs(
        d,
        Event::Hint {
            hint: skein_forge::api::Hint {
                repository: REPO,
                change: skein_forge::api::Change::BranchMoved { branch: GOAL_BRANCH.into(), head: tip, actor },
                key: None,
            },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(d, &later, &mut Queue::uncapped());
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    fixtures::answer(d, call, Ok(Answer::Commit(tip)))
}

fn assert_no_goal_drift(requests: &[Request]) {
    for request in requests {
        if let Request::Drift { .. } = request {
            panic!("no drift on temper's own writes: {request:?}");
        }
    }
}

#[test]
fn a_late_merge_record_never_moves_g_back_from_its_design_push() {
    let mut d = watched_design_goal();
    let merge = delayed_goal_merge(&mut d);
    let design = finish_design(&mut d, DESIGN, TIP);
    assert_saved_goal_tip(&design, DESIGN);
    let merged = finish_goal_merge(&mut d, merge);
    assert_no_goal_drift(&merged);
    assert_waiting_design(&d);
    assert_eq!(
        outputs(
            &mut d,
            Event::Claim {
                task: OTHER,
                attempt: 1,
                writes: Box::new([name(GOAL_BRANCH)]),
                designs: Box::new([]),
                holders: Box::new([GOAL]),
            }
        )
        .as_ref(),
        &[Request::Taken { task: OTHER, resource: name(GOAL_BRANCH), by: GOAL }]
    );
    let compared = answer_written_comparison(&mut d, true);
    assert_no_goal_drift(&compared);
    assert_goal_head(&d, DESIGN);
    for request in merged.iter().chain(compared.iter()) {
        if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request
            && row.watch.resource.what == skein_forge_client::What::Branch(GOAL_BRANCH.into())
        {
            assert_eq!(row.cached.tip, Some(DESIGN), "no late Pushed(MERGED)");
        }
    }
    let watched = observe_goal(&mut d, DESIGN, 7);
    assert_no_goal_drift(&watched);
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().drift, None);
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(DESIGN));
}

fn assert_goal_head(d: &Domain, commit: skein_forge::api::Commit) {
    assert_eq!(d.heads.get(&name(GOAL_BRANCH)), Some(&BranchHead { name: name(GOAL_BRANCH), commit, owned: true }));
    let hold = d.hold(&name(GOAL_BRANCH)).unwrap();
    assert_eq!(hold.drift, None);
    assert_eq!(hold.writer, None);
    assert!(hold.design_writers.is_empty());
    assert_eq!(hold.beside, None);
    assert_eq!(goal_slot(d), Some((GOAL, false)));
}

fn assert_waiting_design(d: &Domain) {
    assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(DESIGN));
    assert_eq!(d.hold(&name(GOAL_BRANCH)).unwrap().beside, Some(HeadBeside { head: DESIGN, waiting: Some(MERGED) }));
    assert_eq!(goal_slot(d), Some((GOAL, true)));
}

fn answer_written_comparison(d: &mut Domain, contains: bool) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: DESIGN, after: MERGED, page: 1 }));
    let commits: Box<[_]> = if contains {
        Box::new([])
    } else {
        Box::new([skein_forge::api::CommitMessage { id: MERGED, parents: Box::new([DESIGN]), message: Box::new([]) }])
    };
    fixtures::answer(d, call, Ok(Answer::Commits { commits, more: false }))
}

fn goal_slot(d: &Domain) -> Option<(u64, bool)> {
    let mut row = fixtures::opening_row();
    row.branch = GOAL_BRANCH.into();
    domain::change_branch_hold(d, &row)
}

fn merge_outcome() -> Request {
    Request::Outcome {
        entry: MERGING,
        task: fixtures::CHANGE,
        account: None,
        outcome: skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Merged(MERGED), found: false },
    }
}

#[test]
fn either_forge_order_and_record_order_keeps_the_latest_owned_goal_head() {
    // The fourth ordering is the late-merge reproduction above.
    for (merge_in_forge_first, design_record_first, tip) in
        [(true, false, DESIGN), (false, true, MERGED), (false, false, MERGED)]
    {
        let mut d = watched_design_goal();
        let merge = delayed_goal_merge(&mut d);
        if design_record_first {
            finish_design(&mut d, DESIGN, TIP);
            assert_no_goal_drift(&finish_goal_merge(&mut d, merge));
            let compared = answer_written_comparison(&mut d, false);
            assert_no_goal_drift(&compared);
            assert_saved_goal_tip(&compared, MERGED);
        } else {
            assert_no_goal_drift(&finish_goal_merge(&mut d, merge));
            let read = if merge_in_forge_first { DESIGN } else { MERGED };
            let design = finish_design(&mut d, read, MERGED);
            assert_saved_goal_tip(&design, tip);
        }
        assert_goal_head(&d, tip);
        assert_no_goal_drift(&observe_goal(&mut d, tip, 7));
    }
}

fn settled_late_merge() -> Domain {
    let mut d = watched_design_goal();
    let merge = delayed_goal_merge(&mut d);
    finish_design(&mut d, DESIGN, TIP);
    finish_goal_merge(&mut d, merge);
    answer_written_comparison(&mut d, true);
    assert_goal_head(&d, DESIGN);
    d
}

#[test]
fn outside_pushes_and_backward_moves_still_drift_after_a_late_merge() {
    const OUTSIDE: skein_forge::api::Commit = [6; 32];
    for (observed, actor) in [(OUTSIDE, 8), (MERGED, 7)] {
        let mut d = settled_late_merge();
        let requests = observe_goal(&mut d, observed, actor);
        assert!(requests.contains(&Request::Drift { task: GOAL, resource: name(GOAL_BRANCH) }), "{requests:?}");
        assert_eq!(
            d.hold(&name(GOAL_BRANCH)).unwrap().drift.as_ref().unwrap().change,
            DriftChange::Branch { expected: DESIGN, observed: Some(observed) }
        );
        assert_eq!(d.branch_head(&name(GOAL_BRANCH)), Some(DESIGN));
    }
}

#[test]
fn a_waiting_late_merge_comparison_is_read_again_after_restore() {
    let mut d = watched_design_goal();
    let merge = delayed_goal_merge(&mut d);
    finish_design(&mut d, DESIGN, TIP);
    let requests = finish_goal_merge(&mut d, merge);
    let hold = d.hold(&name(GOAL_BRANCH)).unwrap().clone();
    assert!(requests.contains(&Request::Save { record: Stored::Hold(hold.clone()), release: None, read_afresh: None }));
    let mut restored = domain();
    for record in [
        Stored::Repository(adopted()),
        Stored::Hold(hold),
        Stored::BranchHead(BranchHead { name: name(GOAL_BRANCH), commit: DESIGN, owned: true }),
        Stored::Names { task: GOAL, resources: Box::new([name(GOAL_BRANCH)]) },
    ] {
        outputs(&mut restored, Event::Restore { record });
    }
    restore_done(&mut restored);
    assert_waiting_design(&restored);
    assert_no_goal_drift(&answer_written_comparison(&mut restored, true));
    assert_goal_head(&restored, DESIGN);
    // Both the hold's restart and the watch read the same settled head.
    for _ in 0_u8..2 {
        let (call, op) = fixtures::next_call(&mut restored);
        assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
        assert_no_goal_drift(&fixtures::answer(&mut restored, call, Ok(Answer::Commit(DESIGN))));
    }
    assert_goal_head(&restored, DESIGN);
}

#[test]
fn a_merge_without_a_head_beside_records_at_once_without_comparison() {
    let mut d = fresh();
    take(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(TIP)));
    let requests = merge_into_goal(&mut d);
    assert_goal_head(&d, MERGED);
    for request in requests.iter().chain(ready(&mut d).iter()) {
        if let Request::Call { op: Op::Read(Read::Commits { .. }), .. } = request {
            panic!("no ancestry read without a head beside: {request:?}");
        }
    }
}

#[test]
fn a_makings_branch_read_gives_up_during_backoff() {
    making_read_bound(
        None,
        Error::Unavailable,
        b"no progress at making for 30 minutes: waiting for branch temper/7/g7; the last answer: forge unavailable",
    );
}

#[test]
fn a_new_making_step_gets_a_full_read_bound() {
    let initial = env();
    let advanced = at_wall(&initial, advanced_wall(initial.wall, Duration::from_secs(1)));
    making_read_bound(
        Some(&advanced),
        Error::Timeout,
        b"no progress at making for 30 minutes: waiting for branch main; the last answer: forge timed out",
    );
}

fn making_read_bound(landing: Option<&Env<Limits>>, error: Error, words: &[u8]) {
    let mut d = fresh();
    let mut initial = env();
    initial.limits.client.read_attempts = 1;
    take(&mut d);
    let clock = if let Some(landing) = landing {
        at_answer(&mut d, landing, Err(Error::Missing));
        Env { wall: landing.wall, ..initial }
    } else {
        initial
    };
    at_answer(&mut d, &clock, Err(error));
    let bound = advanced_wall(clock.wall, clock.limits.waits.step);
    let before = fire_at(&mut d, &clock, Wall::from_nanos(bound.as_nanos().saturating_sub(1)));
    assert_no_making(&before);
    let failed = fire_at(&mut d, &clock, bound);
    let expected = Request::MakeFailed { task: GOAL, resource: name(GOAL_BRANCH), reason: words.into() };
    assert!(failed.contains(&expected), "{failed:?}");
    assert!(!d.goal_branch_waiting(&name(GOAL_BRANCH)));
}

#[test]
fn a_creating_branch_keeps_its_in_flight_making() {
    let mut d = fresh();
    let (call, _) = making(&mut d);
    outputs(&mut d, Event::RetryGoalBranch { task: GOAL });
    let clock = env();
    let out = fire_at(&mut d, &clock, advanced_wall(clock.wall, clock.limits.waits.step));
    assert_no_making(&out);
    let found = fixtures::answer(&mut d, call, Ok(Answer::Branch(BranchCreation::Created)));
    assert!(found.contains(&made()), "{found:?}");
}

#[test]
fn a_release_waiting_for_goal_making_fails_at_its_step_figure() {
    let mut d = fresh();
    let mut clock = env();
    clock.limits.client.read_attempts = 1;
    take(&mut d);
    at_answer(&mut d, &clock, Err(Error::Timeout));
    release(&mut d, ReleaseEnding::Done);
    let bound = advanced_wall(clock.wall, clock.limits.waits.step);
    let failed = fire_at(&mut d, &clock, bound);
    let expected = Request::ReleaseFailed { task: GOAL, entry: None, words: b"no progress at release for 30 minutes: waiting for the goal branch's making; the last answer: forge timed out".as_slice().into() };
    assert!(failed.contains(&expected), "{failed:?}");
    assert!(d.hold(&name(GOAL_BRANCH)).is_some());
}
