//! Process limits for the scripted agent world (smith/docs/design/domain/run.md, section 12).
//! The world supplies immutable caps; none is inferred from domain state.

use skein_lib::Duration;
use smith_domain::{Limits, run, session, tools};

const CONTENT_EDGE_RECORDS: u32 = 1024;

/// The most a charter may ask for in the calm world, which every session may
/// take.
pub const BUDGET: run::Budget = run::Budget { turns: 64, spend: 10_000, time: Duration::from_secs(3600) };

const CEILING: session::Budget = session::Budget {
    turns: BUDGET.turns,
    input: 1 << 24,
    output: 1 << 24,
    cache_read: 1 << 26,
    cache_write: 1 << 24,
    time: BUDGET.time,
};

/// An agent process's limits in the calm world: room for the one run it
/// carries, with room for main and several child conversations.
pub const LIMITS: Limits = Limits {
    content_edge_records: CONTENT_EDGE_RECORDS,
    run: run::Limits {
        runs: 1,
        conversations: 6,
        run_bytes: 1 << 16,
        schema_costs: run::SchemaCosts::NONE,
        calls: 16,
        budget: BUDGET,
        run_conversations: 6,
        answer_bytes: 1024,
        nudges: 1,
        guide_bytes: 1024,
        io_timeout: Duration::from_secs(5),
        outcome_bytes: 4096,
        delivery_timeout: Duration::from_secs(60),
        check_timeout: Duration::from_secs(60),
        check_tail: 512,
        ..smith_domain_run_fixtures::SMALL
    },
    session: session::Limits {
        sessions: 6,
        spend: 10_000,
        protocol_allowance: 4096,
        history_messages: 64,
        history_bytes: 33_554_432,
        delegated_result_bytes: 4_194_304,
        budget: CEILING,
        retries: 3,
        backoff_base: Duration::from_millis(200),
        backoff_max: Duration::from_secs(5),
        parallel_tools: 4,
        tools: tools::Limits {
            shell_output: 512,
            kits: 6,
            repos: 2,
            path_bytes: 256,
            known_files: 16,
            file_bytes: 1 << 16,
            tool_payload: 1 << 16,
            read_bytes: 4096,
            list_entries: 64,
            match_lines: 8,
            file_timeout: Duration::from_secs(30),
            shell_timeout: Duration::from_secs(30),
            tool_deadline: Duration::from_secs(120),
            shell_head: 256,
            shell_tail: 256,
            search_hits: 16,
            search_bytes: 1024,
            ..smith_domain_tools_fixtures::SMALL
        },
        ..smith_domain_session_fixtures::SMALL
    },
    ..smith_domain_fixtures::SMALL
};

/// An agent process's limits in random worlds: room for fewer conversations,
/// sessions and calls than its run may ask for, so that some are refused.
pub const TIGHT: Limits = Limits {
    content_edge_records: LIMITS.content_edge_records,
    run: run::Limits {
        conversations: 4,
        calls: 6,
        run_conversations: 4,
        answer_bytes: 256,
        guide_bytes: 256,
        check_tail: 128,
        ..LIMITS.run
    },
    session: session::Limits {
        threshold: [4, 5],
        token_bytes: 8,
        sessions: 4,
        spend: 10_000,
        protocol_allowance: 4096,
        history_blocks: u32::MAX,
        history_messages: 32,
        history_bytes: 33_554_432,
        completion_bytes: 4096,
        answer_bytes: 4096,
        completion_blocks: 16,
        failure_bytes: 512,
        delegated_result_bytes: 4_194_304,
        retries: 2,
        parallel_tools: 3,
        tools: tools::Limits { kits: 4, calls: 3, ..LIMITS.session.tools },
        ..LIMITS.session
    },
    ..LIMITS
};
