//! Table saturation kinds (`temper/docs/design/testing/workloads.md`, 2.4).

use skein_world::faults::Kind;

pub const FAMILY: &str = "the engine's own bounds";

#[must_use]
pub const fn table(name: &'static str) -> Kind {
    Kind { family: FAMILY, name }
}

pub const FORGE_WRITE_ENTRIES: Kind = table("forge write entries");
pub const FORGE_READ_ADMISSIONS: Kind = table("forge read admissions");
pub const BRANCH_HOLDS: Kind = table("branch holds");
pub const LANDING_ROWS: Kind = table("landing rows");
pub const WORKSPACES: Kind = table("workspaces");
pub const PREPARE_PERMITS: Kind = table("prepare permits");
pub const PER_RUN_SCRATCH: Kind = table("per-run scratch");
pub const RUN_SLOTS: Kind = table("run slots");
pub const BUILD_PLACES: Kind = table("build places");
