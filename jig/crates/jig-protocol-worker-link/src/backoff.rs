//! Jittered redials (protocol/channel/README.md, 9.1).
use crate::Limits;
use skein_lib::{Duration, Rng};
#[derive(Debug)]
pub(crate) struct Backoff {
    first: Duration,
    ceiling: Duration,
    current: Duration,
    rng: Rng,
}
impl Backoff {
    pub(crate) fn new(limits: &Limits, seed: u64) -> Self {
        Self { first: limits.backoff, ceiling: limits.ceiling, current: limits.backoff, rng: Rng::new(seed) }
    }
    /// The next redial starts at the new backoff (protocol/channel/figures.md, 4).
    pub(crate) fn retime(&mut self, first: Duration, ceiling: Duration) {
        self.first = first;
        self.ceiling = ceiling;
        self.current = first;
    }
    pub(crate) fn next(&mut self) -> Duration {
        let high = self.current.as_nanos();
        let span = self.rng.between(high.div_euclid(2), high);
        self.current = self.current.saturating_mul(2).min(self.ceiling);
        Duration::from_nanos(span)
    }
    pub(crate) fn proved(&mut self) {
        self.current = self.first;
    }
}
