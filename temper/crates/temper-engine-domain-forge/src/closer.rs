//! Durable goal closer progress and fresh containment (temper/docs/design/domain/forge/branches.md, 2).
use crate::{Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_lib::{Duration, Env, List, Map, Queue, Token, Wall};
use temper_engine_domain_forge_change::Status;
pub use temper_engine_domain_forge_vocabulary::closer::*;

#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) rows: Map<u64, CloserRow>,
    pub(crate) reads: Map<Token, Reading>,
    pub(crate) selection_reads: Map<Token, u64>,
    checked: Map<u64, (u64, Result<bool, Error>)>,
    waits: Map<u64, Duration>,
}
#[derive(Hash, Debug)]
pub(crate) struct Reading {
    task: u64,
    step: u64,
    repository: Repository,
    base: Box<[u8]>,
    covered: Option<(Commit, Commit)>,
    stage: Stage,
}
#[derive(Hash, Debug)]
enum Stage {
    Branch,
    Base { head: Commit },
    Contains,
}
impl State {
    pub(crate) fn new(tasks: u32) -> Self {
        Self {
            rows: Map::with_capacity(tasks),
            reads: Map::with_capacity(tasks),
            selection_reads: Map::with_capacity(tasks),
            checked: Map::with_capacity(tasks),
            waits: Map::with_capacity(tasks),
        }
    }
}
impl Domain {
    #[must_use]
    pub fn closer(&self, task: u64) -> Option<&CloserRow> {
        self.closing.rows.get(&task)
    }

    #[must_use]
    pub fn closer_checked(&self, task: u64, step: u64) -> Option<Result<bool, Error>> {
        let (checked_step, result) = self.closing.checked.get(&task)?;
        if *checked_step == step { Some(*result) } else { None }
    }
}
pub(crate) fn save(d: &mut Domain, row: CloserRow, out: &mut Queue<Request>) {
    if row.done {
        discard_check(d, row.task);
    }
    d.closing.rows.insert(row.task, row.clone()).expect("closer is bounded by tasks");
    out.push(Request::Save { record: Stored::Closer(row), release: None, read_afresh: None });
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
        discard_check(d, *task);
        d.closing.rows.remove(task);
        out.push(Request::Erase { key: Key::Closer(*task) });
    }
}
fn discard_check(d: &mut Domain, task: u64) {
    d.closing.checked.remove(&task);
    d.closing.waits.remove(&task);
    d.offered_change(task);
    let mut owners = List::with_capacity(d.closing.reads.len());
    for (owner, reading) in &d.closing.reads {
        if reading.task == task {
            owners.push(*owner).expect("bounded closer reads");
        }
    }
    for owner in &owners {
        d.closing.reads.remove(owner);
    }
    let mut owners = List::with_capacity(d.closing.selection_reads.len());
    for (owner, reading) in &d.closing.selection_reads {
        if *reading == task {
            owners.push(*owner).expect("bounded selection reads");
        }
    }
    for owner in &owners {
        d.closing.selection_reads.remove(owner);
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
    d.offered_change(task);
    let Some(sequence) = d.sequence.checked_add(1) else { return };
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 62));
    let covered = covered(d, task);
    d.closing
        .reads
        .insert(owner, Reading { task, step, repository, base, covered, stage: Stage::Branch })
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
        Stage::Contains => {
            let error = match &result {
                Err(error) => *error,
                Ok(_) => Error::InvalidAnswer,
            };
            let contained = match crate::domain::ancestry_status(result) {
                Status::Passed => Ok(true),
                Status::Failed => Ok(false),
                Status::Unknown | Status::Pending => Err(error),
            };
            finish(d, env, reading, contained, out);
            return;
        }
        Stage::Branch | Stage::Base { .. } => {}
    }
    let answer = match result {
        Ok(answer) => observation(answer),
        Err(error) => {
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
                reading.stage = Stage::Contains;
                Read::Commits { before: base, after: head, page: 1 }
            }
            Observed::Other => {
                finish(d, env, reading, Err(Error::InvalidAnswer), out);
                return;
            }
        },
        Stage::Contains => unreachable!("ancestry answered above"),
    };
    let repository = reading.repository;
    d.closing.reads.insert(owner, reading).expect("replaces check");
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
        let wait = match d.closing.waits.get(&reading.task) {
            Some(previous) => Duration::from_nanos(previous.as_nanos().saturating_mul(2)),
            None => env.limits.client.backoff,
        };
        let wait = core::cmp::min(wait, env.limits.client.poll_max);
        d.closing.waits.insert(reading.task, wait).expect("one wait per closer");
        let until = Wall::from_nanos(env.wall.as_nanos().saturating_add(wait.as_nanos()));
        d.wait_change(env, reading.task, Some(until));
    } else {
        d.closing.waits.remove(&reading.task);
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
        temper_engine_domain_forge_change::State::Producing { .. }
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
        if *reading == task {
            return;
        }
    }
    let Some(row) = d.closing.rows.get(&task) else { return };
    let Some(read) = temper_engine_domain_forge_closer::selection_read(&row.closer) else { return };
    d.sequence = d.sequence.checked_add(1).expect("closer read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 62));
    d.closing.selection_reads.insert(owner, task).expect("one selection read per closer");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}

pub(crate) fn selection_answered(
    d: &mut Domain,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(task) = d.closing.selection_reads.remove(&owner) else { return };
    let Some(mut row) = d.closing.rows.get(&task).cloned() else { return };
    if row.done {
        return;
    }
    temper_engine_domain_forge_closer::selection_answer(&mut row.closer, result);
    save(d, row, out);
    out.push(Request::CloserChecked { task });
}
