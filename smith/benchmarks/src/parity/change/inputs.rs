//! Named run inputs retain their path and content hash for offline recomputation.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{ChangeSection, Stop};
use crate::files::{Files, Native};
use crate::{Fault, Mismatch, RunSummary, input, mismatch, refuse_if};

/// A named run file supplied to a change report; missing files have no digest.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub path: PathBuf,
    pub sha256: Option<String>,
}

/// The caller's baseline run and landed change runs, in their supplied order.
#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub baseline_run: Option<PathBuf>,
    pub landed: Vec<PathBuf>,
}

impl Inputs {
    /// Remove the named run options, retaining other command options for their owner.
    pub fn parse(flags: &[String]) -> Result<(Self, Vec<String>), String> {
        if !flags.len().is_multiple_of(2) {
            return Err("options require values".into());
        }
        let mut inputs = Self::default();
        let mut rest = Vec::new();
        for pair in flags.as_chunks::<2>().0 {
            match pair[0].as_str() {
                "--baseline-run" => {
                    if inputs.baseline_run.replace(PathBuf::from(&pair[1])).is_some() {
                        return Err("baseline run given more than once".into());
                    }
                }
                "--landed" => inputs.landed.push(PathBuf::from(&pair[1])),
                _ => rest.extend_from_slice(pair),
            }
        }
        Ok((inputs, rest))
    }

    /// Whether a caller supplied any change-only input.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.baseline_run.is_none() && self.landed.is_empty()
    }
}

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("benchmark repository")
}

fn record(files: &dyn Files, path: &Path) -> Input {
    let absolute = files
        .canonicalize(path)
        .unwrap_or_else(|_| if path.is_absolute() { path.into() } else { repository().join(path) });
    let sha256 = crate::arm::binary_digest_with(files, &absolute).ok();
    // Repository-relative paths survive a checkout move; external inputs keep their absolute path.
    Input { path: absolute.strip_prefix(repository()).unwrap_or(&absolute).into(), sha256 }
}

fn read(files: &dyn Files, input: &Input) -> Result<RunSummary, Fault> {
    let path = repository().join(&input.path);
    let digest = crate::arm::binary_digest_with(files, &path)?;
    if input.sha256.as_ref() != Some(&digest) {
        return Err(mismatch(format!("input SHA-256 differs: {}", input.path.display())));
    }
    Ok(crate::read_summary_with(files, &path)?)
}

fn evaluate(
    files: &dyn Files,
    summary: &RunSummary,
    original: &ChangeSection,
    baseline: &Input,
    landed: &[Input],
) -> Result<ChangeSection, Fault> {
    let baseline = read(files, baseline)?;
    let landed = landed.iter().map(|input| read(files, input)).collect::<Result<Vec<_>, _>>()?;
    Ok(super::project_report(summary, original.clone(), &baseline, &landed).unwrap_or_else(|why| {
        let mut report = original.clone();
        report.stop = Some(Stop::Incomplete { why: why.detail });
        report
    }))
}

/// Attach named inputs and retain an incomplete report when any input is refused.
pub fn attach(summary: &mut RunSummary, inputs: &Inputs) -> Result<(), Fault> {
    attach_with(&Native, summary, inputs)
}

/// Attach named inputs from the supplied filesystem.
pub fn attach_with(files: &dyn Files, summary: &mut RunSummary, inputs: &Inputs) -> Result<(), Fault> {
    let parent = &summary.change.as_ref().ok_or_else(|| input("change section missing"))?.parent_commit;
    let original = super::from_summary(summary, parent)?;
    let baseline_run = inputs.baseline_run.as_deref().map(|path| record(files, path));
    let landed: Vec<_> = inputs.landed.iter().map(|path| record(files, path)).collect();
    let mut report = baseline_run
        .as_ref()
        .ok_or_else(|| input("baseline run missing"))
        .and_then(|baseline| evaluate(files, summary, &original, baseline, &landed))
        .unwrap_or_else(|why| {
            let mut report = original.clone();
            report.stop = Some(Stop::Incomplete { why: why.detail });
            report
        });
    report.baseline_run = baseline_run;
    report.landed = landed;
    summary.change = Some(report);
    Ok(())
}

/// Refuse changed inputs or a stored section that disagrees with its rows and inputs.
pub fn verify(summary: &RunSummary) -> Result<ChangeSection, Fault> {
    verify_with(&Native, summary)
}

/// Recompute a stored change section from the supplied filesystem.
pub fn verify_with(files: &dyn Files, summary: &RunSummary) -> Result<ChangeSection, Fault> {
    let stored = summary.change.as_ref().ok_or_else(|| input("change section missing"))?;
    let baseline = stored.baseline_run.as_ref().ok_or_else(|| input("baseline run missing"))?;
    let original = super::from_summary(summary, &stored.parent_commit)?;
    let mut report = evaluate(files, summary, &original, baseline, &stored.landed)?;
    report.baseline_run.clone_from(&stored.baseline_run);
    report.landed.clone_from(&stored.landed);
    refuse_if(stored != &report, Mismatch, "committed change section disagrees with rows or inputs")?;
    Ok(report)
}

/// Verify and print a change report, returning the offline command's exit code.
#[must_use]
pub fn command(file: &Path) -> u8 {
    command_with(&Native, file)
}

/// Verify a change report from the supplied filesystem and print its verdict.
#[must_use]
pub fn command_with(files: &dyn Files, file: &Path) -> u8 {
    let result =
        crate::read_summary_with(files, file).map_err(Fault::from).and_then(|summary| verify_with(files, &summary));
    match result {
        Ok(report) => {
            for ratio in &report.ratios {
                println!(
                    "{} statistic={} interval=[{}, {}]",
                    ratio.name, ratio.value.statistic, ratio.value.interval.0, ratio.value.interval.1
                );
            }
            println!(
                "O change={} parent={} attempts={} parent_lower={}",
                report.outcome.change_passes,
                report.outcome.parent_passes,
                report.outcome.attempts,
                report.outcome.parent_lower
            );
            for projection in &report.projection {
                println!(
                    "{} projection={} baseline_interval=[{}, {}]",
                    projection.name,
                    projection.statistic,
                    projection.baseline_interval.0,
                    projection.baseline_interval.1
                );
            }
            println!("stop={:?}", report.stop);
            0
        }
        Err(reason) => {
            eprintln!("{reason}");
            3
        }
    }
}
