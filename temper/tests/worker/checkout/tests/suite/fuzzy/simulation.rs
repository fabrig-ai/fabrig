//! The worker's checkout child domain at random: many noisy worlds, each
//! settled with every client heard, every way a client ends reached among them.

use std::collections::BTreeSet;

use skein_lib::{Duration, Rng, Time};
use temper_worker_checkout_world::client::{Interrupt, Plan};
use temper_worker_checkout_world::{Settings, Span, World, noisy, submit_noisily};
use temper_worker_domain_checkout::git::Fault;
use temper_worker_domain_checkout::{Failure, Landing, Prepared};

const ITERATIONS: u32 = 100_000;

/// A thousand noisy worlds and forty calm save races: each settles,
/// with every client's every request ended once and nothing left
/// held or in flight (checked by `World::run`), and between them they reach
/// every way a prepare, a push and a save can end, and every way the cache
/// can find a workspace.
#[test]
fn random_worlds_settle_with_every_client_heard() {
    let mut prepares = BTreeSet::new();
    let mut lands = BTreeSet::new();
    let (mut cached, mut races) = ([0; 4], [0; 9]);
    let mut queued = 0;
    for seed in 0..1040 {
        let mut settings = if seed < 1000 {
            noisy(seed)
        } else {
            Settings {
                local: Span::millis(100, 100),
                remote: Span::millis(1, 1),
                think: Span::millis(0, 0),
                ..Settings::calm(seed)
            }
        };
        settings.checkout.preparations = u32::try_from(Rng::new(seed).between(1, u64::from(settings.checkout.workspaces))).expect("small");
        let mut world = World::new(settings);
        if seed < 1000 {
            submit_noisily(&mut world, &settings, seed);
        } else {
            // Sweep preparation and saves without faults or earlier pushes
            // consuming the interruption window. Store and checkout setup
            // precede a save; a release during its commit stops it before it
            // pushes, with the remaining repositories aborted too.
            let interrupt = Some((Duration::from_millis((seed - 999) * 100), Interrupt::Release));
            world.submit(Time::ZERO, Plan { pushes: 0, save: true, interrupt, ..Plan::simple(0) });
        }
        world.run(ITERATIONS);
        let (told, stats) = (world.told().0, world.stats());
        queued += stats.queued;
        assert!(stats.most_preparing_io <= settings.checkout.preparations, "seed {seed}: preparation io bounded");
        assert_eq!(stats.store_overlaps, 0, "seed {seed}: no two Store operations of one token in flight");
        for (count, more) in cached.iter_mut().zip([told.new, told.reused, told.rebuilt, told.evicted]) {
            *count += more;
        }
        let won = [
            stats.cancels_lost,
            stats.cancels_crossed,
            stats.op_timeouts,
            stats.stale,
            stats.exists,
            stats.op_broken,
            stats.ambiguous,
            stats.verified,
            stats.made_over,
        ];
        for (count, more) in races.iter_mut().zip(won) {
            *count += more;
        }
        for (_, client) in world.clients() {
            let kind = match client.prepared.as_ref().expect("every prepare ends") {
                Prepared::Ready { .. } => "ready".to_string(),
                Prepared::Refused { refusal } => format!("{refusal:?}"),
                Prepared::Failed { failure: Failure::Missing { missing, .. }, .. } => format!("missing {missing:?}"),
                Prepared::Failed { failure, .. } => format!("{failure:?}"),
                Prepared::Aborted => "aborted".to_string(),
            };
            prepares.insert(kind);
            for (saved, landings) in &client.landings {
                for landing in landings {
                    let kind = match landing {
                        Landing::DesignMerged { .. }
                        | Landing::DesignConflict { .. }
                        | Landing::DesignCode { .. }
                        | Landing::DesignMoving
                        | Landing::DesignUnchecked
                        | Landing::Conflicted { .. } => panic!("this fixture prepares one checkout branch"),
                        Landing::Landed { .. } => "landed",
                        Landing::Moved => "moved",
                        Landing::Refused | Landing::Explained { fault: Fault::Refused, .. } => "refused",
                        Landing::Failed | Landing::Explained { .. } => "failed",
                        Landing::Unchanged => "unchanged",
                        Landing::Aborted => "aborted",
                    };
                    lands.insert(format!("{} {kind}", if *saved { "save" } else { "push" }));
                }
            }
        }
    }
    assert!(queued > 0, "some prepares waited for a permit");
    let expected = [
        "ready",
        "Busy",
        "Full",
        "Invalid",
        "Transient",
        "Refused { repository: 0 }",
        "missing Repository",
        "missing Branch",
        "missing Commit",
        "aborted",
    ];
    for kind in expected {
        assert!(prepares.contains(kind), "some prepare ended {kind}: {prepares:?}");
    }
    for verb in ["push", "save"] {
        for kind in ["landed", "moved", "failed", "refused", "unchanged", "aborted"] {
            let kind = format!("{verb} {kind}");
            assert!(lands.contains(&kind), "some repository's {kind}: {lands:?}");
        }
    }
    assert!(cached.iter().all(|&count| count > 0), "workspaces new, reused, rebuilt and evicted: {cached:?}");
    assert!(
        races.iter().all(|&count| count > 0),
        "cancels lost and crossed, deadlines passed, stale handles, base branches created meanwhile, io failed, \
         io in doubt, pushes verified, workspaces made over something: {races:?}"
    );
}
