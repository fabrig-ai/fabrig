//! Jig's host (domain/hosts/README.md, sections 4 to 5 and domain/hosts/hub.md) keeps the lifecycle of runs
//! on the slots of a worker or an engine. It admits assignments, asks an
//! optional application workspace to prepare and serve deliveries, supervises
//! an agent capability, and answers after both have settled. It keeps turns
//! and answers until the core acknowledges them. The root translates the link
//! to the core, workspace and agent capabilities; an engine root gives it a
//! direct link that is never lost and no workspace. The agent may be a process
//! or an inline agent, and the host does not distinguish them.
//! The same slots host jobs through the root's job capability (jig/docs/design/domain/hosts/hub.md, 9),
//! without an agent or a save.
//! The run lifecycle arms no timers; its callers bound each wait. The source
//! machine owns periodic checks, driven by its parent through `next_deadline`
//! and `resume`. Facts are best effort and never govern a decision.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod activity;
mod answers;
mod asks;
mod assignment;
mod boundary;
mod call;
mod domain;
mod facts;
mod hosted;
mod jobs;
mod limits;
mod link;
mod live;
mod model;
pub use live::{LiveAsk, LiveKind, Piece};
pub use model::{ModelEvidence, ModelFailure, ModelFailureKind};
mod refusals;
pub mod sources;
#[cfg(test)]
mod tests;
mod turns;

pub use activity::AgentActivity;
pub use boundary::{
    AgentFailure, AnsweredCall, Ask, Assignment, Bound, CallAnswer, Delivery, DeliveryOutcome, Ending, Event, Exceeded,
    Failure, Finish, FromAgent, FromJob, Invalid, JobAssignment, JobEnd, JobEnding, JobFailure, JobInvalid, JobRefusal,
    MessageRefusal, Mismatch, Preparation, Reason, Refusal, Refused, Request, RunAnswer, RunAssignment, RunFailure,
    SettledAnswer, Start, StartCause, StartMessage, ToAgent, ToJob, Turn, Undecoded, Work, Workspace,
};
pub use domain::{Hub, fire_sources, resume, resume_runs, sources_due, step};
pub use facts::Fact;
pub use limits::{Limits, valid_hub};

pub use jig_domain_link::AskAnswer;
pub use link::{LinkDown, LinkUp, Relayed};

use jig_domain_link::{
    Bounds, CountKind, Form, Grant, Hello, Hosting, JobBound, JobKind, Login, LoginState, Operation, Phase,
    SignOutResult, SourceFailure, SourceName,
};

#[cfg(test)]
use jig_domain_link::{Scheme, valid_program_endpoints};
