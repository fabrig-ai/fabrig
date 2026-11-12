//! A standing task's Now, gathered before its notes index and placed whole
//! after its task (`jig/docs/design/domain/tasks/standing.md`, section 6).

use crate::{
    Core, Event, Limits,
    routing::{NoteBrief, NoteRoute, Work},
};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, Env, List, Map, Queue, bytes, length::len_u64};

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
    drop(core.now.texts.remove(&task));
}

fn scope(core: &Core, task: u64) -> Option<notes::Scope> {
    let context = core.contexts.get(&task)?;
    if context.ended_by != tasks::EndedBy::Standing {
        return None;
    }
    Some(notes::Scope::Goal { project: context.project, goal: task })
}

/// Only saved entries reach here, after whole or range revisions have their complete body.
pub(crate) fn written(core: &Core, work: &mut Queue<Work>, entry: &notes::Entry) {
    let (task, attempt) = match entry.author {
        notes::Author::Task { task, attempt } => (task, attempt),
        notes::Author::Party { .. } => return,
    };
    let Some(row) = core.tasks.task(task) else { return };
    if row.flags.ended_by != tasks::EndedBy::Standing
        || entry.scope != (notes::Scope::Goal { project: row.project, goal: task })
        || entry.description.as_ref() != b"now"
    {
        return;
    }
    let end = bytes::line_end(&entry.body).unwrap_or(entry.body.len());
    let line = entry.body.get(..end).expect("line end within body");
    let line = bytes::utf8_prefix(line, skein_lib::length::usize_of(tasks::NOW_LINE));
    work.push(Work::Event(Event::Tasks(tasks::Event::NowWritten { task, attempt, line: Box::from(line) })));
}

fn current(core: &Core, task: u64) -> bool {
    match core.contexts.get(&task) {
        Some(context) => super::parts::preparation_current(core, context) && scope(core, task).is_some(),
        None => false,
    }
}

pub(crate) fn gather(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Work>, task: u64) -> bool {
    let Some(scope) = scope(core, task) else { return false };
    for _ in 0..core.now.texts.len() {
        let mut ended = None;
        for (&number, _) in &core.now.texts {
            if !core.contexts.contains_key(&number) {
                ended = Some(number);
                break;
            }
        }
        let Some(ended) = ended else { break };
        clear(core, ended);
    }
    if core.now.texts.contains_key(&task) {
        return false;
    }
    if core.now.texts.len() == core.now.texts.capacity() {
        fail(work, task, brief::GatherFate::Capacity, b"no room for the index");
        return true;
    }
    let mut scopes = List::with_capacity(1);
    scopes.push(scope).expect("one task scope");
    let owner = core.begin_note_route(NoteRoute::Now { task });
    work.push(Work::Event(Event::Notes(notes::Event::Index { owner, scopes, most: env.limits.notes.lines })));
    true
}

pub(crate) fn indexed(core: &mut Core, work: &mut Queue<Work>, task: u64, lines: List<notes::Line>) {
    if !current(core, task) {
        return;
    }
    // The index has no description filter. Choose within this scope's page.
    let mut first = None;
    for line in &lines {
        if line.description.as_ref() == b"now" && Some(line.scope.clone()) == scope(core, task) {
            first = Some(match first {
                Some(name) => u64::min(name, line.name),
                None => line.name,
            });
        }
    }
    match first {
        Some(name) => {
            let owner = core.begin_note_route(NoteRoute::Now { task });
            work.push(Work::Event(Event::Notes(notes::Event::Recall {
                owner,
                by: notes::Recall::Name { name },
                page: 0,
            })));
        }
        None => retain(core, work, task, render(None, 0, core.contexts.get(&task).expect("preparing task").numbers)),
    }
}

pub(crate) fn recalled(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    entries: List<notes::Entry>,
    more: bool,
) {
    if !current(core, task) {
        return;
    }
    if entries.len() == 1
        && !more
        && let Some(entry) = entries.get(0)
        && Some(entry.scope.clone()) == scope(core, task)
        && entry.description.as_ref() == b"now"
    {
        retain(
            core,
            work,
            task,
            render(
                Some(entry),
                env.limits.brief_core_budgets.now,
                core.contexts.get(&task).expect("preparing task").numbers,
            ),
        );
    } else {
        fail(work, task, brief::GatherFate::Permanent, b"the note could not be read");
    }
}

fn retain(core: &mut Core, work: &mut Queue<Work>, task: u64, text: Box<[u8]>) {
    if core.now.texts.insert(task, text).is_ok() {
        work.push(Work::StartBrief { task });
    } else {
        fail(work, task, brief::GatherFate::Capacity, b"no room for the note");
    }
}

pub(crate) fn plan(
    core: &Core,
    limits: &Limits,
    work: &mut Queue<Work>,
    task: u64,
    wanted: &mut List<brief::Planned>,
) -> bool {
    if scope(core, task).is_none() {
        return true;
    }
    let Some(text) = text(core, task) else {
        fail(work, task, brief::GatherFate::Permanent, b"the note could not be read");
        return false;
    };
    // The body has its own bound; keep the title, instruction and overflow line too.
    let size = u32::try_from(text.len()).expect("bounded Now text");
    let position = wanted.len();
    if wanted
        .push(brief::Planned::Core { kind: brief::Core::Now, text, limit: size, priority: 0, required: true })
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

pub(crate) fn refused(core: &mut Core, work: &mut Queue<Work>, task: u64, why: notes::Refusal) {
    clear(core, task);
    if !current(core, task) {
        return;
    }
    match why {
        notes::Refusal::Busy => {
            if core.notes.busy() {
                core.pending_note_briefs.insert(task, NoteBrief::Operation).expect("bounded waiting briefs");
            } else {
                super::results::clear(core, task);
                drop(core.contexts.remove(&task));
                drop(core.dependency_results.remove(&task));
                drop(core.transcripts.remove(&task));
                core.pending_note_briefs.insert(task, NoteBrief::Activation).expect("bounded deferred briefs");
                work.push(Work::Event(Event::Tasks(tasks::Event::PreparationWaiting { task })));
            }
        }
        notes::Refusal::Full => fail(work, task, brief::GatherFate::Capacity, b"no room for the note"),
        notes::Refusal::Oversized
        | notes::Refusal::Exists
        | notes::Refusal::Missing
        | notes::Refusal::Moved { .. }
        | notes::Refusal::PastEnd { .. }
        | notes::Refusal::PastBound { .. }
        | notes::Refusal::RevisionExhausted => {
            fail(work, task, brief::GatherFate::Permanent, b"the note could not be read");
        }
    }
}

fn fail(work: &mut Queue<Work>, task: u64, fate: brief::GatherFate, words: &'static [u8]) {
    work.push(Work::PreparationFailed {
        task,
        failure: brief::GatherFailure { fate, section: b"now", words: Box::from(words) },
    });
}

fn render(entry: Option<&notes::Entry>, bound: u32, numbers: tasks::Numbers) -> Box<[u8]> {
    let spent = numbers.spent.checked_add(numbers.spent_below).expect("bounded period spend");
    let period = bytes::concat(&[
        b"This period: spent ",
        Decimal::of(spent).as_bytes(),
        b" of ",
        Decimal::of(numbers.budget).as_bytes(),
        b".\n",
    ]);
    let instruction = b"Write your Now in your own task's goal scope, described `now`, before you park.\n";
    let Some(entry) = entry else {
        return bytes::concat(&[b"None written yet.\n", &period, instruction]);
    };
    let name = Decimal::of(entry.name);
    let revision = Decimal::of(u64::from(entry.revision));
    let changed = Decimal::of(entry.changed.as_nanos());
    let title = bytes::concat(&[
        b"Note ",
        name.as_bytes(),
        b", revision ",
        revision.as_bytes(),
        b", written at ",
        changed.as_bytes(),
        b" ns since Unix epoch.\n",
    ]);
    let body = bytes::utf8_prefix(&entry.body, skein_lib::length::usize_of(bound));
    let marker = if body.len() < entry.body.len() {
        bytes::concat(&[
            b"\n[Now: ",
            Decimal::of(len_u64(entry.body.len())).as_bytes(),
            b" bytes of ",
            Decimal::of(u64::from(bound)).as_bytes(),
            b"; shorten it to the bound.]\n",
        ])
    } else {
        Box::default()
    };
    bytes::concat(&[&title, &period, instruction, b"\n", body, &marker])
}

pub(crate) fn text(core: &Core, task: u64) -> Option<Box<[u8]>> {
    scope(core, task)?;
    core.now.texts.get(&task).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_figures_follow_the_title_with_or_without_a_now() {
        let numbers = tasks::Numbers { budget: 10_000, spent: 1200, spent_below: 34, reserved: 100 };
        let entry = notes::Entry {
            name: 20,
            scope: notes::Scope::Goal { project: 1, goal: 1 },
            description: b"now".as_slice().into(),
            body: b"Watching".as_slice().into(),
            references: List::with_capacity(0),
            author: notes::Author::Task { task: 1, attempt: 3 },
            revision: 2,
            changed: skein_lib::Wall::EPOCH,
        };
        for (entry, title) in [
            (Some(&entry), b"Note 20, revision 2, written at 0 ns since Unix epoch.\n".as_slice()),
            (None, b"None written yet.\n".as_slice()),
        ] {
            let rendered = render(entry, 100, numbers);
            let expected = bytes::concat(&[title, b"This period: spent 1234 of 10000.\n"]);
            assert!(rendered.starts_with(&expected));
        }
    }
}
