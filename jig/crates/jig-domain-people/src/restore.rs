//! Selected start's volatile projections and keyed fit evidence (jig/docs/design/domain/engine.md, 6).
use crate::{Entry, Key, ReplyRef, ResultRef};
use alloc::boxed::Box;

#[derive(Debug)]
pub enum RestoreProjection {
    Waiting {
        task: u64,
        entries: Box<[Entry]>,
    },
    Result {
        party: u64,
        entry: ResultRef,
    },
    ChatState {
        party: u64,
        project: u32,
        task: u64,
        last_reply: u64,
        live: bool,
        seen: Option<u64>,
        replies: Box<[ReplyRef]>,
    },
    EndedConversation {
        task: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreFailure {
    pub key: Key,
    pub cause: RestoreCause,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreCause {
    Bound { which: RestoreBound, held: u64, limit: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreBound {
    IdentityBytes,
    People,
    SignIns,
    Holdings,
    Roles,
    ReadResults,
    ReadConversations,
    Requests,
    ModelNames,
    ModelNameBytes,
    Subagents,
    Operations,
    Batch,
    Words,
    AmendmentBytes,
    NameBytes,
    ProgramNameBytes,
    Uses,
    ProgramHosts,
    ProgramHostNameBytes,
    AddressBytes,
    NamedTasks,
    KeptPending,
}
