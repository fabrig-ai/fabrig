//! A party's batch (domain/tasks/README.md, sections 3 and 4).
use crate::{Authority, Grant, Parameter, PersonAddress, WakePolicy};
use alloc::boxed::Box;
use skein_lib::length::len_u64;
/// A party's explicitly proposed action.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Batch(Box<[Member]>),
    /// A held deployment settings set, identified by its request digest.
    Settings {
        against: u32,
        digest: [u8; 32],
    },
    /// Amend a task at the displayed specification revision.
    Amend {
        task: u64,
        revision: u64,
        amendment: Box<crate::Amendment>,
    },
}

pub(crate) fn invalid(limits: &crate::Limits, batch: &[Member]) -> crate::LimitKind {
    if batch.is_empty() {
        crate::LimitKind::Empty
    } else if batch.len() > skein_lib::length::usize_of(limits.batch) {
        crate::LimitKind::Batch
    } else {
        crate::LimitKind::Words
    }
}

/// A connector's symbol: a resource it names after the task once the task has its
/// number (domain/authority.md, section 4; protocol/configuration.md, 3.3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Symbol {
    pub connector: u16,
    pub symbol: u16,
}

/// One root requested by a party, mirroring the tasks child's New.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub executor: TaskExecutor,
    pub words: Box<[u8]>,
    pub parameters: Box<[Parameter]>,
    pub inputs: Box<[u64]>,
    pub contract: Contract,
    pub authority: Authority,
    pub budget: Budget,
    pub deadline: Deadline,
    pub wake: Option<WakePolicy>,
    pub dependencies: Box<[Dependency]>,
    pub named: Box<[Grant]>,
    /// Connectors' symbols this member holds, each resolved by its connector once the task has its number
    /// (domain/tasks/holds.md, 2).
    pub symbols: Box<[Symbol]>,
    pub tracked: Option<u32>,
    pub label: u32,
    pub flags: Flags,
    /// The agent task's LLM choice; `None` takes its charter's default (domain/tasks/README.md, 3).
    pub llm: Option<crate::LlmChoice>,
}
/// Who carries out a member.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskExecutor {
    Agent { charter: u32 },
    Procedure { connector: u16, code: u32 },
    Person(PersonAddress),
}
/// The member's result contract, mirroring tasks' contracts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Contract {
    Report { words: u32 },
    Change { connector: u16, kind: u16, words: u32 },
    Verdict { choices: Box<[Verdict]> },
}
/// How much of the party's available budget to give a member.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Budget {
    Named,
    AsAllowed { cap: Option<u64> },
}
/// The request deadline retained until the task-making decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Deadline {
    Named,
    Span(skein_lib::Duration),
}
/// A dependency within the batch or the party's reach.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Dependency {
    Batch(u32),
    Existing(u64),
}
/// The resolved amount of a member whose budget was `AsAllowed`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Given {
    pub task: u64,
    pub amount: u64,
}
/// One allowed verdict.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Verdict {
    pub code: u32,
    pub words: u32,
    pub followups: u32,
    pub fields: Box<[FieldRule]>,
}
/// A required machine field.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FieldRule {
    pub name: Box<[u8]>,
    pub max: u32,
}
/// Task flags, mirroring the tasks child's flags.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Flags {
    pub conversation: bool,
    pub listed: bool,
    pub retention: Retention,
    pub ended_by: EndedBy,
}
impl Flags {
    pub const PLAIN: Flags =
        Flags { ended_by: EndedBy::Result, conversation: false, listed: false, retention: Retention::Kept };
}
/// A task's retention class.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Retention {
    Kept,
    ForgottenOnceRead,
}
/// Checked owned payload, excluding the inline member.
#[must_use]
pub fn member_bytes(member: &Member) -> Option<u64> {
    let mut total = crate::authority_bytes(&member.authority)?.checked_add(len_u64(member.words.len()))?;
    total = total
        .checked_add(len_u64(member.parameters.len()).checked_mul(u64::try_from(size_of::<Parameter>()).ok()?)?)?;
    total = total
        .checked_add(len_u64(member.dependencies.len()).checked_mul(u64::try_from(size_of::<Dependency>()).ok()?)?)?;
    total = total.checked_add(len_u64(member.inputs.len()).checked_mul(8)?)?;
    let mut named = member.authority.clone();
    named.grants.clone_from(&member.named);
    named.delegation.kinds = Box::new([]);
    named.note_resources = Box::new([]);
    total = total.checked_add(crate::authority_bytes(&named)?)?;
    total = total.checked_add(crate::llm_bytes(member.llm.as_ref())?)?;
    for parameter in &member.parameters {
        match parameter {
            Parameter::Bytes { value, .. } => total = total.checked_add(len_u64(value.len()))?,
            Parameter::Number { .. } | Parameter::Resource { .. } => {}
        }
    }
    match &member.contract {
        Contract::Report { .. } | Contract::Change { .. } => {}
        Contract::Verdict { choices } => {
            total =
                total.checked_add(len_u64(choices.len()).checked_mul(u64::try_from(size_of::<Verdict>()).ok()?)?)?;
            for choice in choices {
                total = total.checked_add(
                    len_u64(choice.fields.len()).checked_mul(u64::try_from(size_of::<FieldRule>()).ok()?)?,
                )?;
                for field in &choice.fields {
                    total = total.checked_add(len_u64(field.name.len()))?;
                }
            }
        }
    }
    Some(total)
}
pub(crate) fn valid(limits: &crate::Limits, batch: &[Member]) -> bool {
    if batch.is_empty() || batch.len() > skein_lib::length::usize_of(limits.batch) {
        return false;
    }
    for member in batch {
        if member.words.len() > skein_lib::length::usize_of(limits.spec_bytes)
            || !crate::llm::valid(member.llm.as_ref())
        {
            return false;
        }
        match member_bytes(member) {
            Some(bytes) if bytes <= u64::from(limits.amendment_bytes) => {}
            Some(_) | None => return false,
        }
    }
    true
}

/// The immutable authority that may end a task (domain/tasks/lifecycle.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndedBy {
    Result,
    Requester,
    /// A named procedure closes an agent task (jig/docs/design/domain/tasks/lifecycle.md, 11).
    Closer(Closer),
    /// A standing agent task ends only by cancel (jig/docs/design/domain/tasks/standing.md, 2).
    Standing,
}

/// The procedure named to close a task (jig/docs/design/domain/tasks/lifecycle.md, 11).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Closer {
    pub connector: u16,
    pub code: u32,
}

/// Ending authority fits its executor and conversation (jig/docs/design/domain/tasks/lifecycle.md, 9 and 11).
#[must_use]
pub fn shaped(member: &Member) -> bool {
    if member.flags.ended_by == EndedBy::Result {
        return true;
    }
    let agent = match member.executor {
        TaskExecutor::Agent { .. } => true,
        TaskExecutor::Person(_) | TaskExecutor::Procedure { .. } => false,
    };
    match member.flags.ended_by {
        EndedBy::Result => true,
        EndedBy::Requester => member.flags.conversation && agent,
        EndedBy::Closer(_) => !member.flags.conversation && agent,
        EndedBy::Standing => !member.flags.conversation && agent && member.tracked.is_none(),
    }
}
