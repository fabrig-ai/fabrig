//! Exact connector references and executor identities (domain/tasks.md, 2–3;
//! client/domain/root.md, 6). Literal segments stay separate and are never parsed.
use alloc::boxed::Box;
use skein_lib::List;

/// Exact person destination; its project is supplied by the enclosing document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PersonAddress {
    Party(u64),
    Role(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Job tasks: jig/docs/design/client/domain/contract.md, 5.7.
pub enum Executor {
    Agent { charter: u32 },
    Procedure { connector: u16, code: u32 },
    Person(PersonAddress),
    Job { kind: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartKind {
    Resource,
    Pool,
    Result,
    Procedure,
    Effect,
    Outbox,
    /// The supplied connector account for the numbered held task.
    Hold,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PartKey {
    Path(List<Box<[u8]>>),
    Number(u64),
}

/// An application value's exact owner reference, without that value's contents.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Part {
    pub connector: u16,
    pub kind: PartKind,
    pub key: PartKey,
}
