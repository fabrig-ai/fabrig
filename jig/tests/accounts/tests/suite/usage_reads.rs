//! Usage reads are process clocks, not stored work (domain/accounts-lists.md, 1).
use jig_accounts_world::{ALICE, LIMITS, PROVIDER, World, account, issued, program, provider, providers, source};
use jig_domain_accounts::{self as accounts, AccountFact, Change, Event, FactValue, Kind, Request, Stored};
use jig_domain_link::{SourceName, UsageOutcome, UsageWindow};
use skein_lib::{Duration, Time, Token, Wall};

const ACCOUNT: u32 = 10;
const SOURCE: u32 = 11;
const SECOND: u32 = 12;
const QUIET: u64 = 30 * 60; // domain/accounts-lists.md, 1.
const RESET: u64 = 5 * 60;
fn wall(seconds: u64) -> Wall {
    Wall::from_nanos(Duration::from_secs(seconds).as_nanos())
}
fn window(state: u16, resets: Option<u64>) -> UsageWindow {
    UsageWindow {
        label: b"five_hour".as_slice().into(),
        window: accounts::FactWindow {
            length: Some(Duration::from_secs(18_000)),
            used: Some(1200),
            resets: resets.map(wall),
            state,
        },
    }
}
fn fact(window: &UsageWindow, origin: u16) -> AccountFact {
    AccountFact {
        kind: accounts::KIND_USAGE,
        label: window.label.clone(),
        value: FactValue::Window(window.window),
        seen: Wall::EPOCH,
        source: Some(SOURCE),
        origin,
    }
}
fn world(facts: &[AccountFact], kind: Kind) -> World {
    let mut p = provider(PROVIDER, true, true);
    p.offers.programs = Box::new([b"cli".as_slice().into()]);
    let mut rows = vec![account(ACCOUNT, PROVIDER, ALICE, b"subject")];
    if kind == Kind::Program {
        let mut reservation = issued(SOURCE, b"personal");
        if let Stored::Issued { holder, .. } = &mut reservation {
            *holder = ALICE;
        }
        rows.extend([reservation, program(ACCOUNT, SOURCE, 0, b"personal")]);
    } else {
        rows.push(source(ACCOUNT, SOURCE, 0, kind));
    }
    rows.push(Stored::Facts { account: ACCOUNT, facts: facts.into() });
    let mut world = World::with_start(LIMITS, providers(vec![p]), &rows);
    world.step(Event::Want);
    world
}
fn reads(out: &[Request]) -> Vec<(u64, SourceName)> {
    out.iter()
        .filter_map(|request| match request {
            Request::ReadUsage { read, source } => Some((*read, *source)),
            _ => None,
        })
        .collect()
}
fn due(world: &mut World, at: u64) -> u64 {
    let out = world.fire_at(at);
    let reads = reads(&out);
    assert_eq!(reads.len(), 1, "one read due at {at}");
    assert_eq!(reads[0].1.as_bytes(), b"personal");
    assert!(out.iter().all(|r| !matches!(r, Request::Save { .. })), "a clock stores nothing");
    reads[0].0
}
fn answer(world: &mut World, read: u64, outcome: UsageOutcome) -> Vec<Request> {
    world.step(Event::UsageRead { read, outcome })
}
fn change(change: Change) -> Event {
    Event::Change { request: Token::new(70), holder: ALICE, change }
}
fn advance(world: &mut World, seconds: u64) {
    world.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
    world.wall = wall(seconds);
}

#[test]
fn idle_program_accounts_wait_thirty_minutes_and_hold_one_read() {
    let mut world = world(&[], Kind::Program);
    assert_eq!(reads(&world.fire_at(QUIET - 1)), []);
    due(&mut world, QUIET);
    assert!(reads(&world.fire_at(QUIET * 3)).is_empty(), "an in-flight read has no second ask");
    for kind in [Kind::Key, Kind::SignIn, Kind::None] {
        let mut no_program = self::world(&[], kind);
        assert!(reads(&no_program.fire_at(QUIET * 3)).is_empty(), "{kind:?} has no program source");
    }
}

#[test]
fn every_report_including_an_equal_one_moves_the_quiet_clock() {
    for equal in [false, true] {
        let report = window(accounts::WINDOW_OPEN, None);
        let held = if equal { vec![fact(&report, accounts::ORIGIN_PROGRAM)] } else { vec![] };
        let mut world = world(&held, Kind::Program);
        advance(&mut world, 20 * 60);
        let out = world.step(Event::Usage { source: SOURCE, window: report });
        assert_eq!(out.is_empty(), equal, "an equal report writes nothing");
        assert_eq!(reads(&world.fire_at(50 * 60 - 1)), []);
        due(&mut world, 50 * 60);
    }
}

#[test]
fn every_read_end_rearms_from_its_end_and_failures_keep_facts() {
    let report = window(accounts::WINDOW_OPEN, None);
    let held = fact(&report, accounts::ORIGIN_USAGE_READ);
    for outcome in [
        UsageOutcome::Read(Box::new([])),
        UsageOutcome::NotSignedIn,
        UsageOutcome::Failed { reason: jig_domain_link::SourceFailure::Deadline, detail: Box::new([]) },
    ] {
        let mut world = world(std::slice::from_ref(&held), Kind::Program);
        let read = due(&mut world, QUIET);
        advance(&mut world, QUIET + 2);
        assert_eq!(answer(&mut world, read, outcome), []);
        assert_eq!(world.domain.account_facts(ACCOUNT), std::slice::from_ref(&held));
        assert_eq!(reads(&world.fire_at(QUIET * 2 + 1)), []);
        due(&mut world, QUIET * 2 + 2);
    }
}

#[test]
fn unusable_sources_hold_due_reads_and_usability_starts_a_quiet_clock() {
    for (return_at, signed_out) in [(10 * 60, false), (40 * 60, false), (10 * 60, true), (40 * 60, true)] {
        let mut world = world(&[], Kind::Program);
        let lift = if signed_out {
            world.step(Event::SignOut { request: Token::new(71), holder: ALICE, source: SOURCE });
            world.step(Event::ProgramTargets { source: SOURCE, hosts: Box::new([]) });
            Change::CheckProgram { source: SOURCE }
        } else {
            world.step(Event::Limited { account: SOURCE, reset: None });
            Change::Resume { source: SOURCE }
        };
        assert_eq!(reads(&world.fire_at(return_at)), []);
        let out = world.step(change(lift));
        if return_at < QUIET {
            assert_eq!(reads(&out), []);
            assert_eq!(reads(&world.fire_at(return_at + QUIET - 1)), []);
            due(&mut world, return_at + QUIET);
        } else {
            assert_eq!(reads(&out).len(), 1, "due work is not postponed by usability returning");
        }
    }
}

#[test]
fn limited_resets_are_read_once_including_restored_facts() {
    for restored in [false, true] {
        let report = window(accounts::WINDOW_LIMITED, Some(RESET));
        let held = if restored { vec![fact(&report, accounts::ORIGIN_PROGRAM)] } else { vec![] };
        let mut world = world(&held, Kind::Program);
        if !restored {
            world.step(Event::Usage { source: SOURCE, window: report.clone() });
        }
        assert_eq!(reads(&world.fire_at(RESET - 1)), []);
        let read = due(&mut world, RESET);
        answer(&mut world, read, UsageOutcome::NotSignedIn);
        assert!(reads(&world.fire_at(RESET)).is_empty(), "the same held reset is not read again");
        assert_eq!(reads(&world.step(Event::Usage { source: SOURCE, window: report })), []);
        assert_eq!(reads(&world.fire_at(RESET + QUIET - 1)), []);
        due(&mut world, RESET + QUIET);
    }
}

#[test]
fn answer_windows_are_reports_through_the_read_source() {
    let held = window(accounts::WINDOW_OPEN, Some(9_000));
    let mut world = world(&[fact(&held, accounts::ORIGIN_PROGRAM)], Kind::Program);
    let read = due(&mut world, QUIET);
    let report = UsageWindow {
        window: accounts::FactWindow { length: None, resets: None, used: Some(1400), state: accounts::WINDOW_WARNED },
        ..held.clone()
    };
    let out = answer(&mut world, read, UsageOutcome::Read(Box::new([report])));
    let expected = AccountFact {
        seen: wall(QUIET),
        origin: accounts::ORIGIN_USAGE_READ,
        value: FactValue::Window(accounts::FactWindow {
            used: Some(1400),
            state: accounts::WINDOW_WARNED,
            ..held.window
        }),
        ..fact(&held, accounts::ORIGIN_PROGRAM)
    };
    assert_eq!(world.domain.account_facts(ACCOUNT), std::slice::from_ref(&expected));
    assert_eq!(out, [Request::Save { record: Stored::Facts { account: ACCOUNT, facts: Box::new([expected]) } }]);
    assert!(answer(&mut world, read, UsageOutcome::Read(Box::new([held]))).is_empty(), "unheld answers are dropped");
}

#[test]
fn spent_reads_stop_the_account_until_restart() {
    let report = window(accounts::WINDOW_LIMITED, Some(QUIET + RESET));
    let mut world = world(&[], Kind::Program);
    let read = due(&mut world, QUIET);
    answer(&mut world, read, UsageOutcome::Spent);
    world.step(Event::Usage { source: SOURCE, window: report });
    assert!(world.domain.next_deadline().is_none(), "spent suppresses quiet and reset alarms");
    assert_eq!(reads(&world.fire_at(QUIET * 10)), []);
}

#[test]
fn removing_an_account_drops_its_read() {
    let mut world = world(&[], Kind::Program);
    let read = due(&mut world, QUIET);
    let removed = world.step(change(Change::Remove { account: ACCOUNT }));
    assert_eq!(
        removed.iter().filter(|request| matches!(request, Request::CancelUsage { .. })).cloned().collect::<Vec<_>>(),
        [Request::CancelUsage { read }],
        "the fleet is told to drop the removed source's read"
    );
    world.step(Event::ProgramTargets { source: SOURCE, hosts: Box::new([]) });
    assert!(world.domain.source(SOURCE).is_none());
    assert_eq!(answer(&mut world, read, UsageOutcome::Read(Box::new([window(accounts::WINDOW_OPEN, None)]))), []);
    assert_eq!(world.domain.account_facts(ACCOUNT), []);
    assert_eq!(reads(&world.fire_at(QUIET * 2)), []);
    assert!(world.domain.next_deadline().is_none());
    let added = world.step(change(Change::DeclareProgram {
        provider: PROVIDER,
        join: None,
        name: b"personal".as_slice().into(),
    }));
    assert!(
        added
            .iter()
            .any(|request| matches!(request, Request::Answer { answer: accounts::Answer::ProgramDeclared { .. }, .. })),
        "the removed name can be declared again"
    );
    assert_eq!(reads(&added), []);
    due(&mut world, QUIET * 3);
}

#[test]
fn the_first_usable_program_in_source_order_is_read() {
    let mut world = world(&[], Kind::Program);
    world.step(change(Change::DeclareProgram {
        provider: PROVIDER,
        join: Some(ACCOUNT),
        name: b"second".as_slice().into(),
    }));
    assert!(world.domain.source(SECOND).is_some());
    let read = due(&mut world, QUIET);
    answer(&mut world, read, UsageOutcome::NotSignedIn);
    world.step(Event::Limited { account: SOURCE, reset: None });
    let out = world.fire_at(QUIET * 2);
    assert_eq!(reads(&out).len(), 1);
    assert_eq!(reads(&out)[0].1.as_bytes(), b"second");
}
