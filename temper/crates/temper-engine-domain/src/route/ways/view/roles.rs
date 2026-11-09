//! The project's agents, once per charter (temper/docs/design/domain/forge.md, 8.13).
use super::super::super::forge_route::{ASKING_TEMPLATE, FOREMAN_CHARTER, STATUS_CHARTER};
use crate::{Domain, ways};
use jig_domain_people::TemplateExecutor;
use skein_lib::List;
use temper_engine_domain_forge_change::Route;

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
    let foreman = domain.forge.project_questions(project) == Route::Foreman || coordinator.is_some();
    let mut roles = List::<(u32, ways::Card)>::with_capacity(16);
    for (number, who, name) in [
        (template(domain, project, 1), b"For each chat".as_slice(), b"Chat".as_slice()),
        (template(domain, project, ASKING_TEMPLATE), b"When you ask", b"Ask temper"),
        (coordinator, b"For each goal", b"Coordinator"),
        (Some(inputs.producer), b"For each change", b"Coding agent"),
        (if review { Some(inputs.reviewer) } else { None }, b"For each change", b"Reviewer"),
        (if foreman { Some(FOREMAN_CHARTER) } else { None }, b"When a coding agent asks", b"Foreman"),
        (Some(STATUS_CHARTER), b"When you look at Home after landings", b"Status agent"),
    ] {
        let Some(number) = number else { continue };
        let mut held = false;
        for (charter, _) in &roles {
            held |= *charter == number;
        }
        if held {
            continue;
        }
        if let Some(agent) = super::super::agents::agent(domain, project, party, number) {
            roles.push((number, ways::role_card(&agent, who, name))).expect("seven roles at most");
        }
    }
    ways::View::Roles(roles.into_boxed())
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
