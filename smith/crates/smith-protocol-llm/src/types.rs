//! Bounded caller-owned codec data and one observed Client's translation context.
//! No borrowed tool value survives an entry point.
//! Contract: protocol/llm.md, sections 3, 4 and 6.

use alloc::boxed::Box;

use skein_lib::Token;
use smith_domain::{llm, run, tools};

use crate::{Limits, Receiving};

/// A preparation refusal reported by the protocol component to its caller.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The request breaks a local or dialect rule; no call starts.
    Invalid { cause: Cause },

    /// A local tool or client bound refused the value; no call starts.
    Limit,

    /// Native preparation refused an exact path cap, preserved for session policy.
    PathLimit { which: llm::Cap, bound: u64 },

    /// The configured dialect cannot represent the value; no call starts.
    Unsupported { cause: Cause },
}

impl Error {
    /// Preserve the component's existing refusal class at the client boundary.
    /// Smith ownership limits are distinct from the skein Client's caps.
    #[must_use]
    pub const fn from_client(error: skein_llm::Error) -> Self {
        match error {
            skein_llm::Error::Invalid { cause } => Self::Invalid { cause: Cause::Invalid(cause) },
            skein_llm::Error::Limit { which, bound } => Self::PathLimit { which: crate::failure::cap(which), bound },
            skein_llm::Error::Unsupported { cause } => Self::Unsupported { cause: Cause::Unsupported(cause) },
        }
    }
}

/// A refusal's cause, owned here or retained from skein's admission boundary.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    /// Smith cannot build the request.
    Smith(Refusal),
    /// The request breaks its dialect's rule.
    Invalid(skein_llm::Invalid),
    /// The dialect cannot carry the requested value.
    Unsupported(skein_llm::Unsupported),
}

impl Cause {
    /// The fixed local name reported to the owner, never a provider's text.
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::Smith(cause) => cause.name(),
            Self::Invalid(cause) => cause.name(),
            Self::Unsupported(cause) => cause.name(),
        }
    }
}

/// Smith's closed request refusals (protocol/llm.md, section 6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    /// The chosen tool is not among those served.
    ChoiceNotServed,
    /// Two tools are served under one name.
    ToolName,
    /// A host schema is not a plain object.
    HostSchema,
    /// A schema or input is not a JSON object.
    NotObject,
    /// Delivery is offered without its contract.
    DeliverContract,
    /// Failed-write feedback has no preceding call to follow.
    FailedWrite,
    /// A result's error flag differs from its feedback's.
    Feedback,
    /// A tool result carries unsupported provider replay.
    ResultReplay,
    /// A read is rendered from line zero.
    ReadLine,
    /// The guard's start or measured sizes are out of order.
    Guard,
    /// The observation cursor is past the history.
    Observation,
    /// The fixed empty schema was refused.
    FixedSchema,
    /// A completion holds a tool result.
    CompletionResult,
    /// The connection does not hold the named endpoint.
    UnknownEndpoint,
    /// The owner is already calling.
    Owner,
    /// The grant is out of bounds, stale, full or of another scheme.
    Grant,
}

impl Refusal {
    /// The design's fixed name, disjoint from skein's admission causes.
    #[must_use]
    pub const fn name(self) -> &'static [u8] {
        match self {
            Self::ChoiceNotServed => b"choice-not-served",
            Self::ToolName => b"tool-name",
            Self::HostSchema => b"host-schema",
            Self::NotObject => b"not-object",
            Self::DeliverContract => b"deliver-contract",
            Self::FailedWrite => b"failed-write",
            Self::Feedback => b"feedback",
            Self::ResultReplay => b"result-replay",
            Self::ReadLine => b"read-line",
            Self::Guard => b"guard",
            Self::Observation => b"observation",
            Self::FixedSchema => b"fixed-schema",
            Self::CompletionResult => b"completion-result",
            Self::UnknownEndpoint => b"unknown-endpoint",
            Self::Owner => b"owner",
            Self::Grant => b"grant",
        }
    }
}

/// Tool identity used by the LLM and program backends, never inferred from JSON Schema.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToolKind {
    /// Main-only unchecked saving descriptor.
    Save,
    /// A concrete workspace operation, offered only under its existing family grant.
    Owned(
        /// Exact workspace operation decoded from the client's arguments.
        tools::Tool,
    ),

    /// The run's offered finish descriptor.
    Finish,

    /// The separately granted mid-run delivery descriptor.
    Deliver,

    /// The run's offered child-conversation descriptor.
    SubAgent,

    /// The main conversation's offered wait descriptor.
    Wait,
}

/// One whole generated schema for a concrete Smith tool.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ToolSchema {
    /// Semantic classification checked against the prompt's observed grants/descriptors.
    pub kind: ToolKind,

    /// Exact nonempty tool name, unique across tool and host declarations.
    pub name: Box<[u8]>,

    /// Whole tool description passed to the skein Client.
    pub description: Box<[u8]>,

    /// Whole JSON Schema object bytes; syntax is attested by Skein JSON before preparation.
    /// This component never validates arguments against the schema's meaning.
    pub schema: Box<[u8]>,
}

/// Grants, result rules and receiving bounds retained for decoding one client terminal.
/// The component releases this context when it returns the call's one terminal.
#[derive(Debug)]
pub struct Context {
    pub(crate) owner: Token,
    pub(crate) asker: llm::Asker,
    pub(crate) grants: tools::Grants,
    pub(crate) served: Box<[llm::Served]>,
    pub(crate) outcome: run::outcome::OutcomeSpec,
    pub(crate) deliver: Option<run::outcome::ChangeSpec>,
    pub(crate) receiving: Receiving,
    pub(crate) limits: Limits,
}
