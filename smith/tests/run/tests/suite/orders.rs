//! Input and deferred-work orders under output load (`smith/docs/design/domain/run.md`, 9.1 and 13).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use skein_lib::{Duration, Time, Token};
use smith_domain_run::charter::Reserve;
use smith_domain_run::{Answer, Exhausted, Failure};
use smith_run_world::{
    Settings, Span, World,
    order_log::{Input, Record, Seen},
};

const ITERATIONS: u32 = 1_000_000;
// Four simultaneous runs exceed the world's four output cells: a check terminal
// makes feedback and closure requests before the next terminal can enter.
const CHECK_RUNS: u32 = 4;
// Sixteen starts overlap wind-down; six parallel asks can keep several children live.
const WIND_RUNS: u32 = 16;
const PARALLEL_CHILDREN: u32 = 6;
// Main, six parallel children and a spare conversation slot per activation.
const CONVERSATIONS_PER_RUN: u32 = PARALLEL_CHILDREN + 2;
const CONVERSATIONS: u32 = WIND_RUNS * CONVERSATIONS_PER_RUN;
const ORDINARY_TURNS: u32 = 5;
const FINAL_TURNS: u32 = 3;
const CHILD_SETTLEMENT: Duration = Duration::from_millis(500);

fn check_settings() -> Settings {
    let mut settings = Settings::calm(18);
    settings.host.jobs = CHECK_RUNS;
    settings.host.window = Duration::ZERO;
    settings.host.writable = 1000;
    settings.host.changes = 1000;
    settings.host.verdicts = 1000;
    settings.run.check_timeout = Duration::from_millis(3_999);
    settings.host.reserve = Reserve { turns: 1, time: Duration::from_millis(3_999) };
    settings.host.time = Span::millis(4_000, 4_000);
    settings.partner.changes = 1000;
    settings.partner.finishes = 1000;
    settings.partner.yields = 0;
    settings.partner.turn = Span::millis(100, 100);
    settings.checkout.checks = 1000;
    settings.checkout.check = Span::millis(8_000, 8_000);
    settings.checkout.check_failures = 0;
    settings.checkout.io = Span::millis(1, 1);
    settings.hop = Span::millis(0, 0);
    settings.network = Span::millis(0, 0);
    settings.races = 0;
    settings
}

fn wind_settings() -> Settings {
    let mut settings = Settings::calm(34);
    settings.host.jobs = WIND_RUNS;
    settings.host.window = Duration::from_millis(4_000);
    settings.run.runs = WIND_RUNS;
    settings.run.conversations = CONVERSATIONS;
    settings.partner.conversations = CONVERSATIONS;
    settings.host.writable = 0;
    settings.host.agents = 1000;
    settings.host.host_tools = 0;
    settings.host.turns_min = 100;
    settings.host.turns_max = 100;
    settings.host.time = Span::millis(3_000, 3_000);
    settings.host.reserve = Reserve { turns: 2, time: Duration::from_secs(1) };
    settings.partner.asks = 1000;
    settings.partner.finishes = 0;
    settings.partner.yields = 0;
    settings.partner.parallel = 3;
    settings.partner.turn = Span::millis(1_800, 1_800);
    settings.partner.settle = Span { min: CHILD_SETTLEMENT, max: CHILD_SETTLEMENT };
    settings.host.cancels = 500;
    settings.host.cancel = Span::millis(2_100, 2_100);
    settings.hop = Span::millis(0, 0);
    settings.network = Span::millis(0, 0);
    settings.checkout.io = Span::millis(0, 0);
    settings
}

fn child_settings() -> Settings {
    let mut settings = wind_settings();
    settings.seed = 1;
    // Five ordinary turns and three final turns make admission spend the
    // ordinary pool while siblings still have completions in flight.
    settings.run.run_conversations = CONVERSATIONS_PER_RUN;
    settings.host.turns_min = ORDINARY_TURNS + FINAL_TURNS;
    settings.host.turns_max = ORDINARY_TURNS + FINAL_TURNS;
    settings.host.reserve.turns = FINAL_TURNS;
    settings.host.time = Span::millis(10_000, 10_000);
    settings.host.reserve.time = Duration::from_secs(5);
    settings.partner.turn = Span::millis(100, 1000);
    settings.partner.parallel = PARALLEL_CHILDREN;
    settings.host.cancel = Span::millis(1_000, 1_000);
    settings
}

fn recorded(settings: &Settings, open: bool) -> World {
    let mut world = World::new(*settings);
    if open {
        world.open_runs();
    }
    world.record_orders();
    world.run(ITERATIONS);
    world
}

/// Compare boundary records without consulting the run's readiness state.
fn assert_inputs_drained(records: &[Record]) {
    let mut queued = VecDeque::new();
    for record in records {
        match &record.seen {
            Seen::Queued { inputs, .. } => queued = inputs.iter().copied().collect(),
            Seen::Taken(input) => assert_eq!(queued.pop_front(), Some(*input)),
            Seen::Alarm | Seen::Resume => assert!(queued.is_empty(), "input overtaken: {record:?}, {queued:?}"),
            Seen::Pending { .. } | Seen::Close { .. } | Seen::Yield { .. } | Seen::Final { .. } | Seen::Reclaim => {}
        }
    }
}

/// A resume consumes a generation present before the pass, once, after alarms.
fn assert_deferred_generations(records: &[Record]) -> BTreeSet<u64> {
    let mut ready_at_entry = false;
    let mut reclaimed = 0;
    let mut resumed = BTreeSet::new();
    for record in records {
        match &record.seen {
            Seen::Queued { ready, .. } => {
                assert_eq!(record.pass, reclaimed + 1, "each pass crosses reclaim");
                ready_at_entry = *ready;
            }
            Seen::Resume => {
                assert!(ready_at_entry, "work made in this pass must wait: {record:?}");
                assert!(resumed.insert(record.pass), "at most one resume per pass");
            }
            Seen::Alarm => assert!(!resumed.contains(&record.pass), "alarms precede resume"),
            Seen::Reclaim => reclaimed = record.pass,
            Seen::Taken(_) | Seen::Pending { .. } | Seen::Close { .. } | Seen::Yield { .. } | Seen::Final { .. } => {}
        }
    }
    resumed
}

/// Order: `run_inputs_drained` before alarms (phase C).
#[test]
fn run_inputs_drained_before_alarms_at_the_check_deadline() {
    let world = recorded(&check_settings(), false);
    let deadline = Time::ZERO.saturating_add(Duration::from_secs(4));
    assert_inputs_drained(world.orders());
    assert!(
        world.orders().iter().any(|record| {
            record.at == deadline
                && matches!(&record.seen, Seen::Queued { inputs, .. }
                if inputs.iter().filter(|input| matches!(input, Input::Checked(_))).count() == CHECK_RUNS as usize)
        }),
        "all check terminals queue together at the run deadline"
    );
    assert_eq!(world.stats().checks, CHECK_RUNS);
    assert_eq!(world.stats().partner.checks_failed, CHECK_RUNS, "every terminal supplies feedback");
    assert_eq!(world.stats().aborts, 0, "no alarm decides a check is still live");
    assert_eq!(world.stats().deliveries, 0);
    assert!(world.answers().all(|answer| matches!(
        answer,
        Some(Answer::Failed { failure: Failure::Budget { exhausted: Exhausted::Time, .. }, .. })
    )));
}

/// Order: `wind_down_promoted_after_reclaim` for successive child closes (phase C).
#[test]
fn wind_down_promoted_after_reclaim_closes_children_once() {
    let world = recorded(&child_settings(), true);
    let resumed = assert_deferred_generations(world.orders());
    let mut closed = BTreeSet::new();
    let mut pending: BTreeMap<Token, (Token, Time)> = BTreeMap::new();
    let mut deferred: BTreeMap<Token, Vec<u64>> = BTreeMap::new();
    let mut cancelled_while_settling = false;
    let mut started_while_settling = false;
    let mut delayed_terminal = false;
    let mut final_runs = BTreeSet::new();
    let mut peak_pending = 0;
    for record in world.orders() {
        match record.seen {
            Seen::Close { run, conversation, main: false } => {
                assert!(closed.insert(conversation), "each child closes once");
                pending.insert(conversation, (run, record.at));
                peak_pending = peak_pending.max(pending.len());
                if resumed.contains(&record.pass) {
                    let passes = deferred.entry(run).or_default();
                    assert!(
                        passes.last().is_none_or(|previous| *previous < record.pass),
                        "one deferred close per pass"
                    );
                    passes.push(record.pass);
                }
            }
            Seen::Taken(Input::Ended(conversation)) => {
                if let Some((_, at)) = pending.remove(&conversation) {
                    delayed_terminal |= record.at.saturating_since(at) >= CHILD_SETTLEMENT;
                }
            }
            Seen::Taken(Input::Cancel(run)) => {
                cancelled_while_settling |= pending.values().any(|(owner, _)| *owner == run);
            }
            Seen::Taken(Input::Start(_)) => started_while_settling |= !pending.is_empty(),
            Seen::Final { run, live_children } => {
                assert_eq!(live_children, 0, "main finishes after every live child terminal");
                assert!(pending.values().all(|(owner, _)| *owner != run));
                final_runs.insert(run);
            }
            Seen::Queued { .. }
            | Seen::Taken(Input::Checked(_) | Input::Other)
            | Seen::Pending { .. }
            | Seen::Alarm
            | Seen::Resume
            | Seen::Close { main: true, .. }
            | Seen::Yield { .. }
            | Seen::Reclaim => {}
        }
    }
    assert!(deferred.values().any(|passes| passes.len() >= 2), "successive child closes reach real resume");
    assert!(delayed_terminal && cancelled_while_settling && started_while_settling);
    assert!(pending.is_empty());
    assert!(
        deferred.iter().any(|(run, passes)| passes.len() >= 2 && final_runs.contains(run)),
        "main with successive deferred child closes reaches its final allowance"
    );
    assert!(peak_pending >= 2, "several closed children await settlement together");
    assert_eq!(world.answers().count(), WIND_RUNS as usize);
}

/// Order: `wind_down_promoted_after_reclaim` for main's finishing yield (phase C).
#[test]
fn wind_down_promoted_after_reclaim_defers_main_yield() {
    let mut settings = wind_settings();
    settings.partner.asks = 0;
    settings.host.cancels = 0;
    let world = recorded(&settings, true);
    let resumed = assert_deferred_generations(world.orders());
    let mut yielded = BTreeSet::new();
    for record in world.orders() {
        if let Seen::Yield { run } = record.seen {
            assert!(resumed.contains(&record.pass), "main yields from deferred resume");
            assert!(yielded.insert(run), "main's finishing yield occurs once");
        }
    }
    assert!(!yielded.is_empty(), "an in-flight main completion reaches deferred yield");
}

/// Order: `run_inputs_drained` before resume (phase C).
#[test]
fn run_inputs_drained_before_deferred_wind_down() {
    let mut settings = child_settings();
    settings.host.window = Duration::ZERO;
    settings.partner.turn = Span::millis(100, 100);
    let world = recorded(&settings, true);
    assert_inputs_drained(world.orders());
    let resumed = assert_deferred_generations(world.orders());
    assert!(
        world.orders().iter().any(|record| matches!(&record.seen,
            Seen::Queued { inputs, ready: true }
                if inputs.iter().filter(|input| matches!(input, Input::Cancel(_))).count() >= 2
        )),
        "multiple output-producing cancels queue while wind-down is ready"
    );
    assert!(
        world.orders().iter().any(|record| matches!(&record.seen,
            Seen::Queued { inputs, ready: true }
                if inputs.iter().filter(|input| matches!(input, Input::Ended(_))).count() >= 2
        )),
        "child terminals also queue while wind-down is ready"
    );
    assert!(!resumed.is_empty());
    assert!(
        world.orders().iter().any(|record| matches!(record.seen, Seen::Close { main: false, .. } | Seen::Yield { .. })
            && resumed.contains(&record.pass)),
        "the ready work emits a deferred effect"
    );
}
