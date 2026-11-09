//! Preferences brief stub, filled by `party-brief`
//! (`jig/docs/design/domain/engine.md`, 10.1).
//! `gather` precedes the ordinary notes index; `plan` precedes its section;
//! `indexed` and `refused` own the Preferences note route. `text` supplies
//! the section's words through `runs/brief_text.rs`. Until filled, no
//! preference index is gathered or section planned.

use crate::{Core, Limits, Request, routing::Work};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_notes as notes;
use skein_lib::{Env, List, Queue};

/// Module-owned preparation state, bounded by the number of briefs.
#[derive(Hash, Debug)]
pub(crate) struct State;
impl State {
    pub(crate) fn new(_briefs: u32) -> Self {
        Self
    }
}

pub(crate) fn gather(core: &mut Core, _env: &Env<Limits>, _work: &mut Queue<Work>, _task: u64) -> bool {
    let State = &core.preferences;
    false
}

pub(crate) fn plan(_core: &Core, _limits: &Limits, _task: u64, _wanted: &mut List<brief::Planned>) {}

pub(crate) fn indexed(
    _core: &mut Core,
    _env: &Env<Limits>,
    _work: &mut Queue<Work>,
    _task: u64,
    _lines: List<notes::Line>,
    _more: u32,
) -> Queue<Request> {
    Queue::uncapped()
}

pub(crate) fn refused(
    _core: &mut Core,
    _env: &Env<Limits>,
    _work: &mut Queue<Work>,
    _task: u64,
    _why: notes::Refusal,
) -> Queue<Request> {
    Queue::uncapped()
}

pub(crate) fn text(_core: &Core, _task: u64) -> Option<Box<[u8]>> {
    None
}
