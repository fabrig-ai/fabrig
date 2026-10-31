//! Forge removal (temper/docs/design/draft/playground-reset.md, 4).
use crate::{Domain, Request};
use skein_lib::Queue;

/// Begin removal in this decision; false when no kept record is found.
pub(crate) fn begin(_d: &mut Domain, _forge: u16, _out: &mut Queue<Request>) -> bool {
    false
}
