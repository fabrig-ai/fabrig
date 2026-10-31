//! Seeded short IO, completion delays and cancellation races in the real
//! shared shell, replayed using Skein's trace and referee machinery.
use smith_local_process_world::{
    Authentication, Files, World,
    referee::{CheckoutRead, Ending, review},
    terminal::Action,
};

#[test]
fn seeded_hosted_and_colocated_runs_settle_and_replay() {
    for seed in 100..116 {
        for placement in smith_local_process_world::fixture::PLACEMENTS {
            let mut previous = None;
            for _ in 0..2 {
                let files = Files::new();
                let cancelled = seed % 4 == 0;
                let changed = seed % 8 == 1;
                let authentication = match seed % 8 {
                    2 => Some(Authentication::SignIn),
                    3 => Some(Authentication::Refresh),
                    6 => Some(Authentication::Refused),
                    7 => Some(Authentication::Proactive),
                    _ => None,
                };
                let mut world = if let Some(mode) = authentication {
                    World::authenticated(seed, &files, placement, mode)
                } else if changed {
                    World::changed(seed, &files, placement)
                } else if cancelled {
                    World::interrupted(seed, &files, placement)
                } else {
                    World::new(seed, &files, placement, b"First answer")
                };
                world.scenario.faults = skein_sim::Faults {
                    latency: 300,
                    latency_max: skein_lib::Duration::from_millis(1),
                    short_recv: 500,
                    short_send: 500,
                    short_read: 500,
                    short_write: 500,
                    cancel_race: 500,
                    ..skein_sim::Faults::NONE
                };
                let mut rng = skein_lib::Rng::new(seed);
                let lines = rng.between(1, 3);
                let mut commands = Vec::new();
                for line in 0..lines {
                    commands.push(Action::Send(format!("person line {line}\n").into_bytes().into()));
                }
                commands.push(Action::Eof);
                if !changed && !cancelled && authentication != Some(Authentication::Refused) {
                    world.scenario.ending = Ending::Report(b"answer".to_vec());
                }
                world.scenario.commands = commands;
                let before = world.head();
                world.settle();
                let seen = world.seen();
                review(
                    &seen,
                    &world,
                    if authentication == Some(Authentication::Refused) {
                        Ending::Unavailable
                    } else if changed {
                        Ending::Change { before: before.expect("initial head") }
                    } else if cancelled {
                        Ending::Cancelled
                    } else {
                        Ending::Report(b"answer".to_vec())
                    },
                )
                .assert_passed(seed);
                let seen = (world.trace(), seen);
                if let Some(previous) = previous {
                    assert_eq!(seen, previous, "seed {seed}, {placement:?}");
                }
                previous = Some(seen);
            }
        }
    }
}
