//! The clients' own bounds (`jig/docs/design/domain/testing-workloads.md`, 2).

use skein_world::faults::Kind;

pub const PENDING_EXCHANGES: Kind = Kind { family: "the engine's own bounds", name: "a sign-in's pending exchanges" };
