//! Whole changes, oldest last event first (landing-history.md, 4.2, 4.4 and 4.5).
use super::{HOUR_NS, HistoryRange, Last, MARKS, Project, read};
use crate::{Domain, Key, Limits, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Wall};

const PAGE: u32 = 32;
const DAY_NS: u64 = 24 * HOUR_NS;

#[derive(Hash, Debug)]
pub(crate) struct State {
    events: u64,
    next: Option<Wall>,
}
impl State {
    pub(crate) fn new(events: u64) -> Self {
        Self { events, next: None }
    }
}
pub(crate) fn deadline(d: &Domain) -> Option<Wall> {
    if d.history_projects.is_empty() { None } else { Some(d.history_sweep.next.unwrap_or(Wall::from_nanos(0))) }
}
pub(crate) fn fire(d: &mut Domain, wall: Wall, out: &mut Queue<Request>) {
    let Some(due) = deadline(d) else { return };
    if wall < due {
        return;
    }
    d.history_sweep.next = Some(Wall::from_nanos(wall.as_nanos().saturating_add(HOUR_NS)));
    let mut projects = List::with_capacity(d.history_projects.len());
    for (project, _) in &d.history_projects {
        projects.push(*project).expect("one sweep per history project");
    }
    for project in projects.into_boxed() {
        begin(d, project, out);
    }
}
pub(crate) fn written(d: &mut Domain, project: u32, wall: Wall, out: &mut Queue<Request>) {
    if d.history_sweep.next.is_none() {
        d.history_sweep.next = Some(Wall::from_nanos(wall.as_nanos().saturating_add(HOUR_NS)));
    }
    if let Some(row) = d.history_projects.get(&project)
        && (row.kept > d.history_sweep.events || row.removing || row.sweeping.is_some())
    {
        begin(d, project, out);
    }
}
pub(crate) fn remove(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let Some(row) = d.history_projects.get_mut(&project) else { return };
    row.removing = true;
    super::save(out, Stored::HistoryProject(row.clone()));
    begin(d, project, out);
}
fn begin(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    for (_, purpose) in &d.history_reads.loads {
        if let read::Purpose::Sweep { project: pending, .. } = purpose
            && *pending == project
        {
            return;
        }
    }
    read::issue_for(d, read::Purpose::Sweep { project, last: None }, HistoryRange::Last { project }, None, 1, out);
}

pub(super) fn loaded(
    d: &mut Domain,
    env: &Env<Limits>,
    project: u32,
    last: Option<Last>,
    rows: Box<[Stored]>,
    next: Option<Key>,
    out: &mut Queue<Request>,
) {
    let Some(mut history) = d.history_projects.get(&project).cloned() else { return };
    if let Some(last) = last {
        erase_page(d, &mut history, last, rows, next, out);
        return;
    }
    let Some(Stored::HistoryLast(last)) = rows.first() else {
        if history.removing {
            d.history_projects.remove(&project);
            out.push(Request::Erase { key: Key::HistoryProject { project } });
        }
        return;
    };
    let age = env.wall.as_nanos().saturating_sub(last.at.as_nanos());
    if history.removing
        || history.sweeping == Some(last.change)
        || history.kept > env.limits.history_events
        || age > u64::from(env.limits.history_days).saturating_mul(DAY_NS)
    {
        page(d, *last, None, out);
    }
}
fn page(d: &mut Domain, last: Last, after: Option<Key>, out: &mut Queue<Request>) {
    read::issue_for(
        d,
        read::Purpose::Sweep { project: last.project, last: Some(last) },
        HistoryRange::Change { project: last.project, change: last.change },
        after,
        PAGE,
        out,
    );
}
fn erase_page(
    d: &mut Domain,
    history: &mut Project,
    last: Last,
    rows: Box<[Stored]>,
    next: Option<Key>,
    out: &mut Queue<Request>,
) {
    let project = history.project;
    let mut ended = next.is_none();
    for row in rows {
        let Stored::HistoryChange(index) = row else { return };
        if index.number > last.number {
            ended = true;
            break;
        }
        out.push(Request::Erase { key: Key::HistoryChange { project, change: last.change, number: index.number } });
        out.push(Request::Erase { key: Key::HistoryEvent { project, number: index.number } });
        if let Some(landing) = index.landing {
            out.push(Request::Erase { key: Key::HistoryLanding { project, landing, number: index.number } });
        }
        history.kept = history.kept.checked_sub(1).expect("one kept event per index");
    }
    history.sweeping = if ended { None } else { Some(last.change) };
    if ended {
        out.push(Request::Erase { key: Key::HistoryLast { project, number: last.number, change: last.change } });
        history.whole_since = Some(history.whole_since.unwrap_or(last.at).max(last.at));
        trim_marks(history);
    }
    super::save(out, Stored::HistoryProject(history.clone()));
    d.history_projects.insert(project, history.clone()).expect("existing project history");
    if ended {
        begin(d, project, out);
    } else {
        page(d, last, next, out);
    }
}
fn trim_marks(history: &mut Project) {
    let Some(whole_since) = history.whole_since else { return };
    let hour = whole_since.as_nanos().checked_div(HOUR_NS).expect("nonzero hour");
    let mut last_old = None;
    for (index, mark) in history.marks.iter().enumerate() {
        if mark.hour.as_nanos().checked_div(HOUR_NS).expect("nonzero hour") < hour {
            last_old = Some(index);
        }
    }
    let mut marks = List::with_capacity(MARKS);
    for (index, mark) in history.marks.iter().enumerate() {
        if mark.hour.as_nanos().checked_div(HOUR_NS).expect("nonzero hour") >= hour || last_old == Some(index) {
            marks.push(*mark).expect("marks only fall");
        }
    }
    history.marks = marks.into_boxed();
}
