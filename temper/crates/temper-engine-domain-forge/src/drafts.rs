//! Landed drafts (temper/docs/design/domain/forge/drafts.md, sections 1–3).
use crate::{Answered, Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Commit, Error, Read};
use skein_lib::{Env, List, Map, Queue, Token};
pub use temper_engine_domain_forge_vocabulary::drafts::*;

/// Absent until the first draft read or restored row.
#[derive(Hash, Debug)]
pub(crate) struct State {
    pub(crate) rows: Map<(api::Repository, Box<[u8]>), LandedDraft>,
    reads: Map<Token, Pending>,
}
fn state<'a>(d: &'a mut Domain, limits: &Limits) -> &'a mut State {
    if d.drafts.is_none() {
        d.drafts = Some(Box::new(State {
            rows: Map::with_capacity(DRAFTS.saturating_mul(limits.judge_projects)),
            reads: Map::with_capacity(limits.applications),
        }));
    }
    d.drafts.as_deref_mut().expect("draft tables initialized")
}
pub(crate) fn held<'a>(d: &'a Domain, key: &(api::Repository, Box<[u8]>)) -> Option<&'a LandedDraft> {
    d.drafts.as_ref()?.rows.get(key)
}
pub(crate) fn len(d: &Domain) -> u32 {
    match &d.drafts {
        Some(state) => state.rows.len(),
        None => 0,
    }
}
pub(crate) fn reading(d: &Domain, request: Token) -> bool {
    match &d.drafts {
        Some(state) => state.reads.contains_key(&request),
        None => false,
    }
}
pub(crate) fn restore(d: &mut Domain, limits: &Limits, row: LandedDraft) {
    state(d, limits).rows.insert((row.repository, row.path.clone()), row).expect("restored draft room checked");
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
            let Some(mut row) = held(d, &key).cloned() else {
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
            let flights = match &d.drafts {
                Some(state) => state.reads.len(),
                None => 0,
            };
            if !connected || flights == env.limits.applications {
                return refused(d, request, token, DraftRefusal::Unreachable, out);
            }
            let read = Read::Branch { branch: branch.clone() };
            state(d, &env.limits)
                .reads
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
    let Some(tables) = d.drafts.as_deref_mut() else { return };
    let Some(mut pending) = tables.reads.remove(&request) else { return };
    match result {
        Ok(Answer::Commit(head)) if pending.head.is_none() => {
            pending.head = Some(head);
            let repository = pending.repository;
            let read = Read::File { head, path: pending.call.path.clone(), max_bytes: 4096 };
            d.drafts
                .as_deref_mut()
                .expect("pending draft tables")
                .reads
                .insert(request, pending)
                .expect("same draft read");
            crate::domain::child(d, env, skein_forge_client::Event::Read { owner: request, repository, read }, out);
        }
        Ok(Answer::File { head, path, bytes, .. }) if pending.head == Some(head) && path == pending.call.path => {
            if d.repository(pending.repository).is_none() || d.forge_ended(pending.repository.forge) {
                return refused(d, request, pending.token, DraftRefusal::NoRepository, out);
            }
            let key = (pending.repository, path.clone());
            let held = held(d, &key);
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
                for (_, row) in &d.drafts.as_ref().expect("draft rows present").rows {
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
            refused(d, request, pending.token, DraftRefusal::NotAFile, out);
        }
        Err(Error::Missing) => refused(d, request, pending.token, DraftRefusal::Missing, out),
        Ok(_) | Err(_) => refused(d, request, pending.token, DraftRefusal::Unreachable, out),
    }
}
fn keep(d: &mut Domain, request: Token, token: Token, row: LandedDraft, out: &mut Queue<Request>) {
    let key = (row.repository, row.path.clone());
    let tables = d.drafts.as_deref_mut().expect("read or restored draft tables");
    if tables.rows.get(&key) != Some(&row) {
        tables.rows.insert(key, row.clone()).expect("project draft bound checked");
        out.push(Request::Save { record: Stored::LandedDraft(row.clone()), release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([drafts_subject(row.project)]) });
    }
    let repo = d.repository(row.repository).expect("draft repository resolved");
    let view = DraftView { owner: repo.owner.clone(), name: repo.name.clone(), row };
    d.connect.answer(request, token, Answered::SetDraft(SetDraftAnswer::Kept(view)), out);
}
pub(crate) fn count(d: &Domain, project: u32) -> u32 {
    let Some(tables) = &d.drafts else { return 0 };
    let mut count = 0_u32;
    for (_, row) in &tables.rows {
        if row.project == project {
            count = count.saturating_add(1);
        }
    }
    count
}
fn erase(d: &mut Domain, row: &LandedDraft, out: &mut Queue<Request>) {
    d.drafts.as_deref_mut().expect("erased draft tables").rows.remove(&(row.repository, row.path.clone()));
    out.push(Request::Erase { key: Key::LandedDraft { repository: row.repository, path: row.path.clone() } });
    out.push(Request::Marked { tasks: Box::new([drafts_subject(row.project)]) });
}
pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let Some(tables) = &d.drafts else { return };
    let mut rows = List::with_capacity(tables.rows.len());
    for (_, row) in &tables.rows {
        if row.project == project {
            rows.push(row.clone()).expect("draft count");
        }
    }
    for row in rows.into_boxed() {
        erase(d, &row, out);
    }
}
pub(crate) fn erase_repository(d: &mut Domain, repository: api::Repository, out: &mut Queue<Request>) {
    let Some(tables) = &d.drafts else { return };
    let mut rows = List::with_capacity(tables.rows.len());
    for (_, row) in &tables.rows {
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
        let Some(tables) = &self.drafts else { return DraftsView { project, rows: rows.into_boxed() } };
        for (_, row) in &tables.rows {
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
