//! Assignment parts are dropped below wire 22 (jig/docs/design/protocol/channel/bridges.md, 6.1).
use super::{Context, kept::jig_worker_wire::v22::assignment};
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// A resumed run on an older worker starts from its window alone.
pub fn assignment_down(old: &Old, new: &mut Build, at: Node, _cx: &mut Context) -> Result<(), Halt> {
    if !old.content(old.field(at, assignment::PARTS)?)?.is_empty() {
        return Err(new.refuse(super::refusal::p0012::PARTS));
    }
    new.open()?;
    for field in [
        assignment::TASK,
        assignment::ATTEMPT,
        assignment::IDENTITY,
        assignment::WORKSPACE,
        assignment::SAVE,
        assignment::LIVE,
        assignment::CHARTER,
        assignment::TRANSCRIPT,
        assignment::ANSWERED,
        assignment::MESSAGES,
        assignment::GRANTS,
    ] {
        new.carry(old, old.field(at, field)?)?;
    }
    new.close()
}
