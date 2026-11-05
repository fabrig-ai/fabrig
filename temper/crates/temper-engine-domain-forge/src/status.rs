//! Project landings and a party's sittings (temper/docs/design/domain/forge.md, 5.6).
use crate::{Answered, Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::Repository;
use skein_lib::{Duration, Env, List, Queue, Token, Wall};

/// A new sitting past this (temper/docs/design/domain/forge.md, 5.6).
pub const SITTING: Duration = Duration::from_secs(1800);
pub const BRIEF_SENTENCE_BYTES: usize = 200;
pub const BRIEF_LEDE_BYTES: usize = 600;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Brief {
    pub sentence: Box<[u8]>,
    pub lede: Box<[u8]>,
    pub at: Wall,
    pub landed: u64,
}
/// A status run in flight (temper/docs/design/domain/forge.md, 5.7); task none until the root has made it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StatusRunning {
    pub task: Option<u64>,
    pub landed: u64,
    pub since: Wall,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProjectFacts {
    pub project: u32,
    pub landed: u64,
    pub last_landed: Wall,
    pub brief: Option<Brief>,
    pub running: Option<StatusRunning>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Looked {
    pub party: u64,
    pub project: u32,
    pub previous: Option<Wall>,
    pub latest: Wall,
    pub seen: u64,
    pub next_seen: u64,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LookedAnswer {
    Looked,
    TooLarge,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StatusView {
    pub project: u32,
    pub landed: u64,
    pub last_landed: Option<Wall>,
    pub brief: Option<Brief>,
    pub previous: Option<Wall>,
    pub seen: u64,
    pub queued: u32,
    pub red: bool,
    pub branch: Box<[u8]>,
}
/// The subject status, beside `changes_subject`.
#[must_use]
pub fn status_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 34) | u64::from(project)
}

pub(crate) fn landed(d: &mut Domain, now: Wall, repository: Repository, out: &mut Queue<Request>) {
    let Some(adopted) = d.repositories.get(&repository) else { return };
    let project = adopted.project;
    if !d.project_facts.contains_key(&project) && d.project_facts.len() == d.project_facts.capacity() {
        return;
    }
    let mut row = match d.project_facts.remove(&project) {
        Some(row) => row,
        None => ProjectFacts { project, landed: 0, last_landed: now, brief: None, running: None },
    };
    row.landed = row.landed.checked_add(1).expect("project landing count fits u64");
    row.last_landed = now;
    out.push(Request::Save { record: Stored::ProjectFacts(row.clone()), release: None, read_afresh: None });
    out.push(Request::Marked { tasks: Box::new([status_subject(project)]) });
    d.project_facts.insert(project, row).expect("project facts room checked");
}

pub(crate) fn looked(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    party: u64,
    project: u32,
    out: &mut Queue<Request>,
) {
    let key = (party, project);
    if !d.looked.contains_key(&key) && d.looked.len() == d.looked.capacity() {
        d.connect.answer(request, token, Answered::Looked(LookedAnswer::TooLarge), out);
        return;
    }
    let landed = match d.project_facts.get(&project) {
        Some(row) => row.landed,
        None => 0,
    };
    let now = env.wall;
    let mut row = match d.looked.remove(&key) {
        Some(mut row) => {
            if let Some(age) = now.as_nanos().checked_sub(row.latest.as_nanos())
                && age >= SITTING.as_nanos()
            {
                row.previous = Some(row.latest);
                row.seen = row.next_seen;
            }
            row
        }
        None => Looked { party, project, previous: None, latest: now, seen: landed, next_seen: landed },
    };
    row.latest = now;
    row.next_seen = landed;
    out.push(Request::Save { record: Stored::Looked(row.clone()), release: None, read_afresh: None });
    out.push(Request::Marked { tasks: Box::new([status_subject(project)]) });
    d.looked.insert(key, row).expect("looked room checked");
    d.connect.answer(request, token, Answered::Looked(LookedAnswer::Looked), out);
}

pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut changed = d.project_facts.remove(&project).is_some();
    if changed {
        out.push(Request::Erase { key: Key::ProjectFacts { project } });
    }
    let mut erases = List::with_capacity(d.looked.len());
    for ((party, held), _) in &d.looked {
        if *held == project {
            erases.push(*party).expect("bounded looked rows");
        }
    }
    for party in erases.into_boxed() {
        d.looked.remove(&(party, project));
        out.push(Request::Erase { key: Key::Looked { party, project } });
        changed = true;
    }
    if changed {
        out.push(Request::Marked { tasks: Box::new([status_subject(project)]) });
    }
}

impl Domain {
    pub(crate) fn status_view(&self, project: u32, party: u64, now: Wall, limits: &Limits) -> Option<StatusView> {
        let mut view = StatusView {
            project,
            landed: 0,
            last_landed: None,
            brief: None,
            previous: None,
            seen: 0,
            queued: 0,
            red: false,
            branch: Box::default(),
        };
        if let Some(row) = self.project_facts.get(&project) {
            view.landed = row.landed;
            view.last_landed = Some(row.last_landed);
            view.brief.clone_from(&row.brief);
        }
        view.seen = view.landed;
        if let Some(row) = self.looked.get(&(party, project)) {
            view.previous = row.previous;
            view.seen = row.seen;
        }
        let changes = self.changes_view(project, now, limits)?;
        view.queued = changes.more;
        for queue in &changes.queues {
            view.queued = view.queued.checked_add(u32::try_from(queue.changes.len()).ok()?)?;
        }
        for (repository, adopted) in &self.repositories {
            if adopted.project == project && adopted.home {
                view.branch.clone_from(&adopted.settings.default_branch);
                view.red = self.branch_red(*repository, &view.branch);
                break;
            }
        }
        Some(view)
    }
}
