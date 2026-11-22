//! Wire projections of llm facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::{
    wire_account_use, wire_agent_shortfall, wire_document_text, wire_hello_refusal, wire_host_kind,
    wire_task_source_place, wire_task_source_unread,
};
pub(in crate::codec) fn wire_llm_choice(value: &c::LlmChoice, limits: &w::Limits) -> Result<w::LlmChoice, w::Problem> {
    Ok(w::LlmChoice {
        later: value
            .later
            .as_ref()
            .map(|later| {
                Ok(w::LlmLater {
                    main: wire_model_effort(&later.main, limits)?,
                    subagents: list(later.subagents.iter().map(|item| wire_model_effort(item, limits)))?,
                })
            })
            .transpose()?,
        account: value.account.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        main: wire_model_effort(&value.main, limits)?,
        subagents: list(value.subagents.iter().map(|item| wire_model_effort(item, limits)))?,
    })
}

pub(in crate::codec) fn wire_model_effort(
    value: &c::ModelEffort,
    _limits: &w::Limits,
) -> Result<w::ModelEffort, w::Problem> {
    Ok(w::ModelEffort {
        model: bytes::copy_of(value.model.bytes()),
        effort: value.effort.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))).transpose()?,
    })
}

pub(in crate::codec) fn wire_account_why(
    value: &c::AccountWhy,
    _limits: &w::Limits,
) -> Result<w::AccountWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountWhy::Unknown => w::AccountWhy::Unknown,
        c::AccountWhy::Provider => w::AccountWhy::Provider,
        c::AccountWhy::Uses => w::AccountWhy::Uses,
        c::AccountWhy::Limited => w::AccountWhy::Limited,
    })
}

pub(in crate::codec) fn wire_llm_why(value: &c::LlmWhy, _limits: &w::Limits) -> Result<w::LlmWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::LlmWhy::Unconfigured => w::LlmWhy::Unconfigured,
        c::LlmWhy::Effort => w::LlmWhy::Effort,
        c::LlmWhy::Charter => w::LlmWhy::Charter,
        c::LlmWhy::Authority => w::LlmWhy::Authority,
        c::LlmWhy::Procedure => w::LlmWhy::Procedure,
        c::LlmWhy::Repeated => w::LlmWhy::Repeated,
    })
}

pub(in crate::codec) fn wire_task_run_wait(
    value: &c::TaskRunWait,
    limits: &w::Limits,
) -> Result<w::TaskRunWait, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskRunWait::Account { account, source, why } => w::TaskRunWait::Account(w::TaskWaitAccount {
            account: (account).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            source: (source).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            why: wire_task_wait_why(why, limits)?,
        }),
        c::TaskRunWait::Credential => w::TaskRunWait::Credential,
        c::TaskRunWait::Host => w::TaskRunWait::Host,
        c::TaskRunWait::Worker { worker, until } => {
            w::TaskRunWait::Worker(w::TaskWaitWorker { worker: *worker, until: until.as_nanos() })
        }
        c::TaskRunWait::Refused(value) => {
            w::TaskRunWait::Refused(w::TaskRunWaitRefused { value: wire_task_wait_refused(value, limits)? })
        }
    })
}

pub(in crate::codec) fn wire_task_wait_why(
    value: &c::TaskWaitWhy,
    limits: &w::Limits,
) -> Result<w::TaskWaitWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskWaitWhy::Source(value) => {
            w::TaskWaitWhy::Source(w::TaskWaitSource { state: wire_task_source_state(value, limits)? })
        }
        c::TaskWaitWhy::Untooled { worker } => w::TaskWaitWhy::Untooled(w::TaskWaitUntooled { worker: *worker }),
        c::TaskWaitWhy::Refreshing => w::TaskWaitWhy::Refreshing,
        c::TaskWaitWhy::Host => w::TaskWaitWhy::Host,
        c::TaskWaitWhy::Empty => w::TaskWaitWhy::Empty,
        c::TaskWaitWhy::Invalid(value) => {
            w::TaskWaitWhy::Invalid(w::TaskWaitInvalid { why: wire_account_why(value, limits)? })
        }
    })
}

pub(in crate::codec) fn wire_task_source_state(
    value: &c::TaskSourceState,
    limits: &w::Limits,
) -> Result<w::TaskSourceState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskSourceState::Asking => w::TaskSourceState::Asking,
        c::TaskSourceState::Granted => w::TaskSourceState::Granted,
        c::TaskSourceState::Later { until } => w::TaskSourceState::Later(w::TaskSourceStateUntil {
            until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        }),
        c::TaskSourceState::Limited { until } => w::TaskSourceState::Limited(w::TaskSourceStateUntil {
            until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        }),
        c::TaskSourceState::Refused => w::TaskSourceState::Refused,
        c::TaskSourceState::NeedsSignIn => w::TaskSourceState::NeedsSignIn,
        c::TaskSourceState::Misconfigured => w::TaskSourceState::Misconfigured,
        c::TaskSourceState::Unloaded { at, why } => w::TaskSourceState::Unloaded(w::TaskSourceStateUnloaded {
            at: wire_task_source_place(at, limits)?,
            why: wire_task_source_unread(why, limits)?,
        }),
        c::TaskSourceState::Cancelled => w::TaskSourceState::Cancelled,
        c::TaskSourceState::Closing => w::TaskSourceState::Closing,
        c::TaskSourceState::Moving => w::TaskSourceState::Moving,
    })
}

pub(in crate::codec) fn wire_task_wait_refused(
    value: &c::TaskWaitRefused,
    limits: &w::Limits,
) -> Result<w::TaskWaitRefused, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskWaitRefused::Hello { kind, at, why, figures } => w::TaskWaitRefused::Hello(w::TaskWaitHello {
            kind: wire_host_kind(kind, limits)?,
            at: (at).as_nanos(),
            why: wire_hello_refusal(why, limits)?,
            figures: wire_shortfall_figures(figures.as_ref(), limits)?,
        }),
        c::TaskWaitRefused::Bounds { kind, why, figures } => w::TaskWaitRefused::Bounds(w::TaskWaitBounds {
            kind: wire_host_kind(kind, limits)?,
            why: match why {
                Some(why) => Some(wire_agent_shortfall(why, limits)?),
                None => None,
            },
            figures: wire_shortfall_figures(figures.as_ref(), limits)?,
        }),
    })
}

pub(in crate::codec) fn wire_me_model(value: &c::MeModel, limits: &w::Limits) -> Result<w::MeModel, w::Problem> {
    Ok(w::MeModel {
        new_until: value.new_until.map(skein_lib::Wall::as_nanos),
        number: value.number,
        source: match value.source {
            c::ModelSource::Stock => w::ModelSource::Stock,
            c::ModelSource::List => w::ModelSource::List,
            c::ModelSource::Person => w::ModelSource::Person,
        },
        estimated: value.estimated,
        name: bytes::copy_of(value.name.bytes()),
        provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        window: value.window,
        output: value.output,
        efforts: list(value.efforts.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
        effort: value.effort.as_ref().map(|text| bytes::copy_of(text.bytes())),
        shown: value.shown.as_ref().map(|text| bytes::copy_of(text.bytes())),
        accounts: value.accounts.clone(),
        picker: value.picker,
        prices: wire_me_prices(&value.prices, limits)?,
        description: value
            .description
            .as_ref()
            .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
            .transpose()?,
    })
}

pub(in crate::codec) fn wire_me_prices(value: &c::ModelPrices, _limits: &w::Limits) -> Result<w::MePrices, w::Problem> {
    Ok(w::MePrices {
        input: value.input,
        cached: value.cached,
        cache_write: value.cache_write.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        output: value.output,
        unit: value.unit,
    })
}

pub(in crate::codec) fn wire_provider_offer(
    value: &c::ProviderOffer,
    limits: &w::Limits,
) -> Result<w::ProviderOffer, w::Problem> {
    Ok(w::ProviderOffer {
        sign_in: value.sign_in,
        key: value.key,
        programs_cut: value.programs_cut,
        none: value.none,
        signed_in_plan: value.signed_in_plan,
        provider: value.provider,
        name: wire_document_text(&value.name, limits)?,
        programs: list(value.programs.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
        manage_usage: value.manage_usage.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
        key_bytes: value.key_bytes,
    })
}

pub(in crate::codec) fn wire_payer_readiness(
    value: &c::PayerReadiness,
    limits: &w::Limits,
) -> Result<w::PayerReadiness, w::Problem> {
    Ok(w::PayerReadiness {
        provider: value.provider,
        payer: wire_account_use(&value.payer, limits)?,
        readiness: wire_readiness(&value.readiness, limits)?,
        settled: value.settled.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
    })
}

pub(in crate::codec) fn wire_readiness(value: &c::Readiness, _limits: &w::Limits) -> Result<w::Readiness, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Readiness::Usable => w::Readiness::Usable,
        c::Readiness::Unavailable => w::Readiness::Unavailable,
        c::Readiness::Unknown => w::Readiness::Unknown,
    })
}

fn wire_shortfall_figures(
    value: Option<&c::Figures>,
    _limits: &w::Limits,
) -> Result<Option<w::ShortfallFigures>, w::Problem> {
    match value {
        Some(value) => Ok(Some(w::ShortfallFigures { required: value.required, offered: value.offered })),
        None => Ok(None),
    }
}
