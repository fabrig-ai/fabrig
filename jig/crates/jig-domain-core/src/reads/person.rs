//! A run's read as its task's person (`jig/docs/design/domain/engine.md`, 7.3).
use super::{Answer, Destination, Read, Why};
use crate::{CallKey, Core, Limits, Now, Request, ToolKind};
use alloc::boxed::Box;
use jig_domain_people::{ApplicationScope, Proposer};
use jig_domain_tasks::Party;
use skein_lib::{Env, Queue, ReplyTo};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Admission {
    Project,
    Task(u64),
    Refused,
}

pub(crate) fn admitted(read: &Read, project: u32) -> Admission {
    let named = match read {
        Read::Project { project } | Read::Spend { project, .. } | Read::Notes { project, .. } => Some(*project),
        Read::Tasks { query, .. } => query.project,
        Read::Inbox { filter, .. } => filter.project,
        Read::ScopedNotes { scope, .. } => match scope {
            crate::notes::Scope::Project { project } | crate::notes::Scope::Goal { project, .. } => Some(*project),
            crate::notes::Scope::Deployment | crate::notes::Scope::Resources { .. } => None,
        },
        Read::Application { scope, .. } => match scope {
            ApplicationScope::Project(project) => Some(*project),
            ApplicationScope::Own | ApplicationScope::Deployment => None,
        },
        Read::Task { task }
        | Read::TaskText { task, .. }
        | Read::Activity { task, .. }
        | Read::ActivityBeside { task, .. }
        | Read::Turn { task, .. }
        | Read::EndedDelegates { task, .. }
        | Read::History { task, .. }
        | Read::Result { task }
        | Read::Reply { task, .. }
        | Read::Question { task, .. }
        | Read::Escalation { task }
        | Read::PersonTask { task }
        | Read::Proposal { proposer: Proposer::Task(task), .. } => return Admission::Task(*task),
        Read::Frame
        | Read::Deployment
        | Read::Settings
        | Read::Setting { .. }
        | Read::Revision { .. }
        | Read::Revisions { .. }
        | Read::Proposal { proposer: Proposer::Party(_), .. } => None,
    };
    if named == Some(project) && project != jig_domain_people::DEPLOYMENT {
        Admission::Project
    } else {
        Admission::Refused
    }
}

pub(crate) fn run(core: &mut Core, env: &Env<Limits>, to: ReplyTo, key: CallKey, read: Read) -> Queue<Request> {
    if let Some(part) = core.authorize_tool(key, ToolKind::Ask) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::EffectAnswer { to, key, part })));
        return out;
    }
    let task = core.tasks.task(key.task).expect("authorized run's task");
    let party = match task.requester {
        Party::Person(party) => party,
        Party::Task(_) | Party::Deployment { .. } => return unknown(to, key),
    };
    let project = task.project;
    if !valid(core, key, project, party) {
        return unknown(to, key);
    }
    let mut owner = None;
    match admitted(&read, project) {
        Admission::Project => {}
        Admission::Task(number) => match core.tasks.task(number) {
            Some(row) if row.project != project => return unknown(to, key),
            Some(_) => {}
            None => {
                if let Read::Activity { .. }
                | Read::ActivityBeside { .. }
                | Read::History { .. }
                | Read::EndedDelegates { .. } = &read
                {
                    owner = Some(number);
                }
            }
        },
        Admission::Refused => return unknown(to, key),
    }
    if let Some(task) = owner {
        let destination = Destination::Person { to, key, project, owner_read: Some(Box::new(read)) };
        return super::start_jig(
            core,
            env,
            destination,
            0,
            party,
            Read::TaskText { task, field: super::TaskTextField::Specification, from: 0 },
            false,
        );
    }
    let destination = Destination::Person { to, key, project, owner_read: None };
    if let Read::Application { .. } = &read {
        return super::application(core, destination, party, &read);
    }
    super::start_jig(core, env, destination, 0, party, read, false)
}

pub(super) fn valid(core: &Core, key: CallKey, project: u32, party: u64) -> bool {
    core.current_proof(key.task, key.attempt)
        && match core.tasks.task(key.task) {
            Some(task) => task.project == project && task.requester == Party::Person(party),
            None => false,
        }
        && core.open_project(project)
        && core.people.role(party, project).is_some()
}

pub(super) fn answer(to: ReplyTo, key: CallKey, answer: Answer) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::PersonAnswer { to, key, answer: Box::new(answer) })));
    out
}

fn unknown(to: ReplyTo, key: CallKey) -> Queue<Request> {
    answer(to, key, Answer::None(Why::Unknown))
}

/// An empty retained page still needs its task's project proved before paging.
pub(super) fn owner_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    name: skein_lib::Id<super::Pending>,
    result: super::Loaded,
) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("pending owner check");
    let Destination::Person { to, key, project, owner_read: Some(read) } =
        core::mem::replace(&mut pending.destination, Destination::Done)
    else {
        unreachable!("person owner check")
    };
    let range = pending.range.clone();
    let party = pending.party;
    core.reads.retire(name);
    if !core.current_proof(key.task, key.attempt) {
        return Queue::uncapped();
    }
    if !valid(core, key, project, party) {
        return unknown(to, key);
    }
    let checked = match result {
        super::Loaded::Page { rows, next } => {
            super::rows::person_owner(core, &env.limits, &range, &rows, next.as_ref(), project)
        }
        super::Loaded::Busy => Err(Why::Busy),
        super::Loaded::TooLinked { joins } => Err(Why::TooLinked { joins }),
        super::Loaded::Gone => Err(Why::Gone),
        super::Loaded::TurnProblem { .. } => Err(Why::Unknown),
    };
    if let Err(why) = checked {
        return answer(to, key, Answer::None(why));
    }
    // The owner check lends its shared load admission while
    // the person's original read enters the same party pipeline.
    super::start_jig(core, env, Destination::Person { to, key, project, owner_read: None }, 0, party, *read, true)
}
