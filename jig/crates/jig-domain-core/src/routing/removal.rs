//! A project closes in its removal decision (temper/docs/design/draft/playground-reset.md, 5;
//! jig/docs/design/domain/people.md, 5.2).
use crate::{Core, CoreRecord, Event, Limits, Record, Removal, RemovalStage, Request, Requests, Write};
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::{Env, List, Queue, Token};

// Outside the people's slab tokens, beside the core's other internal reply tokens.
pub(super) const CANCEL_REQUEST: Token = Token::new(u64::MAX - 4);

pub(super) fn remove_request(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    person: u64,
    project: u32,
) {
    match crate::acts::admit(
        core,
        &env.limits,
        person,
        crate::acts::Act::RemoveProject,
        &crate::documents::Subject::Project { project },
        env.wall,
    ) {
        crate::acts::Admission::Allowed => remove_act(core, env, work, out, request, project),
        crate::acts::Admission::Proposed { .. } | crate::acts::Admission::Bounded { .. } => {
            unreachable!("project removal has no proposal or draft bounds")
        }
        crate::acts::Admission::Refused(why) => {
            work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::Refused(why) }));
        }
    }
}

pub(super) fn remove_act(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
) {
    if !core.projects.as_slice().contains(&project) || core.removals.contains_key(&project) {
        work.push(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Unknown),
        }));
        return;
    }
    if core.removals.len() == core.removals.capacity() {
        work.push(Event::People(people::Event::Decided {
            request,
            outcome: people::Outcome::Again(people::Again::Busy),
        }));
        return;
    }
    close(core, env, work, out, request, project);
}

/// The decision both share: row, closing, cancels, answer.
fn close(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
) {
    let removal = Removal { project, stage: RemovalStage::Ending };
    core.removals.insert(project, removal.clone()).expect("removal room checked");
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Removal(removal)))));
    core.touch_project_parties(project);
    let mut projects = List::with_capacity(core.projects.capacity());
    for number in &core.projects {
        if *number != project {
            projects.push(*number).expect("subset of projects");
        }
    }
    core.projects = projects;
    drop(core.names.remove(&project));
    drop(core.permission_roles.remove(&project));
    drop(core.template_overrides.remove(&project));
    authority::step(&mut core.authority, authority::Event::Dropped { project }, &mut Queue::uncapped());
    work.push(Event::People(people::Event::ProjectClosed { project }));
    for task in core.tasks.project_roots(project) {
        let mut cancels = Queue::uncapped();
        if super::deployment_acts::cancel_live(
            core,
            &mut cancels,
            CANCEL_REQUEST,
            project,
            task,
            None,
            b"project deleted".as_slice().into(),
        )
        .is_err()
        {
            // The kept Ending row lets deletion resume any cancel that did not fit.
            break;
        }
        for _ in 0..cancels.len() {
            let Event::Tasks(event) = cancels.pop().expect("one compound cancel") else {
                unreachable!("cancel_live emits task events");
            };
            let Requests::Out(mut routed) = super::task_routes::route_tasks(core, env, work, event);
            for _ in 0..routed.len() {
                out.push(routed.pop().expect("routed cancel output"));
            }
        }
    }
    core.touch_deployment();
    work.push(Event::People(people::Event::Decided { request, outcome: people::Outcome::ProjectRemoved { project } }));
}
