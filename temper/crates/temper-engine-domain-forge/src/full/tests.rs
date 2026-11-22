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
        .insert(number, Outboxed { entry: entry(number, task), held: false, handoff: Handoff::Uncommitted })
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
    env.wall = skein_lib::Wall::from_nanos(
        crate::tests::env().wall.as_nanos().saturating_add(Duration::from_secs(seconds).as_nanos()),
    );
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
        expires: skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(Duration::from_secs(600).as_nanos())),
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
    d.room.asked.insert(waiter).unwrap();
    d.full.reserve(Reservation::Asked(waiter), 42);
    assert!(d.reserve_call(&env, Token::new(1), 43));
    d.projection_room.insert(2).unwrap();
    d.full.reserve(Reservation::Projection(2), 44);
    let words = outbox_words(&d, 10);
    assert!(words.ends_with(b"room asked for task 42, and 2 more"));
    for number in 1..=7 {
        d.entries.remove(&number);
    }
    assert_eq!(
        outbox_words(&d, 3).as_ref(),
        b"3 of 3, held by room asked for task 42, room for task 43's call, room for goal 44's projection"
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
