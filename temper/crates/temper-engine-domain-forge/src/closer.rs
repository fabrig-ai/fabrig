//! Durable goal closer progress and fresh containment (temper/docs/design/domain/forge/branches.md, 2).
use crate::{Domain, Key, Limits, Name, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_lib::{Duration, Env, List, Map, Queue, Token, Wall, bytes};
use temper_engine_domain_forge_change::Status;
pub use temper_engine_domain_forge_vocabulary::closer::*;

#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) rows: Map<u64, CloserRow>,
    pub(crate) reads: Map<Token, Reading>,
    pub(crate) selection_reads: Map<Token, (u64, Repository)>,
    pub(crate) branch_waits: Map<u64, Name>,
    checked: Map<u64, (u64, Result<bool, Error>)>,
    backoffs: Map<u64, Duration>,
    pub(crate) progress: Map<u64, Progress>,
}
#[derive(Hash, Debug)]
pub(crate) struct Progress {
    pub(crate) since: Wall,
    pub(crate) wait: Option<Wait>,
    expired: Option<Box<[u8]>>,
}

#[derive(Hash, Debug)]
pub(crate) enum Wait {
    Landing { read: Box<[u8]>, last: Option<LastAnswer> },
    Reporting(ReportRead, Option<Error>),
    Making,
    OperatorHold,
}

#[derive(Clone, Copy, Hash, Debug)]
pub(crate) enum LastAnswer {
    NotThereYet,
    Refused(Error),
}

#[derive(Hash, Debug)]
pub(crate) enum ReportRead {
    Pull(u64),
    Checks,
    Job,
}

#[derive(Hash, Debug)]
pub(crate) struct Reading {
    pub(crate) task: u64,
    step: u64,
    repository: Repository,
    base: Box<[u8]>,
    read: Box<[u8]>,
    covered: Option<(Commit, Commit)>,
    stage: Stage,
}
#[derive(Hash, Debug)]
enum Stage {
    Branch,
    Base { head: Commit },
    Contains { head: Commit, base: Commit },
}
impl State {
    pub(crate) fn raise(&mut self, tasks: u32) {
        self.rows.raise(tasks);
        self.reads.raise(tasks);
        self.selection_reads.raise(tasks);
        self.branch_waits.raise(tasks);
        self.checked.raise(tasks);
        self.backoffs.raise(tasks);
        self.progress.raise(tasks);
    }
    pub(crate) fn new(tasks: u32) -> Self {
        Self {
            rows: Map::with_capacity(tasks),
            reads: Map::with_capacity(tasks),
            selection_reads: Map::with_capacity(tasks),
            branch_waits: Map::with_capacity(tasks),
            checked: Map::with_capacity(tasks),
            backoffs: Map::with_capacity(tasks),
            progress: Map::with_capacity(tasks),
        }
    }
}
impl Domain {
    #[must_use]
    pub fn closer(&self, task: u64) -> Option<&CloserRow> {
        self.closing.rows.get(&task)
    }

    /// The read bound was met; the route submits the ordinary procedure hold.
    #[must_use]
    pub fn closer_stall_words(&self, task: u64) -> Option<&[u8]> {
        self.closing.progress.get(&task)?.expired.as_deref()
    }

    /// A delegate's end advances the closer before it next reads containment, even if its row is unchanged.
    pub fn closer_delegate_ended(&mut self, task: u64, child: u64, at: Wall) {
        let Some(row) = self.closer(task) else { return };
        let running = match row.closer.running {
            Some(running) => Some(running.change),
            None => None,
        };
        if (running == Some(child) || row.closer.landing == Some(child)) && self.closing.progress.contains_key(&task) {
            advance(self, task, at);
            crate::waits::changed(self, task);
        }
    }

    /// A newly offered step after a release starts a fresh wait, even if its row is unchanged.
    pub fn release_closer_wait(&mut self, task: u64, since: Wall) {
        if self.closer(task).is_some() {
            idle(self, task);
            advance(self, task, since);
            self.closing.progress.get_mut(&task).expect("released closer clock").expired = None;
        }
    }

    /// New work during an offered step can invalidate its containment answer.
    pub fn discard_closer_answer(&mut self, task: u64) {
        self.closing.checked.remove(&task);
    }

    #[must_use]
    pub fn closer_checked(&self, task: u64, step: u64) -> Option<Result<bool, Error>> {
        let (checked_step, result) = self.closing.checked.get(&task)?;
        if *checked_step == step { Some(*result) } else { None }
    }
}
pub(crate) fn save(d: &mut Domain, env: &Env<Limits>, row: CloserRow, out: &mut Queue<Request>) {
    let task = row.task;
    let prior = d.closer(task);
    if row.done {
        ended(d, task);
    } else if prior != Some(&row) {
        let previous = match prior {
            Some(row) => row.closer.running,
            None => None,
        };
        let delegated = row.closer.running.is_some() && previous != row.closer.running;
        advance(d, task, env.wall);
        if row.making.is_some() || row.closer.hold.is_some() || delegated {
            idle(d, task);
        }
    }
    d.closing.rows.insert(task, row.clone()).expect("closer is bounded by tasks");
    out.push(Request::Save { record: Stored::Closer(row), release: None, read_afresh: None });
    crate::waits::changed(d, task);
}

pub(crate) fn restored(d: &mut Domain, since: Wall, row: &CloserRow) {
    if !row.done {
        advance(d, row.task, since);
    }
}

pub(crate) fn ended(d: &mut Domain, task: u64) {
    if d.closing.progress.remove(&task).is_some() {
        idle(d, task);
    }
}

// The goal's closing projection has settled before its connector state is forgotten.
pub(crate) fn forget(d: &mut Domain, goal: u64, out: &mut Queue<Request>) {
    let mut tasks = List::with_capacity(d.closing.rows.len());
    for (task, row) in &d.closing.rows {
        if row.goal == goal && row.done {
            tasks.push(*task).expect("bounded closer rows");
        }
    }
    for task in &tasks {
        ended(d, *task);
        d.closing.rows.remove(task);
        crate::waits::forget(d, *task);
        out.push(Request::Erase { key: Key::Closer(*task) });
    }
}
fn idle(d: &mut Domain, task: u64) {
    set_idle(d, task);
    d.closing.checked.remove(&task);
    d.closing.backoffs.remove(&task);
    d.offered_change(task);
    let mut owners = List::with_capacity(d.closing.reads.len());
    for (owner, reading) in &d.closing.reads {
        if reading.task == task {
            owners.push(*owner).expect("bounded closer reads");
        }
    }
    for owner in &owners {
        d.closing.reads.remove(owner);
        crate::domain::outage::forget_read(d, *owner);
    }
    let mut owners = List::with_capacity(d.closing.selection_reads.len());
    for (owner, reading) in &d.closing.selection_reads {
        if reading.0 == task {
            owners.push(*owner).expect("bounded selection reads");
        }
    }
    for owner in &owners {
        d.closing.selection_reads.remove(owner);
        crate::domain::outage::forget_read(d, *owner);
    }
}

#[expect(clippy::too_many_arguments, reason = "a fresh check carries the procedure step and both branch names")]
pub(crate) fn check(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    step: u64,
    repository: Repository,
    branch: Box<[u8]>,
    base: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    for (_, reading) in &d.closing.reads {
        if reading.task == task {
            return;
        }
    }
    if d.closer_stall_words(task).is_some() || !d.closing.progress.contains_key(&task) {
        return;
    }
    d.offered_change(task);
    d.closing.branch_waits.remove(&task);
    if d.landing_held() {
        set_wait(d, task, Wait::OperatorHold);
    } else if !matches_landing_wait(d, task) {
        set_wait(d, task, Wait::Landing { read: branch_words(&branch), last: None });
    }
    let Some(sequence) = d.sequence.checked_add(1) else { return };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 62));
    let covered = covered(d, task);
    d.closing
        .reads
        .insert(
            owner,
            Reading { task, step, repository, base, read: branch_words(&branch), covered, stage: Stage::Branch },
        )
        .expect("one fresh check per closer");
    crate::domain::child(
        d,
        env,
        skein_forge_client::Event::Read { owner, repository, read: Read::Branch { branch } },
        out,
    );
}
pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut reading) = d.closing.reads.remove(&owner) else { return };
    match reading.stage {
        Stage::Contains { .. } => {
            let refused = result.as_ref().err().copied();
            let error = match &result {
                Err(error) => *error,
                Ok(_) => Error::InvalidAnswer,
            };
            let contained = match crate::domain::ancestry_status(result) {
                Status::Passed => Ok(true),
                Status::Failed => Ok(false),
                Status::Unknown | Status::Pending => Err(error),
            };
            if contained.is_err() {
                read_result(d, &reading, refused);
            }
            finish(d, env, reading, contained, out);
            return;
        }
        Stage::Branch | Stage::Base { .. } => {}
    }
    let answer = match result {
        Ok(answer) => observation(answer),
        Err(error) => {
            read_result(d, &reading, Some(error));
            finish(d, env, reading, Err(error), out);
            return;
        }
    };
    let next = match reading.stage {
        Stage::Branch => match answer {
            Observed::Head { commit } => {
                let head = match reading.covered {
                    Some((landed, merge)) if landed == commit => merge,
                    Some(_) | None => commit,
                };
                reading.stage = Stage::Base { head };
                Read::Branch { branch: reading.base.clone() }
            }
            Observed::Other => {
                read_result(d, &reading, None);
                finish(d, env, reading, Err(Error::InvalidAnswer), out);
                return;
            }
        },
        Stage::Base { head } => match answer {
            Observed::Head { commit: base } => {
                crate::domain::child(
                    d,
                    env,
                    skein_forge_client::Event::Hint {
                        hint: skein_forge::api::Hint {
                            repository: reading.repository,
                            change: skein_forge::api::Change::Branch(reading.base.clone()),
                            key: None,
                        },
                    },
                    out,
                );
                if head == base {
                    finish(d, env, reading, Ok(true), out);
                    return;
                }
                reading.stage = Stage::Contains { head, base };
                Read::Commits { before: base, after: head, page: 1 }
            }
            Observed::Other => {
                read_result(d, &reading, None);
                finish(d, env, reading, Err(Error::InvalidAnswer), out);
                return;
            }
        },
        Stage::Contains { .. } => unreachable!("ancestry answered above"),
    };
    let repository = reading.repository;
    let what = match reading.stage {
        Stage::Branch => unreachable!("a branch answer asks for the base"),
        Stage::Base { .. } => branch_words(&reading.base),
        Stage::Contains { head, base } => contains_words(&reading.base, base, head),
    };
    let task = reading.task;
    reading.read = what;
    d.closing.reads.insert(owner, reading).expect("replaces check");
    // Keep the last answer across retries until another answer arrives.
    if let Some(progress) = d.closing.progress.get_mut(&task)
        && let Some(Wait::Landing { read, last: None }) = &mut progress.wait
    {
        read.clone_from(&d.closing.reads.get(&owner).expect("next containment read").read);
    }
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read: next }, out);
}

#[derive(Debug)]
enum Observed {
    Head { commit: Commit },
    Other,
}
fn observation(answer: Answer) -> Observed {
    match answer {
        Answer::Commit(commit) => Observed::Head { commit },
        Answer::Compare { .. }
        | Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Reviews { .. }
        | Answer::Statuses { .. }
        | Answer::Remarks { .. }
        | Answer::Branches(_)
        | Answer::PullFiles { .. }
        | Answer::Checks(_)
        | Answer::JobState { .. }
        | Answer::Job { .. }
        | Answer::File { .. }
        | Answer::Directory { .. }
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators { .. }
        | Answer::Permission(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Queued
        | Answer::Branch(_)
        | Answer::Runs { .. }
        | Answer::Run(_)
        | Answer::Tree { .. }
        | Answer::Commits { .. }
        | Answer::Done => Observed::Other,
    }
}

fn finish(d: &mut Domain, env: &Env<Limits>, reading: Reading, result: Result<bool, Error>, out: &mut Queue<Request>) {
    if result.is_err() {
        let wait = match d.closing.backoffs.get(&reading.task) {
            Some(previous) => Duration::from_nanos(previous.as_nanos().saturating_mul(2)),
            None => env.limits.client.backoff,
        };
        let wait = core::cmp::min(wait, env.limits.client.poll_max);
        d.closing.backoffs.insert(reading.task, wait).expect("one wait per closer");
        let until = Wall::from_nanos(env.wall.as_nanos().saturating_add(wait.as_nanos()));
        d.wait_change(env, reading.task, Some(until));
    } else {
        d.closing.backoffs.remove(&reading.task);
        set_idle(d, reading.task);
    }
    d.closing.checked.insert(reading.task, (reading.step, result)).expect("one answer per closer");
    out.push(Request::CloserChecked { task: reading.task });
}

// A squash's merge proves coverage of the head it landed (temper/docs/design/domain/forge/branches.md, 2).
fn covered(d: &Domain, task: u64) -> Option<(Commit, Commit)> {
    let landing = d.closing.rows.get(&task)?.closer.landing?;
    let row = d.change(landing)?;
    let merge = match &row.change.state {
        temper_engine_domain_forge_change::State::Landed { merge, .. }
        | temper_engine_domain_forge_change::State::Red { merge, .. }
        | temper_engine_domain_forge_change::State::Cleared { merge }
        | temper_engine_domain_forge_change::State::Uncleared { merge } => *merge,
        temper_engine_domain_forge_change::State::Taking { .. }
        | temper_engine_domain_forge_change::State::Producing { .. }
        | temper_engine_domain_forge_change::State::Opening { .. }
        | temper_engine_domain_forge_change::State::Recreating { .. }
        | temper_engine_domain_forge_change::State::Reopening { .. }
        | temper_engine_domain_forge_change::State::Checking { .. }
        | temper_engine_domain_forge_change::State::Judging { .. }
        | temper_engine_domain_forge_change::State::Gating { .. }
        | temper_engine_domain_forge_change::State::CancellingGates { .. }
        | temper_engine_domain_forge_change::State::Ready { .. }
        | temper_engine_domain_forge_change::State::Preparing { .. }
        | temper_engine_domain_forge_change::State::Queued { .. }
        | temper_engine_domain_forge_change::State::First { .. }
        | temper_engine_domain_forge_change::State::Updating { .. }
        | temper_engine_domain_forge_change::State::WaitingProducer { .. }
        | temper_engine_domain_forge_change::State::Resolving { .. }
        | temper_engine_domain_forge_change::State::Repairing { .. }
        | temper_engine_domain_forge_change::State::Landing { .. }
        | temper_engine_domain_forge_change::State::Held { .. }
        | temper_engine_domain_forge_change::State::NoChange { .. }
        | temper_engine_domain_forge_change::State::Ended => return None,
    };
    Some((row.change.last_head?, merge))
}

// Selection reads are safe while the closer is kept: they make no forge effect.
pub(crate) fn read_selection(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    repository: Repository,
    out: &mut Queue<Request>,
) {
    for (_, reading) in &d.closing.selection_reads {
        if reading.0 == task {
            return;
        }
    }
    if d.closer_stall_words(task).is_some() {
        return;
    }
    let Some(row) = d.closing.rows.get(&task) else { return };
    let Some(read) = temper_engine_domain_forge_closer::selection_read(&row.closer) else { return };
    d.closing.branch_waits.remove(&task);
    d.sequence = d.sequence.checked_add(1).expect("closer read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 62));
    d.closing.selection_reads.insert(owner, (task, repository)).expect("one selection read per closer");
    set_wait(d, task, Wait::Reporting(report_read(&read), None));
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}

pub(crate) fn selection_answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some((task, repository)) = d.closing.selection_reads.remove(&owner) else { return };
    if !d.repositories.contains_key(&repository) {
        return;
    }
    let Some(mut row) = d.closing.rows.get(&task).cloned() else { return };
    if row.done {
        return;
    }
    temper_engine_domain_forge_closer::selection_answer(&mut row.closer, result);
    let read = temper_engine_domain_forge_closer::selection_read(&row.closer);
    save(d, env, row, out);
    match read {
        Some(read) => set_wait(d, task, Wait::Reporting(report_read(&read), None)),
        None => set_idle(d, task),
    }
    out.push(Request::CloserChecked { task });
}

pub(crate) fn release_repository(d: &mut Domain, repository: Repository) {
    let mut tasks = List::with_capacity(d.closing.branch_waits.len());
    for (task, name) in &d.closing.branch_waits {
        if name.forge == repository.forge && name.repository == repository.repository {
            tasks.push(*task).expect("bounded closer waits");
        }
    }
    for task in &tasks {
        set_idle(d, *task);
    }
    let mut keys = List::with_capacity(d.closing.reads.len());
    for (key, row) in &d.closing.reads {
        if row.repository == repository {
            keys.push(*key).expect("bounded closing.reads");
        }
    }
    for key in keys.into_boxed() {
        let row = d.closing.reads.remove(&key).expect("collected closing.reads");
        idle(d, row.task);
    }
    let mut keys = List::with_capacity(d.closing.selection_reads.len());
    for (key, row) in &d.closing.selection_reads {
        if row.1 == repository {
            keys.push(*key).expect("bounded closing.selection_reads");
        }
    }
    for key in keys.into_boxed() {
        let (task, _) = d.closing.selection_reads.remove(&key).expect("collected closing.selection_reads");
        set_idle(d, task);
    }
}

fn advance(d: &mut Domain, task: u64, since: Wall) {
    if let Some(progress) = d.closing.progress.get_mut(&task) {
        progress.since = since;
    } else {
        d.closing.progress.insert(task, Progress { since, wait: None, expired: None }).expect("one clock per closer");
    }
}

fn set_wait(d: &mut Domain, task: u64, wait: Wait) {
    if let Some(progress) = d.closing.progress.get_mut(&task)
        && progress.expired.is_none()
    {
        progress.wait = Some(wait);
        crate::waits::changed(d, task);
    }
}

pub(crate) fn set_idle(d: &mut Domain, task: u64) {
    d.closing.branch_waits.remove(&task);
    if let Some(progress) = d.closing.progress.get_mut(&task) {
        progress.wait = None;
    }
    crate::waits::changed(d, task);
}

pub(crate) fn park(d: &mut Domain, task: u64, wait: CloserPark, out: &mut Queue<Request>) {
    match wait {
        CloserPark::DelegateOrHeld => idle(d, task),
        CloserPark::Making(name) => {
            if d.goal_branch_waiting(&name) {
                d.closing.branch_waits.insert(task, name).expect("one branch wait per closer");
                set_wait(d, task, Wait::Making);
            } else {
                set_idle(d, task);
                out.push(Request::WakeProcedure { task });
            }
        }
        CloserPark::OperatorHold => {
            idle(d, task);
            set_wait(d, task, Wait::OperatorHold);
        }
    }
}

fn matches_landing_wait(d: &Domain, task: u64) -> bool {
    if let Some(progress) = d.closing.progress.get(&task)
        && let Some(Wait::Landing { .. }) = progress.wait
    {
        return true;
    }
    false
}

fn branch_words(branch: &[u8]) -> Box<[u8]> {
    bytes::concat(&[b"branch ", branch])
}

fn contains_words(base: &[u8], at: Commit, head: Commit) -> Box<[u8]> {
    let at = crate::commit::commit_text(&at);
    let head = crate::commit::commit_text(&head);
    bytes::concat(&[
        b"whether ",
        base,
        b" at ",
        at.get(..12).expect("commit prefix"),
        b" holds the goal's tip ",
        head.get(..12).expect("commit prefix"),
    ])
}

fn read_result(d: &mut Domain, reading: &Reading, error: Option<Error>) {
    if d.landing_held() {
        return;
    }
    let last = match error {
        Some(error) => LastAnswer::Refused(error),
        None => LastAnswer::NotThereYet,
    };
    set_wait(d, reading.task, Wait::Landing { read: reading.read.clone(), last: Some(last) });
}

// Shared forge outages can retain a read without sending it back to its owner.
pub(crate) fn selection_unavailable(d: &mut Domain, owner: Token, error: Error) {
    if let Some((task, _)) = d.closing.selection_reads.get(&owner)
        && let Some(progress) = d.closing.progress.get_mut(task)
        && let Some(Wait::Reporting(_, last)) = &mut progress.wait
    {
        *last = Some(error);
    }
}

fn report_read(read: &Read) -> ReportRead {
    if let Read::Pull { number, .. } = read {
        ReportRead::Pull(*number)
    } else if let Read::Checks { .. } = read {
        ReportRead::Checks
    } else {
        let Read::Job { .. } = read else { unreachable!("a report reads pulls, checks or jobs") };
        ReportRead::Job
    }
}

fn bounded(wait: &Wait) -> bool {
    match wait {
        Wait::Landing { .. } | Wait::Reporting(..) | Wait::Making => true,
        Wait::OperatorHold => false,
    }
}

pub(crate) fn deadline(d: &Domain) -> Option<Wall> {
    let mut first: Option<Wall> = None;
    for (_, progress) in &d.closing.progress {
        if let Some(wait) = &progress.wait
            && bounded(wait)
        {
            let at = crate::waits::step_bound(progress.since, d.landing.limits.waits.step);
            first = Some(match first {
                Some(first) => first.min(at),
                None => at,
            });
        }
    }
    first
}

pub(crate) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut expired = List::with_capacity(d.closing.progress.len());
    for (task, progress) in &d.closing.progress {
        if let Some(wait) = &progress.wait
            && bounded(wait)
            && crate::waits::step_bound(progress.since, env.limits.waits.step) <= env.wall
        {
            expired.push(*task).expect("one expiry per closer");
        }
    }
    for task in expired.into_boxed() {
        let wait = d.closing.progress.get_mut(&task).expect("collected closer").wait.take().expect("collected wait");
        let words = stall_words(d, task, env.limits.waits.step, wait);
        idle(d, task);
        d.closing.progress.get_mut(&task).expect("collected closer").expired = Some(words);
        out.push(Request::CloserChecked { task });
    }
}

fn stall_words(d: &Domain, task: u64, figure: Duration, wait: Wait) -> Box<[u8]> {
    match wait {
        Wait::Landing { read, last } => {
            let last = match last {
                Some(LastAnswer::Refused(error)) => crate::effect_words::error_words(error),
                Some(LastAnswer::NotThereYet) => Box::from(b"not there yet".as_slice()),
                None => Box::from(b"no answer".as_slice()),
            };
            crate::waits::stalled_words(b"landing", figure, &read, Some(&last))
        }
        Wait::Reporting(read, last) => {
            let what = match read {
                ReportRead::Pull(number) => {
                    bytes::concat(&[b"pull request #", skein_lib::Decimal::of(number).as_bytes()])
                }
                ReportRead::Checks => Box::from(b"checks".as_slice()),
                ReportRead::Job => Box::from(b"a job log".as_slice()),
            };
            let last = crate::waits::read_answer(last);
            crate::waits::stalled_words(b"reporting", figure, &what, Some(&last))
        }
        Wait::Making => {
            let answer = match d.closing.branch_waits.get(&task) {
                Some(name) => match crate::domain::goal_branch::last_answer(d, name) {
                    Some(error) => Some(crate::effect_words::error_words(error)),
                    None => None,
                },
                None => None,
            };
            crate::waits::stalled_words(
                b"its goal branch's making",
                figure,
                b"its goal branch to be made",
                answer.as_deref(),
            )
        }
        Wait::OperatorHold => unreachable!("the operator's hold has no bound"),
    }
}
