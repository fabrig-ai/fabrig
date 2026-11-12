//! Small machine limits for deterministic scenarios (testing-strategy.md, 1).
//! Shared fixtures (smith/docs/design/testing.md, section 7).

#![forbid(unsafe_code)]

use skein_lib::Duration;
use smith_protocol_machine::Limits;

/// Small file, process and output admission edges.
pub const SMALL: Limits = Limits {
    operations: 2,
    roots: 1,
    path_bytes: 64,
    file_bytes: 16,
    tool_payload: 64,
    entries: 4,
    entry_bytes: 512,
    processes: 2,
    output_bytes: 64,
    search_hits: 8,
    search_bytes: 256,
    search_line_bytes: 512,
    env_bytes: 512,
    group_stop: Duration::from_millis(10),
};
