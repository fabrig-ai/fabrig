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
    /// The caller supplied an invalid tool value; no call starts.
    Invalid,

    /// A local tool or client bound refused the value; no call starts.
    Limit,

    /// Native preparation refused an exact path cap, preserved for session policy.
    PathLimit { which: llm::Cap, bound: u64 },

    /// The configured dialect cannot represent the value; no call starts.
    Unsupported,
}

impl Error {
    /// Preserve the component's existing refusal class at the client boundary.
    /// Smith ownership limits are distinct from the skein Client's caps.
    #[must_use]
    pub const fn from_client(error: skein_llm::Error) -> Self {
        match error {
            skein_llm::Error::Invalid => Self::Invalid,
            skein_llm::Error::Limit { which, bound } => Self::PathLimit { which: crate::failure::cap(which), bound },
            skein_llm::Error::Unsupported => Self::Unsupported,
        }
    }
}

/// Tool identity used by the LLM and program backends, never inferred from JSON Schema.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToolKind {
    /// A concrete checkout operation, offered only under its existing family grant.
    Owned(
        /// Exact checkout operation decoded from the client's arguments.
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
