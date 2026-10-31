//! One project decision includes its roster, policy, first period and answer
//! (people.md, 5.1–5.3; core.md, 6.1; contract.md, 3.2).
use crate::{
    Core, Event, Limits, Record, Request, Write,
    routing::{
        Work,
        party::dispatch::{Party, Routed},
    },
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

pub(crate) fn rename_request(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, party: Party, name: Box<str>) {
    let Party { request, person, project, .. } = party;

    match core.rename(&env.limits, person, project, name) {
        Ok(()) => {
            let policy = core.authority.policy(project).expect("renamed project policy");
            let permissions = core.permission_roles.get(&project).expect("project permission mappings");
            let templates = core.template_overrides.get(&project).expect("project template overrides");
            let value =
                crate::admission::policy::snapshot(policy, permissions, templates).expect("checked policy snapshot");
            let name = core.names.get(&project).expect("renamed project name").clone();
            routed.out.push(Request::Write(Write::Save(Record::People(people::Stored::Policy {
                project,
                name,
                value,
            }))));
            core.touch_project_parties(project);
            decided(&mut routed.work, request, people::Outcome::Renamed { project });
        }
        Err(why) => decided(&mut routed.work, request, people::Outcome::Refused(why)),
    }
}

pub(crate) fn make_project_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    name: Box<str>,
    policy: u32,
) {
    let Party { request, person: maker, .. } = party;

    if core.tasks.funding_room() < 1 {
        return crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::Busy);
    }
    match core.make_project(&env.limits, maker, name, policy) {
        Ok(made) => {
            let project = made.project;
            let budget = made.policy.period_spend;
            routed.out.push(Request::Write(Write::Save(Record::People(people::Stored::Policy {
                project,
                name: made.name,
                value: made.policy,
            }))));
            routed.work.push(Work::Event(Event::People(people::Event::Roles {
                project,
                holdings: Box::new([people::Holding { person: maker, role: people::Role::Owner }]),
            })));
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::OpenPeriod {
                reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Admission)),
                project,
                period: 1,
                budget,
            })));
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::PeriodNext { project, period: 1, next: None })));
            core.touch_party(maker);
            routed.out.push(Request::ProjectMade { project });
            decided(&mut routed.work, request, people::Outcome::ProjectMade { project });
        }
        Err(why) => decided(&mut routed.work, request, people::Outcome::Refused(why)),
    }
}

#[expect(clippy::too_many_arguments, reason = "the deployment act names the maker without a sign-in")]
pub(crate) fn make_project_act(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    request: Token,
    maker: u64,
    name: Box<str>,
    policy: u32,
) {
    if core.people.role(maker, people::DEPLOYMENT) != Some(people::Role::Owner) {
        return decided(work, request, people::Outcome::Refused(people::Refusal::Unknown));
    }
    let mut routed = Routed::new(work);
    make_project_request(
        core,
        env,
        &mut routed,
        Party { request, person: maker, project: 0, role: None },
        name,
        policy,
    );
    let mut follow = routed.finish(work);
    for _ in 0..follow.len() {
        let next = follow.pop().expect("project route output count");
        out.push(next);
    }
}

fn decided(work: &mut Queue<Work>, request: Token, outcome: people::Outcome) {
    work.push(Work::Event(Event::People(people::Event::Decided { request, outcome })));
}
