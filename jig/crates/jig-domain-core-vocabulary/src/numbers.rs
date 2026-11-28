//! Deployment numbers owned by the core (domain/engine/README.md, 3, and domain/engine/decisions.md, 1.4; domain/model.md, 6.1).

use crate::{
    CoreRecord, Record, Write,
    waits::{WaitFloors, WaitKind},
};
use skein_lib::Wall;

/// Core to store, then store to core at startup: one fixed-size deployment
/// header with allocated high-water marks. Every writing decision saves it
/// atomically with its other rows (domain/engine/decisions.md, 1.1 and 1.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Deployment {
    /// Shell-supplied identity committed on the deployment's first start and restored thereafter;
    /// exactly sixteen bytes.
    pub id: [u8; 16],
    /// Last task number allocated by the core; zero before its first allocation, never reset or
    /// reused.
    pub tasks: u64,
    /// Last candidate person number allocated for sign-in; refused or existing identities may leave
    /// gaps.
    pub people: u64,
    /// Last sign-in candidate allocated by the core; persisted gaps are allowed and no secret
    /// session bytes are held here.
    pub sign_ins: u64,
    /// Last core-issued result position in commit order.
    pub messages: u64,
    /// Last activation number allocated for a claim; the candidate may leave a gap if preparation
    /// fails.
    pub runs: u64,
    /// Last core-allocated call number.
    pub calls: u64,
    /// Last allocated stable store identity for a connector row.
    pub connector_rows: u64,
    /// Last durable token allocated for a connector-owned party outcome.
    pub applications: u64,
    /// Last issued ordered commit. A restored header is already durable; a live journal may await
    /// this number's store terminal.
    pub commits: u64,
    /// Largest seeded or allocated project number.
    pub projects: u64,
    /// The longest restart wait still kept (domain/engine/restart.md, 5).
    pub restart_wait_ms: Option<u64>,
    /// Each recorded wait kind's first observation.
    pub wait_floors: Option<WaitFloors>,
}

impl Deployment {
    #[must_use]
    pub fn wait_floor(&self, kind: WaitKind) -> Option<Wall> {
        self.wait_floors.as_ref()?.floor(kind)
    }

    /// Keep the first wall observation for a kind, returning whether it changed the header.
    pub fn set_wait_floor(&mut self, kind: WaitKind, at: Wall) -> bool {
        self.wait_floors.get_or_insert(WaitFloors::EMPTY).set_once(kind, at)
    }
}

/// Core-selected counter for `fresh`; each is a separate checked `u64` high-water
/// mark, not a channel token or child-owned handle (domain/engine/decisions.md, 1.4).
/// Selecting a family is pure; allocating it dirties the journal header.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Family {
    /// Project identities, past all seeded and made projects.
    Project,
    /// Task identities saved with the task row.
    Task,
    /// Person candidates supplied to people sign-in admission.
    Person,
    /// Secret-free sign-in candidates supplied to people.
    SignIn,
    /// Result positions issued as tasks end.
    Message,
    /// Fresh attempt identities saved at claims.
    Run,
    /// Count of distinct named calls newly decided by the engine.
    Call,
    /// Stable connector row identity, allocated once per live connector key.
    ConnectorRow,
    /// Connector-owned application request outcomes.
    Application,
}

/// Deployment numbers retained by the core.
#[derive(Hash, Debug)]
#[expect(clippy::partial_pub_fields, reason = "the journal sees the header, but only the allocator edits candidates")]
pub struct Counters {
    pub deployment: Deployment,
    pub header: Header,
    // The ordered connector gathers may still decline to make a row.
    connector_candidates: u64,
}

#[derive(Hash, Debug)]
pub enum Header {
    Clean,
    Dirty,
    Loading,
}

impl Counters {
    /// An empty store needs the deployment identity committed even before its first task. The id is
    /// a root input, drawn once by the shell at startup. Validated limits size the fixed held
    /// queue; the next accepted decision emits the dirty header's commit.
    #[must_use]
    pub fn bootstrap(id: [u8; 16]) -> Counters {
        let deployment = Deployment {
            id,
            tasks: 0,
            people: 0,
            sign_ins: 0,
            messages: 0,
            runs: 0,
            calls: 0,
            connector_rows: 0,
            applications: 0,
            commits: 0,
            projects: 0,
            restart_wait_ms: None,
            wait_floors: None,
        };
        Counters { deployment, header: Header::Dirty, connector_candidates: 0 }
    }

    /// A loaded header names the last fully applied commit. It is durable; outstanding answers from
    /// a previous process need not be reconstructed. The root supplies the decoded store header and
    /// validated startup bounds; construction emits no effect.
    #[must_use]
    pub const fn new(deployment: Deployment) -> Counters {
        Counters { deployment, header: Header::Clean, connector_candidates: 0 }
    }

    /// Pure snapshot of allocated counters, which may run ahead of the durable store until its
    /// issued commits answer.
    #[must_use]
    pub const fn deployment(&self) -> Deployment {
        self.deployment
    }

    /// Whether the next decision must save the deployment numbers.
    #[must_use]
    pub const fn dirty(&self) -> bool {
        match self.header {
            Header::Dirty => true,
            Header::Clean | Header::Loading => false,
        }
    }

    /// The bootstrap header cannot be written until the core's live load has
    /// established whether a durable header already exists (domain/engine/restart.md).
    pub fn begin_start(&mut self) {
        self.header = Header::Loading;
    }

    #[must_use]
    pub fn loading(&self) -> bool {
        match self.header {
            Header::Loading => true,
            Header::Clean | Header::Dirty => false,
        }
    }

    /// A supplied durable header replaces these counters with `new`. Only an
    /// empty load still has its bootstrap identity waiting to be committed.
    pub fn finish_restore(&mut self) {
        match self.header {
            Header::Loading => self.header = Header::Dirty,
            Header::Clean | Header::Dirty => {}
        }
    }

    /// The next connector number, including the candidates holding places ahead of it.
    #[must_use]
    pub fn next_connector_candidate(&self) -> Option<u64> {
        if self.loading() {
            return None;
        }
        self.deployment.connector_rows.max(self.connector_candidates).checked_add(1)
    }

    /// Reserve a candidate in connector order without writing the deployment header.
    pub fn reserve_connector_candidate(&mut self) -> Option<u64> {
        let next = self.next_connector_candidate()?;
        self.connector_candidates = next;
        Some(next)
    }

    /// Take a candidate in the decision that admits it as a connector effect.
    pub fn keep_connector_candidate(&mut self, number: u64) {
        if number <= self.connector_candidates && number > self.deployment.connector_rows {
            self.deployment.connector_rows = number;
            self.header = Header::Dirty;
        }
    }

    /// The reserved ceiling, when there are candidates above the allocated numbers.
    #[must_use]
    pub fn connector_candidate_ceiling(&self) -> Option<u64> {
        if self.connector_candidates > self.deployment.connector_rows { Some(self.connector_candidates) } else { None }
    }

    /// At the end of routing, keep only the ceiling of the candidates still in flight.
    pub fn retain_connector_candidates(&mut self, ceiling: u64) {
        assert!(ceiling <= self.connector_candidates, "retaining connector candidates already reserved");
        self.connector_candidates = ceiling;
    }

    /// Number the decision's one commit and return its header for the store.
    pub fn next_commit(&mut self) -> Deployment {
        match self.header {
            Header::Loading => unreachable!("no writing decision before the deployment header is known"),
            Header::Clean | Header::Dirty => {}
        }
        self.deployment.commits = self.deployment.commits.checked_add(1).expect("admitted commit number");
        self.header = Header::Clean;
        self.deployment
    }

    /// Pure idle query given the journal's own idle result.
    #[must_use]
    pub const fn quiescent(&self, journal_idle: bool) -> bool {
        !self.dirty() && journal_idle
    }
}

/// Root numbers are never reused. Allocation is part of the admitted decision, even if its
/// candidate is unused; the next commit saves the gap. The root chooses the counter family after
/// admission. Returns its next positive `u64`, or `None` while loading or exhausted, with no effect on
/// refusal; emits no request itself.
pub fn fresh(counters: &mut Counters, family: Family) -> Option<u64> {
    match counters.header {
        Header::Loading => return None,
        Header::Clean | Header::Dirty => {}
    }
    let counter = match family {
        Family::Project => &mut counters.deployment.projects,
        Family::Task => &mut counters.deployment.tasks,
        Family::Person => &mut counters.deployment.people,
        Family::SignIn => &mut counters.deployment.sign_ins,
        Family::Message => &mut counters.deployment.messages,
        Family::Run => &mut counters.deployment.runs,
        Family::Call => &mut counters.deployment.calls,
        Family::ConnectorRow => &mut counters.deployment.connector_rows,
        Family::Application => &mut counters.deployment.applications,
    };
    let mark = if family == Family::ConnectorRow { (*counter).max(counters.connector_candidates) } else { *counter };
    let next = mark.checked_add(1)?;
    *counter = next;
    counters.header = Header::Dirty;
    Some(next)
}

impl Counters {
    /// Number a writing decision and construct its deployment row (domain/engine/decisions.md, 1.4).
    #[must_use]
    pub fn commit_header(&mut self) -> Write {
        Write::Save(self.commit_record())
    }

    pub fn commit_record(&mut self) -> Record {
        Record::Core(CoreRecord::Deployment(self.next_commit()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_restored_projects_mark_allocates_the_next_number() {
        let mut header = Counters::bootstrap([1; 16]).deployment();
        header.projects = 5;
        let mut counters = Counters::new(header);
        assert_eq!(fresh(&mut counters, Family::Project), Some(6));
        assert_eq!(counters.next_commit().projects, 6);
    }

    #[test]
    fn connector_candidates_keep_their_order_without_allocating_on_a_wait() {
        let mut counters = Counters::new(Counters::bootstrap([1; 16]).deployment());
        assert_eq!(counters.reserve_connector_candidate(), Some(1));
        assert_eq!(counters.reserve_connector_candidate(), Some(2));
        assert_eq!(counters.deployment().connector_rows, 0);
        assert!(!counters.dirty());

        counters.retain_connector_candidates(1);
        assert_eq!(fresh(&mut counters, Family::ConnectorRow), Some(2));
        counters.keep_connector_candidate(1);
        assert_eq!(counters.next_commit().connector_rows, 2);

        assert_eq!(counters.reserve_connector_candidate(), Some(3));
        counters.keep_connector_candidate(3);
        assert!(counters.dirty());
        assert_eq!(counters.next_commit().connector_rows, 3);
    }

    #[test]
    fn an_exhausted_projects_mark_refuses_without_changing_the_header() {
        let mut header = Counters::bootstrap([1; 16]).deployment();
        header.projects = u64::MAX;
        let mut counters = Counters::new(header);
        assert_eq!(fresh(&mut counters, Family::Project), None);
        assert_eq!(counters.deployment(), header);
        assert!(!counters.dirty());
    }
}
