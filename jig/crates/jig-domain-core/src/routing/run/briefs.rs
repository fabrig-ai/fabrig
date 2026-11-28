//! Connector plans and completed briefs belong to the preparation, not a gathering slot
//! (jig/docs/design/domain/engine/briefs.md, 1.4).

use crate::{Core, Limits, Now, Request, routing::children::append_tasks, vocabulary::connector_ask};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_contract::{Ask, parts};
use skein_lib::{Env, List, Map, Queue, Token};

#[derive(Hash, Debug)]
pub(crate) enum Sections {
    Planning(Planning),
    Complete(Box<[brief::GatherPlaced]>),
}

#[derive(Hash, Debug)]
pub(crate) struct Planning {
    core: Box<[brief::Planned]>,
    documents: Box<[u16]>,
    answers: Map<u16, Option<Box<[parts::Section]>>>,
    waiting: u32,
}

pub(super) fn plan(core: &mut Core, env: &Env<Limits>, task: u64, wanted: List<brief::Planned>) -> Queue<Request> {
    let connectors = &core.connector_recipients.sections;
    if connectors.is_empty() {
        return gather(core, env, task, wanted.into_boxed());
    }
    let row = core.tasks.task(task).expect("preparing task");
    let mut documents = List::with_capacity(env.limits.tasks.charter_documents.min(wanted.room()));
    let mut room = wanted.room();
    for document in &core.charter_of(task).documents {
        if room == 0 {
            break;
        }
        if connectors.contains(&document.connector) {
            documents.push(document.clone()).expect("bounded charter documents");
            room = room.checked_sub(1).expect("a remaining slot");
        }
    }
    let mut claims = List::with_capacity(wanted.len());
    for section in &wanted {
        match section {
            brief::Planned::Core { text, limit, priority, required, .. } => claims
                .push(parts::Claim {
                    size: u32::try_from(text.len()).expect("bounded section text"),
                    limit: *limit,
                    priority: *priority,
                    required: *required,
                })
                .expect("one claim per core section"),
            brief::Planned::Connector { .. } => unreachable!("the engine plans its own text first"),
        }
    }
    let context = parts::Preparation {
        task,
        project: row.project,
        requester: row.requester,
        root: row.root,
        documents: documents.into_boxed(),
        core: claims.into_boxed(),
        slots: room,
        budget: env.limits.brief.brief_bytes,
    };
    let mut names = List::with_capacity(env.limits.tasks.charter_documents);
    for document in &context.documents {
        names.push(document.connector).expect("bounded charter documents");
    }
    let mut out = Queue::uncapped();
    let mut answers = Map::with_capacity(connectors.len());
    for &connector in connectors {
        answers.insert(connector, None).expect("one plan per connector");
        out.push(ask(connector, parts::Ask::Plan(Box::new(context.clone()))));
    }
    let waiting = answers.len();
    let planning = Planning { core: wanted.into_boxed(), documents: names.into_boxed(), answers, waiting };
    assert!(
        core.brief_sections.insert(task, Sections::Planning(planning)).expect("one entry per live task").is_none(),
        "one brief per preparation"
    );
    out
}

pub(crate) fn planned(
    core: &mut Core,
    env: &Env<Limits>,
    connector: u16,
    task: u64,
    mut sections: Box<[parts::Section]>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let Some(Sections::Planning(plan)) = core.brief_sections.get_mut(&task)
        && let Some(answer @ None) = plan.answers.get_mut(&connector)
    {
        if sections.len() > skein_lib::length::usize_of(env.limits.brief.sections) {
            drop_plan(task, connector, &sections, &mut out);
            sections = Box::new([]);
        }
        *answer = Some(sections);
        plan.waiting = plan.waiting.checked_sub(1).expect("an outstanding plan");
        if plan.waiting != 0 {
            return out;
        }
    } else {
        drop_plan(task, connector, &sections, &mut out);
        return out;
    }
    let Some(Sections::Planning(mut plan)) = core.brief_sections.remove(&task) else {
        unreachable!("the last answer names a plan")
    };
    let mut plans = List::with_capacity(plan.answers.len());
    for &connector in &core.connector_recipients.sections {
        let sections = plan.answers.remove(&connector).expect("asked each connector").expect("every plan answered");
        plans.push(brief::ConnectorPlan { connector, sections }).expect("one answer per connector");
    }
    match brief::place(&plan.core, &plan.documents, plans.as_slice(), env.limits.brief.sections) {
        brief::Placing::Placed { sections, dropped } => {
            drop_actions(task, &dropped, &mut out);
            let mut gathered = gather(core, env, task, sections);
            for _ in 0..gathered.len() {
                out.push(gathered.pop().expect("gather output count"));
            }
        }
        brief::Placing::Failed { failure, dropped } => {
            drop_actions(task, &dropped, &mut out);
            fail(core, env, task, failure, &mut out);
        }
    }
    out
}

fn gather(core: &mut Core, env: &Env<Limits>, task: u64, sections: Box<[brief::Planned]>) -> Queue<Request> {
    step(
        core,
        env,
        brief::GatherEvent::Plan {
            brief: Token::new(task),
            budget: env.limits.brief.brief_bytes,
            deadline: env.now.saturating_add(env.limits.brief.gather),
            sections,
        },
    )
}

pub(crate) fn step(core: &mut Core, env: &Env<Limits>, event: brief::GatherEvent) -> Queue<Request> {
    let mut child = Queue::uncapped();
    crate::routing::step::step_brief(core, env, event, &mut child);
    tag(core, env, child)
}

pub(crate) fn tag(core: &mut Core, env: &Env<Limits>, mut child: Queue<brief::GatherRequest>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    for _ in 0..child.len() {
        match child.pop().expect("brief output count") {
            brief::GatherRequest::Gather { connector, brief, section, budget, deadline } => {
                out.push(ask(connector, parts::Ask::Gather { task: brief.raw(), section, budget, deadline }));
            }
            brief::GatherRequest::CutTo { connector, brief, section, size } => {
                out.push(ask(connector, parts::Ask::Cut { task: brief.raw(), section, size }));
            }
            brief::GatherRequest::Drop { connector, brief, section } => {
                out.push(ask(connector, parts::Ask::Drop { task: brief.raw(), section }));
            }
            brief::GatherRequest::Failed { brief, failure, .. } | brief::GatherRequest::Refused { brief, failure } => {
                fail(core, env, brief.raw(), failure, &mut out);
            }
            brief::GatherRequest::Complete { brief, order } => {
                completed(core, env, brief.raw(), order, &mut out);
            }
        }
    }
    out
}

fn completed(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    order: Box<[brief::GatherPlaced]>,
    out: &mut Queue<Request>,
) {
    let Some(context) = core.contexts.get(&task) else {
        drop_complete(task, &order, out);
        return;
    };
    if !super::parts::preparation_current(core, context) {
        drop_complete(task, &order, out);
        return;
    }
    assert!(
        core.brief_sections.insert(task, Sections::Complete(order)).expect("one entry per live task").is_none(),
        "a gathering brief completes only once"
    );
    if !core.stopping && !core.settling {
        match super::preparation::request_workspace(core, context.clone()) {
            Ok(request) => out.push(request),
            Err(failure) => fail(core, env, task, failure, out),
        }
    }
}

fn fail(core: &mut Core, env: &Env<Limits>, task: u64, failure: parts::Failure, out: &mut Queue<Request>) {
    super::preparation::clear_failed_preparation(core, task, out);
    let event = super::preparation::preparation_failure(task, failure, out);
    append_tasks(core, env, event, out);
    out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
}

pub(super) fn take_complete(core: &mut Core, task: u64) -> Box<[brief::GatherPlaced]> {
    let Some(Sections::Complete(order)) = core.brief_sections.remove(&task) else {
        unreachable!("an agent workspace follows a completed brief")
    };
    order
}

pub(crate) fn abandon(core: &mut Core, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    release(core, task, out);
    let mut abandoned = step(core, env, brief::GatherEvent::Abandon { brief: Token::new(task) });
    for _ in 0..abandoned.len() {
        out.push(abandoned.pop().expect("abandoned brief output count"));
    }
}

pub(super) fn release(core: &mut Core, task: u64, out: &mut Queue<Request>) {
    match core.brief_sections.remove(&task) {
        Some(Sections::Planning(plan)) => {
            for (&connector, answer) in &plan.answers {
                match answer {
                    Some(sections) => drop_plan(task, connector, sections, out),
                    None => out.push(ask(connector, parts::Ask::Abandon { task })),
                }
            }
        }
        Some(Sections::Complete(order)) => drop_complete(task, &order, out),
        None => {}
    }
}

fn drop_actions(task: u64, dropped: &[brief::ConnectorAction], out: &mut Queue<Request>) {
    for action in dropped {
        match *action {
            brief::ConnectorAction::Drop { connector, token } => {
                out.push(ask(connector, parts::Ask::Drop { task, section: token }));
            }
            brief::ConnectorAction::CutTo { .. } => unreachable!("placement only drops sections"),
        }
    }
}

fn drop_plan(task: u64, connector: u16, sections: &[parts::Section], out: &mut Queue<Request>) {
    for section in sections {
        out.push(ask(connector, parts::Ask::Drop { task, section: section.token }));
    }
}

fn drop_complete(task: u64, order: &[brief::GatherPlaced], out: &mut Queue<Request>) {
    for placed in order {
        match placed {
            brief::GatherPlaced::Connector { connector, token, .. } => {
                out.push(ask(*connector, parts::Ask::Drop { task, section: *token }));
            }
            brief::GatherPlaced::Core { .. }
            | brief::GatherPlaced::CoreMissing { .. }
            | brief::GatherPlaced::Missing { .. } => {}
        }
    }
}

fn ask(connector: u16, ask: parts::Ask) -> Request {
    connector_ask(connector, Ask::Parts(ask))
}
