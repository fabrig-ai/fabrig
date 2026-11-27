//! Stock relations and mutations (temper/docs/design/testing/settings-tests.md, 3, 4 and 6).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::print_stderr,
    reason = "ordinary step tests collect discovered relations and print each mutation with its design citation"
)]
use jig_domain_core::settings::{
    self as s, CatalogueModel, Current, Defaults, Edit, Failure, Fault, Item, Setting, Value,
};
use jig_domain_people::TemplateExecutor;

#[derive(PartialEq, Eq)]
struct Relation {
    name: &'static str,
    design: &'static str,
    exception: &'static str,
}
impl Relation {
    fn words(&self) -> String {
        format!("{} ({}; exception: {})", self.name, self.design, self.exception)
    }
}
const EXECUTOR: Relation = Relation {
    name: "template executor",
    design: "temper/docs/design/testing/settings-tests.md, 3",
    exception: "a chosen executor names no charter",
};
const CHARTER_MODEL: Relation = Relation {
    name: "charter models",
    design: "temper/docs/design/testing/settings-tests.md, 3",
    exception: "rest may follow at least one live named model",
};
const PRODUCER: Relation = Relation {
    name: "producer",
    design: "temper/docs/design/protocol/configuration/stock.md; settings.md, 2 and 8",
    exception: "none",
};
const TEMPLATE_MODEL: Relation = Relation {
    name: "template model",
    design: "temper/docs/design/testing/settings-tests.md, 3 and 4",
    exception: "a template may name no model",
};
const CAPABILITY: Relation = Relation {
    name: "provider capability",
    design: "temper/docs/design/testing/settings-tests.md, 4 and 6",
    exception: "a model nothing names may lack efforts (demo)",
};

fn fixture() -> (jig_protocol_config::Engine, Defaults) {
    let engine = super::stock::engine(&super::stock::generated());
    let defaults = crate::stock::checked(&engine.llm, engine.limits.core.tasks.result_bytes).expect("stock defaults");
    (engine, defaults)
}

struct ModelReference {
    relation: &'static Relation,
    origin: String,
    name: Box<[u8]>,
}
fn named_models(defaults: &Defaults) -> Vec<ModelReference> {
    let mut references = Vec::new();
    for charter in &defaults.charters {
        let templates: Vec<_> = defaults
            .templates
            .iter()
            .filter(|t| t.executor == TemplateExecutor::Charter(charter.number))
            .map(|t| t.number)
            .collect();
        let origin = format!("templates {templates:?} -> charter {}", charter.number);
        for model in core::iter::once(&charter.policy.model).chain(charter.policy.alternatives.iter()) {
            references.push(ModelReference {
                relation: &CHARTER_MODEL,
                origin: origin.clone(),
                name: model.name.clone(),
            });
        }
    }
    for template in &defaults.templates {
        if let Some(choice) = &template.llm {
            for (main, subagents) in core::iter::once((choice.main, choice.subagents.as_ref()))
                .chain(choice.later.iter().map(|later| (later.main, later.subagents.as_ref())))
            {
                for chosen in core::iter::once(&main).chain(subagents) {
                    let number = u32::from(chosen.model) + 1;
                    let model = defaults.models.iter().find(|m| m.number == number).unwrap_or_else(|| {
                        panic!("{}: template {} -> model {number}", TEMPLATE_MODEL.words(), template.number)
                    });
                    references.push(ModelReference {
                        relation: &TEMPLATE_MODEL,
                        origin: format!("template {}", template.number),
                        name: model.model.name.clone(),
                    });
                }
            }
        }
    }
    references
}
fn named<'a>(defaults: &'a Defaults, reference: &ModelReference) -> &'a CatalogueModel {
    defaults.models.iter().find(|m| m.model.name == reference.name).unwrap_or_else(|| {
        panic!("{}: {} -> missing model {:?}", reference.relation.words(), reference.origin, reference.name)
    })
}
fn model_name(model: &CatalogueModel) -> &str {
    core::str::from_utf8(&model.model.name).expect("catalogue model names are UTF-8")
}
fn kept_by_preset(defaults: &Defaults, model: &CatalogueModel) -> bool {
    defaults.providers.iter().any(|p| {
        !p.withdrawn
            && p.provider.number == model.model.provider
            && p.provider.list.as_ref().is_some_and(|list| s::catalogue::kept(&model.model.name, list))
    })
}
fn named_once(defaults: &Defaults) -> Vec<&CatalogueModel> {
    let mut models = Vec::new();
    for reference in named_models(defaults) {
        let model = named(defaults, &reference);
        if !models.contains(&model) {
            models.push(model);
        }
    }
    models
}
fn producers() -> Vec<(String, u32)> {
    let configured = super::fixtures::engine();
    let limits = crate::limits::engine(&configured.jig.limits, &configured.limits).expect("fixture limits");
    let mut references = Vec::new();
    for row in &crate::stock::forge(&limits) {
        if row.kind == 3 {
            assert_ne!(row.number, 3, "{}: retired kind 3 number 3", PRODUCER.words());
            let temper_engine_domain_forge::settings::Value::Charter(number) = row.value.value else {
                panic!("{}: kind 3 number {} not a charter", PRODUCER.words(), row.number);
            };
            references.push((format!("kind 3 number {}", row.number), number));
        }
    }
    for way in &crate::stock::ways() {
        for role in &way.roles {
            references.push((format!("way {:?} standing role", way.name), role.charter));
        }
        if let Some(owner) = &way.triager {
            let temper_engine_domain_forge::settings::AreaOwner::Charter(number) = owner else {
                panic!("{}: stock way {:?} kind 15 is not a charter", PRODUCER.words(), way.name);
            };
            references.push((format!("way {:?} kind 15", way.name), *number));
        }
    }
    references
}
#[test]
fn the_stocks_references_resolve_to_live_offers() {
    let (_, defaults) = fixture();
    let mut executors = 0_u32;
    for template in &defaults.templates {
        if let TemplateExecutor::Charter(number) = template.executor {
            assert!(
                defaults.charters.iter().any(|c| c.number == number),
                "{}: template {} -> missing charter {number}",
                EXECUTOR.words(),
                template.number
            );
            executors += 1;
        }
    }
    let references = named_models(&defaults);
    for reference in &references {
        let row = named(&defaults, reference);
        assert!(
            !row.withdrawn,
            "{}: {} -> withdrawn model {}",
            reference.relation.words(),
            reference.origin,
            model_name(row)
        );
        if reference.relation == &TEMPLATE_MODEL {
            assert!(
                kept_by_preset(&defaults, row),
                "{}: {} -> model {} not kept by a provider preset",
                TEMPLATE_MODEL.words(),
                reference.origin,
                model_name(row)
            );
        }
    }
    let producers = producers();
    for (origin, number) in &producers {
        assert!(
            ![6, 7, 8].contains(number) && defaults.charters.iter().any(|c| c.number == *number),
            "{}: template work via {origin} -> missing or retired charter {number}",
            PRODUCER.words()
        );
    }
    eprintln!(
        "references: {executors} template executors, {} model edges, {} producer edges",
        references.len(),
        producers.len()
    );
}
#[test]
fn named_models_have_their_provider_capabilities() {
    let (_, defaults) = fixture();
    let references = named_models(&defaults);
    for reference in &references {
        let row = named(&defaults, reference);
        assert!(
            kept_by_preset(&defaults, row),
            "{}: {} -> model {} not kept by its provider preset",
            CAPABILITY.words(),
            reference.origin,
            model_name(row)
        );
        assert!(
            !row.model.efforts.is_empty()
                && row.model.effort.as_ref().is_some_and(|e| row.model.efforts.contains(e))
                && row.model.window != 0
                && row.model.output != 0,
            "{}: {} -> incomplete model {}: {:?}",
            CAPABILITY.words(),
            reference.origin,
            model_name(row),
            row.model
        );
        assert_eq!(
            s::catalogue::shape(row, row.number, &defaults.providers),
            Ok(()),
            "{}: {}",
            CAPABILITY.words(),
            reference.origin
        );
    }
    eprintln!("capabilities: {} named edges, {} distinct models", references.len(), named_once(&defaults).len());
}

// Also used to clear competing references, as settings-tests.md, 6 prescribes.
fn charter_users(defaults: &Defaults, number: u32) -> Vec<Setting> {
    use jig_domain_authority::Executor;
    let mut users = Vec::new();
    if defaults.rules.ceiling.delegation.kinds.contains(&Executor::Charter(number)) {
        users.push(Setting::Rules);
    }
    for (&policy, configured) in &defaults.policies {
        if core::iter::once(&configured.policy.ceiling)
            .chain(configured.policy.roles.iter().map(|r| &r.authority))
            .any(|a| a.delegation.kinds.contains(&Executor::Charter(number)))
            || configured.templates.iter().any(|t| t.executor == Some(TemplateExecutor::Charter(number)))
        {
            users.push(Setting::Policy(policy));
        }
    }
    for template in &defaults.templates {
        if template.executor == TemplateExecutor::Charter(number) {
            users.push(Setting::Template(template.number));
        }
    }
    users
}
fn clear_authority(authority: &mut jig_domain_authority::Authority, number: u32) {
    authority.delegation.kinds = authority
        .delegation
        .kinds
        .iter()
        .copied()
        .filter(|kind| *kind != jig_domain_authority::Executor::Charter(number))
        .collect();
}
fn isolated(defaults: &mut Defaults, charter: u32, template: u32) {
    let users = charter_users(defaults, charter);
    let other_templates: Vec<_> = users
        .iter()
        .filter_map(|user| match user {
            Setting::Template(number) if *number != template => Some(*number),
            Setting::Template(_)
            | Setting::Charter(_)
            | Setting::Policy(_)
            | Setting::Rules
            | Setting::Application { .. }
            | Setting::ProjectPolicy { .. }
            | Setting::Model(_)
            | Setting::Worker(_)
            | Setting::Provider(_)
            | Setting::Limits => None,
        })
        .collect();
    for user in users {
        match user {
            Setting::Rules => clear_authority(&mut defaults.rules.ceiling, charter),
            Setting::Policy(number) => {
                let configured = defaults.policies.get_mut(&number).expect("discovered policy");
                clear_authority(&mut configured.policy.ceiling, charter);
                for role in &mut configured.policy.roles {
                    clear_authority(&mut role.authority, charter);
                }
                configured.templates = configured
                    .templates
                    .iter()
                    .filter(|t| {
                        t.executor != Some(TemplateExecutor::Charter(charter)) && !other_templates.contains(&t.template)
                    })
                    .cloned()
                    .collect();
            }
            Setting::Template(_) => {}
            Setting::Charter(_)
            | Setting::Application { .. }
            | Setting::ProjectPolicy { .. }
            | Setting::Model(_)
            | Setting::Worker(_)
            | Setting::Provider(_)
            | Setting::Limits => panic!("only discovered charter users"),
        }
    }
    defaults.templates = defaults.templates.iter().filter(|t| !other_templates.contains(&t.number)).cloned().collect();
}
fn decide(engine: &jig_protocol_config::Engine, defaults: Defaults, items: &[Item]) -> Result<(), Failure> {
    let current = Current::new(defaults, &engine.limits.core, &[]);
    let config = jig_protocol_config::core(engine, [1; 16], 7);
    s::check::check_change(&current, items, &config.settings, &config.connectors, &engine.limits.core)
}
#[test]
fn removing_a_templates_charter_is_refused() {
    let (_, defaults) = fixture();
    let mut cases = 0_u32;
    for template in &defaults.templates {
        if let TemplateExecutor::Charter(charter) = template.executor {
            let name = format!("{}: template {} -> remove charter {charter}", EXECUTOR.words(), template.number);
            let items = [Item { setting: Setting::Charter(charter), edit: Edit::Removed }];
            eprintln!("{name}, stock as shipped");
            let (engine, defaults) = fixture();
            let users = charter_users(&defaults, charter);
            let result = decide(&engine, defaults, &items);
            let failure = result.expect_err(&name);
            assert_eq!(failure.fault, Fault::Reference, "{name}: {result:?}");
            assert!(users.contains(&failure.setting), "{name}: refused a non-referent: {result:?}");
            eprintln!("{name}, isolated template");
            let (engine, mut defaults) = fixture();
            isolated(&mut defaults, charter, template.number);
            let current = Current::new(defaults, &engine.limits.core, &[]);
            let config = jig_protocol_config::core(&engine, [1; 16], 7);
            assert_eq!(
                s::check::check_change(&current, &[], &config.settings, &config.connectors, &engine.limits.core),
                Ok(()),
                "{name}: valid isolated setup"
            );
            let result =
                s::check::check_change(&current, &items, &config.settings, &config.connectors, &engine.limits.core);
            assert_eq!(
                result,
                Err(Failure { setting: Setting::Template(template.number), fault: Fault::Reference, detail: None }),
                "{name}: {result:?}"
            );
            cases += 2;
        }
    }
    assert!(cases > 0, "stock has charter executors");
    eprintln!("charter removal mutations: {cases} refused");
}
#[test]
fn withdrawing_each_named_model_keeps_charters_resolvable() {
    let (engine, defaults) = fixture();
    for row in named_once(&defaults) {
        let name = format!("{}: withdraw model {}", CHARTER_MODEL.words(), model_name(row));
        eprintln!("{name}");
        let mut withdrawn = row.clone();
        withdrawn.withdrawn = true;
        let items = [Item { setting: Setting::Model(row.number), edit: Edit::Set(Value::Model(Box::new(withdrawn))) }];
        let (_, stock) = fixture();
        let current = Current::new(stock, &engine.limits.core, &[]);
        let config = jig_protocol_config::core(&engine, [1; 16], 7);
        let result =
            s::check::check_change(&current, &items, &config.settings, &config.connectors, &engine.limits.core);
        assert_eq!(result, Ok(()), "{name}: {result:?}");
        let catalogue = s::catalogue::overlay(&current, false, &items);
        for charter in &defaults.charters {
            let resolved = s::charters::resolved(charter, &catalogue, &engine.limits.core).expect(&name);
            for model in core::iter::once(&resolved.policy.model).chain(resolved.policy.alternatives.iter()) {
                assert!(
                    catalogue.iter().any(|row| !row.withdrawn && row.model.name == model.name),
                    "{name}: charter {} resolved withdrawn model",
                    charter.number
                );
            }
        }
    }
    eprintln!(
        "withdrawals: {} accepted, all {} charters live each time",
        named_once(&defaults).len(),
        defaults.charters.len()
    );
}
#[test]
fn malformed_named_capabilities_are_refused_by_the_catalogue() {
    let (engine, defaults) = fixture();
    for row in named_once(&defaults) {
        for empty in [true, false] {
            let mutation = if empty { "empty efforts, keep effort" } else { "effort outside efforts" };
            let name = format!("{}: model {}: {mutation}", CAPABILITY.words(), model_name(row));
            eprintln!("{name}");
            let mut changed = row.clone();
            if empty {
                changed.model.efforts = Box::new([]);
            } else {
                changed.model.effort = Some(b"not-an-effort".as_slice().into());
            }
            assert_eq!(s::catalogue::shape(&changed, row.number, &defaults.providers), Err(Fault::Shape), "{name}");
            let items =
                [Item { setting: Setting::Model(row.number), edit: Edit::Set(Value::Model(Box::new(changed))) }];
            let (_, stock) = fixture();
            let result = decide(&engine, stock, &items);
            assert_eq!(
                result,
                Err(Failure { setting: Setting::Model(row.number), fault: Fault::Shape, detail: None }),
                "{name}: {result:?}"
            );
        }
    }
    eprintln!("capability mutations: {} refused", named_once(&defaults).len() * 2);
}
#[test]
#[ignore = "provider failures: a discovered model has no efforts (temper/docs/design/draft/failure-worlds.md, 7, P128)"]
fn a_discovered_model_has_efforts() {
    let (engine, defaults) = fixture();
    let provider = defaults.providers.iter().find(|p| p.provider.name.as_ref() == b"openai").expect("OpenAI preset");
    let row = jig_domain_link::ListedModel {
        name: b"gpt-6.2-sol".as_slice().into(),
        shown: None,
        window: None,
        output: None,
        efforts: Box::new([]),
    };
    let config = jig_protocol_config::core(&engine, [1; 16], 7);
    let listed = s::catalogue::from_list(
        provider.provider.number,
        provider.provider.list.as_ref().expect("OpenAI list"),
        &[row],
        0,
        skein_lib::Wall::from_nanos(1_000_000_000),
        &defaults.models,
        &config.settings.endpoints,
    );
    assert_eq!(listed.items.len(), 1, "P128: the preset keeps the discovered row");
    let charters = defaults.charters.clone();
    let current = Current::new(defaults, &engine.limits.core, &[]);
    assert_eq!(
        s::check::check_change(&current, &listed.items, &config.settings, &config.connectors, &engine.limits.core),
        Ok(())
    );
    let catalogue = s::catalogue::overlay(&current, false, &listed.items);
    let discovered = catalogue.iter().find(|m| m.model.name.as_ref() == b"gpt-6.2-sol").expect("discovered model");
    assert!(
        charters.iter().filter(|c| c.rest).any(|c| {
            let resolved = s::charters::resolved(c, &catalogue, &engine.limits.core).expect("stock charter resolves");
            core::iter::once(&resolved.policy.model)
                .chain(resolved.policy.alternatives.iter())
                .any(|m| m.name == discovered.model.name)
        }),
        "P128: a stock charter can pick the discovered model through rest"
    );
    assert!(
        !discovered.model.efforts.is_empty(),
        "P128: a discovered model a stock charter can pick must have efforts: gpt-6.2-sol"
    );
}
