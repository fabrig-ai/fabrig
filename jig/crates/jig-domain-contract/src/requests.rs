//! Values carried by an application request (jig/docs/design/domain/connectors.md, 2).

use crate::ops::{Outcome, Role, party::Again};
use skein_lib::{Duration, List, Token};

/// Role scope named by an application's client (`client/domain/contract/watches.md`, 1.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ApplicationScope {
    /// A party must hold a role in this project.
    Project(u32),
    /// A party must hold a deployment role.
    Deployment,
    /// The signed-in party's values across projects, without a role check.
    Own,
}

impl ApplicationScope {
    /// The people child's project identity for this scope.
    #[must_use]
    pub fn project(self) -> u32 {
        match self {
            Self::Project(project) => project,
            Self::Deployment => 0,
            Self::Own => unreachable!("own scope has no project"),
        }
    }
}

/// What the engine asks of an application request connector.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    /// Operation that ends later: decide the admitted request and answer with `Up::Decided`.
    /// The connector can ask for the request's core part first (`ApplicationPart` through `In::Core`).
    Decide {
        request: Token,
        token: Token,
        owner: Token,
        person: u64,
        scope: ApplicationScope,
        role: Option<Role>,
        retention: Duration,
    },
    /// Notification: drop the prepared request; no answer.
    Drop { owner: Token },
    /// Notification: the request's core part completed; no answer to the notification.
    PartAnswered { request: Token, outcome: Outcome },
}

/// An application request connector's decisions and change marks.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Up {
    /// The retained outcome of the request, or why it may need trying again.
    Decided { request: Token, outcome: Result<Token, Again> },
    /// Unasked: the subjects that changed.
    Marked { subjects: List<Token> },
}
