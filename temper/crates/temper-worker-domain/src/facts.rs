//! What the worker tells whoever watches it (`domain/worker.md`, section 7):
//! the child domains' facts, content-free, gathered after every entry point
//! into one bounded queue the loop drains at its own pace. What does not fit
//! is dropped and counted, and nothing the domain decides depends on it.
//!
//! The host names a run by the engine's names for it, and its capabilities by
//! their clients' tokens, which are the host's: the host's for its run with
//! the agent child domain, the root's for its workspace with the checkout.
//! What the run itself tells is not among them: it goes to the engine as
//! jig's activity, through the hub.

use jig_domain_host as host;
use smith_host_domain as agent;
use temper_worker_domain_checkout as checkout;

/// Something that happened in a child domain.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// In the host child domain.
    Host { fact: host::Fact },
    /// In the checkout child domain.
    Checkout { fact: checkout::Fact },
    /// In the agent child domain.
    Agent { fact: agent::Fact },
}
