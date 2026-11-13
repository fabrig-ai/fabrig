//! Prepared replay requests retain explicit task and identity inputs.

use smith_bench::{Identity, run};
use std::path::PathBuf;

#[must_use]
pub fn request(task: PathBuf, identity: Identity) -> run::Request {
    run::Request {
        task,
        // A test without its own directory gets an unavailable listing, never the machine's /tmp.
        shared_temporary: "/nonexistent-shared-temporary".into(),
        identity,
        program: "/bin/sh".into(),
        arguments: Vec::new(),
        logins: None,
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
