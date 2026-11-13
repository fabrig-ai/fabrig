//! Landing targets from stored adoption settings (temper/docs/design/protocol/baseline.md, 7).
use super::{Context, refusal::p0005::LANDING_BRANCH_MISSING};
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// The driver reads the project's stored repository settings for this record.
pub fn landing_state_branch(_old: &Old, new: &mut Build, _at: Node, cx: &mut Context) -> Result<(), Halt> {
    let Some(branch) = &cx.landing_branch else {
        return Err(new.refuse(LANDING_BRANCH_MISSING));
    };
    if branch.is_empty() {
        return Err(new.refuse(LANDING_BRANCH_MISSING));
    }
    new.put_content(branch)
}
