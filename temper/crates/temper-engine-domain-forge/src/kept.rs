//! Kept-act handoffs (temper/docs/design/draft/playground-reset.md, 4).
//! Sending and saved-answer handling are stubs for the kept-acts body.
use crate::{Domain, Request};
use skein_lib::Queue;

pub(crate) fn send_all(_d: &Domain, _out: &mut Queue<Request>) {}

pub(crate) fn answered(_d: &mut Domain, _key: [u8; 16], _keep: bool, _out: &mut Queue<Request>) {}

pub(crate) fn box_answered(_d: &mut Domain, _forge: u16, _removed: bool, _out: &mut Queue<Request>) {}
