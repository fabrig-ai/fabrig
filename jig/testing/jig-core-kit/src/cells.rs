//! Parks against wakes (`jig/docs/design/domain/testing-cells.md`, 2 to 4).
//!
//! A pairing keeps only the halves a driver tells it, keyed by park. It never
//! picks a target or judges the cell's outcome; the world does both.

use std::collections::BTreeMap;

use skein_lib::Time;
use skein_world::faults::{FaultLedger, Kind, Struck};
use smith_faults::parks as smith;

use crate::parks as jig;

/// The fault family for a smith state paired with a jig wake or end.
pub const FAMILY: &str = "parks against wakes";

/// What jig does after the park.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Column {
    /// The task runs another attempt.
    Resumes,
    /// The task does not resume from this park.
    Ends,
}

/// A possible pair, its fault kind and the outcome it must be checked under.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub smith: Kind,
    pub jig: Kind,
    pub kind: Kind,
    pub column: Column,
}

macro_rules! cells {
    ($($constant:ident: $smith:ident, $jig:ident, $column:ident => $name:literal;)*) => {
        $(
            #[doc = $name]
            pub const $constant: Kind = Kind { family: FAMILY, name: $name };
        )*

        /// All possible pairs in the cell matrix.
        pub const CELLS: [Cell; 21] = [
            $(Cell { smith: smith::$smith, jig: jig::$jig, kind: $constant, column: Column::$column },)*
        ];
    };
}

cells! {
    WITHHELD_WAIT_ANSWERED_WHILE_PARKED: WITHHELD_WAIT, ANSWERED_WHILE_PARKED, Resumes =>
        "a withheld wait, a parked run's answer arrives while it is parked";
    WITHHELD_WAIT_ANSWERED_TO_ITS_SUCCESSOR: WITHHELD_WAIT, ANSWERED_TO_ITS_SUCCESSOR, Resumes =>
        "a withheld wait, a parked run's answer reaches its successor";
    WITHHELD_WAIT_PARKS_ON_NOTHING: WITHHELD_WAIT, PARKS_ON_NOTHING, Resumes =>
        "a withheld wait, a run parks with nothing to await";
    WITHHELD_WAIT_WATCHES_A_TASK_THAT_CAN_ANSWER: WITHHELD_WAIT, WATCHES_A_TASK_THAT_CAN_ANSWER, Resumes =>
        "a withheld wait, a watch that can still bring the answer";
    WITHHELD_WAIT_ANSWERED_AFTER_ITS_END: WITHHELD_WAIT, ANSWERED_AFTER_ITS_END, Ends =>
        "a withheld wait, a parked run's answer arrives after it ended";
    WITHHELD_WAIT_COORDINATOR_ENDS_WHILE_A_CHILD_WAITS: WITHHELD_WAIT, COORDINATOR_ENDS_WHILE_A_CHILD_WAITS, Ends =>
        "a withheld wait, a coordinator ends while a child waits for its answer";
    WITHHELD_WAIT_STUCK_IN_A_CIRCLE: WITHHELD_WAIT, STUCK_IN_A_CIRCLE, Ends =>
        "a withheld wait, a circle of questions and watches";

    JOB_RUNNING_ANSWERED_WHILE_PARKED: JOB_RUNNING, ANSWERED_WHILE_PARKED, Resumes =>
        "a job running, a parked run's answer arrives while it is parked";
    JOB_RUNNING_ANSWERED_TO_ITS_SUCCESSOR: JOB_RUNNING, ANSWERED_TO_ITS_SUCCESSOR, Resumes =>
        "a job running, a parked run's answer reaches its successor";
    JOB_RUNNING_PARKS_ON_NOTHING: JOB_RUNNING, PARKS_ON_NOTHING, Resumes =>
        "a job running, a run parks with nothing to await";
    JOB_RUNNING_WATCHES_A_TASK_THAT_CAN_ANSWER: JOB_RUNNING, WATCHES_A_TASK_THAT_CAN_ANSWER, Resumes =>
        "a job running, a watch that can still bring the answer";
    JOB_RUNNING_ANSWERED_AFTER_ITS_END: JOB_RUNNING, ANSWERED_AFTER_ITS_END, Ends =>
        "a job running, a parked run's answer arrives after it ended";
    JOB_RUNNING_COORDINATOR_ENDS_WHILE_A_CHILD_WAITS: JOB_RUNNING, COORDINATOR_ENDS_WHILE_A_CHILD_WAITS, Ends =>
        "a job running, a coordinator ends while a child waits for its answer";
    JOB_RUNNING_STUCK_IN_A_CIRCLE: JOB_RUNNING, STUCK_IN_A_CIRCLE, Ends =>
        "a job running, a circle of questions and watches";

    HOST_CALL_ANSWERED_ANSWERED_WHILE_PARKED: HOST_CALL_ANSWERED, ANSWERED_WHILE_PARKED, Resumes =>
        "a host call answered, a parked run's answer arrives while it is parked";
    HOST_CALL_ANSWERED_ANSWERED_TO_ITS_SUCCESSOR: HOST_CALL_ANSWERED, ANSWERED_TO_ITS_SUCCESSOR, Resumes =>
        "a host call answered, a parked run's answer reaches its successor";
    HOST_CALL_ANSWERED_PARKS_ON_NOTHING: HOST_CALL_ANSWERED, PARKS_ON_NOTHING, Resumes =>
        "a host call answered, a run parks with nothing to await";
    HOST_CALL_ANSWERED_WATCHES_A_TASK_THAT_CAN_ANSWER: HOST_CALL_ANSWERED, WATCHES_A_TASK_THAT_CAN_ANSWER, Resumes =>
        "a host call answered, a watch that can still bring the answer";
    HOST_CALL_ANSWERED_ANSWERED_AFTER_ITS_END: HOST_CALL_ANSWERED, ANSWERED_AFTER_ITS_END, Ends =>
        "a host call answered, a parked run's answer arrives after it ended";
    HOST_CALL_ANSWERED_COORDINATOR_ENDS_WHILE_A_CHILD_WAITS: HOST_CALL_ANSWERED, COORDINATOR_ENDS_WHILE_A_CHILD_WAITS, Ends =>
        "a host call answered, a coordinator ends while a child waits for its answer";
    HOST_CALL_ANSWERED_STUCK_IN_A_CIRCLE: HOST_CALL_ANSWERED, STUCK_IN_A_CIRCLE, Ends =>
        "a host call answered, a circle of questions and watches";
}

/// An impossible pair and why it cannot be a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Never {
    pub smith: Kind,
    pub jig: Kind,
    pub why: &'static str,
}

const WAKE_KINDS: [Kind; 8] = [
    jig::ANSWERED_WHILE_PARKED,
    jig::ANSWERED_TO_ITS_SUCCESSOR,
    jig::PARKS_ON_NOTHING,
    jig::WATCHES_A_TASK_THAT_CAN_ANSWER,
    jig::ANSWERED_AFTER_ITS_END,
    jig::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS,
    jig::STUCK_IN_A_CIRCLE,
    jig::PARKS_WITH_A_QUESTION_OPEN,
];

const BEFORE_OUTCOME: &str = "a park with a question open pairs under the kind of its outcome";

/// All impossible pairs, with cut-only states listed under the cut reason even
/// when the jig kind is the park with a question open.
pub const NEVER: [Never; 27] = {
    let open_wait = Never { smith: smith::WITHHELD_WAIT, jig: jig::PARKS_WITH_A_QUESTION_OPEN, why: BEFORE_OUTCOME };
    let mut pairs = [open_wait; 27];
    pairs[1] = Never { smith: smith::JOB_RUNNING, ..open_wait };
    pairs[2] = Never { smith: smith::HOST_CALL_ANSWERED, ..open_wait };

    let cuts = [
        (smith::CUT_TURN, "a parked answer tells its turn whole, leaving no cut turn"),
        (smith::HOST_CALL_UNANSWERED, "a parked answer tells its turn whole, leaving no unanswered host call"),
        (smith::SUB_AGENT_KEPT, "a parked answer tells its turn whole, leaving no sub-agent conversation to continue"),
    ];
    let mut next = 3;
    let mut cut = 0;
    while cut < cuts.len() {
        let (smith, why) = cuts[cut];
        let mut wake = 0;
        while wake < WAKE_KINDS.len() {
            pairs[next] = Never { smith, jig: WAKE_KINDS[wake], why };
            next += 1;
            wake += 1;
        }
        cut += 1;
    }
    pairs
};

/// Looks up a possible pair, or gives the reason the pair is impossible or unknown.
pub fn cell(smith: Kind, jig: Kind) -> Result<&'static Cell, &'static str> {
    if let Some(cell) = CELLS.iter().find(|cell| cell.smith == smith && cell.jig == jig) {
        return Ok(cell);
    }
    if let Some(pair) = NEVER.iter().find(|pair| pair.smith == smith && pair.jig == jig) {
        return Err(pair.why);
    }
    if !smith::KINDS.contains(&smith) {
        return Err("not a smith park kind");
    }
    Err("not a jig park kind")
}

/// A park keyed by task number, jig attempt and per-task park ordinal.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Park {
    pub task: u64,
    pub attempt: u64,
    /// The park's ordinal among this task's parks, from 1.
    pub number: u32,
}

#[derive(Default)]
struct Halves {
    smith: Option<Kind>,
    jig: Option<Kind>,
}

/// Each park's observed halves; only a new half can complete a pair.
#[derive(Default)]
pub struct Pairing {
    parks: BTreeMap<Park, Halves>,
}

impl Pairing {
    /// Starts an empty pairing for one world.
    #[must_use]
    pub fn new() -> Pairing {
        Pairing::default()
    }

    /// Records a half by its family. The second half injects and returns the
    /// cell once; repeats return `None`. Impossible pairs, conflicting halves
    /// and kinds of neither family are findings.
    pub fn fell(
        &mut self,
        park: Park,
        kind: Kind,
        at: Time,
        ledger: &mut FaultLedger,
    ) -> Result<Option<&'static Cell>, String> {
        let halves = self.parks.entry(park).or_default();
        let half = match kind.family {
            smith::FAMILY => &mut halves.smith,
            jig::FAMILY => &mut halves.jig,
            _ => return Err(format!("{park:?}: {kind:?} is of neither park family")),
        };
        if let Some(previous) = *half {
            if previous == kind {
                return Ok(None);
            }
            return Err(format!("{park:?}: conflicting park kinds {previous:?} and {kind:?}"));
        }
        *half = Some(kind);

        let (Some(smith), Some(jig)) = (halves.smith, halves.jig) else {
            return Ok(None);
        };
        let cell = cell(smith, jig).map_err(|why| format!("{park:?}: {:?}, {:?}: {why}", smith.name, jig.name))?;
        let entry =
            ledger.inject(cell.kind, at, Struck { request: Some(format!("task {}", park.task)), ..Struck::default() });
        ledger.stop(entry, at)?;
        Ok(Some(cell))
    }
}

#[cfg(test)]
mod tests;
