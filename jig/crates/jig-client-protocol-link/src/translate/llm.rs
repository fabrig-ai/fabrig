//! A task's LLM choice and `me`'s models, by configured names (client/domain/contract/requests.md, 2, and client/domain/contract/documents.md, 1).
use super::{too_large, w};
use crate::Limits;
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use skein_lib::{List, bytes};

pub(crate) fn wire_choice(value: Option<&c::LlmChoice>) -> Option<w::LlmChoice> {
    match value {
        Some(value) => Some(client::llm_choice_to_record(value)),
        None => None,
    }
}
pub(crate) fn model_effort_to_record(value: &c::ModelEffort) -> w::ModelEffort {
    w::ModelEffort {
        model: bytes::copy_of(value.model.bytes()),
        effort: match &value.effort {
            Some(effort) => Some(bytes::copy_of(effort.bytes())),
            None => None,
        },
    }
}
pub(crate) fn choice(value: Option<&w::LlmChoice>, limits: &Limits) -> Result<Option<c::LlmChoice>, w::Problem> {
    match value {
        Some(value) => Ok(Some(client::llm_choice_to_domain(value.clone(), limits)?)),
        None => Ok(None),
    }
}
pub(crate) fn model_effort_to_domain(value: w::ModelEffort, limits: &Limits) -> Result<c::ModelEffort, w::Problem> {
    Ok(c::ModelEffort {
        model: checked_name(value.model, limits.receiving().model_effort_model)?,
        effort: match value.effort {
            Some(effort) => Some(checked_name(effort, limits.receiving().model_effort_effort_some)?),
            None => None,
        },
    })
}
pub(crate) fn models(values: &List<w::MeModel>, limits: &Limits) -> Result<List<c::MeModel>, w::Problem> {
    let mut models = List::with_capacity(values.len());
    for value in values {
        let prices = &value.prices;
        if prices.unit == 0
            || prices.input == 0
            || prices.cached == 0
            || prices.cache_write == Some(0)
            || prices.output == 0
        {
            return Err(w::Problem { path: w::paths::ME_MODEL_PRICES, reason: skein_codec::Reason::Malformed });
        }
        let description = match &value.description {
            Some(value) if value.is_empty() => {
                return Err(w::Problem {
                    path: w::paths::ME_MODEL_DESCRIPTION,
                    reason: skein_codec::Reason::Malformed,
                });
            }
            Some(value) => Some(name(value, limits.receiving().me_model_description_some)?),
            None => None,
        };
        let effort = match &value.effort {
            Some(effort) if value.efforts.as_slice().contains(effort) => {
                Some(name(effort, limits.receiving().me_model_effort_some)?)
            }
            None if value.efforts.is_empty() => None,
            Some(_) | None => {
                return Err(w::Problem { path: w::paths::ME_MODEL_EFFORT, reason: skein_codec::Reason::Malformed });
            }
        };
        let shown = match &value.shown {
            Some(shown) if shown.is_empty() => {
                return Err(w::Problem { path: w::paths::ME_MODEL_SHOWN, reason: skein_codec::Reason::Malformed });
            }
            Some(shown) => Some(name(shown, limits.receiving().me_model_shown_some)?),
            None => None,
        };
        let mut efforts = List::with_capacity(value.efforts.len());
        for effort in &value.efforts {
            efforts.push(name(effort, limits.receiving().me_model_efforts_item)?).expect("same capacity");
        }
        models
            .push(c::MeModel {
                new_until: match &value.new_until {
                    Some(at) => Some(skein_lib::Wall::from_nanos(*at)),
                    None => None,
                },
                number: value.number,
                source: client::model_source_to_domain(value.source.clone()),
                estimated: value.estimated,
                name: name(&value.name, limits.receiving().me_model_name)?,
                provider: value.provider,
                window: value.window,
                output: value.output,
                efforts,
                effort,
                shown,
                accounts: value.accounts.clone(),
                picker: value.picker,
                prices: client::me_prices_to_domain(prices.clone()),
                description,
            })
            .expect("same capacity");
    }
    Ok(models)
}
pub(crate) fn refusal(value: &w::RefusalLlm, limits: &Limits) -> Result<c::Refusal, w::Problem> {
    Ok(c::Refusal::Llm {
        model: name(&value.model, limits.receiving().refusal_llm_model)?,
        why: client::llm_why_to_domain(value.why.clone()),
    })
}
fn name(value: &[u8], limit: u32) -> Result<c::Text, w::Problem> {
    checked_name(bytes::copy_of(value), limit)
}
fn checked_name(value: Box<[u8]>, limit: u32) -> Result<c::Text, w::Problem> {
    match c::Text::new(value, limit) {
        Ok(text) => Ok(text),
        Err(_) => Err(too_large()),
    }
}

pub(crate) fn defaults(values: &List<w::MeDefault>, limits: &Limits) -> Result<List<c::MeDefault>, w::Problem> {
    let mut defaults = List::with_capacity(values.len());
    for value in values {
        defaults.push(client::me_default_to_domain(value.clone(), limits)?).expect("same capacity");
    }
    Ok(defaults)
}
