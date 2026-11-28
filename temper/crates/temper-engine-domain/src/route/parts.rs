//! Make row parts from live state and bounded archive loads
//! (temper/docs/design/client/forge/parts.md, 2–4).
use super::{Decision, Delivery, Domain, Env, Limits, Work, core, emit};
use crate::{CoreRange, Key, Range, Record, boundary::Read, loads};
use alloc::boxed::Box;

use jig_domain_core::documents::{self as d, MadePart, PartFailure, TaskLine};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Id, List, Map, Queue, Token, Wall};
use temper_client_protocol_wire as wire;
use temper_engine_domain_forge::{MilestoneTotals, parts as encode};

mod load;
mod milestones;
pub(crate) use load::{failed, joined, loaded};
use milestones::{delegates, milestone};

const KEPT: u32 = 4096;

#[derive(Hash, Debug)]
pub(crate) struct Memory {
    tick: u64,
    epoch: u64,
    entries: Map<u64, Entry>,
}
#[derive(Hash, Debug)]
struct Entry {
    project: u32,
    used: u64,
    notes: Option<Notes>,
    ended: Option<Ended>,
}
#[derive(Clone, Hash, Debug)]
struct Notes {
    report: Option<(Box<[u8]>, Wall)>,
    document: Option<Box<[u8]>>,
}
#[derive(Clone, Hash, Debug)]
struct Ended {
    count: u32,
    loaded: bool,
    whole: Option<Wall>,
    totals: Option<MilestoneTotals>,
    lines: List<TaskLine>,
}
impl Memory {
    pub(crate) fn new() -> Self {
        Self { tick: 0, epoch: 0, entries: Map::with_capacity(KEPT) }
    }
    fn entry(&mut self, project: u32, task: u64) -> &mut Entry {
        self.tick = self.tick.checked_add(1).expect("part use counter");
        if !self.entries.contains_key(&task) {
            if self.entries.len() == KEPT {
                let mut oldest = None;
                for (number, entry) in &self.entries {
                    let key = (entry.used, *number);
                    if oldest.is_none() || Some(key) < oldest {
                        oldest = Some(key);
                    }
                }
                self.entries.remove(&oldest.expect("full kept table").1);
            }
            self.entries.insert(task, Entry { project, used: self.tick, notes: None, ended: None }).expect("kept room");
        }
        let entry = self.entries.get_mut(&task).expect("kept entry");
        assert!(entry.project == project, "task numbers name one project");
        entry.used = self.tick;
        entry
    }
}

#[derive(Hash, Debug)]
pub(crate) struct Flight {
    connector: u16,
    ask: core::PartsAsk,
    made: List<MadePart>,
    row: u32,
    kind: u32,
    phase: Phase,
}
#[derive(Hash, Debug)]
enum Phase {
    Making,
    Index(NoteLoad),
    Note(NoteLoad, u64),
    Milestone { project: u32, task: u64 },
    Delegates { project: u32, task: u64, ended: Ended, page: u32 },
}
#[derive(Hash, Debug)]
struct NoteLoad {
    project: u32,
    task: u64,
    epoch: u64,
    after: Option<u64>,
    next: bool,
    names: Queue<u64>,
    best: Option<(Wall, u64)>,
    kept: Notes,
}

enum Making {
    Made(MadePart),
    Load(Phase, Range),
    Joined(Phase, Box<core::reads::Range>),
}

pub(super) fn ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    connector: u16,
    ask: core::PartsAsk,
) {
    let flight = Flight { connector, ask, made: List::uncapped(), row: 0, kind: 0, phase: Phase::Making };
    drive(domain, env, decision, None, Box::new(flight));
}

fn answer(domain: &mut Domain, flight: Flight) {
    use jig_domain_contract::{Connector, Up, parts};
    domain.work.push(Work::Connector(
        Connector(flight.connector),
        Up::Parts(parts::Up::Lines { token: flight.ask.token, parts: flight.made }),
    ));
}
fn push(flight: &mut Flight, part: MadePart) {
    flight.made.push(part).expect("ask shape");
    flight.kind = flight.kind.checked_add(1).expect("bounded kinds");
    if flight.kind == flight.ask.kinds.len() {
        flight.kind = 0;
        flight.row = flight.row.checked_add(1).expect("bounded rows");
    }
}
fn drive(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    mut waiter: Option<Token>,
    mut flight: Box<Flight>,
) {
    while flight.row < flight.ask.rows.len() && !flight.ask.kinds.is_empty() {
        let row = flight.ask.rows.get(flight.row).expect("ask row");
        let kind = flight.ask.kinds.get(flight.kind).expect("ask kind");
        let made = if flight.connector == domain.config.forge_connector
            && kind.owner == wire::FORGE
            && kind.version == wire::CLIENT_VERSION
        {
            make(domain, env, flight.ask.party, row, kind.kind)
        } else {
            Making::Made(MadePart::Failed(PartFailure::Unreadable))
        };
        let (phase, ordinary, joined) = match made {
            Making::Made(part) => {
                push(&mut flight, part);
                continue;
            }
            Making::Load(phase, range) => (phase, Some(range), None),
            Making::Joined(phase, range) => (phase, None, Some(range)),
        };
        if waiter.is_none() && (domain.archive_reads.is_full() || !domain.loads.has_room()) {
            push(&mut flight, MadePart::Failed(PartFailure::Busy));
            continue;
        }
        let token = match waiter.take() {
            Some(token) => token,
            None => domain.archive_reads.insert(None).expect("admitted part read").token(),
        };
        flight.phase = phase;
        *domain.archive_reads.get_mut(Id::from_token(token)).expect("part read slot") = Some(Read::Parts(flight));
        match (ordinary, joined) {
            (Some(range), None) => emit(decision, &env.limits, Delivery::Load { waiter: token, range, after: None }),
            (None, Some(range)) => emit(decision, &env.limits, Delivery::PartLoad { waiter: token, range }),
            (Some(_), Some(_)) | (None, None) => unreachable!("one part load"),
        }
        return;
    }
    if let Some(waiter) = waiter {
        domain.archive_reads.retire(Id::from_token(waiter));
    }
    answer(domain, *flight);
}

fn encoded(value: Option<Box<[u8]>>) -> Making {
    Making::Made(match value {
        Some(bytes) => MadePart::Made(bytes),
        None => MadePart::Failed(PartFailure::TooLarge),
    })
}
fn unreadable() -> Making {
    Making::Made(MadePart::Failed(PartFailure::Unreadable))
}
fn note_load(domain: &Domain, project: u32, task: u64) -> Making {
    Making::Load(
        Phase::Index(NoteLoad {
            project,
            task,
            epoch: domain.parts.epoch,
            after: None,
            next: false,
            names: Queue::uncapped(),
            best: None,
            kept: Notes { report: None, document: None },
        }),
        Range::Core(CoreRange::ApplicationNotes(notes::Range::Lines {
            scope: notes::Scope::Goal { project, goal: task },
            after: None,
        })),
    )
}
fn make(domain: &mut Domain, env: &Env<Limits>, party: u64, row: &TaskLine, kind: u32) -> Making {
    let task = row.chip.number;
    match kind {
        wire::GOAL_PART | wire::CHAT_PART => {
            let Some(kept) = domain.parts.entry(row.project, task).notes.clone() else {
                return note_load(domain, row.project, task);
            };
            if kind == wire::CHAT_PART {
                return encoded(encode::chat(kept.document.as_deref()));
            }
            let milestone = match row.requester {
                tasks::Party::Task(parent) => {
                    let live = domain.engine.core().tasks.task(parent);
                    let is_milestone = match live {
                        Some(parent) => parent.project == row.project && tasks::gathering::executor(parent.executor),
                        None => match domain.forge.milestone(parent) {
                            Some(parent) => parent.project == row.project,
                            None => false,
                        },
                    };
                    if is_milestone { Some(parent) } else { None }
                }
                tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
            };
            encoded(encode::goal(kept.report.as_ref(), milestone))
        }
        wire::MILESTONE_PART => milestone(domain, env, party, row),
        _ => unreadable(),
    }
}

pub(super) fn marked(domain: &mut Domain, task: u64) {
    let mut tasks = List::with_capacity(1);
    tasks.push(task).expect("one milestone mark");
    super::forge_up(domain, jig_domain_contract::Up::Parts(jig_domain_contract::parts::Up::Marked { tasks }));
}

pub(super) fn saved(domain: &mut Domain, record: &core::Record) {
    if let core::Record::Notes(record) = record {
        let scope = match record {
            notes::Record::Entry(row) => &row.scope,
            notes::Record::Line(row) => &row.scope,
        };
        invalidate(domain, scope);
    }
}
fn invalidate(domain: &mut Domain, scope: &notes::Scope) {
    domain.parts.epoch = domain.parts.epoch.checked_add(1).expect("note epoch");
    if let notes::Scope::Goal { goal, .. } = scope
        && let Some(entry) = domain.parts.entries.get_mut(goal)
    {
        entry.notes = None;
    }
}
pub(super) fn erased(domain: &mut Domain, key: &core::Key, projects: &skein_lib::Set<u32>) {
    if let core::Key::Notes(key) = key {
        match key {
            notes::Key::Line { scope, .. } => invalidate(domain, scope),
            notes::Key::Entry { .. } => {
                domain.parts.epoch = domain.parts.epoch.checked_add(1).expect("note epoch");
                let mut affected = List::uncapped();
                for (task, entry) in &domain.parts.entries {
                    if projects.contains(&entry.project) {
                        affected.push(*task).expect("kept task count");
                    }
                }
                for task in affected.into_boxed() {
                    domain.parts.entries.get_mut(&task).expect("affected entry").notes = None;
                }
            }
        }
    }
}

/// Entry erasures omit scope; their decision's index writes name the project.
pub(super) fn note_projects(requests: &Queue<core::engine::Out>) -> skein_lib::Set<u32> {
    let mut projects = skein_lib::Set::with_capacity(requests.len());
    for output in requests {
        let core::engine::Out::Write(write) = output else { continue };
        let scope = if let core::Write::Save(core::Record::Notes(record)) = write {
            match record {
                notes::Record::Entry(entry) => Some(&entry.scope),
                notes::Record::Line(line) => Some(&line.scope),
            }
        } else if let core::Write::Erase(core::Key::Notes(notes::Key::Line { scope, .. })) = write {
            Some(scope)
        } else {
            None
        };
        if let Some(
            notes::Scope::Goal { project, .. }
            | notes::Scope::Project { project }
            | notes::Scope::Resources { project, .. },
        ) = scope
        {
            projects.insert(*project).expect("decision projects");
        }
    }
    projects
}
