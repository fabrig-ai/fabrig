//! Durable goal closer progress and fresh containment (temper/docs/design/domain/forge/branches.md, 2).
use alloc::boxed::Box;
use temper_engine_domain_forge_closer::{Closer, Step};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CloserRow {
    pub task: u64,
    pub goal: u64,
    pub closer: Closer,
    pub making: Option<Step>,
    pub close_changes: Box<[u64]>,
    pub landing_changes: Box<[u64]>,
    pub done: bool,
    pub relay: temper_engine_domain_forge_closer::relay::Relay,
}
