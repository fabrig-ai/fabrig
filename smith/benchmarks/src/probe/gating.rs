//! Calibration gates smoke selection and probe regressions, keeping raw results.
//! Contract: ../../../docs/design/benchmarks/README.md, sections 3.1 and 9, and benchmarks/catalogue.md, section 5.

use std::collections::BTreeSet;

use crate::{CatalogueTask, Fault, Judgment, batch::Batch, run::Request};

/// Select smoke tasks whose effective calibration digests still match.
pub fn smoke_tasks(entries: &[CatalogueTask], names: &[String]) -> Result<Vec<String>, Fault> {
    let mut selected = Vec::new();
    for name in names {
        if super::task(entries, name)?.task.calibrated()? {
            selected.push(name.clone());
        } else {
            println!("{name}: not calibrated: skipped");
        }
    }
    Ok(selected)
}

pub(crate) fn uncalibrated(requests: &[Request]) -> Result<BTreeSet<String>, Fault> {
    let mut names = BTreeSet::new();
    for request in requests {
        if request.identity.task.starts_with("probes/")
            && !super::selected_task(&request.task, &request.identity.task)?.calibrated()?
        {
            names.insert(request.identity.task.clone());
        }
    }
    Ok(names)
}

pub(crate) fn suite_tasks(
    suite: &crate::Suite,
    logical: bool,
    requests: &[Request],
) -> Result<BTreeSet<String>, Fault> {
    if suite.name == "probes" && !logical { uncalibrated(requests) } else { Ok(BTreeSet::new()) }
}

pub(crate) fn report(mut batch: Batch, names: &BTreeSet<String>) -> Batch {
    batch.probes.retain(|key, verdict| {
        if key.rsplit_once('/').is_some_and(|(task, _)| names.contains(task)) {
            println!("{key}: not calibrated: {verdict:?}; no regression counted");
            false
        } else {
            true
        }
    });
    batch
}

/// Judge calibrated evidence while preserving infrastructure failures and raw attempts.
#[must_use]
pub fn judgment(batch: &Batch, names: &BTreeSet<String>) -> Judgment {
    let results = batch
        .results
        .iter()
        .filter(|result| {
            !names.contains(&result.identity.task)
                || matches!(result.outcome.end, crate::End::HarnessError { .. })
                || (matches!(result.outcome.end, crate::End::Refused { .. }) && !crate::batch::expected_refusal(result))
        })
        .cloned()
        .collect::<Vec<_>>();
    if results.is_empty() {
        return if batch.incomplete || batch.results.is_empty() { Judgment::Incomplete } else { Judgment::Pass };
    }
    let gated = report(Batch { results, incomplete: batch.incomplete, probes: batch.probes.clone() }, names);
    super::judgment(&gated)
}
