//! Party acts judged from current engine facts (`client/domain/contract/requests.md`, section 2, and `client/domain/contract/documents.md`, section 3
//! ; `domain/people/acts.md`, section 1).
//! No state, drafts, keys or document cache is retained. A request and a
//! document use the same judgments. Draft-dependent acts carry numerical and
//! listed bounds; the client compares its form with these engine facts.

pub mod effects;

use crate::documents;
use alloc::boxed::Box;
pub use effects::{Effect, Impact, ReleaseEffect};
use jig_domain_people as people;
use skein_lib::{List, Wall};

/// An act in the client's closed order, translated between vocabularies by the wire.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Act {
    /// Delete the project (jig/docs/design/domain/people/acts.md, 2).
    RemoveProject,
    /// Write to an agent.
    Message,
    Talk,
    Leave,
    /// Answer a question or person task.
    Answer,
    /// Take a role task.
    Take,
    /// Hand a task back.
    HandBack,
    /// Accept a proposal.
    Accept,
    /// Reject a proposal.
    Reject,
    /// Pass a decision up.
    PassUp,
    /// Release a held task.
    Release,
    /// Leave a task held.
    LeaveHeld,
    /// Amend a specification.
    Amend,
    /// Stop a live run.
    Stop,
    /// Cancel a subtree.
    Cancel,
    /// Become a requester.
    Move,
    /// Reorder tracked work in its project.
    Reorder,
    /// Mark updates read.
    Seen,
    /// Edit a note.
    EditNote,
    /// Delete a note.
    DeleteNote,
    /// Adopt a resource.
    Adopt,
    /// Change policy.
    ChangePolicy,
    /// Rename the project.
    Rename,
    /// Change roles.
    SetRoles,
    /// Make a service.
    MakeService,
    /// Set a pool.
    SetPool,
    /// Withdraw the party's own pending proposal.
    Withdraw,
}

/// Proposal and escalation decisions (`client/domain/contract/documents.md`, section 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decision {
    Accept,
    Reject,
    PassUp,
    Release,
    LeaveHeld,
}

impl Act {
    /// Select a decision without changing the act's object-specific admission.
    #[must_use]
    pub fn decision(self) -> Option<Decision> {
        match self {
            Self::Accept => Some(Decision::Accept),
            Self::Reject => Some(Decision::Reject),
            Self::PassUp => Some(Decision::PassUp),
            Self::Release => Some(Decision::Release),
            Self::LeaveHeld => Some(Decision::LeaveHeld),
            Self::RemoveProject
            | Self::Talk
            | Self::Leave
            | Self::Message
            | Self::Answer
            | Self::Take
            | Self::HandBack
            | Self::Amend
            | Self::Stop
            | Self::Cancel
            | Self::Move
            | Self::Reorder
            | Self::Seen
            | Self::EditNote
            | Self::DeleteNote
            | Self::Adopt
            | Self::ChangePolicy
            | Self::Rename
            | Self::SetRoles
            | Self::MakeService
            | Self::SetPool
            | Self::Withdraw => None,
        }
    }
}

/// What a current engine judgment says about an act.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Admission {
    /// The act is made directly.
    Allowed,
    /// The act becomes a proposal waiting at this holder.
    Proposed { to: Recipient },
    /// The act is refused with the same facts as its request.
    Refused(people::Refusal),
    /// The draft is compared with supplied engine facts before confirmation.
    Bounded { within: Bounds, beyond: Beyond },
}

/// Configured connector and permitted procedure code for the existing goal form.
/// This is an authority fact, not a connector's procedure schema or display name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CreateProcedure {
    pub connector: u16,
    pub code: u32,
}

/// Creating bounds and the judgement beyond the first (client/domain/contract/documents.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Creating {
    pub within: Bounds,
    pub beyond: Beyond,
}

/// A draft's numerical and listed bounds, supplied by the engine.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Bounds {
    /// A goal's direct funding, executor and deadline choices.
    Create {
        authority: Box<documents::AuthorityLine>,
        pool_left: u64,
        charters: List<u32>,
        procedures: List<CreateProcedure>,
        latest: Option<Wall>,
        budget: u64,
        maximum_budget: u64,
        maximum_deadline: Option<Wall>,
    },
    /// Authority the party may add to the task's current authority.
    Amend {
        budget_left: u64,
        grants: List<documents::GrantLine>,
        /// Complete configured connector-kind relation; equality is implicit.
        implications: List<jig_domain_authority::Implication>,
        authority: Box<documents::AuthorityLine>,
        ceiling: Box<documents::AuthorityLine>,
    },
}

/// The result when a supported draft exceeds the direct bounds.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Beyond {
    /// The draft becomes a proposal for this holder.
    Proposed { to: Recipient },
    /// The draft is refused with these engine facts.
    Refused(people::Refusal),
}

/// A holder named by current routing policy.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Recipient {
    /// A task above.
    Task(u64),
    /// A named party.
    Party(u64),
    /// A role in a project.
    Role { project: u32, role: u32 },
    /// Every policy role eligible for this collective decision.
    Roles { project: u32, roles: List<u32> },
}

/// One act and its current engine judgment.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Offered {
    pub effect: Option<Effect>,
    pub act: Act,
    pub admission: Admission,
}
