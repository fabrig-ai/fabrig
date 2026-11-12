//! One turn drain before the one polite cancellation per activation (smith/docs/design/protocol/channel.md, section 5).
use crate::Error;
use core::mem;

#[derive(Debug)]
pub(super) enum Cancellation {
    Waiting,
    Requested,
    Draining,
}

impl Cancellation {
    pub(super) fn drain(&mut self) -> Result<(), Error> {
        match self {
            Self::Waiting => { *self = Self::Draining; Ok(()) }
            Self::Draining | Self::Requested => Err(Error::Order),
        }
    }

    pub(super) fn request(&mut self) -> Result<(), Error> {
        let state = mem::replace(self, Cancellation::Requested);
        *self = match state {
            Cancellation::Waiting | Cancellation::Draining => Cancellation::Requested,
            Cancellation::Requested => return Err(Error::Order),
        };
        Ok(())
    }
}
