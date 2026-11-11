//! Routes for run preparation and named calls (`jig/docs/design/domain/engine/README.md`, 4.4).

pub(crate) mod calls;
pub(crate) mod controls;
pub(crate) mod delegation;
pub(crate) mod jobs;
pub(crate) mod now;
pub(crate) mod parts;
pub(crate) mod payloads;
pub(crate) mod preferences;
pub(crate) mod preparation;
pub(crate) mod results;
pub(crate) mod turns;

use crate::CallKey;
use alloc::boxed::Box;
use skein_lib::ReplyTo;

pub(crate) struct Call {
    pub(crate) to: ReplyTo,
    pub(crate) key: CallKey,
}

pub(crate) struct Proposing {
    pub(crate) to: ReplyTo,
    pub(crate) key: CallKey,
    pub(crate) reason: Box<[u8]>,
    pub(crate) as_holder: bool,
}
