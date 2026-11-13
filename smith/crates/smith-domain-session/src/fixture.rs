//! Small session admission limits for deterministic worlds and inline tests.
//! Contract: smith/docs/design/domain/session.md, sections 3 and 6.

use crate::{Budget, Limits};
use skein_lib::Duration;
use smith_domain_tools as tools;

/// The small fixture's token, turn and time budget.
pub const BUDGET: Budget = Budget {
    turns: 4,
    input: 1_000_000,
    output: 1_000_000,
    cache_read: 1_000_000,
    cache_write: 1_000_000,
    time: Duration::from_secs(600),
};

/// The tools' limits: a kit for each session, room for a batch and more.
pub const TOOLS: tools::Limits = tools::Limits {
    shell_output: 128,
    calls: 8,
    repos: 2,
    file_bytes: 65_536,
    tool_payload: 65_536,
    read_bytes: 4096,
    file_timeout: Duration::from_secs(60),
    shell_tail: 64,
    ..smith_domain_tools_fixtures::SMALL
};

/// Two sessions with room for a small transcript and two parallel tools.
pub const SMALL: Limits = Limits {
    threshold: [4, 5],
    token_bytes: 8,
    sessions: 2,
    spend: 100,
    protocol_allowance: 0,
    history_blocks: u32::MAX,
    history_messages: 8,
    history_bytes: 65_536,
    completion_bytes: 4096,
    answer_bytes: 4096,
    completion_blocks: 16,
    failure_bytes: 512,
    delegated_result_bytes: 16_384,
    budget: BUDGET,
    retries: 2,
    backoff_base: Duration::from_millis(100),
    backoff_max: Duration::from_secs(1),
    parallel_tools: 2,
    tools: TOOLS,
};
