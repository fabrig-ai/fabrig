//! Charter policy and endpoint resolution (smith/docs/design/protocol/charter.md, sections 2, 5 and 7).
use crate::Endpoints;
use alloc::boxed::Box;
use skein_lib::{List, Reader};
use smith_charter as wire;
use smith_domain::run;

/// Decode one charter and resolve all LLM endpoint names before admission.
pub fn decode_charter(
    bytes: &[u8],
    limits: &wire::v4::Limits,
    endpoints: &Endpoints,
) -> Result<run::Charter, run::Invalid> {
    if bytes.len() < 2 {
        return Err(run::Invalid::MalformedCharter);
    }
    if bytes.get(..2) != Some(&[0, 4][..]) {
        return Err(run::Invalid::CharterVersion);
    }
    let Ok(charter) = wire::Charter::decode(limits, &mut Reader::new(bytes)) else {
        return Err(run::Invalid::MalformedCharter);
    };
    translate_charter(&charter, endpoints)
}

#[expect(clippy::manual_map, reason = "production steps do not use closures")]
fn translate_charter(charter: &wire::Charter, endpoints: &Endpoints) -> Result<run::Charter, run::Invalid> {
    let mut sections = List::with_capacity(charter.brief().len());
    for section in charter.brief() {
        let item = run::Section { title: Box::from(section.title()), text: Box::from(section.body()) };
        if sections.push(item).is_err() {
            return Err(run::Invalid::TooLarge);
        }
    }

    let tools = charter.tools();
    let families = tools.families();
    let mut host_tools = List::with_capacity(tools.host().len());
    for tool in tools.host() {
        let effect = match tool.effect() {
            wire::Effect::Read => run::HostEffect::Read,
            wire::Effect::Write => run::HostEffect::Write,
        };
        let item = run::HostTool {
            name: Box::from(tool.name()),
            description: Box::from(tool.description()),
            schema: Box::from(tool.input()),
            effect,
            timeout: tool.deadline(),
        };
        if host_tools.push(item).is_err() {
            return Err(run::Invalid::TooLarge);
        }
    }

    let deliver = match tools.deliver() {
        Some(rule) => Some(change_rule(rule)?),
        None => None,
    };
    let conventions = match charter.conventions() {
        Some(paths) => Some(run::Conventions { guide: Box::from(paths.guide()), checks: Box::from(paths.checks()) }),
        None => None,
    };
    let mut models = List::with_capacity(charter.models().len());
    for model in charter.models() {
        if models.push(llm(model, endpoints)?).is_err() {
            return Err(run::Invalid::TooLarge);
        }
    }
    let budget = charter.budget();
    Ok(run::Charter {
        mode: match charter.mode() {
            wire::Mode::Finite => run::Mode::Finite,
            wire::Mode::Open => run::Mode::Open,
        },
        resume: charter.resume(),
        waiting: charter.waiting(),
        instructions: Box::from(charter.instructions()),
        brief: run::Brief { sections: sections.into_boxed() },
        conventions,
        grants: run::charter::Grants {
            wait: tools.wait(),
            deliver,
            tools: run::charter::Tools {
                inspect: families.inspect(),
                modify: families.modify(),
                shell: families.shell(),
            },
            agents: families.agents(),
            host_tools: host_tools.into_boxed(),
        },
        outcome: contract(charter.contract())?,
        budget: run::Budget { turns: budget.turns(), spend: budget.spend(), time: budget.time() },
        reserve: run::charter::Reserve { turns: budget.reserve().turns(), time: budget.reserve().time() },
        llm: llm(charter.main(), endpoints)?,
        models: models.into_boxed(),
    })
}

fn llm(value: &wire::Llm, endpoints: &Endpoints) -> Result<run::charter::Llm, run::Invalid> {
    let Some(endpoint) = endpoints.resolve(value.endpoint()) else {
        return Err(run::Invalid::Endpoint);
    };
    let prices = value.prices();
    Ok(run::charter::Llm {
        prices: run::Prices {
            input: prices.input(),
            cached: prices.cached(),
            cache_write: *prices.cache_write(),
            output: prices.output(),
            unit: prices.unit(),
        },
        dialect: endpoint.dialect,
        account: *value.account(),
        endpoint: run::charter::Endpoint(endpoint.number),
        model: Box::from(value.model()),
        effort: value.effort().clone(),
        window: value.window(),
        output: value.output(),
    })
}

fn contract(value: &wire::Contract) -> Result<run::outcome::OutcomeSpec, run::Invalid> {
    let report = match value.report() {
        Some(rule) => Some(text_rule(rule)?),
        None => None,
    };
    let change = match value.change() {
        Some(rule) => Some(change_rule(rule)?),
        None => None,
    };
    let failure = match value.failure() {
        Some(rule) => Some(text_rule(rule)?),
        None => None,
    };
    let mut verdicts = List::with_capacity(value.verdicts().len());
    for rule in value.verdicts() {
        let mut kinds = List::with_capacity(rule.kinds().len());
        for kind in rule.kinds() {
            let item = run::outcome::ItemRule { kind: Box::from(kind.kind()), fields: field_rules(kind.fields())? };
            if kinds.push(item).is_err() {
                return Err(run::Invalid::TooLarge);
            }
        }
        let item = run::outcome::VerdictRule {
            name: Box::from(rule.label()),
            text_max: rule.text_max(),
            fields: field_rules(rule.fields())?,
            items: run::outcome::ItemSpec { min: rule.least(), max: rule.most(), kinds: kinds.into_boxed() },
        };
        if verdicts.push(item).is_err() {
            return Err(run::Invalid::TooLarge);
        }
    }
    Ok(run::outcome::OutcomeSpec { change, verdicts: verdicts.into_boxed(), report, failure })
}

fn text_rule(value: &wire::TextRule) -> Result<run::outcome::TextSpec, run::Invalid> {
    Ok(run::outcome::TextSpec { max: value.max(), fields: field_rules(value.fields())? })
}

fn change_rule(value: &wire::ChangeRule) -> Result<run::outcome::ChangeSpec, run::Invalid> {
    Ok(run::outcome::ChangeSpec { checks_must_pass: value.checks_must_pass(), fields: field_rules(value.fields())? })
}

fn field_rules(value: &List<wire::FieldRule>) -> Result<Box<[run::outcome::FieldRule]>, run::Invalid> {
    let mut fields = List::with_capacity(value.len());
    for field in value {
        let item = run::outcome::FieldRule { name: Box::from(field.name()), max: field.max() };
        if fields.push(item).is_err() {
            return Err(run::Invalid::TooLarge);
        }
    }
    Ok(fields.into_boxed())
}
