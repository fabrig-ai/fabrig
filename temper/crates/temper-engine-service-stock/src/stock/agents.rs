//! Stock agents figures (temper/docs/design/protocol/configuration/figures.md, 5).
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

pub const AGENT_PROFILE: &[(&[u8], u64)] = &[
    (b"declared.memory", 2_147_483_648),
    (b"declared.tool_payload", 10_662),
    (b"declared.read_window", 256),
    (b"declared.conversations", 26),
    (b"declared.calls_per_response", 48),
    (b"declared.charter_models", 64),
    (b"policy.max_turns", 800),
    (b"policy.check_timeout", 60_000),
    (b"policy.group_stop", 10),
    (b"policy.inbox", 82),
    (b"policy.unacknowledged", 8),
];
/// Window, output, reasoning-item bytes, head milliseconds, idle milliseconds.
pub const AGENT_FALLBACK: (u32, u32, u32, u64, u64) = (1_048_576, 8192, 65_536, 60_000, 300_000);
type EndpointFigures = (u32, u32, u32, u64, u64);
pub const AGENT_ENDPOINTS: &[(&[u8], EndpointFigures)] = &[
    (b"openai", AGENT_FALLBACK),
    (b"anthropic", (1_048_576, 8192, 65_536, 60_000, 120_000)),
    (b"anthropic-api", (1_048_576, 8192, 65_536, 60_000, 120_000)),
    (b"demo", (1_048_576, 8192, 2048, 60_000, 30_000)),
];
#[must_use]
pub fn agents() -> w::AgentFigures {
    let mut profile = List::with_capacity(128);
    for &(name, value) in AGENT_PROFILE {
        profile
            .push(
                w::AgentFigure::new(&w::CEILINGS, w::AgentFigureParts { name: name.into(), value })
                    .expect("stock figure"),
            )
            .expect("stock profile bound");
    }
    let mut endpoints = List::with_capacity(64);
    for &(endpoint, figures) in AGENT_ENDPOINTS {
        endpoints
            .push(
                w::AgentEndpointLimits::new(
                    &w::CEILINGS,
                    w::AgentEndpointLimitsParts { endpoint: endpoint.into(), limits: limits(figures) },
                )
                .expect("stock endpoint"),
            )
            .expect("stock endpoint bound");
    }
    w::AgentFigures::new(&w::CEILINGS, w::AgentFiguresParts { profile, endpoints, fallback: limits(AGENT_FALLBACK) })
        .expect("stock agents bounds")
}
fn limits((window, output, reasoning_item, head_ms, idle_ms): (u32, u32, u32, u64, u64)) -> w::AgentLimits {
    w::AgentLimits::new(
        &w::CEILINGS,
        w::AgentLimitsParts { window, output, reasoning_item, head_ms, idle_ms, oversized: w::Oversized::Fail },
    )
    .expect("fixed limits fields")
}
