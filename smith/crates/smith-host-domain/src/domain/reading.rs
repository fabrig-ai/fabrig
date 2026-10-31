//! One channel read right through EOF (domain/host.md, section 6).
//! `domain::follow` demands another record only after the current terminal.
use core::mem;

#[derive(Debug)]
pub(super) enum State {
    Idle,
    Pending,
    Ended,
}

impl State {
    pub(super) fn pending(&self) -> bool {
        match self {
            Self::Pending => true,
            Self::Idle | Self::Ended => false,
        }
    }

    pub(super) fn ended(&self) -> bool {
        match self {
            Self::Ended => true,
            Self::Idle | Self::Pending => false,
        }
    }

    pub(super) fn demand(&mut self) {
        let state = mem::replace(self, Self::Ended);
        *self = match state {
            Self::Idle => Self::Pending,
            Self::Pending | Self::Ended => unreachable!("one read before its terminal, no read after EOF"),
        };
    }

    pub(super) fn received(&mut self) {
        let state = mem::replace(self, Self::Ended);
        *self = match state {
            Self::Pending => Self::Idle,
            Self::Idle | Self::Ended => unreachable!("a record or piece ends one requested read"),
        };
    }

    pub(super) fn end(&mut self) {
        let state = mem::replace(self, Self::Ended);
        *self = match state {
            Self::Pending => Self::Ended,
            Self::Idle | Self::Ended => unreachable!("EOF or malformed ends one requested read"),
        };
    }
}
