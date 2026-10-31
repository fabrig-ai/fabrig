//! Calibration persists counted results against frozen task inputs.
//! No agent state is read; benchmark requests and settled public evidence are inputs.
//! Contract: smith/docs/design/benchmarks.md, sections 5.1 and 11.5.

use super::{Order, Planned, calibration_attempts, count, judgment};
use crate::{AttemptResult, Judgment};
use std::{fs, path::Path};

pub(super) fn finish_calibration(
    repository: &Path,
    order: &Order,
    planned: &Planned,
    before: Option<&(crate::arm::Build, Planned, crate::batch::Batch)>,
    required: u32,
    seed: u64,
) -> Result<Judgment, String> {
    let task = selected_task(&planned.requests[0].task, &order.probe)?;
    let batch = crate::batch::run_calibration(
        &planned.suite,
        &planned.directory,
        seed,
        &order.probe,
        required,
        order.attempts,
    )?;
    let results = batch.results;
    if results.first().is_some_and(crate::cache::selected) {
        print!("{}", crate::cache::notes(&results));
    }
    let counted = count(&results, required, order.attempts);
    println!(
        "summary {} passed={} failed={} deviations={} unjudged={} required={} ceiling={}",
        planned.run,
        counted.passed,
        counted.failed,
        counted.deviations,
        counted.unjudged,
        counted.required,
        counted.ceiling
    );
    let control_failed = before.as_ref().is_none_or(|(_, _, batch)| {
        judgment(batch) == Judgment::Fail
            && (!batch.results.iter().any(crate::cache::selected) || crate::cache::control_failure(&batch.results))
    });
    if counted.calibrated() && control_failed && !batch.incomplete {
        let failed_before = before.as_ref().map_or_else(Vec::new, |(_, _, batch)| {
            batch
                .results
                .iter()
                .filter(|result| crate::judge(result) == Judgment::Fail)
                .map(|result| format!("{}/{}", result.identity.run, result.identity.attempt))
                .collect()
        });
        let (attempts, passes, deviations) = calibration_attempts(&results, &planned.run);
        let first = &results[0].identity;
        let block = crate::Calibration {
            binary: first.agent_version.clone(),
            binary_sha256: first.binary_sha256.clone(),
            date: first.started_utc.clone(),
            summary: format!("summaries/{}.json", planned.run),
            attempts,
            passes,
            deviations,
            control: before.as_ref().map_or_else(
                || {
                    if order.probe == "probes/first-request" {
                        "none: the behaviour predates the oldest runnable face".into()
                    } else {
                        "none: no runnable control was named".into()
                    }
                },
                |(build, _, _)| build.arm.commit.clone(),
            ),
            failed_before,
            prompt_sha256: first.prompt_sha256.clone(),
            seed_sha256: first.seed_sha256.clone(),
            checks_sha256: task.checks_sha256()?,
        };
        record_calibration(repository, &planned.requests[0].task, &order.probe, &block, &results, seed)?;
        println!("{}", toml::to_string(&block).map_err(|error| error.to_string())?);
        Ok(Judgment::Pass)
    } else if counted.unjudged > 0
        || batch.incomplete
        || before.as_ref().is_some_and(|(_, _, batch)| judgment(batch) == Judgment::Incomplete)
    {
        Ok(Judgment::Incomplete)
    } else {
        Ok(Judgment::Fail)
    }
}

pub(super) fn selected_task(file: &Path, name: &str) -> Result<crate::Task, String> {
    let task = crate::read_task(file).map_err(|error| error.to_string())?;
    name.splitn(3, '/').nth(2).map_or(Ok(task.clone()), |variant| task.with_variant(variant))
}

/// Persist settled calibration evidence and its task or variant block, without committing.
pub fn record_calibration(
    repository: &Path,
    file: &Path,
    name: &str,
    block: &crate::Calibration,
    results: &[AttemptResult],
    seed: u64,
) -> Result<(), String> {
    let mut document: toml::Value = crate::formats::read_document(file).map_err(|error| error.to_string())?;
    let target = if let Some(name) = name.splitn(3, '/').nth(2) {
        document
            .get_mut("variant")
            .and_then(toml::Value::as_array_mut)
            .and_then(|variants| variants.iter_mut().find(|variant| variant["name"].as_str() == Some(name)))
            .ok_or("missing calibration variant")?
    } else {
        &mut document
    };
    target
        .as_table_mut()
        .ok_or("task table required")?
        .insert("calibration".into(), toml::Value::try_from(block).map_err(|error| error.to_string())?);
    let text = toml::to_string(&document).map_err(|error| error.to_string())?;
    let task = selected_task(file, name)?;
    let summary = crate::summarise(results, "calibrate", &task.tier, crate::Design::Single, seed, None, None)?;
    let path = repository.join("benchmarks").join(&block.summary);
    fs::create_dir_all(path.parent().ok_or("summary directory required")?).map_err(|error| error.to_string())?;
    crate::write_summary(&path, &summary)?;
    fs::write(file, text).map_err(|error| error.to_string())
}

#[cfg(test)]
mod calibration_tests {
    #[test]
    fn ten_credentials_pairs_form_a_valid_calibration_block() {
        use sha2::{Digest, Sha256};
        let fixture: crate::AttemptResult =
            serde_json::from_str(include_str!("../../tests/recorded/results/attempt.json")).expect("result fixture");
        let results: Vec<_> = (0..20)
            .map(|index| {
                let mut result = fixture.clone();
                result.identity.task = "probes/credentials".into();
                result.identity.attempt = format!("{index:02}");
                result
            })
            .collect();
        assert!(super::count(&results, 10, 12).calibrated());
        let (attempts, passes, deviations) = super::calibration_attempts(&results, "pairs");
        assert_eq!(attempts, 10);
        assert_eq!(passes.len(), 10);
        assert_eq!(super::count(&results[..19], 10, 12).unjudged, 1);
        let mut task: crate::Task =
            toml::from_str(include_str!("../../tasks/probes/credentials/task.toml")).expect("task");
        task.calibration = Some(crate::Calibration {
            binary: "a".repeat(40),
            binary_sha256: "b".repeat(64),
            date: "2026-10-20".into(),
            summary: "summaries/pairs.json".into(),
            attempts,
            passes,
            deviations,
            control: "none: synthetic control".into(),
            failed_before: Vec::new(),
            prompt_sha256: format!("{:x}", Sha256::digest(task.prompt.as_bytes())),
            seed_sha256: task.seed_sha256.clone(),
            checks_sha256: task.checks_sha256().expect("checks"),
        });
        crate::check::calibration(std::path::Path::new("synthetic-task.toml"), "calibration", &task)
            .expect("accepted calibration");
        assert!(task.calibrated().expect("matching inputs"));
    }
}
