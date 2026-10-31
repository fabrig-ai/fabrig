//! Pending person work and shown results (client/domain/objects.md, 7;
//! contract.md, 5–6). Optional facts and source order stay independent.
use super::{
    Chip, Offered, OwnerRequester, OwnerTaskLink, PersonAddress, Result, Revision, SpecificationDocument,
    TaskContractDocument,
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerShownResult {
    pub task: OwnerTaskLink,
    pub result: Option<Box<Result>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnerPersonTask {
    pub task: Chip,
    pub revision: Revision,
    pub requester: OwnerRequester,
    pub spec: SpecificationDocument,
    pub address: PersonAddress,
    pub taken_by: Option<u64>,
    pub since: Option<Wall>,
    pub stall: Option<Duration>,
    pub deadline: Option<Wall>,
    pub contract: TaskContractDocument,
    pub results: List<OwnerShownResult>,
    pub cut: bool,
    pub acts: List<Offered>,
}
impl OwnerPersonTask {
    /// Routing comes only from the supplied task metadata.
    #[must_use]
    pub fn project(&self) -> Option<u32> {
        Some(self.task.details.details.project)
    }
}
