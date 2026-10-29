//! Random agent memory stays under scenario ceilings at every entry point.

use skein_lib::Duration;
use skein_world::domain::heap::Meter;
use smith_agent_world::memory::{LIMITS, churn};
use smith_domain::{Limits, run};

// Content capture setting from the session observation inventory:
// three parallel tools, sixteen completion blocks and four jobs.
const WIDE_CONTENT_EDGE_RECORDS: u32 = 105;

// Peak for 3_000 rounds of default-limit root-domain churn over seeds 0..12 (highest: seed 4).
// Measured 187_897 B on 2026-10-29; a quarter of headroom, rounded up.
const RANDOM_AGENT_CEILING: u64 = 240_000;

// Peak for 3_000 rounds of wider-limit root-domain churn over seeds 0..12 (highest: seed 11).
// Measured 254_648 B on 2026-10-29; a quarter of headroom, rounded up.
const WIDE_RANDOM_AGENT_CEILING: u64 = 320_000;

#[test]
fn a_domain_driven_at_random_stays_under_its_scenario_ceilings_at_every_entry_point() {
    let mut wider = Limits {
        configured_model_bytes: LIMITS.configured_model_bytes,
        accounts: LIMITS.accounts,
        endpoints: LIMITS.endpoints,
        decoded_call_bytes: LIMITS.decoded_call_bytes,
        skew: LIMITS.skew,
        run: run::Limits { runs: 3, conversations: 8, run_conversations: 4, calls: 8, ..LIMITS.run },
        session: smith_domain::session::Limits {
            sessions: 8,
            history_messages: 24,
            history_bytes: 1_048_576,
            completion_bytes: 4096,
            answer_bytes: 4096,
            completion_blocks: 16,
            failure_bytes: 512,
            delegated_result_bytes: 131_072,
            parallel_tools: 3,
            tools: smith_domain::tools::Limits { kits: 8, calls: 3, ..LIMITS.session.tools },
            ..LIMITS.session
        },
    };
    wider.session.facts = WIDE_CONTENT_EDGE_RECORDS;
    let mut seen = [0; 18];
    for seed in 0..12 {
        for (limits, ceiling) in [(LIMITS, RANDOM_AGENT_CEILING), (wider, WIDE_RANDOM_AGENT_CEILING)] {
            eprintln!("agent memory replay: seed={seed}, rounds=3000, limits={limits:?}");
            let counted = churn(&limits, seed, 3_000, ceiling);
            for (all, one) in seen.iter_mut().zip(counted) {
                *all += one;
            }
        }
    }
    eprintln!("agent memory actual boundary coverage: {seen:?}");
    assert!(seen[..7].iter().all(|count| *count > 10), "every kind of request was made: {seen:?}");
    assert!(seen[7..].iter().all(|count| *count > 0), "actual terminal and refusal classes occurred: {seen:?}");
}

#[test]
fn longest_child_usage_feedback_stays_within_its_measured_reservation() {
    let mut limits = LIMITS.run;
    limits.answer_bytes = 8192;
    let bound = smith_domain::feedback_bound(&limits).expect("checked receiving bound");
    let used = run::facts::Used { turns: u32::MAX, time: Duration::from_nanos(u64::MAX), spend: u64::MAX };
    let left = run::Budget { turns: u32::MAX, time: Duration::from_nanos(u64::MAX), spend: u64::MAX };
    let meter = Meter::new();
    let returned = run::Returned::Answered {
        text: vec![b'x'; 8192].into_boxed_slice(),
        cut: u64::MAX,
        stop: run::Stop::MaxTokens,
        used,
        left,
    };
    meter.start();
    let result = smith_domain::feedback(returned, bound).expect("largest complete result admitted");
    let measured = meter.end();
    meter.check(measured, bound, &limits);
    assert!(result.text.ends_with(b"Child usage: used 4294967295 turns and 18446744073 seconds; stopped for budget: no; ordinary left 4294967295 turns and 18446744073 seconds."));
}
