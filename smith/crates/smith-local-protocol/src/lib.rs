//! The local host's terminal, durable chat store, and checkout adapters
//! (smith/docs/design/protocol/hosts.md, section 5). The terminal retains bounded
//! continued lines and interrupt state; the store retains acknowledged chat
//! records; git, plain-tree and marker adapters retain bounded operations until
//! their terminals arrive. These adapters know no model policy or credential values.
//! Owners feed terminal bytes, store requests and filesystem or child completions,
//! then drain typed events and actions. Store acknowledgements follow durable writes.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod borrowed;
mod delivery_store;

#[cfg(any(test, feature = "fixture"))]
pub mod fixture;
pub mod format;

#[rustfmt::skip]
#[path = "generated/local_store_v1.rs"]
pub mod local_store_v1;
mod git;
mod git_child;
mod markers;
mod plain;
mod store;
mod terminal;

#[cfg(test)]
mod tests;

pub use borrowed::Borrowed;
pub use delivery_store::{decode_delivery, save_delivery};
pub use git::{Git, GitAction, GitCompletion, GitLimits};
pub use git_child::GitChild;
pub use markers::{MarkerAction, Markers};
pub use plain::{Plain, PlainAction, PlainLimits, Snapshot};
pub use store::{
    Action, CHAT_STORE_VERSION, File, History, RecordName, Store, StoreError, StoreFault, StoreLimits, StoreRequest,
    decode_state, decode_turns, save_state, save_turn, turn_name, turn_opens_window,
};
pub use terminal::{
    Entered, Event as TerminalEvent, Limits as TerminalLimits, Slash, Terminal, choice_text, entered, notice_text,
    run_failure_text, valid as terminal_valid,
};
