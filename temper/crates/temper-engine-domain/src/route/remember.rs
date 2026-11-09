//! The asking family's memory call (temper/docs/design/domain/agent.md, 8.4.4).
use crate::{CallAnswer, Domain, RememberAction, RememberRefusal, Remembered};
use alloc::boxed::Box;
use skein_lib::ReplyTo;

pub(crate) fn call(domain: &mut Domain, to: ReplyTo, _action: RememberAction) {
    super::host_route::read(domain, to, CallAnswer::Remember(Box::new(Remembered::Refused(RememberRefusal::Unread))));
}
