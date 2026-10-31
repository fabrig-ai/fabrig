//! Party pools, task creation, adoption, service, policy and role requests
//! routed through authority, people, tasks and connectors
//! (`domain/people.md`, section 5.1). Controls and decisions are separate
//! families so each route file stays under 1,500 lines.

use crate::{
    Ask, Core, Event, Limits, PersonTaskRoute, Record, Request, Write,
    routing::{
        Creation, CreationKind, Work,
        party::dispatch::{Party, Routed},
        refusals::pool_refused,
    },
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(crate) fn route_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    request: Token,
    connector: u16,
    holdings: Option<Box<[tasks::Holding]>>,
) -> Queue<Request> {
    let mut routed = Routed::new(work);
    created_holdings(core, env, &mut routed, request, connector, holdings);
    routed.finish(work)
}
pub(crate) fn pool_route(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    person: u64,
    budget: u64,
) {
    let Party { request, person: owner, project, .. } = party;

    if let Err(why) = core.role_allowed(owner, project, env.limits.tasks.tree_tasks) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    if core.person_task_flights() >= env.limits.tasks.tasks {
        crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::Busy);
        return;
    }
    let Some(role) = core.people.role(person, project) else {
        pool_refused(&mut routed.work, request, people::Refusal::Unknown);
        return;
    };
    let Some(allowed) = core.authority.role(project, role.number()) else {
        pool_refused(&mut routed.work, request, people::Refusal::Unknown);
        return;
    };
    if budget > allowed.period_spend {
        pool_refused(
            &mut routed.work,
            request,
            people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: budget, left: allowed.period_spend },
            },
        );
        return;
    }
    let Some(policy) = core.authority.policy(project) else {
        pool_refused(&mut routed.work, request, people::Refusal::Unknown);
        return;
    };
    if core.period_budget(project) > policy.period_spend {
        pool_refused(
            &mut routed.work,
            request,
            people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: core.period_budget(project), left: policy.period_spend },
            },
        );
        return;
    }
    let period = tasks::Funder::Period { project, period: core.period(project) };
    let funder = tasks::Funder::Pool { project, person, period: core.period(project) };
    let needed =
        u32::from(core.tasks.funding(period).is_none()).saturating_add(u32::from(core.tasks.funding(funder).is_none()));
    if core.tasks.funding_room() < needed {
        crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::Busy);
        return;
    }
    if core.tasks.funding(period).is_none() && budget > core.period_budget(project) {
        pool_refused(
            &mut routed.work,
            request,
            people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: budget, left: core.period_budget(project) },
            },
        );
        return;
    }
    if core.tasks.funding(period).is_none() {
        routed.work.push(Work::Event(Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(0)),
            project,
            period: core.period(project),
            budget: core.period_budget(project),
        })));
    }
    let request = core.fly_person_task(request, PersonTaskRoute::PoolSet { project, person });
    if core.tasks.funding(funder).is_some() {
        routed.work.push(Work::Event(Event::Tasks(tasks::Event::ResizePool {
            reply_to: ReplyTo::new(request),
            project,
            person,
            period: core.period(project),
            budget,
        })));
    } else {
        routed.work.push(Work::Event(Event::Tasks(tasks::Event::CarvePool {
            reply_to: ReplyTo::new(request),
            project,
            person,
            period: core.period(project),
            budget,
        })));
    }
}

pub(crate) fn begin_creation(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    request: Token,
    kind: CreationKind,
    members: Box<[tasks::New]>,
) {
    if core.creating_flights() == core.flight_people_capacity {
        crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::Busy);
        return;
    }
    let remaining = u32::try_from(core.connectors.len()).expect("configured connector count fits u32");
    assert!(remaining > 0, "task creation has a numbered connector");
    let first = members.first().expect("nonempty creation").clone();
    let request = core.fly(crate::routing::flight::Flight::Creating {
        request,
        route: Creation {
            kind,
            members,
            at: 0,
            remaining,
            holdings: List::with_capacity(env.limits.tasks.holdings),
            answered: List::with_capacity(remaining),
            failed: false,
        },
    });
    creation_holdings(core, &mut routed.out, request, &first);
}

pub(crate) fn created_holdings(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    request: Token,
    connector: u16,
    holdings: Option<Box<[tasks::Holding]>>,
) {
    let known = core.connectors.contains(&connector);
    let connectors = u32::try_from(core.connectors.len()).expect("connector count");
    let Some((_, creating)) = core.creation_mut(request) else { return };
    assert!(creating.remaining > 0, "one holdings answer per connector");
    assert!(known && !creating.answered.as_slice().contains(&connector), "one answer from each numbered connector");
    creating.answered.push(connector).expect("one answer per configured connector");
    creating.remaining = creating.remaining.checked_sub(1).expect("one holdings answer per connector");
    match holdings {
        Some(holdings) => {
            for holding in holdings {
                if creating.holdings.push(holding).is_err() {
                    creating.failed = true;
                }
            }
        }
        None => creating.failed = true,
    }
    if creating.remaining != 0 {
        return;
    }
    let member = creating.members.get_mut(creating.at).expect("creation member");
    member.holdings =
        core::mem::replace(&mut creating.holdings, List::with_capacity(env.limits.tasks.holdings)).into_boxed();
    creating.at = creating.at.checked_add(1).expect("bounded batch");
    if let Some(member) = creating.members.get(creating.at) {
        creating.remaining = connectors;
        creating.answered.clear();
        let member = member.clone();
        creation_holdings(core, &mut routed.out, request, &member);
        return;
    }
    let (request, creating) = core.take_creation(request).expect("completed creation flight");
    if creating.failed {
        pool_refused(&mut routed.work, request, people::Refusal::Limit { which: people::LimitKind::Holds });
        return;
    }
    finish_creation(core, env, routed, request, creating);
}

fn finish_creation(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, request: Token, creating: Creation) {
    match creating.kind {
        CreationKind::Propose { person, reason, to, stubs } => {
            super::propose::keep(
                core,
                env,
                &mut routed.work,
                super::propose::Proposal { request, person, reason, to, batch: creating.members, stubs },
            );
            return;
        }
        CreationKind::Batch { .. } => {}
    }
    let CreationKind::Batch { person, .. } = creating.kind else { unreachable!("proposal handled above") };
    {
        let first = creating.members.first().expect("nonempty batch").number;
        let (period, pool) = core.open_person_pool(creating.members.first().expect("nonempty batch").project, person);
        if let Some(event) = period {
            routed.work.push(Work::Event(Event::Tasks(event)));
        }
        if let Some(event) = pool {
            routed.work.push(Work::Event(Event::Tasks(event)));
        }
        let CreationKind::Batch { given, stubs, .. } = creating.kind else { unreachable!("batch creation") };
        let request = core
            .fly(crate::routing::flight::Flight::Made { request, route: crate::routing::Made { first, given, stubs } });
        routed.work.push(Work::Event(Event::Tasks(tasks::Event::Make {
            reply_to: ReplyTo::new(request),
            creator: tasks::Party::Person(person),
            batch: creating.members,
        })));
    }
}

pub(crate) fn roles_route(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    holdings: Box<[people::Holding]>,
) {
    let Party { request, person, project, .. } = party;

    if let Err(why) = core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    for holding in &holdings {
        match holding.role {
            people::Role::Policy { role } => {
                if core.authority.role(project, role).is_none() {
                    pool_refused(&mut routed.work, request, people::Refusal::Unknown);
                    return;
                }
            }
            people::Role::Owner | people::Role::Maintainer | people::Role::Member | people::Role::Observer => {}
        }
    }
    let contexts = match core.tasks.project_escalations(&env.limits.tasks, project) {
        Ok(contexts) => contexts,
        Err(refusal) => {
            assert!(refusal == tasks::Refusal::NotReady, "project inspection refusal is readiness");
            crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::NotReady);
            return;
        }
    };
    if let Err(why) = core.roles_preflight(&env.limits, &contexts, &holdings) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    drop(contexts);
    drop(holdings);
    if let Err(why) = core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    core.touch_accounts(None);
    core.touch_deployment();
    core.touch_project_parties(project);
    let original = core.people_request(request).expect("party route");
    let mut applied = Queue::uncapped();
    crate::routing::step::step_people(
        core,
        env,
        people::Event::ApplyRoles { reply_to: ReplyTo::new(request), request: original },
        &mut applied,
    );
    let mut terminal = None;
    for _ in 0..applied.len() {
        match applied.pop().expect("role application output count") {
            people::Request::Save { record } => {
                crate::routing::view::routes::touch_people_row(core, &record);
                routed.out.push(Request::Write(Write::Save(Record::People(record))));
            }
            people::Request::RolesApplied { reply_to, request: actual, result } => {
                assert!(
                    reply_to.into_token() == request && actual == original && terminal.is_none(),
                    "one role terminal"
                );
                terminal = Some(result);
            }
            people::Request::Erase { .. }
            | people::Request::Act { .. }
            | people::Request::Acted { .. }
            | people::Request::Route { .. }
            | people::Request::Reply { .. }
            | people::Request::RolesRefused { .. }
            | people::Request::ServiceMade { .. }
            | people::Request::RestoreRefused { .. } => unreachable!("role application has one save and terminal"),
        }
    }
    if let Err(why) = terminal.expect("role application terminal") {
        routed.work.push(Work::Event(Event::People(people::Event::Decided { request, outcome: why })));
        return;
    }
    routed
        .work
        .push(Work::Event(Event::Tasks(tasks::Event::RecheckEscalations { reply_to: ReplyTo::new(request), project })));
    let mut reroutes = core.proposal_recheck_project(&env.limits, project, env.wall);
    for _ in 0..reroutes.len() {
        routed.work.push(Work::Event(Event::Tasks(reroutes.pop().expect("bounded proposal recheck"))));
    }
    routed.work.push(Work::Event(Event::People(people::Event::Decided {
        request,
        outcome: people::Outcome::RolesSet { project },
    })));
}

pub(crate) fn make_service_request(
    core: &mut Core,
    env: &Env<Limits>,
    party: Party,
    pending: &mut Queue<people::Request>,
    service_role: people::Role,
) {
    let Party { request, person, project, .. } = party;
    let original = core.people_request(request).expect("party route");
    let outcome = match core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        Ok(()) if core.authority.role(project, service_role.number()).is_some() => {
            match crate::fresh(&mut core.counters, crate::Family::Person) {
                Some(candidate) => {
                    let mut follow = Queue::uncapped();
                    crate::routing::step::step_people(
                        core,
                        env,
                        people::Event::MakeService { request: original, person: candidate },
                        &mut follow,
                    );
                    for _ in 0..follow.len() {
                        let next = match follow.pop().expect("service output count") {
                            people::Request::ServiceMade { outcome, .. } => {
                                people::Request::ServiceMade { request, outcome }
                            }
                            other @ (people::Request::Act { .. }
                            | people::Request::Acted { .. }
                            | people::Request::RolesApplied { .. }
                            | people::Request::Route { .. }
                            | people::Request::Reply { .. }
                            | people::Request::Save { .. }
                            | people::Request::Erase { .. }
                            | people::Request::RolesRefused { .. }
                            | people::Request::RestoreRefused { .. }) => other,
                        };
                        pending.push(next);
                    }
                    None
                }
                None => Some(people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic })),
            }
        }
        Ok(()) => Some(people::Outcome::Refused(people::Refusal::Unknown)),
        Err(refusal) => Some(people::Outcome::Refused(refusal)),
    };
    if let Some(outcome) = outcome {
        pending.push(people::Request::ServiceMade { request, outcome });
    }
}

pub(crate) fn adopt_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    adoption: people::Adoption,
) {
    let Party { request, person, project, .. } = party;

    match core.role_allowed(person, project, env.limits.tasks.tree_tasks) {
        Ok(()) => {
            routed.out.push(Request::Ask {
                connector: adoption.resource.connector,
                ask: Ask::Adopt { request, project, adoption },
            });
        }
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn change_policy_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    change: people::PolicyChange,
) {
    let Party { request, person, project, .. } = party;

    let period = core.period(project);
    match core.change_policy(&env.limits, person, project, change) {
        Ok((snapshot, resize)) => {
            core.touch_project_parties(project);
            let name = core.names.get(&project).expect("project policy has a name").clone();
            routed.out.push(Request::Write(Write::Save(Record::People(people::Stored::Policy {
                project,
                name,
                value: snapshot,
            }))));
            if let Some(budget) = resize.period {
                routed.work.push(Work::Event(Event::Tasks(tasks::Event::ResizePeriod {
                    reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Admission)),
                    project,
                    period,
                    budget,
                })));
            }
            for cuts in [true, false] {
                for pool in &resize.pools {
                    if (pool.to < pool.from) == cuts {
                        routed.work.push(Work::Event(Event::Tasks(tasks::Event::ResizePool {
                            reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Admission)),
                            project,
                            person: pool.person,
                            period,
                            budget: pool.to,
                        })));
                    }
                }
            }
            let mut reroutes = core.proposal_recheck_project(&env.limits, project, env.wall);
            for _ in 0..reroutes.len() {
                routed.work.push(Work::Event(Event::Tasks(reroutes.pop().expect("policy holder recheck"))));
            }
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::PolicyChanged { project },
            })));
        }
        Err(why) => pool_refused(&mut routed.work, request, why),
    }
}

fn creation_holdings(core: &Core, out: &mut Queue<Request>, request: Token, new: &tasks::New) {
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask {
            connector,
            ask: Ask::TaskHoldings {
                request,
                project: new.project,
                root: new.number,
                number: new.number,
                executor: new.executor,
                spec: new.spec.clone(),
            },
        });
    }
}
