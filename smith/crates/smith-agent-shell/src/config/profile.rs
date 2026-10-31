//! Profile overrides checked before startup (smith/docs/design/protocol/agent.md, section 4).

use super::{ConfigurationRefusal, DeclaredOverrides, PolicyOverrides, ProfileDocument, duration};
use smith_agent_service::{
    self as service,
    profile::{Layer, LimitName},
};

macro_rules! positive {
    ($profile:expr, $overrides:expr, $prefix:literal, $($field:ident),+ $(,)?) => {
        $(if let Some(value) = $overrides.$field {
            if value == 0 {
                return Err(concat!($prefix, stringify!($field), " must be positive").into());
            }
            $profile.$field = value;
        })+
    };
}

pub(super) fn build_profile(document: &ProfileDocument) -> Result<service::profile::Profile, ConfigurationRefusal> {
    if document.name != "standard" {
        return Err("unknown profile name".into());
    }
    let mut profile = service::profile::standard();
    apply_declared(&mut profile, &document.declared)?;
    apply_counts(&mut profile, &document.policy)?;
    apply_durations(&mut profile, &document.policy)?;
    Ok(profile)
}

fn apply_declared(
    profile: &mut service::profile::Profile,
    overrides: &DeclaredOverrides,
) -> Result<(), ConfigurationRefusal> {
    positive!(
        profile.declared,
        overrides,
        "profile.declared.",
        conversations,
        tool_payload,
        file_bytes,
        calls_per_response,
        read_window,
        shell_output,
        shell_head,
        shell_tail,
        search_hits,
        search_bytes,
        list_entries,
        guide,
        llm_pool,
        memory
    );
    if let Some(value) = overrides.jobs {
        profile.declared.jobs = value;
    }
    Ok(())
}

fn apply_counts(
    profile: &mut service::profile::Profile,
    overrides: &PolicyOverrides,
) -> Result<(), ConfigurationRefusal> {
    positive!(
        profile.policy,
        overrides,
        "profile.policy.",
        max_turns,
        max_spend,
        sections,
        host_tools,
        verdicts,
        items,
        fields,
        inbox,
        unacknowledged
    );
    if let Some(value) = overrides.parallel_tools {
        profile.policy.parallel_tools = value;
    }
    if let Some(value) = &overrides.compaction_threshold {
        if value.numerator == 0 || value.numerator >= value.denominator {
            return Err("profile.policy.compaction_threshold must be between zero and one".into());
        }
        profile.policy.compaction_threshold =
            service::profile::Fraction { numerator: value.numerator, denominator: value.denominator };
    }
    Ok(())
}

fn apply_durations(
    profile: &mut service::profile::Profile,
    overrides: &PolicyOverrides,
) -> Result<(), ConfigurationRefusal> {
    if let Some(value) = overrides.max_waiting {
        profile.policy.max_waiting = duration(value, LimitName::deployment(Layer::Policy, "max_waiting"))?;
    }
    if let Some(value) = overrides.max_time {
        profile.policy.max_time = duration(value, LimitName::deployment(Layer::Policy, "max_time"))?;
    }
    if let Some(value) = overrides.check_timeout {
        profile.policy.check_timeout = duration(value, LimitName::deployment(Layer::Policy, "check_timeout"))?;
    }
    if let Some(value) = overrides.wrap_up {
        profile.policy.wrap_up = duration(value, LimitName::deployment(Layer::Policy, "wrap_up"))?;
    }
    if let Some(value) = overrides.connection_keep {
        profile.policy.connection_keep = duration(value, LimitName::deployment(Layer::Policy, "connection_keep"))?;
    }
    if let Some(value) = overrides.tool_deadline {
        profile.policy.tool_deadline = duration(value, LimitName::deployment(Layer::Policy, "tool_deadline"))?;
    }
    if let Some(value) = overrides.shell_timeout {
        profile.policy.shell_timeout = duration(value, LimitName::deployment(Layer::Policy, "shell_timeout"))?;
    }
    if let Some(value) = overrides.job_poll {
        profile.policy.job_poll = duration(value, LimitName::deployment(Layer::Policy, "job_poll"))?;
    }
    if let Some(value) = overrides.job_wait {
        profile.policy.job_wait = duration(value, LimitName::deployment(Layer::Policy, "job_wait"))?;
    }
    if let Some(value) = overrides.group_stop {
        profile.policy.group_stop = duration(value, LimitName::deployment(Layer::Policy, "group_stop"))?;
    }
    if let Some(value) = overrides.close {
        profile.policy.close = duration(value, LimitName::deployment(Layer::Policy, "close"))?;
    }
    if let Some(value) = overrides.write_deadline {
        profile.policy.write_deadline = duration(value, LimitName::deployment(Layer::Policy, "write_deadline"))?;
    }
    if let Some(value) = overrides.cancel_grace {
        profile.policy.cancel_grace = duration(value, LimitName::deployment(Layer::Policy, "cancel_grace"))?;
    }
    if let Some(value) = overrides.refresh_lead {
        profile.policy.refresh_lead = duration(value, LimitName::deployment(Layer::Policy, "refresh_lead"))?;
    }
    if let Some(value) = overrides.exit_grace {
        profile.policy.exit_grace = duration(value, LimitName::deployment(Layer::Policy, "exit_grace"))?;
    }
    Ok(())
}
