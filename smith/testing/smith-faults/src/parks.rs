//! Session states at a park (`smith/docs/design/parks.md`, 2).

use skein_world::faults::Kind;

/// The fault family for the state a park or cut leaves behind.
pub const FAMILY: &str = "a session's state at a park";

/// The last turn was told without its wait's result.
pub const WITHHELD_WAIT: Kind = Kind { family: FAMILY, name: "a withheld wait" };

/// The session closes a running job before answering.
pub const JOB_RUNNING: Kind = Kind { family: FAMILY, name: "a job running" };

/// The parts told before the cut can be eligible for restore.
pub const CUT_TURN: Kind = Kind { family: FAMILY, name: "a cut turn with kept parts" };

/// The host call's answer was told before the park or cut.
pub const HOST_CALL_ANSWERED: Kind = Kind { family: FAMILY, name: "a host call answered" };

/// A cut left a host call without a told result.
pub const HOST_CALL_UNANSWERED: Kind = Kind { family: FAMILY, name: "a host call unanswered" };

/// A cut left told parts of a sub-agent's conversation.
pub const SUB_AGENT_KEPT: Kind = Kind { family: FAMILY, name: "a sub-agent's conversation kept" };

/// The six kinds in contract order.
pub const KINDS: [Kind; 6] =
    [WITHHELD_WAIT, JOB_RUNNING, CUT_TURN, HOST_CALL_ANSWERED, HOST_CALL_UNANSWERED, SUB_AGENT_KEPT];
