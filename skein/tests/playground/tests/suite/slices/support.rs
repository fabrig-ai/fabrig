//! The same scripted world paced by slices or replayed whole (playground.md, 8).

use skein_lib::{Rng, Time};
use skein_playground::slices::{Reach, Slices};
use skein_playground_world::scripted::{self, Plan, Process};
use skein_sim::Entry;
use skein_sim_world::{Finished, World};
use skein_ui_web::{
    bridge::{self, Input, Speed, Stamp, ToPage},
    fetch,
};

pub const SEEDS: [u64; 5] = [1, 7, 19, 97, u64::MAX];
// Small budgets split a job across slices, including while paused.
pub const BUDGET: u32 = 3;
pub const ANSWER_STEPS: u32 = 3;

#[derive(Debug, Default)]
pub struct Said {
    pub reports: Vec<fetch::Report>,
    pub lines: Vec<bridge::Printed>,
    pub stamps: Vec<Stamp>,
    pub stops: u32,
}

impl Said {
    pub fn hear(&mut self, posts: Vec<ToPage>, seed: u64) {
        let closes = match posts.last() {
            Some(ToPage::Clock(_)) => true,
            Some(
                ToPage::Report(_) | ToPage::Printed(_) | ToPage::Stamp(_) | ToPage::Stopped | ToPage::Failed { .. },
            )
            | None => false,
        };
        assert!(closes, "seed {seed}: a clock closes the posts");
        for post in posts {
            match post {
                ToPage::Report(report) => self.reports.push(report),
                ToPage::Printed(line) => self.lines.push(line),
                ToPage::Stamp(stamp) => self.stamps.push(stamp),
                ToPage::Stopped => self.stops += 1,
                ToPage::Clock(clock) => {
                    assert_eq!(clock.memory, 0, "seed {seed}");
                    assert_eq!(clock.slice, skein_lib::Duration::ZERO, "seed {seed}");
                }
                ToPage::Failed { words } => unreachable!("seed {seed}: slice failed: {words:?}"),
            }
        }
    }
}

pub fn slices(seed: u64, plans: &[Plan], budget: u32, printing: bool) -> Slices<Process> {
    let (world, reach) = scripted::world(seed, plans, printing);
    Slices::new(world, reach, budget, Time::ZERO)
}

pub fn close() -> Input {
    Input::Abandon { exchange: 0 }
}

pub fn trace(slices: &mut Slices<Process>) -> Vec<Entry> {
    slices.world_mut().parts().0.trace().to_vec()
}

pub fn collect(world: &mut World<Process>, reach: Reach<Process>, said: &mut Said) {
    (reach.take)(world.process(reach.at), &mut said.reports);
    for line in world.printed() {
        let stream = match line.stream {
            skein_sim_world::Stream::Output => bridge::Stream::Output,
            skein_sim_world::Stream::Error => bridge::Stream::Error,
        };
        said.lines.push(bridge::Printed { process: line.process, stream, line: line.line });
    }
}

/// Runs directly to each stamp, then finishes without slices or a browser clock.
pub fn whole(seed: u64, plans: &[Plan], printing: bool, inputs: &[(Stamp, Input)]) -> (Finished<Process>, Said) {
    let (mut world, reach) = scripted::world(seed, plans, printing);
    let mut said = Said::default();
    for (stamp, input) in inputs {
        while world.steps() < stamp.step {
            if world.busy() {
                world.step();
                collect(&mut world, reach, &mut said);
            } else {
                let due =
                    world.next_due().unwrap_or_else(|| unreachable!("seed {seed}: a step before the stamp is due"));
                assert!(due <= stamp.instant, "seed {seed}: a due instant before the stamp");
                world.advance(due);
            }
        }
        if world.now() < stamp.instant {
            assert!(!world.busy(), "seed {seed}: an idle advance to the input");
            world.advance(stamp.instant);
        }
        assert_eq!(
            (world.steps(), world.now()),
            (stamp.step, stamp.instant),
            "seed {seed}: replay reaches the input's stamp"
        );
        (reach.enter)(world.process(reach.at), input.clone());
    }
    while !world.settled() {
        if world.busy() {
            world.step();
            collect(&mut world, reach, &mut said);
        } else {
            let due = world.next_due().unwrap_or_else(|| unreachable!("seed {seed}: the script has work due"));
            world.advance(due);
        }
    }
    (world.finish(), said)
}

pub fn draw(random: &mut Rng, bound: u64) -> u64 {
    random.next_u64() % bound
}

pub fn answer(exchange: u32) -> ToPage {
    ToPage::Report(fetch::Report::Answered {
        exchange,
        status: 200,
        body: Box::from(&b"answer"[..]),
        headers: Vec::new(),
    })
}

pub fn line(stream: bridge::Stream, bytes: &[u8]) -> ToPage {
    ToPage::Printed(bridge::Printed { process: Box::from(&b"script"[..]), stream, line: Box::from(bytes) })
}

pub fn speed(random: &mut Rng) -> Speed {
    // Includes pause, a fractional pace, 1×, and 10×.
    match draw(random, 4) {
        0 => Speed::PAUSED,
        1 => Speed(333),
        2 => Speed::ONE,
        _ => Speed(10_000),
    }
}
