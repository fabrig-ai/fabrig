//! Agents setting validation (temper/docs/design/protocol/configuration/figures.md, 5).
use super::{AgentFigures, AgentLimits, Refusal};

/// Smith's document's integer overrides and their inclusive maxima. Rust exposes
/// no member reflection; a focused test compares these to smith's declarations.
pub const PROFILE_FIGURES: &[(&[u8], u64)] = &[
    (b"declared.charter_models", 4_294_967_295),
    (b"declared.conversations", 4_294_967_295),
    (b"declared.tool_payload", 4_294_967_295),
    (b"declared.file_bytes", 4_294_967_295),
    (b"declared.calls_per_response", 4_294_967_295),
    (b"declared.jobs", 4_294_967_295),
    (b"declared.read_window", 4_294_967_295),
    (b"declared.shell_output", 4_294_967_295),
    (b"declared.shell_head", 4_294_967_295),
    (b"declared.shell_tail", 4_294_967_295),
    (b"declared.search_hits", 4_294_967_295),
    (b"declared.search_bytes", 4_294_967_295),
    (b"declared.list_entries", 4_294_967_295),
    (b"declared.guide", 4_294_967_295),
    (b"declared.llm_pool", u64::MAX),
    (b"declared.memory", u64::MAX),
    (b"policy.max_turns", 4_294_967_295),
    (b"policy.max_spend", u64::MAX),
    (b"policy.max_time", u64::MAX),
    (b"policy.max_waiting", u64::MAX),
    (b"policy.sections", 4_294_967_295),
    (b"policy.host_tools", 4_294_967_295),
    (b"policy.verdicts", 4_294_967_295),
    (b"policy.items", 4_294_967_295),
    (b"policy.fields", 4_294_967_295),
    (b"policy.inbox", 4_294_967_295),
    (b"policy.unacknowledged", 4_294_967_295),
    (b"policy.parallel_tools", 4_294_967_295),
    (b"policy.connection_keep", u64::MAX),
    (b"policy.tool_deadline", u64::MAX),
    (b"policy.shell_timeout", u64::MAX),
    (b"policy.job_poll", u64::MAX),
    (b"policy.job_wait", u64::MAX),
    (b"policy.check_timeout", u64::MAX),
    (b"policy.wrap_up", u64::MAX),
    (b"policy.group_stop", u64::MAX),
    (b"policy.close", u64::MAX),
    (b"policy.write_deadline", u64::MAX),
    (b"policy.cancel_grace", u64::MAX),
    (b"policy.exit_grace", u64::MAX),
    (b"policy.refresh_lead", u64::MAX),
];

pub(super) fn judge(value: &AgentFigures) -> Result<(), Refusal> {
    for (at, figure) in value.profile.iter().enumerate() {
        let name = figure.name.as_ref();
        let mut maximum = None;
        for &(known, most) in PROFILE_FIGURES {
            if name == known {
                maximum = Some(most);
            }
        }
        let Some(maximum) = maximum else {
            return Err(refuse(name, b"unknown profile figure"));
        };
        if figure.value > maximum {
            return Err(refuse(name, b"past the figure's type"));
        }
        for prior in value.profile.iter().take(at) {
            if prior.name.as_ref() == name {
                return Err(refuse(name, b"profile name given twice"));
            }
        }
    }
    for (at, endpoint) in value.endpoints.iter().enumerate() {
        for prior in value.endpoints.iter().take(at) {
            if prior.endpoint == endpoint.endpoint {
                return Err(refuse(&endpoint.endpoint, b"endpoint named twice"));
            }
        }
        limits(&endpoint.endpoint, &endpoint.limits)?;
    }
    limits(b"fallback", &value.fallback)
}
fn limits(endpoint: &[u8], value: &AgentLimits) -> Result<(), Refusal> {
    for (name, amount) in [
        (b"window".as_slice(), u64::from(value.window)),
        (b"output", u64::from(value.output)),
        (b"reasoning_item", u64::from(value.reasoning_item)),
        (b"head_ms", value.head_ms),
        (b"idle_ms", value.idle_ms),
    ] {
        if amount == 0 {
            let mut item = skein_lib::List::uncapped();
            for part in [endpoint, b".", name] {
                for &byte in part {
                    item.push(byte).expect("bounded item name");
                }
            }
            return Err(Refusal::Agent { name: item.into_boxed(), rule: b"must be positive" });
        }
    }
    Ok(())
}
fn refuse(name: &[u8], rule: &'static [u8]) -> Refusal {
    Refusal::Agent { name: name.into(), rule }
}
