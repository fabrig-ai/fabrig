//! A landing's volatile progress clock and fresh-read log (temper/docs/design/domain/landing-waits.md, 3–4).
use super::{
    Domain, Env, Limits, Pending, Request, RoundRead, Stage, Waiter, append_words, commit_hex, emit, policy, repository,
};
use crate::{LandingEntry, LandingFollow, LandingReadNote, LandingReadNoteKind as Note, LandingRow};
use alloc::boxed::Box;
use skein_forge::api::{Error, Read, Repository};
use skein_lib::{Decimal, Duration, List, Queue, Time, Token, Wall};

#[derive(Hash, Debug)]
pub(super) struct Progress {
    clock: Clock,
    repository: Repository,
    paused: bool,
    reads: u64,
    last: Option<Failure>,
    wait: Option<Wait>,
    run: Option<Run>,
}

#[derive(Hash, Debug)]
struct Run {
    round: u16,
    since: Time,
    last: Failure,
}

#[derive(Clone, Copy, Hash, Debug)]
struct Clock {
    since: Time,
    wall: Wall,
    until: Time,
}

#[derive(Hash, Debug)]
enum Wait {
    Read(Failure),
    Poll,
    Write(Box<[u8]>),
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
    Answered,
    NotThere,
    Behind { order: u64 },
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
            Self::Answered => Box::from(b"answered".as_slice()),
            Self::NotThere => Box::from(b"not there yet".as_slice()),
            Self::Behind { order } => skein_lib::bytes::concat(&[
                b"an older state of temper's writes, behind entry ",
                Decimal::of(order).as_bytes(),
                b"; the read may be stale or another's write",
            ]),
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
            | Stage::Paths(member)
            | Stage::ContainsCandidate(member)
            | Stage::CleanupBranch(member)
            | Stage::Round {
                member,
                stage: RoundRead::Branch | RoundRead::Prefix(_) | RoundRead::Head | RoundRead::Contained,
                ..
            }
            | Stage::Tree { member, main: false, .. } => Some(member),
            Stage::Run { .. } | Stage::FailedReport { .. } | Stage::FailedJobs { .. } => {
                if let Some(entry) = p.flight_entry
                    && let Some(entry) = row.run_entry(entry)
                {
                    Some(entry.member)
                } else {
                    None
                }
            }
            Stage::Main(_)
            | Stage::TurnPull
            | Stage::Collect
            | Stage::CleanupPull
            | Stage::StallRun
            | Stage::StallJobs
            | Stage::TemperDir
            | Stage::Round { stage: RoundRead::Pull | RoundRead::Upstream, .. }
            | Stage::Tree { main: true, .. } => None,
        };
        let mut change = None;
        if let Some(position) = position
            && let Some(member) = row.landing.members.get(usize::from(position))
        {
            change = Some(member.change);
        }
        Self { repository: p.repository, read: Some(p.read.clone()), change, cause }
    }
    pub(super) const fn missing_pull(repository: Repository, change: u64) -> Self {
        Self { repository, read: None, change: Some(change), cause: Cause::NotThere }
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
        Self { since: env.now, wall: env.wall, until: env.now.saturating_add(env.limits.change_policy.stall) }
    }
}

fn elapsed(now: Time, since: Time) -> Duration {
    Duration::from_nanos(now.as_nanos().saturating_sub(since.as_nanos()))
}

fn note(task: u64, round: u16, reading: Box<[u8]>, kind: Note, out: &mut Queue<Request>) {
    emit(out, Request::LandingRead(LandingReadNote { task, round, reading, kind }));
}

fn connected(d: &Domain, forge: u16) -> bool {
    !crate::domain::connection_wait::disconnected(&d.connection(forge))
}

// Only the substance of the row advances the landing. Room, like `waiting`,
// is a wait; the timestamps and hold text only describe it to a person.
pub(super) fn changed(old: &LandingRow, row: &LandingRow) -> bool {
    let LandingRow {
        landing,
        previous_asks: _,
        entries,
        follows: _,
        flake_reruns,
        job,
        waiting: _,
        room: _,
        last_event: _,
        turn_since: _,
        read_hold: _,
        gate_hold: _,
    } = row;
    landing_changed(&old.landing, landing)
        || entries_changed(&old.entries, entries)
        || follows_changed(old.follows(), row.follows())
        || *flake_reruns != old.flake_reruns
        || *job != old.job
}

fn landing_changed(old: &policy::Landing, landing: &policy::Landing) -> bool {
    let policy::Landing {
        last_left,
        searched,
        task,
        project,
        phase,
        members,
        preparation,
        stale_proofs,
        branch,
        round,
        last_prepare,
    } = landing;
    *last_left != old.last_left
        || *searched != old.searched
        || *task != old.task
        || *project != old.project
        || *phase != old.phase
        || *members != old.members
        || *preparation != old.preparation
        || *stale_proofs != old.stale_proofs
        || *branch != old.branch
        || *round != old.round
        || prepare_changed(old.last_prepare.as_ref(), last_prepare.as_ref())
}

fn prepare_changed(old: Option<&policy::LastPrepare>, next: Option<&policy::LastPrepare>) -> bool {
    match (old, next) {
        (Some(old), Some(next)) => {
            // Arming a retry is a wait. Asking for a new prepare advances the round.
            let policy::LastPrepare { member, head, main, suffix, asks, again: _ } = next;
            *member != old.member
                || *head != old.head
                || *main != old.main
                || *suffix != old.suffix
                || *asks != old.asks
        }
        (None, None) => false,
        (None, Some(_)) | (Some(_), None) => true,
    }
}

fn entries_changed(old: &[LandingEntry], next: &[LandingEntry]) -> bool {
    if old.len() != next.len() {
        return true;
    }
    for (old, next) in old.iter().zip(next) {
        if entry_changed(*old, *next) {
            return true;
        }
    }
    false
}

fn follows_changed(old: &[LandingFollow], next: &[LandingFollow]) -> bool {
    if old.len() != next.len() {
        return true;
    }
    for (old, next) in old.iter().zip(next) {
        let LandingFollow { entry, dispatch, start, since, wall, try_number, failed, reruns } = next;
        if entry_changed(old.entry, *entry)
            || *dispatch != old.dispatch
            || *start != old.start
            || *since != old.since
            || *wall != old.wall
            || *try_number != old.try_number
            || *failed != old.failed
            || *reruns != old.reruns
        {
            return true;
        }
    }
    false
}

fn entry_changed(old: LandingEntry, next: LandingEntry) -> bool {
    if let (Some(a), Some(b)) = (old.run, next.run)
        && let (Some(a_clock), Some(b_clock)) = (a.clock, b.clock)
    {
        // A gate poll accumulates time without learning anything new. A new
        // run, attempt, execution state or verdict is a real observation.
        return LandingEntry { run: Some(crate::LandingRun { clock: None, ..a }), ..old }
            != LandingEntry { run: Some(crate::LandingRun { clock: None, ..b }), ..next }
            || a_clock.seen != b_clock.seen;
    }
    old != next
}

pub(super) fn advanced(d: &mut Domain, env: &Env<Limits>, row: &LandingRow) {
    let task = row.landing.task;
    if super::matches_terminal(&row.landing.phase) {
        clear(d, task);
        return;
    }
    if let Some(progress) = d.landing.progress.get_mut(&task) {
        progress.clock = Clock::new(env);
        progress.reads = 0;
    } else if let Some(member) = row.landing.members.first() {
        let repository = repository(member);
        let paused = !connected(d, repository.forge);
        d.landing
            .progress
            .insert(
                task,
                Progress { clock: Clock::new(env), repository, paused, reads: 0, last: None, wait: None, run: None },
            )
            .expect("one clock per live landing");
    }
}

pub(super) fn last_advance(d: &Domain, task: u64) -> Option<(Wall, u64)> {
    let progress = d.landing.progress.get(&task)?;
    Some((progress.clock.wall, progress.reads))
}

pub(super) fn issued(d: &mut Domain, env: &Env<Limits>, owner: Token) {
    let p = d.landing.pending.get(&owner).expect("issued gather");
    let Some(row) = d.landing.rows.get(&p.task) else { return };
    let task = p.task;
    let failure = Failure::pending(row, p, Cause::Pending);
    let repository = p.repository;
    let available = connected(d, repository.forge);
    if !d.landing.progress.contains_key(&task) {
        let row = row.clone();
        advanced(d, env, &row);
    }
    if let Some(progress) = d.landing.progress.get_mut(&task) {
        let was_paused = progress.paused;
        access(progress, env, repository, available);
        progress.reads = progress.reads.saturating_add(1);
        progress.last = Some(failure);
        if was_paused != progress.paused {
            crate::waits::changed(d, task);
        }
    }
}

pub(super) fn answered(d: &mut Domain, p: &Pending, error: Option<Error>) {
    if let Some(progress) = d.landing.progress.get_mut(&p.task)
        && let Some(last) = &mut progress.last
    {
        last.cause = match error {
            None => Cause::Answered,
            Some(error) => Cause::answered(Some(error)),
        };
    }
}

fn access(progress: &mut Progress, env: &Env<Limits>, repository: Repository, available: bool) {
    progress.repository = repository;
    if progress.paused && available {
        progress.clock = Clock::new(env);
        progress.reads = 0;
    }
    progress.paused = !available;
}

pub(super) fn pause(d: &mut Domain, task: u64) {
    if let Some(progress) = d.landing.progress.get_mut(&task)
        && !progress.paused
    {
        progress.paused = true;
        crate::waits::changed(d, task);
    }
}

pub(super) fn resume(d: &mut Domain, env: &Env<Limits>, task: u64, repository: Repository) {
    let available = connected(d, repository.forge);
    if let Some(progress) = d.landing.progress.get_mut(&task) {
        let was_paused = progress.paused;
        access(progress, env, repository, available);
        if was_paused != progress.paused {
            crate::waits::changed(d, task);
        }
    }
}

pub(super) fn connection(d: &mut Domain, env: &Env<Limits>, forge: u16, available: bool) {
    let mut affected = List::with_capacity(d.landing.progress.len());
    for (task, progress) in &d.landing.progress {
        if progress.repository.forge == forge {
            affected.push((*task, progress.repository)).expect("one clock per landing");
        }
    }
    for (task, repository) in affected.into_boxed() {
        if available
            && let Some(owner) = super::pending_for(d, task)
            && crate::domain::outage::reading(d, owner)
        {
            continue;
        }
        let progress = d.landing.progress.get_mut(&task).expect("affected clock");
        let was_paused = progress.paused;
        access(progress, env, repository, available);
        if was_paused != progress.paused {
            crate::waits::changed(d, task);
        }
    }
}

pub(super) fn released(d: &mut Domain, env: &Env<Limits>, row: &LandingRow) {
    advanced(d, env, row);
    if let Some(progress) = d.landing.progress.get_mut(&row.landing.task) {
        progress.run = None;
        progress.wait = None;
    }
}

pub(super) fn failed(d: &mut Domain, env: &Env<Limits>, row: &LandingRow, failure: Failure) {
    let task = row.landing.task;
    if !d.landing.progress.contains_key(&task) {
        advanced(d, env, row);
    }
    resume(d, env, task, failure.repository);
    if let Some(progress) = d.landing.progress.get_mut(&task) {
        progress.wait = Some(Wait::Read(failure));
    }
}

pub(crate) fn read_change(d: &Domain, task: u64) -> Option<u64> {
    if let Some(Wait::Read(failure)) = &d.landing.progress.get(&task)?.wait { failure.change } else { None }
}

pub(crate) fn reported_deadline(d: &Domain, task: u64) -> Option<Wall> {
    let until = bound(d, task)?;
    let clock = d.landing.progress.get(&task)?.clock;
    Some(crate::waits::step_bound(clock.wall, elapsed(until, clock.since)))
}

pub(super) fn repository_waiting(d: &Domain, task: u64) -> Option<Repository> {
    Some(d.landing.progress.get(&task)?.repository)
}

pub(super) fn restore_waits(d: &mut Domain, env: &Env<Limits>) {
    let mut waits = List::with_capacity(d.landing.rows.len());
    for (task, row) in &d.landing.rows {
        if let Some(on) = row.waiting
            && !super::matches_terminal(&row.landing.phase)
        {
            waits.push((*task, on)).expect("one wait per landing");
        }
    }
    for (task, on) in waits.into_boxed() {
        let row = d.landing.rows.get(&task).expect("restored landing").clone();
        policy_wait(d, env, &row, on, false);
    }
}

pub(super) fn policy_wait(d: &mut Domain, env: &Env<Limits>, row: &LandingRow, on: policy::Awaited, shared: bool) {
    use policy::Awaited;
    let task = row.landing.task;
    let top = super::round_effect_top(row);
    let (member, wait) = match on {
        Awaited::Ready { member } | Awaited::Judgement { member } => (member, None),
        Awaited::Release | Awaited::Turn | Awaited::Prepare => (top, None),
        Awaited::Read | Awaited::Prefixes => (top, Some(Wait::Poll)),
        Awaited::Ancestry { member } => (member, Some(Wait::Poll)),
        Awaited::Open | Awaited::Close | Awaited::Delete { .. } | Awaited::Post { .. } | Awaited::Record { .. } => {
            write_wait(d, row, on, top)
        }
        Awaited::Merge => {
            if pending_merge(d, row) {
                write_wait(d, row, on, top)
            } else {
                (top, Some(Wait::Poll))
            }
        }
        Awaited::Run { member, gate } | Awaited::Receipt { member, gate } => {
            let flight = gate_flight(d, row, member, gate);
            let exempt = if let Awaited::Run { .. } = on {
                if let Some(flight) = flight {
                    flight.task == task && flight.clock.is_some() && flight.outcome.is_none()
                } else {
                    false
                }
            } else {
                shared || if let Some(flight) = flight { flight.task != task } else { false }
            };
            (member, if exempt { None } else { Some(Wait::Poll) })
        }
    };
    let Some(progress) = d.landing.progress.get(&task) else { return };
    let repository = match (&wait, &progress.last) {
        (Some(Wait::Poll), Some(last)) => last.repository,
        _ => match row.landing.members.get(usize::from(member)) {
            Some(member) => repository(member),
            None => progress.repository,
        },
    };
    // A disconnected targeted read may leave no answer to change the clock's
    // provider. The wait's repository, rather than the train as a whole, owns it.
    resume(d, env, task, repository);
    d.landing.progress.get_mut(&task).expect("landing clock").wait = wait;
}

fn pending_merge(d: &Domain, row: &LandingRow) -> bool {
    for item in &row.entries {
        if item.operation == crate::Operation::Merge
            && super::result(d, row.landing.task, item.member, 0, super::operation(item.operation))
                == policy::Status::Pending
        {
            return true;
        }
    }
    false
}

pub(crate) fn gate_flight<'a>(
    d: &'a Domain,
    row: &LandingRow,
    member: u16,
    gate: u16,
) -> Option<&'a crate::runs::Follow> {
    let position = (row.landing.task, member, gate);
    let placed = row.landing.members.get(usize::from(member))?;
    let name = &d.train_gates(row, repository(placed)).get(usize::from(gate))?.name;
    let proof =
        super::proof_key(placed.gates.get(usize::from(gate))?.proof).or(match d.landing.keys.get(&position) {
            Some((_, _, key)) => Some(*key),
            None => None,
        })?;
    d.landing.flights.get(&super::gate_key(row, member, gate, name, proof)).or(d.landing.faults.get(&position))
}

fn write_wait(d: &Domain, row: &LandingRow, on: policy::Awaited, top: u16) -> (u16, Option<Wait>) {
    if let Some((member, words)) = wait_words(d, row, on) {
        (member.unwrap_or(top), Some(Wait::Write(words)))
    } else {
        (top, Some(Wait::Poll))
    }
}

pub(super) fn wait_words(d: &Domain, row: &LandingRow, on: policy::Awaited) -> Option<(Option<u16>, Box<[u8]>)> {
    use policy::Awaited;
    let mut words = List::uncapped();
    let (member, before, after): (Option<u16>, &[u8], &[u8]) = match on {
        Awaited::Release => (None, b"its release", b""),
        Awaited::Ready { member } => (Some(member), b"", b" to be ready"),
        Awaited::Turn => {
            append_words(&mut words, b"the turn");
            if let Some(holder) = d.turn_holder(row.landing.project)
                && holder != row.landing.task
            {
                append_words(&mut words, b", held by landing ");
                append_words(&mut words, Decimal::of(holder).as_bytes());
            }
            (None, b"", b"")
        }
        Awaited::Prepare => (None, b"the round's prepare job", b""),
        Awaited::Prefixes => (None, b"the prepared prefixes on the members' branches", b""),
        Awaited::Open => (None, b"the round's pull request to open", b""),
        Awaited::Close => (None, b"the round's pull request to close", b""),
        Awaited::Merge => (None, b"the turn's merge", b""),
        Awaited::Delete { member } => (Some(member), b"", b"'s branch in the landing to be deleted"),
        Awaited::Record { member } => (Some(member), b"the record of ", b"'s pull request as merged"),
        Awaited::Judgement { member } => (Some(member), b"", b"'s judgement at its prefix"),
        Awaited::Ancestry { member } => (Some(member), b"whether the branch holds ", b"'s head"),
        Awaited::Read => (None, b"a fresh read of its facts from the forge", b""),
        Awaited::Post { member, gate } | Awaited::Run { member, gate } | Awaited::Receipt { member, gate } => {
            let m = row.landing.members.get(usize::from(member))?;
            append_words(&mut words, b"gate ");
            if let Some(gate) = d.train_gates(row, repository(m)).get(usize::from(gate)) {
                append_words(&mut words, &gate.name);
            } else {
                append_words(&mut words, Decimal::of(u64::from(gate)).as_bytes());
            }
            let before: &[u8] = if let Awaited::Run { .. } = on {
                append_words(&mut words, b"'s run ");
                if let Some(flight) = gate_flight(d, row, member, gate)
                    && let Some(run) = &flight.run
                {
                    append_words(&mut words, Decimal::of(run.number).as_bytes());
                    append_words(&mut words, b" ");
                }
                b"at "
            } else if let Awaited::Receipt { .. } = on {
                b"'s receipt for "
            } else {
                b"'s status at "
            };
            (Some(member), before, b"'s prefix")
        }
    };
    append_words(&mut words, before);
    if let Some(member) = member {
        let m = row.landing.members.get(usize::from(member))?;
        append_words(&mut words, b"change ");
        append_words(&mut words, Decimal::of(m.change).as_bytes());
    }
    append_words(&mut words, after);
    Some((member, words.into_boxed()))
}

fn active(d: &Domain, row: &LandingRow) -> bool {
    if d.landing.held || row.room.is_some() {
        return false;
    }
    match row.landing.phase {
        policy::Phase::Held { .. } | policy::Phase::Landed | policy::Phase::Cancelled => false,
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. } => true,
    }
}

pub(super) fn bound(d: &Domain, task: u64) -> Option<Time> {
    let row = d.landing.rows.get(&task)?;
    if !active(d, row) {
        return None;
    }
    let progress = d.landing.progress.get(&task)?;
    if progress.paused || progress.wait.is_none() {
        return None;
    }
    Some(progress.clock.until)
}

// A live gather can run out of time without an answer (including before
// admission); gathers polling an exempt wait leave that wait's clock alone.
fn gather_bound(d: &Domain, p: &Pending) -> Option<Time> {
    if p.entry == 0 || super::matches_stall(&p.stage) {
        return None;
    }
    if let Some(until) = bound(d, p.task) {
        return Some(until);
    }
    let row = d.landing.rows.get(&p.task)?;
    let progress = d.landing.progress.get(&p.task)?;
    if !active(d, row) || row.waiting.is_some() || progress.paused {
        return None;
    }
    if row.landing.phase == policy::Phase::Preparing
        && let Some(job) = &row.job
        && job.step.outcome.is_none()
    {
        return None;
    }
    Some(progress.clock.until)
}

pub(super) fn armed(d: &Domain, task: u64) -> Option<Time> {
    if let Some(owner) = super::pending_for(d, task)
        && let Some(p) = d.landing.pending.get(&owner)
        && p.entry != 0
    {
        return gather_bound(d, p);
    }
    if d.landing.due.contains_key(&task) && !d.landing.decisions.contains_key(&task) {
        return bound(d, task);
    }
    None
}

pub(super) fn deadline(d: &Domain) -> Option<Time> {
    if d.landing.pending.is_empty() && d.landing.due.is_empty() {
        return None;
    }
    let mut next: Option<Time> = None;
    for (task, _) in &d.landing.progress {
        if let Some(until) = armed(d, *task) {
            next = Some(match next {
                Some(at) => at.min(until),
                None => until,
            });
        }
    }
    next
}

pub(super) fn fire(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if d.landing.pending.is_empty() && d.landing.due.is_empty() {
        return;
    }
    let mut due = List::with_capacity(d.landing.progress.len());
    for (task, _) in &d.landing.progress {
        if let Some(until) = armed(d, *task)
            && until <= env.now
        {
            due.push(*task).expect("one clock per landing");
        }
    }
    for task in due.into_boxed() {
        d.landing.due.remove(&task);
        if let Some(owner) = super::pending_for(d, task)
            && let Some(p) = d.landing.pending.get(&owner)
            && p.entry != 0
        {
            let p = d.landing.pending.remove(&owner).expect("due gather");
            let row = d.landing.rows.get(&task).expect("live gather");
            let reason = if let Some(Wait::Write(_)) = &d.landing.progress.get(&task).expect("due clock").wait {
                super::ReadStep::Stalled
            } else {
                super::ReadStep::Failed(Failure::pending(row, &p, Cause::Pending))
            };
            super::read_ready(d, task, reason, out);
            d.read_line.leave(Waiter::Landing(owner));
        } else {
            emit(out, Request::WakeProcedure { task });
        }
    }
}

pub(crate) fn clear(d: &mut Domain, task: u64) {
    d.landing.progress.remove(&task);
}

fn waiting_ends(decision: policy::Decision) -> bool {
    if let policy::Decision::Wait { on, .. } = decision {
        return on == policy::Awaited::Release;
    }
    true
}

pub(super) fn decided(
    d: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    decision: policy::Decision,
    out: &mut Queue<Request>,
) {
    let Some(progress) = d.landing.progress.get_mut(&task) else { return };
    if let policy::Decision::Wait { on: policy::Awaited::Read, .. } = decision {
        let Some(Wait::Read(last)) = &progress.wait else { return };
        if let Some(run) = &mut progress.run {
            run.last = Failure { read: last.read.clone(), ..*last };
        } else if let Some(row) = d.landing.rows.get(&task) {
            let round = row.landing.preparation;
            note(task, round, last.reading(), Note::Started { why: last.cause.words() }, out);
            progress.run = Some(Run { round, since: env.now, last: Failure { read: last.read.clone(), ..*last } });
        }
        return;
    }
    if waiting_ends(decision) {
        progress.wait = None;
    }
    if let Some(run) = progress.run.take()
        && decision != (policy::Decision::Wait { until: None, on: policy::Awaited::Release })
        && decision != policy::Decision::Cancel
        && decision != policy::Decision::Finish
    {
        note(task, run.round, run.last.reading(), Note::Read { elapsed: elapsed(env.now, run.since) }, out);
    }
}

pub(super) fn held(d: &mut Domain, env: &Env<Limits>, row: &mut LandingRow, out: &mut Queue<Request>) {
    let progress = d.landing.progress.get_mut(&row.landing.task).expect("expired clock");
    let (reading, why) = match progress.wait.take() {
        Some(Wait::Write(words)) => (words, Cause::Pending),
        Some(Wait::Read(failure)) => (failure.reading(), failure.cause),
        Some(Wait::Poll) | None => match &progress.last {
            Some(last) => (last.reading(), last.cause),
            None => (Box::from(b"a fresh read of its facts from the forge".as_slice()), Cause::Pending),
        },
    };
    hold(progress, env, row, reading, &why.words(), out);
}

pub(super) fn room_held(d: &mut Domain, env: &Env<Limits>, row: &mut LandingRow, why: &[u8], out: &mut Queue<Request>) {
    let reading = Box::from(b"room in the forge's outbox".as_slice());
    let progress = d.landing.progress.get_mut(&row.landing.task).expect("room wait's landing clock");
    hold(progress, env, row, reading, why, out);
}

fn hold(
    progress: &mut Progress,
    env: &Env<Limits>,
    row: &mut LandingRow,
    reading: Box<[u8]>,
    why: &[u8],
    out: &mut Queue<Request>,
) {
    let elapsed = elapsed(env.now, progress.clock.since);
    let mut words = List::with_capacity(crate::history::REASON_BYTES);
    for part in [b"read: ".as_slice(), &reading, b": ", why] {
        append_words(&mut words, part);
    }
    if progress.reads != 0 {
        let minutes = progress.clock.wall.as_nanos() / 60_000_000_000;
        append_words(&mut words, b"; ");
        append_words(&mut words, Decimal::of(progress.reads).as_bytes());
        append_words(&mut words, b" reads since ");
        two_digits(&mut words, (minutes / 60) % 24);
        append_words(&mut words, b":");
        two_digits(&mut words, minutes % 60);
        append_words(&mut words, b"Z, none advanced the round");
    }
    let words = words.into_boxed();
    row.read_hold = Some(words.clone());
    progress.run = None;
    note(row.landing.task, row.landing.preparation, reading, Note::Held { elapsed, words }, out);
}

fn two_digits(out: &mut List<u8>, value: u64) {
    if value < 10 {
        append_words(out, b"0");
    }
    append_words(out, Decimal::of(value).as_bytes());
}

// Access checks in the parent's read path don't expose the client's outage state.
pub(crate) fn unavailable(d: &mut Domain, owner: Token) {
    if let Some(p) = d.landing.pending.get(&owner) {
        pause(d, p.task);
    }
}
