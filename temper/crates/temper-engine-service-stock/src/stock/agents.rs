//! Stock agents figures (temper/docs/design/protocol/configuration/figures.md, 5).
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

pub const AGENT_PROFILE: &[(&[u8], u64)] = &[
    (b"declared.memory", 2_147_483_648),
    // Fabrig's own development's tool payload.
    (b"declared.tool_payload", 63_740),
    (b"declared.read_window", 256),
    // Fabrig's own development's 26 conversations, with room for 32.
    (b"declared.conversations", 32),
    // Fabrig's own development's calls per response.
    (b"declared.calls_per_response", 48),
    (b"declared.charter_models", 64),
    (b"policy.max_turns", 800),
    // Fabrig's own development's check timeout in milliseconds.
    (b"policy.check_timeout", 300_000),
    (b"policy.group_stop", 10),
    // Fabrig's own development's inbox.
    (b"policy.inbox", 82),
    (b"policy.unacknowledged", 8),
];
/// Window, output, reasoning-item bytes, head milliseconds, idle milliseconds.
// Fabrig's own development's 48,000-token output, also used by openai.
pub const AGENT_FALLBACK: (u32, u32, u32, u64, u64) = (1_048_576, 48_000, 65_536, 60_000, 300_000);
type EndpointFigures = (u32, u32, u32, u64, u64);
pub const AGENT_ENDPOINTS: &[(&[u8], EndpointFigures)] = &[
    (b"openai", AGENT_FALLBACK),
    // Fabrig's own development's output for each endpoint.
    (b"anthropic", (1_048_576, 48_000, 65_536, 60_000, 120_000)),
    // Fabrig's own development's output for this endpoint.
    (b"anthropic-api", (1_048_576, 48_000, 65_536, 60_000, 120_000)),
    // Fabrig's own development's output for this endpoint.
    (b"demo", (1_048_576, 48_000, 2048, 60_000, 30_000)),
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
