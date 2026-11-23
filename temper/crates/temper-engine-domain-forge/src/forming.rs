//! Sealed, indivisible units selected in queue order (domain/landing.md, 4.1, domain/landing-turn.md, 2).
use crate::{ChangeDependencies, Domain};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_change as change;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct WaitingUnit {
    /// The first queue candidate, which owns a refusal hold.
    pub task: u64,
    pub changes: Box<[u64]>,
    pub refused: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Selection {
    /// Positions in the supplied units, in their queue order.
    pub take: Box<[u32]>,
    pub hold: Box<[u32]>,
}
/// Take a prefix of admitted units, passing refusals, never splitting a unit.
/// The first may pass batch, but no unit or landing may pass `landing_members`.
#[must_use]
pub fn select_units(units: &[WaitingUnit], batch: u16, landing_members: u32) -> Selection {
    let capacity = u32::try_from(units.len()).expect("bounded waiting units");
    let mut take = List::with_capacity(capacity);
    let mut hold = List::with_capacity(capacity);
    let mut count = 0_u32;
    for (index, unit) in units.iter().enumerate() {
        let size = u32::try_from(unit.changes.len()).expect("bounded unit");
        let index = u32::try_from(index).expect("bounded units");
        if unit.refused || size > landing_members {
            hold.push(index).expect("one position per unit");
            continue;
        }
        let next = count.saturating_add(size);
        if count != 0 && (next > u32::from(batch) || next > landing_members) {
            break;
        }
        take.push(index).expect("one position per unit");
        count = next;
        if count >= u32::from(batch) {
            break;
        }
    }
    Selection { take: take.into_boxed(), hold: hold.into_boxed() }
}
impl Domain {
    /// Waiting closures ordered by their first queue candidate. Every member must wait;
    /// task holds and pending task work are supplied by the root, not stored here.
    #[must_use]
    pub fn waiting_units(
        &self,
        repository: skein_forge::api::Repository,
        branch: &[u8],
        kept: &[u64],
        graph: &[ChangeDependencies],
        now: Wall,
    ) -> Box<[WaitingUnit]> {
        let mut ready = List::from_boxed(self.waiting_changes(repository, branch));
        let mut seen = List::with_capacity(self.landing.limits.changes);
        let mut units = List::with_capacity(self.landing.limits.changes);
        let Some(repo) = self.repository(repository) else { return units.into_boxed() };
        while let Some(task) = change::first(ready.as_slice(), self.landing.limits.queue_window, now) {
            let mut remaining = List::with_capacity(self.landing.limits.changes);
            for candidate in &ready {
                if candidate.task != task {
                    remaining.push(*candidate).expect("bounded changes");
                }
            }
            ready = remaining;
            if seen.as_slice().contains(&task) {
                continue;
            }
            let joined = self.joined_changes(repo.project, task, graph);
            let mut eligible = true;
            let mut refused = false;
            for member in &joined {
                seen.push(*member).expect("disjoint closures");
                let Some(row) = self.change(*member) else {
                    eligible = false;
                    continue;
                };
                eligible &= super::landing::waiting(row) && !kept.contains(member);
                refused |= row.repository != repository;
            }
            if eligible {
                units.push(WaitingUnit { task, changes: joined, refused }).expect("bounded units");
            }
        }
        units.into_boxed()
    }
    /// The forming decision for a free train with room for a landing.
    #[must_use]
    pub fn select_waiting(
        &self,
        repository: skein_forge::api::Repository,
        branch: &[u8],
        units: &[WaitingUnit],
    ) -> Selection {
        let selection = select_units(units, self.train_batch(repository, branch), self.landing.limits.landing_members);
        if self.landing.limits.landing_members == 0 || !self.train_free(repository, branch) {
            return Selection { take: Box::new([]), hold: Box::new([]) };
        }
        let Some(index) = selection.take.first() else { return selection };
        let Some(unit) = units.get(skein_lib::usize_of(*index)) else { return selection };
        let Some(first) = unit.changes.first() else { return selection };
        let Some(repo) = self.repository(repository) else {
            return Selection { take: Box::new([]), hold: Box::new([]) };
        };
        if self.landing_admits(repo.project, &[*first]).is_err() {
            return Selection { take: Box::new([]), hold: Box::new([]) };
        }
        selection
    }

    /// Task dependencies used to seal a train's units (domain/landing.md, 4.1).
    #[must_use]
    pub fn forming_dependencies(
        &self,
        tasks: &jig_domain_tasks::Domain,
        project: u32,
        limit: u32,
    ) -> Box<[ChangeDependencies]> {
        let mut graph = List::with_capacity(limit);
        for repository in self.writable_repositories(project) {
            for change in self.changes_for(repository.provider, limit) {
                if let Some(row) = tasks.task(change) {
                    graph
                        .push(ChangeDependencies { change, pending: row.waiting_on.clone() })
                        .expect("bounded changes");
                }
            }
        }
        graph.into_boxed()
    }

    /// A ready member already recorded by its train needs its ending step.
    #[must_use]
    pub fn landing_step_due(&self, task: u64) -> bool {
        let Some(change) = self.change(task) else { return false };
        if let change::State::Ready { .. } = change.change.state
            && let Some(task) = change.change.landing
            && let Some(row) = self.landing(task)
        {
            for member in &row.landing.members {
                if member.change == change.task {
                    return member.landed.is_some();
                }
            }
        }
        false
    }

    /// Form a member from the connector's adoption and gate facts (domain/landing.md, 4.2).
    #[must_use]
    pub fn forming_member(&self, task: u64, gate_limit: u32) -> Option<temper_engine_domain_forge_landing::Member> {
        use temper_engine_domain_forge_landing as landing;
        let row = self.change(task)?;
        let project = self.repository(row.repository)?.project;
        let mut gates = List::with_capacity(gate_limit);
        if let Some(repository) = self.repository_gates(project, row.repository)
            && self.repository(row.repository)?.landing == row.base
        {
            for _ in &repository.gates {
                gates
                    .push(landing::GateProof {
                        proof: landing::Proof::Unkeyed,
                        posted: landing::Posted::Nothing,
                        faults: 0,
                    })
                    .expect("configured gates");
            }
        }
        Some(landing::Member {
            round: None,
            unit: 0,
            branch: Box::new([]),
            combined: None,
            recorded: false,
            told: false,
            judging: false,
            leave_told: false,
            left: None,
            change: task,
            repository: landing::Repository { forge: row.repository.forge, repository: row.repository.repository },
            // A producing member has no pull yet; the top fills it before reading.
            pull: row.pull.unwrap_or(0),
            head: None,
            base: None,
            candidate: None,
            gates: gates.into_boxed(),
            asked: None,
            landed: None,
        })
    }

    /// The policy owns gate retries and merge outcomes; other terminal writes need an account.
    #[must_use]
    pub fn landing_effect_needs_hold(&self, task: u64, entry: u64) -> bool {
        let Some(row) = self.landing(task) else { return false };
        for pending in &row.entries {
            if pending.entry != entry {
                continue;
            }
            match pending.operation {
                crate::Operation::Dispatch | crate::Operation::Rerun | crate::Operation::Merge => return false,
                crate::Operation::Pending
                | crate::Operation::Verdict
                | crate::Operation::Open
                | crate::Operation::Close
                | crate::Operation::Delete
                | crate::Operation::Record => {}
            }
        }
        true
    }

    /// A landing ends for its train alone (domain/landing.md, 4.2, and domain/landing-turn.md, 2).
    #[must_use]
    pub fn waiting_after_landing(&self, task: u64, limit: u32) -> Box<[u64]> {
        let mut waiting = List::with_capacity(limit);
        let Some(landing) = self.landing(task) else { return waiting.into_boxed() };
        let Some(member) = landing.landing.members.first() else { return waiting.into_boxed() };
        let repository =
            skein_forge::api::Repository { forge: member.repository.forge, repository: member.repository.repository };
        for change in self.changes_for(repository, limit) {
            if let Some(row) = self.change(change)
                && row.change.landing.is_none()
                && let change::State::Ready { .. } = row.change.state
                && row.base == landing.landing.branch
            {
                waiting.push(change).expect("bounded train followers");
            }
        }
        waiting.into_boxed()
    }

    /// Aggregate report: owner/name, a space, then the landed commit in hex, one line each.
    #[must_use]
    pub fn landing_result_words(&self, task: u64, limit: u32) -> Box<[u8]> {
        use crate::landing::append_words as append;
        let mut words = List::with_capacity(limit);
        for (index, member) in self.landing(task).expect("finished landing").landing.members.iter().enumerate() {
            let provider = skein_forge::api::Repository {
                forge: member.repository.forge,
                repository: member.repository.repository,
            };
            if let Some(repository) = self.repository(provider) {
                append(&mut words, &repository.owner);
                append(&mut words, b"/");
                append(&mut words, &repository.name);
            }
            append(&mut words, b" ");
            let Some(commit) = member.landed else {
                append(&mut words, b"change ");
                append(&mut words, skein_lib::Decimal::of(member.change).as_bytes());
                append(&mut words, b" left: ");
                if let Some(why) = self.landing_leave_words(task, u16::try_from(index).expect("member position")) {
                    append(&mut words, &why);
                }
                append(&mut words, b"\n");
                continue;
            };
            let width = if commit.get(20..).expect("commit suffix") == [0; 12] { 20 } else { 32 };
            for byte in commit.get(..width).expect("commit width") {
                append(&mut words, &crate::hex(*byte));
            }
            append(&mut words, b"\n");
        }
        words.into_boxed()
    }
}
