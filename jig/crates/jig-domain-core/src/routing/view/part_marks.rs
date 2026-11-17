//! Part input commits mark matching queries, never task activity (domain/engine/views.md, 2).

use crate::{
    Core, PartInputs,
    documents::{Subject, task_query},
};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::List;

#[derive(Clone, Copy)]
enum Input {
    Notes,
    DelegateNotes,
    Records(u16),
}

fn uses(inputs: PartInputs, connector: u16, input: Input) -> bool {
    match input {
        Input::Notes => inputs.notes,
        Input::DelegateNotes => inputs.delegate_notes,
        Input::Records(owner) => inputs.records && connector == owner,
    }
}

fn touch(core: &mut Core, task: u64, input: Input) {
    let row = core.tasks.task(task);
    let ended = match row {
        Some(row) => match row.phase {
            tasks::Phase::Ended(_) => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                false
            }
        },
        None => true,
    };
    if ended {
        for kind in &core.part_kinds {
            if uses(kind.inputs, kind.connector, input) {
                core.ended_parts_changed();
                break;
            }
        }
        return;
    }
    let row = row.expect("live input owner");
    for (name, watched) in &core.view_names {
        let Subject::Query { query } = &watched.subject else { continue };
        if !task_query::may_touch(query, row) {
            continue;
        }
        let mut affected = false;
        for ask in &query.parts {
            for kind in &core.part_kinds {
                affected |=
                    (ask.owner, ask.kind) == (kind.owner, kind.kind) && uses(kind.inputs, kind.connector, input);
            }
        }
        if affected {
            let _new = core.changed_views.insert(name).expect("bounded part input marks");
        }
    }
}

pub(crate) fn records(core: &mut Core, connector: u16, tasks: &List<u64>) {
    for task in tasks {
        touch(core, *task, Input::Records(connector));
    }
}

pub(crate) fn note(core: &mut Core, scope: &notes::Scope) {
    if let notes::Scope::Goal { project, goal } = scope {
        let requester = match core.tasks.task(*goal) {
            Some(row) => Some(row.requester),
            None => None,
        };
        touch(core, *goal, Input::Notes);
        match requester {
            Some(tasks::Party::Task(parent)) => touch(core, parent, Input::DelegateNotes),
            None => {
                touch(core, *goal, Input::DelegateNotes);
                // Ended owners retain no requester in the hub. Any live row in this project may be it.
                let mut candidates =
                    List::with_capacity(core.tasks.view_tasks().len().try_into().expect("bounded task views"));
                for task in core.tasks.view_tasks().as_ref() {
                    if core.tasks.task(task.number).expect("live note requester").project == *project {
                        candidates.push(task.number).expect("bounded requester candidates");
                    }
                }
                for task in candidates.into_boxed() {
                    touch(core, task, Input::DelegateNotes);
                }
            }
            Some(tasks::Party::Person(_) | tasks::Party::Deployment { .. }) => {}
        }
        return;
    }
    let mut affected = List::with_capacity(core.tasks.view_tasks().len().try_into().expect("bounded task views"));
    for task in core.tasks.view_tasks().as_ref() {
        if core.note_readable(task.number, scope) {
            affected.push(task.number).expect("one input per live task");
        }
    }
    for task in affected.into_boxed() {
        let requester = core.tasks.task(task).expect("live note owner").requester;
        touch(core, task, Input::Notes);
        if let tasks::Party::Task(parent) = requester {
            touch(core, parent, Input::DelegateNotes);
        }
    }
    // Ended rows are not held in the live task hub. A scope may contain their notes too.
    match scope {
        notes::Scope::Goal { .. } | notes::Scope::Party { .. } => {}
        notes::Scope::Deployment | notes::Scope::Project { .. } | notes::Scope::Resources { .. } => {
            touch(core, 0, Input::Notes);
            touch(core, 0, Input::DelegateNotes);
        }
    }
}
