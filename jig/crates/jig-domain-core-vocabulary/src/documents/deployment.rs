//! Deployment facts, store fill and census (client/domain/pages.md, 7; protocol/store/README.md, 6.2–6.3).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use super::{Chip, TaskLink};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, List, Wall};

/// Census bounds (client/domain/contract/documents.md, 1).
pub const CENSUS_ROWS: usize = 129;
pub const CENSUS_PREFIX: usize = 8;
pub const CENSUS_NAME: usize = 48;
pub const CENSUS_HEADS: usize = 8;

/// Last store report (protocol/store/README.md, 6.3; client/domain/contract/documents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StoreCensus {
    pub commit: u64,
    pub since: u64,
    pub keyspaces: List<CensusKeyspace>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CensusKeyspace {
    pub prefix: Box<[u8]>,
    pub name: Box<str>,
    pub rows: u64,
    pub keys: u64,
    pub values: u64,
    pub heads: List<CensusHead>,
    pub puts: u64,
    pub erases: u64,
    pub written: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CensusHead {
    pub head: Option<u16>,
    pub rows: u64,
}

/// Store map bytes at the last commit answer and its admission mark (protocol/store/README.md, 6.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct StoreFill {
    pub bytes: u64,
    pub mark: u64,
}

/// Lost identity is present when this engine observed the channel before loss.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RunLocation {
    None,
    Connected { channel: u64, kind: fleet::HostKind },
    Adrift { channel: Option<u64>, until: Option<Wall>, remaining: Option<Duration> },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HostRun {
    pub task: TaskLink,
    pub attempt: u64,
    pub kept: u32,
    pub listed: bool,
    pub state: fleet::AttemptState,
    pub location: RunLocation,
    pub grace_until: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent host, inventory and document facts")]
pub struct Host {
    pub name: Option<Box<[u8]>>,
    pub awaiting_logins: bool,
    pub rule: crate::HostRule,
    pub programs: List<super::ProgramLogin>,
    pub channel: u64,
    pub kind: fleet::HostKind,
    pub slots: u32,
    pub used: u32,
    pub free: u32,
    pub draining: bool,
    pub connected: bool,
    pub grace_until: Option<Wall>,
    pub grace_remaining: Option<Duration>,
    pub runs: List<HostRun>,
    pub cut: bool,
}
/// Account state at document time; the child's retry clock crosses as a wall.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AccountState {
    Moving,
    Asking,
    Granted,
    Later { until: Option<Wall> },
    SignInAgain,
    Misconfigured,
    Unloaded { at: accounts::Place, why: accounts::Unread },
    Cancelled,
    Closing,
}
/// Secret-free account status, with runs using the configured account and tasks
/// currently waiting for it; no secret or protocol refresh token crosses here.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent child-owned flags plus document truncation")]
pub struct Account {
    pub list: Option<super::AccountList>,
    pub number: u32,
    pub place: u32,
    pub uses: Option<List<accounts::Use>>,
    pub plan: bool,
    pub provider: Option<u32>,
    pub provider_name: Option<super::Text>,
    pub manage_usage: Option<Box<[u8]>>,
    pub display: super::Text,
    pub holder: accounts::Holder,
    pub sources: List<super::SourceLine>,
    pub facts: List<accounts::AccountFact>,
    pub generation: Option<u64>,
    pub state: AccountState,
    pub usable: bool,
    pub expiring: bool,
    pub attention: bool,
    pub spent_until: Option<Wall>,
    pub runs: List<Chip>,
    pub waiting: List<Chip>,
    pub cut: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Capacity {
    pub used: u32,
    pub limit: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectCapacity {
    pub project: u32,
    pub tasks: Capacity,
    pub proposals: u32,
}
/// Connected hosts and all tracked attempts (including adrift), account status,
/// and authentic finite capacities. Refusal history and connector panels are
/// owned by their facts/store and application subjects, not a new cache here.
/// Deployment facts and configured connector references; connector values remain
/// their owners' documents (`client/domain/pages.md`, 7; `client/domain/root.md`, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent aggregate and list loss facts")]
pub struct Deployment {
    pub store: StoreFill,
    pub census: Option<Box<StoreCensus>>,
    pub hosts: List<Host>,
    pub attempts: List<HostRun>,
    pub accounts: List<Account>,
    pub tasks: Capacity,
    pub workers: Capacity,
    pub attempts_capacity: Capacity,
    pub waiting_attempts: u32,
    pub proposals: Capacity,
    pub projects: List<ProjectCapacity>,
    pub grace_bound: Duration,
    pub hello_refusals: List<HelloRefusalLine>,
    pub hosts_forgotten: u64,
    pub hello_refusals_forgotten: u64,
    /// Person-keyed room refusals; other task callers have their own owner facts.
    pub person_room_refusals: List<people::CapacityRefusal>,
    pub person_room_refusals_forgotten: u64,
    /// Permanent refusal answers follow this retention; unsaved entrance history is live-only.
    pub person_answer_retention: Duration,
    pub task_room_refusals: List<tasks::CapacityRefusal>,
    pub task_room_refusals_forgotten: u64,
    pub connectors: List<u16>,
    pub connectors_cut: bool,
    pub cut: bool,
    /// Public configured provider offers within the document line bound.
    pub providers: List<super::ProviderOffer>,
    /// Whether additional provider offers were omitted.
    pub providers_cut: bool,
    /// Actual startup key ceiling, including the client carried-value bound.
    pub key_bytes: u32,
    /// Current observation from the engine selection and host checks.
    pub readiness: List<super::PayerReadiness>,
    /// Whether additional provider/payer rows were omitted.
    pub readiness_cut: bool,
}
/// Recent refused hello facts, without protocol rights or advertised workstream payloads.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HelloRefusalLine {
    pub channel: u64,
    pub at: Wall,
    pub slots: u32,
    pub stop_bound: Duration,
    pub why: fleet::HelloRefusal,
}

/// Heap of the boxed report, its rows and every row's independent prefix, name and heads.
pub fn census_bound() -> Option<u64> {
    let row = u64::try_from(CENSUS_PREFIX.checked_add(CENSUS_NAME)?)
        .ok()?
        .checked_add(List::<CensusHead>::price(u32::try_from(CENSUS_HEADS).ok()?)?)?;
    u64::try_from(size_of::<StoreCensus>())
        .ok()?
        .checked_add(List::<CensusKeyspace>::price(u32::try_from(CENSUS_ROWS).ok()?)?)?
        .checked_add(u64::try_from(CENSUS_ROWS).ok()?.checked_mul(row)?)
}

pub fn document_bound(limits: &super::DocumentLimits) -> Option<u64> {
    let carriers = u64::from(limits.lines)
        .checked_mul(u64::from(limits.lines))?
        .checked_add(u64::from(limits.rows).checked_mul(2)?)?
        .checked_add(u64::from(limits.lines).checked_mul(3)?)?
        .checked_add(4)?;
    // One nested host-run/task/account carrier is smaller than a task page,
    // whose bound prices all its chip/part/list children. Hosts nest run lists.
    let account_heap = List::<super::SourceLine>::price(limits.lines)?
        .checked_add(u64::from(limits.lines).checked_mul(u64::from(accounts::NAME))?)?
        .checked_add(u64::from(limits.text_bytes).checked_mul(2)?)?
        .checked_add(u64::from(accounts::MANAGE_USAGE))?
        .checked_add(List::<accounts::Use>::price(accounts::USES)?)?
        .checked_add(super::accounts::facts_bound()?)?;
    super::task_tree::document_bound(limits)?
        .checked_mul(carriers)?
        .checked_add(
            u64::from(limits.lines)
                .checked_mul(u64::from(fleet::WORKER_NAME).checked_add(super::programs::logins_bound(limits)?)?)?,
        )?
        // HostRule's tag and u64 payload occupy 16 aligned bytes per host.
        .checked_add(u64::from(limits.lines).checked_mul(u64::try_from(size_of::<crate::HostRule>()).ok()?)?)?
        .checked_add(List::<Account>::price(limits.lines)?)?
        .checked_add(u64::from(limits.lines).checked_mul(account_heap)?)?
        .checked_add(List::<HelloRefusalLine>::price(limits.lines)?)?
        .checked_add(List::<people::CapacityRefusal>::price(limits.lines)?)?
        .checked_add(List::<tasks::CapacityRefusal>::price(limits.lines)?)?
        .checked_add(List::<u16>::price(limits.lines)?)?
        .checked_add(u64::try_from(size_of::<Deployment>()).ok()?)?
        .checked_add(super::catalogue::offers_bound(limits)?)?
        .checked_add(super::catalogue::rows_bound(limits)?)?
        .checked_add(census_bound()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployment_bound_includes_a_census_at_every_independent_bound() {
        let mut keyspaces = List::with_capacity(129);
        for _ in 0_u32..129 {
            let mut heads = List::with_capacity(8);
            for head in 0..8 {
                heads.push(CensusHead { head: Some(head), rows: u64::MAX }).unwrap();
            }
            keyspaces.push(CensusKeyspace {
                prefix: Box::new([255; 8]),
                name: "n".repeat(48).into_boxed_str(),
                rows: u64::MAX,
                keys: u64::MAX,
                values: u64::MAX,
                heads,
                puts: u64::MAX,
                erases: u64::MAX,
                written: u64::MAX,
            }).unwrap();
        }
        let census = Box::new(StoreCensus { commit: u64::MAX, since: 1, keyspaces });
        let mut heap = u64::try_from(size_of_val(census.as_ref())).unwrap()
            + List::<CensusKeyspace>::price(census.keyspaces.capacity()).unwrap();
        for row in &census.keyspaces {
            heap += u64::try_from(row.prefix.len() + row.name.len()).unwrap()
                + List::<CensusHead>::price(row.heads.capacity()).unwrap();
        }
        assert_eq!(census_bound(), Some(heap));
        let limits = super::super::DocumentLimits {
            roles: 0, segments: 0, grants: 0, executors: 0, implications: 0,
            rows: 0, text_bytes: 0, chips: 0, lines: 0,
            versions: super::super::Versions { minimum: 1, maximum: 1 },
        };
        assert!(document_bound(&limits).unwrap() >= heap, "a census fits even with no configured document rows");
    }
}
