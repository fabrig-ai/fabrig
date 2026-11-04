//! One configured forge and its dialect's state (skein/docs/design/forge-connection.md, 7.1).
use super::{OwnPlan, Start, initial};
use crate::{Limits, settings::Forge};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};

#[derive(Debug)]
pub(crate) struct ForgeEntry {
    pub base: Base,
    pub dialect: Dialect,
}

/// What every forge has, whichever its dialect.
#[derive(Debug)]
pub(crate) struct Base {
    pub forge: Forge,
    pub start: Start,
    pub own: Option<Token>,
}

/// A forge's dialect, and what only that dialect has.
#[derive(Debug)]
pub(crate) enum Dialect {
    Forgejo { plan: Option<skein_forge::forgejo::plans::start::Plan> },
    GitHub(Box<GitHubForge>),
}

#[derive(Debug)]
pub(crate) struct GitHubForge {
    pub state: crate::installations::GitHub,
    pub plan: Option<OwnPlan>,
}

impl ForgeEntry {
    /// A forge as it's added: its start from its access, its dialect from its kind.
    pub fn new(forge: Forge, limits: &Limits) -> Self {
        let start = if forge.access.is_some() { initial(forge.kind, limits.start_backoff) } else { Start::Unconnected };
        let dialect = match forge.kind {
            crate::settings::Kind::Forgejo => Dialect::Forgejo { plan: None },
            crate::settings::Kind::GitHub => {
                let mut state = crate::installations::GitHub::new(limits.installations);
                state.said_origins = Queue::with_capacity(limits.log_origins);
                Dialect::GitHub(Box::new(GitHubForge { state, plan: None }))
            }
        };
        Self { base: Base { forge, start, own: None }, dialect }
    }

    pub fn github(&self) -> Option<&GitHubForge> {
        match &self.dialect {
            Dialect::Forgejo { .. } => None,
            Dialect::GitHub(github) => Some(github),
        }
    }

    pub fn github_mut(&mut self) -> Option<&mut GitHubForge> {
        match &mut self.dialect {
            Dialect::Forgejo { .. } => None,
            Dialect::GitHub(github) => Some(github),
        }
    }
}
