//! Landed drafts (temper/docs/design/domain/forge/drafts.md, sections 1–3).
use crate::{Answered, Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Commit, Error, Read};
use skein_lib::{Env, List, Map, Queue, Token, Wall};

pub const DRAFTS: u32 = 64;
#[must_use]
pub fn drafts_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 39) | u64::from(project)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DraftRepository {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DraftChange {
    Ready { chat: Option<u64> },
    NotReady,
    Started { goal: u64 },
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SetDraft {
    pub project: u32,
    pub repository: Option<DraftRepository>,
    pub path: Box<[u8]>,
    pub change: DraftChange,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandedDraft {
    pub repository: api::Repository,
    pub path: Box<[u8]>,
    pub project: u32,
    pub branch: Box<[u8]>,
    pub commit: Commit,
    pub title: Box<[u8]>,
    pub writer: u64,
    pub at: Wall,
    pub chat: Option<u64>,
    pub goal: Option<u64>,
    pub ready: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DraftRefusal {
    NoRepository,
    Path,
    Missing,
    NotAFile,
    Unreachable,
    Unknown,
    Full,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum SetDraftAnswer {
    Kept(DraftView),
    Refused(DraftRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DraftView {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub row: LandedDraft,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DraftsView {
    pub project: u32,
    pub rows: Box<[DraftView]>,
}
#[derive(Hash, Debug)]
pub(crate) struct Pending {
    token: Token,
    party: u64,
    call: SetDraft,
    repository: api::Repository,
    branch: Box<[u8]>,
    head: Option<Commit>,
}
#[must_use]
#[expect(clippy::disallowed_methods, reason = "drafts.md requires validating the UTF-8 path at connector admission")]
pub fn draft_path(path: &[u8]) -> bool {
    if path.is_empty() || path.len() > 1024 || !path.ends_with(b".md") || core::str::from_utf8(path).is_err() {
        return false;
    }
    let mut start = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            if !segment(path.get(start..index).expect("path segment")) {
                return false;
            }
            start = index.saturating_add(1);
        }
    }
    segment(path.get(start..).expect("last segment"))
}
fn segment(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes != b"." && bytes != b".."
}
#[expect(clippy::disallowed_methods, reason = "drafts.md defines the title on UTF-8 file bytes, empty on invalid text")]
fn title(bytes: &[u8]) -> Box<[u8]> {
    let bytes = bytes.get(..bytes.len().min(4096)).expect("bounded prefix");
    let Ok(text) = core::str::from_utf8(bytes) else { return Box::default() };
    for line in text.lines() {
        if let Some(text) = line.strip_prefix("# ") {
            let text = text.trim_matches(' ');
            let mut end = text.len().min(200);
            while !text.is_char_boundary(end) {
                end = end.saturating_sub(1);
            }
            return Box::from(text.get(..end).expect("title character boundary").as_bytes());
        }
    }
    Box::default()
}
#[expect(clippy::manual_find, reason = "the strict subset resolves repositories in key order without closures")]
fn repository<'a>(d: &'a Domain, call: &SetDraft) -> Option<&'a crate::Repository> {
    let Some(named) = &call.repository else { return d.home(call.project) };
    for (_, row) in &d.repositories {
        if row.project == call.project
            && row.owner == named.owner
            && row.name == named.name
            && !d.forge_ended(row.provider.forge)
        {
            return Some(row);
        }
    }
    None
}
fn refused(d: &mut Domain, request: Token, token: Token, why: DraftRefusal, out: &mut Queue<Request>) {
    d.connect.answer(request, token, Answered::SetDraft(SetDraftAnswer::Refused(why)), out);
}
pub(crate) fn start(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    party: u64,
    call: SetDraft,
    out: &mut Queue<Request>,
) {
    if !draft_path(&call.path) {
        return refused(d, request, token, DraftRefusal::Path, out);
    }
    let Some(repo) = repository(d, &call) else {
        return refused(d, request, token, DraftRefusal::NoRepository, out);
    };
    let repository = repo.provider;
    let branch = repo.settings.default_branch.clone();
    let key = (repository, call.path.clone());
    match call.change {
        DraftChange::NotReady | DraftChange::Started { .. } => {
            let Some(mut row) = d.drafts.get(&key).cloned() else {
                return refused(d, request, token, DraftRefusal::Unknown, out);
            };
            match call.change {
                DraftChange::NotReady => row.ready = false,
                DraftChange::Started { goal } => row.goal = Some(goal),
                DraftChange::Ready { .. } => unreachable!("no ready read here"),
            }
            keep(d, request, token, row, out);
        }
        DraftChange::Ready { .. } => {
            let connected = match d.connection(repository.forge) {
                crate::Connection::Connected { .. } => true,
                crate::Connection::NotConnected => d.writers.contains_key(&repository.forge),
                crate::Connection::NotSetUp
                | crate::Connection::SettingUp { .. }
                | crate::Connection::SetUpFailed { .. }
                | crate::Connection::Connecting
                | crate::Connection::Stopped { .. } => false,
            };
            if !connected || d.draft_reads.len() == d.draft_reads.capacity() {
                return refused(d, request, token, DraftRefusal::Unreachable, out);
            }
            let read = Read::Branch { branch: branch.clone() };
            d.draft_reads
                .insert(request, Pending { token, party, call, repository, branch, head: None })
                .expect("draft read room checked");
            crate::domain::child(d, env, skein_forge_client::Event::Read { owner: request, repository, read }, out);
        }
    }
}
pub(crate) fn answer(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.draft_reads.remove(&request) else { return };
    match result {
        Ok(Answer::Commit(head)) if pending.head.is_none() => {
            pending.head = Some(head);
            let repository = pending.repository;
            let read = Read::File { head, path: pending.call.path.clone(), max_bytes: 4096 };
            d.draft_reads.insert(request, pending).expect("same draft read");
            crate::domain::child(d, env, skein_forge_client::Event::Read { owner: request, repository, read }, out);
        }
        Ok(Answer::File { head, path, bytes, .. }) if pending.head == Some(head) && path == pending.call.path => {
            if d.repository(pending.repository).is_none() || d.forge_ended(pending.repository.forge) {
                return refused(d, request, pending.token, DraftRefusal::NoRepository, out);
            }
            let key = (pending.repository, path.clone());
            let held = d.drafts.get(&key);
            let chat = match pending.call.change {
                DraftChange::Ready { chat } => chat,
                DraftChange::NotReady | DraftChange::Started { .. } => unreachable!("only ready reads"),
            };
            let chat = match chat {
                Some(chat) => Some(chat),
                None => match held {
                    Some(row) => row.chat,
                    None => None,
                },
            };
            if held.is_none() && count(d, pending.call.project) >= DRAFTS {
                let mut oldest: Option<LandedDraft> = None;
                for (_, row) in &d.drafts {
                    if row.project != pending.call.project || (row.ready && row.goal.is_none()) {
                        continue;
                    }
                    let earlier = match &oldest {
                        Some(old) => (row.at, &row.path, row.repository) < (old.at, &old.path, old.repository),
                        None => true,
                    };
                    if earlier {
                        oldest = Some(row.clone());
                    }
                }
                let Some(old) = oldest else { return refused(d, request, pending.token, DraftRefusal::Full, out) };
                erase(d, &old, out);
            }
            let row = LandedDraft {
                repository: pending.repository,
                path,
                project: pending.call.project,
                branch: pending.branch,
                commit: head,
                title: title(&bytes),
                writer: pending.party,
                at: env.wall,
                chat,
                goal: None,
                ready: true,
            };
            keep(d, request, pending.token, row, out);
        }
        Ok(Answer::Directory { .. }) if pending.head.is_some() => {
            refused(d, request, pending.token, DraftRefusal::NotAFile, out)
        }
        Err(Error::Missing) => refused(d, request, pending.token, DraftRefusal::Missing, out),
        Ok(_) | Err(_) => refused(d, request, pending.token, DraftRefusal::Unreachable, out),
    }
}
fn keep(d: &mut Domain, request: Token, token: Token, row: LandedDraft, out: &mut Queue<Request>) {
    let key = (row.repository, row.path.clone());
    if d.drafts.get(&key) != Some(&row) {
        d.drafts.insert(key, row.clone()).expect("project draft bound checked");
        out.push(Request::Save { record: Stored::LandedDraft(row.clone()), release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([drafts_subject(row.project)]) });
    }
    let repo = d.repository(row.repository).expect("draft repository resolved");
    let view = DraftView { owner: repo.owner.clone(), name: repo.name.clone(), row };
    d.connect.answer(request, token, Answered::SetDraft(SetDraftAnswer::Kept(view)), out);
}
pub(crate) fn count(d: &Domain, project: u32) -> u32 {
    let mut count = 0_u32;
    for (_, row) in &d.drafts {
        if row.project == project {
            count = count.saturating_add(1);
        }
    }
    count
}
fn erase(d: &mut Domain, row: &LandedDraft, out: &mut Queue<Request>) {
    d.drafts.remove(&(row.repository, row.path.clone()));
    out.push(Request::Erase { key: Key::LandedDraft { repository: row.repository, path: row.path.clone() } });
    out.push(Request::Marked { tasks: Box::new([drafts_subject(row.project)]) });
}
pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut rows = List::with_capacity(d.drafts.len());
    for (_, row) in &d.drafts {
        if row.project == project {
            rows.push(row.clone()).expect("draft count");
        }
    }
    for row in rows.into_boxed() {
        erase(d, &row, out);
    }
}
pub(crate) fn erase_repository(d: &mut Domain, repository: api::Repository, out: &mut Queue<Request>) {
    let mut rows = List::with_capacity(d.drafts.len());
    for (_, row) in &d.drafts {
        if row.repository == repository {
            rows.push(row.clone()).expect("draft count");
        }
    }
    for row in rows.into_boxed() {
        erase(d, &row, out);
    }
}
impl Domain {
    pub(crate) fn drafts_view(&self, project: u32) -> DraftsView {
        let mut sorted = Map::with_capacity(DRAFTS);
        let mut rows = List::with_capacity(DRAFTS);
        for (_, row) in &self.drafts {
            if row.project == project
                && let Some(repo) = self.repository(row.repository)
            {
                sorted
                    .insert(
                        (core::cmp::Reverse(row.at), row.path.clone(), row.repository),
                        DraftView { owner: repo.owner.clone(), name: repo.name.clone(), row: row.clone() },
                    )
                    .expect("project draft bound");
            }
        }
        for (_, row) in &sorted {
            rows.push(row.clone()).expect("project draft bound");
        }
        DraftsView { project, rows: rows.into_boxed() }
    }
}
