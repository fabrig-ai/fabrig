//! Demand-loaded pages, never restored events (landing-history.md, 4.4 and 5.1).
use super::{Cursor, Event, HOUR_NS, HistoryRange, Of, Query, Reply};
use crate::{Domain, Key, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue, Wall};

const READS: u32 = 8;
const PAGE: u32 = 32;
const ROWS: u32 = 256;

#[derive(Hash, Debug)]
pub(crate) struct State {
    reads: Map<u64, Read>,
    pub(crate) loads: Map<u64, Purpose>,
    next: u64,
}
#[derive(Clone, Copy, Hash, Debug)]
pub(crate) enum Purpose {
    Read { read: u64 },
    Sweep { project: u32, last: Option<super::Last> },
}
#[derive(Hash, Debug)]
struct Read {
    query: Query,
    upper: u64,
    whole_since: Option<Wall>,
    events: List<Event>,
    examined: u32,
    last: u64,
    gone: bool,
    phase: Phase,
}
#[derive(Hash, Debug)]
enum Phase {
    Cursor,
    Scan,
    Numbers { next: Option<Key>, ended: bool, range: HistoryRange },
}
impl State {
    pub(crate) fn new(projects: u32) -> Self {
        Self { reads: Map::with_capacity(READS), loads: Map::with_capacity(READS.saturating_add(projects)), next: 0 }
    }
}

pub(crate) fn begin(d: &mut Domain, read: u64, query: Query, out: &mut Queue<Request>) {
    if d.history_reads.reads.len() >= READS || d.history_reads.reads.contains_key(&read) {
        out.push(Request::HistoryPage { read, answer: Reply::Busy });
        return;
    }
    let project = d.history_projects.get(&query.project);
    let upper = match query.after {
        Some(cursor) => cursor.upper,
        None => match project {
            Some(project) => last_number(project),
            None => 0,
        },
    };
    let whole_since = match project {
        Some(project) => project.whole_since,
        None => None,
    };
    let mut first = 1;
    if let (Some(project), Some(since), Of::Project) = (project, query.since, query.of) {
        let hour = since.as_nanos().checked_div(HOUR_NS).expect("nonzero hour").saturating_mul(HOUR_NS);
        for mark in &project.marks {
            if mark.hour.as_nanos() <= hour {
                first = mark.first;
            }
        }
    }
    let last = match query.after {
        Some(cursor) => cursor.after,
        None => first.saturating_sub(1),
    };
    let state = Read {
        query,
        upper,
        whole_since,
        events: List::with_capacity(PAGE),
        examined: 0,
        last,
        gone: false,
        phase: if query.after.is_some() { Phase::Cursor } else { Phase::Scan },
    };
    d.history_reads.reads.insert(read, state).expect("admitted history read");
    if let Some(cursor) = query.after {
        issue(
            d,
            read,
            HistoryRange::Numbers { project: query.project, numbers: Box::new([cursor.after]) },
            None,
            1,
            out,
        );
    } else if sweeping(d, &query, None) || upper == 0 || last >= upper {
        finish(d, read, false, out);
    } else {
        scan(d, read, out);
    }
}
fn last_number(project: &super::Project) -> u64 {
    project.next.saturating_sub(1)
}

fn sweeping(d: &Domain, query: &Query, change: Option<u64>) -> bool {
    let Some(project) = d.history_projects.get(&query.project) else { return false };
    if project.removing {
        return true;
    }
    let change = match query.of {
        Of::Change(change) => Some(change),
        Of::Project | Of::Landing(_) => change,
    };
    change.is_some() && project.sweeping == change
}
fn issue(d: &mut Domain, read: u64, range: HistoryRange, after: Option<Key>, most: u32, out: &mut Queue<Request>) {
    if !issue_for(d, Purpose::Read { read }, range, after, most, out) {
        busy(d, read, out);
    }
}
pub(super) fn issue_for(
    d: &mut Domain,
    purpose: Purpose,
    range: HistoryRange,
    after: Option<Key>,
    most: u32,
    out: &mut Queue<Request>,
) -> bool {
    let Some(load) = d.history_reads.next.checked_add(1) else { return false };
    d.history_reads.next = load;
    d.history_reads.loads.insert(load, purpose).expect("one load per admitted read or project sweep");
    out.push(Request::LoadHistory { load, range, after, most });
    true
}
fn range(query: &Query) -> HistoryRange {
    match query.of {
        Of::Project => HistoryRange::Events { project: query.project },
        Of::Change(change) => HistoryRange::Change { project: query.project, change },
        Of::Landing(landing) => HistoryRange::Landing { project: query.project, landing },
    }
}
fn position(query: &Query, number: u64) -> Key {
    match query.of {
        Of::Project => Key::HistoryEvent { project: query.project, number },
        Of::Change(change) => Key::HistoryChange { project: query.project, change, number },
        Of::Landing(landing) => Key::HistoryLanding { project: query.project, landing, number },
    }
}
fn scan(d: &mut Domain, read: u64, out: &mut Queue<Request>) {
    let state = d.history_reads.reads.get_mut(&read).expect("live read");
    let remaining = ROWS.saturating_sub(state.examined);
    let rows = match state.query.of {
        Of::Project => remaining,
        Of::Change(_) | Of::Landing(_) => remaining / 2,
    };
    let most = rows.min(state.events.room());
    if most == 0 {
        finish(d, read, true, out);
        return;
    }
    state.phase = Phase::Scan;
    let range = range(&state.query);
    let after = Some(position(&state.query, state.last));
    issue(d, read, range, after, most, out);
}

pub(crate) fn unloaded(d: &mut Domain, load: u64, out: &mut Queue<Request>) {
    match d.history_reads.loads.remove(&load) {
        Some(Purpose::Read { read }) => busy(d, read, out),
        Some(Purpose::Sweep { .. }) | None => {}
    }
}
fn busy(d: &mut Domain, read: u64, out: &mut Queue<Request>) {
    if d.history_reads.reads.remove(&read).is_some() {
        out.push(Request::HistoryPage { read, answer: Reply::Busy });
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one load terminal advances cursor, index and event phases under one row budget"
)]
pub(crate) fn loaded(
    d: &mut Domain,
    env: &Env<crate::Limits>,
    load: u64,
    rows: Box<[Stored]>,
    next: Option<Key>,
    out: &mut Queue<Request>,
) {
    let read = match d.history_reads.loads.remove(&load) {
        Some(Purpose::Read { read }) => read,
        Some(Purpose::Sweep { project, last }) => {
            super::sweep::loaded(d, env, project, last, rows, next, out);
            return;
        }
        None => return,
    };
    let Some(mut state) = d.history_reads.reads.remove(&read) else { return };
    let phase = core::mem::replace(&mut state.phase, Phase::Scan);
    if let Phase::Cursor = phase {
        let mut present = false;
        for row in rows {
            if let Stored::HistoryEvent(event) = row {
                present |= event.number == state.last
                    && belongs(&state.query, &event)
                    && !sweeping(d, &state.query, Some(event.change));
            }
        }
        state.examined = 1;
        state.gone = !present;
        d.history_reads.reads.insert(read, state).expect("read slot retained");
        scan(d, read, out);
        return;
    }
    let indexed = match (state.query.of, &phase) {
        (Of::Change(_) | Of::Landing(_), Phase::Scan) => true,
        (Of::Project, Phase::Scan | Phase::Cursor | Phase::Numbers { .. })
        | (Of::Change(_) | Of::Landing(_), Phase::Cursor | Phase::Numbers { .. }) => false,
    };
    let (continuation, mut ended, number_page) = match phase {
        Phase::Numbers { next: continuation, ended, range } => (continuation, ended, Some((range, next))),
        Phase::Scan => (next, false, None),
        Phase::Cursor => unreachable!("cursor checked above"),
    };
    let mut numbers = List::with_capacity(PAGE);
    for row in rows {
        state.examined = state.examined.checked_add(1).expect("bounded examined rows");
        if indexed {
            let pair = if let Stored::HistoryChange(index) = &row {
                Some((index.number, index.change))
            } else if let Stored::HistoryLanding(index) = &row {
                Some((index.number, index.change))
            } else {
                None
            };
            let Some((number, change)) = pair else {
                out.push(Request::HistoryPage { read, answer: Reply::Busy });
                return;
            };
            if !state.gone && number > state.upper {
                ended = true;
                break;
            }
            state.last = number;
            if !sweeping(d, &state.query, Some(change)) {
                numbers.push(number).expect("bounded index page");
            }
        } else {
            let Stored::HistoryEvent(event) = row else {
                out.push(Request::HistoryPage { read, answer: Reply::Busy });
                return;
            };
            if !state.gone && (event.number > state.upper || at_end(&state.query, event.at)) {
                ended = true;
                break;
            }
            if let Of::Project = state.query.of {
                state.last = event.number;
            }
            if sweeping(d, &state.query, Some(event.change)) {
                continue;
            }
            if state.gone {
                out.push(Request::HistoryPage {
                    read,
                    answer: Reply::Gone {
                        from: state.query.after.expect("gone cursor").after,
                        first: Some(event.number),
                    },
                });
                return;
            }
            if belongs(&state.query, &event) && since(&state.query, event.at) {
                state.events.push(event).expect("requested only remaining page room");
            }
        }
    }
    if indexed && !numbers.is_empty() {
        let most = numbers.len();
        let range = HistoryRange::Numbers { project: state.query.project, numbers: numbers.into_boxed() };
        state.phase = Phase::Numbers { next: continuation, ended, range: range.clone() };
        d.history_reads.reads.insert(read, state).expect("read slot retained");
        issue(d, read, range, None, most, out);
        return;
    }
    if let Some((range, Some(after))) = number_page
        && !ended
    {
        let most = ROWS.saturating_sub(state.examined).min(state.events.room());
        if most != 0 {
            state.phase = Phase::Numbers { next: continuation, ended, range: range.clone() };
            d.history_reads.reads.insert(read, state).expect("read slot retained");
            issue(d, read, range, Some(after), most, out);
            return;
        }
    }
    ended |= continuation.is_none() || (!state.gone && state.last >= state.upper);
    let cut = state.examined >= ROWS || state.events.room() == 0;
    d.history_reads.reads.insert(read, state).expect("read slot retained");
    if ended || cut {
        finish(d, read, !ended, out);
    } else {
        scan(d, read, out);
    }
}
fn belongs(query: &Query, event: &Event) -> bool {
    event.project == query.project
        && match query.of {
            Of::Project => true,
            Of::Change(change) => event.change == change,
            Of::Landing(landing) => match event.landing {
                Some(value) => value.task == landing,
                None => false,
            },
        }
}
fn since(query: &Query, at: Wall) -> bool {
    match query.since {
        Some(since) => at >= since,
        None => true,
    }
}
fn at_end(query: &Query, at: Wall) -> bool {
    match query.until {
        Some(until) => at >= until,
        None => false,
    }
}
fn finish(d: &mut Domain, read: u64, more: bool, out: &mut Queue<Request>) {
    let Some(state) = d.history_reads.reads.remove(&read) else { return };
    let answer = if state.gone {
        if more {
            Reply::Busy
        } else {
            Reply::Gone { from: state.query.after.expect("gone cursor").after, first: None }
        }
    } else {
        Reply::Page {
            events: state.events.into_boxed(),
            upper: state.upper,
            whole_since: state.whole_since,
            after: if more { Some(Cursor { upper: state.upper, after: state.last }) } else { None },
        }
    };
    out.push(Request::HistoryPage { read, answer });
}
