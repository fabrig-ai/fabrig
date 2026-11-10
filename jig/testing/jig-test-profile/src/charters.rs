//! Charter profiles (`jig/docs/design/domain/testing.md`, 2).
use jig_domain_accounts as accounts;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_test_domain as root;

/// Shared instructions and one part for each activation in the per-wake story
/// (`jig/docs/design/domain/tasks/messages.md`, 6).
#[must_use]
pub fn per_wake_charter() -> core::Charter {
    use jig_domain_tasks::Reasons;
    let mut charter = research_charter();
    charter.policy.resume = false;
    charter.policy.instructions = b"S".as_slice().into();
    charter.policy.alternatives = Box::new([charter.policy.model.clone()]);
    charter.policy.model = crate::scripted_model();
    charter.parts =
        [(Reasons::FIRST, b"F"), (Reasons::QUESTION, b"Q"), (Reasons::ESCALATION, b"H"), (Reasons::RESULT, b"R")]
            .into_iter()
            .map(|(on, instructions)| core::CharterPart { on, instructions: instructions.as_slice().into() })
            .collect();
    charter
}

#[must_use]
pub fn research_charter() -> core::Charter {
    let mut charter = crate::config(0).core.defaults.charters[0].clone();
    charter.number = 2;
    charter.policy.instructions = b"Research the question.".as_slice().into();
    charter.policy.model.provider = 2;
    charter.policy.model.name = b"research".as_slice().into();
    charter
}

#[must_use]
pub fn two_charters(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = crate::effect_fixture(seed, false);
    limits.core.tasks.charters = 2;
    limits.core.tasks.executor_kinds = 3;
    limits.core.fleet.slots = 2;
    limits.core.authority.executors = 3;
    limits.core.documents.executors = 3;
    limits.core.accounts.accounts = 2;
    let kinds: Box<[authority::Executor]> =
        Box::new([authority::Executor::Charter(1), authority::Executor::Charter(2), authority::Executor::Procedure(1)]);
    let mut rules = config.core.defaults.rules.clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = config.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds.clone_from(&kinds);

    config.core.defaults.rules =
        (authority::Domain::new(rules, limits.core.authority).expect("charter policy")).rules().clone();

    config.core.seed_projects[0].policy.policy = policy;
    let research = research_charter();
    config.core.settings.models = Box::new([config.core.settings.models[0].clone(), research.policy.model.clone()]);
    config.core.defaults.charters = Box::new([config.core.defaults.charters[0].clone(), research]);
    config.core.settings.llm_providers.providers = Box::new([
        config.core.settings.llm_providers.providers[0].clone(),
        accounts::Provider {
            list: None,
            name: b"Fake".as_slice().into(),
            signed_in_plan: true,
            manage_usage: Some(b"https://usage.example/".as_slice().into()),
            number: 2,
            offers: accounts::Offers { sign_in: true, key: false, programs: Box::new([]) },
        },
    ]);
    let room = limits.core.routes;
    let connectors = jig_test_connector::OUTPUTS_BOUND * 2;
    limits.journal.writes = limits.journal.writes.max(room + connectors);
    (config, limits)
}
