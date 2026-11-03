//! Fixture-owned restart configuration (temper/docs/design/domain/root.md, 12).
use jig_domain_core as core;
use temper_engine_domain as engine;

/// Copy startup inputs before the domain consumes them; no state is read from the domain.
#[must_use]
pub fn copy(config: &engine::Config) -> engine::Config {
    let c = &config.core;
    let s = &c.settings;
    let mut policies = skein_lib::Map::with_capacity(c.defaults.policies.capacity());
    for (key, value) in &c.defaults.policies {
        policies.insert(*key, value.clone()).expect("copied policy");
    }
    let mut projects = skein_lib::Map::with_capacity(config.landing_rules.projects.capacity());
    for (key, value) in &config.landing_rules.projects {
        projects.insert(*key, value.clone()).expect("copied project");
    }
    let mut landings = skein_lib::Map::with_capacity(config.landings.capacity());
    for (key, value) in &config.landings {
        landings.insert(*key, value.clone()).expect("copied landing");
    }
    engine::Config {
        kinds_stock: Box::default(),
        period_span: config.period_span,
        core: core::Config {
            jobs: c.jobs.clone(),
            kept_starts: Box::new([]),
            worker_names: c.worker_names.clone(),
            deployment: c.deployment,
            seed: c.seed,
            workers_in_process: c.workers_in_process,
            hub_turns: c.hub_turns,
            owners: c.owners.clone(),
            grants: c.grants.clone(),
            kinds: Box::default(),
            defaults: core::settings::Defaults {
                charters: c.defaults.charters.clone(),
                templates: c.defaults.templates.clone(),
                policies,
                rules: c.defaults.rules.clone(),
            },
            settings: core::Settings {
                deployment_provider: s.deployment_provider,
                models: s.models.clone(),
                resume_bytes: s.resume_bytes,
                tail_bytes: s.tail_bytes,
                tools: s.tools,
                llm_providers: s.llm_providers.clone(),
                key_bytes: s.key_bytes,
                redirect_bytes: s.redirect_bytes,
            },
            seed_projects: c
                .seed_projects
                .iter()
                .map(|p| core::SeedProject { number: p.number, name: p.name.clone(), policy: p.policy.clone() })
                .collect(),
            connectors: c.connectors.clone(),
        },
        landing_rules: engine::LandingRules { deployment: config.landing_rules.deployment.clone(), projects },
        landings,
        forge: config.forge.clone(),
        identities: config.identities.clone(),
        forge_charter: config.forge_charter,
        forges: config.forges.clone(),
        under_box: config.under_box,
        machine: config.machine.clone(),
        playground: config.playground.clone(),
    }
}
