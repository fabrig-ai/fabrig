//! Faults, drawn from the seed per call (testing-strategy.md, section 4): a
//! rate limit per user per window, failures before a call is made and after,
//! and the latency of its answer, sometimes late.

use skein_lib::{Duration, Env, Time, earliest};

use crate::api::{Answer, Error};
use crate::domain::{self, Config, Domain};
use crate::fell::{Fault, Fell};

/// A user's rate window: when it ends, and the calls taken in it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Window {
    end: Time,
    calls: u32,
}

/// A namespace for a protocol admission and an ordinary domain caller.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RateKey {
    User(u64),
    Installation(u64),
}
/// A window snapshot after admission, using the forge's wall clock.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rate {
    pub limit: u32,
    pub used: u32,
    pub reset: skein_lib::Wall,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RateDenied {
    Limited(Rate),
    Crowded { rate: Rate, after: Duration },
}
/// Charges one protocol request. Continuations use `step_admitted`.
/// The caller admits a positive quota before calling.
pub fn rate(
    domain: &mut Domain,
    env: &Env<Config>,
    key: RateKey,
    limit: u32,
    window: Duration,
) -> Result<Rate, RateDenied> {
    let mut subject = Fell::new(env.now, Fault::Limited);
    subject.user = match key {
        RateKey::User(user) => Some(user),
        RateKey::Installation(_) => None,
    };
    charge(domain, env, key, limit, window, &subject, None)
}

fn charge(
    domain: &mut Domain,
    env: &Env<Config>,
    key: RateKey,
    limit: u32,
    window: Duration,
    subject: &Fell,
    repository: Option<&[u8]>,
) -> Result<Rate, RateDenied> {
    assert!(limit > 0, "admitted rate quota");
    let fresh = Window { end: env.now.saturating_add(window), calls: 1 };
    match domain.windows.get_mut(&key) {
        Some(stored) if env.now < stored.end => {
            if stored.calls >= limit {
                domain.tally.limited = domain.tally.limited.saturating_add(1);
                let rate = snapshot(env, limit, *stored);
                domain.call_fell(subject, repository, Fault::Limited, None);
                return Err(RateDenied::Limited(rate));
            }
            stored.calls = stored.calls.saturating_add(1);
            Ok(snapshot(env, limit, *stored))
        }
        Some(stored) => {
            *stored = fresh;
            Ok(snapshot(env, limit, fresh))
        }
        None => {
            if domain.windows.len() >= domain.windows.capacity() {
                let mut ended = None;
                let mut first: Option<Time> = None;
                for (&key, stored) in &domain.windows {
                    if stored.end <= env.now {
                        ended = Some(key);
                        break;
                    }
                    first = earliest(first, Some(stored.end));
                }
                match ended {
                    Some(key) => {
                        domain.windows.remove(&key);
                    }
                    None => {
                        domain.tally.crowded = domain.tally.crowded.saturating_add(1);
                        domain.call_fell(subject, repository, Fault::Crowded, None);
                        let end = first.unwrap_or(fresh.end);
                        return Err(RateDenied::Crowded {
                            rate: snapshot(env, limit, Window { end, calls: 0 }),
                            after: end.saturating_since(env.now),
                        });
                    }
                }
            }
            domain.windows.insert(key, fresh).expect("rate table room checked");
            Ok(snapshot(env, limit, fresh))
        }
    }
}
fn snapshot(env: &Env<Config>, limit: u32, window: Window) -> Rate {
    let reset = env.wall.as_nanos().saturating_add(window.end.saturating_since(env.now).as_nanos());
    Rate { limit, used: window.calls, reset: skein_lib::Wall::from_nanos(reset) }
}
/// Takes an ordinary domain call, including its rate and unavailable draw.
pub(crate) fn admit(
    domain: &mut Domain,
    env: &Env<Config>,
    user: u64,
    subject: &Fell,
    repository: &[u8],
) -> Result<(), Error> {
    if env.limits.rate_limit > 0 {
        match charge(
            domain,
            env,
            RateKey::User(user),
            env.limits.rate_limit,
            env.limits.rate_window,
            subject,
            Some(repository),
        ) {
            Ok(_) => {}
            Err(RateDenied::Limited(_)) => {
                let end = domain.windows.get(&RateKey::User(user)).expect("limited window").end;
                return Err(Error::RateLimited { reset: domain::time(&env.limits, end) });
            }
            Err(RateDenied::Crowded { after, .. }) => {
                return Err(Error::RateLimited { reset: domain::time(&env.limits, env.now.saturating_add(after)) });
            }
        }
    }
    available(domain, env, subject, repository)
}
pub(crate) fn available(
    domain: &mut Domain,
    env: &Env<Config>,
    subject: &Fell,
    repository: &[u8],
) -> Result<(), Error> {
    if domain.rng.chance(env.limits.unavailable) {
        domain.tally.unavailable = domain.tally.unavailable.saturating_add(1);
        domain.call_fell(subject, Some(repository), Fault::Unavailable, None);
        return Err(Error::Unavailable);
    }
    Ok(())
}

/// Whether a call is to land late: answered as timed out, and made after.
pub(crate) fn lands_late(domain: &mut Domain, env: &Env<Config>) -> bool {
    domain.rng.chance(env.limits.landing)
}

/// What the caller hears of a call that was made: `result`, or, by chance, a
/// timeout.
pub(crate) fn finish(
    domain: &mut Domain,
    env: &Env<Config>,
    result: Result<Answer, Error>,
    subject: &Fell,
    repository: &[u8],
) -> Result<Answer, Error> {
    if domain.rng.chance(env.limits.timeouts) {
        domain.tally.timeouts = domain.tally.timeouts.saturating_add(1);
        domain.call_fell(subject, Some(repository), Fault::TimedOut, None);
        return Err(Error::Timeout);
    }
    result
}

/// When a call's answer goes out: after its latency, or late.
pub(crate) fn latency(
    domain: &mut Domain,
    env: &Env<Config>,
    subject: &Fell,
    repository: &[u8],
    extra: Duration,
) -> Time {
    let config = &env.limits;
    let late = domain.rng.chance(config.late);
    let span = if late {
        domain.tally.late = domain.tally.late.saturating_add(1);
        draw(domain, config.late_min, config.late_max)
    } else {
        draw(domain, config.latency_min, config.latency_max)
    };
    let until = env.now.saturating_add(span).saturating_add(extra);
    if late {
        domain.call_fell(subject, Some(repository), Fault::Late, Some(until));
    }
    until
}

/// A span drawn from `min..=max`.
pub(crate) fn draw(domain: &mut Domain, min: Duration, max: Duration) -> Duration {
    Duration::from_nanos(domain.rng.between(min.as_nanos(), max.as_nanos()))
}
