//! A request's LLM choice and a refused model, by index as the tasks child keeps them
//! (protocol/store/records.md, 1; domain/tasks/README.md, 3).
use super::{List, domain, wire};

/// Translate a request's LLM choice, if any.
pub(super) fn encode_llm(
    source: Option<&domain::LlmChoice>,
    limits: &wire::Limits,
) -> Result<Option<wire::LlmChoice>, wire::Problem> {
    let Some(domain::LlmChoice { account, main, subagents, later }) = source else {
        return Ok(None);
    };
    let mut chosen = List::with_capacity(limits.llm_choice_subagents);
    for subagent in subagents {
        if chosen.push(encode_model_effort(*subagent, limits)?).is_err() {
            return Err(wire::Problem { path: wire::Path::LlmChoiceSubagents, reason: skein_codec::Reason::Bound });
        }
    }
    let later = match later {
        None => None,
        Some(later) => {
            let mut chosen = List::with_capacity(limits.llm_later_subagents);
            for subagent in &later.subagents {
                if chosen.push(encode_model_effort(*subagent, limits)?).is_err() {
                    return Err(wire::Problem {
                        path: wire::Path::LlmLaterSubagents,
                        reason: skein_codec::Reason::Bound,
                    });
                }
            }
            Some(wire::LlmLater::new(
                limits,
                wire::LlmLaterParts { main: encode_model_effort(later.main, limits)?, subagents: chosen },
            )?)
        }
    };
    Ok(Some(wire::LlmChoice::new(
        limits,
        wire::LlmChoiceParts { account: *account, main: encode_model_effort(*main, limits)?, subagents: chosen, later },
    )?))
}
fn encode_model_effort(source: domain::ModelEffort, limits: &wire::Limits) -> Result<wire::ModelEffort, wire::Problem> {
    let domain::ModelEffort { model, effort } = source;
    wire::ModelEffort::new(limits, wire::ModelEffortParts { model, effort })
}
/// Read a request's LLM choice, if any; its decode bounds its sub-agents.
pub(super) fn decode_llm(source: Option<wire::LlmChoice>) -> Option<domain::LlmChoice> {
    let wire::LlmChoiceParts { account, main, subagents, later } = source?.into_parts();
    let mut chosen = List::with_capacity(subagents.len());
    for subagent in subagents.into_boxed() {
        chosen.push(decode_model_effort(subagent)).expect("same number of sub-agents");
    }
    let later = match later {
        None => None,
        Some(later) => {
            let wire::LlmLaterParts { main, subagents } = later.into_parts();
            let mut chosen = List::with_capacity(subagents.len());
            for subagent in subagents.into_boxed() {
                chosen.push(decode_model_effort(subagent)).expect("decoded later sub-agents bound");
            }
            Some(domain::LlmLater { main: decode_model_effort(main), subagents: chosen.into_boxed() })
        }
    };
    Some(domain::LlmChoice { later, account, main: decode_model_effort(main), subagents: chosen.into_boxed() })
}
fn decode_model_effort(source: wire::ModelEffort) -> domain::ModelEffort {
    let wire::ModelEffortParts { model, effort } = source.into_parts();
    domain::ModelEffort { model, effort }
}
pub(super) fn encode_why(value: domain::LlmWhy) -> wire::LlmWhy {
    match value {
        domain::LlmWhy::Unconfigured => wire::LlmWhy::Unconfigured,
        domain::LlmWhy::Effort => wire::LlmWhy::Effort,
        domain::LlmWhy::Charter => wire::LlmWhy::Charter,
        domain::LlmWhy::Authority => wire::LlmWhy::Authority,
        domain::LlmWhy::Procedure => wire::LlmWhy::Procedure,
        domain::LlmWhy::Repeated => wire::LlmWhy::Repeated,
    }
}
pub(super) fn decode_why(value: wire::LlmWhy) -> domain::LlmWhy {
    match value {
        wire::LlmWhy::Unconfigured => domain::LlmWhy::Unconfigured,
        wire::LlmWhy::Effort => domain::LlmWhy::Effort,
        wire::LlmWhy::Charter => domain::LlmWhy::Charter,
        wire::LlmWhy::Authority => domain::LlmWhy::Authority,
        wire::LlmWhy::Procedure => domain::LlmWhy::Procedure,
        wire::LlmWhy::Repeated => domain::LlmWhy::Repeated,
    }
}
