//! Random settings and sessions: small limits, a provider that fails and
//! answers slowly or badly, tools that fail, openers that abandon, cancel
//! and close twice; and sessions opened at random, with budgets of every
//! size, some too large and some spent before they start.

use skein_fake_llm_domain::Config;
use skein_lib::{Duration, Rng, Time};
use skein_world::streams::{Stream, Streams};
use smith_domain_session::{Budget, Limits, Spec};
use smith_domain_tools as tools;

use crate::{BUDGET, Count, Settings, Span, World, spec};

/// Leave enough tool rounds within a session's turn budget to fill its job cells.
const NOISY_TURNS: u64 = 20;

/// Some worlds offer only shell and job tools, with room to poll a job again after its end.
fn job_heavy(seed: u64) -> bool {
    Streams::new(seed).rng(Stream::Workload, "tool-grants").chance(125)
}

/// Settings drawn from `seed`: small limits, faults, latencies that race the
/// deadlines, and an opener that nudges, dawdles and abandons.
#[must_use]
pub fn noisy(seed: u64) -> Settings {
    let mut rng = Rng::new(seed);
    let calm = Settings::calm(seed);
    let mut millis = |low: u64, high: u64| Duration::from_millis(rng.between(low, high));
    let call_head = millis(500, 5_000);
    let tool_deadline = millis(500, 5_000);
    let session_timeout = millis(2_000, 120_000);
    let latency_max = millis(10, 4_000);
    let think = millis(0, 5_000);
    let mut rng = Rng::new(seed.wrapping_add(1));
    let mut pick = |low: u64, high: u64| u32::try_from(rng.between(low, high)).expect("small numbers");
    let (sessions, parallel_tools) = (pick(1, 4), pick(1, 4));
    let mut settings = Settings {
        progress: smith_domain_session::Progress { head: call_head, idle: call_head },
        agent: Limits {
            sessions,
            history_messages: pick(4, 16),
            history_bytes: u64::from(pick(2_000, 8_000)),
            completion_bytes: 4096,
            answer_bytes: 4096,
            completion_blocks: 16,
            failure_bytes: 512,
            delegated_result_bytes: 16_384,
            budget: Budget { turns: pick(1, NOISY_TURNS), time: session_timeout, ..BUDGET },
            retries: pick(0, 4),
            backoff_base: Duration::from_millis(50),
            backoff_max: Duration::from_secs(2),
            parallel_tools: {
                let _ = pick(0, 24); // The draw keeps the seed's sequence.
                parallel_tools
            },
            tools: tools::Limits {
                kits: sessions,
                calls: parallel_tools + pick(0, 2),
                known_files: pick(1, 4),
                file_timeout: millis(200, 5_000),
                // The draw keeps the seed's sequence.
                tool_deadline: {
                    let _ = pick(0, 24);
                    tool_deadline
                },
                ..calm.agent.tools
            },
            ..calm.agent
        },
        provider: Config {
            calls: pick(1, 8),
            conversations: sessions,
            latency_min: Duration::from_millis(10),
            latency_max,
            overloaded: pick(0, 300),
            rate_limited: pick(0, 200),
            unavailable: pick(0, 100),
            too_long: pick(0, 20),
            unauthorized: pick(0, 10),
            refused: pick(0, 100),
            no_calls: pick(0, 100),
            answer_tokens: pick(1, 40),
            calls_per_answer: pick(1, 3),
            malformed: pick(0, 200),
            tool_rounds: pick(0, NOISY_TURNS),
            ..calm.provider
        },
        tool: Span::millis(10, 3_000),
        tool_errors: pick(0, 300),
        nudges: Count { min: 0, max: pick(0, 3) },
        think: Span { min: Duration::ZERO, max: think },
        abandon: pick(0, 400),
        abandon_after: Span::millis(0, 30_000),
        cancels_lost: pick(0, 600),
        double_close: pick(0, 500),
        serve: pick(0, 800),
        granule: Duration::from_millis((pick(0, 1) * pick(100, 1_000)).into()),
        // Leave delegated answers in flight long enough for withdrawal to race them.
        serving: Span::millis(10, 6_000),
        ..calm
    };
    // The 2000..8000 draw pressures retained payload; provider/result credit
    // reserves additional room before effects (smith/docs/design/domain/session.md, 3 and 5).
    // The receiving maxima and seeded draw order stay fixed.
    settings.agent.history_bytes = settings
        .agent
        .history_bytes
        .checked_add(crate::reservation_envelope(&settings.agent).expect("bounded scripted credits"))
        .expect("bounded pressure plus credit envelope");
    // Draw the job cells last, keeping every earlier draw in its seed's place.
    settings.agent.tools.jobs = pick(1, u64::from(calm.agent.tools.jobs));
    if job_heavy(seed) {
        settings.provider.calls_per_answer = 1;
    } else {
        // An entrance refusal can suppress a later write in a multi-call
        // answer. Keep the short, otherwise uninteresting one-call worlds
        // weighted toward refusals without adding tool rounds.
        if settings.provider.calls_per_answer == 3 && parallel_tools >= 3 {
            settings.provider.calls_per_answer = parallel_tools + 1;
        }
        // Keep a low-refusal tail for multi-turn reads, lists, and searches.
        settings.provider.malformed += if settings.provider.malformed < 35 { 200 } else { 780 };
    }
    settings
}

/// Up to eight sessions opened at random times in the first minute, each with
/// a budget of its own, small enough to run out in any dimension, now and then
/// empty in one, and now and then more than the limits allow; and room for
/// short answers only.
pub fn submit_noisily(world: &mut World, settings: &Settings, seed: u64) {
    let mut rng = Rng::new(seed.wrapping_add(2));
    let job_heavy = job_heavy(seed);
    let (cache_read, cache_write) = if job_heavy { (1_200, 512) } else { (600, 80) };
    let most = settings.agent.budget;
    for _ in 0..rng.between(1, 8) {
        let at = Time::from_nanos(rng.between(0, 60_000_000_000));
        let turns = if rng.chance(50) { most.turns + 1 } else { most.turns };
        let mut budget = Budget {
            turns: u32::try_from(rng.between(1, turns.into())).expect("a small number"),
            input: rng.between(5, 80),
            output: rng.between(5, 150),
            cache_read: rng.between(10, cache_read),
            cache_write: rng.between(5, cache_write),
            time: Duration::from_nanos(rng.between(1_000_000_000, most.time.as_nanos())),
        };
        if rng.chance(60) {
            match rng.below(6) {
                0 => budget.turns = 0,
                1 => budget.input = 0,
                2 => budget.output = 0,
                3 => budget.cache_read = 0,
                4 => budget.cache_write = 0,
                _ => budget.time = Duration::ZERO,
            }
        }
        // Answers short enough that some are cut, a tool call included.
        let max_tokens = u32::try_from(rng.between(4, 64)).expect("a small number");
        let mut opening = Spec {
            progress: settings.progress,
            budget,
            output: max_tokens,
            ..spec(if job_heavy { b"Run jobs for `README.md`." } else { b"Work on `src/lib.rs` and `README.md`." })
        };
        if job_heavy {
            opening.authority.grants = tools::Grants { inspect: false, modify: false, shell: true };
        }
        world.submit(at, opening);
    }
}
