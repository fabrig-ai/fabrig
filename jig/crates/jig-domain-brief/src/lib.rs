//! Brief planning and gathering (domain/engine/briefs.md, section 1).
//!
//! The core gives the brief text and connector section tokens with
//! reported sizes. The brief keeps core text, tokens and sizes; connector
//! bytes stay with their owners. It plans cuts by priority within one byte
//! budget and one deadline, then tells the core which sections to take.
//!
//! `step` and `fire` change only the supplied domain. Every connector handoff
//! is a request routed by the parent. The domain knows connector numbers, not
//! their systems or their section content.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod limits;
mod orchestrated;
mod planned;

pub use jig_domain_contract::parts::{Failure as GatherFailure, Fate as GatherFate};
pub use limits::Limits;
pub use orchestrated::{
    Domain as GatherDomain, Event as GatherEvent, Missing as GatherMissing, Placed as GatherPlaced,
    Request as GatherRequest, fire as gather_fire, step as gather_step, valid,
};
pub use planned::{ConnectorAction, ConnectorPlan, Core, Placement, Placing, Plan, Planned, core_name, place, plan};
