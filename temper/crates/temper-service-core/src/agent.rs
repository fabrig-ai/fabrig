//! The worker agent's declaration to the engine (temper/docs/design/protocol/services.md, 3).
use alloc::boxed::Box;

/// The allowances and names an engine checks, without the worker's host profile.
#[derive(Clone, Debug)]
pub struct Declaration {
    /// The host's charter allowance, in bytes.
    pub charter_bytes: u64,
    /// The declared tool payload allowance, in bytes.
    pub tool_payload: u32,
    /// The host's tool-call input allowance, in bytes.
    pub host_ask_input: u32,
    /// The host's reply text allowance, in bytes.
    pub host_reply_text: u32,
    /// The largest encoded turn body, in bytes.
    pub turn_body: u32,
    /// The host's outcome allowance, in bytes.
    pub outcome_bytes: u64,
    /// Receiving run bounds, including the protocol's measured schema costs.
    pub run_limits: smith_domain_run::Limits,
    /// The adapters' names declared by the worker's programs.
    pub programs: Box<[Box<[u8]>]>,
    /// The configured endpoint names the engine's checks resolve.
    pub endpoints: Box<[Box<[u8]>]>,
    /// Jig's advertised receiving and run bounds.
    pub bounds: jig_domain_link::Bounds,
}
