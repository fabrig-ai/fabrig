//! Restart fault kinds (`temper/docs/design/testing/restarts.md`, section 2).

use skein_world::faults::Kind;

/// The restart fault family.
pub const FAMILY: &str = "restarts";
/// The part of `Stream::Faults` every restart injector draws from.
pub const PART: &str = "restarts";

/// The engine restarted cold (sections 4 and 5).
pub const ENGINE_CRASH: Kind = Kind { family: FAMILY, name: "engine crash" };
/// A settled stop at any moment (section 4).
pub const SETTLED_STOP: Kind = Kind { family: FAMILY, name: "settled stop" };
/// A stop before the start's mark (section 4).
pub const STOP_BEFORE_MARK: Kind = Kind { family: FAMILY, name: "stop before the mark" };
/// A stop after the mark, before the opening (section 4).
pub const STOP_BEFORE_OPENING: Kind = Kind { family: FAMILY, name: "stop before the opening" };
/// A stop after the opening, while the forge reconciles (section 4).
pub const STOP_AFTER_OPENING: Kind = Kind { family: FAMILY, name: "stop after the opening" };
/// A second signal during a stop's settle (section 4).
pub const SECOND_SIGNAL: Kind = Kind { family: FAMILY, name: "second signal" };
/// The forge down through a start, until a named moment (section 5).
pub const FORGE_DOWN_UNTIL: Kind = Kind { family: FAMILY, name: "forge down until a moment" };
/// The forge down for good (section 5).
pub const FORGE_DOWN_FOR_GOOD: Kind = Kind { family: FAMILY, name: "forge down for good" };
/// Write entries all held by ended owners (section 5).
pub const ENTRIES_HELD_BY_ENDED: Kind = Kind { family: FAMILY, name: "write entries held by ended owners" };
/// A held goal branch the forge lacks, with no creation entry (section 5).
pub const BRANCH_THE_FORGE_LACKS: Kind = Kind { family: FAMILY, name: "held branch the forge lacks" };
/// An uncertain write whose find keeps failing (section 5).
pub const FIND_KEEPS_FAILING: Kind = Kind { family: FAMILY, name: "uncertain write's find failing" };
/// A read rate-limited at every attempt (section 5).
pub const READ_ALWAYS_LIMITED: Kind = Kind { family: FAMILY, name: "read limited at every attempt" };
/// The engine restarted while workers are mid-turn, announced (section 7).
pub const ENGINE_RESTART_MID_TURN: Kind = Kind { family: FAMILY, name: "engine restarted mid-turn" };
/// The engine restarted while workers are mid-turn, not announced (section 7).
pub const ENGINE_CRASH_MID_TURN: Kind = Kind { family: FAMILY, name: "engine crashed mid-turn" };
/// Every process of a unit signalled (section 8).
pub const UNIT_SIGNALLED: Kind = Kind { family: FAMILY, name: "unit signalled mid-turn" };
/// An older layout (section 8).
pub const OLDER_LAYOUT: Kind = Kind { family: FAMILY, name: "older layout" };

/// How a worker stops mid-turn (section 6).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Way {
    Drained,
    Signalled,
    Crashed,
}

/// Where a worker stops mid-turn (section 6).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Point {
    LlmCall,
    ToolCall,
    DeliveryCheck,
    Save,
}

/// The kind of a worker stopped or crashed at a point of a turn: one of twelve.
#[must_use]
pub const fn worker(way: Way, point: Point) -> Kind {
    match (way, point) {
        (Way::Drained, Point::LlmCall) => Kind { family: FAMILY, name: "worker drained in an LLM call" },
        (Way::Drained, Point::ToolCall) => Kind { family: FAMILY, name: "worker drained in a tool call" },
        (Way::Drained, Point::DeliveryCheck) => Kind { family: FAMILY, name: "worker drained in a delivery check" },
        (Way::Drained, Point::Save) => Kind { family: FAMILY, name: "worker drained in a save" },
        (Way::Signalled, Point::LlmCall) => Kind { family: FAMILY, name: "worker signalled in an LLM call" },
        (Way::Signalled, Point::ToolCall) => Kind { family: FAMILY, name: "worker signalled in a tool call" },
        (Way::Signalled, Point::DeliveryCheck) => Kind { family: FAMILY, name: "worker signalled in a delivery check" },
        (Way::Signalled, Point::Save) => Kind { family: FAMILY, name: "worker signalled in a save" },
        (Way::Crashed, Point::LlmCall) => Kind { family: FAMILY, name: "worker crashed in an LLM call" },
        (Way::Crashed, Point::ToolCall) => Kind { family: FAMILY, name: "worker crashed in a tool call" },
        (Way::Crashed, Point::DeliveryCheck) => Kind { family: FAMILY, name: "worker crashed in a delivery check" },
        (Way::Crashed, Point::Save) => Kind { family: FAMILY, name: "worker crashed in a save" },
    }
}

/// The engine world's kinds (section 4).
pub const ENGINE: [Kind; 5] = [ENGINE_CRASH, SETTLED_STOP, STOP_BEFORE_MARK, STOP_AFTER_OPENING, SECOND_SIGNAL];
/// The forge world's kinds (section 5).
pub const FORGE: [Kind; 7] = [
    ENGINE_CRASH,
    FORGE_DOWN_UNTIL,
    FORGE_DOWN_FOR_GOOD,
    ENTRIES_HELD_BY_ENDED,
    BRANCH_THE_FORGE_LACKS,
    FIND_KEEPS_FAILING,
    READ_ALWAYS_LIMITED,
];
/// The worker world's kinds, by way then point (section 6).
pub const WORKER: [Kind; 12] = [
    worker(Way::Drained, Point::LlmCall),
    worker(Way::Drained, Point::ToolCall),
    worker(Way::Drained, Point::DeliveryCheck),
    worker(Way::Drained, Point::Save),
    worker(Way::Signalled, Point::LlmCall),
    worker(Way::Signalled, Point::ToolCall),
    worker(Way::Signalled, Point::DeliveryCheck),
    worker(Way::Signalled, Point::Save),
    worker(Way::Crashed, Point::LlmCall),
    worker(Way::Crashed, Point::ToolCall),
    worker(Way::Crashed, Point::DeliveryCheck),
    worker(Way::Crashed, Point::Save),
];
/// The mid-turn engine restart kinds (section 7).
pub const MID_TURN: [Kind; 2] = [ENGINE_RESTART_MID_TURN, ENGINE_CRASH_MID_TURN];
/// The simulated world's kinds (section 8).
pub const SIMULATED: [Kind; 2] = [UNIT_SIGNALLED, OLDER_LAYOUT];
