//! The agent's run child domain at random: many noisy worlds, each settled with
//! every start answered once, with broad ending coverage among them.

use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
};

use skein_lib::{Duration, Rng};
use skein_world::sweep::{Case, Sweep, cases, gate};
use smith_domain_run as run;
use smith_domain_run::{Answer, Exhausted, Fault, Invalid, Limits, Policy, Refusal, RunFailure};
use smith_faults::parks;
use smith_run_world::host;
use smith_run_world::partner::Script;
use smith_run_world::{Checkouts, Settings, Span, World, noisy};

use crate::support::{answers, settled};

fn time_pressure(seed: u64) -> Settings {
    let mut settings = Settings::calm(seed);
    settings.host.jobs = 1;
    settings.host.agents = 1000;
    settings.host.writable = 0;
    settings.host.reports = 1000;
    settings.host.verdicts = 0;
    settings.host.host_tools = 0;
    settings.host.turns_min = 20;
    settings.host.turns_max = 20;
    settings.host.time = smith_run_world::Span::millis(3000, 3000);
    settings.host.reserve = smith_domain_run::charter::Reserve { turns: 2, time: Duration::from_secs(1) };
    settings.partner.turn = smith_run_world::Span::millis(1800, 1800);
    settings.partner.asks = 1000;
    settings.partner.yields = 0;
    settings.partner.changes = 0;
    settings
}

/// Hundreds of worlds with random limits, scripts, faults and schedules: each
/// settles with its invariants holding (checked by `World::run`) and reaches
/// the listed endings. A dedicated focused story covers stale as a final answer.
#[test]
fn random_worlds_settle_with_every_start_answered_once() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut endings = BTreeSet::new();
    let (mut cancels, mut races, mut stale) = (0, 0, 0);
    let (mut waited, mut delivered, mut host_answered) = (0, 0, 0);
    let (mut busy, mut lost, mut answered) = (0, 0, 0);
    let mut sweep = park_sweep();
    let mut park_reach = BTreeMap::new();
    let mut check = |settings: Settings| {
        let seed = settings.seed;
        let world = settled(&settings);
        assert_eq!(
            smith_run_world::child_final_referee::judge(world.facts_seen()),
            skein_world::domain::Verdict::Passed,
            "child quota seed {seed}"
        );
        if collect_reach {
            let stats = world.stats();
            cancels += stats.cancels;
            races += stats.partner.races;
            stale += stats.partner.stale;
            waited += stats.partner.waiting;
            delivered += stats.partner.delivered;
            host_answered += stats.partner.host_answered;
            busy += stats.relay_busy;
            lost += stats.relay_lost;
            answered += stats.relay_answered;
        }
        for answer in answers(&world) {
            let kind = answer_kind(answer);
            if collect_reach {
                endings.insert(kind);
            }
        }
        Ok(())
    };
    cases(skein_world::this_test!("random_worlds_settle_with_every_start_answered_once", "noisy"), 0..300, |seed| {
        let settings = noisy(seed);
        check(settings)?;
        check_parks(&settings, &mut sweep, &mut park_reach);
        Ok(())
    })?;
    // A long child turn crosses the full run deadline.
    cases(
        skein_world::this_test!("random_worlds_settle_with_every_start_answered_once", "time pressure"),
        [34],
        |seed| check(time_pressure(seed)),
    )?;
    cases(skein_world::this_test!("random_worlds_settle_with_every_start_answered_once", "odd stops"), [3], |seed| {
        let mut settings = Settings::calm(seed);
        settings.run.nudges = 0;
        settings.partner.yields = 1000;
        settings.partner.odd_stops = 1000;
        check(settings)
    })?;
    cases(skein_world::this_test!("random_worlds_settle_with_every_start_answered_once", "high spend"), [5], |seed| {
        let mut settings = Settings::calm(seed);
        settings.host.tokens_min = 80_000;
        settings.host.tokens_max = 100_000;
        settings.host.turns_min = 100;
        settings.host.turns_max = 100;
        settings.partner.yields = 0;
        check(settings)
    })?;
    cases(
        skein_world::this_test!("random_worlds_settle_with_every_start_answered_once", "relay loss"),
        [303],
        |seed| {
            let mut settings = Settings::calm(seed);
            settings.host.jobs = 1;
            settings.host.host_tools = 1000;
            settings.host.relay_lost = 1000;
            settings.host.relay_busy = 0;
            settings.partner.host_calls = 1000;
            settings.partner.yields = 0;
            check(settings)
        },
    )?;
    sweep.check_fell().expect("each configured park fell");
    if !collect_reach {
        return Ok(());
    }
    eprintln!("park draws in noisy seeds: {park_reach:?}");
    assert_noisy_endings(&endings);
    // And the races: turns spent after a close, a close or nudge crossing a
    // conversation's own end, conversations out of time, cancels.
    assert!(cancels > 0 && races > 0 && stale > 0, "{cancels} {races} {stale}");
    assert!(
        waited > 0 && delivered > 0 && host_answered > 0 && busy > 0 && lost > 0 && answered > 0,
        "wait {waited}, delivered {delivered}, host answered {host_answered}, relays {busy}/{lost}/{answered}"
    );
    Ok(())
}

fn assert_noisy_endings(endings: &BTreeSet<&str>) {
    let expected = [
        "accepted",
        "busy",
        "cancelled",
        "conversation invalid",
        "fault",
        "invalid",
        "parked",
        "stopped",
        "time",
        "spend",
        "turns",
        "unfinished",
    ];
    assert!(expected.iter().all(|kind| endings.contains(kind)), "missing sampled ending: {endings:?}");
}

fn park_sweep() -> Sweep {
    let mut sweep = Sweep::new("random_worlds_settle_with_every_start_answered_once");
    // The noisy portion of random_worlds_settle_with_every_start_answered_once
    // already samples parked endings. Exercise the park scripts on the same
    // draws in a companion world, leaving the ordinary workload and its reach
    // checks intact. No JOB_RUNNING: the fake session runs no tools.
    for kind in [
        parks::WITHHELD_WAIT,
        parks::CUT_TURN,
        parks::HOST_CALL_ANSWERED,
        parks::HOST_CALL_UNANSWERED,
        parks::SUB_AGENT_KEPT,
    ] {
        sweep.configure(kind);
    }
    sweep
}

fn check_parks(settings: &Settings, sweep: &mut Sweep, reach: &mut BTreeMap<&'static str, u32>) {
    let mut world = World::new(*settings);
    world.draw_parks();
    world.run(1_000_000);
    let ledger = world.park_ledger();
    sweep.ran(ledger);
    if gate() {
        for park in ledger.entries() {
            *reach.entry(park.kind.name).or_insert(0_u32) += 1;
        }
    }
}

#[test]
fn random_reserves_keep_parallel_children_above_the_finishing_pool() -> Result<(), Case<Infallible>> {
    let mut children = 0;
    let mut main_in_reserve = 0;
    cases(
        skein_world::this_test!("random_reserves_keep_parallel_children_above_the_finishing_pool"),
        900..960,
        |seed| {
            let mut rng = Rng::new(seed);
            let calm = Settings::calm(seed);
            let settings = Settings {
                host: smith_run_world::host::Script {
                    jobs: 1,
                    agents: 1000,
                    writable: 0,
                    reports: 1000,
                    verdicts: 0,
                    turns_min: u32::try_from(rng.between(4, 10)).expect("small drawn turns"),
                    turns_max: 12,
                    reserve: smith_domain_run::charter::Reserve {
                        turns: u32::try_from(rng.between(1, 3)).expect("small drawn reserve"),
                        time: Duration::from_secs(rng.between(1, 30)),
                    },
                    ..calm.host
                },
                partner: smith_run_world::partner::Script {
                    asks: 800,
                    finishes: 200,
                    reports: 1000,
                    yields: 0,
                    shares: u32::try_from(rng.between(0, 1000)).expect("per-mille draw"),
                    parallel: 3,
                    ..calm.partner
                },
                ..calm
            };
            let world = settled(&settings);
            let observed = world.reserve_progress();
            children += observed.0;
            main_in_reserve += observed.1;
            Ok(())
        },
    )?;
    if gate() {
        assert!(children > 0 && main_in_reserve > 0, "the sweep admits children and reserved main turns");
    }
    Ok(())
}

#[test]
fn drawn_prices_reserve_main_and_children_without_spending_past_the_ceiling() -> Result<(), Case<Infallible>> {
    let mut charged = 0;
    cases(
        skein_world::this_test!("drawn_prices_reserve_main_and_children_without_spending_past_the_ceiling"),
        0..48,
        |seed| {
            let mut rng = Rng::new(seed ^ 0x0712);
            let mut settings = Settings::calm(seed);
            settings.host.jobs = 1;
            settings.host.writable = 0;
            settings.host.verdicts = 0;
            settings.host.reports = 1000;
            settings.host.agents = 1000;
            settings.host.host_tools = 0;
            settings.host.reserve = smith_domain_run::charter::Reserve { turns: 2, time: Duration::from_secs(1) };
            settings.partner.prices = Some(smith_domain_run::Prices {
                input: rng.between(1, 5),
                cached: rng.between(1, 9),
                cache_write: Some(rng.between(1, 7)),
                output: rng.between(1, 8),
                unit: u32::try_from(rng.between(1, 100)).expect("small denominator"),
            });
            settings.partner.asks = 600;
            settings.partner.finishes = 300;
            settings.partner.changes = 0;
            settings.partner.parallel = 3;
            settings.partner.yields = 0;
            let world = settled(&settings);
            for (_, _, charge) in world.charges_seen() {
                assert!(charge.spent.units <= charge.budget.spend, "price seed {seed}: {charge:?}");
                charged += 1;
            }
            Ok(())
        },
    )?;
    if gate() {
        assert!(charged > 48, "priced main and child callbacks enter the shared pool");
    }
    Ok(())
}

/// The common landing recipe: many finishing runs with slow, failing checks.
fn finishing(seed: u64) -> Settings {
    let calm = Settings::calm(seed);
    Settings {
        run: Limits {
            runs: 16,
            conversations: 16,
            calls: 16,
            check_timeout: Duration::from_millis(1_999),
            wrap_up: Duration::from_secs(60),
            ..calm.run
        },
        host: host::Script {
            jobs: 16,
            writable: 1000,
            changes: 1000,
            verdicts: 1000,
            tokens_min: 80_000,
            tokens_max: 100_000,
            ..calm.host
        },
        partner: Script { changes: 1000, finishes: 300, yields: 0, ..calm.partner },
        checkout: Checkouts { checks: 1000, check: Span::millis(2_000, 8_000), check_failures: 3, ..calm.checkout },
        ..calm
    }
}

fn wind_down_pressure(seed: u64, cancel: bool) -> Settings {
    let mut settings = Settings::calm(seed);
    settings.host.jobs = 1;
    settings.host.window = Duration::ZERO;
    settings.host.writable = 0;
    settings.host.verdicts = 0;
    settings.host.reports = 1000;
    settings.host.host_tools = 0;
    settings.host.time = Span::millis(6000, 6000);
    settings.host.reserve = run::charter::Reserve { turns: 2, time: Duration::from_secs(2) };
    settings.host.cancels = if cancel { 1000 } else { 0 };
    settings.host.cancel = Span::millis(5000, 5000);
    settings.partner.turn = Span::millis(3000, 3000);
    settings.partner.yields = 0;
    settings.partner.changes = 0;
    settings.partner.finishes = 0;
    settings
}

fn tally(cells: &mut BTreeMap<&'static str, u32>, observed: &BTreeMap<&'static str, u32>) {
    for (&cell, count) in observed {
        *cells.entry(cell).or_default() += count;
    }
}

/// Cancels at random moments, from the host (once, twice, late) and handed
/// to the run right before or after any of its events, find runs in every
/// state, and each answers once (checked by `World::run`).
#[test]
fn cancels_find_runs_in_every_state_over_many_seeds() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut cancels = BTreeMap::new();
    let mut check = |settings| {
        let world = settled(&settings);
        if collect_reach {
            tally(&mut cancels, world.cancel_cells());
        }
        Ok(())
    };
    cases(skein_world::this_test!("cancels_find_runs_in_every_state_over_many_seeds", "noisy"), 0..250, |seed| {
        check(noisy(seed))
    })?;
    // Cancelled twice while the run reads a slow checkout.
    cases(
        skein_world::this_test!("cancels_find_runs_in_every_state_over_many_seeds", "slow checkout"),
        [20],
        |seed| {
            let calm = Settings::calm(seed);
            let host = host::Script { cancels: 1000, cancel: Span::millis(0, 1_000), recancels: 1000, ..calm.host };
            let checkout = Checkouts { io: Span::millis(3_000, 5_000), ..calm.checkout };
            check(Settings { host, checkout, ..calm })
        },
    )?;
    // Cancelled as changes are checked and delivered, and as the LLM spends past
    // its budget.
    cases(skein_world::this_test!("cancels_find_runs_in_every_state_over_many_seeds", "landing"), 21..24, |seed| {
        let mut settings = finishing(seed);
        settings.host.cancels = 1000;
        settings.host.cancel = Span::millis(2_000, 30_000);
        settings.inject = 20;
        check(settings)
    })?;
    // Grant one new operation at a time, so cancellation can find its retained
    // phase rather than an unrelated completion or a denied charter.
    cases(
        skein_world::this_test!("cancels_find_runs_in_every_state_over_many_seeds", "wait and relays"),
        1000..1020,
        |seed| {
            let calm = Settings::calm(seed);
            let host = host::Script {
                cancels: 1000,
                cancel: Span::millis(100, 2_000),
                time: Span::millis(60_000, 60_000),
                ..calm.host
            };
            let partner = Script { waits: 1000, yields: 0, turn: Span::millis(10, 50), ..calm.partner };
            check(Settings { host, partner, ..calm })?;

            let host = host::Script {
                cancels: 1000,
                cancel: Span::millis(100, 6_000),
                host_tools: 1000,
                relay: Span::millis(8_000, 8_000),
                relay_busy: 500,
                relay_lost: 500,
                time: Span::millis(60_000, 60_000),
                ..calm.host
            };
            let partner = Script { host_calls: 1000, yields: 0, turn: Span::millis(10, 50), ..calm.partner };
            check(Settings { host, partner, ..calm })?;

            let host = host::Script {
                cancels: 1000,
                cancel: Span::millis(30, 600),
                host_tools: 1000,
                relay: Span::millis(1, 1),
                relay_busy: 1000,
                relay_lost: 0,
                time: Span::millis(60_000, 60_000),
                ..calm.host
            };
            check(Settings { host, partner, ..calm })
        },
    )?;
    cases(skein_world::this_test!("cancels_find_runs_in_every_state_over_many_seeds", "wind-down"), [901], |seed| {
        check(wind_down_pressure(seed, true))
    })?;
    if !collect_reach {
        return Ok(());
    }
    let cells = [
        "preparing",
        "stopping",
        "opening",
        "working",
        "waiting",
        "relay sending",
        "relay withdrawing",
        "relay backoff",
        "landing",
        "wind-down",
        "winding",
        "answered",
        "gone",
    ];
    for cell in cells {
        assert!(cancels.get(cell).is_some_and(|count| *count > 0), "no cancel found a run {cell}: {cancels:?}");
    }
    Ok(())
}

/// Deadlines short against a slow checkout and slow conversations fall due in
/// every state a run's deadline runs in.
#[test]
fn deadlines_find_runs_in_every_state_they_run_in_over_many_seeds() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut deadlines = BTreeMap::new();
    let mut check = |settings| {
        let world = settled(&settings);
        if collect_reach {
            tally(&mut deadlines, world.deadline_cells());
        }
        Ok(())
    };
    cases(
        skein_world::this_test!("deadlines_find_runs_in_every_state_they_run_in_over_many_seeds", "slow checkout"),
        0..50,
        |seed| {
            let calm = Settings::calm(seed);
            let host = host::Script { time: Span::millis(1_000, 6_000), writable: 0, verdicts: 1000, ..calm.host };
            let checkout = Checkouts { io: Span::millis(0, 2_000), ..calm.checkout };
            let partner = Script { turn: Span::millis(100, 2_000), ..calm.partner };
            let settings = Settings { host, checkout, partner, hop: Span::millis(0, 1_500), ..calm };
            check(settings)
        },
    )?;
    // And as changes are checked and delivered, and as the LLM spends past its
    // budget.
    cases(
        skein_world::this_test!("deadlines_find_runs_in_every_state_they_run_in_over_many_seeds", "landing"),
        50..56,
        |seed| {
            let mut settings = finishing(seed);
            settings.host.time = Span::millis(2_000, 15_000);
            // Keep submitted delivery pending until the run's deadline.
            settings.host.delivery = Span::millis(20_000, 20_000);
            settings.partner.turn = Span::millis(100, 2_000);
            settings.hop = Span::millis(0, 1_500);
            check(settings)
        },
    )?;
    cases(
        skein_world::this_test!("deadlines_find_runs_in_every_state_they_run_in_over_many_seeds", "wait and relays"),
        2000..2020,
        |seed| {
            let calm = Settings::calm(seed);
            let partner = Script { waits: 1000, yields: 0, turn: Span::millis(10, 50), ..calm.partner };
            let host = host::Script { time: Span::millis(100, 500), ..calm.host };
            check(Settings { host, partner, ..calm })?;

            let partner = Script { host_calls: 1000, yields: 0, turn: Span::millis(10, 50), ..calm.partner };
            let host = host::Script {
                host_tools: 1000,
                relay: Span::millis(8_000, 8_000),
                time: Span::millis(100, 6_000),
                ..calm.host
            };
            check(Settings { host, partner, ..calm })?;
            let host = host::Script {
                host_tools: 1000,
                relay: Span::millis(1, 1),
                relay_busy: 1000,
                time: Span::millis(30, 300),
                ..calm.host
            };
            check(Settings { host, partner, ..calm })
        },
    )?;
    cases(
        skein_world::this_test!("deadlines_find_runs_in_every_state_they_run_in_over_many_seeds", "wind-down"),
        [902],
        |seed| check(wind_down_pressure(seed, false)),
    )?;
    if !collect_reach {
        return Ok(());
    }
    for cell in [
        "preparing",
        "opening",
        "working",
        "waiting",
        "relay sending",
        "relay withdrawing",
        "relay backoff",
        "landing",
        "wind-down",
    ] {
        assert!(deadlines.get(cell).is_some_and(|count| *count > 0), "no deadline found a run {cell}: {deadlines:?}");
    }
    Ok(())
}

fn answer_kind(answer: &Answer) -> &'static str {
    match answer {
        Answer::Parked { .. } => "parked",
        Answer::Accepted { .. } => "accepted",
        Answer::Refused(Refusal::Busy) => "busy",
        Answer::Refused(Refusal::Invalid(Invalid::Conversation)) => "conversation invalid",
        Answer::Refused(Refusal::Invalid(_)) => "invalid",
        Answer::Failed { failure, .. } => match failure {
            RunFailure::Model(
                Fault::Completion { .. }
                | Fault::Provider
                | Fault::Unauthorized { .. }
                | Fault::Program { .. }
                | Fault::Untooled { .. }
                | Fault::Limited { .. }
                | Fault::Refused { .. },
            ) => "fault",
            RunFailure::Model(Fault::Truncated | Fault::Declined | Fault::Malformed) => "stopped",
            RunFailure::Budget { exhausted: Exhausted::Turns, .. } => "turns",
            RunFailure::Budget { exhausted: Exhausted::Time, .. } => "time",
            RunFailure::Budget { exhausted: Exhausted::Spend, .. } => "spend",
            RunFailure::Budget { exhausted: Exhausted::Overflow(_), .. } => {
                panic!("bounded random partner usage and prices remain representable")
            }
            RunFailure::Budget { exhausted: Exhausted::Tokens(_), .. } => {
                panic!("source random partner has no receiving refusal")
            }
            RunFailure::Policy(Policy::Unfinished { .. }) => "unfinished",
            RunFailure::Cancelled => "cancelled",
            RunFailure::Stale => "stale",
            RunFailure::Transcript(_) => panic!("source random partner does not restore history"),
        },
    }
}
