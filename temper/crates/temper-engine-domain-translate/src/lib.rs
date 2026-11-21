//! The engine's typed host and forge boundary translations (temper/docs/design/domain/routing.md, 2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
mod adoption;
pub use adoption::adoption_refusal_words;
mod tools;
pub use tools::*;
mod shape;
pub use shape::{call_needs_input, checked_call, is_core_read, tool_kind};
mod inbox;
pub use inbox::{CHANGE_RESULT_ENVELOPE_BYTES, inbox_sender, inbox_words, message};
mod resources;
pub use resources::*;
mod parameters;
pub use parameters::*;
pub mod change_llm;
#[cfg(test)]
mod tests;

fn append(out: &mut skein_lib::List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if out.room() > 0 {
            out.push(*byte).expect("checked message room");
        }
    }
}
