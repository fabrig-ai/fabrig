//! Exact pending proposal facts (client/domain/objects.md, 2 and 4;
//! contract.md, 3.2 and 5). Ended routing and terminal decisions are
//! separate documents; missing funding observations remain absent.
use super::{
    AddressChange, AmendmentState, AuthorityDocument, ChipHold, DocumentText, Funder, HistoryHolder, Offered, Part,
    ProposalKey, ProposedTask, TaskLink,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Proposal {
    pub key: ProposalKey,
    pub project: u32,
    pub reason: DocumentText,
    pub action: ProposalAction,
    pub as_holder: bool,
    pub waiting: ProposalWaiting,
    pub funding: List<FundingOffer>,
    pub acts: List<Offered>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProposalWaiting {
    pub holder: HistoryHolder,
    pub since: Wall,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FundingOffer {
    pub source: Funder,
    pub needed: u64,
    pub left: Option<u64>,
    pub after: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ProposalAction {
    Settings(Box<super::ProposalActionSettings>),
    Batch(List<ProposedTask>),
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
        task: TaskLink,
        addressed: Option<AddressChange>,
        before: Box<AmendmentState>,
        after: Box<AmendmentState>,
        reason: DocumentText,
        release: Option<u64>,
    },
    Widen {
        task: TaskLink,
        before: AuthorityDocument,
        after: AuthorityDocument,
    },
    Release {
        task: TaskLink,
        hold: Option<Box<ChipHold>>,
    },
}
