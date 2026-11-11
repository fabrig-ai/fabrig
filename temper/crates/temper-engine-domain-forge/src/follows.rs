//! Hearing source landings (temper/docs/design/domain/forge/follows.md, 3, 7 and 9).
use crate::{Domain, Key, Limits, Name, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::{self, Commit, Repository};
use skein_lib::{Env, List, Queue, Time, Token};
mod acts;
pub mod words;
pub use acts::FollowAct;
pub(crate) use acts::asked;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowRow {
    pub project: u32,
    pub follow: u32,
    pub source: Repository,
    pub branch: Box<[u8]>,
    pub heard: Option<Commit>,
    pub task: Option<u64>,
    pub stopped: Option<Box<[u8]>>,
    pub queue: Box<[FollowEntry]>,
    pub skips: Box<[FollowSkip]>,
    pub parity: Option<FollowParity>,
    pub questions: Box<[FollowQuestion]>,
    pub pending: Option<FollowAct>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowEntry {
    pub before: Commit,
    pub after: Commit,
    pub pull: Option<u64>,
    pub commits: Box<[FollowCommit]>,
    pub more_commits: u32,
    pub files: Option<Box<[Box<[u8]>]>>,
    pub more_files: u32,
    pub change: Option<u64>,
    pub state: FollowState,
    pub word: bool,
    pub remake: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowQuestion {
    pub change: u64,
    pub number: u64,
    pub words: Box<[u8]>,
    pub person: Option<u64>,
    pub answer: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowCommit {
    pub id: Commit,
    pub message: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowState {
    NotMade,
    WaitsWord,
    HeldByOwner(Box<[u8]>),
    WaitsFor(u64),
    Producing,
    Held(Box<[u8]>),
    Ended(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowSkip {
    pub after: Commit,
    pub reason: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowParity {
    pub source: Commit,
    pub head: Commit,
    pub verdict: FollowVerdict,
    pub run: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowVerdict {
    Passed,
    Failing(u32),
    Many,
}
/// Pure snapshot, in the queue's oldest-first order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowLag {
    pub count: u32,
    pub entries: Box<[FollowEntry]>,
}

#[derive(Hash, Debug)]
pub(crate) enum Pending {
    Baseline { key: (u32, u32) },
    Range(Range),
}
impl Pending {
    const fn key(&self) -> (u32, u32) {
        match self {
            Self::Baseline { key } => *key,
            Self::Range(range) => range.key,
        }
    }
}
#[derive(Hash, Debug)]
pub(crate) struct Range {
    key: (u32, u32),
    before: Commit,
    after: Commit,
    files: Option<Box<[Box<[u8]>]>>,
    commits: List<FollowCommit>,
    more_commits: u32,
    page: u32,
}
pub(crate) fn capacity(l: &Limits) -> Option<u32> {
    l.judge_projects.checked_mul(l.follows)
}
/// Refusals of the domain's owner-word entrance (follows.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FollowRefusal {
    UnknownFollow,
    Stopped,
    UnknownEntry,
    Skipped,
    AlreadyStarted,
    NotConsecutive,
    MergeBound,
    ReasonBound,
    NoWord,
    Pending,
    NotOwner,
}
impl Domain {
    pub fn follow_start(&self, project: u32, follow: u32, after: Commit) -> Result<FollowRow, FollowRefusal> {
        let mut row = self.follow_row(project, follow).cloned().ok_or(FollowRefusal::UnknownFollow)?;
        if row.stopped.is_some() {
            return Err(FollowRefusal::Stopped);
        }
        let mut found = false;
        for entry in &mut row.queue {
            if entry.after != after {
                continue;
            }
            found = true;
            match &entry.state {
                FollowState::Ended(_) => {
                    entry.remake = true;
                    entry.word = true;
                }
                FollowState::NotMade
                | FollowState::WaitsWord
                | FollowState::WaitsFor(_)
                | FollowState::HeldByOwner(_) => {
                    entry.word = true;
                    entry.state = FollowState::NotMade;
                }
                FollowState::Producing | FollowState::Held(_) => return Err(FollowRefusal::AlreadyStarted),
            }
        }
        if found {
            return Ok(row);
        }
        for skip in &row.skips {
            if skip.after == after {
                return Err(FollowRefusal::Skipped);
            }
        }
        Err(FollowRefusal::UnknownEntry)
    }
    #[must_use]
    pub fn follow_task(&self, task: u64) -> Option<&FollowRow> {
        let mut found = None;
        for (_, row) in &self.follow_rows {
            if row.task == Some(task) {
                found = Some(row);
                break;
            }
        }
        found
    }
    #[must_use]
    pub fn follow_row(&self, project: u32, follow: u32) -> Option<&FollowRow> {
        self.follow_rows.get(&(project, follow))
    }
    #[must_use]
    pub fn follow_lag(&self, project: u32, follow: u32) -> FollowLag {
        let entries = match self.follow_row(project, follow) {
            Some(row) => row.queue.clone(),
            None => Box::new([]),
        };
        FollowLag { count: u32::try_from(entries.len()).expect("bounded follow queue"), entries }
    }
}
pub(crate) fn register(d: &mut Domain, row: FollowRow, out: &mut Queue<Request>) {
    let key = (row.project, row.follow);
    let Some(prior) = d.follow_rows.get(&key) else { return };
    if prior == &row {
        return;
    }
    for entry in &row.queue {
        if let Some(change) = entry.change {
            out.push(Request::WakeProcedure { task: change });
        }
    }
    save(&row, out);
    d.follow_rows.insert(key, row).expect("existing follow");
}
fn save(row: &FollowRow, out: &mut Queue<Request>) {
    if let Some(task) = row.task {
        out.push(Request::WakeProcedure { task });
    }
    out.push(Request::Save { read_afresh: None, release: None, record: Stored::Follow(row.clone()) });
    out.push(Request::Marked { tasks: Box::new([crate::follows_subject(row.project)]) });
}
/// Remove followed entries; resume admits the next range once there is room.
pub fn followed(d: &mut Domain, project: u32, follow: u32, afters: &[Commit], out: &mut Queue<Request>) {
    let Some(row) = d.follow_rows.get_mut(&(project, follow)) else { return };
    let mut queue = List::with_capacity(u32::try_from(row.queue.len()).expect("bounded queue"));
    for entry in &row.queue {
        if !afters.contains(&entry.after) {
            queue.push(entry.clone()).expect("remaining queue");
        }
    }
    row.queue = queue.into_boxed();
    save(row, out);
}
pub(crate) fn keep(d: &mut Domain, project: u32, follow: u32, out: &mut Queue<Request>) {
    let key = (project, follow);
    if let Some(setting) = d.follow(project, follow).cloned() {
        if let Some(row) = d.follow_rows.get_mut(&key) {
            if row.stopped.take().is_some() {
                row.task = None;
            }
            save(row, out);
        } else {
            let row = FollowRow {
                project,
                follow,
                source: Repository {
                    forge: setting.source.repository.forge,
                    repository: setting.source.repository.repository,
                },
                branch: setting.source.branch.clone(),
                heard: None,
                task: None,
                stopped: None,
                queue: Box::new([]),
                skips: Box::new([]),
                parity: None,
                questions: Box::new([]),
                pending: None,
            };
            save(&row, out);
            d.follow_rows.insert(key, row).expect("accepted follow bound");
        }
    } else if let Some(row) = d.follow_rows.remove(&key) {
        if let Some(task) = row.task {
            out.push(Request::CancelFollow { project, task });
        }
        d.follow_retry.remove(&key);
        let mut owners = List::with_capacity(d.follow_reads.capacity());
        for (owner, read) in &d.follow_reads {
            if read.key() == key {
                owners.push(*owner).expect("follow reads");
            }
        }
        for owner in owners.into_boxed() {
            d.follow_reads.remove(&owner);
        }
        out.push(Request::Erase { key: Key::Follow { project, follow } });
        out.push(Request::Marked { tasks: Box::new([crate::follows_subject(project)]) });
    }
    if let Some(row) = d.follow_rows.get(&key)
        && row.task.is_none()
        && row.stopped.is_none()
    {
        out.push(Request::FollowRun { project, follow });
    }
    d.tier_ops.dirty = true;
}
pub(crate) fn branch<'a>(d: &'a Domain, row: &'a FollowRow) -> Option<&'a [u8]> {
    if row.branch.is_empty() { Some(&d.repository(row.source)?.settings.default_branch) } else { Some(&row.branch) }
}
pub(crate) fn watches(d: &Domain, names: &mut List<Name>) {
    for (_, row) in &d.follow_rows {
        if !d.forge_ended(row.source.forge)
            && let Some(branch) = branch(d, row)
        {
            let name = crate::tiers::name(row.source, branch);
            if !names.as_slice().contains(&name) {
                names.push(name).expect("one watch per follow");
            }
        }
    }
}
pub(crate) fn observed(d: &mut Domain, repository: Repository, branch: &[u8], after: Commit, out: &mut Queue<Request>) {
    let mut keys = List::with_capacity(d.follow_rows.capacity());
    for (key, row) in &d.follow_rows {
        if row.source == repository && self::branch(d, row) == Some(branch) {
            keys.push(*key).expect("follow keys");
        }
    }
    let source = !keys.is_empty();
    for key in keys.into_boxed() {
        d.follow_retry.remove(&key);
        let row = d.follow_rows.get_mut(&key).expect("collected follow");
        if row.heard.is_none() {
            row.heard = Some(after);
            save(row, out);
        }
    }
    if source {
        let name = crate::tiers::name(repository, branch);
        if let Some(head) = d.heads.get_mut(&name)
            && head.commit != after
        {
            head.commit = after;
            out.push(Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(head.clone()) });
        }
    }
}
fn tip(d: &Domain, row: &FollowRow) -> Option<Commit> {
    Some(d.heads.get(&crate::tiers::name(row.source, branch(d, row)?))?.commit)
}
fn next(d: &Domain) -> Option<((u32, u32), Option<Commit>)> {
    for (key, row) in &d.follow_rows {
        if d.forge_ended(row.source.forge) || row.queue.len() >= skein_lib::usize_of(d.follow_queue) {
            continue;
        }
        let mut reading = false;
        for (_, pending) in &d.follow_reads {
            if pending.key() == *key {
                reading = true;
            }
        }
        if reading {
            continue;
        }
        if row.heard.is_none() {
            if !d.follow_retry.contains_key(key) && branch(d, row).is_some() {
                return Some((*key, None));
            }
        } else if let Some(after) = tip(d, row)
            && row.heard != Some(after)
        {
            return Some((*key, Some(after)));
        }
    }
    None
}
pub(crate) fn waiting(d: &Domain) -> bool {
    next(d).is_some()
}
pub(crate) fn resume(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let next = next(d);
    let Some((key, after)) = next else { return };
    let row = d.follow_rows.get(&key).expect("collected follow");
    let repository = row.source;
    d.sequence = d.sequence.checked_add(1).expect("follow read sequence");
    let owner = Token::new(d.sequence | (1_u64 << 63));
    let (pending, read) = match after {
        None => (
            Pending::Baseline { key },
            api::Read::Branch { branch: Box::from(branch(d, row).expect("adopted source branch")) },
        ),
        Some(after) => {
            let before = row.heard.expect("range has a baseline");
            (
                Pending::Range(Range {
                    key,
                    before,
                    after,
                    files: None,
                    commits: List::with_capacity(env.limits.follow_commits),
                    more_commits: 0,
                    page: 0,
                }),
                api::Read::Compare { before, after },
            )
        }
    };
    d.follow_reads.insert(owner, pending).expect("one read per follow");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}
pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(read) = d.follow_reads.remove(&owner) else { return };
    let mut pending = match read {
        Pending::Range(range) => range,
        Pending::Baseline { key } => {
            let Some(row) = d.follow_rows.get(&key) else { return };
            if row.heard.is_none() {
                if let Ok(api::Answer::Commit(after)) = result {
                    let repository = row.source;
                    let branch = Box::from(branch(d, row).expect("read's source branch"));
                    observed(d, repository, &branch, after, out);
                } else {
                    d.follow_retry
                        .insert(key, env.now.saturating_add(env.limits.client.backoff))
                        .expect("one retry per follow");
                }
            }
            return;
        }
    };
    if !d.follow_rows.contains_key(&pending.key) {
        return;
    }
    if pending.page == 0 {
        pending.files = match result {
            Ok(api::Answer::Compare { before, after, files, changed, .. })
                if before == pending.before && after == pending.after && skein_lib::len_u64(files.len()) == changed =>
            {
                Some(files)
            }
            Ok(_) | Err(_) => None,
        };
        pending.page = 1;
        ask_commits(d, env, owner, pending, out);
        return;
    }
    match result {
        Ok(api::Answer::Commits { commits, more }) if !commits.is_empty() || !more => {
            for commit in commits {
                if pending.commits.len() == env.limits.follow_commits {
                    let mut newest = List::with_capacity(env.limits.follow_commits);
                    for commit in pending.commits.as_slice().get(1..).expect("full commits") {
                        newest.push(commit.clone()).expect("retained newest");
                    }
                    pending.commits = newest;
                    pending.more_commits = pending.more_commits.saturating_add(1);
                }
                let end = commit.message.len().min(skein_lib::usize_of(env.limits.follow_message_bytes));
                pending
                    .commits
                    .push(FollowCommit {
                        id: commit.id,
                        message: Box::from(commit.message.get(..end).expect("message prefix")),
                    })
                    .expect("newest commits");
            }
            if more && let Some(page) = pending.page.checked_add(1) {
                pending.page = page;
                ask_commits(d, env, owner, pending, out);
                return;
            }
            if more {
                pending.files = None;
            }
        }
        Ok(_) | Err(_) => pending.files = None,
    }
    finish_range(d, env, pending, out);
}
fn ask_commits(d: &mut Domain, env: &Env<Limits>, owner: Token, pending: Range, out: &mut Queue<Request>) {
    let repository = d.follow_rows.get(&pending.key).expect("read's follow").source;
    let read = api::Read::Commits { before: pending.before, after: pending.after, page: pending.page };
    d.follow_reads.insert(owner, Pending::Range(pending)).expect("replaces follow read");
    crate::domain::child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
}
fn finish_range(d: &mut Domain, env: &Env<Limits>, pending: Range, out: &mut Queue<Request>) {
    let Some(setting) = d.follow(pending.key.0, pending.key.1) else { return };
    let mut files = List::with_capacity(env.limits.follow_files);
    let mut more_files = 0_u32;
    let mut keep = setting.source.paths.is_empty() || pending.files.is_none();
    if let Some(known) = &pending.files {
        for file in known {
            if setting.source.paths.is_empty()
                || crate::domain::overlap(core::slice::from_ref(file), &setting.source.paths)
            {
                keep = true;
                if files.len() < env.limits.follow_files {
                    files.push(file.clone()).expect("bounded files");
                } else {
                    more_files = more_files.saturating_add(1);
                }
            }
        }
    }
    let mut pull = None;
    if let Some(task) = d.landed.get(&pending.after)
        && let Some(change) = d.changes.get(task)
        && change.repository == d.follow_rows.get(&pending.key).expect("read's follow").source
    {
        pull = change.pull;
    }
    let row = d.follow_rows.get_mut(&pending.key).expect("read's follow");
    if row.heard != Some(pending.before) {
        return;
    }
    let mut queued = false;
    for entry in &row.queue {
        if entry.after == pending.after {
            queued = true;
        }
    }
    if keep && !queued {
        let mut queue = List::with_capacity(env.limits.follow_queue);
        for entry in &row.queue {
            queue.push(entry.clone()).expect("existing queue");
        }
        queue
            .push(FollowEntry {
                before: pending.before,
                after: pending.after,
                pull,
                commits: pending.commits.into_boxed(),
                more_commits: pending.more_commits,
                files: match pending.files {
                    Some(_) => Some(files.into_boxed()),
                    None => None,
                },
                more_files,
                change: None,
                state: FollowState::NotMade,
                word: false,
                remake: false,
            })
            .expect("queue room reserved before read");
        row.queue = queue.into_boxed();
    }
    row.heard = Some(pending.after);
    save(row, out);
}
pub(crate) fn row_bytes(row: &FollowRow) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(row.branch.len());
    let pending = match &row.pending {
        Some(FollowAct::Hold { reason, .. } | FollowAct::Skip { reason, .. }) => skein_lib::len_u64(reason.len()),
        Some(FollowAct::Merge { afters }) => skein_lib::len_u64(afters.len()).checked_mul(32)?,
        Some(FollowAct::Start { .. }) | None => 0,
    };
    bytes = bytes.checked_add(pending)?;
    bytes = bytes.checked_add(
        skein_lib::len_u64(row.questions.len()).checked_mul(u64::try_from(size_of::<FollowQuestion>()).ok()?)?,
    )?;
    for question in &row.questions {
        bytes = bytes.checked_add(skein_lib::len_u64(question.words.len()))?;
        if let Some(answer) = &question.answer {
            bytes = bytes.checked_add(skein_lib::len_u64(answer.len()))?;
        }
    }
    if let Some(stopped) = &row.stopped {
        bytes = bytes.checked_add(skein_lib::len_u64(stopped.len()))?;
    }
    bytes = bytes
        .checked_add(skein_lib::len_u64(row.queue.len()).checked_mul(u64::try_from(size_of::<FollowEntry>()).ok()?)?)?;
    for entry in &row.queue {
        bytes = bytes.checked_add(
            skein_lib::len_u64(entry.commits.len()).checked_mul(u64::try_from(size_of::<FollowCommit>()).ok()?)?,
        )?;
        for commit in &entry.commits {
            bytes = bytes.checked_add(skein_lib::len_u64(commit.message.len()))?;
        }
        if let Some(files) = &entry.files {
            bytes = bytes.checked_add(
                skein_lib::len_u64(files.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
            )?;
            for file in files {
                bytes = bytes.checked_add(skein_lib::len_u64(file.len()))?;
            }
        }
        match &entry.state {
            FollowState::HeldByOwner(words) | FollowState::Held(words) | FollowState::Ended(words) => {
                bytes = bytes.checked_add(skein_lib::len_u64(words.len()))?;
            }
            FollowState::NotMade | FollowState::WaitsWord | FollowState::WaitsFor(_) | FollowState::Producing => {}
        }
    }
    bytes = bytes
        .checked_add(skein_lib::len_u64(row.skips.len()).checked_mul(u64::try_from(size_of::<FollowSkip>()).ok()?)?)?;
    for skip in &row.skips {
        bytes = bytes.checked_add(skein_lib::len_u64(skip.reason.len()))?;
    }
    if let Some(parity) = &row.parity {
        bytes = bytes.checked_add(skein_lib::len_u64(parity.run.len()))?;
    }
    Some(bytes)
}
pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut keys = List::with_capacity(d.follow_rows.capacity());
    for ((owner, follow), _) in &d.follow_rows {
        if *owner == project {
            keys.push(*follow).expect("project follows");
        }
    }
    for follow in keys.into_boxed() {
        keep(d, project, follow, out);
    }
}
pub(crate) fn deadline(d: &Domain) -> Option<Time> {
    let mut next: Option<Time> = None;
    for (_, due) in &d.follow_retry {
        next = Some(match next {
            Some(next) => next.min(*due),
            None => *due,
        });
    }
    next
}
pub(crate) fn fire(d: &mut Domain, now: Time) {
    let mut due = List::with_capacity(d.follow_retry.capacity());
    for (key, at) in &d.follow_retry {
        if *at <= now {
            due.push(*key).expect("baseline retries");
        }
    }
    for key in due.into_boxed() {
        d.follow_retry.remove(&key);
    }
}
