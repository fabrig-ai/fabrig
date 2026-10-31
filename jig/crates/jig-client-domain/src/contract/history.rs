//! Whole immutable semantic history (client/domain/objects.md, 11;
//! domain/tasks.md, 7). Revision is not an activity commit position.
use super::{
    AddressChange, AmendmentState, AuthorityDocument, DocumentText, MessageProposalKind, Part, ProposedTask, Requester,
    SpecificationDocument, WakePolicy,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// One immutable semantic revision and its independently supplied retained facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HistoryLine {
    pub task: u64,
    pub revision: u64,
    pub by: Requester,
    pub change: HistoryChange,
    pub reason: DocumentText,
    pub proposal: Option<ProposalHistory>,
    /// Stored Settings snapshots, not a reconstruction from the selected delta.
    /// Absent instructions mean no stored override, not today's effective text.
    pub amendment: Option<(Box<AmendmentState>, Box<AmendmentState>)>,
    pub at: Option<Wall>,
}
/// The owner's ten recorded semantic changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HistoryChange {
    Cancelled,
    Released,
    Amended,
    Moved,
    Proposed,
    ProposalPassed,
    ProposalAccepted,
    ProposalRejected,
    ProposalWithdrawn,
}
/// One retained task-origin proposal and its actual decision state.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProposalHistory {
    pub action: RetainedAction,
    pub number: u64,
    /// Raw task-origin proposer number; the source supplies no maker discriminator.
    pub proposer: u64,
    pub as_holder: bool,
    pub state: ProposalHistoryState,
}
/// The recorded holder or decision, without invented actor or clock facts.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProposalHistoryState {
    Pending { holder: HistoryHolder, since: Wall },
    Accepted { by: Requester },
    Rejected { by: Requester, reason: DocumentText },
    Withdrawn,
}
/// The exact named holder category, independent of equal numeric identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HistoryHolder {
    Task(u64),
    Person(u64),
    Policy { project: u32, kind: MessageProposalKind },
}
/// The whole retained action, distinct from a live request or inferred scope.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RetainedAction {
    Settings(super::RetainedActionSettings),
    /// Whole admitted batch; no synthetic outer loss marker or clipped batch.
    Batch(List<ProposedTask>),
    /// Raw owner reference; application scope is not supplied by this source.
    Effect {
        part: Part,
        authority: AuthorityDocument,
        attempt: u64,
        completion: u32,
        position: u32,
        purpose: u64,
    },
    ProcedureEffect {
        part: Part,
        authority: AuthorityDocument,
        step: u64,
        purpose: u64,
    },
    Amend {
        task: u64,
        change: Box<RetainedAmendment>,
    },
    Widen {
        task: u64,
        authority: AuthorityDocument,
    },
    Release {
        task: u64,
    },
}
/// Selected retained replacements, distinct from immutable stored settings snapshots.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RetainedAmendment {
    pub addressed: Option<AddressChange>,
    pub instructions: Option<DocumentText>,
    /// Named hold revision, independent of the person address and history revisions.
    pub release: Option<u64>,
    pub spec: Option<SpecificationDocument>,
    pub authority: Option<AuthorityDocument>,
    pub wake: Option<WakePolicy>,
    pub dependencies: Option<List<u64>>,
    pub reason: DocumentText,
    /// Dependency-list omission only; nested text/spec/authority loss stays independent.
    pub cut: bool,
}
