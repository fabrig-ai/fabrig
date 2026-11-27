#![expect(clippy::disallowed_types, reason = "ordinary tests collect diagnostic lines as text")]
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect captured requests and diagnostic text")]
#![expect(
    clippy::wildcard_enum_match_arm,
    reason = "the fixture filters table facts and client calls from unrelated outputs"
)]
use super::*;
use crate::domain::{self, Handoff, Outboxed, RestorePhase};
use skein_forge_client::{Condition, Effect, Entry, Outcome};
use skein_lib::{Duration, Token};
const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
fn setup(entries: u32, pending: u32) -> (Domain, Env<Limits>) {
    let mut env = crate::tests::env();
    env.limits.entries = entries;
    env.limits.client.entries = entries;
    env.limits.client.pending = pending;
    let mut d = crate::tests::domain_with_limits(&env.limits);
    d.restore_phase = RestorePhase::Active;
    let mut repository = crate::tests::adopted();
    repository.owner = b"work".as_slice().into();
    repository.name = b"hello".as_slice().into();
    d.repositories.insert(REPO, Box::new(repository)).unwrap();
    (d, env)
}
fn entry(number: u64, task: u64) -> Entry {
    Entry {
        number,
        task,
        repository: REPO,
        effect: Effect { condition: Condition::None, write: skein_forge::api::Write::Close { number: 12 } },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}
fn keep(d: &mut Domain, number: u64, task: u64) {
    d.entries
        .insert(
            number,
            Outboxed { entry: entry(number, task), owner_ended: None, held: false, handoff: Handoff::Uncommitted },
        )
        .unwrap();
}
fn facts(out: &Queue<Request>) -> Vec<String> {
    out.iter()
        .filter_map(|r| match r {
            Request::FullTableFact { words } => Some(String::from_utf8(words.to_vec()).unwrap()),
            _ => None,
        })
        .collect()
}
fn at(env: &mut Env<Limits>, seconds: u64) {
    env.now = crate::tests::env().now.saturating_add(Duration::from_secs(seconds));
    env.wall =
        Wall::from_nanos(crate::tests::env().wall.as_nanos().saturating_add(Duration::from_secs(seconds).as_nanos()));
}
fn full_outbox() -> (Domain, Env<Limits>) {
    // One retained entry fills the only outbox place; reads have spare room.
    let (mut d, env) = setup(1, 4);
    keep(&mut d, 7, 1);
    update(&mut d, &env, &mut Queue::uncapped());
    (d, env)
}

fn published_outbox() -> (Domain, Env<Limits>) {
    let (mut d, mut env) = full_outbox();
    at(&mut env, 30);
    update(&mut d, &env, &mut Queue::uncapped());
    (d, env)
}

fn waiting_change(d: &mut Domain, task: u64, order: u64) {
    let mut row = crate::tests::fixtures::opening_row();
    row.task = task;
    row.change.task = task;
    row.room = Some(order);
    d.changes.insert(task, row).unwrap();
}

fn waiting_making(d: &mut Domain, env: &Env<Limits>, task: u64, order: u64) -> Token {
    let name = crate::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: crate::What::Branch(Box::new([
            b"temper".as_slice().into(),
            task.to_string().into_bytes().into_boxed_slice(),
            format!("g{task}").into_bytes().into_boxed_slice(),
        ])),
    };
    d.holds
        .insert(
            name.clone(),
            crate::Hold {
                name: name.clone(),
                task,
                writer: None,
                design_writers: Box::new([]),
                beside: None,
                drift: None,
            },
        )
        .unwrap();
    domain::goal_branch::start(d, env, name, None, &mut Queue::uncapped());
    let owner = *d.makings.iter().find(|(_, pending)| pending.task == task).unwrap().0;
    let pending = d.makings.get_mut(&owner).unwrap();
    pending.outbox_wait = Some(domain::goal_branch::RoomWait { order, since: env.wall });
    pending.state = domain::LostReadState::WaitingForOutbox;
    owner
}

#[test]
fn leaving_the_line_removes_an_ask_from_the_next_full_report() {
    let (mut d, env) = setup(1, 4);
    let waiter = crate::room::Waiter::Change(7);
    waiting_change(&mut d, 7, 1);
    crate::room::reserve_ask(&mut d, waiter, 7);
    update(&mut d, &env, &mut Queue::uncapped());
    assert_eq!(d.inspection_reservations().as_ref(), &[7]);
    crate::room::leave(&mut d, waiter);
    d.changes.get_mut(&7).unwrap().room = None;
    domain::resume(&mut d, &env, &mut Queue::uncapped());
    assert!(d.inspection_reservations().is_empty());
    assert!(!d.full.reservations.contains_key(&Reservation::Asked(waiter)));
    assert_eq!(d.full.sequence, 1);
    assert!(d.full.outbox.is_none());
}

#[test]
fn pruning_inside_ask_reports_room_before_admitting_the_next_waiter() {
    let (mut d, mut env) = setup(1, 4);
    let owner = waiting_making(&mut d, &env, 7, 1);
    let waiter = crate::room::Waiter::Making(owner);
    crate::room::reserve_ask(&mut d, waiter, 7);
    waiting_change(&mut d, 8, 2);
    d.room.ready = true;
    update(&mut d, &env, &mut Queue::uncapped());
    // The first update in ask still sees the reservation. Its stale-making
    // pruning frees it before the next waiter fills the only place again.
    let name = d.makings.get(&owner).unwrap().name.clone();
    d.holds.remove(&name);
    at(&mut env, 30);
    let mut out = Queue::uncapped();
    crate::room::ask(&mut d, &env, &mut out);
    let expected = [
        Request::MarkedForges,
        Request::FullTableFact {
            words: b"the forge's outbox full for 30 s: 1 of 1, held by room asked for task 7; 2 waiting for room"
                .as_slice()
                .into(),
        },
        Request::FullTableFact { words: b"the forge's outbox has room again".as_slice().into() },
        Request::MarkedForges,
        Request::WakeProcedure { task: 8 },
    ];
    assert!(out.iter().eq(&expected), "{out:?}");
    assert!(!d.makings.contains_key(&owner));
    assert_eq!(d.inspection_reservations().as_ref(), &[8]);
    assert_eq!(d.full.sequence, 2);
    assert!(d.forges().full.is_empty(), "the new full interval has not reached its bound");
    let full = d.full.outbox.as_ref().unwrap();
    assert_eq!(full.wall, env.wall.as_nanos());
    assert_eq!(full.clock.deadline(), env.now.saturating_add(crate::WAIT_REPORT));
}

#[test]
fn ask_keeps_one_first_come_line_and_skips_ineligible_waiters() {
    use crate::room::Waiter;
    use temper_engine_domain_forge_change as change;
    use temper_engine_domain_forge_landing as landing;
    for globally_held in [false, true] {
        let (mut d, env) = setup(4, 4);
        d.room.ready = true;
        d.landing.held = globally_held;
        let row = crate::LandingRow {
            read_hold: None,
            gate_hold: None,
            last_event: None,
            turn_since: None,
            waiting: None,
            follows: None,
            flake_reruns: Box::new([]),
            room: Some(1),
            job: None,
            landing: landing::Landing {
                last_left: None,
                searched: None,
                round: landing::Round::new(b"temper/1/l50".as_slice().into()),
                task: 50,
                project: 5,
                phase: if globally_held {
                    landing::Phase::Preparing
                } else {
                    landing::Phase::Held {
                        was: Box::new(landing::Phase::Preparing),
                        why: landing::Hold::Faults { member: 0, gate: 0 },
                    }
                },
                members: Box::new([]),
                preparation: 0,
                stale_proofs: 0,
                last_prepare: None,
                branch: b"main".as_slice().into(),
            },
            previous_asks: Box::new([]),
            entries: Box::new([]),
        };
        assert!(d.landing.remember(row).is_none());
        waiting_change(&mut d, 20, 2);
        crate::room::reserve_ask(&mut d, Waiter::Change(20), 20);
        waiting_change(&mut d, 21, 3);
        crate::room::pass(&mut d, 21);
        waiting_change(&mut d, 22, 4);
        d.changes.get_mut(&22).unwrap().change.state = change::State::Ended;
        waiting_change(&mut d, 23, 5);
        d.changes.get_mut(&23).unwrap().change.state = change::State::Held {
            was: Box::new(change::State::Opening { requested: false }),
            why: change::Hold::Failed,
        };
        waiting_change(&mut d, 30, 6);
        d.releases
            .insert(
                10,
                crate::ReleaseRow {
                    room: Some(7),
                    task: 10,
                    root: 10,
                    ending: crate::ReleaseEnding::Cancelled,
                    close_pull: None,
                    comment: None,
                    pending: None,
                    failed: false,
                    effects_settled: true,
                    releasing: true,
                },
            )
            .unwrap();
        let owner = waiting_making(&mut d, &env, 40, 8);
        let mut out = Queue::uncapped();
        crate::room::ask(&mut d, &env, &mut out);
        let expected = [
            Request::WakeProcedure { task: 30 },
            Request::ContinueRelease { task: 10 },
            Request::ContinueGoalBranch { task: 40 },
        ];
        assert!(out.iter().eq(&expected), "{out:?}");
        assert_eq!(
            d.room.asked.iter().map(|(waiter, _)| *waiter).collect::<Vec<_>>(),
            [Waiter::Change(20), Waiter::Change(30), Waiter::Making(owner), Waiter::Release(10),]
        );
        for (waiter, sequence) in [(Waiter::Change(30), 2), (Waiter::Release(10), 3), (Waiter::Making(owner), 4)] {
            assert_eq!(d.full.reservations.get(&Reservation::Asked(waiter)).unwrap().0, sequence);
        }
        assert_eq!(crate::room::free(&d, &env.limits), 0);
        out.clear();
        crate::room::leave(&mut d, Waiter::Release(10));
        d.releases.get_mut(&10).unwrap().room = None;
        d.landing.held = false;
        d.landing.rows.get_mut(&50).unwrap().landing.phase = landing::Phase::Preparing;
        crate::room::ask(&mut d, &env, &mut out);
        assert!(out.iter().eq(&[Request::WakeProcedure { task: 50 }]), "{out:?}");
        assert_eq!(d.full.reservations.get(&Reservation::Asked(Waiter::Landing(50))), Some(&(5, 50, None)));
    }
}

#[test]
fn saturated_report_deadlines_keep_each_update_observable() {
    let (mut d, mut env) = full_outbox();
    d.room.ready = true;
    env.now = Time::from_nanos(u64::MAX);
    let mut out = Queue::uncapped();
    domain::resume(&mut d, &env, &mut out);
    // One at drive's entrance, then three in each ask. The second ask's
    // repeated reports spend this turn's output burst, so drive returns there.
    let lines = facts(&out);
    assert_eq!(lines.len(), 7);
    assert!(lines.iter().all(|line| line == &lines[0]));
    assert_eq!(d.full.outbox.as_ref().unwrap().clock.deadline(), env.now);
}

#[test]
fn a_full_table_stays_unpublished_before_its_report_bound() {
    let (mut d, mut env) = full_outbox();
    let mut out = Queue::uncapped();
    at(&mut env, 29);
    update(&mut d, &env, &mut out);
    assert!(out.is_empty());
    assert!(d.forges().full.is_empty());
    assert_eq!(d.next_deadline(), Some(crate::tests::env().now.saturating_add(crate::WAIT_REPORT)));
}

#[test]
fn a_full_table_reports_once_at_its_report_bound() {
    let (mut d, mut env) = full_outbox();
    let mut out = Queue::uncapped();
    at(&mut env, 30);
    update(&mut d, &env, &mut out);
    assert_eq!(facts(&out), ["the forge's outbox full for 30 s: 1 of 1, held by entry 7 (task 1)"]);
    assert_eq!(d.forges().full[0].words.as_ref(), b"1 of 1, held by entry 7 (task 1)");
    out.clear();
    update(&mut d, &env, &mut out);
    assert!(out.is_empty(), "the same instant cannot report twice");
}

#[test]
fn a_published_table_marks_only_changed_holders_between_reports() {
    let (mut d, mut env) = published_outbox();
    let mut out = Queue::uncapped();
    at(&mut env, 31);
    update(&mut d, &env, &mut out);
    assert!(out.is_empty(), "unchanged holders need no new census");
    d.entries.get_mut(&7).unwrap().entry.task = 2;
    update(&mut d, &env, &mut out);
    assert_eq!(out.len(), 1);
    assert!(out.iter().any(|request| matches!(request, Request::MarkedForges)));
    assert_eq!(d.forges().full[0].words.as_ref(), b"1 of 1, held by entry 7 (task 2)");
}

#[test]
fn clearing_a_published_table_reports_room_again() {
    let (mut d, mut env) = published_outbox();
    let mut out = Queue::uncapped();
    at(&mut env, 31);
    d.entries.remove(&7);
    update(&mut d, &env, &mut out);
    assert_eq!(facts(&out), ["the forge's outbox has room again"]);
    assert!(out.iter().any(|request| matches!(request, Request::MarkedForges)));
    assert!(d.forges().full.is_empty());
}

#[test]
fn clearing_an_unpublished_table_stays_silent() {
    let (mut d, mut env) = full_outbox();
    let mut out = Queue::uncapped();
    at(&mut env, 29);
    d.entries.remove(&7);
    update(&mut d, &env, &mut out);
    assert!(out.is_empty());
    assert!(d.forges().full.is_empty());
    assert_eq!(d.full.deadline(), None);
}

#[test]
fn outbox_reports_bound_doublings_recovery_and_document_changes_without_inputs() {
    let (mut d, mut env) = setup(2, 4);
    keep(&mut d, 9, 2);
    keep(&mut d, 7, 1);
    let mut waiter = crate::tests::fixtures::opening_row();
    waiter.room = Some(1);
    d.changes.insert(waiter.task, waiter).unwrap();
    let mut out = Queue::uncapped();
    update(&mut d, &env, &mut out);
    assert_eq!(d.next_deadline(), Some(env.now.saturating_add(crate::WAIT_REPORT)));
    assert!(d.forges().full.is_empty());
    for (seconds, expected) in [(29, None), (30, Some("30 s")), (45, None), (60, Some("60 s"))] {
        at(&mut env, seconds);
        domain::fire(&mut d, &env, &mut out);
        let lines = facts(&out);
        let expected: Vec<String> = expected.into_iter().map(|age| format!("the forge's outbox full for {age}: 2 of 2, held by entry 7 (task 1), entry 9 (task 2); 1 waiting for room")).collect();
        assert_eq!(lines, expected);
        out.clear();
    }
    let full = d.forges().full;
    assert_eq!(
        full.as_ref(),
        &[FullTable {
            table: FullKind::Outbox,
            since: crate::tests::env().wall.as_nanos(),
            words: b"2 of 2, held by entry 7 (task 1), entry 9 (task 2); 1 waiting for room".as_slice().into()
        }]
    );
    // A changed holder is watched immediately, but does not add a timed fact.
    d.entries.get_mut(&9).unwrap().entry.task = 3;
    update(&mut d, &env, &mut out);
    assert!(out.iter().any(|r| matches!(r, Request::MarkedForges)));
    assert_eq!(facts(&out), Vec::<String>::new());
    assert!(d.forges().full[0].words.windows(6).any(|w| w == b"task 3"));
    out.clear();
    domain::scripted_outcome(&mut d, &env, 7, 1, Outcome::Withdrawn, &mut out);
    assert_eq!(facts(&out), ["the forge's outbox has room again"]);
    assert!(d.forges().full.is_empty());
    assert!(out.iter().any(|r| matches!(r, Request::MarkedForges)));
    out.clear();
    keep(&mut d, 7, 1);
    update(&mut d, &env, &mut out);
    at(&mut env, 80);
    domain::scripted_outcome(&mut d, &env, 7, 1, Outcome::Withdrawn, &mut out);
    assert!(facts(&out).is_empty(), "short full intervals stay silent");
}
#[test]
fn catching_up_reports_both_halves_and_stops_only_when_both_finish() {
    use crate::tests::{fixtures::next_call, outputs};
    use crate::{Event, Stored};
    use skein_forge::api::{Answer, Op};
    let mut d = crate::tests::fixtures::connected_domain();
    let mut env = crate::tests::env();
    let mut repository = crate::tests::adopted();
    repository.owner = b"work".as_slice().into();
    repository.name = b"hello".as_slice().into();
    let repository_id = repository.provider;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    let mut entry = crate::tests::status(2, 40);
    entry.effect.write = skein_forge::api::Write::CreateBranch { branch: b"saved".as_slice().into(), commit: [3; 32] };
    entry.start = Some(skein_forge_client::Position { at: env.now, comment: 0, review: 0, review_page: 1 });
    entry.attempt = Some(skein_forge_client::Attempt {
        sent: env.now,
        deadline: env.now.saturating_add(Duration::from_secs(600)),
        wall: env.wall,
        expires: Wall::from_nanos(env.wall.as_nanos().saturating_add(Duration::from_secs(600).as_nanos())),
    });
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: false } });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(
        &mut d,
        Event::Names {
            task: 7,
            resources: Box::new([crate::Name {
                forge: repository_id.forge,
                repository: repository_id.repository,
                what: crate::What::Branch(Box::new([b"main".as_slice().into()])),
            }]),
        },
    );
    let initial = outputs(&mut d, Event::CatchUp);
    assert!(!initial.iter().any(|r| matches!(r, Request::FullTableFact { .. })));
    let first = next_call(&mut d);
    let second = next_call(&mut d);
    let (fresh, find) = if first.1 == Op::Read(Read::Branch { branch: b"main".as_slice().into() }) {
        (first, second)
    } else {
        (second, first)
    };
    assert_eq!(find.1, Op::Read(Read::Branch { branch: b"saved".as_slice().into() }));
    let mut out = Queue::uncapped();
    for (seconds, age) in [(29, None), (30, Some("30 s")), (45, None), (60, Some("60 s"))] {
        at(&mut env, seconds);
        domain::fire(&mut d, &env, &mut out);
        let expected: Vec<_> = age.into_iter().map(|age| format!("the forge catching up for {age}: 1 fresh read (work/hello branch main), 1 uncertain write (entry 2 of task 40)")).collect();
        assert_eq!(facts(&out), expected);
        out.clear();
    }
    for (call, complete) in [(find.0, false), (fresh.0, true)] {
        domain::step(
            &mut d,
            &env,
            Event::Client(skein_forge_client::Event::Answered {
                call,
                cost: 1,
                free: 0,
                rate: None,
                result: Ok(Answer::Commit([3; 32])),
            }),
            &mut out,
        );
        let expected: Vec<_> = if complete { vec!["the forge has caught up".to_owned()] } else { vec![] };
        assert_eq!(facts(&out), expected);
        out.clear();
    }
    at(&mut env, 120);
    domain::fire(&mut d, &env, &mut out);
    assert_eq!(facts(&out), Vec::<String>::new());
}

#[test]
fn catching_up_report_deadline_needs_no_client_call() {
    let (mut d, mut env) = setup(8, 4);
    d.restart_fresh = Some(false);
    d.catching_up_report = Some(Report::new(env.now));
    assert_eq!(d.next_deadline(), Some(env.now.saturating_add(crate::WAIT_REPORT)));
    let mut out = Queue::uncapped();
    at(&mut env, 30);
    domain::fire(&mut d, &env, &mut out);
    assert_eq!(facts(&out), ["the forge catching up for 30 s: nothing named"]);
}

#[test]
fn reservations_follow_entries_in_creation_order_and_only_eight_holders_are_named() {
    let (mut d, env) = setup(10, 4);
    for number in 1..=7 {
        keep(&mut d, number, number);
    }
    let waiter = crate::room::Waiter::Change(42);
    waiting_change(&mut d, 42, 1);
    crate::room::reserve_ask(&mut d, waiter, 42);
    assert!(d.reserve_call(&env, Token::new(1), 43));
    d.projection_room.insert(2).unwrap();
    d.full.reserve(Reservation::Projection(2), 44, None);
    let words = outbox_words(&d, 10);
    assert!(words.ends_with(b"room asked for task 42, and 2 more; 1 waiting for room"));
    for number in 1..=7 {
        d.entries.remove(&number);
    }
    assert_eq!(
        outbox_words(&d, 3).as_ref(),
        b"3 of 3, held by room asked for task 42, room for task 43's call, room for goal 44's projection; 1 waiting for room"
    );
    assert_eq!(d.room.asked.len(), 1);
    assert_eq!(d.call_room.len(), 1);
    assert_eq!(d.projection_room.len(), 1);
}
#[test]
fn read_admission_reports_oldest_holders_doublings_and_room_after_reclaim() {
    let (mut d, mut env) = setup(8, 2);
    let owner = Token::new(41);
    d.lost
        .insert(
            owner,
            super::super::PendingLost {
                task: 42,
                attempt: 1,
                name: crate::Name {
                    forge: 1,
                    repository: 2,
                    what: crate::What::Branch(Box::new([b"temper".as_slice().into(), b"42".as_slice().into()])),
                },
                comparison: None,
                baseline: None,
                state: super::super::LostReadState::Asked,
                last_answer: None,
                kind: super::super::LostWriterKind::Written,
            },
        )
        .unwrap();
    let mut out = Queue::uncapped();
    for (owner, read) in [
        (owner, Read::Pull { number: 12, ci: false }),
        (Token::new(99), Read::Branch { branch: b"main".as_slice().into() }),
    ] {
        domain::child(&mut d, &env, skein_forge_client::Event::Read { owner, repository: REPO, read }, &mut out);
    }
    assert!(d.forges().full.is_empty());
    for (seconds, expected) in [(29, None), (30, Some("30 s")), (45, None), (60, Some("60 s"))] {
        at(&mut env, seconds);
        domain::fire(&mut d, &env, &mut out);
        let expected: Vec<String> = expected.into_iter().map(|age| format!("the forge's read admission full for {age}: 2 of 2, held by pull 12 of work/hello for task 42, branch main of work/hello")).collect();
        assert_eq!(facts(&out), expected);
        out.clear();
    }
    assert_eq!(d.forges().full[0].table, FullKind::ReadAdmission);
    // Free the unowned branch read; its admission place remains until reclaim.
    skein_forge_client::resume(&mut d.client, &env.with(env.limits.client), &mut d.client_out);
    d.client_out.clear();
    skein_forge_client::resume(&mut d.client, &env.with(env.limits.client), &mut d.client_out);
    let call = d
        .client_out
        .iter()
        .find_map(|r| match r {
            skein_forge_client::Request::Call { call, .. } => Some(*call),
            _ => None,
        })
        .unwrap();
    d.client_out.clear();
    domain::child(
        &mut d,
        &env,
        skein_forge_client::Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: None,
            result: Err(skein_forge::api::Error::Missing),
        },
        &mut out,
    );
    assert_eq!(facts(&out), Vec::<String>::new());
    d.reclaim();
    assert!(d.is_ready());
    domain::resume(&mut d, &env, &mut out);
    assert_eq!(facts(&out), ["the forge's read admission has room again"]);
    assert!(d.forges().full.is_empty());
    assert_write_census();
}

fn assert_write_census() {
    let mut out = Queue::uncapped();
    // A write occupies the same slab, but its content never enters the census.
    let (mut d, mut env) = setup(8, 2);
    keep(&mut d, 7, 43);
    let mut client_out = Queue::uncapped();
    let child_env = env.with(env.limits.client);
    skein_forge_client::step(
        &mut d.client,
        &child_env,
        skein_forge_client::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut client_out,
    );
    skein_forge_client::step(
        &mut d.client,
        &child_env,
        skein_forge_client::Event::Make { entry: entry(7, 43) },
        &mut client_out,
    );
    skein_forge_client::resume(&mut d.client, &child_env, &mut client_out);
    skein_forge_client::resume(&mut d.client, &child_env, &mut client_out);
    let clock = client_out
        .iter()
        .find_map(|r| match r {
            skein_forge_client::Request::Call { call, .. } => Some(*call),
            _ => None,
        })
        .expect("clock call queued");
    client_out.clear();
    skein_forge_client::step(
        &mut d.client,
        &child_env,
        skein_forge_client::Event::Answered {
            call: clock,
            cost: 1,
            free: 0,
            rate: None,
            result: Ok(skein_forge::api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env.now }),
        },
        &mut client_out,
    );
    d.client.reclaim();
    client_out.clear();
    domain::child(
        &mut d,
        &env,
        skein_forge_client::Event::Read {
            owner: Token::new(99),
            repository: REPO,
            read: Read::Pull { number: 12, ci: false },
        },
        &mut out,
    );
    skein_forge_client::resume(&mut d.client, &child_env, &mut client_out);
    update(&mut d, &env, &mut out);
    out.clear();
    at(&mut env, 30);
    domain::fire(&mut d, &env, &mut out);
    assert_eq!(
        facts(&out),
        [
            "the forge's read admission full for 30 s: 2 of 2, held by pull 12 of work/hello, a write of work/hello for task 43"
        ]
    );
}
