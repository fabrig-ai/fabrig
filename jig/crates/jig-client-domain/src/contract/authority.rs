//! Owner authority facts without interpretation (client/domain/contract.md, 3.2 and 5).
//! Exact field vocabulary follows domain/authority.md, sections 3 and 4.
//! Document completeness belongs to the document wrapper; an admitted payload
//! carries only the authority. Literal pattern bytes are never rendered as paths.
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// Literal resource pattern, preserving segment boundaries and terminal bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pattern {
    pub segments: List<Box<[u8]>>,
    pub last: PatternEnd,
}
/// The owner's exact terminal or open terminal prefix.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PatternEnd {
    Exact(Box<[u8]>),
    Open(Box<[u8]>),
}
/// A permission for one connector's numbered kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Grant {
    pub connector: u16,
    pub kind: u16,
    pub pattern: Pattern,
}
/// Authority executor codes, distinct from a task's concrete executor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Job tasks: jig/docs/design/client/domain/contract.md, 5.7.
pub enum AuthorityExecutor {
    Charter(u32),
    Procedure(u32),
    Role(u32),
    Job(u32),
}
/// Delegation ceilings and the exact allowed executor codes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Delegation {
    pub kinds: List<AuthorityExecutor>,
    pub tasks: u32,
    pub depth: u32,
}
/// A resource note permission, independent of note identity and project scope.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceScope {
    pub connector: u16,
    pub pattern: Pattern,
}
pub const AUTHORITY_MODELS: u32 = 64;
pub const AUTHORITY_MODEL_BYTES: u32 = 4_096;

/// An explicit model selector; a named set may be empty.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AuthorityModels {
    All,
    Named(List<Box<[u8]>>),
}
/// Whole authority, with omission retaining the existing/default models.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Authority {
    pub models: Option<AuthorityModels>,
    pub tools: u64,
    pub grants: List<Grant>,
    pub delegation: Delegation,
    pub spend: u64,
    pub deadline: Option<Wall>,
    pub notes: u8,
    pub note_resources: List<ResourceScope>,
}
/// An owner document's authority projection and explicit cut metadata.
/// A cut projection does not establish a whole admitted replacement payload.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AuthorityDocument {
    pub authority: Authority,
    pub cut: bool,
}
impl AuthorityDocument {
    /// Mark an explicitly complete supplied value.
    #[must_use]
    pub const fn complete(authority: Authority) -> Self {
        Self { authority, cut: false }
    }
    /// A whole replacement exists only when this owner projection is complete.
    #[must_use]
    pub fn whole(&self) -> Option<Authority> {
        if self.cut { None } else { Some(self.authority.clone()) }
    }
}
