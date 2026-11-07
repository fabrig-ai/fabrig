//! The application-held client (client/domain/client.md, 3 and 4).
#![cfg_attr(test, expect(clippy::manual_map, reason = "step code matches optional fields without closures"))]
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod accounts;
mod client;
pub mod facts;
mod limits;
pub mod shapes;
mod store;
pub mod thread;
pub mod words;
pub use client::{
    APPROVAL_AGAIN, Admission, ApprovalAct, Client, Event, LinkView, NotSignedIn, Request, UnlinkedRequest,
};
pub use jig_client_domain::Limits as DomainLimits;
pub use jig_client_domain::{
    AccountCounts, Answered, Ask, Asker, Backoff, Document, Down, Family, Handle, Held, Key, Latest, Link, Outcome,
    Pending, Progress, Readable, Readout, Retry, Spoken, State, StreamBound, StreamsFull, Told, Up, Watchable, Why,
    contract,
};
pub use jig_client_protocol_link::{
    Approval, ApprovalCode, ApprovalState, AuthorizeWhy, Authorized, Decided, Message, NotAsked, Record, STREAM_KEY,
    Signing, Streamed, Taking, With,
};
pub use limits::{Limits, Unusable, valid};
#[cfg(test)]
mod tests;
