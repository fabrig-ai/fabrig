//! Boundary traces and deterministic replay (testing-strategy.md, section 6).
//! Worlds provide observations and seeded runs; these helpers compare the
//! trace and scenario outcome without reading a clock or domain state.

use alloc::{format, string::String, vec::Vec};

use core::fmt::{Debug, Display};

use skein_lib::Time;

/// What crossed a world's boundaries, in order, each line with the time it
/// crossed at: a seed replays to the same trace (testing-strategy.md, section 6).
#[derive(Default, Debug)]
pub struct Trace {
    lines: Vec<String>,
}

impl Trace {
    /// Records one boundary observation at the world-supplied time.
    pub fn log<D: Display>(&mut self, now: Time, line: D) {
        self.lines.push(format!("{:>16} {line}", now.as_nanos()));
    }

    /// Returns recorded observations in their original order for replay checks.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

/// Checks that a world replays from its seed: `run` runs a world of a seed to
/// its end, and returns any comparable trace and whatever else must come out the same
/// (its stats, the time it settled at, its state digests). `seed` run twice comes out the same,
/// and `other` takes a course of its own. Returns the trace of `seed`, for the
/// caller to check that the world did something.
pub fn check_replays<R: PartialEq, T: PartialEq + Debug, F: Fn(u64) -> (R, T)>(
    seed: u64,
    other: u64,
    run: F,
) -> Result<R, String> {
    let (trace, end) = run(seed);
    let again = run(seed);
    if again.0 != trace {
        return Err(format!("seed {seed} replays to the same trace"));
    }
    if again.1 != end {
        return Err(format!(
            "assertion `left == right` failed: seed {seed} replays to the same end\n  left: {:?}\n right: {end:?}",
            again.1
        ));
    }
    if run(other).0 == trace {
        return Err(format!("seeds {seed} and {other} take courses of their own"));
    }
    Ok(trace)
}

/// Checks replay and seed diversity as [`check_replays`] does, panicking on a
/// differing trace or end, or seeds that take the same course.
pub fn assert_replays<R: PartialEq, T: PartialEq + Debug, F: Fn(u64) -> (R, T)>(seed: u64, other: u64, run: F) -> R {
    match check_replays(seed, other, run) {
        Ok(trace) => trace,
        Err(why) => crate::fail(&why),
    }
}
