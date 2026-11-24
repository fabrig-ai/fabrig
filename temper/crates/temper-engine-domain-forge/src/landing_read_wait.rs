//! The top's volatile fresh-read wait runs (temper/docs/design/domain/landing-waits.md, 3–4).
use super::{Domain, Env, Limits, Pending, Request, RoundRead, Stage, Waiter, append_words, commit_hex, emit, policy};
use crate::{LandingReadNote, LandingReadNoteKind as Note, LandingRow};
use alloc::boxed::Box;
use skein_forge::api::{Error, Read, Repository};
use skein_lib::{Decimal, Duration, List, Queue, Time, Token};

#[derive(Hash, Debug)]
pub(super) struct Run {
    round: u16,
    // A run stays the same across loss of access, but its clock starts anew.
    clock: Option<Clock>,
    last: Failure,
}
#[derive(Clone, Copy, Hash, Debug)]
struct Clock {
    since: Time,
    until: Time,
}
#[derive(Hash, Debug)]
pub(super) struct Failure {
    repository: Repository,
    read: Option<Read>,
    change: Option<u64>,
    cause: Cause,
}
#[derive(Clone, Copy, Hash, Debug)]
pub(super) enum Cause {
    Pending,
    NotThere,
    Refused(Error),
}

impl Cause {
    pub(super) fn answered(error: Option<Error>) -> Self {
        match error {
            None | Some(Error::Missing) => Self::NotThere,
            Some(error) => Self::Refused(error),
        }
    }
    fn words(self) -> Box<[u8]> {
        match self {
            Self::Pending => Box::from(b"no answer".as_slice()),
            Self::NotThere => Box::from(b"not there yet".as_slice()),
            Self::Refused(error) => crate::effect_words::error_words(error),
        }
    }
}

impl Failure {
    pub(super) fn pending(row: &LandingRow, p: &Pending, cause: Cause) -> Self {
        let position = match p.stage {
            Stage::Branch(member)
            | Stage::Ancestry(member)
            | Stage::Pull(member)
            | Stage::Reviews(member, _)
            | Stage::CleanupBranch(member)
            | Stage::Round {
                member,
                stage: RoundRead::Branch | RoundRead::Prefix(_) | RoundRead::Head | RoundRead::Contained,
                ..
            }
            | Stage::Tree { member, main: false, .. } => Some(member),
            Stage::Main(_)
            | Stage::ContainsCandidate(_)
            | Stage::TurnPull
            | Stage::Paths(_)
            | Stage::Collect
            | Stage::CleanupPull
            | Stage::StallRun
            | Stage::StallJobs
            | Stage::Round { stage: RoundRead::Pull | RoundRead::Upstream, .. }
            | Stage::Run { .. }
            | Stage::FailedReport { .. }
            | Stage::FailedJobs { .. }
            | Stage::Tree { main: true, .. } => None,
        };
        let change = match position {
            Some(position) => Some(row.landing.members.get(usize::from(position)).expect("read member").change),
            None => None,
        };
        Self { repository: p.repository, read: Some(p.read.clone()), change, cause }
    }
    pub(super) const fn missing_pull(repository: Repository, change: u64) -> Self {
        Self { repository, read: None, change: Some(change), cause: Cause::NotThere }
    }
    pub(super) const fn repository(&self) -> Repository {
        self.repository
    }
    fn reading(&self) -> Box<[u8]> {
        let mut words = List::uncapped();
        match self.read.as_ref() {
            Some(Read::Commits { before, after, page }) => {
                append_words(&mut words, b"commits after ");
                append_words(&mut words, &commit_hex(*before));
                append_words(&mut words, b" up to ");
                append_words(&mut words, &commit_hex(*after));
                append_words(&mut words, b", page ");
                append_words(&mut words, Decimal::of(u64::from(*page)).as_bytes());
            }
            Some(Read::Branch { branch }) => {
                append_words(&mut words, b"branch ");
                append_words(&mut words, branch);
            }
            Some(Read::Pull { number, .. } | Read::Reviews { number, .. }) => {
                append_words(&mut words, b"pull request #");
                append_words(&mut words, Decimal::of(*number).as_bytes());
                if let Some(Read::Reviews { page, .. }) = &self.read {
                    append_words(&mut words, b"'s reviews, page ");
                    append_words(&mut words, Decimal::of(u64::from(*page)).as_bytes());
                }
            }
            Some(Read::Compare { before, after }) => {
                append_words(&mut words, b"the comparison of ");
                append_words(&mut words, &commit_hex(*before));
                append_words(&mut words, b" and ");
                append_words(&mut words, &commit_hex(*after));
            }
            Some(Read::Tree { object, page }) => {
                append_words(&mut words, b"tree ");
                append_words(&mut words, &commit_hex(*object));
                append_words(&mut words, b", page ");
                append_words(&mut words, Decimal::of(u64::from(*page)).as_bytes());
            }
            read => append_words(&mut words, name(read)),
        }
        if let Some(change) = self.change {
            append_words(&mut words, b", for change ");
            append_words(&mut words, Decimal::of(change).as_bytes());
        }
        words.into_boxed()
    }
}

fn name(read: Option<&Read>) -> &'static [u8] {
    match read {
        None | Some(Read::Pull { .. }) => b"pull request",
        Some(Read::Locate { .. }) => b"locate",
        Some(Read::Items { .. }) => b"items",
        Some(Read::Item { .. }) => b"item",
        Some(Read::PullFor { .. }) => b"pull for",
        Some(Read::Reviews { .. }) => b"reviews",
        Some(Read::Statuses { .. }) => b"statuses",
        Some(Read::Remarks { .. }) => b"remarks",
        Some(Read::Branch { .. }) => b"branch",
        Some(Read::Branches { .. }) => b"branches",
        Some(Read::PullFiles { .. }) => b"pull files",
        Some(Read::Compare { .. }) => b"compare",
        Some(Read::Checks { .. }) => b"checks",
        Some(Read::Job { .. }) => b"job",
        Some(Read::JobState { .. }) => b"job state",
        Some(Read::File { .. }) => b"file",
        Some(Read::Protection { .. }) => b"protection",
        Some(Read::Settings) => b"settings",
        Some(Read::Collaborators { .. }) => b"collaborators",
        Some(Read::Permission { .. }) => b"permission",
        Some(Read::Runs { .. }) => b"runs",
        Some(Read::Run { .. }) => b"run",
        Some(Read::Tree { .. }) => b"tree",
        Some(Read::Artifact { .. }) => b"artifact",
        Some(Read::Commits { .. }) => b"commits",
    }
}

impl Clock {
    fn new(env: &Env<Limits>) -> Self {
        Self { since: env.now, until: env.now.saturating_add(env.limits.change_policy.stall) }
    }
    fn elapsed(self, now: Time) -> Duration {
        Duration::from_nanos(now.as_nanos().saturating_sub(self.since.as_nanos()))
    }
}

fn note(task: u64, round: u16, reading: Box<[u8]>, kind: Note, out: &mut Queue<Request>) {
    emit(out, Request::LandingRead(LandingReadNote { task, round, reading, kind }));
}

fn connected(d: &Domain, forge: u16) -> bool {
    !crate::domain::connection_wait::disconnected(&d.connection(forge))
}

pub(super) fn record(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    failure: Failure,
    exempt: bool,
    out: &mut Queue<Request>,
) {
    let Some(row) = d.landing.rows.get(&task) else { return };
    if super::matches_terminal(&row.landing.phase) {
        return;
    }
    let access = !exempt && connected(d, failure.repository.forge);
    if let Some(run) = d.landing.read_runs.get_mut(&task) {
        run.last = failure;
        if !access {
            run.clock = None;
        } else if run.clock.is_none() {
            run.clock = Some(Clock::new(env));
        }
    } else {
        let round = row.landing.preparation;
        note(task, round, failure.reading(), Note::Started { why: failure.cause.words() }, out);
        d.landing
            .read_runs
            .insert(task, Run { round, clock: if access { Some(Clock::new(env)) } else { None }, last: failure })
            .expect("one read run per landing");
    }
}

pub(super) fn bound(d: &Domain, task: u64) -> Option<Time> {
    Some(d.landing.read_runs.get(&task)?.clock?.until)
}

pub(super) fn repository(d: &Domain, task: u64) -> Option<Repository> {
    Some(d.landing.read_runs.get(&task)?.last.repository)
}

pub(super) fn running(d: &Domain, task: u64) -> bool {
    d.landing.read_runs.contains_key(&task)
}

pub(super) fn pause(d: &mut Domain, task: u64) {
    if let Some(run) = d.landing.read_runs.get_mut(&task) {
        run.clock = None;
    }
}

pub(super) fn resume(d: &mut Domain, env: &Env<Limits>, task: u64, repository: Repository) {
    if let Some(run) = d.landing.read_runs.get(&task)
        && run.clock.is_none()
        && connected(d, repository.forge)
    {
        d.landing.read_runs.get_mut(&task).expect("paused read run").clock = Some(Clock::new(env));
    }
}

pub(super) fn connection(d: &mut Domain, env: &Env<Limits>, forge: u16, available: bool) {
    // A gather may have advanced to another read since its last failure.
    let mut affected = List::with_capacity(d.landing.read_runs.len());
    for (task, run) in &d.landing.read_runs {
        let mut provider = run.last.repository.forge;
        let mut retrying = false;
        for (owner, pending) in &d.landing.pending {
            if pending.task == *task {
                provider = pending.repository.forge;
                retrying = crate::domain::outage::reading(d, *owner);
                break;
            }
        }
        if provider == forge && (!available || !retrying) {
            affected.push(*task).expect("one run per landing");
        }
    }
    for task in affected.into_boxed() {
        let run = d.landing.read_runs.get_mut(&task).expect("affected run");
        if !available {
            run.clock = None;
        } else if run.clock.is_none() {
            run.clock = Some(Clock::new(env));
        }
    }
}

pub(super) fn deadline(d: &Domain) -> Option<Time> {
    if d.landing.read_runs.is_empty() {
        return None;
    }
    let mut next: Option<Time> = None;
    for (_, pending) in &d.landing.pending {
        if pending.entry != 0
            && let Some(until) = bound(d, pending.task)
        {
            next = Some(match next {
                Some(at) => at.min(until),
                None => until,
            });
        }
    }
    next
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.landing.read_runs.is_empty() {
        return;
    }
    let mut due = List::with_capacity(d.landing.pending.len());
    for (owner, pending) in &d.landing.pending {
        if pending.entry != 0
            && let Some(until) = bound(d, pending.task)
            && until <= env.now
        {
            due.push(*owner).expect("one bound per gather");
        }
    }
    for owner in due.into_boxed() {
        let p = d.landing.pending.remove(&owner).expect("due gather");
        if let Some(row) = d.landing.rows.get(&p.task)
            && !super::matches_terminal(&row.landing.phase)
        {
            let failure = Failure::pending(row, &p, Cause::Pending);
            super::read_ready(d, p.task, super::ReadStep::Failed(failure), out);
        }
        d.read_line.leave(Waiter::Landing(owner));
    }
}

pub(super) fn clear(d: &mut Domain, task: u64) {
    d.landing.read_runs.remove(&task);
}

pub(super) fn decided(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    decision: policy::Decision,
    out: &mut Queue<Request>,
) {
    if decision == (policy::Decision::Wait { until: None, on: policy::Awaited::Release })
        || decision == policy::Decision::Cancel
        || decision == policy::Decision::Finish
    {
        clear(d, task);
        return;
    }
    if let policy::Decision::Wait { on: policy::Awaited::Read, .. } = decision {
        return;
    }
    let Some(run) = d.landing.read_runs.remove(&task) else { return };
    if let Some(row) = d.landing.rows.get(&task)
        && !super::matches_terminal(&row.landing.phase)
    {
        let elapsed = match run.clock {
            Some(clock) => clock.elapsed(env.now),
            None => Duration::from_nanos(0),
        };
        note(task, run.round, run.last.reading(), Note::Read { elapsed }, out);
    }
}

pub(super) fn held(d: &mut Domain, env: &Env<Limits>, row: &mut LandingRow, out: &mut Queue<Request>) {
    let run = d.landing.read_runs.remove(&row.landing.task).expect("expired run");
    let elapsed = run.clock.expect("expired clock").elapsed(env.now);
    let reading = run.last.reading();
    let why = run.last.cause.words();
    let mut words = List::with_capacity(crate::history::REASON_BYTES);
    for part in [b"read: ".as_slice(), &reading, b": ", &why] {
        append_words(&mut words, part);
    }
    row.read_hold = Some(words.into_boxed());
    note(row.landing.task, run.round, reading, Note::Held { elapsed, why }, out);
}

// Access checks in the parent's read path don't expose the client's outage state.
pub(crate) fn unavailable(d: &mut Domain, owner: Token) {
    if let Some(p) = d.landing.pending.get(&owner) {
        pause(d, p.task);
    }
}
