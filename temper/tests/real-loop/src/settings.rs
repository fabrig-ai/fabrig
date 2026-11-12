//! Whole stock charter and template values in ctl's JSON vocabulary.
use serde_json::{Value, json};
fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("stock text")
}
fn model(value: &jig_domain_core::Model) -> Value {
    json!({"dialect":value.dialect,"endpoint":value.endpoint,"program":value.program.as_deref().map(text),
        "name":text(&value.name),"window":value.window,"output":value.output,"input_price":value.input_price,
        "cached_price":value.cached_price,"cache_write_price":value.cache_write_price,"output_price":value.output_price,
        "price_unit":value.price_unit,"efforts":value.efforts.iter().map(|effort| text(effort)).collect::<Vec<_>>(),"effort":value.effort.as_deref().map(text)})
}
/// Use the binary's defaults, with this deployment's models.
#[must_use]
pub fn stock(deployment: &Value) -> Value {
    let engine = configuration(deployment);
    let defaults = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let c = &defaults.charters[0];
    assert!(c.documents.is_empty(), "stock charter has no documents");
    let p = &c.policy;
    let charter = json!({"number":c.number,"rest":c.rest,"documents":[],"instructions":text(&p.instructions),"model":model(&p.model),"alternatives":p.alternatives.iter().map(model).collect::<Vec<_>>(),
        "waiting":p.waiting.as_nanos(),"resume":p.resume,"turns":p.turns,"time":p.time.as_nanos(),"reserve_turns":p.reserve_turns,"reserve_time":p.reserve_time.as_nanos(),
        "inspect":p.inspect,"modify":p.modify,"shell":p.shell,"agents":p.agents,"call_timeout":p.call_timeout.as_nanos(),"hosts":match c.hosts { jig_domain_fleet::Kinds::Workers=>"workers",jig_domain_fleet::Kinds::Engine=>"engine",jig_domain_fleet::Kinds::Both=>"both" },"spend":c.spend});
    let t = &defaults.templates[0];
    let jig_domain_people::TemplateExecutor::Charter(executor) = t.executor else { panic!("stock chat executor") };
    let jig_domain_tasks::Contract::Report { words } = t.contract else { panic!("stock chat report") };
    let jig_domain_people::TemplateBudget::AsAllowed { cap } = t.budget else { panic!("stock chat budget") };
    assert_eq!(t.deadline, jig_domain_people::TemplateDeadline::None);
    assert!(t.llm.is_none());
    let template = json!({"number":t.number,"name":{"bytes":text(&t.name),"cut":false},"executor":{"charter":executor},"contract":{"report":{"words":words}},"shallower":t.shallower,
        "budget":{"as_allowed":{"cap":cap}},"deadline":"none","tracked":t.tracked,"label":t.label,"grace":t.grace.map(skein_lib::Duration::as_nanos),"llm":null,
        "flags":{"conversation":t.flags.conversation,"listed":t.flags.listed,"retention":match t.flags.retention { jig_domain_tasks::Retention::Kept=>"kept",jig_domain_tasks::Retention::ForgottenOnceRead=>"forgotten_once_read" },
        "ended_by":match t.flags.ended_by { jig_domain_tasks::EndedBy::Requester=>"requester",jig_domain_tasks::EndedBy::Standing=>"standing",jig_domain_tasks::EndedBy::Result | jig_domain_tasks::EndedBy::Closer(_)=>"result" }}});
    json!({"charter":charter,"template":template})
}
#[must_use]
pub fn item(kind: &str, number: u32, value: Value) -> Value {
    let mut item = json!({"setting":{"kind":kind,"number":number}});
    item["set"] = value;
    item
}

/// Parse only startup configuration; the caller supplies typed defaults afterwards.
#[must_use]
pub fn configuration(deployment: &Value) -> jig_protocol_config::Engine {
    temper_playground::documents::settings::configuration(deployment)
}

/// Separate the imported world's policy values from seeds that name their settings.
pub fn stage_policies(value: &mut serde_json::Value) {
    let mut policies = value["engine"]["projects"].clone();
    for (index, policy) in policies.as_array_mut().expect("seed projects").iter_mut().enumerate() {
        if policy["policy"].is_number() {
            let original: Value =
                serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("world defaults");
            policy["policy"] = original["engine"]["projects"][0]["policy"].clone();
            policy["roles"] = original["engine"]["projects"][0]["roles"].clone();
        }
        policy["number"] = (index + 1).into();
        let seed = &mut value["engine"]["projects"][index];
        seed["policy"] = (index + 1).into();
        seed.as_object_mut().expect("seed").remove("roles");
    }
    value["world_settings"]["policies"] = policies;
}

/// The world's forge defaults are independent of the startup document.
#[must_use]
pub fn forge(deployment: &Value) -> Box<[temper_engine_domain_forge::settings::Stock]> {
    let flags = &deployment["world_settings"];
    let landing = &flags["landing"];
    let deployment = landing_rules(&landing["deployment"]);
    let mut projects = skein_lib::Map::with_capacity(
        u32::try_from(landing["projects"].as_array().map_or(0, Vec::len)).expect("world projects"),
    );
    if let Some(entries) = landing["projects"].as_array() {
        for entry in entries {
            let project = u32::try_from(entry["project"].as_u64().expect("world project")).expect("project number");
            projects.insert(project, landing_rules(&entry["rules"])).expect("world project rules");
        }
    }
    temper_engine_domain::landing_stock(
        &deployment,
        &projects,
        u32::try_from(flags["forge_charter"].as_u64().unwrap_or(1)).expect("world change charter"),
        4,
    )
}
fn landing_rules(values: &Value) -> Box<[temper_engine_domain::LandingRule]> {
    use temper_engine_domain::{Approval, Freshness, Gate, LandingRule};
    let mut rules = Vec::new();
    if let Some(values) = values.as_array() {
        for rule in values {
            let pattern = &rule["pattern"];
            let word = pattern["value"].as_str().expect("world pattern end").as_bytes().into();
            let mut gates = Vec::new();
            for gate in rule["gates"].as_array().expect("world gates") {
                gates.push(Gate {
                    number: u32::try_from(gate["number"].as_u64().expect("gate number")).expect("gate number"),
                    blocking: gate["blocking"].as_bool().expect("gate blocking"),
                    freshness: if gate["freshness"] == "clean" { Freshness::Clean } else { Freshness::Exact },
                });
            }
            let mut approvals = Vec::new();
            for approval in rule["approvals"].as_array().expect("world approvals") {
                approvals.push(Approval {
                    role: u32::try_from(approval["role"].as_u64().expect("approval role")).expect("role number"),
                    people: u32::try_from(approval["people"].as_u64().expect("approval people")).expect("people count"),
                    freshness: if approval["freshness"] == "clean" { Freshness::Clean } else { Freshness::Exact },
                });
            }
            rules.push(LandingRule {
                sets: Box::new([]),
                connector: 1,
                kind: 4,
                pattern: jig_domain_authority::Pattern {
                    segments: pattern["segments"]
                        .as_array()
                        .expect("world segments")
                        .iter()
                        .map(|segment| segment.as_str().expect("world segment").as_bytes().into())
                        .collect(),
                    last: if pattern["last"] == "exact" {
                        jig_domain_authority::Last::Exact(word)
                    } else {
                        jig_domain_authority::Last::Open(word)
                    },
                },
                enforce: rule["enforce"].as_bool().expect("world rule enforce"),
                ci: rule["ci"].as_bool().expect("world rule CI"),
                up_to_date: rule["up_to_date"].as_bool().expect("world rule up to date"),
                gates: gates.into_boxed_slice(),
                approvals: approvals.into_boxed_slice(),
            });
        }
    }
    rules.into_boxed_slice()
}
