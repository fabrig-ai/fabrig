//! The runner's clock, advanced by tests and waits (temper/docs/design/testing/qa/README.md, 6).
use crate::{
    events::Events,
    faces::{ClockFace, Moment},
};
use std::time::Duration;

#[derive(Default)]
pub struct Clock {
    pub time: u64,
    subsecond: Duration,
    after_sleep: Option<Box<dyn FnOnce()>>,
}

impl Clock {
    /// Let a scripted peer act when the caller next yields by sleeping.
    pub fn after_sleep(&mut self, action: impl FnOnce() + 'static) {
        assert!(self.after_sleep.is_none(), "one action per sleep");
        self.after_sleep = Some(Box::new(action));
    }
}

impl ClockFace for Clock {
    fn now(&self) -> Moment {
        Moment { wall: self.time, monotonic: self.time }
    }
    fn monotonic(&self) -> Duration {
        Duration::from_secs(self.time) + self.subsecond
    }
    fn sleep(&mut self, duration: Duration) {
        let time = self.monotonic() + duration;
        self.time = time.as_secs();
        self.subsecond = Duration::from_nanos(u64::from(time.subsec_nanos()));
        if let Some(action) = self.after_sleep.take() {
            action();
        }
    }
    fn wait(&mut self, events: &Events, timeout: Option<u64>) {
        if !events.take() {
            self.time += timeout.expect("fake wait needs a deadline or an event");
        }
    }
}
