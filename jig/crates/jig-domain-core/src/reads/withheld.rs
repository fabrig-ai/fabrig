//! Turn and live-input withholding stub, filled by `withheld`
//! (`jig/docs/design/domain/engine.md`, 10.1).
//! Readers supply the durable owner where available. A person's run read
//! passes through `person_answer`, independent of its own party. Both live
//! stream outputs and opening snapshots pass their follower through `live` and `snapshot`.
//! Until filled, every turn is carried and every call input reaches its follower.

use crate::{
    Core, TurnRecord,
    reads::{Answer, EndedValue, PageTurn},
};
use jig_domain_tasks::TaskRecord;
use jig_domain_views::{Call, Writing, WritingCall};

pub(crate) fn turn(_core: &Core, _party: u64, _owner: Option<&TaskRecord>, record: TurnRecord) -> EndedValue {
    EndedValue::Turn(record)
}

pub(crate) fn page_turn(_core: &Core, _party: u64, _owner: Option<&TaskRecord>, record: TurnRecord) -> PageTurn {
    PageTurn::Carried(record)
}

pub(crate) fn person_answer(answer: Answer) -> Answer {
    answer
}

pub(crate) fn live(
    _core: &Core,
    _party: u64,
    _task: u64,
    _call: &mut Option<Call>,
    _writing: &mut Option<WritingCall>,
) {
}

/// The opening snapshot has both current calls and calls inside its writing.
pub(crate) fn snapshot(
    _core: &Core,
    _party: u64,
    _task: u64,
    _call: &mut Option<Call>,
    _writing: &mut Option<Writing>,
) {
}
