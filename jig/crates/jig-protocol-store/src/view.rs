//! One held, rerunnable read over map rows and fetched history (protocol/store/logs.md, 4).
mod fetch;
mod rows;
use crate::{Below, Cursor, Event, Get, Limits, Load, Phase, Restore, Store, Write};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::reads;
pub(crate) use rows::{get, overlay, raw_get};
use skein_kv::Range;
use skein_lib::{Env, List, Map, Queue, Token};

#[derive(Clone, Debug)]
pub(crate) enum Ask {
    Read { owner: Token, range: reads::Range },
    Get { owner: Token, key: Get },
    Load { owner: Token, range: Load, after: Option<Cursor>, rows: u32, bytes: u64 },
    Restore { owner: Token, range: Restore, after: Option<Cursor>, rows: u32, bytes: u64 },
}
impl Ask {
    fn owner(&self) -> Token {
        match self {
            Self::Read { owner, .. }
            | Self::Get { owner, .. }
            | Self::Load { owner, .. }
            | Self::Restore { owner, .. } => *owner,
        }
    }
}
#[derive(Debug)]
enum Work {
    Read(Ask),
    Commit { number: u64, writes: Box<[Write]> },
}
#[derive(Debug)]
struct Task {
    asked: List<Range>,
    covered: List<Range>,
}
#[derive(Debug)]
struct Active {
    ask: Ask,
    phase: Phase,
    running: bool,
    tasks: Map<u64, Task>,
    rows: Map<Box<[u8]>, Box<[u8]>>,
    bytes: u64,
}
#[derive(Debug)]
struct Fetch {
    task: u64,
    at: u64,
    extent: u64,
    ended: Option<u64>,
    direct: bool,
    ranges: Box<[Range]>,
    waiting: bool,
}
#[derive(Debug)]
pub(crate) struct State {
    active: Option<Active>,
    queue: Queue<Work>,
    fetch: Option<Fetch>,
}
impl State {
    pub(crate) fn new() -> Self {
        Self { active: None, queue: Queue::uncapped(), fetch: None }
    }
}
pub(crate) fn held(store: &Store) -> bool {
    store.view.active.is_some()
}
pub(crate) fn missing(store: &Store) -> bool {
    if let Some(active) = &store.view.active {
        for (_, task) in &active.tasks {
            for range in &task.asked {
                if !covered(&task.covered, range) {
                    return true;
                }
            }
        }
    }
    false
}
fn covered(ranges: &List<Range>, range: &Range) -> bool {
    for known in ranges {
        if known.start <= range.start
            && match (&known.end, &range.end) {
                (None, Some(_) | None) => true,
                (Some(known), Some(end)) => known >= end,
                (Some(_), None) => false,
            }
        {
            return true;
        }
    }
    false
}
pub(crate) fn count(store: &Store) -> u32 {
    let mut count = store.transcripts.reads.len();
    if let Some(active) = &store.view.active
        && !store.transcripts.reads.contains_key(&active.ask.owner())
    {
        count = count.saturating_add(1);
    }
    for work in &store.view.queue {
        match work {
            Work::Read(_) => count = count.saturating_add(1),
            Work::Commit { .. } => {}
        }
    }
    count
}
pub(crate) fn owner(store: &Store) -> Option<Token> {
    match &store.view.active {
        Some(active) => Some(active.ask.owner()),
        None => None,
    }
}
pub(crate) fn ask(store: &mut Store, ask: Ask, above: &mut Queue<Event>) {
    match ask {
        Ask::Read { owner, .. } if count(store) >= store.limits.reads => {
            above.push(Event::Read { owner, result: reads::Loaded::Busy });
        }
        Ask::Read { .. } | Ask::Get { .. } | Ask::Load { .. } | Ask::Restore { .. } => {
            store.view.queue.push(Work::Read(ask));
        }
    }
}
pub(crate) fn commit(
    store: &mut Store,
    env: &Env<Limits>,
    number: u64,
    writes: Box<[Write]>,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if held(store) || !store.view.queue.is_empty() {
        store.view.queue.push(Work::Commit { number, writes });
    } else {
        crate::transcripts::write::begin(store, env, number, writes, above, below);
    }
}

/// Route a read's terminal only after its run has found no missing interval.
pub(crate) fn events(store: &mut Store, events: &mut Queue<Event>, above: &mut Queue<Event>) {
    for _ in 0..events.len() {
        let Some(event) = events.pop() else { break };
        let owner = match &event {
            Event::Read { owner, .. }
            | Event::Got { owner, .. }
            | Event::Loaded { owner, .. }
            | Event::Page { owner, .. }
            | Event::PageRefused { owner, .. }
            | Event::Corrupt { owner }
            | Event::Turns { owner, .. }
            | Event::Unread { owner } => Some(*owner),
            Event::Rehearsing { .. }
            | Event::Decoded { .. }
            | Event::Undecodable { .. }
            | Event::Back { .. }
            | Event::BackRefused { .. }
            | Event::Closed { .. }
            | Event::Opened { .. }
            | Event::Switched { .. }
            | Event::OpenRefused { .. }
            | Event::Committed { .. }
            | Event::Census { .. }
            | Event::Failed { .. }
            | Event::FrameTooLarge { .. }
            | Event::SecretsNotReady { .. }
            | Event::Kept { .. }
            | Event::Carried { .. }
            | Event::Stale { .. }
            | Event::Erased { .. }
            | Event::Secret { .. }
            | Event::SecretsLoaded { .. }
            | Event::Site { .. }
            | Event::Job { .. }
            | Event::Ran { .. }
            | Event::Patched { .. } => None,
        };
        let active_owner = match &store.view.active {
            Some(active) => Some(active.ask.owner()),
            None => None,
        };
        if owner.is_some() && owner == active_owner {
            if missing(store) {
                let active = store.view.active.as_mut().expect("held read");
                store.phase = active.phase;
                active.running = false;
            } else {
                store.view.active = None;
                above.push(event);
            }
        } else {
            above.push(event);
        }
    }
}

pub(crate) fn fire(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    // Each fetch advances a frame, each rerun adds interval coverage. Extent
    // rows bound the tasks and joins/tree rows bound discovered intervals.
    // Memory logs finish here; file logs yield on their first pending page.
    let frames = store.limits.store.log.checked_div(17).expect("nonzero KV frame overhead").saturating_add(4);
    let intervals =
        u64::from(store.limits.joins).saturating_add(u64::from(store.limits.core.tasks.tree_tasks)).saturating_add(16);
    let tasks = u64::from(store.stores.store.len()).saturating_add(1);
    let work =
        u64::from(store.view.queue.len()).saturating_add(u64::from(store.transcripts.reads.len())).saturating_add(1);
    let steps = frames.saturating_mul(intervals).saturating_mul(tasks).saturating_mul(work);
    for _ in 0..steps {
        if store.phase == Phase::Stopped {
            return;
        }
        if store.view.active.is_none() {
            let waiting = store.transcripts.reads.len();
            if waiting > 0 {
                crate::transcripts::read::fire(store, above, below);
                if store.transcripts.reads.len() < waiting {
                    continue;
                }
                return;
            }
            if store.transcripts.current.is_some()
                || !store.transcripts.queue.is_empty()
                || match store.phase {
                    Phase::Ready { next, .. } => next.saturating_sub(1) > store.stores.store.last(),
                    Phase::Closed | Phase::Stopped => false,
                }
            {
                return;
            }
            let Some(work) = store.view.queue.pop() else { return };
            match work {
                Work::Commit { number, writes } => {
                    crate::transcripts::write::begin(store, env, number, writes, above, below);
                    continue;
                }
                Work::Read(ask) => {
                    store.view.active = Some(Active {
                        ask,
                        phase: store.phase,
                        running: false,
                        tasks: Map::uncapped(),
                        rows: Map::uncapped(),
                        bytes: 0,
                    });
                }
            }
        }
        if store.view.fetch.is_some() {
            if !fetch::advance(store, above, below) {
                return;
            }
            continue;
        }
        if store.view.active.as_ref().expect("held read").running {
            return;
        }
        if missing(store) {
            if let Err(failure) = fetch::prepare(store) {
                fail(store, failure, above);
            }
            continue;
        }
        let active = store.view.active.as_mut().expect("held read");
        active.running = true;
        let ask = active.ask.clone();
        let mut output = Queue::uncapped();
        run(store, env, ask, &mut output, below);
        events(store, &mut output, above);
    }
}
fn run(store: &mut Store, env: &Env<Limits>, ask: Ask, above: &mut Queue<Event>, below: &mut Below) {
    match ask {
        Ask::Read { owner, range } => crate::reads::read(store, owner, range, above, below),
        Ask::Get { owner, key } => crate::gets::get(store, owner, key, above),
        Ask::Load { owner, range, after, rows, bytes } => {
            crate::loads::load(store, owner, range, after, rows, bytes, above);
        }
        Ask::Restore { owner, range, after, rows, bytes } => {
            crate::restore::restore(
                store,
                env,
                owner,
                crate::restore::Page { range, after, rows, bytes },
                above,
                below,
            );
        }
    }
}
#[derive(Debug)]
enum Failure {
    Bounds { needs: u64, key: Box<[u8]> },
    Corrupt,
}
fn fail(store: &mut Store, failure: Failure, above: &mut Queue<Event>) {
    let Some(active) = store.view.active.take() else { return };
    store.view.fetch = None;
    let owner = active.ask.owner();
    match failure {
        Failure::Bounds { needs, key } => match active.ask {
            Ask::Read { .. } => above.push(Event::Read { owner, result: crate::reads::empty() }),
            Ask::Restore { .. } => {
                let failure = crate::restore::Failure::at(
                    crate::restore::Cause::RowPastPage { needs, has: store.limits.fetch },
                    crate::restore::site(store, &key),
                );
                store.phase = Phase::Stopped;
                above.push(Event::PageRefused { owner, failure });
            }
            Ask::Get { .. } | Ask::Load { .. } => crate::loads::corrupt(store, owner, above),
        },
        Failure::Corrupt => match active.ask {
            Ask::Restore { .. } => {
                store.phase = Phase::Stopped;
                above.push(Event::PageRefused {
                    owner,
                    failure: crate::restore::Failure::rowless(crate::restore::Cause::Decode(
                        crate::restore::DecodeFailure::Value,
                    )),
                });
            }
            Ask::Read { .. } | Ask::Get { .. } | Ask::Load { .. } => {
                crate::loads::corrupt(store, owner, above);
            }
        },
    }
}

pub(crate) use fetch::{read, unread};
