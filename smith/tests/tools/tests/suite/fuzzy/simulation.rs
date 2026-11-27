//! The tools child domain at random: many noisy worlds, each settled with every
//! call answered, every kind of answer and every fault reached among them.

use std::collections::BTreeSet;
use std::convert::Infallible;

use skein_world::sweep::{Case, cases, gate};
use smith_tools_world::{Stats, kind, noisy_world};

const ITERATIONS: u32 = 100_000;

/// Hundreds of worlds with random limits, faults, latencies and scripts: each
/// settles, with every call answered once and nothing left alive or in flight
/// (checked by `World::run`), and between them they reach every way a call
/// can end.
#[test]
fn random_worlds_settle_with_every_call_answered() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut seen = BTreeSet::new();
    let mut faults = Stats::default();
    cases(skein_world::this_test!("random_worlds_settle_with_every_call_answered"), 0..300, |seed| {
        let mut world = noisy_world(seed);
        world.run(ITERATIONS);
        if collect_reach {
            let stats = world.stats();
            faults.faults += stats.faults;
            faults.timeouts += stats.timeouts;
            faults.late_effects += stats.late_effects;
            faults.cancels += stats.cancels;
            faults.late_cancels += stats.late_cancels;
            faults.stale_cancels += stats.stale_cancels;
        }
        for session in world.sessions() {
            if world.refusal(session).is_some() {
                if collect_reach {
                    seen.insert("refused");
                }
                continue;
            }
            let answers = world.answers(session);
            if collect_reach {
                seen.extend(answers.into_iter().map(kind));
            }
        }
        Ok(())
    })?;
    if !collect_reach {
        return Ok(());
    }
    let expected = [
        "ambiguous",
        "busy",
        "cancelled",
        "edited",
        "exited",
        "failed",
        "found",
        "linked",
        "listed",
        "no match",
        "not a directory",
        "not a file",
        "not found",
        "not granted",
        "not read",
        "outside",
        "protected",
        "read",
        "read only",
        "refused",
        "stale",
        "timed out",
        "too large",
        "unchanged",
        "written",
    ];
    assert_eq!(seen, expected.into_iter().collect());
    // Between them, the worlds faulted at every point.
    let Stats { faults, timeouts, late_effects, cancels, late_cancels, .. } = faults;
    for (count, what) in [
        (faults, "faults"),
        (timeouts, "timeouts"),
        (late_effects, "late effects"),
        (cancels, "cancels"),
        (late_cancels, "late cancels"),
    ] {
        assert!(count > 10, "only {count} {what}");
    }
    Ok(())
}
