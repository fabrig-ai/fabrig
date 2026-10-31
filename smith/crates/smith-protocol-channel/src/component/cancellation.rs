//! One polite cancellation per activation (smith/docs/design/protocol/channel.md, section 5).
use crate::Error;
use core::mem;

#[derive(Debug)]
pub(super) enum Cancellation {
    Waiting,
    Requested,
}

impl Cancellation {
    pub(super) fn request(&mut self) -> Result<(), Error> {
        let state = mem::replace(self, Cancellation::Requested);
        *self = match state {
            Cancellation::Waiting => Cancellation::Requested,
            Cancellation::Requested => return Err(Error::Order),
        };
        Ok(())
    }
}
