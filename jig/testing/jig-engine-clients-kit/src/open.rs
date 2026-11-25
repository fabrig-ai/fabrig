//! Open-problem exclusions (`jig/docs/design/domain/testing-fakes.md`, 1.3).
//! The sweeps drawing these faults take these exclusions.

use skein_world::{faults::Kind, sweep::Exclusion};

pub const RESENDS_A_PENDING_REQUEST: Kind =
    Kind { family: "people and parties", name: "a sign-in sends a request again while the first is pending" };

pub const EXCLUSIONS: [Exclusion; 0] = [];
