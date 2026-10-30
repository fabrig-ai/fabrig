//! Configuration's first-start project seed and the deployment's roles
//! (protocol/configuration.md, 3 and 8; domain/people.md, 3).

use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::{Env, List, Map, Queue};

use crate::{Core, Event, Family, Limits, Record, Request, Requests, Write, fresh};

pub(super) fn seed(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>) -> Requests {
    seed_projects(core);
    let mut out = Queue::uncapped();
    if !crate::settings::restart::first_start(core, env, &mut out) {
        return Requests::Out(out);
    }
    let mut rows = people_rows(core, env);
    for _ in 0..rows.len() {
        let record = rows.pop().expect("bounded seed rows");
        let mut child = Queue::uncapped();
        #[expect(clippy::wildcard_enum_match_arm, reason = "the seed generator emits only people, roles and policies")]
        match &record {
            people::Stored::Person { .. } => people::step(
                &mut core.people,
                &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                people::Event::Restore { record: record.clone() },
                &mut child,
            ),
            people::Stored::Roles { project, holdings } => people::step(
                &mut core.people,
                &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                people::Event::Roles { project: *project, holdings: holdings.clone() },
                &mut child,
            ),
            people::Stored::Policy { .. } => {}
            _ => unreachable!("seed row family"),
        }
        if let people::Stored::Roles { .. } = &record {
            for _ in 0..child.len() {
                let people::Request::Save { record } = child.pop().expect("seed role row") else {
                    unreachable!("configured roster fits people limits");
                };
                out.push(Request::Write(Write::Save(Record::People(record))));
            }
        } else {
            assert!(child.is_empty(), "configured owner restores within people limits");
            out.push(Request::Write(Write::Save(Record::People(record))));
        }
    }
    let mut child = Queue::uncapped();
    people::step(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        people::Event::Restored,
        &mut child,
    );
    let Requests::Out(mut restored) = super::people_routes::tag_people(core, env, child, work);
    for _ in 0..restored.len() {
        out.push(restored.pop().expect("people restored requests"));
    }
    Requests::Out(out)
}

pub(crate) fn seed_projects(core: &mut Core) {
    let mut rows = project_rows(core);
    for _ in 0..rows.len() {
        let (project, policy) = rows.pop().expect("configured project");
        let mut facts = Queue::uncapped();
        authority::step(&mut core.authority, authority::Event::Policy { project, policy }, &mut facts);
        assert!(facts.pop() == Some(authority::PolicyFact::Added { project }), "checked seed policy admission");
    }
}

fn project_rows(core: &mut Core) -> Queue<(u32, authority::Policy)> {
    let projects = core::mem::replace(&mut core.seed_projects, Box::new([]));
    let mut rows = Queue::uncapped();
    for project in projects {
        rows.push((project.number, project.policy.policy));
        core.projects.push(project.number).expect("bounded seed projects");
        core.names.insert(project.number, project.name).expect("bounded seed names");
        core.permission_roles.insert(project.number, project.policy.permissions).expect("bounded seed permissions");
        core.template_overrides.insert(project.number, project.policy.templates).expect("bounded seed templates");
    }
    rows
}

pub(crate) fn start_seed(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut projects = project_rows(core);
    for _ in 0..projects.len() {
        let (project, policy) = projects.pop().expect("configured project");
        let mut facts = Queue::uncapped();
        authority::step(&mut core.authority, authority::Event::Restore { project, policy }, &mut facts);
        match facts.pop().expect("seed policy terminal") {
            authority::PolicyFact::Added { project: found } => assert!(found == project, "seed policy owner"),
            authority::PolicyFact::Refused { .. } => {
                let _refused = core.start_refuse(crate::start::Failure::ProjectPolicy { project });
                return;
            }
            authority::PolicyFact::Changed { .. } | authority::PolicyFact::Dropped { .. } => {
                unreachable!("duplicate {:?}", crate::Key::People(people::Key::Policy(project)));
            }
        }
    }
    if !crate::settings::restart::first_start(core, env, out) {
        return;
    }
    let mut rows = people_rows(core, env);
    for _ in 0..rows.len() {
        let record = rows.pop().expect("configured seed row");
        if let people::Stored::Policy { .. } = &record {
            // The authority restore already owns this policy.
        } else {
            let mut child = Queue::uncapped();
            people::step(
                &mut core.people,
                &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                people::Event::Restore { record: record.clone() },
                &mut child,
            );
            assert!(child.is_empty(), "selected seed emits only explicit writes");
        }
        out.push(Request::Write(Write::Save(Record::People(record))));
    }
}

fn people_rows(core: &mut Core, env: &Env<Limits>) -> Queue<people::Stored> {
    let owners = core::mem::replace(&mut core.seed_owners, Box::new([]));
    let mut identities = Map::with_capacity(env.limits.people.initial_owners);
    let mut rows = Queue::uncapped();
    let mut holdings = List::with_capacity(env.limits.people.holdings);
    for owner in &owners {
        if identities.get(&owner.identity).is_some() {
            continue;
        }
        let person = fresh(&mut core.counters, Family::Person).expect("first-start person space");
        let record = people::Stored::Person {
            number: person,
            identity: people::Identity { key: owner.identity.clone(), login: Box::new([]), name: Box::new([]) },
        };
        identities.insert(owner.identity.clone(), person).expect("bounded configured identities");
        rows.push(record);
        holdings.push(people::Holding { person, role: people::Role::Owner }).expect("configured owner roster");
    }
    rows.push(people::Stored::Roles { project: people::DEPLOYMENT, holdings: holdings.into_boxed() });
    for &project in &core.projects {
        core.counters.deployment.projects = core.counters.deployment.projects.max(u64::from(project));
        let policy = core.authority.policy(project).expect("configured project policy");
        let permissions = match core.permission_roles.get(&project) {
            Some(roles) => roles.as_ref(),
            None => &[],
        };
        let templates = match core.template_overrides.get(&project) {
            Some(entries) => entries.as_ref(),
            None => &[],
        };
        let name = core.names.get(&project).expect("project policy has a name").clone();
        let value = crate::policy::snapshot(policy, permissions, templates).expect("configured policy snapshot");
        rows.push(people::Stored::Policy { project, name, value });
        let mut holdings = List::with_capacity(env.limits.people.holdings);
        for owner in &owners {
            if owner.project != project {
                continue;
            }
            let person = *identities.get(&owner.identity).expect("configured owner person seeded");
            holdings.push(people::Holding { person, role: people::Role::Owner }).expect("configured owner roster");
        }
        rows.push(people::Stored::Roles { project, holdings: holdings.into_boxed() });
    }
    rows
}
