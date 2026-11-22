//! Inputs and entry points for replaying task and suite checks.

use std::path::{Path, PathBuf};

use crate::{AttemptResult, CatalogueTask, Configuration, PinnedConfiguration, Refusal, Task};

pub use crate::check::{SeedEntry, SeedFiles, SeedKind, check_task_with, seed_digest_with};
pub use crate::formats::parse_document;
pub use crate::suite::validate_suite_with;
pub use crate::task::parse_task;

pub fn probe_cases(result: AttemptResult) {
    crate::batch::probe_cases(result.clone());
    crate::fix::probe_cases(result);
}

pub fn configuration(
    file: &Path,
    text: &str,
    contents: impl FnOnce(&Path) -> Result<Vec<u8>, Refusal>,
) -> Result<PinnedConfiguration, Refusal> {
    let pin: Configuration = parse_document(file, text)?;
    crate::agent::configuration_with(file, pin, |path| Ok((text.as_bytes().to_vec(), contents(path)?)))
}

#[must_use]
pub fn sample_catalogue(manifest: PathBuf, mut task: Task) -> Vec<CatalogueTask> {
    task.behaviours.push("read".into());
    task.variant.push(crate::Variant {
        calibration: None,
        replace_checks: false,
        deciding: Vec::new(),
        name: "short".into(),
        prompt: None,
        effort: None,
        person: None,
        input_end: None,
        smith: std::collections::BTreeMap::new(),
        environment: std::collections::BTreeMap::new(),
        outcome: Vec::new(),
        event: Vec::new(),
        waives: Vec::new(),
        budget: None,
    });
    vec![CatalogueTask { name: "probes/recorded".into(), manifest, task }]
}
