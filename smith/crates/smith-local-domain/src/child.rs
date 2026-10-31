//! The child's token and settlement evidence (domain/host.md, sections 6 and 9).
//! A token is held only between Started and Gone; a failure survives Gone.
use crate::AgentFailure;
use core::mem::replace;
use skein_lib::Token;

#[derive(Debug)]
pub(crate) enum Child {
    Starting,
    Running { run: Token },
    Unstarted { failure: AgentFailure },
    Failed { run: Token, failure: AgentFailure },
    Gone { failure: Option<AgentFailure> },
}

impl Child {
    pub(crate) fn run(&self) -> Option<Token> {
        match self {
            Self::Running { run } | Self::Failed { run, .. } => Some(*run),
            Self::Starting | Self::Unstarted { .. } | Self::Gone { .. } => None,
        }
    }

    pub(crate) fn failure(&self) -> Option<AgentFailure> {
        match self {
            Self::Unstarted { failure } | Self::Failed { failure, .. } => Some(*failure),
            Self::Gone { failure } => *failure,
            Self::Starting | Self::Running { .. } => None,
        }
    }

    pub(crate) fn is_gone(&self) -> bool {
        match self {
            Self::Gone { .. } => true,
            Self::Starting | Self::Running { .. } | Self::Unstarted { .. } | Self::Failed { .. } => false,
        }
    }

    pub(crate) fn started(&mut self, run: Token) {
        let state = replace(self, Self::Gone { failure: None });
        *self = match state {
            Self::Starting => Self::Running { run },
            Self::Unstarted { failure } => Self::Failed { run, failure },
            Self::Running { .. } | Self::Failed { .. } | Self::Gone { .. } => unreachable!("one Started before Gone"),
        };
    }

    pub(crate) fn faulted(&mut self, failure: AgentFailure) {
        let state = replace(self, Self::Gone { failure: None });
        *self = match state {
            Self::Starting | Self::Unstarted { .. } => Self::Unstarted { failure },
            Self::Running { run } | Self::Failed { run, .. } => Self::Failed { run, failure },
            Self::Gone { .. } => Self::Gone { failure: Some(failure) },
        };
    }

    pub(crate) fn gone(&mut self) {
        let state = replace(self, Self::Gone { failure: None });
        *self = match state {
            Self::Starting | Self::Running { .. } => Self::Gone { failure: None },
            Self::Unstarted { failure } | Self::Failed { failure, .. } => Self::Gone { failure: Some(failure) },
            Self::Gone { failure } => Self::Gone { failure },
        };
    }
}
