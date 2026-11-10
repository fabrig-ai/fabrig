//! One configured forge and its dialect's state (skein/docs/design/forge/connection.md, 7.1).
use super::{OwnPlan, Start, initial};
use crate::{Limits, settings::Forge};
use alloc::boxed::Box;
use skein_lib::{Queue, Token};

#[derive(Debug)]
pub(crate) struct ForgeEntry {
    pub(crate) base: Base,
    pub(crate) dialect: Dialect,
}

/// What every forge has, whichever its dialect.
#[derive(Debug)]
pub(crate) struct Base {
    pub(crate) forge: Forge,
    pub(crate) start: Start,
    pub(crate) own: Option<Token>,
}

/// A forge's dialect, and what only that dialect has.
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "the Forgejo variant holds its own plan and exchange together")]
pub(crate) enum Dialect {
    Forgejo { plan: Option<skein_forge::forgejo::plans::start::Plan>, exchange: crate::connection::forgejo::Exchange },
    GitHub(Box<GitHubForge>),
}

#[derive(Debug)]
pub(crate) struct GitHubForge {
    pub(crate) exchange: crate::connection::github::Exchange,
    pub(crate) state: crate::installations::GitHub,
    pub(crate) plan: Option<OwnPlan>,
}

impl Dialect {
    /// The forge's own exchange goes with its connection (forge/connection.md, 7.1).
    pub(crate) fn forget_exchange(&mut self) {
        match self {
            Self::Forgejo { exchange, .. } => *exchange = crate::connection::forgejo::Exchange::new(),
            Self::GitHub(gh) => gh.exchange = crate::connection::github::Exchange::new(),
        }
    }
}

impl ForgeEntry {
    /// A forge as it's added: its start from its access, its dialect from its kind.
    pub(crate) fn new(forge: Forge, limits: &Limits) -> Self {
        let dialect = match forge.kind {
            crate::settings::Kind::Forgejo => {
                Dialect::Forgejo { plan: None, exchange: crate::connection::forgejo::Exchange::new() }
            }
            crate::settings::Kind::GitHub => {
                let mut state = crate::installations::GitHub::new(limits.installations);
                state.said_origins = Queue::with_capacity(limits.log_origins);
                Dialect::GitHub(Box::new(GitHubForge {
                    state,
                    plan: None,
                    exchange: crate::connection::github::Exchange::new(),
                }))
            }
        };
        let start = if forge.access.is_some() { initial(&dialect, limits.start_backoff) } else { Start::Unconnected };
        Self { base: Base { forge, start, own: None }, dialect }
    }
}
