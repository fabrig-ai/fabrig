//! Write the Smith charter codec from the shared charter
//! (domain/hosts/charters.md; smith's `protocol/charter.md`, section 5).
//! The configured endpoint table supplies names; the charter keeps numbers.

use alloc::boxed::Box;
use skein_lib::List;
use smith_charter::Record;
use smith_charter::v5 as wire;
use smith_domain_run as run;

/// One configured endpoint name and its domain-visible identities.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EndpointName {
    pub number: u32,
    pub dialect: u32,
    pub name: Box<[u8]>,
}

fn count(length: usize) -> Option<u32> {
    u32::try_from(length).ok()
}

fn endpoint(value: &run::charter::Llm, names: &[EndpointName]) -> Option<Box<[u8]>> {
    for name in names {
        if name.number == value.endpoint.0 && name.dialect == value.dialect {
            return Some(name.name.clone());
        }
    }
    None
}

fn llm(value: run::charter::Llm, names: &[EndpointName]) -> Option<wire::Llm> {
    let name = endpoint(&value, names)?;
    let prices = wire::Prices {
        input: value.prices.input,
        cached: value.prices.cached,
        cache_write: value.prices.cache_write,
        output: value.prices.output,
        unit: value.prices.unit,
    };
    Some(wire::Llm {
        endpoint: name,
        account: value.account,
        model: value.model,
        effort: value.effort,
        window: value.window,
        output: value.output,
        prices,
    })
}

fn fields(source: Box<[run::outcome::FieldRule]>) -> Option<List<wire::FieldRule>> {
    let mut out = List::with_capacity(count(source.len())?);
    for field in source {
        out.push(wire::FieldRule { name: field.name, max: field.max }).ok()?;
    }
    Some(out)
}

fn text(source: run::outcome::TextSpec) -> Option<wire::TextRule> {
    Some(wire::TextRule { max: source.max, fields: fields(source.fields)? })
}

fn change(source: run::outcome::ChangeSpec) -> Option<wire::ChangeRule> {
    Some(wire::ChangeRule { checks_must_pass: source.checks_must_pass, fields: fields(source.fields)? })
}

fn verdict(source: run::outcome::VerdictRule) -> Option<wire::VerdictRule> {
    let mut kinds = List::with_capacity(count(source.items.kinds.len())?);
    for kind in source.items.kinds {
        kinds.push(wire::ItemKind { kind: kind.kind, fields: fields(kind.fields)? }).ok()?;
    }
    Some(wire::VerdictRule {
        label: source.name,
        text_max: source.text_max,
        fields: fields(source.fields)?,
        least: source.items.min,
        most: source.items.max,
        kinds,
    })
}

fn contract(source: run::outcome::OutcomeSpec) -> Option<wire::Contract> {
    let mut verdicts = List::with_capacity(count(source.verdicts.len())?);
    for item in source.verdicts {
        verdicts.push(verdict(item)?).ok()?;
    }
    Some(wire::Contract {
        report: match source.report {
            Some(rule) => Some(text(rule)?),
            None => None,
        },
        verdicts,
        change: match source.change {
            Some(rule) => Some(change(rule)?),
            None => None,
        },
        failure: match source.failure {
            Some(rule) => Some(text(rule)?),
            None => None,
        },
    })
}

fn tools(source: run::charter::Grants) -> Option<wire::Tools> {
    let families = wire::Families {
        inspect: source.tools.inspect,
        modify: source.tools.modify,
        shell: source.tools.shell,
        agents: source.agents,
    };
    let mut host = List::with_capacity(count(source.host_tools.len())?);
    for tool in source.host_tools {
        let effect = match tool.effect {
            run::HostEffect::Read => wire::Effect::Read,
            run::HostEffect::Write => wire::Effect::Write,
        };
        host.push(wire::HostTool {
            reserve: tool.reserve,
            name: tool.name,
            description: tool.description,
            input: tool.schema,
            effect,
            deadline: tool.timeout,
        })
        .ok()?;
    }
    Some(wire::Tools {
        families,
        wait: source.wait,
        save: source.save,
        deliver: match source.deliver {
            Some(rule) => Some(change(rule)?),
            None => None,
        },
        host,
        contract: source.contract,
    })
}

/// Encode the complete charter for worker transport or an inline start.
/// The endpoint identities must match their configured Smith wire names.
#[must_use]
pub fn encode(source: run::Charter, names: &[EndpointName], limits: &wire::Limits) -> Option<Box<[u8]>> {
    let mut brief = List::with_capacity(count(source.brief.sections.len())?);
    for section in source.brief.sections {
        brief.push(wire::Section { title: section.title, body: section.text }).ok()?;
    }
    let mut models = List::with_capacity(count(source.models.len())?);
    for candidate in source.models {
        models.push(llm(candidate, names)?).ok()?;
    }
    let conventions = match source.conventions {
        Some(paths) => Some(wire::Conventions { guide: paths.guide, checks: paths.checks }),
        None => None,
    };
    let reserve = wire::Reserve { turns: source.reserve.turns, time: source.reserve.time };
    let budget =
        wire::Budget { turns: source.budget.turns, spend: source.budget.spend, time: source.budget.time, reserve };
    let charter = wire::Charter {
        check_timeout: None,
        mode: match source.mode {
            run::Mode::Finite => wire::Mode::Finite,
            run::Mode::Open => wire::Mode::Open,
        },
        instructions: source.instructions,
        brief,
        tools: tools(source.grants)?,
        contract: contract(source.outcome)?,
        conventions,
        budget,
        main: llm(source.llm, names)?,
        models,
        waiting: source.waiting,
        resume: source.resume,
    };
    charter.encode(limits).ok()
}
