//! Lower routing for an inline run (domain/host.md, sections 9.1 and 9.4).
//! These stateless records retain owned requests and exact callback owners;
//! no operation is answered here and each provider request owes one terminal.

use skein_lib::Token;
use smith_host_domain::parent;

pub use smith_domain::{Below, Lower};

/// Ordered requests sent to the parent or its lower protocol components.
#[derive(PartialEq, Eq, Debug)]
pub enum Output {
    /// A notice in the same parent vocabulary as the spawned capability.
    Parent(parent::Request),
    /// A root request scoped by its agent handle; its original owner is unchanged.
    Lower { agent: Token, request: Lower },
}

/// A parent command or lower terminal sent by the owning host.
#[derive(PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "the parent face moves values without an extra allocation")]
pub enum Input {
    /// A command in the face shared by both agent kinds.
    Parent(parent::Event),
    /// One terminal for a lower request, scoped to the agent that asked.
    Below { agent: Token, terminal: Below },
}
