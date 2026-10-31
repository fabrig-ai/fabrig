//! One chat's durable activation and transient lifecycle (domain/host.md,
//! sections 2, 8 and 9). Startup owns grants and the head survey; a live run
//! owns its repository heads, and an answered run retains its answer until saves settle.
use crate::credentials::Grants;
use crate::{ChatState, CredentialFailure};
use alloc::boxed::Box;
use skein_lib::{Map, Rng, Time};
use smith_domain::run::Identity;

/// One invocation's chat phase and the data held until its next transition.
#[derive(Debug)]
pub(crate) enum State {
    /// Waiting for the store's typed transcript.
    Loading(Load),
    /// Ready for a person line or exit.
    Idle,
    /// Saving names and obtaining grants before the child starts.
    Starting(Starting),
    /// Child run is live and can receive named messages.
    Running { live: Live, deadline: Time },
    /// A cancellation was sent; the child's answer is still owed.
    Ending(Live),
    /// The answer arrived; saves and the child end may still be owed.
    Answered { answer: smith_host_domain::Answer, detail: Box<[u8]> },
    /// Final exit was emitted.
    Done,
}

/// The admission work retained before spawning the child.
#[derive(Debug)]
pub(crate) struct Starting {
    pub(crate) grants: Grants,
    pub(crate) grant_failed: Option<(u32, CredentialFailure)>,
    pub(crate) saved: bool,
    pub(crate) survey: HeadSurvey,
    pub(crate) heads: Map<u32, Box<[u8]>>,
}

/// Writable repository heads retained throughout one running or ending activation.
#[derive(Debug)]
pub(crate) struct Live {
    pub(crate) heads: Map<u32, Box<[u8]>>,
    pub(crate) own_heads: Map<u32, Box<[u8]>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HeadSurvey {
    NotStarted,
    Ready(u32),
    Reading(u32),
    Complete,
}

/// The store request right owned while loading a selected chat.
#[derive(Debug)]
pub(crate) enum Load {
    Ready,
    Waiting,
}

#[derive(Debug)]
enum Metadata {
    Ready,
    Pending,
}

#[derive(Debug)]
enum Input {
    Open,
    Closed,
}

/// Durable metadata and independent store/input terminal rights.
#[derive(Debug)]
pub(crate) struct Chat {
    pub(crate) state: ChatState,
    pub(crate) phase: State,
    metadata: Metadata,
    input: Input,
}

impl Chat {
    pub(crate) fn new(seed: u64) -> Chat {
        let mut random = Rng::new(seed);
        let low = u128::from(random.next_u64());
        let high = u128::from(random.next_u64()).checked_shl(64).expect("fixed upper half shift");
        let identity = Identity::new((low | high).to_le_bytes());
        Chat {
            state: ChatState { identity, activation: 0, next_message: 0, read: None },
            phase: State::Loading(Load::Ready),
            metadata: Metadata::Ready,
            input: Input::Open,
        }
    }

    pub(crate) fn load_requested(&self) -> bool {
        match &self.phase {
            State::Loading(load) => match load {
                Load::Ready => false,
                Load::Waiting => true,
            },
            State::Idle
            | State::Starting(_)
            | State::Running { .. }
            | State::Ending(_)
            | State::Answered { .. }
            | State::Done => false,
        }
    }

    pub(crate) fn request_load(&mut self) {
        let state = core::mem::replace(&mut self.phase, State::Done);
        self.phase = match state {
            State::Loading(Load::Ready) => State::Loading(Load::Waiting),
            State::Loading(Load::Waiting)
            | State::Idle
            | State::Starting(_)
            | State::Running { .. }
            | State::Ending(_)
            | State::Answered { .. }
            | State::Done => unreachable!("one request owns the selected chat's load"),
        };
    }

    pub(crate) fn state_pending(&self) -> bool {
        match self.metadata {
            Metadata::Ready => false,
            Metadata::Pending => true,
        }
    }

    pub(crate) fn metadata_started(&mut self) {
        let state = core::mem::replace(&mut self.metadata, Metadata::Ready);
        self.metadata = match state {
            Metadata::Ready => Metadata::Pending,
            Metadata::Pending => unreachable!("one metadata save at a time"),
        };
    }

    pub(crate) fn metadata_saved(&mut self) {
        let state = core::mem::replace(&mut self.metadata, Metadata::Ready);
        self.metadata = match state {
            Metadata::Pending => Metadata::Ready,
            Metadata::Ready => unreachable!("one terminal for each metadata save"),
        };
    }

    pub(crate) fn closed(&self) -> bool {
        match self.input {
            Input::Open => false,
            Input::Closed => true,
        }
    }

    pub(crate) fn close_input(&mut self) {
        let state = core::mem::replace(&mut self.input, Input::Closed);
        self.input = match state {
            Input::Open | Input::Closed => Input::Closed,
        };
    }

    pub(crate) fn starting(&self) -> &Starting {
        match &self.phase {
            State::Starting(starting) => starting,
            State::Loading(_)
            | State::Idle
            | State::Running { .. }
            | State::Ending(_)
            | State::Answered { .. }
            | State::Done => unreachable!("startup owns the admission work"),
        }
    }

    pub(crate) fn live(&self) -> &Live {
        match &self.phase {
            State::Running { live, .. } | State::Ending(live) => live,
            State::Loading(_) | State::Idle | State::Starting(_) | State::Answered { .. } | State::Done => {
                unreachable!("delivery belongs to a live activation")
            }
        }
    }

    pub(crate) fn live_mut(&mut self) -> &mut Live {
        match &mut self.phase {
            State::Running { live, .. } | State::Ending(live) => live,
            State::Loading(_) | State::Idle | State::Starting(_) | State::Answered { .. } | State::Done => {
                unreachable!("delivery belongs to a live activation")
            }
        }
    }

    pub(crate) fn deadline(&self) -> Option<Time> {
        match self.phase {
            State::Running { deadline, .. } => Some(deadline),
            State::Loading(_)
            | State::Idle
            | State::Starting(_)
            | State::Ending(_)
            | State::Answered { .. }
            | State::Done => None,
        }
    }
}

impl State {
    pub(crate) fn is_loading(&self) -> bool {
        match self {
            Self::Loading(_) => true,
            Self::Idle
            | Self::Starting(_)
            | Self::Running { .. }
            | Self::Ending(_)
            | Self::Answered { .. }
            | Self::Done => false,
        }
    }
    pub(crate) fn is_idle(&self) -> bool {
        match self {
            Self::Idle => true,
            Self::Loading(_)
            | Self::Starting(_)
            | Self::Running { .. }
            | Self::Ending(_)
            | Self::Answered { .. }
            | Self::Done => false,
        }
    }
    pub(crate) fn is_starting(&self) -> bool {
        match self {
            Self::Starting(_) => true,
            Self::Loading(_)
            | Self::Idle
            | Self::Running { .. }
            | Self::Ending(_)
            | Self::Answered { .. }
            | Self::Done => false,
        }
    }
    pub(crate) fn is_running(&self) -> bool {
        match self {
            Self::Running { .. } => true,
            Self::Loading(_)
            | Self::Idle
            | Self::Starting(_)
            | Self::Ending(_)
            | Self::Answered { .. }
            | Self::Done => false,
        }
    }
    pub(crate) fn is_answered(&self) -> bool {
        match self {
            Self::Answered { .. } => true,
            Self::Loading(_) | Self::Idle | Self::Starting(_) | Self::Running { .. } | Self::Ending(_) | Self::Done => {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Chat;

    #[test]
    fn a_new_chat_mints_one_replayable_identity_from_its_own_seed() {
        let first = Chat::new(8084);
        assert_eq!(first.state.identity, Chat::new(8084).state.identity);
        assert_ne!(first.state.identity, Chat::new(8085).state.identity);
    }
}
