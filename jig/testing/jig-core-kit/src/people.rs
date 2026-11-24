//! People's workload acts (`jig/docs/design/domain/testing-workloads.md`, 2).
use skein_world::faults::Kind;

pub const FAMILY: &str = "people and parties";
pub const CANCELS_A_TASK: Kind = Kind { family: FAMILY, name: "cancelling a task" };
pub const HOLDS_A_TASK: Kind = Kind { family: FAMILY, name: "holding a task" };
pub const RELEASES_A_TASK: Kind = Kind { family: FAMILY, name: "releasing a held task" };
pub const ANSWERS_LATE: Kind = Kind { family: FAMILY, name: "answering a question late" };
pub const NEVER_ANSWERS: Kind = Kind { family: FAMILY, name: "never answering a question" };
pub const ANSWERS_TWICE: Kind = Kind { family: FAMILY, name: "answering a question twice" };
pub const ANSWERS_AN_ENDED_RUN: Kind = Kind { family: FAMILY, name: "answering a question of a run that has ended" };
pub const WORDS_AFTER_ITS_RUN_ENDED: Kind = Kind { family: FAMILY, name: "words to a task whose run has ended" };
