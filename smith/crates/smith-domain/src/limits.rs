//! Admission limits and checked capacities. This module retains no runtime state.
//! `valid` checks compatibility and constructor capacities, without knowing live ownership.
//!
//! Contract: smith/docs/design/domain/run.md, section 13; skein/docs/foundation/programming-model.md, sections 4.4, 4.5 and 6.3.

use smith_domain_run as run;
use smith_domain_session as session;

/// The agent domain's limits (skein/docs/foundation/programming-model.md, sections 4.5 and 6.3), handed to every step read-only: its
/// child domains', each handed down to the one it bounds. The session's include
/// its tools'.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Maximum configured credential accounts retained by the root.
    pub accounts: u32,
    /// Maximum endpoint names in the agent configuration.
    pub endpoints: u32,
    /// Owned configured model cells and names, derived from startup declarations.
    pub configured_model_bytes: u64,

    /// Aggregate decoded application-call bytes admitted from one completion.
    /// The adapter includes these owning call fields in its translated completion
    /// bound before provider work; oversized classifications become `TooLarge`.
    /// This includes one Decoded cell for every possible completion block,
    /// including classifications with no payload after a refusal.
    pub decoded_call_bytes: u64,

    /// Optional capture capacity in records; overflow drops records and counts them.
    pub content_edge_records: u32,

    /// Safety margin subtracted from credential validity before use.
    pub skew: skein_lib::Duration,
    /// Immutable run-child ownership and admission limits.
    pub run: run::Limits,
    /// Immutable session-child ownership and admission limits.
    pub session: session::Limits,
}

/// Whether a domain can be built under `limits`: compatible limits and
/// representable constructor capacities (skein/docs/foundation/programming-model.md, section 6.3).
/// The shell refuses limits for which this is false.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    let run_limits = &limits.run;
    let session_limits = &limits.session;
    let budget = run_limits.budget;
    let ceiling = session_limits.budget;
    let fits = budget.turns <= ceiling.turns && budget.spend <= session_limits.spend && budget.time <= ceiling.time;
    if !fits
        || run_limits.conversations > session_limits.sessions
        || run_limits.directories > session_limits.tools.repos
        || run_limits.directory_name_bytes > session_limits.tools.path_bytes
        || run_limits.host_path_bytes > session_limits.tools.host_path_bytes
        || u64::from(run_limits.answered_bytes) > session_limits.history_bytes
    {
        return false;
    }
    let Some(feedback) = crate::feedback_bound(run_limits) else {
        return false;
    };
    if session_limits.spend == 0 || feedback > session_limits.delegated_result_bytes {
        return false;
    }
    let Some(decoded_cells) = u64::from(session_limits.completion_blocks)
        .checked_mul(u64::try_from(size_of::<crate::llm::Decoded>()).expect("decoded cell size fits u64"))
    else {
        return false;
    };
    if decoded_cells > limits.decoded_call_bytes.min(session_limits.history_bytes) {
        return false;
    }
    flights(limits).is_some()
        && handoffs(limits).is_some()
        && commands(limits).is_some()
        && run::valid(run_limits)
        && session::valid(session_limits)
}

/// Deferred concrete payload outside session while one complete delegate batch
/// owns its pre-effect receiving credit. Queued bytes are separately
/// allocated and priced; no post-effect history-full result is discarded.
#[cfg(test)]
pub(crate) fn uncharged(limits: &Limits) -> Option<u64> {
    u64::from(limits.session.parallel_tools).checked_mul(limits.session.delegated_result_bytes)
}

/// Delegated calls in flight at once: a batch of each session.
pub(crate) fn flights(limits: &Limits) -> Option<u32> {
    limits.run.conversations.checked_mul(limits.session.parallel_tools)
}

/// Hand-offs on the ready list at once: a close of each conversation, and the
/// answer to each delegated call.
pub(crate) fn handoffs(limits: &Limits) -> Option<u32> {
    limits.run.conversations.checked_add(flights(limits)?)
}

/// History payload and separate allocated record/message envelopes. The source
/// session byte cap prices Block cells/payloads, never these arrays.
pub(crate) fn record_payload(limits: &Limits) -> Option<u64> {
    let messages = u64::from(limits.session.history_messages);
    limits
        .session
        .history_bytes
        // Each turn copies its transcript model (128 bytes) and optional effort (32).
        .checked_add(messages.checked_mul(160)?)?
        // One window opener is retained or handed off; ordinary turns have no box.
        .checked_add(u64::try_from(size_of::<session::record::Opener>()).ok()?)?
        .checked_add(messages.checked_mul(u64::try_from(size_of::<session::record::Turn>()).ok()?)?)?
        .checked_add(messages.checked_mul(u64::try_from(size_of::<session::llm::Message>()).ok()?)?)
}

/// Largest owned concrete turn the domain may tell, including allocated record envelopes.
/// A host's acknowledgement window must reserve this much for its next turn.
#[must_use]
pub fn max_turn_bytes(limits: &Limits) -> Option<u64> {
    record_payload(limits)
}

/// One command per live tools call; retired job slots own no IO rights.
pub(crate) fn commands(limits: &Limits) -> Option<u32> {
    limits.session.tools.kits.checked_mul(limits.session.tools.calls)
}
