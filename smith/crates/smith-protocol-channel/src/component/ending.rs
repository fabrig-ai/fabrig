//! The channel's single terminal notification (smith/docs/design/protocol/channel.md, section 2).
use crate::OpenEvent;
use core::mem;
use skein_channel::Closed;
use skein_lib::Queue;

#[derive(Debug)]
pub(super) enum Ending {
    Listening,
    Reported,
}

impl Ending {
    pub(super) const fn reported(&self) -> bool {
        match self {
            Ending::Listening => false,
            Ending::Reported => true,
        }
    }

    pub(super) fn end(&mut self, why: Closed, to_service: &mut Queue<OpenEvent>) {
        let state = mem::replace(self, Ending::Reported);
        *self = match state {
            Ending::Listening => {
                to_service.push(OpenEvent::Ended { why });
                Ending::Reported
            }
            Ending::Reported => Ending::Reported,
        };
    }
}
