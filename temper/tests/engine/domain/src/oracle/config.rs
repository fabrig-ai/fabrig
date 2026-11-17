//! Fixture-owned restart configuration (temper/docs/design/domain/root.md, 11).
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
    engine::Config {
        host_part: config.host_part,
        kinds_stock: config
            .kinds_stock
            .iter()
            .map(|v| {
                let mut v = v.clone();
                if v.kind == 4 || v.kind == 5 {
                    v.value.size = temper_engine_smith::kinds::size(&v.value.value);
                }
                v
            })
            .collect(),
        ways_stock: config.ways_stock.clone(),
        period_span: config.period_span,
        core: core::Config {
            parts: config.core.parts.clone(),
            jobs: c.jobs.clone(),
            kept_starts: Box::new([]),
            worker_names: c.worker_names.clone(),
            deployment: c.deployment,
            seed: c.seed,
            workers_in_process: c.workers_in_process,
            hub_turns: c.hub_turns,
            owners: c.owners.clone(),
            grants: c.grants.clone(),
            kinds: c.kinds.clone(),
            defaults: core::settings::Defaults {
                models: c.defaults.models.clone(),
                charters: c.defaults.charters.clone(),
                templates: c.defaults.templates.clone(),
                policies,
                rules: c.defaults.rules.clone(),
            },
            settings: core::Settings {
                private_tools: s.private_tools.clone(),
                endpoints: s.endpoints.clone(),
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
        landing_held: config.landing_held,
        forge: config.forge.clone(),
        identities: config.identities.clone(),
        forges: config.forges.clone(),
        under_box: config.under_box,
        machine: config.machine.clone(),
        playground: config.playground.clone(),
    }
}

/// Typed settings that a world stores as its start's defaults.
pub fn set_landing(config: &mut engine::Config, project: u32, landing: temper_fixtures::landing::ProjectLanding) {
    use temper_engine_domain_forge::settings as s;
    let (landing, gates) = landing.values();
    let mut stock: Vec<_> =
        config.kinds_stock.iter().filter(|v| v.scope != project || (v.kind != 4 && v.kind != 5)).cloned().collect();
    for (kind, value) in [(4, s::Value::Landing(landing)), (5, s::Value::Gates(gates))] {
        stock.push(s::Stock {
            kind,
            scope: project,
            number: 1,
            value: s::SizedValue { size: temper_engine_smith::kinds::size(&value), value },
        });
    }
    config.kinds_stock = stock.into_boxed_slice();
}
pub fn landing(config: &mut engine::Config, project: u32) -> &mut temper_engine_domain_forge::settings::ProjectLanding {
    use temper_engine_domain_forge::settings as s;
    let index = config.kinds_stock.iter().position(|v| v.kind == 4 && v.scope == project);
    let index = index.unwrap_or_else(|| {
        let mut stock = config.kinds_stock.to_vec();
        let value = s::Value::Landing(s::ProjectLanding::default());
        stock.push(s::Stock {
            kind: 4,
            scope: project,
            number: 1,
            value: s::SizedValue { size: temper_engine_smith::kinds::size(&value), value },
        });
        config.kinds_stock = stock.into_boxed_slice();
        config.kinds_stock.len() - 1
    });
    let s::Value::Landing(value) = &mut config.kinds_stock[index].value.value else { panic!("kind 4 landing") };
    value
}
