//! Shared fault names (temper/docs/design/testing/worker.md, 3.5).

use skein_world::faults::Kind;

pub const FETCH_CUT: Kind = Kind { family: "worker", name: "a workspace fetch timing out partway" };
pub const DISK_FULL: Kind = Kind { family: "worker", name: "the fake disk filling" };
pub const TRASH_STUCK: Kind = Kind { family: "worker", name: "a trash entry that can't be removed" };
pub const PLACEMENT_BUSY: Kind =
    Kind { family: "worker", name: "a placement refused as busy while build places are full" };
