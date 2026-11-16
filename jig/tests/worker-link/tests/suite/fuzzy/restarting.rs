//! Seeded words, admissions and losses checked against the channel's last word.
use super::super::restarting::{limits, passed, sources, word};
use jig_protocol_worker_link::Event;
use jig_worker_link_world::{World, fixtures};
use skein_lib::{Duration, Rng, Time};

#[derive(Default)]
struct Clock {
    end: Option<Time>,
    wait: Option<Duration>,
    passed: usize,
    losses: usize,
}
impl Clock {
    fn lost(&mut self, now: Time) {
        self.expire(now);
        if self.end.is_none() {
            self.end = Some(now.saturating_add(self.wait.unwrap_or(Duration::ZERO).max(Duration::from_secs(300))));
        }
        self.wait = None;
    }
    fn expire(&mut self, now: Time) {
        if self.end.is_some_and(|at| at <= now) {
            self.passed += 1;
            self.end = None;
        }
    }
    fn check(&mut self, world: &World, seed: u64, step: u32) {
        for loss in world.losses().iter().skip(self.losses) {
            self.lost(*loss);
        }
        self.losses = world.losses().len();
        self.expire(world.now());
        assert_eq!(passed(world), self.passed, "seed {seed}, step {step}, now {:?}, end {:?}", world.now(), self.end);
    }
}

#[test]
fn fuzzy_restart_words_bound_the_clock_from_the_loss_that_armed_it() {
    let mut coverage = [0; 6];
    for seed in 0..32 {
        let mut bounds = limits();
        bounds.silence = Duration::from_secs(100_000);
        bounds.ping = Duration::from_secs(30_000);
        let mut world = World::new(&bounds, seed);
        world.script_root();
        world.engine(fixtures::engine_accepting());
        world.run();
        let mut clock = Clock::default();
        let mut rng = Rng::new(seed);
        for step in 0..64 {
            if world.has_channel() {
                let action = rng.below(6);
                match action {
                    0 | 1 => {
                        let wait = [0, 60_000, 1_200_000, 600_000][usize::try_from(rng.below(4)).expect("small index")];
                        world.play(word(wait));
                        clock.wait = Some(Duration::from_millis(wait));
                        clock.end = None;
                        coverage[0] += 1;
                    }
                    2 => {
                        world.play(sources());
                        clock.end = None;
                        coverage[1] += 1;
                    }
                    3 => {
                        world.lose_channel();
                        coverage[2] += 1;
                    }
                    4 => {
                        world.play(
                            skein_channel::control_frame(
                                &skein_channel::Control::Refuse { reason: 260, text: Box::new([]) },
                                &fixtures::channel_limits(),
                            )
                            .expect("hello refused"),
                        );
                        coverage[3] += 1;
                    }
                    _ => world.advance(Duration::from_secs(rng.between(1, 400))),
                }
            } else if rng.below(3) == 0 {
                world.advance(Duration::from_secs(rng.between(1, 400)));
            } else {
                let accept = rng.below(2) == 0;
                world.engine(if accept { fixtures::engine_accepting() } else { fixtures::engine_refusing(258) });
                if world.dial_pending() {
                    world.connect_pending();
                } else {
                    world.advance(Duration::from_secs(75));
                }
                clock.wait = None;
                coverage[if accept { 4 } else { 5 }] += 1;
            }
            clock.check(&world, seed, step);
        }
        if world.has_channel() {
            world.lose_channel();
        }
        world.advance(Duration::from_secs(1201));
        clock.check(&world, seed, 64);
        world.down(Event::Stop);
    }
    assert!(coverage.iter().all(|count| *count > 0), "every action occurred: {coverage:?}");
}
