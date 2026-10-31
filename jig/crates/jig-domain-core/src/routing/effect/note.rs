//! Party note edits and run note requests translated through the notes child
//! (`domain/engine.md`, section 4.4; `domain/people.md`, section 5.1).

use crate::{
    CallPart, Core, Event, Held, Key, Limits, Now, Record, Request, Write,
    routing::{
        NoteBrief, NoteRoute, Work,
        party::dispatch::{Party, Routed},
        refusals::note_person_refusal,
        run::calls::call_decision,
    },
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue};

pub(crate) fn route_notes(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    event: notes::Event,
) -> Queue<Request> {
    let mut child = Queue::uncapped();
    crate::routing::step::step_notes(core, env, event, &mut child);
    tag_notes(core, env, work, child)
}

pub(crate) fn person_note_scope(project: u32, scope: people::NoteScope) -> notes::Scope {
    match scope {
        people::NoteScope::Deployment => notes::Scope::Deployment,
        people::NoteScope::Project => notes::Scope::Project { project },
        people::NoteScope::Goal { goal } => notes::Scope::Goal { project, goal },
        people::NoteScope::Resources { connector, pattern } => {
            let mut segments = List::with_capacity(u32::try_from(pattern.segments.len()).expect("bounded pattern"));
            for segment in pattern.segments {
                segments.push(segment).expect("pattern segment room");
            }
            let last = match pattern.last {
                people::Last::Exact(value) => notes::Last::Exact(value),
                people::Last::Open(value) => notes::Last::Open(value),
            };
            notes::Scope::Resources {
                project,
                connector,
                pattern: notes::Pattern { segments: segments.into_boxed(), last },
            }
        }
    }
}

pub(crate) fn person_note_change(change: people::NoteChange) -> notes::Change {
    match change {
        people::NoteChange::Correct { description, body, references, recalled } => {
            let mut kept = List::with_capacity(u32::try_from(references.len()).expect("bounded references"));
            for reference in references {
                kept.push(reference).expect("reference room");
            }
            notes::Change::Correct { description, body, references: kept, recalled }
        }
        people::NoteChange::Delete { recalled } => notes::Change::Delete { recalled },
    }
}

pub(crate) fn tag_notes(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    mut child: Queue<notes::Request>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    for _ in 0..child.len() {
        match child.pop().expect("note output count") {
            notes::Request::ProjectDropped { owner } => {
                crate::admission::deletion::notes_dropped(core, env, owner, &mut out);
            }
            notes::Request::Save { record } => out.push(Request::Write(Write::Save(Record::Notes(record)))),
            notes::Request::Erase { key } => out.push(Request::Write(Write::Erase(Key::Notes(key)))),
            notes::Request::Load { owner, range } => {
                out.push(Request::Held(Box::new(Held::NotesLoad { owner, range })));
            }
            notes::Request::Written { owner, name, revision } => match core.take_note(owner) {
                Some(NoteRoute::Call { to, key }) => {
                    call_decision(core, &mut out, to, key, CallPart::NoteWritten { name, revision });
                }
                Some(NoteRoute::Person { request }) => work.push(Work::Event(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::NoteEdited { name },
                }))),
                Some(NoteRoute::Brief { .. } | NoteRoute::Deletion { .. }) => {
                    unreachable!("note reads and project deletion cannot write an entry")
                }
                None => {}
            },
            notes::Request::Deleted { owner, name } => match core.take_note(owner) {
                Some(NoteRoute::Person { request }) => work.push(Work::Event(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::NoteEdited { name },
                }))),
                Some(NoteRoute::Call { .. } | NoteRoute::Brief { .. } | NoteRoute::Deletion { .. }) => {
                    unreachable!("only a party deletes a note")
                }
                None => {}
            },
            notes::Request::Recalled { owner, entries, more } => match core.take_note(owner) {
                Some(NoteRoute::Call { to, key }) => {
                    let part = recalled_part(core, key.task, entries, more);
                    call_decision(core, &mut out, to, key, part);
                }
                Some(NoteRoute::Person { .. } | NoteRoute::Brief { .. } | NoteRoute::Deletion { .. }) => {
                    unreachable!("a run owns this recall")
                }
                None => {}
            },
            notes::Request::Indexed { owner, lines, more } => match core.take_note(owner) {
                Some(NoteRoute::Brief { task }) => work.push(Work::BriefNotes { task, lines, more }),
                Some(NoteRoute::Call { .. } | NoteRoute::Person { .. } | NoteRoute::Deletion { .. }) => {
                    unreachable!("brief preparation owns note index")
                }
                None => {}
            },
            notes::Request::Refused { owner, why } => match core.take_note(owner) {
                Some(NoteRoute::Call { to, key }) => match why {
                    notes::Refusal::Busy => {
                        let _pending = core.pending_calls.remove(&key);
                        out.push(Request::Now(Box::new(Now::NoteBusy { to, key })));
                    }
                    notes::Refusal::Oversized
                    | notes::Refusal::Full
                    | notes::Refusal::Exists
                    | notes::Refusal::Missing
                    | notes::Refusal::Moved { .. }
                    | notes::Refusal::RevisionExhausted => {
                        call_decision(core, &mut out, to, key, CallPart::NoteRefused(why));
                    }
                },
                Some(NoteRoute::Person { request }) => {
                    work.push(Work::Event(Event::People(people::Event::Decided {
                        request,
                        outcome: note_person_refusal(why),
                    })));
                }
                Some(NoteRoute::Brief { task }) => match why {
                    notes::Refusal::Busy => {
                        if core.notes.busy() {
                            core.pending_note_briefs
                                .insert(task, NoteBrief::Operation)
                                .expect("brief count bounds waiting notes");
                        } else {
                            defer_brief(core, work, task);
                        }
                    }
                    notes::Refusal::Oversized
                    | notes::Refusal::Full
                    | notes::Refusal::Exists
                    | notes::Refusal::Missing
                    | notes::Refusal::Moved { .. }
                    | notes::Refusal::RevisionExhausted => {
                        work.push(Work::Event(Event::Tasks(tasks::Event::PreparationFailed { task })));
                    }
                },
                Some(NoteRoute::Deletion { project }) => crate::admission::deletion::notes_refused(core, project),
                None => out.push(Request::Now(Box::new(Now::NotesRefused { owner, why }))),
            },
        }
    }
    resume_brief(core, work);
    out
}

fn resume_brief(core: &mut Core, work: &mut Queue<Work>) {
    if !core.notes.busy() {
        let mut waiting = None;
        for (&task, state) in &core.pending_note_briefs {
            if *state == NoteBrief::Operation {
                waiting = Some(task);
                break;
            }
        }
        if let Some(task) = waiting {
            let _pending = core.pending_note_briefs.remove(&task);
            work.push(Work::StartBrief { task });
        }
    }
}

fn defer_brief(core: &mut Core, work: &mut Queue<Work>, task: u64) {
    crate::routing::run::results::clear(core, task);
    drop(core.contexts.remove(&task));
    drop(core.dependency_results.remove(&task));
    drop(core.transcripts.remove(&task));
    core.pending_note_briefs.insert(task, NoteBrief::Activation).expect("brief count bounds deferred activations");
    work.push(Work::Event(Event::Tasks(tasks::Event::PreparationWaiting { task })));
}

pub(crate) fn recalled_part(core: &Core, task: u64, entries: List<notes::Entry>, more: bool) -> CallPart {
    for entry in &entries {
        if !core.note_readable(task, &entry.scope) {
            return CallPart::ToolDenied {
                answer: authority::Answer::Refuse,
                findings: Box::new([authority::Finding::Scope { source: authority::Source::Task }]),
            };
        }
    }
    CallPart::NoteRecalled { entries: entries.into_boxed(), more }
}

pub(crate) fn edit_note_request(
    core: &mut Core,
    routed: &mut Routed,
    party: Party,
    name: u64,
    scope: Box<people::NoteScope>,
    change: people::NoteChange,
) {
    let Party { request, person, project, .. } = party;

    let act = match &change {
        people::NoteChange::Correct { .. } => crate::acts::Act::EditNote,
        people::NoteChange::Delete { .. } => crate::acts::Act::DeleteNote,
    };
    if let Err(why) = crate::acts::note(core, person, project, &scope, act) {
        routed.work.push(Work::Event(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Refused(why),
        })));
    } else {
        let owner = core.begin_note_route(NoteRoute::Person { request });
        let scope = person_note_scope(project, *scope);
        let change = person_note_change(change);
        routed.work.push(Work::Event(Event::Notes(notes::Event::Edit { owner, party: person, name, scope, change })));
    }
}
