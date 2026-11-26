//! A person's own change, built and admitted as their Create batch
//! (temper/docs/design/domain/forge/change/opened.md, 2 and 3).
//! Landing targets use the adoption (temper/docs/design/domain/forge/adoption.md, 12).
use super::{Domain, Work, forge, forge_route};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, List, Queue, Token};

#[expect(clippy::too_many_arguments, reason = "the authenticated application envelope accompanies its value")]
pub(super) fn admitted(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    decision: &mut crate::Decision,
    request: Token,
    token: Token,
    retention: Duration,
    person: u64,
    call: forge::OpenChange,
) {
    let project = call.project;
    let member = member(domain, env, person, call);
    let outcome = match &member {
        Ok(_) => None,
        Err(outcome) => Some(outcome.clone()),
    };
    let mut out = Queue::uncapped();
    let ready =
        domain.forge.opening(&crate::translate::environment_forge(env), request, token, retention, outcome, &mut out);
    forge_route::outputs(domain, env, decision, &mut out);
    if ready {
        domain.work.push(Work::Core(core::Event::ApplicationPart {
            request,
            ask: Box::new(people::Ask::Create { project, batch: Box::new([member.expect("prepared opening")]) }),
        }));
    }
}

fn repository(domain: &Domain, call: &forge::OpenChange) -> Result<forge::Repository, forge::OpenRefusal> {
    let repositories = domain.forge.opening_repositories(call.project);
    if let Some(named) = &call.repository {
        for repository in &repositories {
            if repository.owner == named.owner && repository.name == named.name {
                return Ok((*repository).clone());
            }
        }
        return Err(forge::OpenRefusal::NotARepository);
    }
    if repositories.is_empty() {
        Err(forge::OpenRefusal::NoRepository)
    } else if let [repository] = repositories.as_ref() {
        Ok((*repository).clone())
    } else {
        let mut names = List::with_capacity(u32::try_from(repositories.len()).expect("bounded repositories"));
        for repository in &repositories {
            names
                .push(forge::DraftRepository { owner: repository.owner.clone(), name: repository.name.clone() })
                .expect("bounded repositories");
        }
        Err(forge::OpenRefusal::Several(names.into_boxed()))
    }
}

fn member(
    domain: &Domain,
    env: &Env<crate::Limits>,
    person: u64,
    call: forge::OpenChange,
) -> Result<people::Member, forge::ApplicationOutcome> {
    let repository = match repository(domain, &call) {
        Ok(repository) => repository,
        Err(why) => return Err(forge::ApplicationOutcome::OpenRefused(why)),
    };
    let base = match &call.into {
        Some(branch) => branch.clone(),
        None => repository.landing.clone(),
    };
    if base.starts_with(&repository.prefix) {
        return Err(forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::UnderPrefix {
            owner: repository.owner.clone(),
            name: repository.name.clone(),
            prefix: repository.prefix.clone(),
        }));
    }
    if call.branch.len() > skein_lib::usize_of(env.limits.forge.name_bytes)
        || base.len() > skein_lib::usize_of(env.limits.forge.name_bytes)
    {
        return Err(forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::NamePast {
            bound: env.limits.forge.name_bytes,
        }));
    }
    let offer = match domain.engine.core().creating_offer(call.project, person, env.limits.tasks.tree_tasks) {
        Ok(offer) => offer,
        Err(why) => return Err(forge::ApplicationOutcome::CoreRefused(why)),
    };
    let authority = core::task_authority(&offer);
    let tasks_left = offer.delegation.tasks;
    let spend = match call.budget {
        forge::OpenBudget::Amount(amount) => amount,
        forge::OpenBudget::AsAllowed => offer.budget.spend,
    };
    let change = crate::ChangeCall::words(words(person, &call, &base), None);
    let mut source = List::with_capacity(env.limits.forge.name_bytes.saturating_add(65));
    append(&mut source, &call.branch);
    if let Some(at) = &call.at {
        append(&mut source, b":");
        append(&mut source, at);
    }
    let origin = forge_route::ChangeOrigin {
        authority,
        tasks_left,
        source: Some(forge_route::OpenedSource { title: call.title, parameter: source.into_boxed() }),
    };
    let delegate = match forge_route::chat_change(domain, env, &origin, &change, &repository, base, None, spend) {
        Ok(delegate) => delegate,
        Err(why) => return Err(forge::ApplicationOutcome::CoreRefused(refusal(domain, &origin, spend, why))),
    };
    let mut parameters = List::with_capacity(env.limits.tasks.parameters);
    for parameter in delegate.spec.parameters {
        parameters
            .push(match parameter {
                tasks::Parameter::Number { name, value } => people::Parameter::Number { name, value },
                tasks::Parameter::Bytes { name, value } => people::Parameter::Bytes { name, value },
                tasks::Parameter::Resource { name, connector, resource } => {
                    people::Parameter::Resource { name, connector, resource }
                }
            })
            .expect("builder checked parameters");
    }
    let mut authority =
        core::authority_to_people(&core::tasks_authority(&delegate.authority)).expect("builder authority fits");
    // A core part takes the role's models; it carries no model-name context.
    authority.models = None;
    Ok(people::Member {
        executor: people::TaskExecutor::Procedure {
            connector: domain.config.forge_connector,
            code: forge_route::CHANGE_CODE,
        },
        words: delegate.spec.words,
        parameters: parameters.into_boxed(),
        inputs: Box::new([]),
        contract: people::Contract::Change {
            connector: domain.config.forge_connector,
            kind: 1,
            words: env.limits.tasks.result_bytes,
        },
        authority,
        budget: match call.budget {
            forge::OpenBudget::Amount(_) => people::Budget::Named,
            forge::OpenBudget::AsAllowed => people::Budget::AsAllowed { cap: None },
        },
        deadline: people::Deadline::Named,
        wake: None,
        dependencies: Box::new([]),
        named: Box::new([]),
        symbols: Box::new([]),
        tracked: None,
        label: 0,
        flags: people::Flags::PLAIN,
        llm: None,
    })
}

fn refusal(domain: &Domain, origin: &forge_route::ChangeOrigin, spend: u64, why: tasks::Refusal) -> people::Refusal {
    let lacking = if why == tasks::Refusal::Tree {
        people::Lack::Tasks { left: origin.tasks_left }
    } else if why == tasks::Refusal::Depth {
        people::Lack::Depth { most: origin.authority.delegation.depth.saturating_sub(1) }
    } else if why == tasks::Refusal::Funding {
        people::Lack::Budget {
            needs: domain.engine.core().authority.rules().minimum_run_spend.saturating_add(1),
            left: spend,
        }
    } else if why == tasks::Refusal::Executor {
        people::Lack::TaskKind { executor: None }
    } else {
        return people::Refusal::Limit { which: people::LimitKind::Spec };
    };
    people::Refusal::Authority { proposable: None, lacking }
}

fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("bounded opening words");
    }
}

fn words(person: u64, call: &forge::OpenChange, base: &[u8]) -> Box<[u8]> {
    // Fixed sentence, person digits, two bounded branch names, commit and body.
    let mut words = List::with_capacity(18_624);
    append(&mut words, b"Opened by person ");
    append(&mut words, &forge_route::decimal(person));
    append(&mut words, b" from ");
    append(&mut words, &call.branch);
    append(&mut words, b" at ");
    append(&mut words, call.at.as_deref().unwrap_or(b"its head"));
    append(&mut words, b" into ");
    append(&mut words, base);
    append(&mut words, b".");
    if let Some(body) = &call.body {
        append(&mut words, b"\n\n");
        append(&mut words, body);
    }
    words.into_boxed()
}
