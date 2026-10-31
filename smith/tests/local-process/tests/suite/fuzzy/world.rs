//! Seeded short IO, completion delays and cancellation races in the real
//! shared shell, replayed using Skein's trace and referee machinery.
use skein_world::domain::assert_replays;
use smith_local_process_world::{
    Authentication, Files, World,
    fixture::PLACEMENTS,
    referee::{CheckoutRead, Ending},
    terminal::Action,
};

#[derive(Clone, Copy)]
enum Story {
    Report,
    Changed,
    Cancelled,
    Authenticated(Authentication),
}

#[test]
fn seeded_hosted_and_colocated_runs_settle_and_replay() {
    for (seed, other, story) in [
        (100, 108, Story::Cancelled),
        (101, 109, Story::Report),
        (102, 110, Story::Authenticated(Authentication::Refused)),
        (103, 111, Story::Authenticated(Authentication::Proactive)),
        (104, 112, Story::Cancelled),
        (105, 113, Story::Changed),
        (106, 114, Story::Authenticated(Authentication::SignIn)),
        (107, 115, Story::Authenticated(Authentication::Refresh)),
    ] {
        for placement in PLACEMENTS {
            assert_replays(seed, other, |seed| {
                let files = Files::new();
                let mut world = match story {
                    Story::Authenticated(mode) => World::authenticated(seed, &files, placement, mode),
                    Story::Changed => World::changed(seed, &files, placement),
                    Story::Cancelled => World::interrupted(seed, &files, placement),
                    Story::Report => World::new(seed, &files, placement, b"First answer"),
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
                let ending = match story {
                    Story::Authenticated(Authentication::Refused) => Ending::Unavailable,
                    Story::Changed => Ending::Change { before: world.head().expect("initial head") },
                    Story::Cancelled => Ending::Cancelled,
                    Story::Report | Story::Authenticated(_) => Ending::Report(b"answer".to_vec()),
                };
                world.scenario.ending = ending.clone();
                world.scenario.commands = commands;
                world.settle();
                world.assert_ended(ending);
                (world.trace(), world.seen())
            });
        }
    }
}
