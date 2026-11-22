//! Charter values become bounded wire policy (smith/docs/design/protocol/charter.md, sections 2
//! and 5). This translation retains no state, addresses or credentials;
//! `encode_charter` resolves domain endpoint names without admitting policy.
use crate::{Endpoints, Error};
use alloc::boxed::Box;
use skein_lib::List;
use smith_charter as wire;
use smith_charter::Record;
use smith_domain::run;

/// Encode every declared charter form using the configured endpoint names.
#[expect(clippy::too_many_lines, reason = "total checked translation of the complete charter vocabulary")]
pub fn encode_charter(
    source: &run::Charter,
    limits: &wire::v5::Limits,
    endpoints: &Endpoints,
) -> Result<Box<[u8]>, Error> {
    let mut brief = List::with_capacity(count(source.brief.sections.len())?);
    for section in &source.brief.sections {
        if brief.push(wire::Section { title: section.title.clone(), body: section.text.clone() }).is_err() {
            return Err(Error::Capacity);
        }
    }
    let mut host = List::with_capacity(count(source.grants.host_tools.len())?);
    for tool in &source.grants.host_tools {
        let effect = match tool.effect {
            run::HostEffect::Read => wire::Effect::Read,
            run::HostEffect::Write => wire::Effect::Write,
        };
        let item = wire::HostTool {
            reserve: tool.reserve,
            name: tool.name.clone(),
            description: tool.description.clone(),
            input: tool.schema.clone(),
            effect,
            deadline: tool.timeout,
        };
        if host.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    let deliver = match &source.grants.deliver {
        Some(rule) => Some(change(rule)?),
        None => None,
    };
    let tools = wire::Tools {
        families: wire::Families {
            inspect: source.grants.tools.inspect,
            modify: source.grants.tools.modify,
            shell: source.grants.tools.shell,
            agents: source.grants.agents,
        },
        wait: source.grants.wait,
        deliver,
        save: source.grants.save,
        host,
        contract: source.grants.contract,
    };
    let mut verdicts = List::with_capacity(count(source.outcome.verdicts.len())?);
    for rule in &source.outcome.verdicts {
        let mut kinds = List::with_capacity(count(rule.items.kinds.len())?);
        for kind in &rule.items.kinds {
            let item = wire::ItemKind { kind: kind.kind.clone(), fields: fields(&kind.fields)? };
            if kinds.push(item).is_err() {
                return Err(Error::Capacity);
            }
        }
        let item = wire::VerdictRule {
            label: rule.name.clone(),
            text_max: rule.text_max,
            fields: fields(&rule.fields)?,
            least: rule.items.min,
            most: rule.items.max,
            kinds,
        };
        if verdicts.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    let contract = wire::Contract {
        change: match &source.outcome.change {
            Some(rule) => Some(change(rule)?),
            None => None,
        },
        report: match &source.outcome.report {
            Some(rule) => Some(text(rule)?),
            None => None,
        },
        failure: match &source.outcome.failure {
            Some(rule) => Some(text(rule)?),
            None => None,
        },
        verdicts,
    };
    let conventions = match &source.conventions {
        Some(paths) => Some(wire::Conventions { guide: paths.guide.clone(), checks: paths.checks.clone() }),
        None => None,
    };
    let mut models = List::with_capacity(count(source.models.len())?);
    for model in &source.models {
        if models.push(llm(model, endpoints)?).is_err() {
            return Err(Error::Capacity);
        }
    }
    let record = wire::Charter {
        check_timeout: source.check_timeout,
        mode: match source.mode {
            run::Mode::Finite => wire::Mode::Finite,
            run::Mode::Open => wire::Mode::Open,
        },
        instructions: source.instructions.clone(),
        brief,
        tools,
        contract,
        conventions,
        budget: wire::Budget {
            turns: source.budget.turns,
            spend: source.budget.spend,
            time: source.budget.time,
            reserve: wire::Reserve { turns: source.reserve.turns, time: source.reserve.time },
        },
        main: llm(&source.llm, endpoints)?,
        models,
        waiting: source.waiting,
        resume: source.resume,
    };
    match record.encode(limits) {
        Ok(bytes) => Ok(bytes),
        Err(problem) => Err(Error::CharterBody(problem)),
    }
}

fn count(length: usize) -> Result<u32, Error> {
    match u32::try_from(length) {
        Ok(count) => Ok(count),
        Err(_) => Err(Error::Capacity),
    }
}

fn fields(source: &[run::outcome::FieldRule]) -> Result<List<wire::FieldRule>, Error> {
    let mut fields = List::with_capacity(count(source.len())?);
    for field in source {
        let item = wire::FieldRule { name: field.name.clone(), max: field.max };
        if fields.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    Ok(fields)
}

fn change(source: &run::outcome::ChangeSpec) -> Result<wire::ChangeRule, Error> {
    Ok(wire::ChangeRule { checks_must_pass: source.checks_must_pass, fields: fields(&source.fields)? })
}

fn text(source: &run::outcome::TextSpec) -> Result<wire::TextRule, Error> {
    Ok(wire::TextRule { max: source.max, fields: fields(&source.fields)? })
}

fn llm(source: &run::charter::Llm, endpoints: &Endpoints) -> Result<wire::Llm, Error> {
    let endpoint = match endpoints.name_of(source.endpoint.0, source.dialect) {
        Some(name) => Box::from(name),
        None => return Err(Error::Endpoints),
    };
    Ok(wire::Llm {
        endpoint,
        account: source.account,
        model: source.model.clone(),
        effort: source.effort.clone(),
        window: source.window,
        output: source.output,
        prices: wire::Prices {
            input: source.prices.input,
            cached: source.prices.cached,
            cache_write: source.prices.cache_write,
            output: source.prices.output,
            unit: source.prices.unit,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Duration, Reader};

    #[test]
    fn optional_check_bounds_cross_the_charter_boundary() {
        let mut endpoints = List::with_capacity(1);
        endpoints.push(crate::Endpoint { name: Box::default(), number: 17, dialect: 23 }).expect("one endpoint");
        let endpoints = Endpoints::new(endpoints);
        let source = &charter_fixture();
        let mut charter = crate::decode_charter(source, &wire::CEILINGS, &endpoints).expect("smallest charter");
        assert_eq!(charter.check_timeout, None);
        for bound in [None, Some(Duration::from_secs(1)), Some(Duration::from_secs(86_400))] {
            charter.check_timeout = bound;
            let encoded = encode_charter(&charter, &wire::CEILINGS, &endpoints).expect("bounded charter");
            assert_eq!(crate::decode_charter(&encoded, &wire::CEILINGS, &endpoints), Ok(charter));
            charter = crate::decode_charter(source, &wire::CEILINGS, &endpoints).expect("smallest charter");
        }
    }

    #[test]
    fn both_lifecycle_modes_cross_the_charter_boundary() {
        let mut endpoints = List::with_capacity(1);
        endpoints.push(crate::Endpoint { name: Box::default(), number: 17, dialect: 23 }).expect("one endpoint");
        let endpoints = Endpoints::new(endpoints);
        let fixed_value = &charter_fixture();
        let charter =
            wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(fixed_value)).expect("current fixed-rule value");
        let source = charter.encode(&wire::CEILINGS).expect("bounded charter");
        for mode in [run::Mode::Finite, run::Mode::Open] {
            let mut charter = crate::decode_charter(&source, &wire::CEILINGS, &endpoints).expect("known endpoint");
            charter.mode = mode;
            charter.grants.wait = true;
            charter.waiting = Duration::from_secs(30);
            let encoded = encode_charter(&charter, &wire::CEILINGS, &endpoints).expect("charter encodes");
            assert_eq!(crate::decode_charter(&encoded, &wire::CEILINGS, &endpoints), Ok(charter));
        }
        assert_eq!(crate::decode_charter(&[0, 3], &wire::CEILINGS, &endpoints), Err(run::Invalid::CharterVersion));
    }
    fn charter_fixture() -> Box<[u8]> {
        let values = skein_codegen::values(smith_charter::SCHEMA);
        for value in values.fixed.clone() {
            if value.name == "record_charter_smallest.bin" {
                return value.bytes.into_boxed_slice();
            }
        }
        panic!("smallest charter")
    }
}
