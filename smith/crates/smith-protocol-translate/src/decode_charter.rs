//! Charter policy and endpoint resolution (smith/docs/design/protocol/charter.md, sections 2, 5 and 7).
use crate::Endpoints;
use crate::generated::charter as mapped;
use skein_lib::{List, Reader};
use smith_charter as wire;
use smith_charter::Record;
use smith_domain::run;

/// Decode one charter and resolve all LLM endpoint names before admission.
pub fn decode_charter(
    bytes: &[u8],
    limits: &wire::v5::Limits,
    endpoints: &Endpoints,
) -> Result<run::Charter, run::Invalid> {
    if bytes.len() < 2 {
        return Err(run::Invalid::MalformedCharter);
    }
    if bytes.get(..2) != Some(wire::wire::VERSION.to_be_bytes().as_slice()) {
        return Err(run::Invalid::CharterVersion);
    }
    let Ok(charter) = wire::Charter::decode(limits, &mut Reader::new(bytes)) else {
        return Err(run::Invalid::MalformedCharter);
    };
    translate_charter(charter, endpoints)
}

fn translate_charter(charter: wire::Charter, endpoints: &Endpoints) -> Result<run::Charter, run::Invalid> {
    let mut sections = List::with_capacity(charter.brief.len());
    for section in charter.brief.into_boxed() {
        assert!(sections.push(mapped::section_to_domain(section)).is_ok(), "one output per section");
    }

    let tools = charter.tools;
    let families = tools.families;
    let mut host_tools = List::with_capacity(tools.host.len());
    for tool in tools.host.into_boxed() {
        assert!(host_tools.push(mapped::host_tool_to_domain(tool)).is_ok(), "one output per host tool");
    }

    let deliver = match tools.deliver {
        Some(rule) => Some(mapped::change_rule_to_domain(rule)),
        None => None,
    };
    let conventions = match charter.conventions {
        Some(paths) => Some(mapped::conventions_to_domain(paths)),
        None => None,
    };
    let mut models = List::with_capacity(charter.models.len());
    for model in charter.models.into_boxed() {
        assert!(models.push(llm(model, endpoints)?).is_ok(), "one output per model");
    }
    let budget = charter.budget;
    Ok(run::Charter {
        check_timeout: charter.check_timeout,
        mode: mapped::mode_to_domain(charter.mode),
        resume: charter.resume,
        waiting: charter.waiting,
        instructions: charter.instructions,
        brief: run::Brief { sections: sections.into_boxed() },
        conventions,
        grants: run::charter::Grants {
            contract: tools.contract,
            wait: tools.wait,
            deliver,
            save: tools.save,
            tools: run::charter::Tools { inspect: families.inspect, modify: families.modify, shell: families.shell },
            agents: families.agents,
            host_tools: host_tools.into_boxed(),
        },
        outcome: mapped::contract_to_domain(charter.contract),
        budget: run::Budget { turns: budget.turns, spend: budget.spend, time: budget.time },
        reserve: mapped::reserve_to_domain(budget.reserve),
        llm: llm(charter.main, endpoints)?,
        models: models.into_boxed(),
    })
}

fn llm(value: wire::Llm, endpoints: &Endpoints) -> Result<run::charter::Llm, run::Invalid> {
    let Some(endpoint) = endpoints.resolve(&value.endpoint) else {
        return Err(run::Invalid::Endpoint);
    };
    Ok(run::charter::Llm {
        prices: mapped::prices_to_domain(value.prices),
        dialect: endpoint.dialect,
        account: value.account,
        endpoint: run::charter::Endpoint(endpoint.number),
        model: value.model,
        effort: value.effort,
        window: value.window,
        output: value.output,
    })
}
