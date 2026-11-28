//! Ops's observability words and delegates (jig/docs/design/examples.md, 2 and 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use jig_domain_core_vocabulary as core;
use jig_ops_domain_observability as observability;
use jig_protocol_tools::Limits;

pub use observability::{alert_words, health_words};

/// Decode one triage delegate and append the alert batch to its words.
#[must_use]
pub fn triage(delegate: &[u8], alerts: &[u64], limits: &Limits) -> Option<Box<[core::Delegate]>> {
    observability::triage_batch(jig_protocol_tools::batch(delegate, limits).ok(), alerts)
}

/// The owner templates always request one delegate, matching the outcome's one child.
#[must_use]
pub fn lead(delegate: &[u8], limits: &Limits) -> Option<Box<[core::Delegate]>> {
    let batch = jig_protocol_tools::batch(delegate, limits).ok()?;
    if batch.len() != 1 {
        return None;
    }
    Some(batch)
}

#[cfg(test)]
mod tests;
