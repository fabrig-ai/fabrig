//! Patch 0001: the command wrote the skeleton; its functions are the author's (data/codec/patches.md, 5).

use super::Context;
use skein_codec::patch::{Build, Halt, Old, Site};
use skein_codec::value::Node;

/// Site 0, `written TaskRecord.record`, jig-store-tasks 9 to 10.
///
/// - `add TaskRecord.record BeforeTalk written`
/// - `add TaskRecord.talk option Talk none`
/// - `drop TaskRecord.closer`
/// - `drop TaskRecord.history`
/// - `drop TaskRecord.run`
/// - `drop TaskRecord.definition`
/// - `drop TaskRecord.started_from`
/// - `drop TaskRecord.interests`
/// - `drop TaskRecord.outcome`
/// - `drop TaskRecord.asked_requester`
pub fn task_record_record(_old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {
    Err(Halt::NoSite(Site::new(1, 0)))
}

/// Site 1, `written Value`, jig-store-tasks 9 to 10.
///
/// - `payload Value.live TaskRecord BeforeTalk`
/// - `payload Value.ended TaskRecord BeforeTalk`
/// - `add Value.live_talk`
/// - `add Value.ended_talk`
pub fn value(_old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {
    Err(Halt::NoSite(Site::new(1, 1)))
}
