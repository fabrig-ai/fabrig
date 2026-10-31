//! Small supervision bounds for deterministic worlds (domain/host.md, section 10).
use crate::{Delivered, Limits};
use skein_lib::Duration;

/// Small valid capacities; every sealed delivery fits the reply cap.
#[must_use]
pub fn small() -> Limits {
    Limits {
        agents: 1,
        directories: 2,
        conflicts: 2,
        path_bytes: 64,
        name_bytes: 64,
        accounts: 2,
        charter_bytes: 256,
        transcript_bytes: 1024,
        answered_bytes: 256,
        message_bytes: 64,
        messages: 3,
        calls: 2,
        call_bytes: 128,
        answer_bytes: Delivered::owned_bound(),
        turns: 2,
        turn_bytes: 512,
        unacknowledged_bytes: 1024,
        fact_bytes: 64,
        outcome_bytes: 128,
        detail_bytes: 32,
        spawn_timeout: Duration::from_secs(2),
        no_progress: Duration::from_secs(10),
        silence: Duration::ZERO,
        long_span: Duration::from_secs(120),
        long_open: 64,
        wall_time: Duration::from_secs(900),
        cancel_grace: Duration::from_secs(5),
        exit_grace: Duration::from_secs(5),
        teardown: Duration::ZERO,
        kill_after: Duration::from_secs(2),
        facts: 16,
    }
}
