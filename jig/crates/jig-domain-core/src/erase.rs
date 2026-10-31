//! Whole-task erasure (jig/docs/design/protocol/store.md, 8.5, 11.1).
use crate::{Core, Limits, Request};
use skein_lib::{Env, Queue};

/// What selects ended tasks for erasure; retention adds its horizon later.
#[expect(dead_code, reason = "selection is reserved for the whole-task erasure driver")]
pub(crate) enum Select {
    Project(u32),
}

/// Erasures in flight.
#[derive(Debug)]
pub(crate) struct Erasures {}

/// Walk selected ended tasks, then call `deletion::tasks_erased`.
#[expect(dead_code, reason = "entry point is reserved for the whole-task erasure driver")]
pub(crate) fn scan(_core: &mut Core, _env: &Env<Limits>, _out: &mut Queue<Request>, _select: Select) {}
