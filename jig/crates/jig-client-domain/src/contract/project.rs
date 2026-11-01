//! Whole project owner facts; client/domain/contract.md, 5.1 and 5.3.
//! Lists retain their source order and allocated capacities. Neither a name,
//! a period nor permission mappings are inferred from these values.
use super::{
    AuthorityDocument, Chip, DocumentText, Funder, Grant, HeldRole, Numbers, Offered, Part, PartyKind,
    PolicyPermission, PolicyRequirement, PoolLedger, Recurring,
};
use skein_lib::List;

/// The current project supplied by its owner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Project {
    pub acts: List<Offered>,
    pub number: u32,
    pub name: DocumentText,
    pub resources: List<ProjectResource>,
    pub policy: ProjectPolicy,
    pub members: List<ProjectMember>,
    pub current_period: Option<PoolLedger>,
    pub pools: List<PoolLedger>,
    pub configured_period: u64,
    pub configured_pools: List<PoolLedger>,
    pub configured_pools_cut: bool,
    pub spend: List<ProjectSpend>,
    pub recurring: List<Recurring>,
    pub cut: bool,
}
/// One resource's whole connector part and independently supplied role.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectResource {
    pub part: Part,
    pub role: ProjectResourceRole,
    pub home: bool,
}
/// The roles supplied by the resource owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProjectResourceRole {
    Owned,
    Fork,
    Participant,
    Unavailable,
    Context,
}
/// Policy facts, with a loss flag independent of each authority document.
/// Missing mappings remain absent; reserved lists and loss remain independent.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectPolicy {
    pub templates: List<super::TemplateOverride>,
    pub ceiling: AuthorityDocument,
    pub period_spend: u64,
    pub escalation_role: Option<u32>,
    pub projections: List<Grant>,
    pub roles: List<ProjectPolicyRole>,
    pub requirements: List<PolicyRequirement>,
    pub permissions: Option<List<PolicyPermission>>,
    pub permissions_cut: bool,
    pub cut: bool,
}
/// A role's whole authority, allotment and full raw request/decision bits.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectPolicyRole {
    pub number: u32,
    pub authority: AuthorityDocument,
    pub period_spend: u64,
    pub requests: u16,
    pub decides: u8,
}
/// The same authentic membership facts for people and services.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectMember {
    pub party: u64,
    pub kind: PartyKind,
    pub role: HeldRole,
    pub login: DocumentText,
    pub name: DocumentText,
}
/// One expense row in the order its accounting owner supplied.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectSpend {
    pub chip: Chip,
    pub funder: Funder,
    pub numbers: Numbers,
}
