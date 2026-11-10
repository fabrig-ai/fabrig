//! A task's LLM choice and `me`'s models, by configured names (client/domain/contract/requests.md, 2, and client/domain/contract/documents.md, 1).
use super::{too_large, w};
use crate::Limits;
use jig_client_domain::contract as c;
use skein_lib::{List, bytes};

pub(crate) fn wire_choice(
    value: Option<&c::LlmChoice>,
    limits: &w::Limits,
) -> Result<Option<w::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(value.subagents.len());
    for item in &value.subagents {
        subagents.push(wire_model_effort(item, limits)?).expect("same capacity");
    }
    Ok(Some(w::LlmChoice::new(
        limits,
        w::LlmChoiceParts {
            account: value.account,
            main: wire_model_effort(&value.main, limits)?,
            subagents,
            later: match &value.later {
                Some(later) => Some(wire_later(later, limits)?),
                None => None,
            },
        },
    )?))
}
fn wire_later(value: &c::LlmLater, limits: &w::Limits) -> Result<w::LlmLater, w::Problem> {
    let mut subagents = List::with_capacity(value.subagents.len());
    for item in &value.subagents {
        subagents.push(wire_model_effort(item, limits)?).expect("same capacity");
    }
    w::LlmLater::new(limits, w::LlmLaterParts { main: wire_model_effort(&value.main, limits)?, subagents })
}
fn wire_model_effort(value: &c::ModelEffort, limits: &w::Limits) -> Result<w::ModelEffort, w::Problem> {
    w::ModelEffort::new(
        limits,
        w::ModelEffortParts {
            model: bytes::copy_of(value.model.bytes()),
            effort: match &value.effort {
                Some(effort) => Some(bytes::copy_of(effort.bytes())),
                None => None,
            },
        },
    )
}
pub(crate) fn choice(value: Option<&w::LlmChoice>, limits: &Limits) -> Result<Option<c::LlmChoice>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut subagents = List::with_capacity(value.subagents().len());
    for item in value.subagents() {
        subagents.push(model_effort(item, limits)?).expect("same capacity");
    }
    Ok(Some(c::LlmChoice {
        account: *value.account(),
        main: model_effort(value.main(), limits)?,
        subagents,
        later: match value.later() {
            Some(later) => Some(restore_later(later, limits)?),
            None => None,
        },
    }))
}
fn restore_later(value: &w::LlmLater, limits: &Limits) -> Result<c::LlmLater, w::Problem> {
    let mut subagents = List::with_capacity(value.subagents().len());
    for item in value.subagents() {
        subagents.push(model_effort(item, limits)?).expect("same capacity");
    }
    Ok(c::LlmLater { main: model_effort(value.main(), limits)?, subagents })
}
fn model_effort(value: &w::ModelEffort, limits: &Limits) -> Result<c::ModelEffort, w::Problem> {
    Ok(c::ModelEffort {
        model: name(value.model(), limits.receiving().model_effort_model)?,
        effort: match value.effort() {
            Some(effort) => Some(name(effort, limits.receiving().model_effort_effort_some)?),
            None => None,
        },
    })
}
pub(crate) fn models(values: &List<w::MeModel>, limits: &Limits) -> Result<List<c::MeModel>, w::Problem> {
    let mut models = List::with_capacity(values.len());
    for value in values {
        let prices = value.prices();
        if prices.unit() == 0
            || prices.input() == 0
            || prices.cached() == 0
            || *prices.cache_write() == Some(0)
            || prices.output() == 0
        {
            return Err(w::Problem { path: w::Path::MeModelPrices, reason: skein_codec::Reason::Malformed });
        }
        let description = match value.description() {
            Some(value) if value.is_empty() => {
                return Err(w::Problem { path: w::Path::MeModelDescription, reason: skein_codec::Reason::Malformed });
            }
            Some(value) => Some(name(value, limits.receiving().me_model_description_some)?),
            None => None,
        };
        let effort = match value.effort() {
            Some(effort) if value.efforts().as_slice().contains(effort) => {
                Some(name(effort, limits.receiving().me_model_effort_some)?)
            }
            None if value.efforts().is_empty() => None,
            Some(_) | None => {
                return Err(w::Problem { path: w::Path::MeModelEffort, reason: skein_codec::Reason::Malformed });
            }
        };
        let shown = match value.shown() {
            Some(shown) if shown.is_empty() => {
                return Err(w::Problem { path: w::Path::MeModelShown, reason: skein_codec::Reason::Malformed });
            }
            Some(shown) => Some(name(shown, limits.receiving().me_model_shown_some)?),
            None => None,
        };
        let mut efforts = List::with_capacity(value.efforts().len());
        for effort in value.efforts() {
            efforts.push(name(effort, limits.receiving().me_model_efforts_item)?).expect("same capacity");
        }
        models
            .push(c::MeModel {
                new_until: match value.new_until() {
                    Some(at) => Some(skein_lib::Wall::from_nanos(*at)),
                    None => None,
                },
                number: value.number(),
                source: match value.source() {
                    w::ModelSource::Stock => c::ModelSource::Stock,
                    w::ModelSource::List => c::ModelSource::List,
                    w::ModelSource::Person => c::ModelSource::Person,
                },
                estimated: value.estimated(),
                name: name(value.name(), limits.receiving().me_model_name)?,
                provider: *value.provider(),
                window: value.window(),
                output: value.output(),
                efforts,
                effort,
                shown,
                accounts: value.accounts().clone(),
                picker: value.picker(),
                prices: c::ModelPrices {
                    input: prices.input(),
                    cached: prices.cached(),
                    cache_write: *prices.cache_write(),
                    output: prices.output(),
                    unit: prices.unit(),
                },
                description,
            })
            .expect("same capacity");
    }
    Ok(models)
}
pub(crate) fn refusal(value: &w::RefusalLlm, limits: &Limits) -> Result<c::Refusal, w::Problem> {
    Ok(c::Refusal::Llm {
        model: name(value.model(), limits.receiving().refusal_llm_model)?,
        why: match value.why() {
            w::LlmWhy::Unconfigured => c::LlmWhy::Unconfigured,
            w::LlmWhy::Effort => c::LlmWhy::Effort,
            w::LlmWhy::Charter => c::LlmWhy::Charter,
            w::LlmWhy::Authority => c::LlmWhy::Authority,
            w::LlmWhy::Procedure => c::LlmWhy::Procedure,
            w::LlmWhy::Repeated => c::LlmWhy::Repeated,
        },
    })
}
fn name(value: &[u8], limit: u32) -> Result<c::Text, w::Problem> {
    match c::Text::new(bytes::copy_of(value), limit) {
        Ok(text) => Ok(text),
        Err(_) => Err(too_large()),
    }
}

pub(crate) fn defaults(values: &List<w::MeDefault>, limits: &Limits) -> Result<List<c::MeDefault>, w::Problem> {
    let mut defaults = List::with_capacity(values.len());
    for value in values {
        defaults
            .push(c::MeDefault {
                template: value.template(),
                choice: choice(Some(value.choice()), limits)?.expect("present choice"),
            })
            .expect("same capacity");
    }
    Ok(defaults)
}
