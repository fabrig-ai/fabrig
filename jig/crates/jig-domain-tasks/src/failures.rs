//! Per-class task failure history and configured equal-jitter backoff
//! (domain/tasks/lifecycle-holds.md, section 1). Root classifies external failures;
//! this module uses injected randomness and owns no clock or peer state.
pub use jig_domain_contract::ops::Tries;

pub use jig_domain_contract::ops::Class;
use skein_lib::{Duration, Rng};

/// Root-configured retry count and equal-jitter exponential delay bounds; startup validates
/// representable usable settings. (domain/tasks/lifecycle-holds.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Retry {
    /// Failures allowed before the task is held; startup rejects `u32::MAX`.
    pub retries: u32,
    /// Positive initial exponential-backoff delay; startup rejects zero.
    pub base: Duration,
    /// Backoff ceiling, required to be at least `base`; jitter selects within the upper half of the
    /// capped delay.
    pub max: Duration,
}

/// One configured retry policy per exhaustive failure class. (domain/tasks/lifecycle-holds.md, section 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Retries {
    /// `Transient`-failure and pre-execution refusal pause policy.
    pub transient: Retry,
    /// Policy for failures requiring an external correction.
    pub permanent: Retry,
    pub run: Retry,
    pub agent: Retry,
    pub lost: Retry,
    pub invalid: Retry,
    pub infrastructure: Retry,
}

impl Retries {
    /// Pure selection of the counter or retry policy for `class`; no allocation, mutation, output
    /// or lifecycle transition.
    #[must_use]
    pub const fn of(&self, class: Class) -> Retry {
        match class {
            Class::Transient => self.transient,
            Class::Permanent => self.permanent,
            Class::Run => self.run,
            Class::Agent => self.agent,
            Class::Lost => self.lost,
            Class::Invalid => self.invalid,
            Class::Infrastructure => self.infrastructure,
        }
    }
}

pub(crate) fn backoff(times: u32, retry: Retry, rng: &mut Rng) -> Duration {
    let factor = 1_u64.checked_shl(times.saturating_sub(1)).unwrap_or(u64::MAX);
    let ceiling = retry.base.saturating_mul(factor).min(retry.max);
    let half = ceiling.as_nanos().div_euclid(2);
    Duration::from_nanos(half.saturating_add(rng.below(half.saturating_add(1))))
}

pub(crate) fn add_try(tries: &mut Tries, class: Class) {
    match class {
        Class::Transient => tries.transient = tries.transient.saturating_add(1),
        Class::Permanent => tries.permanent = tries.permanent.saturating_add(1),
        Class::Run => tries.run = tries.run.saturating_add(1),
        Class::Agent => tries.agent = tries.agent.saturating_add(1),
        Class::Lost => tries.lost = tries.lost.saturating_add(1),
        Class::Invalid => tries.invalid = tries.invalid.saturating_add(1),
        Class::Infrastructure => tries.infrastructure = tries.infrastructure.saturating_add(1),
    }
}
