//! Shared protocol story setup (smith/docs/design/protocol/llm.md, section 10).
use smith_protocol_llm::{ComponentLimits, Limits};

/// The protocol world's client and workspace description preset.
#[must_use]
pub fn limits() -> Limits {
    smith_protocol_llm::fixture::small(skein_llm_world::limits(), crate::codec::stated())
}

/// The same component capacities for manual and scripted peers.
#[must_use]
pub fn component() -> ComponentLimits {
    smith_protocol_llm::fixture::component(limits(), smith_agent_service::profile::standard().policy.connection_keep)
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
    /// Synthetic public OpenAI Responses grammar and replay envelopes.
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
