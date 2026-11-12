//! Pure, typed translation from the engine's owned run and call vocabulary
//! into Smith's generic agent boundary (jig's domain/hosts/README.md, section 5 and domain/hosts/charters.md;
//! jig's domain/engine/executors.md; smith's domain/run.md, sections 3, 5 and 9).
//!
//! Jig's core keeps task policy and numbered calls; the root wraps records and
//! orders their commits. This crate keeps no state and performs no IO. Its `start` takes
//! already decoded concrete history and already prepared workspace mounts from
//! the composing world; a later protocol adapter supplies the same values from
//! bytes and worker preparation. It never chooses authority or commits a call.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(test, expect(clippy::manual_map, reason = "step code matches optional fields without closures"))]
extern crate alloc;
use alloc::boxed::Box;

/// The requester's preferences section title (domain/agent/ask-temper.md, 4).
pub const PREFERENCES_TITLE: &[u8] = b"Kept about this person";

/// The tools whose calls are the person's alone (domain/agent/ask-temper.md, 4).
pub const PRIVATE_TOOLS: [&[u8]; 1] = [b"remember"];

/// The turn parts shown to its person alone (domain/agent/ask-temper.md, 4).
#[must_use]
pub fn private() -> jig_protocol_transcript::Private {
    let mut tools = skein_lib::List::with_capacity(1);
    for tool in PRIVATE_TOOLS {
        tools.push(Box::from(tool)).expect("private tool names");
    }
    jig_protocol_transcript::Private { title: PREFERENCES_TITLE.into(), tools }
}

mod answers;
mod calls;
mod docs;
mod document;
mod finding;
mod finish;
mod forge;
mod json;
pub mod kinds;
mod nested;
mod notes;
mod reference;
mod remember;
pub mod room;
mod see;
mod settings;
mod start;

pub use answers::{
    ForgeReadBounds, answer, answer_bounded, answer_for_job, answer_for_page, answer_for_task, answer_for_tool,
    unasked_kind,
};
pub use calls::{call, call_configured, call_in, tools};
pub use finish::{ChangeResource, Terminal, result, turn};
pub use reference::reference;
pub use see::addresses::see_reference;
pub use see::{ADDRESSES, SEE_PART, see_answer};
pub use settings::{
    settings_answer, settings_answer_bounded, settings_note, settings_repository_answer, settings_text,
};
pub use start::{charter, section, start};

fn decimal(bytes: &[u8], member: &'static [u8]) -> Result<u64, Problem> {
    if bytes.is_empty() {
        return Err(Problem::Range(member));
    }
    let mut value = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return Err(Problem::Range(member));
        }
        value = value.checked_mul(10).ok_or(Problem::Range(member))?;
        value = value.checked_add(u64::from(byte.saturating_sub(b'0'))).ok_or(Problem::Range(member))?;
    }
    Ok(value)
}

/// What a host call's input decodes to: a call for the root to decide,
/// or its answer, made here at decode, deciding nothing
/// (protocol/documentation.md, section 5).
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "the stateless decode boundary moves the engine's bounded call whole")]
pub enum Decoded {
    Call(temper_engine_domain::Call),
    Answered(smith_domain_run::HostAnswer),
}

/// The most a call answered at decode holds, as the root counts a
/// settled call (jig's `SettledCall::owned_bytes`: its tool and body);
/// the start refuses a smaller answer bound
/// (protocol/documentation.md, section 5).
pub const DOC_ANSWER: u32 = 2_048;

/// Smith channel v6's `CallName`: fixed-width activation (u64), completion
/// (u32), position (u32), with no record prefix or tail.
pub const CALL_NAME_BYTES: u32 = 16;

/// A host call that cannot be translated into the engine's declared schema.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Problem {
    Malformed,
    TooLarge,
    Missing(&'static [u8]),
    /// A setting's kind that needs a member its read left out (domain/agent/agents.md, 1).
    Needs {
        kind: &'static [u8],
        member: &'static [u8],
    },
    Type(&'static [u8]),
    Range(&'static [u8]),
    /// A member beside `changes` belongs in each change (domain/agent/tools.md, 1).
    Beside(&'static [u8]),
    UnknownTool,
    /// A forge read's answer has a kind no agent read asks for (domain/agent/agents.md).
    Unasked(&'static str),
}

#[cfg(test)]
mod tests;
