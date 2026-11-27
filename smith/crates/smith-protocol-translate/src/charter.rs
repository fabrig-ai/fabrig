//! Charter values become bounded wire policy (smith/docs/design/protocol/charter.md, sections 2
//! and 5). This translation retains no state, addresses or credentials;
//! `encode_charter` resolves domain endpoint names without admitting policy.
use crate::generated::charter as mapped;
use crate::{Endpoints, Error};
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::List;
use smith_charter as wire;
use smith_charter::Record;
use smith_domain::run;

/// Encode every declared charter form using the configured endpoint names.
pub fn encode_charter(
    source: &run::Charter,
    limits: &wire::v5::Limits,
    endpoints: &Endpoints,
) -> Result<Box<[u8]>, Error> {
    let mut brief = List::with_capacity(count(source.brief.sections.len())?);
    for section in &source.brief.sections {
        if brief.push(mapped::section_to_record(section)).is_err() {
            return Err(Error::Capacity);
        }
    }
    let mut host = List::with_capacity(count(source.grants.host_tools.len())?);
    for tool in &source.grants.host_tools {
        if host.push(mapped::host_tool_to_record(tool)).is_err() {
            return Err(Error::Capacity);
        }
    }
    let deliver = match &source.grants.deliver {
        Some(rule) => Some(mapped::change_rule_to_record(rule).or(Err(Error::Capacity))?),
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
    let contract = mapped::contract_to_record(&source.outcome).or(Err(Error::Capacity))?;
    let conventions = match &source.conventions {
        Some(paths) => Some(mapped::conventions_to_record(paths)),
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
        mode: mapped::mode_to_record(&source.mode),
        instructions: source.instructions.clone(),
        brief,
        tools,
        contract,
        conventions,
        budget: wire::Budget {
            turns: source.budget.turns,
            spend: source.budget.spend,
            time: source.budget.time,
            reserve: mapped::reserve_to_record(&source.reserve),
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

pub(super) fn verdict_rule_to_record(
    source: &run::outcome::VerdictRule,
) -> Result<wire::VerdictRule, wire::v5::Problem> {
    let kind_count =
        u32::try_from(source.items.kinds.len()).or(Err(wire::v5::paths::VERDICT_RULE_KINDS.problem(Reason::Bound)))?;
    let mut kinds = List::with_capacity(kind_count);
    for kind in &source.items.kinds {
        assert!(kinds.push(mapped::item_kind_to_record(kind)?).is_ok(), "one output per item kind");
    }
    let count =
        u32::try_from(source.fields.len()).or(Err(wire::v5::paths::VERDICT_RULE_FIELDS.problem(Reason::Bound)))?;
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

pub(super) fn verdict_rule_to_domain(source: wire::VerdictRule) -> run::outcome::VerdictRule {
    let mut fields = List::with_capacity(source.fields.len());
    for field in source.fields.into_boxed() {
        assert!(fields.push(mapped::field_rule_to_domain(field)).is_ok(), "one output per field rule");
    }
    let mut kinds = List::with_capacity(source.kinds.len());
    for kind in source.kinds.into_boxed() {
        assert!(kinds.push(mapped::item_kind_to_domain(kind)).is_ok(), "one output per item kind");
    }
    run::outcome::VerdictRule {
        name: source.label,
        text_max: source.text_max,
        fields: fields.into_boxed(),
        items: run::outcome::ItemSpec { min: source.least, max: source.most, kinds: kinds.into_boxed() },
    }
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
        prices: mapped::prices_to_record(&source.prices),
        speed: match &source.speed {
            Some(speed) => Some(mapped::speed_to_record(speed).or(Err(Error::Capacity))?),
            None => None,
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
    #[test]
    fn optional_main_and_child_speeds_cross_the_charter_boundary() {
        let mut endpoints = List::with_capacity(1);
        endpoints.push(crate::Endpoint { name: Box::default(), number: 17, dialect: 23 }).expect("one endpoint");
        let endpoints = Endpoints::new(endpoints);
        let source = charter_fixture();
        let mut charter = crate::decode_charter(&source, &wire::CEILINGS, &endpoints).expect("charter without speed");
        assert_eq!(charter.llm.speed, None);
        let mut child = charter.llm.clone();
        child.model = b"child".as_slice().into();
        child.speed = Some(run::charter::Speed {
            name: b"fast".as_slice().into(),
            also: Box::new([b"priority".as_slice().into()]),
            prices: run::Prices { input: 3, cached: 2, cache_write: Some(4), output: 9, unit: 100 },
        });
        charter.llm.speed = Some(run::charter::Speed {
            name: b"ultrafast".as_slice().into(),
            also: Box::default(),
            prices: run::Prices { input: 7, cached: 5, cache_write: None, output: 11, unit: 1000 },
        });
        charter.models = Box::new([child]);
        let encoded = encode_charter(&charter, &wire::CEILINGS, &endpoints).expect("bounded charter");
        assert_eq!(crate::decode_charter(&encoded, &wire::CEILINGS, &endpoints), Ok(charter));
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
