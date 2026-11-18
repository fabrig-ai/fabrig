//! Project landings, sittings and briefs (temper/docs/design/domain/forge/attention.md, 2, 3).
use crate::{Answered, Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::Repository;
use skein_lib::{Duration, Env, List, Queue, Token, Wall};
pub use temper_engine_domain_forge_vocabulary::status::*;

mod words;

pub(crate) fn landed(d: &mut Domain, now: Wall, repository: Repository, out: &mut Queue<Request>) {
    let Some(adopted) = d.repositories.get(&repository) else { return };
    let project = adopted.project;
    if !d.project_facts.contains_key(&project) && d.project_facts.len() == d.project_facts.capacity() {
        return;
    }
    let mut row = match d.project_facts.remove(&project) {
        Some(row) => row,
        None => ProjectFacts { project, landed: 0, last_landed: now },
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
    let mut sitting = false;
    let mut row = match d.looked.remove(&key) {
        Some(mut row) => {
            if let Some(age) = now.as_nanos().checked_sub(row.latest.as_nanos())
                && age >= env.limits.sitting.as_nanos()
            {
                sitting = true;
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
    if sitting {
        ask_run(d, env, party, project, out);
    }
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
    let mut erases = List::with_capacity(d.person_briefs.len());
    for ((party, held), _) in &d.person_briefs {
        if *held == project {
            erases.push(*party).expect("bounded person briefs");
        }
    }
    for party in erases.into_boxed() {
        d.person_briefs.remove(&(party, project));
        out.push(Request::Erase { key: Key::PersonBrief { party, project } });
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
            unpaid: false,
            previous: None,
            seen: 0,
            queued: 0,
            red: false,
            branch: Box::default(),
        };
        if let Some(row) = self.project_facts.get(&project) {
            view.landed = row.landed;
            view.last_landed = if row.landed == 0 { None } else { Some(row.last_landed) };
        }
        if let Some(row) = self.person_briefs.get(&(party, project)) {
            view.brief.clone_from(&row.brief);
            view.unpaid = row.unpaid.is_some();
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

// A brief and a stuck run each get an hour (temper/docs/design/domain/forge/attention.md, 3).
const HOUR: Duration = Duration::from_secs(3600);

fn old(now: Wall, then: Wall) -> bool {
    match now.as_nanos().checked_sub(then.as_nanos()) {
        Some(age) => age > HOUR.as_nanos(),
        None => false,
    }
}
fn save(row: &PersonBrief, out: &mut Queue<Request>) {
    out.push(Request::Save { record: Stored::PersonBrief(row.clone()), release: None, read_afresh: None });
}
fn ask_run(d: &mut Domain, env: &Env<Limits>, party: u64, project: u32, out: &mut Queue<Request>) {
    let (findings, mark) = words::findings(d, project);
    if !d.project_facts.contains_key(&project) {
        if mark.is_none() || d.project_facts.len() == d.project_facts.capacity() {
            return;
        }
        let row = ProjectFacts { project, landed: 0, last_landed: env.wall };
        out.push(Request::Save { record: Stored::ProjectFacts(row.clone()), release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([status_subject(project)]) });
        d.project_facts.insert(project, row).expect("project facts room checked");
    }
    let facts = d.project_facts.get(&project).expect("held facts");
    let key = (party, project);
    if !d.person_briefs.contains_key(&key) && d.person_briefs.len() == d.person_briefs.capacity() {
        return;
    }
    let mut row = d.person_briefs.get(&key).cloned().unwrap_or(PersonBrief {
        party,
        project,
        brief: None,
        running: None,
        unpaid: None,
    });
    let covered = match &row.brief {
        Some(brief) => brief.landed,
        None => 0,
    };
    let previous = match &row.brief {
        Some(brief) => brief.findings,
        None => None,
    };
    let fresh = facts.landed > covered;
    let changed = mark != previous;
    if !fresh && !changed {
        return;
    }
    if let Some(brief) = &row.brief
        && !old(env.wall, brief.at)
    {
        return;
    }
    if let Some(running) = &row.running
        && !old(env.wall, running.since)
    {
        return;
    }
    let since = match &row.brief {
        Some(brief) => brief.at,
        None => Wall::EPOCH,
    };
    let (words, signers) = words::specification(d, facts, row.brief.as_ref(), since, env.wall);
    row.running = Some(StatusRunning { task: None, landed: facts.landed, since: env.wall, findings: mark });
    save(&row, out);
    d.person_briefs.insert(key, row).expect("person brief room checked");
    out.push(Request::Marked { tasks: Box::new([status_subject(project)]) });
    out.push(Request::StatusRun { project, party, since, words, findings, signers });
}

pub(crate) fn task(d: &mut Domain, now: Wall, party: u64, project: u32, task: Option<u64>, out: &mut Queue<Request>) {
    let Some(row) = d.person_briefs.get_mut(&(party, project)) else { return };
    let Some(running) = &mut row.running else { return };
    match task {
        Some(task) => running.task = Some(task),
        None => {
            row.running = None;
            row.unpaid = Some(now);
        }
    }
    save(row, out);
    out.push(Request::Marked { tasks: Box::new([status_subject(project)]) });
}

pub(crate) fn heard(d: &mut Domain, now: Wall, task: u64, report: Option<Box<[u8]>>, out: &mut Queue<Request>) {
    if let Some(owner) = d.status_owner(task) {
        let row = d.person_briefs.get_mut(&owner).expect("named status owner");
        let running = row.running.as_ref().expect("named run");
        if !old(now, running.since)
            && let Some(report) = &report
        {
            let (sentence, lede) = status_paragraphs(report);
            if !sentence.is_empty() {
                row.brief = Some(Brief {
                    sentence: crate::text::fit(sentence, BRIEF_SENTENCE_BYTES, b"..."),
                    lede: crate::text::fit(lede, BRIEF_LEDE_BYTES, b"..."),
                    at: now,
                    landed: running.landed,
                    findings: running.findings,
                });
            }
        }
        row.running = None;
        row.unpaid = None;
        save(row, out);
        out.push(Request::Marked { tasks: Box::new([status_subject(row.project)]) });
    }
}

impl Domain {
    /// The party and project whose current status run names this task (temper/docs/design/domain/forge/attention.md, 3).
    #[must_use]
    pub fn status_owner(&self, task: u64) -> Option<(u64, u32)> {
        for (owner, row) in &self.person_briefs {
            if let Some(running) = &row.running
                && running.task == Some(task)
            {
                return Some(*owner);
            }
        }
        None
    }
}
