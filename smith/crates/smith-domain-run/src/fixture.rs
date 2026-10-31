//! Small run limits for deterministic fixtures.
//! Contract: smith/docs/design/domain/run.md, section 13.

use skein_lib::Duration;

use crate::{Budget, Limits};

/// Small admission and ownership bounds; tests override the bound they exercise.
pub const SMALL: Limits = Limits {
    runs: 2,
    conversations: 4,
    run_bytes: 4096,
    brief_sections: 4,
    host_part: 1 << 20,
    room: 0,
    compaction_threshold: [4, 5],
    compaction_material: 0,
    schema_costs: crate::SchemaCosts::NONE,
    directories: 2,
    directory_name_bytes: 256,
    host_path_bytes: 4096,
    conflicts: 64,
    conflict_path_bytes: 4096,
    host_tools: 2,
    host_input_bytes: 65_536,
    host_reply_bytes: 65_536,
    answered_calls: 16,
    answered_bytes: 4096,
    host_timeout: Duration::from_secs(60),
    host_timeout_max: Duration::from_secs(60),
    host_backoff: Duration::from_millis(50),
    verdicts: 2,
    calls: 2,
    budget: Budget { turns: 100, spend: 1_000_000, time: Duration::from_secs(3600) },
    models: 2,
    run_conversations: 3,
    answer_bytes: 16,
    nudges: 2,
    guide_bytes: 64,
    io_timeout: Duration::from_secs(10),
    outcome_bytes: 1024,
    delivery_timeout: Duration::from_secs(300),
    check_timeout: Duration::from_secs(300),
    wrap_up: Duration::from_secs(60),
    check_tail: 4096,
    messages: 8,
    message_bytes: 4096,
    offer_messages: 8,
    offer_bytes: 32_782,
    waiting: Duration::from_secs(300),
};
