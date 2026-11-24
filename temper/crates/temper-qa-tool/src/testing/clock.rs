//! The runner's clock, advanced by tests and waits (temper/docs/design/testing/qa/contracts.md, 4).
use crate::{
    events::Events,
    faces::{ClockFace, Moment},
};
use std::time::Duration;

#[derive(Default)]
pub struct Clock {
    pub time: u64,
    subsecond: Duration,
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
    }
    fn wait(&mut self, events: &Events, timeout: Option<u64>) {
        if !events.take() {
            self.time += timeout.expect("fake wait needs a deadline or an event");
        }
    }
}
