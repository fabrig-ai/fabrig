//! Host bounds derived from the agent profile, never a chat or run charter.
//! `host_limits` grants a window large enough for the agent's largest turn.
//! Contracts: smith/docs/design/shell.md, sections 2.1 and 2.6;
//! smith/docs/design/protocol/limits.md, sections 3.6 and 3.8.

use crate::profile::{Check, Layer, LimitName, Refusal, Value};
use skein_lib::Duration;
use smith_host_domain as host;

const HOST_FACTS: u32 = 1024;

/// Build the standard host bounds around the largest turn its agent can emit.
pub fn host_limits(profile: &crate::profile::Profile, agent: &crate::Limits) -> Result<host::Limits, Refusal> {
    let run = agent.domain.run;
    let largest_turn = smith_domain::max_turn_bytes(&agent.domain)
        .ok_or(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "turn_bytes") })?;
    // Concrete history and turn payloads share one window's aggregate bound,
    // rather than charging each reserved turn at the largest payload (rule 5).
    let answer_bytes = u64::from(run.host_reply_bytes).max(host::Delivered::owned_bound()).max(
        u64::try_from(host::Marker::CAPACITY + host::DeliveryRefusal::CAPACITY).expect("sealed feedback fits u64"),
    );
    let answered_bytes = u64::from(run.answered_calls)
        .checked_mul(
            u64::from(agent.channel.bodies.answered_call_tool)
                .checked_add(answer_bytes)
                .ok_or(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "answered_bytes") })?,
        )
        .ok_or(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "answered_bytes") })?;
    let fact_bytes = smith_channel::Fact::encoded_bound(&agent.channel.bodies)
        .ok_or(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "fact_bytes") })?;
    let name = LimitName::deployment(Layer::Service, "silence");
    let silence = crate::profile::silence_bound(agent).ok_or(Refusal::Derived { limit: name })?;
    let no_progress = silence
        .checked_add(agent.io.close_timeout)
        .ok_or(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "no_progress") })?;
    let teardown_name = LimitName::deployment(Layer::Service, "teardown");
    let teardown = crate::profile::teardown_bound(agent).ok_or(Refusal::Derived { limit: teardown_name })?;
    if profile.policy.exit_grace <= teardown {
        return Err(Refusal::Broken {
            check: Check::GraceOverTeardown,
            left: LimitName::deployment(Layer::Policy, "exit_grace").value(Value::Duration(profile.policy.exit_grace)),
            right: teardown_name.value(Value::Duration(teardown)),
        });
    }
    let exit_grace = teardown
        .checked_add(agent.io.close_timeout)
        .ok_or(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "exit_grace") })?;
    let limits = host::Limits {
        agents: run.runs,
        directories: run.directories,
        conflicts: run.conflicts,
        path_bytes: run.host_path_bytes.max(run.conflict_path_bytes),
        name_bytes: run.directory_name_bytes.max(agent.channel.bodies.host_ask_tool),
        accounts: agent.domain.accounts,
        charter_bytes: run.run_bytes,
        transcript_bytes: largest_turn,
        answered_bytes,
        message_bytes: u64::from(run.message_bytes),
        messages: run.messages,
        calls: agent.channel.calls,
        call_bytes: u64::from(run.host_input_bytes).max(run.outcome_bytes),
        answer_bytes,
        turns: agent.channel.turns,
        turn_bytes: largest_turn,
        unacknowledged_bytes: largest_turn,
        fact_bytes,
        outcome_bytes: run.outcome_bytes,
        detail_bytes: agent.domain.session.failure_bytes,
        spawn_timeout: Duration::from_secs(20),
        no_progress,
        silence,
        long_span: profile.policy.max_time,
        long_open: profile
            .declared
            .conversations
            .checked_mul(profile.declared.calls_per_response.checked_add(1).expect("validated calls bound"))
            .expect("validated simultaneous provider and command bounds")
            .checked_add(1)
            .expect("validated check span bound"),
        wall_time: profile.policy.max_time,
        cancel_grace: profile.policy.cancel_grace,
        exit_grace,
        teardown,
        kill_after: Duration::from_secs(2),
        facts: HOST_FACTS,
    };
    match host::check(&limits) {
        Ok(()) => Ok(limits),
        Err(host::Check::GraceOverTeardown { exit_grace, teardown }) => Err(Refusal::Broken {
            check: Check::GraceOverTeardown,
            left: LimitName::deployment(Layer::Service, "exit_grace").value(Value::Duration(exit_grace)),
            right: teardown_name.value(Value::Duration(teardown)),
        }),
        Err(host::Check::WatchdogOverSilence { no_progress, silence }) => Err(Refusal::Broken {
            check: Check::WatchdogOverSilence,
            left: LimitName::deployment(Layer::Service, "no_progress").value(Value::Duration(no_progress)),
            right: name.value(Value::Duration(silence)),
        }),
    }
}
