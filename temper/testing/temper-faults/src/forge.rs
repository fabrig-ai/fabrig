//! Forge fault kinds (`temper/docs/design/testing/workloads.md`, section 2.2).

use skein_world::faults::Kind;

/// The forge has no room for a call.
pub const NO_ROOM: Kind = Kind { family: "busy", name: "no_room" };
/// The forge is unavailable.
pub const UNAVAILABLE: Kind = Kind { family: "busy", name: "unavailable" };
/// A call times out.
pub const TIMED_OUT: Kind = Kind { family: "busy", name: "timed_out" };
/// A merge lands after its timeout.
pub const LANDING: Kind = Kind { family: "busy", name: "landing" };
/// A call answers late.
pub const LATE: Kind = Kind { family: "busy", name: "late" };
/// A call is rate-limited.
pub const LIMITED: Kind = Kind { family: "busy", name: "limited" };

/// The head view lags behind.
pub const HEAD_BEHIND: Kind = Kind { family: "lagging", name: "head_behind" };
/// The base view lags behind.
pub const BASE_BEHIND: Kind = Kind { family: "lagging", name: "base_behind" };
/// A merge record arrives late.
pub const MERGE_LATE: Kind = Kind { family: "lagging", name: "merge_late" };

/// Pull files exceed the forge's bound.
pub const PULL_FILES: Kind = Kind { family: "bounds", name: "pull_files" };
/// A comparison exceeds the forge's bound.
pub const COMPARISON: Kind = Kind { family: "bounds", name: "comparison" };
/// A job log exceeds the forge's bound.
pub const JOB_LOG: Kind = Kind { family: "bounds", name: "job_log" };

/// The forge refuses to close a pull.
pub const CLOSE_REFUSED: Kind = Kind { family: "refusing", name: "close_refused" };
/// Another user closes a pull.
pub const CLOSED_BY_ANOTHER: Kind = Kind { family: "refusing", name: "closed_by_another" };
/// Another user merges a pull.
pub const MERGED_BY_ANOTHER: Kind = Kind { family: "refusing", name: "merged_by_another" };

/// The busy sweep's kinds, in configuration order.
pub const BUSY: [Kind; 6] = [NO_ROOM, UNAVAILABLE, TIMED_OUT, LANDING, LATE, LIMITED];
/// The lagging sweep's kinds, in configuration order.
pub const LAGGING: [Kind; 3] = [HEAD_BEHIND, BASE_BEHIND, MERGE_LATE];
/// The past-bounds sweep's kinds, in configuration order.
pub const PAST_BOUNDS: [Kind; 3] = [PULL_FILES, COMPARISON, JOB_LOG];
/// The refusing sweep's kinds, in configuration order.
pub const REFUSING: [Kind; 3] = [CLOSE_REFUSED, CLOSED_BY_ANOTHER, MERGED_BY_ANOTHER];
