//! Printed units (hearth/docs/design/domain/render/README.md, 3.8, and hearth/docs/design/domain/render/spec.md, 1).
use crate::{errors::Error, outcome::Outcome, startup::Startup};
use alloc::boxed::Box;
use hearth_domain_render_vocabulary::{
    diff::{Change, Tally},
    parts::Spec,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Record {
    Resolving(crate::progress::Resolving),
    Resolved(crate::progress::Resolved),
    Unresolved(crate::progress::Unresolved),
    Waiting(crate::progress::Waiting),
    Strays(crate::progress::Strays),
    Spec(Box<Spec>),
    Terminal(Terminal),
    Change(Change),
    Error(Error),
    Startup(Startup),
    Layer(crate::build::Layer),
    Failed(crate::build::Failed),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    Resolving,
    Resolved,
    Unresolved,
    Waiting,
    Strays,
    Spec,
    Terminal,
    Change,
    Error,
    Startup,
    Layer,
    Failed,
}
impl Record {
    #[must_use]
    pub fn kind(&self) -> Type {
        match self {
            Self::Resolving(_) => Type::Resolving,
            Self::Resolved(_) => Type::Resolved,
            Self::Unresolved(_) => Type::Unresolved,
            Self::Waiting(_) => Type::Waiting,
            Self::Strays(_) => Type::Strays,
            Self::Spec(_) => Type::Spec,
            Self::Terminal(_) => Type::Terminal,
            Self::Change(_) => Type::Change,
            Self::Error(_) => Type::Error,
            Self::Startup(_) => Type::Startup,
            Self::Layer(_) => Type::Layer,
            Self::Failed(_) => Type::Failed,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Terminal {
    pub outcome: Outcome,
    pub diff: Option<Tally>,
    pub left_out: u32,
}
