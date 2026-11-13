//! Small agent admission bounds (smith/docs/design/domain/run.md, section 12).
use crate::{Limits, run, session, tools};
use alloc::boxed::Box;
use skein_lib::Duration;

const CEILING: session::Budget = session::Budget {
    turns: 100,
    input: 1_000_000,
    output: 100_000,
    cache_read: 1_000_000,
    cache_write: 1_000_000,
    time: Duration::from_secs(3600),
};

/// One small agent profile with bounded run, session and tools entrances.
pub const SMALL: Limits = Limits {
    accounts: 4,
    endpoints: 3,
    configured_endpoint_bytes: 4096,
    content_edge_records: 128,
    decoded_call_bytes: 4096,
    skew: Duration::ZERO,
    run: run::Limits {
        schema_costs: run::SchemaCosts::NONE,
        directory_name_bytes: 64,
        calls: 4,
        budget: run::Budget { turns: CEILING.turns, spend: 10_000, time: CEILING.time },
        models: 1,
        answer_bytes: 64,
        nudges: 1,
        check_tail: 256,
        ..smith_domain_run_fixtures::SMALL
    },
    session: session::Limits {
        sessions: 4,
        spend: 10_000,
        protocol_allowance: 4096,
        history_messages: 16,
        history_bytes: 2_097_152,
        delegated_result_bytes: 262_144,
        budget: CEILING,
        retries: 1,
        tools: tools::Limits {
            shell_output: 128,
            kits: 4,
            repos: 2,
            file_bytes: 65_536,
            tool_payload: 65_536,
            read_bytes: 4096,
            file_timeout: Duration::from_secs(60),
            shell_tail: 64,
            ..smith_domain_tools_fixtures::SMALL
        },
        ..smith_domain_session_fixtures::SMALL
    },
};

/// A fresh host activation with unbounded test credit and no supplied history or grants.
#[must_use]
pub fn start(call: skein_lib::Token, charter: run::Charter) -> crate::Start {
    crate::Start {
        reply_to: skein_lib::ReplyTo::new(call),
        host_run: call,
        activation: 1,
        identity: None,
        window: crate::Window { turns: u32::MAX, bytes: u64::MAX },
        messages: Box::default(),
        charter,
        workspace: None,
        transcript: None,
        answered: Box::default(),
        grants: Box::default(),
    }
}
