//! Whole requester preferences in asking briefs
//! (`jig/docs/design/domain/engine/briefs.md`, sections 1, 1.1 and 2.1).
//! The run's charter must name the asking family and its authority must hold it.
//! The party index is gathered before the ordinary notes index and retained
//! for this preparation alone. Its required section cannot be cut.

use crate::{
    Core, Event, Limits, Request,
    routing::{NoteBrief, NoteRoute, Work},
};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, Env, List, Map, Queue, Writer};

/// Text retained for at most the configured number of briefs.
#[derive(Hash, Debug)]
pub(crate) struct State {
    texts: Map<u64, Box<[u8]>>,
}
impl State {
    pub(crate) fn new(briefs: u32) -> Self {
        Self { texts: Map::with_capacity(briefs) }
    }
}

pub(crate) fn clear(core: &mut Core, task: u64) {
    drop(core.preferences.texts.remove(&task));
}

fn party(core: &Core, task: u64) -> Option<u64> {
    let context = core.contexts.get(&task)?;
    let charter = match context.executor {
        tasks::Executor::Agent { .. } => core.charter_of(task),
        tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => return None,
    };
    if context.authority.tools.0 & core.settings.tools.ask.0 == 0
        || charter.policy.tools.0 & core.settings.tools.ask.0 == 0
    {
        return None;
    }
    match context.requester {
        tasks::Party::Person(party) => Some(party),
        tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
    }
}

pub(crate) fn gather(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Work>, task: u64) -> bool {
    let Some(party) = party(core, task) else { return false };
    // A completed claim no longer has a preparation context.
    for _ in 0..core.preferences.texts.len() {
        let mut ended = None;
        for (&number, _) in &core.preferences.texts {
            if !core.contexts.contains_key(&number) {
                ended = Some(number);
                break;
            }
        }
        let Some(ended) = ended else { break };
        clear(core, ended);
    }
    if core.preferences.texts.contains_key(&task) {
        return false;
    }
    if core.preferences.texts.len() == core.preferences.texts.capacity() {
        fail(work, task, brief::GatherFate::Capacity, b"no room for the index");
        return true;
    }
    let mut scopes = List::with_capacity(1);
    scopes.push(notes::Scope::Party { party }).expect("one party scope");
    let owner = core.begin_note_route(NoteRoute::Preferences { task });
    work.push(Work::Event(Event::Notes(notes::Event::Index { owner, scopes, most: env.limits.notes.party_entries })));
    true
}

pub(crate) fn plan(
    core: &Core,
    limits: &Limits,
    work: &mut Queue<Work>,
    task: u64,
    wanted: &mut List<brief::Planned>,
) -> bool {
    if party(core, task).is_none() {
        return true;
    }
    let Some(text) = text(core, task) else {
        fail(work, task, brief::GatherFate::Permanent, b"the index could not be read");
        return false;
    };
    let size = u32::try_from(text.len()).expect("bounded party index text");
    let position = wanted.len();
    if wanted
        .push(brief::Planned::Core {
            kind: brief::Core::Preferences,
            text,
            limit: limits.brief_core_budgets.preferences,
            priority: 3,
            required: true,
        })
        .is_ok()
        && let Some(plan) = brief::plan(wanted.as_slice(), limits.brief.brief_bytes)
    {
        for placed in &plan.order {
            if placed.index == position && placed.size == size {
                return true;
            }
        }
    }
    fail(work, task, brief::GatherFate::Permanent, b"does not fit the brief's budget");
    false
}

pub(crate) fn indexed(
    core: &mut Core,
    _env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    lines: List<notes::Line>,
    more: u32,
) -> Queue<Request> {
    if let Some(context) = core.contexts.get(&task)
        && super::parts::preparation_current(core, context)
        && party(core, task).is_some()
    {
        if more != 0 {
            fail(work, task, brief::GatherFate::Permanent, b"the index could not be read");
        } else if core.preferences.texts.insert(task, render(&lines)).is_ok() {
            work.push(Work::StartBrief { task });
        } else {
            fail(work, task, brief::GatherFate::Capacity, b"no room for the index");
        }
    }
    Queue::uncapped()
}

pub(crate) fn refused(
    core: &mut Core,
    _env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    why: notes::Refusal,
) -> Queue<Request> {
    clear(core, task);
    match why {
        notes::Refusal::Busy => {
            if core.notes.busy() {
                crate::routing::effect::note::wait_for_notes(core, work, task, NoteBrief::Operation);
            } else {
                super::results::clear(core, task);
                drop(core.contexts.remove(&task));
                drop(core.dependency_results.remove(&task));
                drop(core.transcripts.remove(&task));
                if crate::routing::effect::note::wait_for_notes(core, work, task, NoteBrief::Activation) {
                    work.push(Work::Event(Event::Tasks(tasks::Event::PreparationWaiting { task })));
                }
            }
        }
        notes::Refusal::Full => fail(work, task, brief::GatherFate::Capacity, b"no room for the index"),
        notes::Refusal::Oversized
        | notes::Refusal::Exists
        | notes::Refusal::Missing
        | notes::Refusal::Moved { .. }
        | notes::Refusal::PastEnd { .. }
        | notes::Refusal::PastBound { .. }
        | notes::Refusal::RevisionExhausted => {
            fail(work, task, brief::GatherFate::Permanent, b"the index could not be read");
        }
    }
    Queue::uncapped()
}

fn fail(work: &mut Queue<Work>, task: u64, fate: brief::GatherFate, words: &'static [u8]) {
    work.push(Work::PreparationFailed {
        task,
        failure: brief::GatherFailure { fate, section: b"preferences", words: Box::from(words) },
    });
}

fn render(lines: &List<notes::Line>) -> Box<[u8]> {
    if lines.is_empty() {
        return skein_lib::bytes::copy_of(b"None kept.\n");
    }
    let mut length = 0_usize;
    for (index, line) in lines.iter().enumerate() {
        let place = u64::try_from(index).expect("bounded party entries").checked_add(1).expect("place from one");
        length = length
            .saturating_add(Decimal::of(place).as_bytes().len())
            .saturating_add(b". \n".len())
            .saturating_add(line.description.len());
    }
    let mut writer = Writer::new(length);
    for (index, line) in lines.iter().enumerate() {
        let place = u64::try_from(index).expect("bounded party entries").checked_add(1).expect("place from one");
        writer.put(Decimal::of(place).as_bytes()).expect("measured place");
        writer.put(b". ").expect("measured separator");
        writer.put(&line.description).expect("measured description");
        writer.put(b"\n").expect("measured newline");
    }
    writer.finish()
}

pub(crate) fn text(core: &Core, task: u64) -> Option<Box<[u8]>> {
    party(core, task)?;
    core.preferences.texts.get(&task).cloned()
}
