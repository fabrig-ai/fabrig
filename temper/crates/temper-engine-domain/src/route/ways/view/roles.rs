//! The project's agents, once per charter (temper/docs/design/domain/forge/change/branch-path.md).
use super::super::super::forge_route::{ASKING_TEMPLATE, DRAFTING_TEMPLATE, STATUS_CHARTER};
use crate::{Domain, ways};
use jig_domain_people::TemplateExecutor;
use jig_domain_tasks::{Executor, Phase};
use skein_lib::{List, Set};
use temper_engine_domain_forge::settings;

pub(super) fn draw(domain: &Domain, project: u32, party: u64) -> ways::View {
    let inputs = super::super::inputs(domain, project, party, &[], None, None);
    let path = ways::derive(&inputs);
    let coordinator = match inputs.start {
        ways::Start::Chat => None,
        ways::Start::Goal { charter, .. } => Some(charter),
    };
    let mut review = false;
    for stop in &path.stops {
        review |= stop.step == ways::Step::Review;
    }
    let mut roles = List::<(u32, ways::Card)>::with_capacity(domain.limits.tasks.charters);
    for (number, when, name) in [
        (template(domain, project, 1), b"For each chat".as_slice(), b"Chat".as_slice()),
        (template(domain, project, ASKING_TEMPLATE), b"When you ask", b"Ask temper"),
        (template(domain, project, DRAFTING_TEMPLATE), b"When you start a draft", b"Drafter"),
        (coordinator, b"For each goal", b"Coordinator"),
        (Some(inputs.producer), b"For each change", b"Coding agent"),
        (if review { Some(inputs.reviewer) } else { None }, b"For each change", b"Reviewer"),
        (Some(STATUS_CHARTER), b"When you look at Home after landings", b"Status agent"),
    ] {
        let Some(number) = number else { continue };
        push(domain, project, party, &mut roles, number, when, name);
    }
    for &number in &standing(domain, project) {
        push(domain, project, party, &mut roles, number, b"For each standing role", b"Standing role");
    }
    for (_, follow) in domain.forge.follows(project).as_ref() {
        let Some(producer) = &follow.producer else { continue };
        let Some(Some(number)) = settings::producer_charter(producer) else { continue };
        push(domain, project, party, &mut roles, number, b"For each follow change", b"Follow's coding agent");
    }
    ways::View::Roles(roles.into_boxed())
}

fn push(
    domain: &Domain,
    project: u32,
    party: u64,
    roles: &mut List<(u32, ways::Card)>,
    number: u32,
    when: &[u8],
    name: &[u8],
) {
    for (charter, _) in &*roles {
        if *charter == number {
            return;
        }
    }
    if let Some(agent) = super::super::agents::agent(domain, project, party, number) {
        let does = domain.forge.charter_words(number).unwrap_or(&agent.asked);
        let who = ways::role_who(when, does);
        roles.push((number, ways::role_card(&agent, &who, name))).expect("one role per configured charter");
    }
}

fn standing(domain: &Domain, project: u32) -> Set<u32> {
    let mut charters = Set::with_capacity(domain.limits.tasks.charters);
    for task in domain.core.tasks.view_tasks().as_ref() {
        let row = domain.core.tasks.task(task.number).expect("live task view");
        if row.project != project || row.label != 3 {
            continue;
        }
        match row.phase {
            Phase::Waiting | Phase::Active(_) | Phase::Held { .. } | Phase::Closing(_) => {}
            Phase::Ended(_) => continue,
        }
        let number = match row.executor {
            Executor::Agent { charter } => charter,
            Executor::Procedure { .. } | Executor::Person(_) | Executor::Job { .. } => continue,
        };
        if domain.core.current.charter(number).is_some() {
            charters.insert(number).expect("one entry per configured charter");
        }
    }
    charters
}

fn template(domain: &Domain, project: u32, number: u32) -> Option<u32> {
    for template in &domain.core.project_templates(project) {
        if template.number == number {
            return match template.executor {
                TemplateExecutor::Charter(charter) => Some(charter),
                TemplateExecutor::Chosen => None,
            };
        }
    }
    None
}
