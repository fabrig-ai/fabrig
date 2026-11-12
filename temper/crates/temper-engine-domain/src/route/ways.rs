//! Gather a project's branch path in the answering decision
//! (temper/docs/design/domain/forge/change/branch-path.md; temper/docs/design/protocol/configuration/settings.md, 8).
use crate::{Domain, Work, ways};
use alloc::boxed::Box;
use jig_domain_people::TemplateExecutor;
use skein_lib::{Decimal, List, Map, Token};
use temper_engine_domain_forge::{self as forge, settings};

mod agents;
pub(crate) mod view;
use agents::agent;

pub(crate) fn inputs(
    domain: &Domain,
    project: u32,
    party: u64,
    items: &[ways::Item],
    repository: Option<&forge::Repository>,
    branch: Option<&[u8]>,
) -> ways::Inputs {
    let repositories = domain.forge.readable_repositories(project);
    let repository = match repository.or(domain.forge.home(project)) {
        Some(home) => Some(home),
        None => repositories.first().copied(),
    };
    let landing = match repository {
        Some(repository) => repository.settings.default_branch.clone(),
        None => b"main".as_slice().into(),
    };
    let branch = match branch {
        Some(branch) => branch.into(),
        None => landing.clone(),
    };
    let other = branch != landing;
    let repository = match repository {
        Some(repository) => Some(repository_inputs(domain, project, repository, other)),
        None => None,
    };
    let held = domain.forge.landing_rules(project);
    let rules = ways::laid(&held, domain.forge.every_project_rules(), project, items);
    let charters = domain.forge.charters();
    let start = if other { ways::Start::Chat } else { start(domain, project, items) };
    let mut by_number = Map::uncapped();
    if let Some(policy) = domain.core.authority.policy(project) {
        for role in &policy.roles {
            by_number
                .insert(
                    role.number,
                    ways::Role {
                        number: role.number,
                        name: role_name(role.number),
                        yours: match domain.core.people.role(party, project) {
                            Some(held) => held.number() == role.number,
                            None => false,
                        },
                    },
                )
                .expect("one policy role per number");
        }
    }
    let mut ordered = List::uncapped();
    for (_, role) in &by_number {
        ordered.push(role.clone()).expect("bounded policy roles");
    }
    let mut agents = List::<ways::Agent>::uncapped();
    let goal = match start {
        ways::Start::Chat => None,
        ways::Start::Goal { charter, .. } => Some(charter),
    };
    for number in [Some(charters.change), Some(charters.review), goal].into_iter().flatten() {
        let mut held = false;
        for agent in &agents {
            held |= agent.charter == number;
        }
        if held {
            continue;
        }
        if let Some(agent) = agent(domain, project, party, number) {
            agents.push(agent).expect("three named charters at most");
        }
    }
    ways::Inputs {
        branch,
        repository,
        rules,
        start,
        producer: charters.change,
        reviewer: charters.review,
        by: domain.forge.landing_way(project),
        merge_job: domain.forge.merge_job(project).is_some(),
        roles: ordered.into_boxed(),
        agents: agents.into_boxed(),
    }
}

fn repository_inputs(domain: &Domain, project: u32, repository: &forge::Repository, other: bool) -> ways::Repository {
    let mut workflows = List::uncapped();
    let checks = domain.forge.repository_gates(project, repository.provider);
    for workflow in &repository.workflows {
        let mut dispatched = false;
        if let Some(checks) = checks {
            for gate in &checks.gates {
                dispatched |= workflow == &gate.workflow;
            }
            for tier in &checks.tiers {
                dispatched |= workflow == &tier.workflow;
            }
        }
        if !dispatched {
            workflows.push(workflow.clone()).expect("repository workflows");
        }
    }
    let mut gates = List::uncapped();
    let mut tiers = List::uncapped();
    if let Some(checks) = checks {
        for gate in &checks.gates {
            gates.push(gate.name.clone()).expect("configured test gates");
        }
        for tier in &checks.tiers {
            if other && tier.follows == forge::Follows::Landing {
                continue;
            }
            tiers
                .push(ways::Tier {
                    name: tier.name.clone(),
                    follows: tier.follows.clone(),
                    description: tier.description.clone(),
                })
                .expect("configured tiers");
        }
    }
    ways::Repository {
        host: repository.host.clone(),
        owner: repository.owner.clone(),
        name: repository.name.clone(),
        ci: repository.ci,
        checks: repository.checks.clone(),
        contexts: match &repository.protection {
            forge::Protection::Rule(rule) => rule.contexts.clone(),
            forge::Protection::Unknown | forge::Protection::Absent => Box::default(),
        },
        workflows: workflows.into_boxed(),
        gates: gates.into_boxed(),
        tiers: tiers.into_boxed(),
    }
}

fn project_ways(domain: &Domain, project: u32) -> Option<&settings::ProjectWays> {
    match domain.forge.setting(7, project, 1) {
        Some(settings::SizedValue { value: settings::Value::Ways(value), .. }) => Some(value),
        Some(settings::SizedValue {
            value:
                settings::Value::Rule(_)
                | settings::Value::Charter(_)
                | settings::Value::Goal(_)
                | settings::Value::Questions(_)
                | settings::Value::CharterWords(_)
                | settings::Value::Classes(_)
                | settings::Value::ProjectAreas(_) | settings::Value::ProjectTriager(_) | settings::Value::ProjectWords(_),
            ..
        })
        | None => None,
    }
}

fn start(domain: &Domain, project: u32, items: &[ways::Item]) -> ways::Start {
    let value = ways::laid_ways(project_ways(domain, project), project, items);
    if !value.goals {
        return ways::Start::Chat;
    }
    let templates = domain.core.project_templates(project);
    let mut chosen: Option<&jig_domain_core::Template> = None;
    for template in &templates {
        if (value.way != 0 && template.number == value.way)
            || (value.way == 0
                && template.tracked.is_some()
                && match chosen {
                    Some(previous) => template.number < previous.number,
                    None => true,
                })
        {
            chosen = Some(template);
        }
    }
    if let Some(template) = chosen {
        for resource in &template.resources {
            if resource.connector == domain.config.forge_connector
                && resource.symbol == super::forge_route::GOAL_BRANCH_SYMBOL
            {
                return match template.executor {
                    TemplateExecutor::Charter(charter) => {
                        let goal = ways::laid_goal(domain.forge.template_goal(template.number), template.number, items);
                        ways::Start::Goal { charter, design: goal.design, close_out: goal.close_out }
                    }
                    TemplateExecutor::Chosen => ways::Start::Chat,
                };
            }
        }
    }
    ways::Start::Chat
}

fn role_name(number: u32) -> Box<[u8]> {
    match number {
        0 => b"owner".as_slice().into(),
        1 => b"maintainer".as_slice().into(),
        2 => b"member".as_slice().into(),
        3 => b"observer".as_slice().into(),
        4.. => {
            let mut name = List::uncapped();
            for &byte in b"role ".iter().chain(Decimal::of(u64::from(number)).as_bytes()) {
                name.push(byte).expect("role number");
            }
            name.into_boxed()
        }
    }
}

pub(crate) fn document(domain: &Domain, project: u32, party: u64) -> ways::WaysDocument {
    let mut entries = List::uncapped();
    let mut paths = List::uncapped();
    for way in &domain.config.ways_stock {
        let items = ways::items(way, project, project_ways(domain, project));
        let path = ways::derive(&inputs(domain, project, party, &items, None, None));
        paths.push(path.clone()).expect("stock ways");
        let mut roles = List::uncapped();
        for role in &way.roles {
            roles.push(ways::WayRoleEntry { role: role.clone(), running: role_running(domain, project, role.charter) })
                .expect("stock way roles");
        }
        entries
            .push(ways::WayEntry {
                name: way.name.clone(),
                headline: way.headline.clone(),
                line: way.line.clone(),
                path,
                items,
                roles: roles.into_boxed(),
                delegation: way.delegation && lacks_delegation(domain, project),
            })
            .expect("stock ways");
    }
    let own = ways::derive(&inputs(domain, project, party, &[], None, None));
    ways::WaysDocument {
        project,
        ways: entries.into_boxed(),
        current: ways::current(&paths.into_boxed(), &own),
        own,
        words: domain.forge.project_words(project).cloned(),
        may_change: domain.core.people.role(party, project) == Some(jig_domain_people::Role::Owner),
    }
}

fn role_running(domain: &Domain, project: u32, charter: u32) -> bool {
    for view in domain.core.tasks.view_tasks() {
        if view.project == project
            && let Some(task) = domain.core.tasks.task(view.number)
            && task.label == 3
            && task.ended_at.is_none()
            && task.executor == (jig_domain_tasks::Executor::Agent { charter })
        {
            return true;
        }
    }
    false
}

fn lacks_delegation(domain: &Domain, project: u32) -> bool {
    use jig_domain_authority::Executor;
    let Some(policy) = domain.core.authority.policy(project) else { return true };
    let mut owner = None;
    for role in &policy.roles {
        if role.number == 0 {
            owner = Some(&role.authority.delegation);
            break;
        }
    }
    let Some(owner) = owner else { return true };
    for delegation in [&policy.ceiling.delegation, owner] {
        for executor in
            [Executor::Charter(5), Executor::Procedure(6), Executor::Role(0), Executor::Job(3), Executor::Job(4)]
        {
            if !delegation.kinds.contains(&executor) {
                return true;
            }
        }
    }
    false
}

pub(crate) fn mark(domain: &mut Domain, project: u32) {
    let mut subjects = List::with_capacity(1);
    subjects.push(Token::new(ways::subject(project))).expect("one project's ways");
    domain.work.push(Work::Core(jig_domain_core::Event::Marked { connector: domain.config.forge_connector, subjects }));
}

/// The root sees the saved revision and role set in their committing decision.
pub(crate) fn saved(domain: &mut Domain, record: &jig_domain_core::Record) {
    use jig_domain_core::{CoreRecord, Record};
    match record {
        Record::Core(CoreRecord::Settings(jig_domain_core::settings::Record::Revision { history: false, .. })) => {
            mark_all(domain);
        }
        Record::People(
            jig_domain_people::Stored::Roles { project, .. } | jig_domain_people::Stored::Policy { project, .. },
        ) => mark(domain, *project),
        Record::Tasks(jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task)) if task.label == 3 => {
            mark(domain, task.project);
        }
        Record::Core(_) | Record::People(_) | Record::Tasks(_) | Record::Notes(_) | Record::Accounts(_) => {}
    }
}

pub(crate) fn mark_all(domain: &mut Domain) {
    let projects = domain.core.projects.clone();
    for &project in &projects {
        mark(domain, project);
    }
}

pub(crate) fn erased(domain: &mut Domain, key: &forge::Key) {
    match key {
        forge::Key::Repository(_) => mark_all(domain),
        forge::Key::Call(_)
        | forge::Key::ProposedEffect(_)
        | forge::Key::Hold(_)
        | forge::Key::Names(_)
        | forge::Key::Subscription { .. }
        | forge::Key::BranchHead(_)
        | forge::Key::PullState(_)
        | forge::Key::Ci { .. }
        | forge::Key::Landed(_)
        | forge::Key::Entry(_)
        | forge::Key::Client(_)
        | forge::Key::Closer(_)
        | forge::Key::Change(_)
        | forge::Key::Issue(_)
        | forge::Key::Release(_)
        | forge::Key::Application(_)
        | forge::Key::Writer(_)
        | forge::Key::Binding(_)
        | forge::Key::Setting(_)
        | forge::Key::Flight { .. }
        | forge::Key::Landing(_)
        | forge::Key::Receipt(_)
        | forge::Key::Forge(_)
        | forge::Key::ForgeNumber
        | forge::Key::Playground
        | forge::Key::Tier { .. }
        | forge::Key::Verification { .. }
        | forge::Key::LandingLog { .. }
        | forge::Key::Dismissed { .. }
        | forge::Key::Making(_)
        | forge::Key::ProjectFacts { .. }
        | forge::Key::Looked { .. }
        | forge::Key::Away { .. }
        | forge::Key::Decided { .. } => {}
    }
}

#[cfg(test)]
mod tests;
