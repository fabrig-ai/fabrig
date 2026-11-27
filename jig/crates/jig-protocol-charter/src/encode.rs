//! Write the Smith charter codec from the shared charter
//! (domain/hosts/charters.md; smith's `protocol/charter.md`, section 5).
//! The configured endpoint table supplies names; the charter keeps numbers.

use crate::generated::charter as mapped;
use alloc::boxed::Box;
use skein_codec::Reason;
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
    Some(wire::Llm {
        endpoint: name,
        account: value.account,
        model: value.model,
        effort: value.effort,
        speed: match value.speed {
            Some(speed) => {
                count(speed.also.len())?;
                Some(wire::Speed {
                    name: speed.name,
                    also: List::from_boxed(speed.also),
                    prices: mapped::prices_to_record(&speed.prices),
                })
            }
            None => None,
        },
        window: value.window,
        output: value.output,
        prices: mapped::prices_to_record(&value.prices),
    })
}

pub(super) fn verdict_rule_to_record(source: &run::outcome::VerdictRule) -> Result<wire::VerdictRule, wire::Problem> {
    let kind_count =
        u32::try_from(source.items.kinds.len()).or(Err(wire::paths::VERDICT_RULE_KINDS.problem(Reason::Bound)))?;
    let mut kinds = List::with_capacity(kind_count);
    for kind in &source.items.kinds {
        assert!(kinds.push(mapped::item_kind_to_record(kind)?).is_ok(), "one output per item kind");
    }
    let count = u32::try_from(source.fields.len()).or(Err(wire::paths::VERDICT_RULE_FIELDS.problem(Reason::Bound)))?;
    let mut fields = List::with_capacity(count);
    for field in &source.fields {
        assert!(fields.push(mapped::field_rule_to_record(field)).is_ok(), "one output per field rule");
    }
    Ok(wire::VerdictRule {
        label: source.name.clone(),
        text_max: source.text_max,
        fields,
        least: source.items.min,
        most: source.items.max,
        kinds,
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
        host.push(mapped::host_tool_to_record(&tool)).ok()?;
    }
    Some(wire::Tools {
        families,
        wait: source.wait,
        save: source.save,
        deliver: match source.deliver {
            Some(rule) => Some(mapped::change_rule_to_record(&rule).ok()?),
            None => None,
        },
        host,
        contract: source.contract,
    })
}

/// Encode the complete charter for worker transport or an inline start.
/// The endpoint identities must match their configured Smith wire names.
#[must_use]
#[expect(
    clippy::manual_map,
    reason = "an Option is mapped without a closure in step code (programming-model.md, 10.3)"
)]
pub fn encode(source: run::Charter, names: &[EndpointName], limits: &wire::Limits) -> Option<Box<[u8]>> {
    let mut brief = List::with_capacity(count(source.brief.sections.len())?);
    for section in source.brief.sections {
        brief.push(mapped::section_to_record(&section)).ok()?;
    }
    let mut models = List::with_capacity(count(source.models.len())?);
    for candidate in source.models {
        models.push(llm(candidate, names)?).ok()?;
    }
    let conventions = match source.conventions {
        Some(paths) => Some(mapped::conventions_to_record(&paths)),
        None => None,
    };
    let reserve = mapped::reserve_to_record(&source.reserve);
    let budget =
        wire::Budget { turns: source.budget.turns, spend: source.budget.spend, time: source.budget.time, reserve };
    let charter = wire::Charter {
        check_timeout: None,
        mode: mapped::mode_to_record(&source.mode),
        instructions: source.instructions,
        brief,
        tools: tools(source.grants)?,
        contract: mapped::contract_to_record(&source.outcome).ok()?,
        conventions,
        budget,
        main: llm(source.llm, names)?,
        models,
        waiting: source.waiting,
        resume: source.resume,
    };
    charter.encode(limits).ok()
}
