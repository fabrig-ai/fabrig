//! Settled attempt lines, logical calibration counts and judgments.

use super::*;

/// The settled attempt's printable outcome and measured wall and spend.
#[derive(Debug)]
pub struct Line<'a>(&'a AttemptResult);

pub(super) fn verdict_name(verdict: Judgment) -> &'static str {
    match verdict {
        Judgment::Pass => "pass",
        Judgment::Fail => "fail",
        Judgment::Incomplete => "incomplete",
    }
}

impl Display for Line<'_> {
    fn fmt(&self, text: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.0;
        let end = match result.outcome.end {
            End::Completed => "completed",
            End::Failed { .. } => "failed",
            End::Budget { .. } => "budget",
            End::Refused { .. } => "refused",
            End::Timeout => "timeout",
            End::HarnessError { .. } => "harness-error",
        };
        write!(
            text,
            "{}/{} end={end} verdict={} wall=",
            result.identity.run,
            result.identity.attempt,
            verdict_name(crate::judge(result))
        )?;
        match result.timing.task_wall_ms {
            Measure::Observed { value } => write!(text, "{}.{:03}s", value / 1000, value % 1000)?,
            Measure::LowerBound { .. } | Measure::Unavailable { .. } => text.write_str("-")?,
        }
        text.write_str(" spend=")?;
        match result.spend.usd {
            Measure::Observed { value } => write!(text, "${value:.6}"),
            Measure::LowerBound { .. } | Measure::Unavailable { .. } => text.write_str("-"),
        }
    }
}

/// Retain unavailable measurements while preparing one settled line.
#[must_use]
pub fn line(result: &AttemptResult) -> Line<'_> {
    Line(result)
}

/// Calibration counts supplied by settled logical attempts.
#[derive(Debug, PartialEq, Eq)]
pub struct Counted {
    pub passed: u32,
    pub failed: u32,
    pub deviations: u32,
    pub unjudged: u32,
    pub required: u32,
    pub ceiling: u32,
}

impl Counted {
    /// Enough passes arrived within the command's attempt ceiling.
    #[must_use]
    pub const fn calibrated(&self) -> bool {
        self.passed >= self.required
            && self.failed == 0
            && self.unjudged == 0
            && self.passed + self.deviations <= self.ceiling
    }

    /// Remaining attempts can still attain the required number of passes.
    #[must_use]
    pub fn reachable(&self) -> bool {
        self.failed == 0
            && self.unjudged == 0
            && self.ceiling.saturating_sub(self.deviations) >= self.required
            && self.passed + self.failed + self.deviations + self.unjudged < self.ceiling
    }
}

fn physical_size(results: &[AttemptResult]) -> usize {
    if results.first().is_some_and(crate::cache::selected) {
        3
    } else if results.first().is_some_and(|result| result.identity.task == "probes/credentials") {
        2
    } else {
        1
    }
}

pub(super) fn calibration_attempts(results: &[AttemptResult], run: &str) -> (u32, Vec<String>, Vec<String>) {
    let mut passes = Vec::new();
    let mut deviations = Vec::new();
    let physical = physical_size(results);
    for attempt in results.chunks(physical) {
        if attempt.len() != physical {
            continue;
        }
        let id = format!("{run}/{}", attempt[0].identity.attempt);
        if crate::judge_suite(attempt, false) == Judgment::Pass {
            passes.push(id);
        } else if attempt.iter().all(model_deviation) {
            deviations.push(id);
        }
    }
    (u32::try_from(passes.len() + deviations.len()).expect("bounded calibration attempts"), passes, deviations)
}

/// Count each logical attempt, including cache triplets, without passing missing evidence.
#[must_use]
pub fn count(results: &[AttemptResult], required: u32, ceiling: u32) -> Counted {
    let mut counted = Counted { passed: 0, failed: 0, deviations: 0, unjudged: 0, required, ceiling };
    let physical = physical_size(results);
    for attempt in results.chunks(physical) {
        if attempt.len() != physical {
            counted.unjudged += 1;
            continue;
        }
        match crate::judge_suite(attempt, false) {
            Judgment::Pass => counted.passed += 1,
            Judgment::Fail | Judgment::Incomplete if attempt.iter().all(model_deviation) => counted.deviations += 1,
            Judgment::Fail if attempt.iter().all(|result| deviation(result, true)) => counted.unjudged += 1,
            Judgment::Fail => counted.failed += 1,
            Judgment::Incomplete => counted.unjudged += 1,
        }
    }
    counted
}

/// A completed answer may deviate from the task while smith's contracts hold.
#[must_use]
pub fn model_deviation(result: &AttemptResult) -> bool {
    deviation(result, false)
}

fn deviation(result: &AttemptResult, permit_unavailable: bool) -> bool {
    if result.outcome.end != End::Completed {
        return false;
    }
    if !permit_unavailable
        && result.outcome.deciding.iter().any(|name| !result.outcome.checks.iter().any(|check| check.check == *name))
    {
        return false;
    }
    let mut deviation = false;
    for check in &result.outcome.checks {
        if !result.outcome.deciding.contains(&check.check) || check.passed == (Measure::Observed { value: true }) {
            continue;
        }
        if permit_unavailable && matches!(&check.passed, Measure::Unavailable { .. }) {
            continue;
        }
        let model = check.check == "answer"
            || check.check == "call-count"
            || check.check == "budget-report"
            || ["file-content:", "file-digest:", "pattern:"].iter().any(|prefix| check.check.starts_with(prefix))
            || matches!(&check.passed, Measure::Unavailable { reason } if reason.starts_with("not exercised:"));
        if !model {
            return false;
        }
        deviation = true;
    }
    deviation
}

pub(crate) fn judgment(batch: &crate::batch::Batch) -> Judgment {
    if batch.results.iter().any(|result| {
        matches!(result.outcome.end, End::HarnessError { .. })
            || (matches!(result.outcome.end, End::Refused { .. }) && !crate::batch::expected_refusal(result))
    }) || batch.probes.values().any(|verdict| matches!(verdict, crate::ProbeVerdict::Quarantined))
    {
        Judgment::Incomplete
    } else {
        crate::judge_suite(&batch.results, batch.incomplete)
    }
}

pub(super) fn summary(planned: &Planned, batch: &crate::batch::Batch) -> Judgment {
    let verdict = judgment(batch);
    println!("summary {} attempts={} verdict={}", planned.run, batch.results.len(), verdict_name(verdict));
    verdict
}
