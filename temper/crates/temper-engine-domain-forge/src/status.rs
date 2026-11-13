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
        ask_run(d, env, project, out);
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

// A brief and a stuck run each get an hour (temper/docs/design/domain/forge/attention.md, 3).
const HOUR: Duration = Duration::from_secs(3600);

fn old(now: Wall, then: Wall) -> bool {
    match now.as_nanos().checked_sub(then.as_nanos()) {
        Some(age) => age > HOUR.as_nanos(),
        None => false,
    }
}
fn save(row: &ProjectFacts, out: &mut Queue<Request>) {
    out.push(Request::Save { record: Stored::ProjectFacts(row.clone()), release: None, read_afresh: None });
}
fn ask_run(d: &mut Domain, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    let Some(row) = d.project_facts.get(&project) else { return };
    let covered = match &row.brief {
        Some(brief) => brief.landed,
        None => 0,
    };
    if row.landed <= covered {
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
    let (words, signers) = words::specification(d, row, env.wall);
    let row = d.project_facts.get_mut(&project).expect("held facts");
    row.running = Some(StatusRunning { task: None, landed: row.landed, since: env.wall });
    save(row, out);
    out.push(Request::StatusRun { project, words, signers });
}

pub(crate) fn task(d: &mut Domain, project: u32, task: Option<u64>, out: &mut Queue<Request>) {
    let Some(row) = d.project_facts.get_mut(&project) else { return };
    let Some(running) = &mut row.running else { return };
    match task {
        Some(task) => running.task = Some(task),
        None => row.running = None,
    }
    save(row, out);
}

pub(crate) fn heard(d: &mut Domain, now: Wall, task: u64, report: Option<Box<[u8]>>, out: &mut Queue<Request>) {
    if let Some(project) = d.status_project(task) {
        let row = d.project_facts.get_mut(&project).expect("named status project");
        let running = row.running.as_ref().expect("named run");
        if let Some(report) = &report {
            let (sentence, lede) = status_paragraphs(report);
            if !sentence.is_empty() {
                row.brief = Some(Brief {
                    sentence: crate::text::fit(sentence, BRIEF_SENTENCE_BYTES, b"..."),
                    lede: crate::text::fit(lede, BRIEF_LEDE_BYTES, b"..."),
                    at: now,
                    landed: running.landed,
                });
                out.push(Request::Marked { tasks: Box::new([status_subject(row.project)]) });
            }
        }
        row.running = None;
        save(row, out);
    }
}

impl Domain {
    /// The project whose current status run names this task (temper/docs/design/domain/forge/attention.md, 3).
    #[must_use]
    pub fn status_project(&self, task: u64) -> Option<u32> {
        for (project, row) in &self.project_facts {
            if let Some(running) = &row.running
                && running.task == Some(task)
            {
                return Some(*project);
            }
        }
        None
    }
}
