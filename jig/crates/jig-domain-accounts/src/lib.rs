//! Accounts and their sources, component grants and usability (domain/accounts.md, section 3;
//! domain/engine.md, section 12). Each account is the deployment's or a party's, numbered by a
//! counter that never gives a number twice, with its sources in its holder's order; each source
//! keeps its own state. The component owns refreshing, records and generation numbering. This
//! child keeps names, validity, limits, refusals and failures, and releases all wants at close.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
mod boundary;
mod changes;
mod checks;
mod domain;
mod facts;
mod keys;
mod limits;
mod ordering;
pub mod programs;
mod providers;
mod records;
mod restore;
mod selection;
mod signing;
pub use boundary::{
    ADDRESS, Answer, Change, Deadline, End, Ended, Event, Fact, Failure, FromComponent, KeyFailure, Pending, Place,
    Program, ProgramFailure, ProgramHost, ProgramResult, Request, Signing, State, ToComponent, Turned, Unread,
};
pub use checks::{Bound, Refusal, SignIn, SignInRedirectWhy, room_for_sign_in};
pub use domain::{Account, Domain, fire, step};
pub use facts::{
    AccountFact, Audience, FACT_LABEL, FACT_TEXT, FACTS, FactValue, FactWindow, KIND_EMAIL, KIND_ENDS, KIND_NAME,
    KIND_PLAN, KIND_RENEWS, KIND_USAGE, KIND_WORKSPACE, ORIGIN_ANSWER, ORIGIN_LIMIT, ORIGIN_PROGRAM, ORIGIN_REFRESH,
    ORIGIN_SIGN_IN, ORIGIN_USAGE_READ, SHARE_WHOLE, WINDOW_LIMITED, WINDOW_OPEN, WINDOW_WARNED, audience,
};
pub use keys::{KEY_VALID, PROGRAM_VALID};
pub use limits::{Limits, valid};
pub use providers::{MANAGE_USAGE, Offers, PROVIDER_NAME, Provider, Providers};
pub use records::{DISPLAY, Held, Holder, Key, Kind, NAME, SUBJECT, Stored, USES, Use};
pub use restore::{RestoreLimit, RestoreSet};
pub use selection::{Coverage, Excluded, HostSource, Selection, WaitWhy};

use jig_domain_link::{Form, Grant, SourceName, Worker};
