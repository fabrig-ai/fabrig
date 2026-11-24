//! People's acts (`temper/docs/design/testing/workloads.md`, 2.3).

use skein_world::faults::Kind;

pub const FAMILY: &str = "people and parties";

pub const CANCELS_A_GOAL: Kind = Kind { family: FAMILY, name: "cancelling a goal" };
pub const CANCELS_A_CHANGE: Kind = Kind { family: FAMILY, name: "cancelling a change" };
pub const CANCELS_A_LANDING: Kind = Kind { family: FAMILY, name: "cancelling a landing" };
pub const HOLDS_A_MEMBER: Kind = Kind { family: FAMILY, name: "holding a member of a live landing" };
pub const RELEASES_A_MEMBER: Kind = Kind { family: FAMILY, name: "releasing a held member of a live landing" };
pub const ANSWERS_LATE: Kind = Kind { family: FAMILY, name: "answering a change's question late" };
pub const NEVER_ANSWERS: Kind = Kind { family: FAMILY, name: "never answering a change's question" };
pub const ANSWERS_TWICE: Kind = Kind { family: FAMILY, name: "answering a change's question twice" };
pub const ANSWERS_AN_ENDED_RUN: Kind = Kind { family: FAMILY, name: "answering a question whose run has ended" };
pub const WORDS_TO_AN_ENDED_PRODUCER: Kind =
    Kind { family: FAMILY, name: "words to a change whose producer has ended" };
