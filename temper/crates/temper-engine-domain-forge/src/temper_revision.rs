//! Whole reads of the project's home (temper/docs/design/domain/forge/temper-dir.md, 2–3).
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_lib::{Decimal, Env, List, Map, Queue, Time, Token, bytes, len_u64};
use temper_protocol_temper_dir::{Home, Reading, Touch, Verdict};

use crate::{Domain, Key, Limits, Request, Role, Stored, TemperFile, TemperRefusal, TemperRevision, effect_words};

const WORDS: usize = 1024;

#[derive(Hash, Debug)]
struct Pending {
    project: u32,
    home: Repository,
    branch: Box<[u8]>,
    observed: u64,
    tree: Option<Tree>,
    ask: Option<Read>,
    poll: Option<Movement>,
    again: bool,
}

#[derive(Hash, Debug)]
struct Tree {
    commit: Commit,
    reading: Reading,
}

#[derive(Clone, Copy, Hash, Debug)]
struct Watch {
    home: Repository,
    tip: Commit,
    at: Time,
}

#[derive(Hash, Debug)]
enum Movement {
    Tip(Commit),
    Paths { before: Commit, after: Commit },
}

#[derive(Hash, Debug)]
struct Retry {
    home: Repository,
    at: Time,
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) rows: Map<u32, TemperRevision>,
    provisional: Map<Repository, Token>,
    stamped: Map<u32, u64>,
    reads: Map<Token, Pending>,
    retries: Map<u32, Retry>,
    watches: Map<u32, Watch>,
}

impl State {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            rows: Map::with_capacity(limits.judge_projects),
            provisional: Map::with_capacity(limits.repositories),
            stamped: Map::with_capacity(limits.judge_projects),
            reads: Map::with_capacity(limits.judge_projects),
            retries: Map::with_capacity(limits.judge_projects),
            watches: Map::with_capacity(limits.judge_projects),
        }
    }

    pub(crate) fn reading(&self, owner: Token) -> bool {
        self.reads.contains_key(&owner)
    }

    fn observed(&self, project: u32) -> u64 {
        match self.rows.get(&project) {
            Some(row) => row.observed,
            None => 0,
        }
    }
}

impl Domain {
    /// The retained repository revision, including observations before the first good read.
    #[must_use]
    pub fn temper_revision(&self, project: u32) -> Option<&TemperRevision> {
        self.temper_revisions.rows.get(&project)
    }
}

pub(crate) fn provisional(d: &mut Domain, repository: Repository, request: Token) {
    d.temper_revisions.provisional.insert(repository, request).expect("one mark per adopted repository");
}

pub(crate) fn forget(d: &mut Domain, repository: Repository) {
    d.temper_revisions.provisional.remove(&repository);
}

pub(crate) fn adopted(d: &mut Domain, env: &Env<Limits>, home: Repository, request: Token) {
    if d.temper_revisions.provisional.get(&home) != Some(&request) {
        return;
    }
    forget(d, home);
    let Some(row) = d.repository(home) else { return };
    if row.home && row.role != Role::Context {
        read(d, env, row.project, None);
    } else {
        retire(d, home);
    }
}

fn home(d: &Domain, project: u32) -> Option<&crate::Repository> {
    let row = d.home(project)?;
    d.configured(row.provider.forge)?;
    if row.role == Role::Context || d.temper_revisions.provisional.contains_key(&row.provider) {
        return None;
    }
    Some(row)
}

fn is_home(d: &Domain, project: u32, repository: Repository) -> bool {
    if let Some(row) = home(d, project) {
        return row.provider == repository;
    }
    false
}

pub(crate) fn restart(d: &mut Domain, env: &Env<Limits>) {
    let mut projects = List::with_capacity(d.repositories.len());
    for (_, row) in &d.repositories {
        if row.home && !projects.as_slice().contains(&row.project) {
            projects.push(row.project).expect("one project per home");
        }
    }
    for project in projects.into_boxed() {
        read(d, env, project, None);
    }
}

pub(crate) fn watches_branch(d: &Domain, repository: Repository, branch: &[u8]) -> bool {
    let Some(row) = d.repository(repository) else { return false };
    row.landing.as_ref() == branch && is_home(d, row.project, repository)
}

pub(crate) fn landed(d: &mut Domain, env: &Env<Limits>, repository: Repository, branch: &[u8], tip: Option<Commit>) {
    if watches_branch(d, repository, branch) {
        let project = d.repository(repository).expect("watched home").project;
        read(d, env, project, tip);
    }
}

pub(crate) fn touched(files: Option<&[Box<[u8]>]>, behind: bool) -> bool {
    if behind {
        return true;
    }
    let Some(files) = files else { return true };
    match temper_protocol_temper_dir::touched(files, true) {
        Touch::Method | Touch::Temper | Touch::Unknown => true,
        Touch::None => false,
    }
}

fn read(d: &mut Domain, env: &Env<Limits>, project: u32, tip: Option<Commit>) {
    queue(d, env, project, tip, None);
}

fn queue(d: &mut Domain, env: &Env<Limits>, project: u32, tip: Option<Commit>, poll: Option<Movement>) {
    if d.stopping {
        return;
    }
    let Some(row) = home(d, project) else { return };
    for (owner, pending) in &d.temper_revisions.reads {
        if pending.project == project {
            let owner = *owner;
            d.temper_revisions.reads.get_mut(&owner).expect("in-flight project read").again = true;
            return;
        }
    }
    let repository = row.provider;
    let branch = row.landing.clone();
    let Some(sequence) = d.sequence.checked_add(1) else { return };
    if sequence >= 1_u64 << 60_u32 {
        return;
    }
    if !d.temper_revisions.rows.contains_key(&project)
        && d.temper_revisions.rows.len() == d.temper_revisions.rows.capacity()
    {
        return;
    }
    let last = d.temper_revisions.stamped.get(&project).copied().unwrap_or(0).max(d.temper_revisions.observed(project));
    let Some(observed) = last.checked_add(1) else { return };
    if d.temper_revisions.stamped.insert(project, observed).is_err() {
        return;
    }
    d.temper_revisions.retries.remove(&project);
    d.temper_revisions.watches.remove(&project);
    d.sequence = sequence;
    let owner = Token::new(sequence | (1_u64 << 60_u32));
    let (tree, read) = match tip {
        Some(commit) => {
            let tree = tree(env, commit);
            let read = tree.reading.next().expect("a new tree asks for its root");
            (Some(tree), read)
        }
        None => (None, Read::Branch { branch: branch.clone() }),
    };
    let pending = Pending { project, home: repository, branch, observed, tree, ask: Some(read), poll, again: false };
    d.temper_revisions.reads.insert(owner, pending).expect("one read per observed project");
}

fn tree(env: &Env<Limits>, commit: Commit) -> Tree {
    Tree { commit, reading: Reading::new(commit, env.limits.client.answer_bytes) }
}

fn next(d: &Domain) -> Option<Token> {
    if !d.room_for_revision_read() {
        return None;
    }
    let mut next = None;
    for (owner, pending) in &d.temper_revisions.reads {
        pending.ask.as_ref()?;
        if next.is_none() {
            next = Some(*owner);
        }
    }
    next
}

pub(crate) fn ready(d: &Domain) -> bool {
    next(d).is_some()
}

pub(crate) fn resume(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(owner) = next(d) else { return };
    let ready = d.temper_revisions.reads.get(&owner).expect("waiting read");
    if !watches_branch(d, ready.home, &ready.branch) {
        d.temper_revisions.reads.remove(&owner);
        return;
    }
    let pending = d.temper_revisions.reads.get_mut(&owner).expect("waiting read");
    let repository = pending.home;
    let read = pending.ask.take().expect("waiting read has an ask");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}

pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.temper_revisions.reads.remove(&owner) else { return };
    let project = pending.project;
    let same = watches_branch(d, pending.home, &pending.branch)
        && d.repository(pending.home).expect("watched home").project == project;
    if !d.stopping && same && pending.observed > d.temper_revisions.observed(project) {
        let result = match pending.poll.take() {
            Some(Movement::Tip(before)) => match result {
                Ok(Answer::Commit(after)) if before == after => {
                    rearm(d, env, &pending, after);
                    return;
                }
                Ok(Answer::Commit(after)) => {
                    pending.ask = Some(Read::Compare { before, after });
                    pending.poll = Some(Movement::Paths { before, after });
                    d.temper_revisions.reads.insert(owner, pending).expect("poll retains its place");
                    return;
                }
                other => other,
            },
            Some(Movement::Paths { before, after }) => {
                if let Ok(Answer::Compare { before: from, after: to, changed, files, behind, .. }) = &result
                    && *from == before
                    && *to == after
                    && *changed == len_u64(files.len())
                    && !touched(Some(files), *behind > 0)
                {
                    rearm(d, env, &pending, after);
                    return;
                }
                Ok(Answer::Commit(after))
            }
            None => result,
        };
        let mut error = None;
        if let Some(tree) = &mut pending.tree {
            if tree.reading.take(&result).is_err() {
                error = Some(result.err().unwrap_or(Error::InvalidAnswer));
            }
        } else if let Ok(Answer::Commit(commit)) = result {
            pending.tree = Some(tree(env, commit));
        } else {
            error = Some(result.err().unwrap_or(Error::InvalidAnswer));
        }
        if let Some(error) = error {
            let commit = match &pending.tree {
                Some(tree) => Some(tree.commit),
                None => None,
            };
            take(d, &pending, commit, Err(effect_words::error_words(error)), out);
            if !pending.again {
                let retry = Retry { home: pending.home, at: env.now.saturating_add(env.limits.client.backoff) };
                d.temper_revisions.retries.insert(project, retry).expect("one retry per observed project");
            }
        } else if let Some(tree) = &pending.tree {
            if let Some(next) = tree.reading.next() {
                pending.ask = Some(next);
                d.temper_revisions.reads.insert(owner, pending).expect("read retains its place");
                return;
            }
            let judged = tree.reading.judge(Home::Here);
            let tip = tree.commit;
            take(d, &pending, Some(tip), judgement(&tree.reading, &judged), out);
            if judged.method {
                watch(d, env, &pending, tip);
            }
        }
    }
    if pending.again {
        read(d, env, project, None);
    }
}

struct Good {
    method: bool,
    files: Box<[TemperFile]>,
}

fn judgement(reading: &Reading, verdict: &Verdict) -> Result<Good, Box<[u8]>> {
    if verdict.method
        && let Some(first) = verdict.faults.first()
    {
        let after = len_u64(verdict.faults.len()).saturating_add(u64::from(verdict.more)).saturating_sub(1);
        let count = Decimal::of(after);
        let suffix = if after == 0 { Box::default() } else { bytes::concat(&[b" (and ", count.as_bytes(), b" more)"]) };
        let line = Decimal::of(u64::from(first.line));
        let number = if first.line == 0 { Box::default() } else { bytes::concat(&[b":", line.as_bytes()]) };
        let path_room = WORDS.saturating_sub(suffix.len()).saturating_sub(number.len()).saturating_sub(2);
        let path = prefix(&first.path, path_room);
        let why = prefix(&first.why, path_room.saturating_sub(path.len()));
        return Err(bytes::concat(&[path, &number, b": ", why, &suffix]));
    }
    let method_files = reading.method_files();
    let mut files = List::with_capacity(u32::try_from(method_files.len()).expect("bounded method files"));
    for file in method_files {
        files.push(TemperFile { path: file.path, bytes: file.bytes }).expect("one row per method file");
    }
    Ok(Good { method: verdict.method, files: files.into_boxed() })
}

fn prefix(words: &[u8], room: usize) -> &[u8] {
    words.get(..words.len().min(room)).expect("prefix is within the slice")
}

fn take(
    d: &mut Domain,
    pending: &Pending,
    commit: Option<Commit>,
    result: Result<Good, Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    let project = pending.project;
    let rows = &mut d.temper_revisions.rows;
    let mut row = rows.remove(&project).unwrap_or(TemperRevision {
        project,
        number: 0,
        home: pending.home,
        commit: None,
        method: false,
        files: Box::default(),
        observed: 0,
        refused: None,
    });
    row.observed = pending.observed;
    match result {
        Ok(good) => {
            if row.number == 0 || row.home != pending.home || row.method != good.method || row.files != good.files {
                row.number = row.number.saturating_add(1);
                row.home = pending.home;
                row.commit = commit;
                row.method = good.method;
                row.files = good.files;
            }
            row.refused = None;
        }
        Err(words) => {
            let words = if words.len() > WORDS { bytes::copy_of(prefix(&words, WORDS)) } else { words };
            row.refused = Some(TemperRefusal { commit, observed: row.observed, words });
        }
    }
    rows.insert(project, row.clone()).expect("one revision per observed project");
    out.push(Request::Save { record: Stored::TemperRevision(row), read_afresh: None, release: None });
}

// Only a home known to have method has its own branch observer. Its tip
// does not consume a shared watch place or write the task branch-head cache.
fn watch(d: &mut Domain, env: &Env<Limits>, pending: &Pending, tip: Commit) {
    let at = d.poll_at(env, pending.home);
    d.temper_revisions
        .watches
        .insert(pending.project, Watch { home: pending.home, tip, at })
        .expect("at most one watch per revision");
}

fn rearm(d: &mut Domain, env: &Env<Limits>, pending: &Pending, tip: Commit) {
    watch(d, env, pending, tip);
    if pending.again {
        read(d, env, pending.project, None);
    }
}

pub(crate) fn hinted(d: &mut Domain, env: &Env<Limits>, hint: &skein_forge::api::Hint) {
    let branch = match &hint.change {
        skein_forge::api::Change::Branch(branch) | skein_forge::api::Change::BranchMoved { branch, .. } => branch,
        skein_forge::api::Change::Item(_) | skein_forge::api::Change::Commit(_) => return,
    };
    if !watches_branch(d, hint.repository, branch) {
        return;
    }
    let project = d.repository(hint.repository).expect("home branch").project;
    if let Some(watch) = d.temper_revisions.watches.get_mut(&project) {
        watch.at = watch.at.min(env.now.saturating_add(env.limits.client.hinted));
    }
}

pub(crate) fn deadline(d: &Domain) -> Option<Time> {
    let mut next: Option<Time> = None;
    for (_, watch) in &d.temper_revisions.watches {
        next = skein_lib::earliest(next, Some(watch.at));
    }
    for (_, retry) in &d.temper_revisions.retries {
        next = Some(match next {
            Some(at) => at.min(retry.at),
            None => retry.at,
        });
    }
    next
}

pub(crate) fn fire(d: &mut Domain, env: &Env<Limits>) {
    let mut due = List::with_capacity(d.temper_revisions.retries.len());
    for (project, retry) in &d.temper_revisions.retries {
        if retry.at <= env.now {
            due.push((*project, retry.home)).expect("one retry per project");
        }
    }
    for (project, repository) in due.into_boxed() {
        d.temper_revisions.retries.remove(&project);
        if is_home(d, project, repository) {
            read(d, env, project, None);
        }
    }
    let mut due = List::with_capacity(d.temper_revisions.watches.len());
    for (project, watch) in &d.temper_revisions.watches {
        if watch.at <= env.now {
            due.push((*project, *watch)).expect("one poll per project");
        }
    }
    for (project, watch) in due.into_boxed() {
        d.temper_revisions.watches.remove(&project);
        if is_home(d, project, watch.home) {
            queue(d, env, project, None, Some(Movement::Tip(watch.tip)));
        }
    }
}

pub(crate) fn retire(d: &mut Domain, repository: Repository) {
    forget(d, repository);
    let mut watched = None;
    for (project, watch) in &d.temper_revisions.watches {
        if watch.home == repository {
            watched = Some(*project);
            break;
        }
    }
    if let Some(project) = watched {
        d.temper_revisions.watches.remove(&project);
    }
    let mut reads = List::with_capacity(d.temper_revisions.reads.len());
    for (owner, pending) in &d.temper_revisions.reads {
        if pending.home == repository {
            reads.push(*owner).expect("one owner per read");
        }
    }
    for owner in reads.into_boxed() {
        d.temper_revisions.reads.remove(&owner);
    }
    let mut retries = List::with_capacity(d.temper_revisions.retries.len());
    for (project, retry) in &d.temper_revisions.retries {
        if retry.home == repository {
            retries.push(*project).expect("one retry per project");
        }
    }
    for project in retries.into_boxed() {
        d.temper_revisions.retries.remove(&project);
    }
}

pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    d.temper_revisions.watches.remove(&project);
    d.temper_revisions.stamped.remove(&project);
    d.temper_revisions.retries.remove(&project);
    let mut reading = None;
    for (owner, pending) in &d.temper_revisions.reads {
        if pending.project == project {
            reading = Some(*owner);
            break;
        }
    }
    if let Some(owner) = reading {
        d.temper_revisions.reads.remove(&owner);
    }
    if d.temper_revisions.rows.remove(&project).is_some() {
        out.push(Request::Erase { key: Key::TemperRevision { project } });
    }
}
