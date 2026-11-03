//! Index-only public reads and their completion (jig/docs/design/domain/people.md, 13.9–13.10).
//!
//! | State | Event | Next | Output |
//! | --- | --- | --- | --- |
//! | absent | excluded read | absent | held unknown or busy |
//! | absent | jig document | waiting | held durability callback |
//! | absent | application read | asking | connector ask |
//! | asking | connector answer | waiting | held durability callback |
//! | waiting | durable | absent | held rechecked answer |
//! | absent or waiting | unasked connector answer | unchanged | none |
use super::{
    PublicAnswer, PublicApplication, PublicBoard, PublicCard, PublicDocument, PublicGoal, PublicLine, PublicProject,
    PublicProjects, PublicRead,
    index::{Entry, Project},
};
use crate::{Ask, Core, Held, Limits, Request};
use alloc::boxed::Box;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Token};

/// One read, bounded by `Limits::load_slots`, apart from authenticated loads.
#[derive(Debug)]
pub(crate) struct Pending {
    reply: ReplyTo,
    read: PublicRead,
    state: State,
}

#[derive(Debug)]
enum State {
    Asking { public: Box<[u64]> },
    Waiting { document: Option<PublicDocument> },
}

pub(crate) fn start(core: &mut Core, reply: ReplyTo, read: PublicRead) -> Queue<Request> {
    if read.owner == 0 && read.kind == 1 && read.project.is_none() && read.task.is_none() && read.version == 1 {
        let document = projects(core);
        return keep(core, reply, read, State::Waiting { document: Some(document) });
    }
    let project = project_number(&read);
    // A failed conversion and an absent project share the index's missing key.
    let entry = core.public_index.project(project.unwrap_or(0));
    let project = match entry {
        None => return terminal(reply, PublicAnswer::Unknown),
        Some(Entry::Busy) => return terminal(reply, PublicAnswer::Busy),
        Some(Entry::Ready(project)) => project,
    };
    if let Some(task) = read.task
        && project.goal_of(task).is_none()
    {
        return terminal(reply, PublicAnswer::Unknown);
    }
    if read.owner == 0 {
        let Some(document) = jig(project, read) else { return terminal(reply, PublicAnswer::Unknown) };
        keep(core, reply, read, State::Waiting { document: Some(document) })
    } else {
        let connector = read.owner.checked_sub(1).expect("application owner");
        if !core.connectors.contains(&connector) {
            return terminal(reply, PublicAnswer::Unknown);
        }
        let public = project.public_tasks();
        keep(core, reply, read, State::Asking { public })
    }
}

fn project_number(read: &PublicRead) -> Option<u32> {
    match read.project {
        Some(number) => u32::try_from(number).ok(),
        None => None,
    }
}

fn keep(core: &mut Core, reply: ReplyTo, read: PublicRead, state: State) -> Queue<Request> {
    let pending = Pending { reply, read, state };
    let id = match core.public_reads.insert(Some(pending)) {
        Ok(id) => id,
        Err(Some(pending)) => return terminal(pending.reply, PublicAnswer::Busy),
        Err(None) => unreachable!("a read owns its pending value"),
    };
    let pending = core.public_reads.get(id).expect("inserted public read").as_ref().expect("live public read");
    let request = match &pending.state {
        State::Asking { public } => Request::Ask {
            connector: read.owner.checked_sub(1).expect("application owner"),
            ask: Ask::Public {
                reply: ReplyTo::new(id.token()),
                kind: read.kind,
                version: read.version,
                project: project_number(&read).expect("admitted project"),
                task: read.task,
                public: public.clone(),
            },
        },
        State::Waiting { .. } => Request::Held(Box::new(Held::PublicWait { read: id.token() })),
    };
    let mut out = Queue::uncapped();
    out.push(request);
    out
}

pub(crate) fn answered(core: &mut Core, reply: ReplyTo, answer: Option<PublicApplication>) -> Queue<Request> {
    let id = Id::from_token(reply.into_token());
    let Some(Some(pending)) = core.public_reads.get_mut(id) else { return Queue::uncapped() };
    let State::Asking { public } = &pending.state else { return Queue::uncapped() };
    let document = match answer {
        Some(answer) => {
            let mut allowed = true;
            for task in &answer.tasks {
                allowed &= public.binary_search(task).is_ok();
            }
            if allowed { Some(PublicDocument::Application(answer)) } else { None }
        }
        None => None,
    };
    pending.state = State::Waiting { document };
    let mut out = Queue::uncapped();
    out.push(Request::Held(Box::new(Held::PublicWait { read: id.token() })));
    out
}

pub(crate) fn durable(core: &mut Core, env: &Env<Limits>, read: Token) -> Queue<Request> {
    let id = Id::from_token(read);
    let Some(Some(pending)) = core.public_reads.get(id) else { return Queue::uncapped() };
    let State::Waiting { .. } = &pending.state else { return Queue::uncapped() };
    let pending = core.public_reads.get_mut(id).expect("pending read").take().expect("live read");
    core.public_reads.retire(id);
    core.public_reads.reclaim();
    let State::Waiting { document } = pending.state else { unreachable!("durable read was waiting") };
    let answer = match document {
        Some(document) => match complete(core, pending.read, document) {
            Some(document) => PublicAnswer::Document { decided: env.now, document },
            None => PublicAnswer::Unknown,
        },
        None => PublicAnswer::Unknown,
    };
    terminal(pending.reply, answer)
}

fn complete(core: &Core, read: PublicRead, document: PublicDocument) -> Option<PublicDocument> {
    if let PublicDocument::Projects(projects) = &document {
        for project in &projects.projects {
            if !core.public_project(project.project) {
                return None;
            }
        }
        return Some(self::projects(core));
    }
    let number = project_number(&read)?;
    if !core.public_project(number) {
        return None;
    }
    let Some(Entry::Ready(project)) = core.public_index.project(number) else { return None };
    if let Some(task) = read.task
        && project.goal_of(task).is_none()
    {
        return None;
    }
    if !names_public(project, &document) {
        return None;
    }
    match document {
        PublicDocument::Application(answer) => Some(PublicDocument::Application(answer)),
        PublicDocument::Board(_) | PublicDocument::Card(_) => jig(project, read),
        PublicDocument::Projects(_) => unreachable!("projects completed above"),
    }
}

fn names_public(project: &Project, document: &PublicDocument) -> bool {
    match document {
        PublicDocument::Projects(_) => unreachable!("projects have no task names"),
        PublicDocument::Application(answer) => {
            for &task in &answer.tasks {
                if project.goal_of(task).is_none() {
                    return false;
                }
            }
        }
        PublicDocument::Board(board) => {
            for goal in board.live.iter().chain(board.ended.iter()) {
                if !line_public(project, &goal.line) {
                    return false;
                }
                for line in &goal.tree {
                    if !line_public(project, line) {
                        return false;
                    }
                }
            }
        }
        PublicDocument::Card(card) => {
            if !line_public(project, &card.line) {
                return false;
            }
            for &task in &card.lineage {
                if project.goal_of(task).is_none() {
                    return false;
                }
            }
            for line in &card.delegates {
                if !line_public(project, line) {
                    return false;
                }
            }
        }
    }
    true
}

fn line_public(project: &Project, line: &PublicLine) -> bool {
    project.goal_of(line.task).is_some()
        && match line.parent {
            Some(parent) => project.goal_of(parent).is_some(),
            None => true,
        }
}

fn projects(core: &Core) -> PublicDocument {
    let mut projects = List::with_capacity(core.projects.capacity());
    // The deployment's project bound is independent of public reads in flight.
    for (project, entry) in core.public_index.projects() {
        projects.push(PublicProject { project, name: entry.name.clone() }).expect("bounded public projects");
    }
    PublicDocument::Projects(PublicProjects { projects: projects.into_boxed() })
}

fn jig(project: &Project, read: PublicRead) -> Option<PublicDocument> {
    if read.version != 1 {
        return None;
    }
    match (read.kind, read.task) {
        (2, None) => {
            let mut live = List::with_capacity(u32::try_from(project.live().len()).expect("bounded goals"));
            for &goal in project.live() {
                let (tree, folded) = project.tree(goal).expect("indexed goal");
                live.push(PublicGoal { line: project.line(goal)?.clone(), tree: lines(project, tree), folded })
                    .expect("bounded goals");
            }
            let mut ended = List::with_capacity(u32::try_from(project.ended().len()).expect("bounded ended goals"));
            for goal in project.ended() {
                ended
                    .push(PublicGoal { line: goal.line.clone(), tree: goal.tree.clone(), folded: goal.folded })
                    .expect("bounded ended goals");
            }
            Some(PublicDocument::Board(PublicBoard {
                project: project_number(&read)?,
                name: project.name.clone(),
                live: live.into_boxed(),
                live_folded: project.live_folded(),
                ended: ended.into_boxed(),
            }))
        }
        (3, Some(task)) => {
            let (delegates, folded) = project.delegates(task);
            Some(PublicDocument::Card(PublicCard {
                line: project.line(task)?.clone(),
                lineage: Box::from(project.lineage(task)?),
                delegates: lines(project, delegates),
                folded,
            }))
        }
        _ => None,
    }
}

fn lines(project: &Project, ids: &[u64]) -> Box<[PublicLine]> {
    let mut lines = List::with_capacity(u32::try_from(ids.len()).expect("bounded public lines"));
    for &task in ids {
        lines.push(project.line(task).expect("indexed public task").clone()).expect("bounded public lines");
    }
    lines.into_boxed()
}

fn terminal(reply: ReplyTo, answer: PublicAnswer) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Held(Box::new(Held::PublicRead { reply, answer })));
    out
}
