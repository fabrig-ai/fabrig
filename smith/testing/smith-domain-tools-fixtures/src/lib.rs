//! Small limits for step tests and deterministic tools worlds.
//! Contract: smith/docs/design/domain/tools.md, section 4.
//! Shared fixtures: smith/docs/design/testing.md, section 7.
#![no_std]

use skein_lib::Duration;

use smith_domain_tools::Limits;

/// A small tools admission envelope; tests override the bounds they exercise.
pub const SMALL: Limits = Limits {
    shell_output: 192,
    kits: 2,
    calls: 4,
    repos: 4,
    path_bytes: 64,
    host_path_bytes: 4096,
    known_files: 8,
    file_bytes: 1024,
    tool_payload: 1024,
    read_bytes: 256,
    list_entries: 16,
    list_bytes: 4096,
    match_lines: 4,
    file_timeout: Duration::from_secs(10),

    jobs: 4,
    job_poll: Duration::from_secs(30),
    job_wait: Duration::from_secs(30),
    shell_timeout: Duration::from_secs(60),
    tool_deadline: Duration::from_secs(600),
    shell_head: 64,
    shell_tail: 128,
    search_hits: 8,
    search_bytes: 256,
    search_timeout: Duration::from_secs(30),
};
