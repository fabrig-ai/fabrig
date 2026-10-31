//! Run admission and the parent's one terminal (domain/host.md, sections 3 and 6).
//! The process may keep running after this lifecycle ends. `domain::follow`
//! applies process demands and deadlines after each entrance changes a word.
use core::mem;

#[derive(Clone, Copy, Debug)]
pub(super) enum Admission {
    Pending,
    Admitted,
}

#[derive(Debug)]
pub(super) enum State {
    Open(Admission),
    Dropped(Admission),
    Answered,
    Faulted,
}

impl State {
    pub(super) fn admitted(&self) -> bool {
        match self {
            Self::Open(Admission::Admitted) | Self::Dropped(Admission::Admitted) => true,
            Self::Open(Admission::Pending) | Self::Dropped(Admission::Pending) | Self::Answered | Self::Faulted => {
                false
            }
        }
    }

    pub(super) fn last_word(&self) -> bool {
        match self {
            Self::Answered => true,
            Self::Open(_) | Self::Dropped(_) | Self::Faulted => false,
        }
    }

    pub(super) fn told(&self) -> bool {
        match self {
            Self::Answered | Self::Faulted => true,
            Self::Open(_) | Self::Dropped(_) => false,
        }
    }

    pub(super) fn dropped(&self) -> bool {
        match self {
            Self::Dropped(_) => true,
            Self::Open(_) | Self::Answered | Self::Faulted => false,
        }
    }

    pub(super) fn admit(&mut self) {
        let state = mem::replace(self, Self::Faulted);
        *self = match state {
            Self::Open(Admission::Pending) => Self::Open(Admission::Admitted),
            Self::Dropped(Admission::Pending) => Self::Dropped(Admission::Admitted),
            Self::Open(Admission::Admitted) | Self::Dropped(Admission::Admitted) | Self::Answered | Self::Faulted => {
                unreachable!("admission is validated before transition")
            }
        };
    }

    pub(super) fn answer(&mut self) {
        let state = mem::replace(self, Self::Faulted);
        *self = match state {
            Self::Open(_) | Self::Dropped(_) | Self::Faulted => Self::Answered,
            Self::Answered => unreachable!("one last word per run"),
        };
    }

    pub(super) fn fault(&mut self) {
        let state = mem::replace(self, Self::Faulted);
        *self = match state {
            Self::Open(_) | Self::Dropped(_) => Self::Faulted,
            Self::Answered | Self::Faulted => unreachable!("the parent's terminal is still owed"),
        };
    }

    pub(super) fn drop_run(&mut self) {
        let state = mem::replace(self, Self::Faulted);
        *self = match state {
            Self::Open(admission) | Self::Dropped(admission) => Self::Dropped(admission),
            terminal @ (Self::Answered | Self::Faulted) => terminal,
        };
    }
}
