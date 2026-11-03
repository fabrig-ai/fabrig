//! The worker's loop over a world, in slices paced by the browser (playground.md, 3).

use skein_lib::{Duration, Time};
use skein_shell::Host;
use skein_sim_world::{Machine, NoMachine, World};
use skein_ui_web::{bridge, fetch};

/// How the loop reaches the visitor among the world's processes (playground.md, 3).
#[derive(Debug)]
pub struct Reach<P> {
    /// A process started by the world before any hosted child.
    pub at: usize,
    /// Gives the visitor an input between steps (world.md, 5).
    pub enter: fn(&mut P, bridge::Input),
    /// Takes the visitor's reports in order.
    pub take: fn(&mut P, &mut Vec<fetch::Report>),
}

impl<P> Clone for Reach<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P> Copy for Reach<P> {}

/// When the loop wants its next slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// Busy: at once, whatever the speed.
    Now,
    /// Idle: the browser instant at which the next due instant comes at this speed.
    At(Time),
    /// Paused, settled, or nothing due: at the next message.
    Message,
}

/// The worker's loop over a world whose processes include the visitor.
#[derive(Debug)]
pub struct Slices<P, M = NoMachine> {
    world: World<P, M>,
    reach: Reach<P>,
    inputs: Vec<bridge::Input>,
    reports: Vec<fetch::Report>,
    budget: u32,
    browser: Time,
    speed: bridge::Speed,
    stopped: bool,
}

impl<P: Host, M: Machine> Slices<P, M> {
    /// At 1×, its browser time counted from `browser`; `budget` steps a slice, at least 1.
    #[must_use]
    pub fn new(world: World<P, M>, reach: Reach<P>, budget: u32, browser: Time) -> Self {
        assert!(budget > 0, "a slice has at least one step");
        assert!(reach.at < world.processes().len(), "the visitor is an admitted process");
        Self {
            world,
            reach,
            inputs: Vec::new(),
            reports: Vec::new(),
            budget,
            browser,
            speed: bridge::Speed::ONE,
            stopped: false,
        }
    }

    /// Queued; it enters at the next slice.
    pub fn input(&mut self, input: bridge::Input) {
        self.inputs.push(input);
    }

    /// Changes the pace, counting browser time anew at `browser`.
    pub fn pace(&mut self, speed: bridge::Speed, browser: Time) {
        self.speed = speed;
        self.browser = browser;
    }

    /// One slice (playground.md, 3 and 5): posts appended in order, its clock last.
    pub fn slice(&mut self, browser: Time, posts: &mut Vec<bridge::ToPage>) -> Wake {
        let elapsed = browser.saturating_since(self.browser);
        self.browser = browser;
        for input in self.inputs.drain(..) {
            (self.reach.enter)(self.world.process(self.reach.at), input);
            posts.push(bridge::ToPage::Stamp(bridge::Stamp { step: self.world.steps(), instant: self.world.now() }));
        }

        let mut left = self.budget;
        self.step_busy(&mut left);
        if !self.world.busy() && !self.world.settled() && self.speed != bridge::Speed::PAUSED {
            let target = self.world.now().saturating_add(scaled(elapsed, self.speed.0, 1000, false));
            while left > 0 {
                match self.world.next_due() {
                    Some(due) if due <= target => {
                        self.world.advance(due);
                        self.step_busy(&mut left);
                        if self.world.settled() {
                            break;
                        }
                    }
                    Some(_) | None => {
                        self.world.advance(target);
                        break;
                    }
                }
            }
        }

        (self.reach.take)(self.world.process(self.reach.at), &mut self.reports);
        posts.extend(self.reports.drain(..).map(bridge::ToPage::Report));
        for line in self.world.printed() {
            posts.push(bridge::ToPage::Printed(printed(line)));
        }
        let settled = self.world.settled();
        if settled && !self.stopped {
            posts.push(bridge::ToPage::Stopped);
            self.stopped = true;
        }
        let busy = self.world.busy();
        let next_due = self.world.next_due();
        posts.push(bridge::ToPage::Clock(bridge::Clock {
            step: self.world.steps(),
            now: self.world.now(),
            wall: self.world.wall(),
            idle: !busy,
            next_due,
            memory: 0,
            slice: Duration::ZERO,
        }));
        if busy {
            Wake::Now
        } else if settled || self.speed == bridge::Speed::PAUSED {
            Wake::Message
        } else {
            match next_due {
                Some(due) => Wake::At(browser.saturating_add(scaled(
                    due.saturating_since(self.world.now()),
                    1000,
                    self.speed.0,
                    true,
                ))),
                None => Wake::Message,
            }
        }
    }

    fn step_busy(&mut self, left: &mut u32) {
        while *left > 0 && self.world.busy() {
            self.world.step();
            *left = left.checked_sub(1).expect("a step has budget left");
        }
    }

    #[must_use]
    pub fn world(&self) -> &World<P, M> {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World<P, M> {
        &mut self.world
    }
}

// A u64 span times a u32 factor fits u128 before division; only the result saturates.
fn scaled(span: Duration, numerator: u32, denominator: u32, ceil: bool) -> Duration {
    let product = u128::from(span.as_nanos()).checked_mul(u128::from(numerator)).expect("u64 times u32 fits u128");
    let divisor = u128::from(denominator);
    let nanos = if ceil { product.div_ceil(divisor) } else { product.checked_div(divisor).expect("nonzero scale") };
    Duration::from_nanos(u64::try_from(nanos).unwrap_or(u64::MAX))
}

fn printed(line: skein_sim_world::Printed) -> bridge::Printed {
    let stream = match line.stream {
        skein_sim_world::Stream::Output => bridge::Stream::Output,
        skein_sim_world::Stream::Error => bridge::Stream::Error,
    };
    bridge::Printed { process: line.process, stream, line: line.line }
}
