//! The change's own `.temper/` check (temper/docs/design/domain/forge/temper-dir.md, 4–5).
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, State};
use skein_lib::{Decimal, Env, Overflow, Queue, Token, Writer};
use temper_engine_domain_forge_change as change;
use temper_protocol_temper_dir::{Home, Reading, Touch, Verdict};

use super::{PendingStep, StepStage, finish_step, gather_read, step_evidence};
use crate::{ChangeRow, Criterion, Domain, Limits, Repository, Request, Reviewer, Role, TemperDirEvidence, judge};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Pair {
    head: Commit,
    base: Commit,
}

impl Pair {
    fn at(step: &PendingStep) -> Option<Self> {
        let base = step.base_read?;
        if step.base_tip != Some(base) {
            return None;
        }
        Some(Self { head: step.branch?, base })
    }
}

#[derive(Hash, Debug)]
pub(super) struct Pass {
    pair: Pair,
    owner_short: bool,
}

#[derive(Hash, Debug)]
pub(crate) struct Check {
    pub(super) touch: Touch,
    fact: change::TemperDir,
    tree: Option<Commit>,
    pub(super) fault: Option<Box<[u8]>>,
    reading: Option<Box<TreeRead>>,
}

#[derive(Hash, Debug)]
struct TreeRead {
    head: Reading,
    base: Option<Reading>,
    covered: bool,
}

impl Check {
    pub(crate) const fn new() -> Self {
        Self { touch: Touch::Unknown, fact: change::TemperDir::Pending, tree: None, fault: None, reading: None }
    }
}

/// A projected landing head can establish coverage for a `.temper/`-only change;
/// a partial comparison can also use the method at the base tip.
pub(crate) fn covered(touch: Touch, head_method: bool, base_method: bool) -> bool {
    match touch {
        Touch::Method => true,
        Touch::Temper => head_method,
        Touch::Unknown => head_method || base_method,
        Touch::None => false,
    }
}

fn eligible(d: &Domain, row: &ChangeRow, repository: &Repository) -> bool {
    repository.role != Role::Context && repository.kinds.push && row.base == repository.landing
        // Already joined members use the turn's candidate, not this check of their own head.
        && !(row.change.way == change::Way::Turn && d.landing_of(row.task).is_some())
}

pub(super) fn should_compare(d: &Domain, row: &ChangeRow, repository: &Repository, step: &PendingStep) -> bool {
    if !eligible(d, row, repository)
        || step.heard.cancelled
        || step.contains_head == change::Status::Passed
        || Pair::at(step).is_none()
        || super::matches_updating(&row.change.state)
    {
        return false;
    }
    if let Some(pull) = &step.pull { pull.state == State::Open && pull.merged.is_none() } else { false }
}

pub(super) fn unread(d: &Domain, row: &ChangeRow) -> TemperDirEvidence {
    match d.repository(row.repository) {
        Some(repository) if eligible(d, row, repository) => TemperDirEvidence::Pending,
        Some(_) | None => TemperDirEvidence::Uncovered,
    }
}

pub(super) fn fact(d: &Domain, row: &ChangeRow, step: &PendingStep) -> change::TemperDir {
    if unread(d, row) == TemperDirEvidence::Uncovered { change::TemperDir::Uncovered } else { step.temper_dir.fact }
}

pub(super) fn evidence(d: &Domain, row: &ChangeRow, step: &PendingStep) -> TemperDirEvidence {
    if fact(d, row, step) == change::TemperDir::Uncovered {
        TemperDirEvidence::Uncovered
    } else if let Some(tree) = step.temper_dir.tree {
        TemperDirEvidence::Passed { tree }
    } else {
        TemperDirEvidence::Pending
    }
}

pub(super) fn after_paths(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: PendingStep,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.changes.get(&pending.task) else { return };
    let Some(repository) = d.repository(row.repository) else { return };
    if pending.woken || !should_compare(d, row, repository, &pending) {
        finish_step(d, env, pending, out);
        return;
    }
    let pair = Pair::at(&pending).expect("comparison has endpoints");
    if pending.temper_dir.touch == Touch::None {
        pending.temper_dir.fact = change::TemperDir::Uncovered;
        finish_step(d, env, pending, out);
    } else if let Some(pass) = d.temper_dir_passes.get(&pending.task)
        && pass.pair == pair
        && (crate::readiness::selected_way(d, row) == change::Way::Turn
            || pending.contains_base == change::Status::Passed)
    {
        passed(d, env, owner, pending, pair, out);
    } else {
        let covered = covered(pending.temper_dir.touch, false, false);
        if covered && !may_judge(d, row, &mut pending) {
            finish_step(d, env, pending, out);
            return;
        }
        pending.temper_dir.reading = Some(Box::new(TreeRead {
            head: Reading::new(pair.head, env.limits.client.answer_bytes),
            base: None,
            covered,
        }));
        next(d, env, owner, pending, out);
    }
}

fn may_judge(d: &Domain, row: &ChangeRow, pending: &mut PendingStep) -> bool {
    if crate::readiness::selected_way(d, row) == change::Way::Turn || pending.contains_base == change::Status::Passed {
        true
    } else {
        if pending.contains_base == change::Status::Failed {
            pending.temper_dir.fact = change::TemperDir::Update;
        }
        false
    }
}

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: PendingStep,
    answer: &Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.changes.get(&pending.task) else { return };
    let mut reading = pending.temper_dir.reading.take().expect("file read has its reader");
    let reader = match &mut reading.base {
        Some(base) => base,
        None => &mut reading.head,
    };
    if reader.take(answer).is_err() || pending.woken {
        finish_step(d, env, pending, out);
        return;
    }
    if !reading.covered {
        let pair = Pair::at(&pending).expect("reading has endpoints");
        let touch = pending.temper_dir.touch;
        let head_method = reading.head.method_present() != Some(false);
        // A cut listing doesn't establish absence. A covered cut head is judged, not waived.
        let needs_base = touch == Touch::Unknown
            || (touch == Touch::Temper
                && crate::readiness::selected_way(d, row) == change::Way::Queue
                && pending.contains_base != change::Status::Passed);
        if !head_method && needs_base && reading.base.is_none() && pair.head != pair.base {
            reading.base = Some(Reading::new(pair.base, env.limits.client.answer_bytes));
            pending.temper_dir.reading = Some(reading);
            next(d, env, owner, pending, out);
            return;
        }
        let base_method =
            if let Some(base) = reading.base.take() { base.method_present() != Some(false) } else { false };
        // Before a queue update, the base's method can enter the head that will land.
        let projected_method = head_method || (touch == Touch::Temper && base_method);
        if !covered(touch, projected_method, base_method) {
            pending.temper_dir.fact = change::TemperDir::Uncovered;
            finish_step(d, env, pending, out);
            return;
        }
        reading.covered = true;
        if !may_judge(d, row, &mut pending) {
            finish_step(d, env, pending, out);
            return;
        }
    }
    pending.temper_dir.reading = Some(reading);
    next(d, env, owner, pending, out);
}

fn next(d: &mut Domain, env: &Env<Limits>, owner: Token, mut pending: PendingStep, out: &mut Queue<Request>) {
    let Some(row) = d.changes.get(&pending.task) else { return };
    let reading = pending.temper_dir.reading.as_ref().expect("working reading");
    let reader = match &reading.base {
        Some(base) => base,
        None => &reading.head,
    };
    if let Some(read) = reader.next() {
        let repository = row.repository;
        pending.stage = StepStage::TemperDir;
        d.steps.insert(owner, pending).expect("one step per change");
        gather_read(d, env, owner, repository, read, out);
        return;
    }
    let pair = Pair::at(&pending).expect("judgement has endpoints");
    let root_known = reader.method_present().is_some();
    let verdict = reader.judge(home(d, row));
    pending.temper_dir.reading = None;
    if (verdict.method || !root_known) && !verdict.faults.is_empty() {
        pending.temper_dir.fact = change::TemperDir::Fault { head: pair.head };
        pending.temper_dir.fault = fault_words(pair.head, &verdict, 65_536);
        finish_step(d, env, pending, out);
    } else {
        d.temper_dir_passes
            .insert(pending.task, Pass { pair, owner_short: false })
            .expect("one pass per bounded live change");
        passed(d, env, owner, pending, pair, out);
    }
}

fn home(d: &Domain, row: &ChangeRow) -> Home {
    let repository = d.repository(row.repository).expect("adopted repository");
    match d.home(repository.project) {
        Some(home) if home.provider == row.repository => Home::Here,
        Some(home) => {
            let mut name = Writer::new(home.owner.len().saturating_add(1).saturating_add(home.name.len()));
            name.put(&home.owner).expect("sized owner");
            name.put(b"/").expect("sized separator");
            name.put(&home.name).expect("sized name");
            Home::Elsewhere { name: name.finish() }
        }
        None => Home::None,
    }
}

fn fault_words(head: Commit, verdict: &Verdict, bound: usize) -> Option<Box<[u8]>> {
    let digits = crate::commit_text(&head);
    let mut size = Writer::measure(bound);
    let mut count = verdict.faults.len();
    loop {
        if write_faults(&mut size, &digits, verdict, count).is_ok() {
            let mut words = Writer::new(size.written());
            write_faults(&mut words, &digits, verdict, count).expect("measured fault words");
            return Some(words.finish());
        }
        count = count.checked_sub(1)?;
        size = Writer::measure(bound);
    }
}

fn write_faults(words: &mut Writer, head: &[u8], verdict: &Verdict, count: usize) -> Result<(), Overflow> {
    words.put(b"temper's check of .temper/ at ")?;
    words.put(head)?;
    words.put(b":")?;
    for fault in verdict.faults.get(..count).expect("bounded fault count") {
        words.put(b"\n")?;
        fault.write(words)?;
    }
    let more = u64::from(verdict.more).saturating_add(skein_lib::len_u64(verdict.faults.len().saturating_sub(count)));
    if more != 0 {
        words.put(b"\nand ")?;
        words.put(Decimal::of(more).as_bytes())?;
        words.put(b" more")?;
    }
    Ok(())
}

fn passed(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: PendingStep,
    pair: Pair,
    out: &mut Queue<Request>,
) {
    pending.temper_dir.tree = Some(pair.head);
    pending.temper_dir.fact = change::TemperDir::Passed;
    let Some(row) = d.changes.get(&pending.task) else { return };
    let repository = d.repository(row.repository).expect("adopted repository");
    if d.temper_dir_sign_off(repository.project, TemperDirEvidence::Passed { tree: pair.head }).is_some() {
        let evidence = step_evidence(d, row, &pending, row.change.clone());
        let task = pending.task;
        pending.read = None;
        d.steps.insert(owner, pending).expect("one step per change");
        super::emit(out, Request::Reviewed { task, evidence, temper_dir: Some(owner) });
    } else {
        finish_step(d, env, pending, out);
    }
}

impl Domain {
    pub(crate) fn temper_dir_owner_short(&self, row: &ChangeRow) -> bool {
        let Some(pass) = self.temper_dir_passes.get(&row.task) else { return false };
        pass.owner_short
            && row.change.last_head == Some(pass.pair.head)
            && self.branch_head(&crate::tiers::name(row.repository, &row.base)) == Some(pass.pair.base)
    }

    /// Take the root's mapped reviews for this very step before deciding readiness.
    pub fn temper_dir_reviewed(
        &mut self,
        env: &Env<Limits>,
        owner: Token,
        reviewers: &[Reviewer],
        out: &mut Queue<Request>,
    ) {
        let Some(mut pending) = self.steps.remove(&owner) else { return };
        let Some(row) = self.changes.get(&pending.task) else { return };
        let Some(repository) = self.repository(row.repository) else { return };
        let head = pending.temper_dir.tree.expect("only a passed check asks for sign-off");
        if let Some(Criterion::Approval { role, freshness, people }) =
            self.temper_dir_sign_off(repository.project, TemperDirEvidence::Passed { tree: head })
        {
            pending.temper_dir.fact = match judge::approval(
                role,
                people,
                freshness,
                head,
                &row.change.clean,
                reviewers,
                pending.reviews_complete,
            ) {
                crate::JudgeVerdict::Met => change::TemperDir::Passed,
                crate::JudgeVerdict::Wait => change::TemperDir::AwaitingOwner {
                    until: skein_lib::Wall::from_nanos(
                        env.wall.as_nanos().saturating_add(env.limits.queue_window.as_nanos()),
                    ),
                },
                crate::JudgeVerdict::Refuse => {
                    let (_, requested) = judge::approval_count(role, freshness, head, &row.change.clean, reviewers);
                    change::TemperDir::OwnerRequested { head: requested.expect("approval refusal names a review") }
                }
            };
        }
        if let Some(pass) = self.temper_dir_passes.get_mut(&pending.task)
            && Some(pass.pair) == Pair::at(&pending)
            && !pending.woken
        {
            pass.owner_short = match pending.temper_dir.fact {
                change::TemperDir::AwaitingOwner { .. } => true,
                change::TemperDir::Uncovered
                | change::TemperDir::Pending
                | change::TemperDir::Update
                | change::TemperDir::Passed
                | change::TemperDir::Fault { .. }
                | change::TemperDir::OwnerRequested { .. } => false,
            };
        }
        finish_step(self, env, pending, out);
    }
}
