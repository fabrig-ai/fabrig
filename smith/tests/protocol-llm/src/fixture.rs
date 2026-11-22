//! Shared protocol story setup (smith/docs/design/protocol/llm.md, section 10).
use smith_protocol_llm::{ComponentLimits, Limits};

// Fault stories carry one scalar progress result or a four-byte final report.
// Their transport need not fragment those records into codec-test-sized pieces.
const FAULT_PACKET_BYTES: u32 = 4096;
const FAULT_REQUEST_BYTES: u32 = 16_384;
const FAULT_REPORT_BYTES: u32 = 4;

/// The protocol world's client and workspace description preset.
#[must_use]
pub fn limits() -> Limits {
    smith_protocol_llm_fixtures::small(skein_llm_kit::limits(), crate::codec::stated())
}

/// The same component capacities for manual and scripted peers.
#[must_use]
pub fn component() -> ComponentLimits {
    smith_protocol_llm_fixtures::component(limits(), smith_agent_service::profile::standard().policy.connection_keep)
        .expect("fixture receiving bound")
}

/// The shared agent settings with protocol token pricing and model window.
#[must_use]
pub fn settings(seed: u64) -> smith_agent_world::Settings {
    smith_agent_world::Settings {
        window: crate::MODEL_WINDOW,
        limits: crate::LIMITS,
        ..smith_agent_world::Settings::calm(seed)
    }
}

/// A finite fault run with one retry, a "done" report, and caller-selected tools.
/// A read-only workspace exists only when the story needs inspection.
#[must_use]
pub fn provider_fault_world(
    seed: u64,
    configuration: Configuration,
    turns: Box<[skein_fake_llm_domain::api::Turn]>,
    inspect: bool,
    host_tools: Box<[smith_domain::run::HostTool]>,
) -> crate::World {
    let mut settings = settings(seed);
    settings.job = smith_agent_world::Job::Reporting;
    settings.writable = false;
    // One retry shows backoff and typed exhaustion; no story needs a third attempt.
    settings.limits.session.retries = 1;
    let mut charter = smith_agent_world::scripted_charter(&settings);
    charter.grants.tools.inspect = inspect;
    charter.grants.host_tools = host_tools;
    charter.grants.agents = false;
    charter.grants.wait = false;
    charter.models = Box::new([]);
    charter.conventions = None;
    // These stories finish with the four-byte report "done", without custom fields.
    let report = charter.outcome.report.as_mut().expect("reporting contract");
    report.max = FAULT_REPORT_BYTES;
    report.fields = Box::new([]);
    let mut disk = skein_fake_checkout::Checkout::new();
    let workspace = inspect.then(|| {
        disk.mkdir(b"work");
        disk.write(b"work/src/lib.rs", b"pub fn answer() -> u32 { 42 }\n");
        smith_domain::run::Workspace {
            directories: Box::new([smith_domain::run::Directory {
                root: skein_lib::Token::new(disk.root(b"work")),
                host: b"/host".as_slice().into(),
                name: b"work".as_slice().into(),
                writable: false,
                git: false,
                conflicts: Box::new([]),
            }]),
        }
    });
    let mut limits = limits();
    limits.client.request = FAULT_REQUEST_BYTES;
    limits.client.http.request = FAULT_REQUEST_BYTES;
    // Faults are provider responses and holds, not transport fragmentation.
    // Send a complete request in one grant instead of the codec fixture's 31-byte pieces.
    limits.client.http.send = limits.client.http.request;
    limits.client.http.read = FAULT_PACKET_BYTES;
    limits.client.sse.chunk = FAULT_PACKET_BYTES;
    let receiving = smith_protocol_llm_fixtures::receiving(&limits.client, settings.limits.decoded_call_bytes)
        .expect("provider receiving bounds");
    settings.limits.session.completion_bytes = receiving.max_completion_bytes;
    settings.limits.session.completion_blocks = receiving.max_completion_blocks;
    settings.limits.session.failure_bytes = receiving.max_failure_bytes;
    crate::World::with_workspace_protocol_charter(
        settings,
        None,
        workspace,
        disk,
        (configuration, limits),
        Box::new([skein_fake_llm_domain::api::Script { cue: b"@report".as_slice().into(), turns }]),
        charter,
    )
}

/// The finite report shared by the provider-fault stories.
#[must_use]
pub fn fault_finish() -> skein_fake_llm_domain::api::Turn {
    calls(
        Box::new([skein_fake_llm_domain::api::Line::Call {
            name: b"finish".as_slice().into(),
            arguments: br#"{"form":"report","text":"done","fields":{}}"#.as_slice().into(),
        }]),
        7,
    )
}

/// Caller-supplied endpoint and credential data for the wire fixture.
/// Caller owns this finite configuration; admission and terminals belong to Wire.
#[expect(missing_debug_implementations, reason = "fixture owns caller credential bytes")]
pub struct Configuration {
    /// Whole shared endpoint configuration, supplied as neutral caller data.
    pub endpoint: skein_llm::Endpoint,
    /// Fields emitted by the shared peer, preserving absent native counts.
    pub usage_fields: skein_fake_llm_protocol::documents::UsageFields,
    /// Observed grant supplied to one Client and separately to its independent peer.
    pub credential: skein_llm::Credential,
    /// Independent outside expectation for a native error result's text prefix.
    pub error_prefix: Box<[u8]>,
    /// Independent outside expectation for a native error result's parsed flag.
    pub error_flag: bool,
    /// Handwritten outside expectation for this fixture's continued arguments.
    /// Incoming delta bytes remain exact at the root. A native embedded-object
    /// continuation may serialize object whitespace while preserving every field.
    pub continuation_arguments: Box<[u8]>,
}

/// Native configurations are shared kit data, with no Smith dialect branching.
/// Returns two caller fixtures; it starts no effect and carries outside literal
/// expectations.
#[must_use]
pub fn configurations() -> Box<[Configuration]> {
    Dialect::ALL.into_iter().map(Dialect::configuration).collect()
}

/// The named wire dialects exercised by protocol stories.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dialect {
    /// Synthetic public `OpenAI` Responses grammar and replay envelopes.
    OpenAi,
    /// Anthropic's Messages grammar and signed thinking.
    Anthropic,
}
impl Dialect {
    /// The supported fixture dialects.
    pub const ALL: [Self; 2] = [Self::OpenAi, Self::Anthropic];
    /// Endpoint, grant and independent expectations for this dialect.
    #[must_use]
    pub fn configuration(self) -> Configuration {
        match self {
            Self::OpenAi => Configuration {
                endpoint: skein_llm::Endpoint::openai(),
                usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
                credential: skein_llm::Credential::bearer(b"fixture-token".as_slice().into()),
                error_prefix: b"Error: ".as_slice().into(),
                error_flag: false,
                continuation_arguments:
                    br#"{ "opaque" : {"future":[1,true,null]}, "extra":"unchanged" }"#.as_slice().into(),
            },
            Self::Anthropic => Configuration {
                endpoint: skein_llm::Endpoint::anthropic(),
                usage_fields: skein_fake_llm_protocol::documents::UsageFields::ALL,
                credential: skein_llm::Credential::key(b"fixture-token".as_slice().into()),
                error_prefix: Box::new([]),
                error_flag: true,
                continuation_arguments: br#"{"opaque":{"future":[1,true,null]},"extra":"unchanged"}"#.as_slice().into(),
            },
        }
    }
    /// The fake peer's independent optional reasoning observation.
    #[must_use]
    pub const fn reasoning(self) -> Option<u64> {
        match self {
            Self::OpenAi => Some(0),
            Self::Anthropic => None,
        }
    }
}

/// Scripted provider turns shared with the typed agent world.
pub use smith_agent_world::fixture::{calls, says};
