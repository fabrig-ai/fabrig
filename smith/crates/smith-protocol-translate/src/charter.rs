//! Charter values become bounded wire policy (smith/docs/design/protocol/charter.md, sections 2
//! and 5). This translation retains no state, addresses or credentials;
//! `encode_charter` resolves domain endpoint names without admitting policy.
use crate::{Endpoints, Error};
use alloc::boxed::Box;
use skein_lib::{List, Writer};
use smith_charter as wire;
use smith_domain::run;

/// Encode every declared charter form using the configured endpoint names.
#[expect(clippy::too_many_lines, reason = "total checked translation of the complete charter vocabulary")]
pub fn encode_charter(
    source: &run::Charter,
    limits: &wire::v4::Limits,
    endpoints: &Endpoints,
) -> Result<Box<[u8]>, Error> {
    let mut brief = List::with_capacity(count(source.brief.sections.len())?);
    for section in &source.brief.sections {
        if brief
            .push(
                match wire::Section::new(
                    limits,
                    wire::SectionParts { title: section.title.clone(), body: section.text.clone() },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::CharterBody(problem)),
                },
            )
            .is_err()
        {
            return Err(Error::Capacity);
        }
    }
    let mut host = List::with_capacity(count(source.grants.host_tools.len())?);
    for tool in &source.grants.host_tools {
        let effect = match tool.effect {
            run::HostEffect::Read => wire::Effect::Read,
            run::HostEffect::Write => wire::Effect::Write,
        };
        let item = match wire::HostTool::new(
            limits,
            wire::HostToolParts {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input: tool.schema.clone(),
                effect,
                deadline: tool.timeout,
            },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::CharterBody(problem)),
        };
        if host.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    let deliver = match &source.grants.deliver {
        Some(rule) => Some(change(rule, limits)?),
        None => None,
    };
    let tools = match wire::Tools::new(
        limits,
        wire::ToolsParts {
            families: match wire::Families::new(
                limits,
                wire::FamiliesParts {
                    skein_bools: [
                        source.grants.tools.inspect,
                        source.grants.tools.modify,
                        source.grants.tools.shell,
                        source.grants.agents,
                    ],
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::CharterBody(problem)),
            },
            wait: source.grants.wait,
            deliver,
            host,
        },
    ) {
        Ok(value) => value,
        Err(problem) => return Err(Error::CharterBody(problem)),
    };
    let mut verdicts = List::with_capacity(count(source.outcome.verdicts.len())?);
    for rule in &source.outcome.verdicts {
        let mut kinds = List::with_capacity(count(rule.items.kinds.len())?);
        for kind in &rule.items.kinds {
            let item = match wire::ItemKind::new(
                limits,
                wire::ItemKindParts { kind: kind.kind.clone(), fields: fields(&kind.fields, limits)? },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::CharterBody(problem)),
            };
            if kinds.push(item).is_err() {
                return Err(Error::Capacity);
            }
        }
        let item = match wire::VerdictRule::new(
            limits,
            wire::VerdictRuleParts {
                label: rule.name.clone(),
                text_max: rule.text_max,
                fields: fields(&rule.fields, limits)?,
                least: rule.items.min,
                most: rule.items.max,
                kinds,
            },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::CharterBody(problem)),
        };
        if verdicts.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    let contract = match wire::Contract::new(
        limits,
        wire::ContractParts {
            change: match &source.outcome.change {
                Some(rule) => Some(change(rule, limits)?),
                None => None,
            },
            report: match &source.outcome.report {
                Some(rule) => Some(text(rule, limits)?),
                None => None,
            },
            failure: match &source.outcome.failure {
                Some(rule) => Some(text(rule, limits)?),
                None => None,
            },
            verdicts,
        },
    ) {
        Ok(value) => value,
        Err(problem) => return Err(Error::CharterBody(problem)),
    };
    let conventions = match &source.conventions {
        Some(paths) => Some(
            match wire::Conventions::new(
                limits,
                wire::ConventionsParts { guide: paths.guide.clone(), checks: paths.checks.clone() },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::CharterBody(problem)),
            },
        ),
        None => None,
    };
    let mut models = List::with_capacity(count(source.models.len())?);
    for model in &source.models {
        if models.push(llm(model, limits, endpoints)?).is_err() {
            return Err(Error::Capacity);
        }
    }
    let record = match wire::Charter::new(
        limits,
        wire::CharterParts {
            mode: match source.mode {
                run::Mode::Finite => wire::Mode::Finite,
                run::Mode::Open => wire::Mode::Open,
            },
            instructions: source.instructions.clone(),
            brief,
            tools,
            contract,
            conventions,
            budget: match wire::Budget::new(
                limits,
                wire::BudgetParts {
                    turns: source.budget.turns,
                    spend: source.budget.spend,
                    time: source.budget.time,
                    reserve: match wire::Reserve::new(
                        limits,
                        wire::ReserveParts { turns: source.reserve.turns, time: source.reserve.time },
                    ) {
                        Ok(value) => value,
                        Err(problem) => return Err(Error::CharterBody(problem)),
                    },
                },
            ) {
                Ok(value) => value,
                Err(problem) => return Err(Error::CharterBody(problem)),
            },
            main: llm(&source.llm, limits, endpoints)?,
            models,
            waiting: source.waiting,
            resume: source.resume,
        },
    ) {
        Ok(value) => value,
        Err(problem) => return Err(Error::CharterBody(problem)),
    };
    let Ok(length) = usize::try_from(record.measure()) else {
        return Err(Error::Capacity);
    };
    let mut writer = Writer::new(length);
    match record.encode(&mut writer) {
        Ok(value) => value,
        Err(problem) => return Err(Error::Write(problem)),
    }
    Ok(writer.finish())
}

fn count(length: usize) -> Result<u32, Error> {
    match u32::try_from(length) {
        Ok(count) => Ok(count),
        Err(_) => Err(Error::Capacity),
    }
}

fn fields(source: &[run::outcome::FieldRule], limits: &wire::v4::Limits) -> Result<List<wire::FieldRule>, Error> {
    let mut fields = List::with_capacity(count(source.len())?);
    for field in source {
        let item = match wire::FieldRule::new(limits, wire::FieldRuleParts { name: field.name.clone(), max: field.max })
        {
            Ok(value) => value,
            Err(problem) => return Err(Error::CharterBody(problem)),
        };
        if fields.push(item).is_err() {
            return Err(Error::Capacity);
        }
    }
    Ok(fields)
}

fn change(source: &run::outcome::ChangeSpec, limits: &wire::v4::Limits) -> Result<wire::ChangeRule, Error> {
    Ok(
        match wire::ChangeRule::new(
            limits,
            wire::ChangeRuleParts {
                checks_must_pass: source.checks_must_pass,
                fields: fields(&source.fields, limits)?,
            },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::CharterBody(problem)),
        },
    )
}

fn text(source: &run::outcome::TextSpec, limits: &wire::v4::Limits) -> Result<wire::TextRule, Error> {
    Ok(
        match wire::TextRule::new(
            limits,
            wire::TextRuleParts { max: source.max, fields: fields(&source.fields, limits)? },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::CharterBody(problem)),
        },
    )
}

fn llm(source: &run::charter::Llm, limits: &wire::v4::Limits, endpoints: &Endpoints) -> Result<wire::Llm, Error> {
    let endpoint = match endpoints.name_of(source.endpoint.0, source.dialect) {
        Some(name) => Box::from(name),
        None => return Err(Error::Endpoints),
    };
    Ok(
        match wire::Llm::new(
            limits,
            wire::LlmParts {
                endpoint,
                account: source.account,
                model: source.model.clone(),
                effort: source.effort.clone(),
                window: source.window,
                output: source.output,
                prices: match wire::Prices::new(
                    limits,
                    wire::PricesParts {
                        input: source.prices.input,
                        cached: source.prices.cached,
                        cache_write: source.prices.cache_write,
                        output: source.prices.output,
                        unit: source.prices.unit,
                    },
                ) {
                    Ok(value) => value,
                    Err(problem) => return Err(Error::CharterBody(problem)),
                },
            },
        ) {
            Ok(value) => value,
            Err(problem) => return Err(Error::CharterBody(problem)),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Duration, Reader};

    #[test]
    #[expect(
        clippy::disallowed_macros,
        reason = "programming-model.md, section 10.2: a unit test reads its frozen golden"
    )]
    fn both_lifecycle_modes_cross_the_charter_boundary() {
        let mut endpoints = List::with_capacity(1);
        endpoints.push(crate::Endpoint { name: Box::default(), number: 17, dialect: 23 }).expect("one endpoint");
        let endpoints = Endpoints::new(endpoints);
        let golden = include_bytes!("../../smith-charter/golden/v4/record_charter_smallest.bin");
        let charter = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(golden)).expect("current golden");
        let mut writer = Writer::new(usize::try_from(charter.measure()).expect("bounded charter"));
        charter.encode(&mut writer).expect("measured charter");
        let source = writer.finish();
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
}
