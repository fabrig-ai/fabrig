//! Top-owned landing rows, coherent reads, receipts and effects (domain/landing.md, 2–15).
use crate::domain::{child, emit, keep_entry};
use crate::{
    Check, Domain, Key, LandingEntry, LandingHeard, LandingRow, Limits, Operation, Receipt, ReceiptKey, ReceiptRun,
    RepositoryGates, Request, Stored, runs,
};
use alloc::boxed::Box;
use core::mem::size_of;
use skein_forge::api;
use skein_lib::{Env, List, Map, Queue, Token, Wall};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as policy;

type Position = (u64, u16, u16);
type RepositoryGate = (api::Repository, Box<[u8]>);
type KeyAt = (api::Commit, policy::Key);
type ProjectLanding =
    (change::Way, Box<[RepositoryGates]>, crate::AfterLanding, Option<Box<[u8]>>, Option<Box<[u8]>>, policy::Project);
#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) rows: Map<u64, LandingRow>,
    pub(crate) due: Map<u64, Wall>,
    pub(crate) receipts: Map<ReceiptKey, Receipt>,
    pub(crate) projects: Map<u32, ProjectLanding>,
    pub(crate) pending: Map<Token, Pending>,
    pub(crate) keys: Map<Position, (api::Commit, Option<api::Commit>, policy::Key)>,
    pub(crate) main_keys: Map<RepositoryGate, KeyAt>,
    pub(crate) flights: Map<ReceiptKey, runs::Follow>,
    pub(crate) faults: Map<Position, runs::Follow>,
    pub(crate) results: Map<(u64, u16, u16, u8), policy::Status>,
    pub(crate) limits: Limits,
}
#[derive(Hash, Debug)]
pub(crate) struct Pending {
    task: u64,
    repository: api::Repository,
    resource: Option<GatherRead>,
    cached: bool,
    pub(crate) entry: u64,
    heard: LandingHeard,
    facts: Box<[policy::MemberFacts]>,
    stage: Stage,
    polled: List<ReceiptKey>,
    trees: List<policy::Tree>,
    base_trees: List<policy::Tree>,
    entries: List<policy::Entry>,
    woken: bool,
    evidence: Box<[crate::LandingEvidence]>,
    opened: Option<u64>,
    upstream: Option<api::Commit>,
    paths: Box<[Paths]>,
    reads: Map<(api::Repository, GatherRead), Result<api::Answer, api::Error>>,
}
#[derive(Hash, Debug)]
struct Paths {
    files: Box<[Box<[u8]>]>,
    whole: bool,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum GatherRead {
    Branch(Box<[u8]>),
    Pull(u64),
}
fn gather_resource(read: &api::Read) -> Option<GatherRead> {
    if let api::Read::Branch { branch } = read {
        Some(GatherRead::Branch(branch.clone()))
    } else if let api::Read::Pull { number, .. } = read {
        Some(GatherRead::Pull(*number))
    } else {
        None
    }
}
#[derive(Clone, Hash, Debug)]
enum Stage {
    Branch(u16),
    Main(u16),
    Ancestry(u16),
    Pull(u16),
    Reviews(u16, u32),
    Paths(u16),
    Collect,
    Round {
        stage: RoundRead,
        candidate: Option<api::Commit>,
    },
    Run {
        key: ReceiptKey,
        pages: u32,
    },
    FailedJobs {
        key: ReceiptKey,
        run: api::Run,
        next: u32,
        words: Box<[u8]>,
    },
    Tree {
        member: u16,
        gate: u16,
        candidate: api::Commit,
        object: api::Commit,
        page: u32,
        main: bool,
        reading_base: bool,
    },
}
#[derive(Clone, Copy, Hash, Debug)]
enum RoundRead {
    Branch,
    Base,
    Head,
    Squash,
    Contained,
    Pull,
    Upstream,
}

fn slots(l: &Limits) -> Option<u32> {
    l.landings.checked_mul(l.landing_members)?.checked_mul(l.gates)
}
fn result_slots(l: &Limits) -> Option<u32> {
    slots(l)?.checked_mul(4)?.checked_add(l.landings.checked_mul(l.landing_members)?.checked_mul(5)?)
}
fn receipt_slots(l: &Limits) -> Option<u32> {
    l.repositories
        .checked_mul(l.gates.checked_add(l.tiers.tiers)?)?
        .checked_add(l.judge_projects.checked_mul(l.follows)?)?
        .checked_mul(l.receipts_per_gate)
}
// Landing row indices and retained table counts must fit before construction.
pub(crate) fn valid(l: &Limits) -> bool {
    if l.landing_members > u32::from(u16::MAX)
        || l.gates > u32::from(u16::MAX)
        || l.receipts_per_gate < 2
        || l.trees_in_flight == 0
        || l.key_entries == 0
        || l.name_bytes > 4096
        || l.key_path_bytes == 0
    {
        return false;
    }
    slots(l).is_some() && result_slots(l).is_some() && receipt_slots(l).is_some()
}
impl State {
    pub fn new(l: &Limits) -> Self {
        let slots = slots(l).expect("landing bounds checked");
        Self {
            rows: Map::with_capacity(l.landings),
            due: Map::with_capacity(l.landings),
            receipts: Map::with_capacity(receipt_slots(l).expect("receipt bounds")),
            projects: Map::with_capacity(l.judge_projects),
            pending: Map::with_capacity(l.landings),
            keys: Map::with_capacity(slots),
            main_keys: Map::with_capacity(l.repositories.checked_mul(l.gates).expect("gate bounds")),
            flights: Map::with_capacity(slots),
            faults: Map::with_capacity(slots),
            results: Map::with_capacity(result_slots(l).expect("effect bounds")),
            limits: *l,
        }
    }
    pub fn way(&self, project: u32) -> change::Way {
        match self.projects.get(&project) {
            Some((way, _, _, _, _, _)) => *way,
            None => change::Way::Queue,
        }
    }
    pub(crate) fn gates(&self, project: u32, repository: api::Repository) -> &[policy::Gate] {
        if let Some((_, repositories, _, _, _, _)) = self.projects.get(&project) {
            for row in repositories {
                if row.repository == repository {
                    return &row.gates;
                }
            }
        }
        &[]
    }
}
fn valid_tier(tier: &crate::Tier, limits: &Limits) -> bool {
    if tier.name.is_empty()
        || tier.name.len() > policy::CHECK_NAME_BYTES
        || tier.description.len() > crate::DESCRIPTION_BYTES
        || tier.workflow.is_empty()
        || tier.workflow.len() > policy::WORKFLOW_BYTES
        || tier.workflow.contains(&b'/')
        || tier.artifact.is_empty()
        || tier.artifact.len() > policy::CHECK_NAME_BYTES
        || tier.artifact.contains(&b'/')
        || tier.ignored.len() > usize::try_from(limits.ignored_paths).expect("u32 fits")
    {
        return false;
    }
    match &tier.follows {
        crate::Follows::Branch(branch) if branch.is_empty() => return false,
        crate::Follows::Branch(_) | crate::Follows::Landing | crate::Follows::Goals => {}
    }
    for byte in &tier.name {
        if !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-' {
            return false;
        }
    }
    for path in &tier.ignored {
        if path.is_empty()
            || path.len() > usize::try_from(limits.name_bytes).expect("u32 fits")
            || path.first() == Some(&b'/')
            || path.last() == Some(&b'/')
        {
            return false;
        }
    }
    true
}
impl Domain {
    /// A project's way and repository test gates (domain/landing.md, 13.1).
    pub fn project_landing(
        &mut self,
        project: u32,
        way: change::Way,
        gates: Box<[RepositoryGates]>,
        gates_policy: policy::Project,
        after: crate::AfterLanding,
    ) -> bool {
        let l = &self.landing.limits;
        if gates.len() > usize::try_from(l.repositories).expect("u32 fits") {
            return false;
        }
        for (index, repository) in gates.iter().enumerate() {
            if way == change::Way::Turn
                && let Some(forge) = self.configured.get(&repository.repository.forge)
                && forge.kind == crate::Kind::GitHub
            {
                return false;
            }
            for prior in gates.get(..index).expect("preceding positions") {
                if prior.repository == repository.repository {
                    return false;
                }
            }
            if repository.gates.len() > usize::try_from(l.gates).expect("u32 fits") {
                return false;
            }
            if repository.tiers.len() > usize::try_from(l.tiers.tiers).expect("u32 fits") {
                return false;
            }
            for (index, tier) in repository.tiers.iter().enumerate() {
                if !valid_tier(tier, l) {
                    return false;
                }
                for gate in &repository.gates {
                    if gate.name == tier.name {
                        return false;
                    }
                }
                for prior in repository.tiers.get(..index).expect("preceding tiers") {
                    if prior.name == tier.name {
                        return false;
                    }
                }
            }
            for (index, gate) in repository.gates.iter().enumerate() {
                if gate.name.is_empty()
                    || gate.name.len() > policy::CHECK_NAME_BYTES
                    || gate.workflow.is_empty()
                    || gate.workflow.len() > policy::WORKFLOW_BYTES
                    || gate.ignored.len() > usize::try_from(l.ignored_paths).expect("u32 fits")
                {
                    return false;
                }
                for byte in &gate.name {
                    if !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-' {
                        return false;
                    }
                }
                for prior in repository.gates.get(..index).expect("preceding positions") {
                    if prior.name == gate.name {
                        return false;
                    }
                }
                for path in &gate.ignored {
                    if path.is_empty()
                        || path.len() > usize::try_from(l.name_bytes).expect("u32 fits")
                        || path.first() == Some(&b'/')
                        || path.last() == Some(&b'/')
                    {
                        return false;
                    }
                }
            }
        }
        let mut has_tiers = false;
        if let Some((_, old, _, _, _, _)) = self.landing.projects.get(&project) {
            for repository in old {
                has_tiers |= !repository.tiers.is_empty();
            }
        }
        for repository in &gates {
            has_tiers |= !repository.tiers.is_empty();
        }
        let saved = self.landing.projects.insert(project, (way, gates, after, None, None, gates_policy)).is_ok();
        if saved && has_tiers {
            self.tier_ops.dirty = true;
        }
        saved
    }
    /// A configured project's merge command (temper/docs/design/protocol/configuration/engine.md).
    pub fn project_merge_job(&mut self, project: u32, merge_job: Box<[u8]>) -> bool {
        let Some((_, _, _, command, _, _)) = self.landing.projects.get_mut(&project) else { return false };
        *command = Some(merge_job);
        true
    }
    /// A configured project's close command (temper/docs/design/domain/forge/change/README.md, 1).
    pub fn project_close_job(&mut self, project: u32, close_job: Box<[u8]>) -> bool {
        let Some((_, _, _, _, command, _)) = self.landing.projects.get_mut(&project) else { return false };
        *command = Some(close_job);
        true
    }
    /// The project's optional close command (temper/docs/design/domain/forge/change/README.md, 1).
    #[must_use]
    pub fn close_job(&self, project: u32) -> Option<&[u8]> {
        let (_, _, _, _, command, _) = self.landing.projects.get(&project)?;
        command.as_deref()
    }
    /// A project's landing way: queue where its configuration gives none (domain/landing.md, 13.1).
    #[must_use]
    pub fn landing_way(&self, project: u32) -> change::Way {
        self.landing.way(project)
    }
    /// A repository's configured gates and tiers in its project, if it has any.
    #[must_use]
    #[expect(clippy::manual_find, reason = "the step subset searches without iterator closures")]
    pub fn repository_gates(&self, project: u32, repository: api::Repository) -> Option<&RepositoryGates> {
        let (_, repositories, _, _, _, _) = self.landing.projects.get(&project)?;
        for row in repositories {
            if row.repository == repository {
                return Some(row);
            }
        }
        None
    }
    /// The project's optional merge command (temper/docs/design/protocol/configuration/engine.md).
    #[must_use]
    pub fn merge_job(&self, project: u32) -> Option<&[u8]> {
        let (_, _, _, command, _, _) = self.landing.projects.get(&project)?;
        command.as_deref()
    }
    /// The project's retained verification settings (domain/after-landing.md, 11).
    #[must_use]
    pub fn after_landing(&self, project: u32) -> Option<crate::AfterLanding> {
        match self.landing.projects.get(&project) {
            Some((_, _, after, _, _, _)) => Some(*after),
            None => None,
        }
    }
    /// The connected changes of a pending cross-repository dependency, upstream
    /// first. The root supplies jig's current graph on each decision; no graph is
    /// stored here (domain/landing.md, 4.1–4.2).
    #[must_use]
    pub fn joined_changes(&self, project: u32, task: u64, graph: &[crate::ChangeDependencies]) -> Box<[u64]> {
        let mut joined = List::with_capacity(self.landing.limits.changes);
        joined.push(task).expect("one change admitted");
        if self.landing.way(project) != change::Way::Turn {
            return joined.into_boxed();
        }
        // Every pass adds at least one vertex or has reached the fixed point.
        for _ in 0..self.landing.limits.changes {
            let before = joined.len();
            for edge in graph {
                if !joinable(self, project, edge.change) {
                    continue;
                }
                for upstream in &edge.pending {
                    if !joinable(self, project, *upstream) {
                        continue;
                    }
                    let down = self.changes.get(&edge.change).expect("joinable change");
                    let up = self.changes.get(upstream).expect("joinable upstream");
                    if down.repository == up.repository {
                        continue;
                    }
                    if joined.as_slice().contains(&edge.change) && !joined.as_slice().contains(upstream) {
                        joined.push(*upstream).expect("bounded changes");
                    } else if joined.as_slice().contains(upstream) && !joined.as_slice().contains(&edge.change) {
                        joined.push(edge.change).expect("bounded changes");
                    }
                }
            }
            if joined.len() == before {
                break;
            }
        }
        ordered_changes(self, joined.as_slice(), graph)
    }
    /// Check admission before the root allocates its deployment-funded task.
    pub fn landing_admits(&self, project: u32, changes: &[u64]) -> Result<(), crate::LandingRefusal> {
        use crate::LandingRefusal;
        if self.landing.way(project) != change::Way::Turn {
            return Err(LandingRefusal::Way);
        }
        if changes.len() != 1 || self.landing.limits.landing_members == 0 {
            return Err(LandingRefusal::Members);
        }
        for task in changes {
            let row = self.changes.get(task).ok_or(LandingRefusal::Repository)?;
            let repo = self.repositories.get(&row.repository).ok_or(LandingRefusal::Repository)?;
            if repo.project != project || row.change.landing.is_some() {
                return Err(LandingRefusal::Repository);
            }
            if !self.train_free(row.repository, &row.base) {
                return Err(LandingRefusal::Train);
            }
        }
        if self.landing.rows.len() == self.landing.limits.landings {
            for (_, row) in &self.landing.rows {
                if row.entries.is_empty() && matches_terminal(&row.landing.phase) {
                    return Ok(());
                }
            }
            return Err(LandingRefusal::Landings);
        }
        Ok(())
    }
    /// The hold account for a unit beyond this milestone (domain/landing.md, 4.1).
    #[must_use]
    pub fn unit_words(&self, joined: &[u64]) -> Box<[u8]> {
        let mut words = List::with_capacity(jig_domain_tasks::HOLD_ACCOUNT_BYTES);
        append_words(&mut words, b"unit of ");
        append_words(&mut words, skein_lib::Decimal::of(u64::try_from(joined.len()).expect("change count")).as_bytes());
        append_words(&mut words, b" changes: ");
        let mut repository = None;
        let mut across = false;
        for (index, task) in joined.iter().enumerate() {
            if index != 0 {
                append_words(&mut words, b", ");
            }
            append_words(&mut words, b"C");
            append_words(&mut words, skein_lib::Decimal::of(*task).as_bytes());
            if let Some(row) = self.change(*task) {
                if let Some(prior) = repository {
                    across |= prior != row.repository;
                }
                repository = Some(row.repository);
            }
        }
        append_words(
            &mut words,
            if across {
                b": a unit across repositories lands with the merge queue's later batches (temper/docs/design/draft/merge-queue.md, 13)"
            } else {
                b": several changes in one repository land with the merge queue's batches (temper/docs/design/draft/merge-queue.md, 4)"
            },
        );
        words.into_boxed()
    }
    /// Trains and their waiting changes are derived, never reserved (domain/landing.md, 9).
    #[must_use]
    pub fn train_free(&self, repository: api::Repository, branch: &[u8]) -> bool {
        for (_, row) in &self.landing.rows {
            if row.landing.branch.as_ref() != branch || matches_terminal(&row.landing.phase) {
                continue;
            }
            for member in &row.landing.members {
                if self::repository(member) == repository {
                    return false;
                }
            }
        }
        true
    }
    /// Each ready turn change names a train to judge; the root filters kept tasks.
    #[must_use]
    pub fn waiting_trains(&self) -> Box<[(api::Repository, Box<[u8]>)]> {
        let mut trains = List::with_capacity(self.landing.limits.changes);
        for (_, row) in &self.changes {
            if waiting(row) && !trains.as_slice().contains(&(row.repository, row.base.clone())) {
                trains.push((row.repository, row.base.clone())).expect("bounded changes");
            }
        }
        trains.into_boxed()
    }
    /// The first eligible waiting change of a free train with room (domain/landing.md, 9).
    #[must_use]
    pub fn first_waiting(&self, repository: api::Repository, branch: &[u8], kept: &[u64], now: Wall) -> Option<u64> {
        if !self.train_free(repository, branch) {
            return None;
        }
        let mut ready = List::with_capacity(self.landing.limits.changes);
        for candidate in self.waiting_changes(repository, branch) {
            if !kept.contains(&candidate.task) {
                ready.push(candidate).expect("bounded changes");
            }
        }
        let task = change::first(ready.as_slice(), self.landing.limits.queue_window, now)?;
        let project = self.repositories.get(&repository)?.project;
        self.landing_admits(project, &[task]).ok()?;
        Some(task)
    }
    /// Queue candidates for one train, before the root filters task holds and dependencies.
    #[must_use]
    pub fn waiting_changes(&self, repository: api::Repository, branch: &[u8]) -> Box<[change::Ready]> {
        let mut ready = List::with_capacity(self.landing.limits.changes);
        for (task, row) in &self.changes {
            if row.repository == repository
                && row.base.as_ref() == branch
                && waiting(row)
                && let change::State::Ready { ready_since, .. } = row.change.state
            {
                ready
                    .push(change::Ready {
                        task: *task,
                        since: row.change.ready_since.unwrap_or(ready_since),
                        priority: row.priority,
                    })
                    .expect("bounded changes");
            }
        }
        ready.into_boxed()
    }
    /// The top owns each landing's bounded retry deadline (domain/landing.md, 4.2).
    pub fn wait_landing(&mut self, env: &Env<Limits>, task: u64, until: Option<Wall>) {
        let until = match until {
            Some(until) => until,
            None => poll_at(self, env, task),
        };
        self.landing.due.insert(task, until).expect("one deadline per admitted landing");
    }
    pub fn offered_landing(&mut self, task: u64) {
        self.landing.due.remove(&task);
    }
    #[must_use]
    pub fn landing(&self, task: u64) -> Option<&LandingRow> {
        self.landing.rows.get(&task)
    }
    #[must_use]
    pub fn receipt(&self, key: &ReceiptKey) -> Option<&Receipt> {
        self.landing.receipts.get(key)
    }
}
fn append_words(words: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if words.push(*byte).is_err() {
            break;
        }
    }
}
#[expect(clippy::match_like_matches_macro, reason = "the foundation requires explicit matching")]
fn waiting(row: &crate::ChangeRow) -> bool {
    row.change.way == change::Way::Turn
        && row.change.landing.is_none()
        && if let change::State::Ready { .. } = row.change.state { true } else { false }
}
fn ordered_changes(d: &Domain, joined: &[u64], graph: &[crate::ChangeDependencies]) -> Box<[u64]> {
    let mut ordered = List::with_capacity(d.landing.limits.changes);
    for _ in 0..joined.len() {
        for member in joined {
            if ordered.as_slice().contains(member) {
                continue;
            }
            let mut blocked = false;
            for edge in graph {
                if edge.change == *member {
                    for upstream in &edge.pending {
                        if joined.contains(upstream) && !ordered.as_slice().contains(upstream) {
                            blocked = true;
                        }
                    }
                }
            }
            if !blocked {
                ordered.push(*member).expect("bounded changes");
            }
        }
    }
    ordered.into_boxed()
}

fn joinable(d: &Domain, project: u32, task: u64) -> bool {
    let Some(row) = d.changes.get(&task) else { return false };
    let Some(repository) = d.repositories.get(&row.repository) else { return false };
    if repository.project != project {
        return false;
    }
    if row.change.landing.is_some() {
        return false;
    }
    match row.change.state {
        change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. } => false,
        change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::CancellingGates { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Held { .. } => true,
    }
}

/// Formed landings are sealed (domain/landing.md, 4.1).
pub(crate) fn join(_d: &mut Domain, _task: u64, _member: policy::Member, _after: &[u64], _out: &mut Queue<Request>) {}
pub(crate) fn order(_d: &mut Domain, _task: u64, _changes: &[u64], _out: &mut Queue<Request>) {}

fn member_position(row: &LandingRow, prior: &LandingRow, old: u16) -> Option<u16> {
    let change = prior.landing.members.get(usize::from(old))?.change;
    for (index, member) in row.landing.members.iter().enumerate() {
        if member.change == change {
            return Some(u16::try_from(index).expect("member position"));
        }
    }
    None
}

// All positional state follows the change when a member joins or leaves.
fn remap_members(d: &mut Domain, row: &mut LandingRow, prior: &LandingRow) {
    let mut asks = List::with_capacity(d.landing.limits.landing_members);
    for member in &row.landing.members {
        let mut ask = None;
        for (index, old) in prior.landing.members.iter().enumerate() {
            if old.change == member.change {
                ask.clone_from(prior.asks.get(index).expect("member ask"));
            }
        }
        asks.push(ask).expect("member asks");
    }
    row.asks = asks.into_boxed();
    let mut entries = List::with_capacity(d.landing.limits.entries);
    for entry in &prior.entries {
        if let Some(member) = member_position(row, prior, entry.member) {
            entries.push(LandingEntry { member, ..*entry }).expect("bounded entries");
        }
    }
    row.entries = entries.into_boxed();
    // Move only this landing's values out before assigning new positions.
    // The scratch lists live for this step and are bounded by its members.
    let members = u32::try_from(prior.landing.members.len()).expect("member count");
    let gates = d.landing.limits.gates;
    let slots = members.checked_mul(gates).expect("gate slots");
    let mut keys = List::with_capacity(slots);
    let mut faults = List::with_capacity(slots);
    let result_slots = slots.checked_mul(4).expect("gate result slots").checked_add(members).expect("result slots");
    let mut results = List::with_capacity(result_slots);
    for old in 0..members {
        let old = u16::try_from(old).expect("member position");
        for gate in 0..gates {
            let gate = u16::try_from(gate).expect("gate position");
            let key = (row.landing.task, old, gate);
            if let Some(value) = d.landing.keys.remove(&key) {
                keys.push((old, gate, value)).expect("member keys");
            }
            if let Some(value) = d.landing.faults.remove(&key) {
                faults.push((old, gate, value)).expect("member faults");
            }
            for operation in 1..=4 {
                if let Some(value) = d.landing.results.remove(&(row.landing.task, old, gate, operation)) {
                    results.push((old, gate, operation, value)).expect("member results");
                }
            }
        }
        if let Some(value) = d.landing.results.remove(&(row.landing.task, old, 0, 5)) {
            results.push((old, 0, 5, value)).expect("member merge result");
        }
    }
    for (old, gate, value) in keys.into_boxed() {
        if let Some(member) = member_position(row, prior, old) {
            d.landing.keys.insert((row.landing.task, member, gate), value).expect("bounded keys");
        }
    }
    for (old, gate, operation, value) in results.into_boxed() {
        if let Some(member) = member_position(row, prior, old) {
            d.landing.results.insert((row.landing.task, member, gate, operation), value).expect("bounded results");
        }
    }
    for (old, gate, value) in faults.into_boxed() {
        if let Some(member) = member_position(row, prior, old) {
            d.landing.faults.insert((row.landing.task, member, gate), value).expect("bounded faults");
        }
    }
}

fn repository(member: &policy::Member) -> api::Repository {
    api::Repository { forge: member.repository.forge, repository: member.repository.repository }
}
fn gate_key(row: &LandingRow, member: u16, _gate: u16, name: &[u8], key: policy::Key) -> ReceiptKey {
    let m = row.landing.members.get(usize::from(member)).expect("member");
    ReceiptKey { project: row.landing.project, repository: repository(m), check: Check::Gate(Box::from(name)), key }
}
fn proof_key(proof: policy::Proof) -> Option<policy::Key> {
    match proof {
        policy::Proof::Unkeyed => None,
        policy::Proof::Keyed { key }
        | policy::Proof::Dispatched { key, .. }
        | policy::Proof::Passed { key }
        | policy::Proof::Failed { key } => Some(key),
    }
}
fn save(d: &mut Domain, row: LandingRow, out: &mut Queue<Request>) {
    let project = row.landing.project;
    let task = row.landing.task;
    let was_turn = match d.landing.rows.get(&task) {
        Some(prior) => crate::views::matches_turn(&prior.landing.phase),
        None => false,
    };
    let turn_changed = was_turn != crate::views::matches_turn(&row.landing.phase);
    d.landing.rows.insert(task, row.clone()).expect("admitted landing");
    emit(out, Request::Save { record: Stored::Landing(row), release: None, read_afresh: None });
    emit(
        out,
        Request::Marked {
            tasks: Box::new([
                task,
                crate::landings_subject(project, false),
                crate::landings_subject(project, true),
                crate::changes_subject(project),
            ]),
        },
    );
    // Every project's landing document names the turn's holder.
    if turn_changed {
        let mut tasks = List::with_capacity(crate::MARKED);
        for (other, saved) in &d.landing.rows {
            if *other != task && saved.landing.project == project {
                tasks.push(*other).expect("mark room");
                if tasks.room() == 0 {
                    emit(out, Request::Marked { tasks: tasks.into_boxed() });
                    tasks = List::with_capacity(crate::MARKED);
                }
            }
        }
        if !tasks.is_empty() {
            emit(out, Request::Marked { tasks: tasks.into_boxed() });
        }
    }
}
pub(crate) fn register(d: &mut Domain, env: &Env<Limits>, row: LandingRow, out: &mut Queue<Request>) {
    let task = row.landing.task;
    let mut changes = List::with_capacity(env.limits.landing_members);
    for member in &row.landing.members {
        changes.push(member.change).expect("preflighted member count");
        let change = d.changes.get(&member.change).expect("preflighted member");
        assert_eq!(change.repository, repository(member), "preflighted repository");
        assert_eq!(row.landing.branch, change.base, "the member lands into its train's branch");
        assert_eq!(change.pull.unwrap_or(0), member.pull, "preflighted pull");
        assert_eq!(member.gates.len(), d.train_gates(&row, repository(member)).len(), "configured gates");
    }
    d.landing_admits(row.landing.project, changes.as_slice()).expect("root preflighted landing in this decision");
    assert!(!d.landing.rows.contains_key(&task), "fresh landing task");
    assert_eq!(row.asks.len(), row.landing.members.len(), "one ask per member");
    assert!(row.entries.is_empty(), "new landing has no effects");
    if d.landing.rows.len() == env.limits.landings {
        assert!(retire_finished(d, out), "preflight found a reclaimable terminal landing");
    }
    for member in &row.landing.members {
        let change = d.changes.get_mut(&member.change).expect("validated member");
        change.change.landing = Some(task);
        let saved = change.clone();
        super::domain::save_change(d, out, saved);
    }
    save(d, row, out);
    emit(out, Request::LandingNews { task, step: crate::LandingNews::Joined });
}
impl Domain {
    fn train_gates(&self, row: &LandingRow, repo: api::Repository) -> &[policy::Gate] {
        if let Some(repository) = self.repository(repo)
            && repository.settings.default_branch != row.landing.branch
        {
            return &[];
        }
        self.landing.gates(row.landing.project, repo)
    }
}

fn fresh(gates: &[policy::Gate]) -> policy::MemberFacts {
    let mut facts = List::with_capacity(u32::try_from(gates.len()).expect("bounded gates"));
    for gate in gates {
        facts
            .push(policy::GateFacts {
                blocking: gate.blocking,
                key: None,
                receipt: policy::Status::Unknown,
                flight: policy::Flight::None,
                posting: policy::Status::Unknown,
                expired: None,
                run: 0,
            })
            .expect("bounded gates");
    }
    policy::MemberFacts {
        outcome: None,
        dependencies: Box::new([]),
        deleted: policy::Status::Unknown,
        ready: false,
        cancelled: false,
        head: [0; 32],
        main: [0; 32],
        contains_main: policy::Status::Unknown,
        asked: false,
        merge: policy::Status::Unknown,
        merged: None,
        gates: facts.into_boxed(),
        contained: policy::Status::Unknown,
        candidate: None,
        preparation: policy::Preparation::Idle,
        record: policy::Status::Unknown,
        held: false,
        judgement: policy::Status::Passed,
        refused: None,
    }
}
fn pending_for(d: &Domain, task: u64) -> Option<Token> {
    let mut existing = None;
    for (token, pending) in &d.landing.pending {
        if pending.task == task {
            existing = Some(*token);
        }
    }
    existing
}
pub(crate) fn step(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
    heard: LandingHeard,
    out: &mut Queue<Request>,
) {
    if d.stopping {
        return;
    }
    if let Some(row) = d.landing.rows.get(&task)
        && row.landing.phase == policy::Phase::Cancelled
    {
        emit(out, Request::LandingDecision { task, decision: policy::Decision::Cancel, entry: None, evidence: None });
        return;
    }
    if let Some(token) = pending_for(d, task) {
        let pending = d.landing.pending.get_mut(&token).expect("pending step");
        pending.entry = entry;
        pending.heard = heard;
        pending.woken = true;
        return;
    }
    let Some(mut row) = d.landing.rows.get(&task).cloned() else {
        emit(out, Request::Refused { task });
        return;
    };
    for member in &mut row.landing.members {
        let Some(change) = d.changes.get(&member.change) else { return };
        let Some(pull) = change.pull else {
            let repository = change.repository;
            row.landing.phase = policy::Phase::Waiting;
            save(d, row, out);
            wait(d, task, env, repository, out);
            return;
        };
        member.pull = pull;
    }
    if d.landing.rows.get(&task) != Some(&row) {
        save(d, row.clone(), out);
    }
    let mut facts = List::with_capacity(env.limits.landing_members);
    let mut evidence = List::with_capacity(env.limits.landing_members);
    let mut paths = List::with_capacity(env.limits.landing_members);
    for member in &row.landing.members {
        let change = d.changes.get(&member.change).expect("member change");
        evidence
            .push(crate::LandingEvidence {
                change: member.change,
                facts: crate::ChangeEvidence {
                    touched: crate::Touched::UNKNOWN,
                    prior: change.change.clone(),
                    head: None,
                    base_tip: [0; 32],
                    contains_base: change::Status::Unknown,
                    ci: change::Status::Unknown,
                    gates: change.verdicts.clone(),
                    reviews: Box::new([]),
                    reviews_complete: false,
                    size: None,
                },
            })
            .expect("bounded evidence");
        facts.push(fresh(d.train_gates(&row, repository(member)))).expect("admitted members");
        paths.push(Paths { files: Box::new([]), whole: false }).expect("admitted members");
    }
    let Some(first) = row.landing.members.first() else {
        return;
    };
    let Some(change) = d.changes.get(&first.change) else {
        return;
    };
    let repo = change.repository;
    let read = api::Read::Branch { branch: change.branch.clone() };
    d.sequence = d.sequence.checked_add(1).expect("read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 62));
    d.landing
        .pending
        .insert(
            owner,
            Pending {
                task,
                repository: repo,
                resource: gather_resource(&read),
                cached: false,
                entry,
                heard,
                facts: facts.into_boxed(),
                stage: Stage::Branch(0),
                polled: List::with_capacity(slots(&env.limits).expect("bounded gates")),
                trees: List::with_capacity(env.limits.trees_in_flight),
                base_trees: List::with_capacity(env.limits.trees_in_flight),
                entries: List::with_capacity(env.limits.key_entries),
                woken: false,
                evidence: evidence.into_boxed(),
                paths: paths.into_boxed(),
                opened: None,
                upstream: None,
                reads: Map::with_capacity(env.limits.landing_members.checked_mul(5).expect("u16 members fit")),
            },
        )
        .expect("one chain per landing");
    child(d, env, skein_forge_client::Event::Read { owner, repository: repo, read }, out);
}
fn ask(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: Pending,
    repository: api::Repository,
    read: api::Read,
    out: &mut Queue<Request>,
) {
    pending.repository = repository;
    pending.resource = gather_resource(&read);
    pending.cached = match pending.resource.clone() {
        Some(resource) => pending.reads.contains_key(&(repository, resource)),
        None => false,
    };
    let cached = pending.cached;
    d.landing.pending.insert(owner, pending).expect("replaces pending chain");
    if !cached {
        child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
    }
}
fn poll_at(d: &Domain, env: &Env<Limits>, task: u64) -> Wall {
    let mut at = None;
    if let Some(row) = d.landing.rows.get(&task) {
        for member in &row.landing.members {
            let due = d.poll_at(env, repository(member));
            at = Some(match at {
                Some(prior) => due.min(prior),
                None => due,
            });
        }
    }
    let at = at.unwrap_or(env.now.saturating_add(env.limits.client.poll));
    wall_at(env, at)
}
fn wall_at(env: &Env<Limits>, at: skein_lib::Time) -> Wall {
    Wall::from_nanos(env.wall.as_nanos().saturating_add(at.as_nanos().saturating_sub(env.now.as_nanos())))
}
fn wait(d: &Domain, task: u64, env: &Env<Limits>, repository: api::Repository, out: &mut Queue<Request>) {
    emit(
        out,
        Request::LandingDecision {
            task,
            decision: policy::Decision::Wait { until: Some(wall_at(env, d.poll_at(env, repository))) },
            entry: None,
            evidence: None,
        },
    );
}
pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    advance(d, env, owner, answer, out);
    while let Some((owner, answer)) = cached_answer(d) {
        advance(d, env, owner, answer, out);
    }
}
fn cached_answer(d: &Domain) -> Option<(Token, Result<api::Answer, api::Error>)> {
    for (owner, pending) in &d.landing.pending {
        if pending.cached {
            let resource = pending.resource.clone().expect("only a resource read is cached");
            let answer = pending.reads.get(&(pending.repository, resource)).expect("cached read remains");
            return Some((*owner, answer.clone()));
        }
    }
    None
}
#[expect(clippy::too_many_lines, reason = "one exhaustive bounded landing read chain")]
#[expect(clippy::match_like_matches_macro, reason = "the foundation forbids matches macros in step code")]
fn advance(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut p) = d.landing.pending.remove(&owner) else {
        return;
    };
    p.cached = false;
    if d.stopping {
        return;
    }
    if answer == Err(api::Error::Busy) {
        wait(d, p.task, env, p.repository, out);
        return;
    }
    let Some(row) = d.landing.rows.get(&p.task).cloned() else {
        return;
    };
    if p.evidence.len() != row.landing.members.len() {
        step(d, env, p.task, p.entry, p.heard, out);
        return;
    }
    for (member, evidence) in row.landing.members.iter().zip(&p.evidence) {
        if member.change != evidence.change {
            step(d, env, p.task, p.entry, p.heard, out);
            return;
        }
    }
    if let Some(resource) = p.resource.take() {
        p.reads.insert((p.repository, resource), answer.clone()).expect("five resources per member");
    }
    match p.stage.clone() {
        Stage::Branch(index) | Stage::Main(index) | Stage::Ancestry(index) | Stage::Pull(index) => {
            let member = row.landing.members.get(usize::from(index)).expect("member");
            let Some(change) = d.changes.get(&member.change) else {
                return;
            };
            let repo = change.repository;
            let fact = p.facts.get_mut(usize::from(index)).expect("member fact");
            let read = match p.stage {
                Stage::Branch(_) => {
                    let head = match answer {
                        Ok(api::Answer::Commit(head)) => head,
                        Ok(_) | Err(_) => {
                            // Its change can finish and release the branch as soon as
                            // the merge is read back, before this landing's next read.
                            let landed = match change.change.state {
                                change::State::Landed { merge, .. }
                                | change::State::Red { merge, .. }
                                | change::State::Cleared { merge }
                                | change::State::Uncleared { merge } => Some(merge),
                                change::State::Producing { .. }
                                | change::State::Opening { .. }
                                | change::State::Recreating { .. }
                                | change::State::Reopening { .. }
                                | change::State::Checking { .. }
                                | change::State::Judging { .. }
                                | change::State::Gating { .. }
                                | change::State::Ready { .. }
                                | change::State::Preparing { .. }
                                | change::State::CancellingGates { .. }
                                | change::State::Queued { .. }
                                | change::State::First { .. }
                                | change::State::Updating { .. }
                                | change::State::WaitingProducer { .. }
                                | change::State::Resolving { .. }
                                | change::State::Repairing { .. }
                                | change::State::Landing { .. }
                                | change::State::NoChange { .. }
                                | change::State::Ended
                                | change::State::Held { .. } => {
                                    if member.left.is_some() {
                                        member.head
                                    } else {
                                        member.landed
                                    }
                                }
                            };
                            let Some(head) = landed else {
                                wait(d, p.task, env, p.repository, out);
                                return;
                            };
                            if Some(head) != member.candidate {
                                wait(d, p.task, env, p.repository, out);
                                return;
                            }
                            head
                        }
                    };
                    fact.head = head;
                    p.stage = Stage::Main(index);
                    api::Read::Branch { branch: change.base.clone() }
                }
                Stage::Main(_) => {
                    let Ok(api::Answer::Commit(main)) = answer else {
                        wait(d, p.task, env, p.repository, out);
                        return;
                    };
                    fact.main = main;
                    if member.candidate.is_some() && !row.landing.round.branch.is_empty() {
                        p.stage = Stage::Pull(index);
                        api::Read::Pull { number: member.pull, ci: false }
                    } else {
                        p.stage = Stage::Ancestry(index);
                        api::Read::Commits { before: fact.head, after: main, page: 1 }
                    }
                }
                Stage::Ancestry(_) => {
                    fact.contains_main = match crate::domain::ancestry_status(answer) {
                        change::Status::Passed => policy::Status::Passed,
                        change::Status::Failed => policy::Status::Failed,
                        change::Status::Unknown | change::Status::Pending => policy::Status::Unknown,
                    };
                    p.stage = Stage::Pull(index);
                    api::Read::Pull { number: member.pull, ci: false }
                }
                Stage::Pull(_) => {
                    let Ok(api::Answer::Pull(pull)) = answer else {
                        wait(d, p.task, env, p.repository, out);
                        return;
                    };
                    let evidence = &mut p.evidence.get_mut(usize::from(index)).expect("member evidence").facts;
                    evidence.head = Some(pull.commit);
                    evidence.size = pull.size;
                    fact.merged = pull.merged;
                    fact.ready =
                        if let change::State::Ready { .. } | change::State::Judging { .. } = change.change.state {
                            true
                        } else {
                            false
                        };
                    fact.held = p.heard.held.contains(&member.change)
                        || if let change::State::Held { .. } = change.change.state { true } else { false };
                    fact.cancelled = change.change.state == change::State::Ended;
                    fact.asked = row.asks.get(usize::from(index)).expect("member ask").is_some();
                    fact.cancelled |= p.heard.cancelled;
                    p.stage = Stage::Reviews(index, 1);
                    api::Read::Reviews { number: member.pull, page: 1, skip: 0 }
                }
                Stage::Reviews(_, _)
                | Stage::Paths(_)
                | Stage::Collect
                | Stage::Round { .. }
                | Stage::Run { .. }
                | Stage::FailedJobs { .. }
                | Stage::Tree { .. } => {
                    unreachable!("member stage")
                }
            };
            ask(d, env, owner, p, repo, read, out);
        }
        Stage::Reviews(index, page) => {
            let evidence = &mut p.evidence.get_mut(usize::from(index)).expect("member evidence").facts;
            if let Ok(api::Answer::Reviews { reviews, more, .. }) = answer {
                let mut gathered = List::with_capacity(env.limits.client.inbox);
                for review in &evidence.reviews {
                    gathered.push(review.clone()).expect("bounded reviews");
                }
                let mut fits = true;
                for review in reviews {
                    if gathered.push(review).is_err() {
                        fits = false;
                        break;
                    }
                }
                evidence.reviews = gathered.into_boxed();
                if fits && more && page < env.limits.client.rows {
                    let member = row.landing.members.get(usize::from(index)).expect("member");
                    let page = page.checked_add(1).expect("bounded review pages");
                    p.stage = Stage::Reviews(index, page);
                    ask(
                        d,
                        env,
                        owner,
                        p,
                        repository(member),
                        api::Read::Reviews { number: member.pull, page, skip: 0 },
                        out,
                    );
                    return;
                }
                evidence.reviews_complete = fits && !more;
            }
            let member = row.landing.members.get(usize::from(index)).expect("member");
            let change = d.changes.get(&member.change).expect("member's change");
            if let (Some(after), Some(repository)) = (member.candidate, d.repositories.get(&change.repository)) {
                for held in &d.landing_rules(repository.project) {
                    if !held.rule.sets.is_empty() && crate::covers_branch(&held.rule.pattern, repository, &change.base)
                    {
                        let before = p.facts.get(usize::from(index)).expect("member fact").main;
                        let repo = change.repository;
                        p.stage = Stage::Paths(index);
                        ask(d, env, owner, p, repo, api::Read::Compare { before, after }, out);
                        return;
                    }
                }
            }
            next_member(d, env, owner, p, &row, index, out);
        }
        Stage::Paths(index) => {
            let member = row.landing.members.get(usize::from(index)).expect("member");
            let fact = p.facts.get(usize::from(index)).expect("member fact");
            let paths = p.paths.get_mut(usize::from(index)).expect("member paths");
            if let Ok(api::Answer::Compare { before, after, files, changed, .. }) = answer {
                paths.whole = before == fact.main
                    && Some(after) == member.candidate
                    && skein_lib::len_u64(files.len()) == changed;
                paths.files = files;
            }
            next_member(d, env, owner, p, &row, index, out);
        }
        Stage::FailedJobs { key, run, next, words } => {
            let mut log = List::with_capacity(8192);
            append_words(&mut log, &words);
            match answer {
                Ok(api::Answer::Job { step, log: tail, truncated, .. }) => {
                    if let Some(step) = step {
                        append_words(&mut log, b"Failed step: ");
                        append_words(&mut log, &step.name);
                        append_words(&mut log, b"\n");
                    }
                    append_words(&mut log, b"End of its log:\n");
                    if truncated {
                        append_words(&mut log, b"[log prefix omitted]\n");
                    }
                    append_words(&mut log, &tail);
                    append_words(&mut log, b"\n");
                }
                Ok(_) | Err(_) => append_words(&mut log, b"[job log could not be read]\n"),
            }
            failed_jobs(d, env, owner, p, key, run, next, log.into_boxed(), out);
        }
        Stage::Round { stage, candidate } => {
            round_read(d, env, owner, p, &row, stage, candidate, answer, out);
        }
        Stage::Run { key, pages } => {
            let writer = d.writers.get(&key.repository.forge).copied().unwrap_or(0);
            let flight = d.landing.flights.get_mut(&key).expect("in-flight read");
            let previous_run = match &flight.run {
                Some(run) => Some(run.id),
                None => None,
            };
            let listing = if let Ok(api::Answer::Runs { .. }) = &answer { true } else { false };
            let ended = flight.answered(answer, writer, env.now, env.limits.client.lifetime);
            let next = if listing && pages < env.limits.client.rows { flight.read() } else { None };
            let candidate = flight.commit;
            let current_run = match &flight.run {
                Some(run) => Some(run.id),
                None => None,
            };
            if previous_run != current_run {
                mark_key(d, &key, out);
            }
            if let Some(ended) = ended {
                match ended.outcome {
                    policy::Verdict::Passed | policy::Verdict::Failed => {
                        let run = ended.run.expect("a verdict has a run");
                        if ended.outcome == policy::Verdict::Failed {
                            failed_jobs(d, env, owner, p, key, run, 0, Box::new([]), out);
                            return;
                        }
                        let receipt = Receipt {
                            logs: Box::new([]),
                            failures: None,
                            passed: ended.outcome == policy::Verdict::Passed,
                            candidate,
                            ended: env.wall,
                            run: ReceiptRun { id: run.id, number: run.number, url: run.url },
                        };
                        put_receipt(d, env, key.clone(), receipt, out);
                        finish_flight(d, &key, out);
                    }
                    policy::Verdict::Fault(_) => {
                        let mut position = None;
                        for (index, member) in row.landing.members.iter().enumerate() {
                            if repository(member) == key.repository {
                                for (number, proof) in member.gates.iter().enumerate() {
                                    if proof_key(proof.proof) == Some(key.key) {
                                        position = Some((
                                            p.task,
                                            u16::try_from(index).expect("member"),
                                            u16::try_from(number).expect("gate"),
                                        ));
                                    }
                                }
                            }
                        }
                        if let Some(position) = position
                            && let Some(flight) = d.landing.flights.remove(&key)
                        {
                            d.landing.faults.insert(position, flight).expect("bounded faults");
                        }
                    }
                    policy::Verdict::Running => {}
                }
            } else if let Some(read) = next {
                p.stage = Stage::Run { key: key.clone(), pages: pages.checked_add(1).expect("bounded pages") };
                ask(d, env, owner, p, key.repository, read, out);
                return;
            }
            p.polled.push(key).expect("one poll per flight");
            p.stage = Stage::Collect;
            collect(d, env, owner, p, out);
        }
        Stage::Tree { member, gate, candidate, object, page, main, reading_base } => {
            let Ok(api::Answer::Tree { entries, more }) = answer else {
                wait(d, p.task, env, p.repository, out);
                return;
            };
            let repo = repository(row.landing.members.get(usize::from(member)).expect("member"));
            for entry in entries {
                if p.entries
                    .push(policy::Entry {
                        name: entry.name,
                        mode: entry.mode,
                        object: entry.object,
                        kind: match entry.kind {
                            api::TreeEntryKind::Blob => policy::EntryKind::Blob,
                            api::TreeEntryKind::Tree => policy::EntryKind::Tree,
                            api::TreeEntryKind::Commit => policy::EntryKind::Commit,
                        },
                    })
                    .is_err()
                {
                    crate::refusal::limit(
                        out,
                        p.task,
                        repo,
                        "forge.key_entries",
                        env.limits.key_entries,
                        u64::from(p.entries.len()).saturating_add(1),
                    );
                    return;
                }
            }
            if more {
                let Some(page) = page.checked_add(1) else {
                    emit(out, Request::Refused { task: p.task });
                    return;
                };
                p.stage = Stage::Tree { member, gate, candidate, object, page, main, reading_base };
                ask(d, env, owner, p, repo, api::Read::Tree { object, page }, out);
                return;
            }
            let entries = core::mem::replace(&mut p.entries, List::with_capacity(env.limits.key_entries));
            let count = p.trees.len().saturating_add(p.base_trees.len());
            if count == env.limits.trees_in_flight {
                crate::refusal::limit(
                    out,
                    p.task,
                    repo,
                    "forge.trees_in_flight",
                    env.limits.trees_in_flight,
                    u64::from(count).saturating_add(1),
                );
                return;
            }
            let trees = if reading_base { &mut p.base_trees } else { &mut p.trees };
            if trees.push(policy::Tree { object, entries: entries.into_boxed() }).is_err() {
                crate::refusal::limit(
                    out,
                    p.task,
                    repo,
                    "forge.trees_in_flight",
                    env.limits.trees_in_flight,
                    u64::from(count).saturating_add(1),
                );
                return;
            }
            let configured = d.train_gates(&row, repo).get(usize::from(gate)).expect("configured gate").clone();
            let mut keyed = policy::key(
                policy::Repository { forge: repo.forge, repository: repo.repository },
                &configured.name,
                &policy::definition(&configured),
                &configured.ignored,
                candidate,
                p.trees.as_slice(),
                None,
                &policy::KeyLimits { entries: env.limits.key_entries, path_bytes: env.limits.key_path_bytes },
            );
            let mut reading_base = false;
            if configured.base
                && let policy::Keyed::Key(_) = keyed
            {
                let base = if main {
                    candidate
                } else {
                    row.landing.members.get(usize::from(member)).expect("member").base.expect("prepared base")
                };
                keyed = policy::key(
                    policy::Repository { forge: repo.forge, repository: repo.repository },
                    &configured.name,
                    &policy::definition(&configured),
                    &configured.ignored,
                    candidate,
                    p.trees.as_slice(),
                    Some((base, if base == candidate { p.trees.as_slice() } else { p.base_trees.as_slice() })),
                    &policy::KeyLimits { entries: env.limits.key_entries, path_bytes: env.limits.key_path_bytes },
                );
                reading_base = true;
            }
            match keyed {
                policy::Keyed::Need(object) => {
                    p.stage = Stage::Tree { member, gate, candidate, object, page: 1, main, reading_base };
                    ask(d, env, owner, p, repo, api::Read::Tree { object, page: 1 }, out);
                }
                policy::Keyed::Key(key) => {
                    if main {
                        d.landing
                            .main_keys
                            .insert((repo, configured.name.clone()), (candidate, key))
                            .expect("bounded repository gates");
                        p.trees = List::with_capacity(env.limits.trees_in_flight);
                        p.base_trees = List::with_capacity(env.limits.trees_in_flight);
                        p.stage = Stage::Collect;
                        collect(d, env, owner, p, out);
                    } else {
                        d.landing
                            .keys
                            .insert(
                                (p.task, member, gate),
                                (
                                    candidate,
                                    if configured.base {
                                        row.landing.members.get(usize::from(member)).expect("member").base
                                    } else {
                                        None
                                    },
                                    key,
                                ),
                            )
                            .expect("bounded landing gates");
                        emit(
                            out,
                            Request::LandingDecision {
                                task: p.task,
                                decision: policy::Decision::Key { member, gate, candidate },
                                entry: None,
                                evidence: None,
                            },
                        );
                    }
                }
                policy::Keyed::TooLarge { bound: None, .. } => emit(out, Request::Refused { task: p.task }),
                policy::Keyed::TooLarge { bound: Some(bound), needed } => {
                    let (member, configured) = match bound {
                        policy::KeyBound::Entries => ("forge.key_entries", env.limits.key_entries),
                        policy::KeyBound::PathBytes => ("forge.key_path_bytes", env.limits.key_path_bytes),
                    };
                    crate::refusal::limit(out, p.task, repo, member, configured, needed);
                }
            }
        }
        Stage::Collect => unreachable!("collect emits the next read itself"),
    }
}
fn next_member(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    index: u16,
    out: &mut Queue<Request>,
) {
    let next = index.checked_add(1).expect("member index");
    if let Some(member) = row.landing.members.get(usize::from(next)) {
        let change = d.changes.get(&member.change).expect("member's change");
        let repo = change.repository;
        let read = api::Read::Branch { branch: change.branch.clone() };
        p.stage = Stage::Branch(next);
        ask(d, env, owner, p, repo, read, out);
        return;
    }
    gathered(d, env, owner, p, row, out);
}
fn gathered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    out: &mut Queue<Request>,
) {
    if row.landing.round.branch.is_empty() {
        p.stage = Stage::Collect;
        collect(d, env, owner, p, out);
        return;
    }
    // A validated immutable candidate is kept for the round. Only preparation
    // and an uncertain opening need the landing branch or pull read again.
    let member = row.landing.members.first().expect("one member");
    if let Some(candidate) = member.candidate
        && let Some(combined) = row.landing.round.combined
    {
        p.facts.first_mut().expect("one fact").candidate = Some((candidate, combined));
    }
    if row.landing.round.ending.is_some() || (member.candidate.is_some() && row.landing.round.pull.is_some()) {
        p.stage = Stage::Collect;
        collect(d, env, owner, p, out);
        return;
    }
    if member.candidate.is_some() {
        p.stage = Stage::Round { stage: RoundRead::Pull, candidate: member.candidate };
        let repo = p.repository;
        ask(
            d,
            env,
            owner,
            p,
            repo,
            api::Read::PullFor { head: row.landing.round.branch.clone(), base: row.landing.branch.clone() },
            out,
        );
        return;
    }
    let upstream = upstream_branch(d, row);
    p.stage = Stage::Round {
        stage: if upstream.is_some() { RoundRead::Upstream } else { RoundRead::Branch },
        candidate: None,
    };
    let read = api::Read::Branch { branch: upstream.unwrap_or(row.landing.round.branch.clone()) };
    let repo = p.repository;
    ask(d, env, owner, p, repo, read, out);
}

#[expect(clippy::too_many_arguments, reason = "one stage of the coherent round read")]
#[expect(clippy::too_many_lines, reason = "the bounded round read keeps each validation beside its next read")]
fn round_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    stage: RoundRead,
    candidate: Option<api::Commit>,
    answer: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let member = row.landing.members.first().expect("one member");
    let fact = p.facts.first_mut().expect("one fact");
    let base = member.base.unwrap_or(fact.main);
    let head = member.head.unwrap_or(fact.head);
    let mut candidate = candidate;
    let read = match stage {
        RoundRead::Upstream => {
            let Ok(api::Answer::Commit(tip)) = answer else {
                wait(d, p.task, env, p.repository, out);
                return;
            };
            p.upstream = Some(tip);
            p.stage = Stage::Round { stage: RoundRead::Branch, candidate: None };
            let repo = p.repository;
            ask(d, env, owner, p, repo, api::Read::Branch { branch: row.landing.round.branch.clone() }, out);
            return;
        }
        RoundRead::Branch => {
            candidate = match answer {
                Ok(api::Answer::Commit(commit)) => Some(commit),
                Err(api::Error::Missing) => None,
                Ok(_) | Err(_) => {
                    wait(d, p.task, env, p.repository, out);
                    return;
                }
            };
            match candidate {
                Some(tip) => api::Read::Commits { before: tip, after: base, page: 1 },
                None => {
                    p.stage = Stage::Round { stage: RoundRead::Contained, candidate };
                    let repo = p.repository;
                    ask(d, env, owner, p, repo, api::Read::Commits { before: base, after: head, page: 1 }, out);
                    return;
                }
            }
        }
        RoundRead::Base => {
            match crate::domain::ancestry_status(answer) {
                change::Status::Passed => {}
                change::Status::Failed => candidate = None,
                change::Status::Unknown | change::Status::Pending => {
                    wait(d, p.task, env, p.repository, out);
                    return;
                }
            }
            match candidate {
                Some(tip) => api::Read::Commits { before: tip, after: head, page: 1 },
                None => api::Read::Commits { before: base, after: head, page: 1 },
            }
        }
        RoundRead::Head => {
            if let Some(tip) = candidate {
                match crate::domain::ancestry_status(answer) {
                    change::Status::Passed => fact.candidate = Some((tip, policy::Combine::Merge)),
                    change::Status::Failed => {}
                    change::Status::Unknown | change::Status::Pending => {
                        wait(d, p.task, env, p.repository, out);
                        return;
                    }
                }
                api::Read::Commits { before: base, after: tip, page: 1 }
            } else {
                fact.contained = if crate::domain::ancestry_status(answer) == change::Status::Passed {
                    policy::Status::Passed
                } else {
                    policy::Status::Failed
                };
                p.stage = Stage::Round { stage: RoundRead::Pull, candidate };
                let repo = p.repository;
                let read =
                    api::Read::PullFor { head: row.landing.round.branch.clone(), base: row.landing.branch.clone() };
                ask(d, env, owner, p, repo, read, out);
                return;
            }
        }
        RoundRead::Squash => {
            if fact.candidate.is_none()
                && let Some(tip) = candidate
                && let Ok(api::Answer::Commits { commits, more: false }) = answer
                && commits.len() == 1
                && let Some(commit) = commits.first()
                && commit.id == tip
                && commit.parents.as_ref() == [base]
                && squash_head(&commit.message, head)
            {
                fact.candidate = Some((tip, policy::Combine::Squash));
            }
            api::Read::Commits { before: base, after: head, page: 1 }
        }
        RoundRead::Contained => {
            fact.contained = if crate::domain::ancestry_status(answer) == change::Status::Passed {
                policy::Status::Passed
            } else {
                policy::Status::Failed
            };
            api::Read::PullFor { head: row.landing.round.branch.clone(), base: row.landing.branch.clone() }
        }
        RoundRead::Pull => {
            if let Ok(api::Answer::Pull(pull)) = answer
                && (pull.state == api::State::Open || pull.merged == member.candidate)
                && Some(pull.commit) == member.candidate
            {
                p.opened = Some(pull.number);
            }
            p.stage = Stage::Collect;
            collect(d, env, owner, p, out);
            return;
        }
    };
    let stage = match stage {
        RoundRead::Branch => RoundRead::Base,
        RoundRead::Base => RoundRead::Head,
        RoundRead::Head => RoundRead::Squash,
        RoundRead::Squash => RoundRead::Contained,
        RoundRead::Contained => RoundRead::Pull,
        RoundRead::Pull | RoundRead::Upstream => unreachable!("terminal stages return above"),
    };
    p.stage = Stage::Round { stage, candidate };
    let repo = p.repository;
    ask(d, env, owner, p, repo, read, out);
}
fn upstream_branch(d: &Domain, row: &LandingRow) -> Option<Box<[u8]>> {
    if row.landing.members.first()?.base.is_some() {
        return None;
    }
    let repo = repository(row.landing.members.first()?);
    let adopted = d.repository(repo)?;
    let resource = skein_forge_client::Resource {
        repository: repo,
        what: skein_forge_client::What::Branch(row.landing.branch.clone()),
    };
    let name = super::domain::from_client(&resource, &d.landing.limits)?;
    if !d.holds.contains_key(&name) || crate::items::goal_branch(&name, &adopted.prefix).is_none() {
        return None;
    }
    Some(adopted.settings.default_branch.clone())
}

fn squash_head(message: &[u8], head: api::Commit) -> bool {
    let expected = concat(&[b"Temper-Head: ", &commit_hex(head)]);
    let mut start = 0;
    for (index, byte) in message.iter().enumerate() {
        if *byte == b'\n' {
            if message.get(start..index) == Some(expected.as_ref()) {
                return true;
            }
            start = index.checked_add(1).expect("message bound");
        }
    }
    message.get(start..) == Some(expected.as_ref())
}

fn collect(d: &mut Domain, env: &Env<Limits>, owner: Token, mut p: Pending, out: &mut Queue<Request>) {
    let row = d.landing.rows.get(&p.task).expect("pending landing");
    for (index, member) in row.landing.members.iter().enumerate() {
        let repo = repository(member);
        for (number, gate) in d.train_gates(row, repo).iter().enumerate() {
            let main = p.facts.get(index).expect("member fact").main;
            if match d.landing.main_keys.get(&(repo, gate.name.clone())) {
                Some((tip, _)) => *tip != main,
                None => true,
            } {
                p.stage = Stage::Tree {
                    member: u16::try_from(index).expect("member position"),
                    gate: u16::try_from(number).expect("gate position"),
                    candidate: main,
                    object: main,
                    page: 1,
                    main: true,
                    reading_base: false,
                };
                ask(d, env, owner, p, repo, api::Read::Tree { object: main, page: 1 }, out);
                return;
            }
        }
    }
    for (key, flight) in &d.landing.flights {
        let mut posted = false;
        for member in &row.landing.members {
            if repository(member) != key.repository {
                continue;
            }
            for proof in &member.gates {
                if proof_key(proof.proof) == Some(key.key) && proof.posted != policy::Posted::Nothing {
                    posted = true;
                }
            }
        }
        if !posted || flight.task != p.task || p.polled.as_slice().contains(key) {
            continue;
        }
        if let Some(read) = flight.read() {
            let key = key.clone();
            p.stage = Stage::Run { key: key.clone(), pages: 1 };
            ask(d, env, owner, p, key.repository, read, out);
            return;
        }
    }
    if p.entry == 0 {
        emit(out, Request::WakeProcedure { task: p.task });
    } else if p.woken {
        step(d, env, p.task, p.entry, p.heard, out);
    } else {
        decide(d, env, owner, p, out);
    }
}
fn result(d: &Domain, task: u64, member: u16, gate: u16, operation: u8) -> policy::Status {
    d.landing.results.get(&(task, member, gate, operation)).copied().unwrap_or(policy::Status::Unknown)
}
#[expect(clippy::too_many_lines, reason = "one policy decision and its atomic durable outputs")]
fn decide(d: &mut Domain, env: &Env<Limits>, owner: Token, mut p: Pending, out: &mut Queue<Request>) {
    let mut row = d.landing.rows.get(&p.task).expect("pending landing").clone();
    if p.heard.released {
        d.landing.results.remove(&(p.task, 0, 0, 9));
    }
    let unprepared = match row.landing.members.first() {
        Some(member) => member.base.is_none(),
        None => false,
    };
    if unprepared {
        row.landing.round.upstream = p.upstream;
    }
    if p.heard.preparation == policy::Preparation::Failed || p.heard.preparation == policy::Preparation::Conflict {
        row.landing.round.failure.clone_from(&p.heard.failure);
    }
    if continue_retry(d, &mut row, p.entry, out) {
        return;
    }
    for (index, (member, fact)) in row.landing.members.iter().zip(p.facts.iter_mut()).enumerate() {
        let position = u16::try_from(index).expect("member position");
        fact.merge = result(d, p.task, position, 0, 5);
        if !row.landing.round.branch.is_empty()
            && let Some(prefix) = member.candidate
        {
            let change = d.changes.get(&member.change).expect("member change");
            fact.judgement = policy::Status::Passed;
            for gate in &change.change.gates {
                if !gate.blocking {
                    continue;
                }
                if !row.landing.round.judging
                    && (gate.kind == change::GateKind::Check || gate.freshness == change::Freshness::Exact)
                {
                    fact.judgement = policy::Status::Pending;
                    continue;
                }
                match change::prefix_verdict(
                    &change.change,
                    *gate,
                    &change.verdicts,
                    member.head.expect("round head"),
                    prefix,
                ) {
                    change::Status::Failed => fact.refused = Some(gate.number),
                    change::Status::Unknown | change::Status::Pending => fact.judgement = policy::Status::Pending,
                    change::Status::Passed => {}
                }
            }
        }
        fact.preparation = p.heard.preparation;
        fact.deleted = result(d, p.task, position, 0, 8);
        if p.heard.preparation == policy::Preparation::Conflict {
            fact.outcome = Some(policy::Outcome::Conflict);
        }
        fact.record = if member.landed.is_some() && fact.merged == member.landed {
            policy::Status::Passed
        } else {
            result(d, p.task, position, 0, 9)
        };
        for (number, gate) in d.train_gates(&row, repository(member)).iter().enumerate() {
            let number = u16::try_from(number).expect("gate position");
            let fact = fact.gates.get_mut(usize::from(number)).expect("gate fact");
            let candidate = member.candidate.unwrap_or([0; 32]);
            fact.key = match d.landing.keys.get(&(p.task, position, number)) {
                Some((tip, base, key)) if *tip == candidate && *base == if gate.base { member.base } else { None } => {
                    Some(*key)
                }
                Some(_) | None => None,
            };
            if let Some(key) = proof_key(member.gates.get(usize::from(number)).expect("gate proof").proof).or(fact.key)
            {
                let key = gate_key(&row, position, number, &gate.name, key);
                fact.receipt = match d.landing.receipts.get(&key) {
                    Some(receipt) if receipt.passed => policy::Status::Passed,
                    Some(_) => policy::Status::Failed,
                    None => policy::Status::Unknown,
                };
                fact.flight = flight_fact(&d.landing, &key, (p.task, position, number));
                if let Some(receipt) = d.landing.receipts.get(&key) {
                    fact.run = receipt.run.id;
                }
                if let Some(flight) = d.landing.flights.get(&key) {
                    if let Some(run) = &flight.run {
                        fact.run = run.id;
                    }
                    let settings = match d.landing.projects.get(&row.landing.project) {
                        Some((_, _, _, _, _, settings)) => *settings,
                        None => policy::Project::DEFAULT,
                    };
                    if let Some(since) = flight.since
                        && env.now.as_nanos().saturating_sub(since.as_nanos()) > settings.gate_timeout.as_nanos()
                        && flight.outcome.is_none()
                        && flight.run.is_some()
                    {
                        fact.expired = Some(fact.run);
                    }
                }
            }
            fact.posting = result(
                d,
                p.task,
                position,
                number,
                if member.gates.get(usize::from(number)).expect("gate proof").posted == policy::Posted::Verdict {
                    4
                } else {
                    3
                },
            );
        }
    }
    let facts = policy::Facts {
        members: p.facts.clone(),
        turn_free: if row.landing.round.branch.is_empty() { single_turn(d, &row, &p.heard.kept) } else { true },
        released: p.heard.released,
        opened: p.opened,
        opening: result(d, p.task, 0, 0, 6),
        closed: result(d, p.task, 0, 0, 7),
        preparation: p.heard.preparation,
    };
    let project = match d.landing.projects.get(&row.landing.project) {
        Some((_, _, _, _, _, settings)) => *settings,
        None => policy::Project::DEFAULT,
    };
    let stepped = policy::step(
        &row.landing,
        &facts,
        &project,
        &policy::Limits {
            members: env.limits.landing_members,
            gates: env.limits.gates,
            poll: skein_lib::Duration::from_nanos(
                poll_at(d, env, p.task).as_nanos().saturating_sub(env.wall.as_nanos()),
            ),
        },
        env.wall,
    );
    let prior = row.clone();
    row.landing = stepped.landing;
    if !row.landing.round.branch.is_empty()
        && let Some(member) = row.landing.members.first()
        && member.candidate != prior.landing.members.first().expect("member").candidate
        && let Some(candidate) = member.candidate
    {
        let resource = skein_forge_client::Resource {
            repository: repository(member),
            what: skein_forge_client::What::Branch(row.landing.round.branch.clone()),
        };
        if let Some(name) = super::domain::from_client(&resource, &env.limits) {
            super::domain::record_written_head(d, env, resource, name, candidate, p.task, out);
        }
    }
    let decision = stepped.decision;
    for (member, old) in row.landing.members.iter().zip(&prior.landing.members) {
        if member.landed.is_some() && old.landed.is_none() {
            emit(out, Request::WakeProcedure { task: member.change });
        }
    }
    if row.landing.members.len() != prior.landing.members.len() {
        remap_members(d, &mut row, &prior);
    }
    let evidence = if let policy::Decision::Merge { member, candidate }
    | policy::Decision::Record { member, prefix: candidate } = decision
    {
        let fact = p.facts.get(usize::from(member)).expect("fresh member facts");
        let mut evidence = p.evidence.get(usize::from(member)).expect("member evidence").clone();
        if evidence.facts.head != Some(candidate) {
            evidence.facts.size = None;
        }
        evidence.facts.head = Some(candidate);
        let paths = p.paths.get(usize::from(member)).expect("member paths");
        let classes = d.project_classes(row.landing.project);
        evidence.facts.touched = crate::touched(&classes, &paths.files, paths.whole);
        evidence.facts.base_tip = fact.main;
        evidence.facts.contains_base =
            match if fact.candidate.is_some() { policy::Status::Passed } else { fact.contains_main } {
                policy::Status::Unknown => change::Status::Unknown,
                policy::Status::Pending => change::Status::Pending,
                policy::Status::Passed => change::Status::Passed,
                policy::Status::Failed => change::Status::Failed,
            };
        evidence.facts.ci = change::Status::Passed;
        for gate in &fact.gates {
            if gate.blocking && gate.receipt != policy::Status::Passed {
                evidence.facts.ci = if gate.receipt == policy::Status::Failed {
                    change::Status::Failed
                } else {
                    change::Status::Unknown
                };
            }
        }
        Some(evidence)
    } else {
        None
    };
    let mut committed = None;
    match decision {
        policy::Decision::Open
        | policy::Decision::Close
        | policy::Decision::Delete { .. }
        | policy::Decision::Record { .. } => {
            let member = row.landing.members.first().expect("one member");
            if d.entries.contains_key(&p.entry) || d.entries.len() == env.limits.entries {
                refuse_entry(d, env, &p, &row, 0, out);
                return;
            }
            let change = d.changes.get(&member.change).expect("member change");
            let (op, write, condition) = match decision {
                policy::Decision::Open => {
                    let number = skein_lib::Decimal::of(member.pull);
                    let change_number = skein_lib::Decimal::of(member.change);
                    let round = skein_lib::Decimal::of(u64::from(row.landing.preparation));
                    let title = concat(&[b"Land #", number.as_bytes(), b": ", &change.title]);
                    let body = concat(&[
                        b"Change ",
                        change_number.as_bytes(),
                        b", pull request #",
                        number.as_bytes(),
                        b", head ",
                        &commit_hex(member.head.expect("round head")),
                        b".\nRound ",
                        round.as_bytes(),
                        b", base ",
                        &commit_hex(member.base.expect("round base")),
                        b".\n",
                    ]);
                    (
                        Operation::Open,
                        api::Write::OpenPull {
                            title,
                            body,
                            head: row.landing.round.branch.clone(),
                            base: row.landing.branch.clone(),
                        },
                        skein_forge_client::Condition::None,
                    )
                }
                policy::Decision::Close => (
                    Operation::Close,
                    api::Write::Close { number: row.landing.round.pull.expect("round pull") },
                    skein_forge_client::Condition::None,
                ),
                policy::Decision::Delete { member: number } => {
                    assert_eq!(number, 0, "the top executes one-member landings");
                    (
                        Operation::Delete,
                        api::Write::DeleteBranch { branch: member.branch.clone() },
                        skein_forge_client::Condition::None,
                    )
                },
                policy::Decision::Record { prefix, .. } => (
                    Operation::Record,
                    api::Write::Merge {
                        number: member.pull,
                        head: member.head.expect("member head"),
                        how: api::MergeHow::Recorded { commit: prefix },
                    },
                    skein_forge_client::Condition::Merge { base: row.landing.branch.clone() },
                ),
                policy::Decision::None
                | policy::Decision::Wait { .. }
                | policy::Decision::Prepare { .. }
                | policy::Decision::Key { .. }
                | policy::Decision::Dispatch { .. }
                | policy::Decision::Retry { .. }
                | policy::Decision::Post { .. }
                | policy::Decision::Repair { .. }
                | policy::Decision::Proved
                | policy::Decision::Merge { .. }
                | policy::Decision::Stale
                | policy::Decision::Finish
                | policy::Decision::Cancel
                | policy::Decision::Hold(_)
                | policy::Decision::Landed { .. }
                | policy::Decision::Resolve { .. }
                | policy::Decision::Leave { .. }
                | policy::Decision::Judge { .. } => unreachable!("round write"),
            };
            keep(d, &mut row, p.entry, 0, 0, op, write, condition, out);
            committed = Some(p.entry);
        }
        policy::Decision::Landed { member, prefix } => {
            let change = row.landing.members.get(usize::from(member)).expect("member").change;
            let signed = match d.changes.get(&change) {
                Some(row) => row.signed.clone(),
                None => Box::new([]),
            };
            crate::landing_log::landed(d, env.wall, prefix, change, signed, out);
            crate::verification::landed(d, prefix, change, out);
            *row.asks.get_mut(usize::from(member)).expect("member ask") = Some(change::Ask::Landed { prefix });
            emit(
                out,
                Request::WakeProcedure { task: row.landing.members.get(usize::from(member)).expect("member").change },
            );
        }
        policy::Decision::Resolve { member, base } => {
            *row.asks.get_mut(usize::from(member)).expect("member ask") = Some(change::Ask::Resolve { base });
            emit(
                out,
                Request::WakeProcedure { task: row.landing.members.get(usize::from(member)).expect("member").change },
            );
        }
        policy::Decision::Judge { member } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            if let Some(change) = d.changes.get_mut(&m.change) {
                let mut verdicts = List::with_capacity(env.limits.change_policy.gates);
                for verdict in &change.verdicts {
                    let mut exact = false;
                    for gate in &change.change.gates {
                        if gate.number == verdict.number
                            && gate.blocking
                            && (gate.freshness == change::Freshness::Exact || gate.kind == change::GateKind::Check)
                        {
                            exact = true;
                        }
                    }
                    if !exact {
                        verdicts.push(*verdict).expect("bounded verdicts");
                    }
                }
                change.verdicts = verdicts.into_boxed();
                let saved = change.clone();
                super::domain::save_change(d, out, saved);
            }
            *row.asks.get_mut(usize::from(member)).expect("ask") = Some(change::Ask::Judge(change::Judgement {
                landing: p.task,
                round: row.landing.preparation,
                pull: row.landing.round.pull.expect("round pull"),
                branch: row.landing.round.branch.clone(),
                base: row.landing.branch.clone(),
                prefix: m.candidate.expect("prefix"),
            }));
            emit(out, Request::WakeProcedure { task: m.change });
        }
        policy::Decision::Leave { member, why } => {
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let ask = match why {
                policy::Leave::Conflict => Some(change::Ask::Resolve { base: m.base.expect("round base") }),
                policy::Leave::Failed { gate, .. } => {
                    Some(change::Ask::Repair { gate, candidate: failed_candidate(d, &row, member, gate) })
                }
                policy::Leave::Cancelled
                | policy::Leave::Held
                | policy::Leave::HeadMoved { .. }
                | policy::Leave::Refused { .. }
                | policy::Leave::EarlierConflict
                | policy::Leave::Unit { .. }
                | policy::Leave::Dependency { .. }
                | policy::Leave::Fallback { .. } => None,
            };
            *row.asks.get_mut(usize::from(member)).expect("ask") = ask;
            emit(out, Request::WakeProcedure { task: m.change });
        }
        policy::Decision::Repair { member, gate } => {
            *row.asks.get_mut(usize::from(member)).expect("member ask") = Some(change::Ask::Repair {
                gate,
                candidate: row
                    .landing
                    .members
                    .get(usize::from(member))
                    .expect("member")
                    .candidate
                    .expect("proved candidate"),
            });
            emit(
                out,
                Request::WakeProcedure { task: row.landing.members.get(usize::from(member)).expect("member").change },
            );
        }
        policy::Decision::Key { member, gate, candidate } => {
            let repo = repository(row.landing.members.get(usize::from(member)).expect("member"));
            p.stage =
                Stage::Tree { member, gate, candidate, object: candidate, page: 1, main: false, reading_base: false };
            save(d, row, out);
            ask(d, env, owner, p, repo, api::Read::Tree { object: candidate, page: 1 }, out);
            return;
        }
        policy::Decision::Dispatch { member, gate, key, try_ }
        | policy::Decision::Retry { member, gate, key, try_ } => {
            if d.entries.contains_key(&p.entry) || d.entries.len() == env.limits.entries {
                refuse_entry(d, env, &p, &row, member, out);
                return;
            }
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let repo = repository(m);
            let configured = d.train_gates(&row, repo).get(usize::from(gate)).expect("configured gate").clone();
            let candidate = m.candidate.expect("candidate before dispatch");
            let key = gate_key(&row, member, gate, &configured.name, key);
            let purpose = purpose_identity(&row, member, gate, 1, try_);
            if let Some(mut fault) = d.landing.faults.remove(&(p.task, member, gate)) {
                runs::retry(&mut fault, try_);
                fault.purpose.clone_from(&purpose);
                d.landing.flights.insert(key.clone(), fault).expect("bounded key flights");
            } else if let Some(flight) = d.landing.flights.get_mut(&key) {
                runs::retry(flight, try_);
                flight.purpose = purpose;
            } else {
                let inputs = inputs(candidate, key.key, if configured.base { m.base } else { None });
                let flight = runs::follow(
                    p.task,
                    p.entry,
                    purpose,
                    repo,
                    configured.workflow.clone(),
                    dispatch_branch(d, &row, m),
                    candidate,
                    inputs,
                );
                d.landing.flights.insert(key.clone(), flight).expect("bounded key flights");
            }
            let flight = d.landing.flights.get_mut(&key).expect("new flight");
            let write = flight.write(p.entry).expect("dispatch or retry write");
            flight.outcome = None;
            let operation = if let api::Write::Rerun { .. } = write { Operation::Rerun } else { Operation::Dispatch };
            flight.purpose = purpose_identity(
                &row,
                member,
                gate,
                match operation {
                    Operation::Dispatch => 1,
                    Operation::Rerun => 2,
                    Operation::Pending => 3,
                    Operation::Verdict => 4,
                    Operation::Merge => 5,
                    Operation::Open => 6,
                    Operation::Close => 7,
                    Operation::Delete => 8,
                    Operation::Record => 9,
                },
                try_,
            );
            keep(d, &mut row, p.entry, member, gate, operation, write, skein_forge_client::Condition::None, out);
            mark_key(d, &key, out);
            committed = Some(p.entry);
        }
        policy::Decision::Post { member, gate, posting } => {
            if d.entries.contains_key(&p.entry) || d.entries.len() == env.limits.entries {
                refuse_entry(d, env, &p, &row, member, out);
                return;
            }
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let repo = repository(m);
            let configured = d.train_gates(&row, repo).get(usize::from(gate)).expect("configured gate").clone();
            let key = gate_key(
                &row,
                member,
                gate,
                &configured.name,
                proof_key(m.gates.get(usize::from(gate)).expect("gate proof").proof).expect("keyed posting"),
            );
            let (write, operation) = posting_write(
                m.candidate.expect("candidate"),
                &configured.name,
                &key,
                posting,
                d.landing.receipts.get(&key),
            );
            keep(d, &mut row, p.entry, member, gate, operation, write, skein_forge_client::Condition::None, out);
            committed = Some(p.entry);
        }
        policy::Decision::Merge { member, candidate } => {
            if d.entries.contains_key(&p.entry) || d.entries.len() == env.limits.entries {
                refuse_entry(d, env, &p, &row, member, out);
                return;
            }
            let m = row.landing.members.get(usize::from(member)).expect("member");
            let change = d.changes.get(&m.change).expect("member change");
            let condition = skein_forge_client::Condition::Merge { base: change.base.clone() };
            let write = api::Write::Merge {
                number: if row.landing.round.branch.is_empty() {
                    m.pull
                } else {
                    row.landing.round.pull.expect("round pull")
                },
                head: candidate,
                how: api::MergeHow::FastForwardOnly,
            };
            keep(d, &mut row, p.entry, member, 0, Operation::Merge, write, condition, out);
            committed = Some(p.entry);
        }
        policy::Decision::Proved => {
            if row.landing.round.branch.is_empty() {
                save(d, row.clone(), out);
                if !single_turn(d, &row, &p.heard.kept) {
                    save(d, row, out);
                    emit(
                        out,
                        Request::LandingDecision {
                            task: p.task,
                            decision: policy::Decision::None,
                            entry: None,
                            evidence: None,
                        },
                    );
                    return;
                }
            }
            if let policy::Phase::Proved { at } = row.landing.phase {
                row.landing.phase = policy::Phase::Turn { at };
            }
        }
        policy::Decision::Finish => {
            for member in &row.landing.members {
                if member.landed.is_none()
                    && row.asks.first().cloned().flatten().is_none()
                    && let Some(change) = d.changes.get_mut(&member.change)
                {
                    change.change.landing = None;
                    let saved = change.clone();
                    super::domain::save_change(d, out, saved);
                }
                emit(out, Request::WakeProcedure { task: member.change });
            }
        }
        policy::Decision::Wait { .. } => {
            // A member may still have been due when its ask was published.
            // Keep the ask level-triggered until its change consumes it.
            for (index, member) in row.landing.members.iter().enumerate() {
                if row.asks.get(index).cloned().flatten().is_some()
                    && let Some(change) = d.changes.get(&member.change)
                    && let change::State::Ready { .. } = change.change.state
                {
                    emit(out, Request::WakeProcedure { task: member.change });
                }
            }
        }
        policy::Decision::Prepare { member, head, main } => {
            if row.landing.round.branch.is_empty() {
                *row.asks.get_mut(usize::from(member)).expect("member ask") = Some(change::Ask::Prepare { head, main });
                emit(
                    out,
                    Request::WakeProcedure {
                        task: row.landing.members.get(usize::from(member)).expect("member").change,
                    },
                );
            }
        }
        policy::Decision::None | policy::Decision::Stale | policy::Decision::Cancel | policy::Decision::Hold(_) => {}
    }
    if row.landing.preparation != prior.landing.preparation || decision == policy::Decision::Stale {
        reset_results(d, &row, &prior);
    }
    let news = match decision {
        policy::Decision::Leave { .. } => Some(crate::LandingNews::Left),
        policy::Decision::Hold(_) => Some(crate::LandingNews::Held),
        policy::Decision::Landed { .. } => Some(crate::LandingNews::Landed),
        policy::Decision::None
        | policy::Decision::Wait { .. }
        | policy::Decision::Prepare { .. }
        | policy::Decision::Key { .. }
        | policy::Decision::Dispatch { .. }
        | policy::Decision::Retry { .. }
        | policy::Decision::Post { .. }
        | policy::Decision::Repair { .. }
        | policy::Decision::Proved
        | policy::Decision::Merge { .. }
        | policy::Decision::Stale
        | policy::Decision::Finish
        | policy::Decision::Cancel
        | policy::Decision::Open
        | policy::Decision::Record { .. }
        | policy::Decision::Close
        | policy::Decision::Delete { .. }
        | policy::Decision::Resolve { .. }
        | policy::Decision::Judge { .. } => {
            if row.landing.phase == policy::Phase::Proving && prior.landing.phase != policy::Phase::Proving {
                Some(crate::LandingNews::Proving)
            } else {
                None
            }
        }
    };
    save(d, row, out);
    if let Some(step) = news {
        emit(out, Request::LandingNews { task: p.task, step });
    }
    let decision = if prior.landing.round.branch.is_empty()
        && let policy::Decision::Prepare { .. } = decision
    {
        policy::Decision::None
    } else {
        decision
    };
    emit(out, Request::LandingDecision { task: p.task, decision, entry: committed, evidence });
}
fn dispatch_branch(d: &Domain, row: &LandingRow, member: &policy::Member) -> Box<[u8]> {
    if row.landing.round.branch.is_empty() {
        d.changes.get(&member.change).expect("member change").branch.clone()
    } else {
        row.landing.round.branch.clone()
    }
}
fn single_turn(d: &Domain, row: &LandingRow, kept: &[u64]) -> bool {
    let mut first = None;
    for (_, other) in &d.landing.rows {
        if other.landing.project != row.landing.project {
            continue;
        }
        match other.landing.phase {
            policy::Phase::Turn { .. } => return false,
            policy::Phase::Proved { at } => {
                if !kept.contains(&other.landing.task) {
                    let candidate = (at, other.landing.task);
                    let earlier = match first {
                        Some(prior) => candidate < prior,
                        None => true,
                    };
                    if earlier {
                        first = Some(candidate);
                    }
                }
            }
            policy::Phase::Waiting
            | policy::Phase::Preparing
            | policy::Phase::Proving
            | policy::Phase::Landed
            | policy::Phase::Held { .. }
            | policy::Phase::Cancelled => {}
        }
    }
    match first {
        Some((_, task)) => task == row.landing.task,
        None => false,
    }
}
fn purpose_identity(row: &LandingRow, member: u16, gate: u16, operation: u8, attempt: u16) -> Box<[u8]> {
    crate::purpose::EffectPurpose::Landing {
        task: row.landing.task,
        member,
        gate,
        operation,
        preparation: row.landing.preparation,
        attempt,
    }
    .identity()
    .expect("fixed purpose")
}
pub(crate) fn inputs(candidate: api::Commit, key: policy::Key, base: Option<api::Commit>) -> Box<[api::Input]> {
    let mut inputs = List::with_capacity(u32::try_from(api::INPUTS).expect("forge input bound"));
    inputs
        .push(api::Input { name: Box::from(b"temper_commit".as_slice()), value: commit_hex(candidate) })
        .expect("commit input");
    inputs.push(api::Input { name: Box::from(b"temper_key".as_slice()), value: hex(&key) }).expect("key input");
    if let Some(base) = base {
        inputs
            .push(api::Input { name: Box::from(b"temper_base".as_slice()), value: commit_hex(base) })
            .expect("base input");
    }
    inputs.into_boxed()
}
fn commit_hex(commit: api::Commit) -> Box<[u8]> {
    let mut length = 32;
    if commit[20..] == [0; 12] {
        length = 20;
    }
    hex(commit.get(..length).expect("commit width"))
}

fn concat(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded words");
    }
    let mut bytes = List::with_capacity(u32::try_from(length).expect("bounded words"));
    for part in parts {
        for byte in *part {
            bytes.push(*byte).expect("sized words");
        }
    }
    bytes.into_boxed()
}
fn posting_write(
    candidate: api::Commit,
    name: &[u8],
    key: &ReceiptKey,
    posting: policy::Posting,
    receipt: Option<&Receipt>,
) -> (api::Write, Operation) {
    let short = hex(&key.key[..8]);
    let (check, ending, operation) = match posting {
        policy::Posting::Pending => (api::Check::Pending, b"".as_slice(), Operation::Pending),
        policy::Posting::Ran { passed: true } => (api::Check::Passed, b" passed".as_slice(), Operation::Verdict),
        policy::Posting::Ran { passed: false } => (api::Check::Failed, b" failed".as_slice(), Operation::Verdict),
        policy::Posting::Received => (api::Check::Passed, b" passed by receipt".as_slice(), Operation::Verdict),
    };
    let link: Box<[u8]> = if posting == policy::Posting::Pending {
        Box::new([])
    } else {
        match receipt {
            Some(receipt) => receipt.run.url.clone(),
            None => Box::new([]),
        }
    };
    (
        api::Write::Status {
            commit: candidate,
            context: concat(&[b"temper/gate/", name]),
            check,
            description: concat(&[b"key ", &short, ending]),
            link,
        },
        operation,
    )
}
fn operation(op: Operation) -> u8 {
    match op {
        Operation::Dispatch => 1,
        Operation::Rerun => 2,
        Operation::Pending => 3,
        Operation::Verdict => 4,
        Operation::Merge => 5,
        Operation::Open => 6,
        Operation::Close => 7,
        Operation::Delete => 8,
        Operation::Record => 9,
    }
}
#[expect(clippy::too_many_arguments, reason = "one effect and its member/gate identity")]
fn keep(
    d: &mut Domain,
    row: &mut LandingRow,
    entry: u64,
    member: u16,
    gate: u16,
    op: Operation,
    write: api::Write,
    condition: skein_forge_client::Condition,
    out: &mut Queue<Request>,
) {
    let mut entries = List::with_capacity(d.landing.limits.entries);
    for old in &row.entries {
        entries.push(*old).expect("bounded entries");
    }
    entries.push(LandingEntry { entry, member, gate, operation: op }).expect("entry room checked");
    row.entries = entries.into_boxed();
    d.landing
        .results
        .insert((row.landing.task, member, gate, operation(op)), policy::Status::Pending)
        .expect("bounded effect positions");
    keep_entry(
        d,
        skein_forge_client::Entry {
            number: entry,
            task: row.landing.task,
            repository: repository(row.landing.members.get(usize::from(member)).expect("member")),
            effect: skein_forge_client::Effect { write, condition },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
        None,
        out,
    );
}

pub(crate) fn put_receipt(
    d: &mut Domain,
    env: &Env<Limits>,
    key: ReceiptKey,
    receipt: Receipt,
    out: &mut Queue<Request>,
) {
    let mut count = 0_u32;
    let mut oldest: Option<(Wall, ReceiptKey)> = None;
    let protected = match &key.check {
        Check::Gate(name) => d.landing.main_keys.get(&(key.repository, name.clone())).copied(),
        Check::Tier(name) => match d.tier_ops.tip_keys.get(&(key.repository, name.clone())) {
            Some(judged) => Some((judged.tip, judged.key)),
            None => None,
        },
    };
    for (saved, value) in &d.landing.receipts {
        if saved.project != key.project || saved.repository != key.repository || saved.check != key.check {
            continue;
        }
        count = count.checked_add(1).expect("receipt bound");
        if Some(saved.key)
            == match protected {
                Some((_, key)) => Some(key),
                None => None,
            }
            || saved == &key
        {
            continue;
        }
        if match &oldest {
            Some((ended, _)) => value.ended < *ended,
            None => true,
        } {
            oldest = Some((value.ended, saved.clone()));
        }
    }
    if !d.landing.receipts.contains_key(&key) && count >= env.limits.receipts_per_gate {
        if let Some((_, oldest)) = oldest {
            d.landing.receipts.remove(&oldest);
            mark_key(d, &oldest, out);
            emit(out, Request::Erase { key: Key::Receipt(oldest) });
        } else {
            return;
        }
    }
    d.landing.receipts.insert(key.clone(), receipt.clone()).expect("bounded receipts per gate");
    mark_key(d, &key, out);
    emit(out, Request::Save { record: Stored::Receipt { key, receipt }, read_afresh: None, release: None });
}
fn finish_flight(d: &mut Domain, key: &ReceiptKey, out: &mut Queue<Request>) {
    let Some(flight) = d.landing.flights.remove(key) else {
        return;
    };
    let Some(mut row) = d.landing.rows.get(&flight.task).cloned() else {
        return;
    };
    let mut entries = List::with_capacity(d.landing.limits.entries);
    for entry in &row.entries {
        let member = row.landing.members.get(usize::from(entry.member)).expect("member");
        let matches = if repository(member) == key.repository {
            match key.check {
                Check::Gate(ref name) => {
                    match d.landing.gates(row.landing.project, key.repository).get(usize::from(entry.gate)) {
                        Some(gate) => &gate.name == name,
                        None => false,
                    }
                }
                Check::Tier(_) => false,
            }
        } else {
            false
        };
        if matches && (entry.operation == Operation::Dispatch || entry.operation == Operation::Rerun) {
            if d.entries.remove(&entry.entry).is_some() {
                emit(out, Request::Erase { key: Key::Entry(entry.entry) });
            }
        } else {
            entries.push(*entry).expect("bounded entries");
        }
    }
    row.entries = entries.into_boxed();
    save(d, row, out);
    for (_, waiting) in &d.landing.rows {
        if waiting.landing.project == key.project {
            emit(out, Request::WakeProcedure { task: waiting.landing.task });
        }
    }
}
/// A turn's merge belongs to its member's change; its effect is the landing's
/// (domain/forge/README.md, 5.3; domain/landing.md, 8).
pub(crate) fn merged_owner(d: &Domain, task: u64, entry: u64) -> u64 {
    let Some(row) = d.landing.rows.get(&task) else {
        return task;
    };
    for item in &row.entries {
        if item.entry == entry {
            assert!(item.operation == Operation::Merge || item.operation == Operation::Record, "merge or record entry");
            return row.landing.members.get(usize::from(item.member)).expect("merge's saved member").change;
        }
    }
    unreachable!("landing merge has a saved member entry");
}

/// Preserve a settled dispatch entry until its run ends: its start window
/// is required to reconstruct a Dispatched proof after restart (landing.md, 15).
pub(crate) fn outcome(
    d: &mut Domain,
    entry: u64,
    outcome: skein_forge_client::Outcome,
    out: &mut Queue<Request>,
) -> bool {
    let mut found = None;
    for (task, row) in &d.landing.rows {
        for item in &row.entries {
            if item.entry == entry {
                found = Some((*task, *item));
            }
        }
    }
    let Some((task, item)) = found else {
        return false;
    };
    let (status, keep_window) = match outcome {
        skein_forge_client::Outcome::Made { .. } => (policy::Status::Passed, true),
        skein_forge_client::Outcome::Failed(_) => (policy::Status::Failed, true),
        skein_forge_client::Outcome::Uncertain | skein_forge_client::Outcome::Queued => {
            (policy::Status::Pending, false)
        }
        skein_forge_client::Outcome::Raced { .. }
        | skein_forge_client::Outcome::Held
        | skein_forge_client::Outcome::Withdrawn => (policy::Status::Failed, false),
    };
    d.landing
        .results
        .insert((task, item.member, item.gate, operation(item.operation)), status)
        .expect("bounded effect positions");
    if item.operation == Operation::Delete && status == policy::Status::Passed {
        let row = d.landing.rows.get(&task).expect("round deletion");
        let resource = skein_forge_client::Resource {
            repository: repository(row.landing.members.first().expect("member")),
            what: skein_forge_client::What::Branch(row.landing.round.branch.clone()),
        };
        if let Some(name) = super::domain::from_client(&resource, &d.landing.limits) {
            // The next round's new branch has no ancestry to the deleted candidate.
            if d.heads.remove(&name).is_some() {
                emit(out, Request::Erase { key: Key::BranchHead(name) });
            }
        }
    }
    let mut owned_key = None;
    for (key, flight) in &d.landing.flights {
        if flight.entry == entry {
            owned_key = Some(key.clone());
        }
    }
    if let Some(key) = owned_key {
        let flight = d.landing.flights.get_mut(&key).expect("owned flight");
        flight.waiting = status == policy::Status::Pending;
        if let Some(entry) = d.entries.get(&entry)
            && let Some(attempt) = entry.entry.attempt
        {
            flight.since = Some(attempt.sent);
        }
        if item.operation == Operation::Dispatch
            && let Some(entry) = d.entries.get(&entry)
        {
            flight.start = match entry.entry.start {
                Some(start) => Some(start.at),
                None => flight.start,
            };
        }
        if status == policy::Status::Failed {
            flight.outcome = Some(policy::Verdict::Fault(policy::Fault::Lost));
        }
    }
    if status != policy::Status::Pending && item.operation != Operation::Dispatch && item.operation != Operation::Rerun
    {
        let mut row = d.landing.rows.get(&task).expect("landing effect").clone();
        let mut entries = List::with_capacity(d.landing.limits.entries);
        for old in &row.entries {
            if old.entry != entry {
                entries.push(*old).expect("bounded entries");
            }
        }
        row.entries = entries.into_boxed();
        if item.operation == Operation::Record
            || item.operation == Operation::Open
            || item.operation == Operation::Merge
        {
            row.landing.round.failure = match outcome {
                skein_forge_client::Outcome::Failed(error) | skein_forge_client::Outcome::Raced { why: error, .. } => {
                    crate::effect_words::error_words(error)
                }
                skein_forge_client::Outcome::Made { .. }
                | skein_forge_client::Outcome::Queued
                | skein_forge_client::Outcome::Uncertain
                | skein_forge_client::Outcome::Held
                | skein_forge_client::Outcome::Withdrawn => Box::new([]),
            };
        }
        save(d, row, out);
    }
    emit(out, Request::WakeProcedure { task });
    keep_window && (item.operation == Operation::Dispatch || item.operation == Operation::Rerun)
}
pub(crate) fn hinted(d: &Domain, hint: &api::Hint, out: &mut Queue<Request>) {
    if let api::Change::Commit(commit) = hint.change {
        for (_, flight) in &d.landing.flights {
            if flight.repository == hint.repository && flight.commit == commit {
                emit(out, Request::WakeProcedure { task: flight.task });
            }
        }
    }
}
pub(crate) fn ask_for(d: &Domain, change: &crate::ChangeRow) -> Option<change::Ask> {
    let row = d.landing.rows.get(&change.change.landing?)?;
    for (index, member) in row.landing.members.iter().enumerate() {
        if member.change == change.task {
            if row.landing.round.branch.is_empty()
                && let Some(prefix) = member.landed
            {
                return Some(change::Ask::Landed { prefix });
            }
            if let Some(ask) = row.asks.get(index).expect("member ask").clone() {
                return Some(ask);
            }
            for (index, gate) in member.gates.iter().enumerate() {
                if let policy::Proof::Failed { .. } = gate.proof
                    && change.change.last_head == member.candidate
                {
                    return Some(change::Ask::Repair {
                        gate: u16::try_from(index).expect("gate position"),
                        candidate: member.candidate?,
                    });
                }
            }
        }
    }
    None
}
#[expect(clippy::match_like_matches_macro, reason = "the foundation forbids matches macros in step code")]
pub(crate) fn change_advanced(
    d: &mut Domain,
    row: &mut crate::ChangeRow,
    prior: &change::Change,
    decision: change::Decision,
    out: &mut Queue<Request>,
) {
    let Some(task) = row.change.landing else {
        return;
    };
    let Some(mut landing) = d.landing.rows.get(&task).cloned() else {
        return;
    };
    let settled = if let change::State::Preparing { .. } = prior.state {
        if let change::State::Preparing { .. } = row.change.state { false } else { true }
    } else {
        false
    };
    let repair = if let change::Decision::Delegate(change::Delegate::Repair(_)) = decision { true } else { false };
    let resolution = match decision {
        change::Decision::Delegate(change::Delegate::Resolve { .. } | change::Delegate::Merge { .. }) => true,
        change::Decision::None
        | change::Decision::Wait { .. }
        | change::Decision::Delegate(
            change::Delegate::Produce
            | change::Delegate::Repair(_)
            | change::Delegate::Gate { .. }
            | change::Delegate::Close,
        )
        | change::Decision::Effect(_)
        | change::Decision::CancelProducer
        | change::Decision::CancelGates { .. }
        | change::Decision::CancelRepair { .. }
        | change::Decision::CancelResolution { .. }
        | change::Decision::EndHold
        | change::Decision::Ready
        | change::Decision::RepairEnded { .. }
        | change::Decision::QueueRepair
        | change::Decision::NoChange
        | change::Decision::Finish { .. }
        | change::Decision::Cancel
        | change::Decision::Hold(_) => false,
    };
    if (resolution || repair)
        && (matches_terminal(&landing.landing.phase) || landing.landing.round.ending == Some(policy::RoundEnd::Leave))
    {
        row.change.landing = None;
    }
    for (index, member) in landing.landing.members.iter().enumerate() {
        if member.change == row.task
            && (settled || repair || resolution || (judging(&prior.state) && !judging(&row.change.state)))
        {
            *landing.asks.get_mut(index).expect("member ask") = None;
        }
    }
    if d.landing.rows.get(&task) != Some(&landing) {
        save(d, landing, out);
    }
    if prior != &row.change {
        emit(out, Request::WakeProcedure { task });
    }
}
pub(crate) fn rebuild(d: &mut Domain) {
    let mut flights = List::with_capacity(slots(&d.landing.limits).expect("flight bound"));
    for (_, row) in &d.landing.rows {
        for (index, member) in row.landing.members.iter().enumerate() {
            let repo = repository(member);
            for (number, proof) in member.gates.iter().enumerate() {
                let policy::Proof::Dispatched { key, try_ } = proof.proof else {
                    continue;
                };
                let Some(gate) = d.train_gates(row, repo).get(number) else {
                    continue;
                };
                let Some(candidate) = member.candidate else {
                    continue;
                };
                let mut newest = None;
                for entry in &row.entries {
                    if usize::from(entry.member) == index
                        && usize::from(entry.gate) == number
                        && entry.operation == Operation::Dispatch
                    {
                        newest = Some(entry.entry);
                    }
                }
                let Some(entry) = newest else {
                    continue;
                };
                let mut follow = runs::follow(
                    row.landing.task,
                    entry,
                    purpose_identity(
                        row,
                        u16::try_from(index).expect("member"),
                        u16::try_from(number).expect("gate"),
                        1,
                        try_,
                    ),
                    repo,
                    gate.workflow.clone(),
                    dispatch_branch(d, row, member),
                    candidate,
                    inputs(candidate, key, if gate.base { member.base } else { None }),
                );
                follow.writes = Box::new([]);
                follow.waiting = d.flights.contains_key(&(row.landing.task, entry));
                if let Some(entry) = d.entries.get(&entry)
                    && let Some(attempt) = entry.entry.attempt
                {
                    follow.since = Some(attempt.sent);
                }
                if let Some(entry) = d.entries.get(&entry) {
                    follow.start = match entry.entry.start {
                        Some(start) => Some(start.at),
                        None => None,
                    };
                }
                flights
                    .push((
                        gate_key(
                            row,
                            u16::try_from(index).expect("member"),
                            u16::try_from(number).expect("gate"),
                            &gate.name,
                            key,
                        ),
                        follow,
                    ))
                    .expect("bounded flights");
            }
        }
    }
    for (key, flight) in flights.into_boxed() {
        d.landing.flights.insert(key, flight).expect("bounded flights");
    }
}
fn hex(bytes: &[u8]) -> Box<[u8]> {
    let mut out =
        List::with_capacity(u32::try_from(bytes.len().checked_mul(2).expect("hex width")).expect("bounded hex"));
    for byte in bytes {
        for digit in crate::hex(*byte) {
            out.push(digit).expect("sized hex");
        }
    }
    out.into_boxed()
}
pub(crate) fn row_bytes(row: &LandingRow) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(row.landing.members.len())
        .checked_mul(u64::try_from(size_of::<policy::Member>()).ok()?)?
        .checked_add(
            skein_lib::len_u64(row.asks.len()).checked_mul(u64::try_from(size_of::<Option<change::Ask>>()).ok()?)?,
        )?
        .checked_add(
            skein_lib::len_u64(row.entries.len()).checked_mul(u64::try_from(size_of::<LandingEntry>()).ok()?)?,
        )?;
    for ask in &row.asks {
        if let Some(change::Ask::Judge(target)) = ask {
            bytes = bytes
                .checked_add(skein_lib::len_u64(target.branch.len()))?
                .checked_add(skein_lib::len_u64(target.base.len()))?;
        }
    }
    for member in &row.landing.members {
        bytes = bytes
            .checked_add(skein_lib::len_u64(member.branch.len()))?
            .checked_add(
                skein_lib::len_u64(member.gates.len()).checked_mul(u64::try_from(size_of::<policy::GateProof>()).ok()?)?,
            )?;
    }
    if let policy::Phase::Held { .. } = row.landing.phase {
        bytes = bytes.checked_add(u64::try_from(size_of::<policy::Phase>()).ok()?)?;
    }
    bytes
        .checked_add(skein_lib::len_u64(row.landing.branch.len()))?
        .checked_add(skein_lib::len_u64(row.landing.round.branch.len()))?
        .checked_add(skein_lib::len_u64(row.landing.round.failure.len()))
}
pub(crate) fn receipt_bytes(key: &ReceiptKey, receipt: &Receipt) -> Option<u64> {
    let name = match &key.check {
        Check::Gate(name) | Check::Tier(name) => name,
    };
    let mut held = skein_lib::len_u64(name.len())
        .checked_add(skein_lib::len_u64(receipt.run.url.len()))?
        .checked_add(skein_lib::len_u64(receipt.logs.len()))?;
    if let Some(failures) = &receipt.failures {
        held = held.checked_add(crate::tiers::failures_bytes(failures)?)?;
    }
    Some(held)
}
fn reset_results(d: &mut Domain, row: &LandingRow, prior: &LandingRow) {
    for (index, member) in row.landing.members.iter().enumerate() {
        let mut unchanged = false;
        for old in &prior.landing.members {
            if old.change == member.change && old.candidate == member.candidate {
                unchanged = true;
            }
        }
        if member.landed.is_some() || unchanged {
            continue;
        }
        let member = u16::try_from(index).expect("member");
        d.landing.results.remove(&(row.landing.task, member, 0, 5));
        for gate in 0..d.landing.limits.gates {
            d.landing.faults.remove(&(row.landing.task, member, u16::try_from(gate).expect("gate")));
            for operation in 1..=9 {
                d.landing.results.remove(&(row.landing.task, member, u16::try_from(gate).expect("gate"), operation));
            }
        }
    }
}
fn continue_retry(d: &mut Domain, row: &mut LandingRow, entry: u64, out: &mut Queue<Request>) -> bool {
    let mut pending = None;
    for (key, flight) in &d.landing.flights {
        if flight.task == row.landing.task && !flight.writes.is_empty() {
            pending = Some(key.clone());
            break;
        }
    }
    let Some(key) = pending else {
        return false;
    };
    let mut position = None;
    for (member, m) in row.landing.members.iter().enumerate() {
        if repository(m) != key.repository {
            continue;
        }
        for (gate, proof) in m.gates.iter().enumerate() {
            if let policy::Proof::Dispatched { key: proof, try_ } = proof.proof
                && proof == key.key
            {
                position = Some((u16::try_from(member).expect("member"), u16::try_from(gate).expect("gate"), try_));
            }
        }
    }
    let Some((member, gate, try_)) = position else {
        return false;
    };
    if result(d, row.landing.task, member, gate, 2) == policy::Status::Pending
        || d.entries.contains_key(&entry)
        || d.entries.len() == d.landing.limits.entries
    {
        emit(
            out,
            Request::LandingDecision {
                task: row.landing.task,
                decision: policy::Decision::Wait { until: None },
                entry: None,
                evidence: None,
            },
        );
        return true;
    }
    let flight = d.landing.flights.get_mut(&key).expect("pending reruns");
    if flight.waiting
        || match &flight.run {
            Some(run) => run.state != api::RunState::Completed,
            None => true,
        }
    {
        emit(
            out,
            Request::LandingDecision {
                task: row.landing.task,
                decision: policy::Decision::Wait { until: None },
                entry: None,
                evidence: None,
            },
        );
        return true;
    }
    let write = flight.write(entry).expect("next failed job");
    keep(d, row, entry, member, gate, Operation::Rerun, write, skein_forge_client::Condition::None, out);
    save(d, row.clone(), out);
    emit(
        out,
        Request::LandingDecision {
            task: row.landing.task,
            decision: policy::Decision::Retry { member, gate, key: key.key, try_ },
            entry: Some(entry),
            evidence: None,
        },
    );
    true
}

pub(crate) fn refresh(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut tasks = List::with_capacity(env.limits.landings);
    for (task, row) in &d.landing.rows {
        if row.landing.phase != policy::Phase::Landed && row.landing.phase != policy::Phase::Cancelled {
            tasks.push(*task).expect("bounded landings");
        }
    }
    for task in tasks.into_boxed() {
        step(
            d,
            env,
            task,
            0,
            LandingHeard {
                held: Box::new([]),
                failure: Box::new([]),
                preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
            out,
        );
    }
}

pub(crate) fn judge(
    d: &Domain,
    row: &LandingRow,
    criterion: crate::Criterion,
    head: api::Commit,
    evidence: &crate::ChangeEvidence,
    reviewers: &[crate::Reviewer],
) -> Option<crate::JudgeVerdict> {
    for (position, member) in row.landing.members.iter().enumerate() {
        if member.candidate != Some(head) {
            continue;
        }
        let repo = repository(member);
        if evidence.head != Some(head) {
            return Some(crate::JudgeVerdict::Wait);
        }
        if criterion == crate::Criterion::Ci {
            for (number, configured) in d.train_gates(row, repo).iter().enumerate() {
                if !configured.blocking {
                    continue;
                }
                let Some(key) = proof_key(member.gates.get(number).expect("gate proof").proof) else {
                    return Some(crate::JudgeVerdict::Wait);
                };
                let key =
                    gate_key(row, u16::try_from(position).ok()?, u16::try_from(number).ok()?, &configured.name, key);
                match d.landing.receipts.get(&key) {
                    Some(receipt) if receipt.passed => {}
                    Some(_) => return Some(crate::JudgeVerdict::Refuse),
                    None => return Some(crate::JudgeVerdict::Wait),
                }
            }
            return Some(crate::JudgeVerdict::Met);
        }
        let change = d.changes.get(&member.change)?;
        let mut clean =
            List::with_capacity(u32::try_from(change.change.clean.len()).expect("bounded clean").saturating_add(1));
        for head in &change.change.clean {
            clean.push(*head).expect("clean room");
        }
        if let Some(head) = member.head {
            clean.push(head).expect("head room");
        }
        return Some(crate::judge::judge(
            criterion,
            d.repositories.get(&repo)?,
            head,
            clean.as_slice(),
            evidence,
            reviewers,
        ));
    }
    None
}

/// An active key's owner takes precedence over a previous owner's fault.
pub(crate) fn flight_fact(state: &State, key: &ReceiptKey, position: Position) -> policy::Flight {
    match state.flights.get(key) {
        Some(flight) if flight.task == position.0 => policy::Flight::Mine { run: flight.outcome },
        Some(_) => policy::Flight::Other,
        None => match state.faults.get(&position) {
            Some(fault) => policy::Flight::Mine { run: fault.outcome },
            None => policy::Flight::None,
        },
    }
}

/// Terminal rows with no effects left surrender their live slot on admission
/// (domain/landing.md, 12); receipts have their independent retention.
fn retire_finished(d: &mut Domain, out: &mut Queue<Request>) -> bool {
    let mut finished = None;
    for (task, row) in &d.landing.rows {
        if !row.entries.is_empty() {
            continue;
        }
        match row.landing.phase {
            policy::Phase::Landed | policy::Phase::Cancelled => {
                finished = Some(*task);
                break;
            }
            policy::Phase::Waiting
            | policy::Phase::Preparing
            | policy::Phase::Proving
            | policy::Phase::Proved { .. }
            | policy::Phase::Turn { .. }
            | policy::Phase::Held { .. } => {}
        }
    }
    let Some(task) = finished else {
        return false;
    };
    let retired = d.landing.rows.remove(&task).expect("selected finished landing");
    emit(
        out,
        Request::Marked {
            tasks: Box::new([
                task,
                crate::landings_subject(retired.landing.project, false),
                crate::landings_subject(retired.landing.project, true),
            ]),
        },
    );
    d.landing.due.remove(&task);
    for member in 0..d.landing.limits.landing_members {
        d.landing.results.remove(&(task, u16::try_from(member).expect("member position"), 0, 5));
        for gate in 0..d.landing.limits.gates {
            let member = u16::try_from(member).expect("admitted member position");
            let gate = u16::try_from(gate).expect("admitted gate position");
            d.landing.keys.remove(&(task, member, gate));
            d.landing.faults.remove(&(task, member, gate));
            for operation in 1..=9 {
                d.landing.results.remove(&(task, member, gate, operation));
            }
        }
    }
    emit(out, Request::Erase { key: Key::Landing(task) });
    true
}

fn matches_terminal(phase: &policy::Phase) -> bool {
    match phase {
        policy::Phase::Landed | policy::Phase::Cancelled => true,
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
        | policy::Phase::Held { .. } => false,
    }
}

impl Domain {
    #[must_use]
    pub fn landing_hold_words(&self, task: u64) -> Option<Box<[u8]>> {
        let row = self.landing.rows.get(&task)?;
        let policy::Phase::Held { why, .. } = row.landing.phase else { return None };
        match why {
            policy::Hold::Prepare => Some(concat(&[b"prepare job: ", &row.landing.round.failure])),
            policy::Hold::Record { member } => Some(concat(&[
                b"turn: record of #",
                skein_lib::Decimal::of(row.landing.members.get(usize::from(member))?.pull).as_bytes(),
                b": ",
                &row.landing.round.failure,
            ])),
            policy::Hold::Turn => Some(concat(&[b"turn: ", &row.landing.round.failure])),
            policy::Hold::Faults { member, gate } | policy::Hold::Timeout { member, gate, .. } => {
                let m = row.landing.members.get(usize::from(member))?;
                let repo = self.repository(repository(m))?;
                let gate = self.train_gates(row, repository(m)).get(usize::from(gate))?;
                let mut words = List::with_capacity(16384);
                for part in [b"gate ".as_slice(), &gate.name, b" of ", &repo.owner, b"/", &repo.name] {
                    append_words(&mut words, part);
                }
                match why {
                    policy::Hold::Faults { .. } => append_words(&mut words, b" failed on its runner past its retries"),
                    policy::Hold::Timeout { run, .. } => {
                        append_words(&mut words, b": run ");
                        append_words(&mut words, skein_lib::Decimal::of(run).as_bytes());
                        append_words(&mut words, b" running past its bound");
                    }
                    policy::Hold::Prepare | policy::Hold::Record { .. } | policy::Hold::Turn => {
                        unreachable!("gate hold")
                    }
                }
                Some(words.into_boxed())
            }
        }
    }
    /// The one source of a member's leaving words (domain/landing.md, 4.3).
    #[must_use]
    pub fn landing_leave_words(&self, task: u64, member: u16) -> Option<Box<[u8]>> {
        let row = self.landing.rows.get(&task)?;
        let m = row.landing.members.get(usize::from(member))?;
        Some(match m.left? {
            policy::Leave::Conflict => concat(&[b"conflict with the branch: ", &row.landing.round.failure]),
            policy::Leave::EarlierConflict => concat(&[b"conflict with an earlier member: ", &row.landing.round.failure]),
            policy::Leave::Unit { change } => concat(&[b"change ", skein_lib::Decimal::of(change).as_bytes(), b" of its unit left"]),
            policy::Leave::Dependency { change } => concat(&[b"its dependency, change ", skein_lib::Decimal::of(change).as_bytes(), b", left"]),
            policy::Leave::Fallback { units, top } => concat(&[b"red round of ", skein_lib::Decimal::of(u64::from(units)).as_bytes(), b" units at ", &commit_hex(top), b": landing one at a time"]),
            policy::Leave::HeadMoved { head } => concat(&[b"head moved to ", &commit_hex(head)]),
            policy::Leave::Cancelled => Box::from(b"cancelled".as_slice()),
            policy::Leave::Held => Box::from(b"held".as_slice()),
            policy::Leave::Refused { gate } => {
                concat(&[b"gate ", skein_lib::Decimal::of(gate).as_bytes(), b" refused"])
            }
            policy::Leave::Failed { gate, run } => concat(&[
                b"gate ",
                &self.train_gates(row, repository(m)).get(usize::from(gate))?.name,
                b" failed in ",
                skein_lib::Decimal::of(run).as_bytes(),
            ]),
        })
    }
    /// The prefix refused by this gate, while its change repairs at its own head.
    #[must_use]
    pub fn landing_refused_at(&self, task: u64, gate: u64) -> Option<api::Commit> {
        let mut prefix = None;
        for (_, row) in &self.landing.rows {
            for member in &row.landing.members {
                if member.change == task && member.left == Some(policy::Leave::Refused { gate }) {
                    prefix = member.candidate;
                }
            }
        }
        prefix
    }
    /// The candidate pins its failure snapshot even after the landing's live row retires.
    #[must_use]
    pub fn candidate_failure_words(&self, change: u64, gate: u16, candidate: api::Commit) -> Option<Box<[u8]>> {
        let change = self.change(change)?;
        let project = self.repository(change.repository)?.project;
        let name = &self.landing.gates(project, change.repository).get(usize::from(gate))?.name;
        for (key, receipt) in &self.landing.receipts {
            if key.project == project
                && key.repository == change.repository
                && key.check == Check::Gate(name.clone())
                && receipt.candidate == candidate
                && !receipt.passed
            {
                return Some(receipt.logs.clone());
            }
        }
        None
    }
    /// The failed jobs are read before the receipt is committed; repairs use that snapshot.
    #[must_use]
    pub fn landing_failure_words(&self, change: u64, gate: u16) -> Option<Box<[u8]>> {
        let mut words = None;
        for (_, row) in &self.landing.rows {
            for member in &row.landing.members {
                if member.change == change && matches_failed(member.left, gate) {
                    words = Some(row.landing.round.failure.clone());
                }
            }
        }
        words
    }
    /// All five news lines are formed from the committed landing (domain/landing.md, 4.2).
    #[must_use]
    pub fn landing_news_words(&self, task: u64, step: crate::LandingNews) -> Option<(u64, Box<[u8]>)> {
        let row = self.landing.rows.get(&task)?;
        let member = row.landing.members.first()?;
        let mut words = List::with_capacity(16384);
        append_words(&mut words, b"landing ");
        append_words(&mut words, skein_lib::Decimal::of(task).as_bytes());
        append_words(&mut words, b": ");
        if step != crate::LandingNews::Held {
            append_words(&mut words, b"change ");
            append_words(&mut words, skein_lib::Decimal::of(member.change).as_bytes());
            append_words(&mut words, b" ");
        }
        match step {
            crate::LandingNews::Joined => {
                let repo = self.repository(repository(member))?;
                for part in [
                    b"joined, into ".as_slice(),
                    &repo.owner,
                    b"/",
                    &repo.name,
                    b" ",
                    &row.landing.branch,
                    b", round ",
                    skein_lib::Decimal::of(u64::from(row.landing.preparation.max(1))).as_bytes(),
                ] {
                    append_words(&mut words, part);
                }
            }
            crate::LandingNews::Proving => {
                for part in
                    [b"proving at ".as_slice(), &commit_hex(member.candidate?), b" on ", &commit_hex(member.base?)]
                {
                    append_words(&mut words, part);
                }
            }
            crate::LandingNews::Landed => {
                append_words(&mut words, b"landed at ");
                append_words(&mut words, &commit_hex(member.landed?));
            }
            crate::LandingNews::Left => {
                append_words(&mut words, b"left: ");
                append_words(&mut words, &self.landing_leave_words(task, 0)?);
            }
            crate::LandingNews::Held => {
                append_words(&mut words, b"held: ");
                append_words(&mut words, &self.landing_hold_words(task)?);
            }
        }
        Some((member.change, words.into_boxed()))
    }
    /// Describe the exact saved landing write before core authorization.
    #[must_use]
    pub fn describe_landing_effect(
        &self,
        task: u64,
        entry: u64,
    ) -> Option<(api::Repository, crate::What, crate::AgentEffect)> {
        let row = self.landing.rows.get(&task)?;
        let mut item = None;
        for candidate in &row.entries {
            if candidate.entry == entry {
                item = Some(*candidate);
            }
        }
        let item = item?;
        let member = row.landing.members.get(usize::from(item.member))?;
        let kept = &self.entries.get(&entry)?.entry;
        if kept.task != task {
            return None;
        }
        let (kind, what, state) = match &kept.effect.write {
            api::Write::Dispatch { .. } => (16, crate::What::Repository, member.candidate?),
            api::Write::Rerun { .. } => (17, crate::What::Repository, member.candidate?),
            api::Write::Status { commit, .. } => (6, crate::What::Repository, *commit),
            api::Write::Merge { head, how, .. } => {
                let change = self.changes.get(&member.change)?;
                let mut parts = List::with_capacity(self.landing.limits.authority_segments);
                let mut start = 0;
                for (index, byte) in change.base.iter().enumerate() {
                    if *byte == b'/' {
                        parts.push(Box::from(change.base.get(start..index)?)).ok()?;
                        start = index.checked_add(1)?;
                    }
                }
                parts.push(Box::from(change.base.get(start..)?)).ok()?;
                (
                    4,
                    crate::What::Branch(parts.into_boxed()),
                    match how {
                        api::MergeHow::Recorded { commit } => *commit,
                        api::MergeHow::FastForwardOnly | api::MergeHow::Style => *head,
                    },
                )
            }
            api::Write::OpenPull { .. } => (3, crate::What::Repository, member.candidate?),
            api::Write::Close { number } => (12, crate::What::Pull(*number), member.candidate.unwrap_or([0; 32])),
            api::Write::DeleteBranch { branch } => {
                let resource = skein_forge_client::Resource {
                    repository: kept.repository,
                    what: skein_forge_client::What::Branch(branch.clone()),
                };
                (
                    15,
                    super::domain::from_client(&resource, &self.landing.limits)?.what,
                    member.candidate.unwrap_or([0; 32]),
                )
            }
            api::Write::CreateIssue { .. }
            | api::Write::Post { .. }
            | api::Write::Review { .. }
            | api::Write::Edit { .. }
            | api::Write::SetReviewers { .. }
            | api::Write::Reopen { .. }
            | api::Write::Update { .. }
            | api::Write::CreateBranch { .. } => return None,
        };
        let (form, recovery) = crate::effect_shape(kind)?;
        Some((kept.repository, what, crate::AgentEffect { effect: kept.effect.clone(), kind, state, form, recovery }))
    }
}
/// A refused entry never reaches the client and is observed as failed next step.
pub(crate) fn veto(d: &mut Domain, task: u64, entry: u64, out: &mut Queue<Request>) {
    if !matches_entry(d, task, entry) {
        return;
    }
    outcome(d, entry, skein_forge_client::Outcome::Failed(api::Error::Refused), out);
    d.entries.remove(&entry);
    emit(out, Request::Erase { key: Key::Entry(entry) });
    if let Some(mut row) = d.landing.rows.get(&task).cloned() {
        let mut entries = List::with_capacity(d.landing.limits.entries);
        for item in &row.entries {
            if item.entry != entry {
                entries.push(*item).expect("bounded entries");
            }
        }
        row.entries = entries.into_boxed();
        save(d, row, out);
    }
}
fn matches_entry(d: &Domain, task: u64, entry: u64) -> bool {
    match d.entries.get(&entry) {
        Some(row) => row.entry.task == task && d.landing.rows.contains_key(&task),
        None => false,
    }
}

/// Cancellation removes a member; cancelling a landing leaves its changes live.
pub(crate) fn closing(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    ending: crate::ReleaseEnding,
    out: &mut Queue<Request>,
) {
    if ending != crate::ReleaseEnding::Cancelled {
        return;
    }
    if let Some(mut row) = d.landing.rows.get(&task).cloned() {
        let tell_landed = !row.landing.round.told
            && match row.landing.members.first() {
                Some(member) => member.landed.is_some(),
                None => false,
            };
        row.landing.round.told |= tell_landed;
        row.landing.phase = policy::Phase::Cancelled;
        d.landing.due.remove(&task);
        for member in &row.landing.members {
            if let Some(change) = d.changes.get_mut(&member.change) {
                change.change.landing = None;
                if let Some(merge) = member.landed {
                    change.change.state = change::State::Landed { merge, since: env.wall };
                } else if let change::State::Judging { head, ready_since, .. } = change.change.state {
                    change.change.state = change::State::Ready { head, ready_since };
                }
                emit(
                    out,
                    Request::Save {
                        record: Stored::Change(Box::new(change.clone())),
                        release: None,
                        read_afresh: None,
                    },
                );
                emit(out, Request::WakeProcedure { task: member.change });
            }
        }
        let project = row.landing.project;
        save(d, row, out);
        if tell_landed {
            emit(out, Request::LandingNews { task, step: crate::LandingNews::Landed });
        }
        wake_project_ready(d, project, out);
    } else if let Some(change) = d.changes.get(&task)
        && let Some(landing) = change.change.landing
    {
        let Some(mut row) = d.landing.rows.get(&landing).cloned() else { return };
        if matches_terminal(&row.landing.phase) {
            return;
        }
        let prior = row.clone();
        row.landing = policy::leave(row.landing, task);
        remap_members(d, &mut row, &prior);
        let change = d.changes.get_mut(&task).expect("cancelled member");
        change.change.landing = None;
        emit(out, Request::Save { record: Stored::Change(Box::new(change.clone())), release: None, read_afresh: None });
        save(d, row, out);
        emit(out, Request::WakeProcedure { task: landing });
    }
}

fn wake_project_ready(d: &Domain, project: u32, out: &mut Queue<Request>) {
    for (task, row) in &d.changes {
        if row.change.landing.is_none()
            && let change::State::Ready { .. } = row.change.state
            && let Some(repository) = d.repositories.get(&row.repository)
            && repository.project == project
        {
            emit(out, Request::WakeProcedure { task: *task });
        }
    }
}

/// Receipt and run observations remake every landing reading this key.
fn mark_key(d: &Domain, key: &ReceiptKey, out: &mut Queue<Request>) {
    let mut tasks = List::with_capacity(crate::MARKED);
    for (task, row) in &d.landing.rows {
        if row.landing.project != key.project {
            continue;
        }
        let mut reads = false;
        for member in &row.landing.members {
            if repository(member) != key.repository {
                continue;
            }
            for (index, proof) in member.gates.iter().enumerate() {
                if proof_key(proof.proof) == Some(key.key)
                    && let Some(gate) = d.landing.gates(key.project, key.repository).get(index)
                    && key.check == Check::Gate(gate.name.clone())
                {
                    reads = true;
                }
            }
        }
        if reads {
            tasks.push(*task).expect("mark room");
            if tasks.room() == 0 {
                emit(out, Request::Marked { tasks: tasks.into_boxed() });
                tasks = List::with_capacity(crate::MARKED);
            }
        }
    }
    if !tasks.is_empty() {
        emit(out, Request::Marked { tasks: tasks.into_boxed() });
    }
}

fn refuse_entry(
    d: &Domain,
    env: &Env<Limits>,
    pending: &Pending,
    row: &LandingRow,
    member: u16,
    out: &mut Queue<Request>,
) {
    let repo = repository(row.landing.members.get(usize::from(member)).expect("member"));
    if d.entries.contains_key(&pending.entry) {
        crate::refusal::read(
            out,
            pending.task,
            b"reserve landing effect",
            repo,
            &api::Read::Settings,
            api::Error::Exists,
        );
    } else {
        crate::refusal::limit(
            out,
            pending.task,
            repo,
            "forge.entries",
            env.limits.entries,
            u64::from(d.entries.len()).saturating_add(1),
        );
    }
}

#[expect(clippy::match_like_matches_macro, reason = "explicit state observation")]
fn judging(state: &change::State) -> bool {
    if let change::State::Judging { .. } = state { true } else { false }
}

fn matches_failed(left: Option<policy::Leave>, gate: u16) -> bool {
    if let Some(policy::Leave::Failed { gate: failed, .. }) = left { gate == failed } else { false }
}

/// Snapshot each failed job before publishing the failed receipt (domain/landing.md, 7.3).
#[expect(clippy::too_many_arguments, reason = "one coherent failed run and its bounded log cursor")]
fn failed_jobs(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    key: ReceiptKey,
    run: api::Run,
    next: u32,
    words: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let mut log = List::with_capacity(8192);
    append_words(&mut log, &words);
    let mut row = d.landing.rows.get(&p.task).expect("failed landing").clone();
    if next == 0 {
        let member = row.landing.members.first().expect("member");
        let name = match &key.check {
            Check::Gate(name) | Check::Tier(name) => name,
        };
        for part in [
            b"Gate: ".as_slice(),
            name,
            b"\nRun: ",
            &run.url,
            b"\nCandidate: ",
            &commit_hex(run.commit),
            b"\nBase: ",
            &commit_hex(member.base.expect("base")),
            b"\n",
        ] {
            append_words(&mut log, part);
        }
    }
    let total = u32::try_from(run.jobs.len()).expect("bounded jobs");
    for index in next..total {
        let job = run.jobs.get(skein_lib::usize_of(index)).expect("job");
        if job.conclusion != Some(api::Conclusion::Failure) {
            continue;
        }
        append_words(&mut log, b"Failed job: ");
        append_words(&mut log, &job.name);
        append_words(&mut log, b"\n");
        let attempt = api::JobAttempt { head: run.commit, run: run.id, job: job.id, attempt: job.attempt };
        let share = log.room().checked_div(total.saturating_sub(index).max(1)).expect("positive job count");
        let read = api::Read::Job {
            part: api::LogPart::End,
            attempt,
            max_bytes: share.min(env.limits.client.answer_bytes).max(1),
        };
        p.stage = Stage::FailedJobs {
            key: key.clone(),
            run,
            next: index.checked_add(1).expect("bounded jobs"),
            words: log.into_boxed(),
        };
        ask(d, env, owner, p, key.repository, read, out);
        return;
    }
    row.landing.round.failure = log.into_boxed();
    let row_logs = row.landing.round.failure.clone();
    save(d, row, out);
    let receipt = Receipt {
        logs: row_logs,
        failures: None,
        passed: false,
        candidate: run.commit,
        ended: env.wall,
        run: ReceiptRun { id: run.id, number: run.number, url: run.url },
    };
    put_receipt(d, env, key.clone(), receipt, out);
    finish_flight(d, &key, out);
    collect(d, env, owner, p, out);
}

fn failed_candidate(d: &Domain, row: &LandingRow, member: u16, gate: u16) -> api::Commit {
    let m = row.landing.members.get(usize::from(member)).expect("member");
    if let Some(key) = proof_key(m.gates.get(usize::from(gate)).expect("gate").proof) {
        let configured = d.train_gates(row, repository(m)).get(usize::from(gate)).expect("configured gate");
        let key = gate_key(row, member, gate, &configured.name, key);
        if let Some(receipt) = d.landing.receipts.get(&key) {
            return receipt.candidate;
        }
    }
    m.candidate.expect("candidate")
}
