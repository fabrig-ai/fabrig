//! Notes and own-row stages (temper/docs/design/draft/playground-reset.md, 5).
use crate::{Core, Limits, RemovalStage, Request, reads};
use skein_lib::{Env, Queue};

pub(super) fn start_notes(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    super::enter(core, env, project, RemovalStage::Rows, out);
}
pub(super) fn loaded(
    _core: &mut Core,
    _env: &Env<Limits>,
    _project: u32,
    _result: reads::Loaded,
    _out: &mut Queue<Request>,
) {
}
pub(super) fn finish(_core: &mut Core, _env: &Env<Limits>, _project: u32, _out: &mut Queue<Request>) {}
