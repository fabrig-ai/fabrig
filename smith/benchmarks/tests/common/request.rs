//! Prepared replay requests retain explicit task and identity inputs.

use super::bench::{Identity, run};
use std::path::PathBuf;

pub fn request(task: PathBuf, identity: Identity) -> run::Prepared {
    run::Prepared {
        task,
        identity,
        program: "/bin/sh".into(),
        arguments: Vec::new(),
        codex_home: None,
        configuration: None,
        teardown_ms: 50,
        grace_ms: 20,
        settlement_ms: 200,
        setup_deadline_seconds: 1,
        max_stdout_bytes: 4096,
        max_stderr_bytes: 4096,
        disk_estimate_bytes: 4096,
        force_walk: true,
    }
}
