//! The worker's platform, its pure half (ui-web.md, 18).
#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::{FailStop, Worker, worker_address};

use crate::{bridge, navigation, store};
use skein_lib::{Duration, Time, Wall};

/// What the world says that is no fetch report, for the application's own report variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    Printed(bridge::Printed),
    Stopped,
    Failed { words: Box<[u8]> },
    Clock,
}

/// What the page holds of its world: the clock, the pace, the messages held until the
/// worker is ready, the small store in memory and the world's high waters.
#[derive(Debug)]
pub struct Page {
    ready: bool,
    outgoing: Vec<bridge::ToWorld>,
    speed: bridge::Speed,
    hidden: bool,
    clock: bridge::Clock,
    browser: Time,
    given: (Time, Wall),
    high_water: (u64, Duration),
    records: Vec<store::Record>,
}

impl Page {
    /// Holds the start first, then the initial pace; reads the start's wall and instant zero
    /// until the world's first clock.
    #[must_use]
    pub fn new(start: bridge::Start, browser: Time) -> Page {
        let wall = start.start;
        Self {
            ready: false,
            outgoing: vec![bridge::ToWorld::Start(start), bridge::ToWorld::Pace(bridge::Speed::ONE)],
            speed: bridge::Speed::ONE,
            hidden: false,
            clock: bridge::Clock {
                step: 0,
                now: Time::ZERO,
                wall,
                idle: false,
                next_due: None,
                memory: 0,
                slice: Duration::ZERO,
            },
            browser,
            given: (Time::ZERO, wall),
            high_water: (0, Duration::ZERO),
            records: Vec::new(),
        }
    }

    /// The worker said it is ready: what was held may go.
    pub fn ready(&mut self) {
        self.ready = true;
    }

    /// Appends the messages to post now, in order: none until ready.
    pub fn outgoing(&mut self, out: &mut Vec<bridge::ToWorld>) {
        if self.ready {
            out.append(&mut self.outgoing);
        }
    }

    /// An input for the visitor, held until ready.
    pub fn send(&mut self, input: bridge::Input) {
        self.outgoing.push(bridge::ToWorld::Input(input));
    }

    /// The frame's speed; sent unless the tab is hidden.
    pub fn pace(&mut self, speed: bridge::Speed, browser: Time) {
        self.rebase(browser);
        self.speed = speed;
        if !self.hidden {
            self.outgoing.push(bridge::ToWorld::Pace(speed));
        }
    }

    /// A hidden tab pauses the world; the shown tab resumes the frame's speed.
    pub fn visibility(&mut self, hidden: bool, browser: Time) {
        if self.hidden == hidden {
            return;
        }
        self.rebase(browser);
        self.hidden = hidden;
        let speed = if hidden { bridge::Speed::PAUSED } else { self.speed };
        self.outgoing.push(bridge::ToWorld::Pace(speed));
    }

    /// A world's message at `browser`: a clock moves the platform's clock and the high waters.
    pub fn heard(&mut self, message: &bridge::ToPage, browser: Time) {
        match message {
            bridge::ToPage::Clock(clock) => {
                self.clock = *clock;
                self.browser = browser;
                self.high_water.0 = self.high_water.0.max(clock.memory);
                self.high_water.1 = self.high_water.1.max(clock.slice);
            }
            bridge::ToPage::Report(_)
            | bridge::ToPage::Printed(_)
            | bridge::ToPage::Stamp(_)
            | bridge::ToPage::Stopped
            | bridge::ToPage::Failed { .. } => {}
        }
    }

    /// The world's clock as the platform gives it at `browser`: never back.
    pub fn clocks(&mut self, browser: Time) -> (Time, Wall) {
        self.given = self.reading(browser);
        self.given
    }

    /// The browser instant at which the platform's clock reaches a world deadline.
    /// None while paused, while the world steps, or past the world's next due instant.
    #[must_use]
    pub fn timer(&self, deadline: Time, browser: Time) -> Option<Time> {
        if !self.running() {
            return None;
        }
        if deadline <= self.reading(browser).0 {
            return Some(browser);
        }
        if self.clock.next_due.is_some_and(|due| deadline > due) {
            return None;
        }
        let remaining = deadline.saturating_since(self.clock.now).as_nanos();
        let scaled = u128::from(remaining).checked_mul(1000)?;
        let delay = scaled.checked_div(u128::from(self.speed.0))?;
        let remainder = scaled.checked_rem(u128::from(self.speed.0))?;
        let delay = delay.checked_add(u128::from(remainder != 0))?;
        let delay = Duration::from_nanos(u64::try_from(delay).ok()?);
        self.browser.checked_add(delay).map(|at| at.max(browser))
    }

    /// The high waters of the world's memory and of its slices.
    #[must_use]
    pub fn high_water(&self) -> (u64, Duration) {
        self.high_water
    }

    /// The whole small-store records, in insertion order.
    #[must_use]
    pub fn read_all(&self) -> Vec<store::Record> {
        self.records.clone()
    }

    /// Replaces a record in place, or appends it; the bound is the store's.
    pub fn write(&mut self, key: &[u8], value: &[u8]) -> bool {
        if let Some(record) = self.records.iter_mut().find(|record| record.key.as_ref() == key) {
            record.value = value.into();
        } else {
            self.records.push(store::Record { key: key.into(), value: value.into() });
        }
        true
    }

    /// Removes the record, if present.
    pub fn remove(&mut self, key: &[u8]) {
        self.records.retain(|record| record.key.as_ref() != key);
    }

    fn running(&self) -> bool {
        self.clock.idle && !self.hidden && self.speed != bridge::Speed::PAUSED
    }

    fn projected(&self, browser: Time) -> (Time, Wall) {
        if !self.running() {
            return (self.clock.now, self.clock.wall);
        }
        // A u64 browser span times a u32 speed fits u128 before division.
        let elapsed = u128::from(browser.saturating_since(self.browser).as_nanos());
        let scaled = elapsed.checked_mul(u128::from(self.speed.0)).expect("u64 times u32 fits u128");
        let nanos = scaled.checked_div(1000).expect("the speed's scale is nonzero");
        let mut nanos = u64::try_from(nanos).unwrap_or(u64::MAX);
        if let Some(due) = self.clock.next_due {
            nanos = nanos.min(due.saturating_since(self.clock.now).as_nanos());
        }
        nanos = nanos.min(u64::MAX.saturating_sub(self.clock.now.as_nanos()));
        let now = self.clock.now.saturating_add(Duration::from_nanos(nanos));
        let wall = Wall::from_nanos(self.clock.wall.as_nanos().saturating_add(nanos));
        (now, wall)
    }

    fn reading(&self, browser: Time) -> (Time, Wall) {
        let (now, wall) = self.projected(browser);
        (now.max(self.given.0), wall.max(self.given.1))
    }

    fn rebase(&mut self, browser: Time) {
        let (now, wall) = self.projected(browser);
        self.clock.now = now;
        self.clock.wall = wall;
        self.browser = browser;
    }
}

/// A navigation the worker's platform follows: a path on the page's own origin
/// (ui-web.md, 10), shown in the address bar.
#[must_use]
pub fn shown(to: &[u8]) -> bool {
    to.starts_with(b"/") && navigation::allowed(to, false)
}
