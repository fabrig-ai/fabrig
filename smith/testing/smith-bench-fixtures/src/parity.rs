//! Parity row fixtures (smith/docs/design/testing.md, 7).

use smith_bench::{Measure, RunSummary};

/// Recorded arm walls and smith priced spend, supplied to the fixture.
#[derive(Clone, Copy, Debug)]
pub struct Rows {
    pub smith_wall_ms: u64,
    pub codex_wall_ms: u64,
    pub smith_usd: f64,
}

impl Rows {
    #[must_use]
    pub const fn equal() -> Self {
        Self { smith_wall_ms: 100_000, codex_wall_ms: 100_000, smith_usd: 1.0 }
    }
}

#[must_use]
pub fn parity_summary(rows: Rows) -> RunSummary {
    let file = crate::common::repository().join("tests/recorded/parity/on-par.json");
    let mut summary = smith_bench::read_summary(&file).expect("recorded parity");
    for row in &mut summary.attempts {
        row.task_wall_ms =
            Measure::Observed { value: if row.agent == "smith" { rows.smith_wall_ms } else { rows.codex_wall_ms } };
        row.priced_usd = Measure::Observed { value: if row.agent == "smith" { rows.smith_usd } else { 1.0 } };
    }
    summary
}

pub const CHANGE_COMMIT: &str = "0000000000000000000000000000000000000000";
pub const PARENT_COMMIT: &str = "1111111111111111111111111111111111111111";

#[must_use]
pub fn change_summary() -> RunSummary {
    let file = crate::common::repository().join("tests/recorded/parity/change.json");
    smith_bench::read_summary(&file).expect("recorded change")
}

#[must_use]
pub fn change_baseline() -> RunSummary {
    let mut summary = parity_summary(Rows::equal());
    summary.attempts.clear();
    summary.groups.clear();
    for criterion in &mut summary.parity.as_mut().expect("recorded parity").criteria {
        if criterion.name != "O" {
            criterion.interval = (0.9, 1.1);
        }
    }
    summary
}

#[must_use]
pub fn recompute(summary: &RunSummary) -> smith_bench::parity::Parity {
    smith_bench::parity::from_summary(summary).expect("verdict")
}
