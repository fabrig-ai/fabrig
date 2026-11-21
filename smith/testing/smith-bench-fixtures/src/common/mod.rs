//! Shared benchmark observations and recorded inputs (smith/docs/design/testing.md, 7).

pub use crate::events;

#[must_use]
pub fn calibration(task: &smith_bench::Task) -> smith_bench::Calibration {
    use sha2::{Digest, Sha256};
    smith_bench::Calibration {
        binary: "a".repeat(40),
        binary_sha256: "b".repeat(64),
        date: "2026-10-14".into(),
        summary: "summaries/calibration.json".into(),
        attempts: 10,
        passes: (1..=10).map(|index| format!("recorded-run/{index:02}")).collect(),
        deviations: vec![],
        control: "c".repeat(40),
        failed_before: vec!["control-run/01".into()],
        prompt_sha256: format!("{:x}", Sha256::digest(task.prompt.as_bytes())),
        seed_sha256: task.seed_sha256.clone(),
        checks_sha256: task.checks_sha256().expect("checks digest"),
    }
}

#[must_use]
pub fn task(name: &str) -> smith_bench::Task {
    let root = repository();
    smith_bench::check_task(&root.join("tasks").join(name).join("task.toml"), &root.join("../docs/design"))
        .expect("authored task and frozen seed")
}

#[must_use]
pub fn repository() -> std::path::PathBuf {
    skein_scratch::paths::tree().join("benchmarks")
}

#[must_use]
pub fn attempt() -> smith_bench::AttemptResult {
    smith_bench::read_result(&repository().join("tests/recorded/results/attempt.json")).expect("recorded attempt")
}

mod observe;
pub use observe::{captured, observe};

pub use crate::request::request;

pub mod edit;

pub use crate::json;

mod tool;
pub use tool::tool_pair;

#[must_use]
pub fn design() -> std::path::PathBuf {
    crate::common::repository().join("../docs/design")
}

#[must_use]
pub fn offline_recorded(name: &str) -> std::path::PathBuf {
    crate::common::repository().join("tests/recorded/tasks").join(name).join("task.toml")
}

#[must_use]
pub fn recorded_passes() -> Vec<smith_bench::AttemptResult> {
    let result = crate::common::attempt();
    (1..=10)
        .map(|index| {
            let mut result = result.clone();
            result.identity.attempt = format!("{index:02}");
            result
        })
        .collect()
}

#[must_use]
pub fn probe_result() -> smith_bench::AttemptResult {
    smith_bench::read_result(&crate::common::repository().join("tests/recorded/results/attempt.json"))
        .expect("unchanged recorded result")
}
