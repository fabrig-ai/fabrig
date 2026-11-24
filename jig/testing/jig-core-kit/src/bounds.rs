//! Adapter table bounds (`jig/docs/design/domain/testing-workloads.md`, 2).
use skein_world::faults::Kind;

pub const FAMILY: &str = "the engine's own bounds";

#[must_use]
pub const fn table(name: &'static str) -> Kind {
    Kind { family: FAMILY, name }
}

pub const SUBSCRIPTIONS: Kind = table("subscriptions");
pub const CREATION_REPORTS: Kind = table("creation reports");
pub const KEPT_ANSWERS: Kind = table("kept answers");
