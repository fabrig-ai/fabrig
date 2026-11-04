//! Wire projections of llm facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, w};
use super::{
    wire_account_use, wire_document_text, wire_hello_refusal, wire_host_kind, wire_task_source_place,
    wire_task_source_unread,
};
pub(in crate::codec) fn wire_llm_choice(value: &c::LlmChoice, limits: &w::Limits) -> Result<w::LlmChoice, w::Problem> {
    w::LlmChoice::new(
        limits,
        w::LlmChoiceParts {
            account: value.account.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            main: wire_model_effort(&value.main, limits)?,
            subagents: list(value.subagents.iter().map(|item| wire_model_effort(item, limits)))?,
        },
    )
}

pub(in crate::codec) fn wire_model_effort(
    value: &c::ModelEffort,
    limits: &w::Limits,
) -> Result<w::ModelEffort, w::Problem> {
    w::ModelEffort::new(
        limits,
        w::ModelEffortParts {
            model: bytes::copy_of(value.model.bytes()),
            effort: value
                .effort
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
        },
    )
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
        c::TaskRunWait::Account { account, source, why } => w::TaskRunWait::Account(w::TaskWaitAccount::new(
            limits,
            w::TaskWaitAccountParts {
                account: (account).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                source: (source).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                why: wire_task_wait_why(why, limits)?,
            },
        )?),
        c::TaskRunWait::Credential => w::TaskRunWait::Credential,
        c::TaskRunWait::Host => w::TaskRunWait::Host,
        c::TaskRunWait::Refused(value) => w::TaskRunWait::Refused(w::TaskRunWaitRefused::new(
            limits,
            w::TaskRunWaitRefusedParts { value: wire_task_wait_refused(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_task_wait_why(
    value: &c::TaskWaitWhy,
    limits: &w::Limits,
) -> Result<w::TaskWaitWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskWaitWhy::Source(value) => w::TaskWaitWhy::Source(w::TaskWaitSource::new(
            limits,
            w::TaskWaitSourceParts { state: wire_task_source_state(value, limits)? },
        )?),
        c::TaskWaitWhy::Refreshing => w::TaskWaitWhy::Refreshing,
        c::TaskWaitWhy::Host => w::TaskWaitWhy::Host,
        c::TaskWaitWhy::Empty => w::TaskWaitWhy::Empty,
        c::TaskWaitWhy::Invalid(value) => w::TaskWaitWhy::Invalid(w::TaskWaitInvalid::new(
            limits,
            w::TaskWaitInvalidParts { why: wire_account_why(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_task_source_state(
    value: &c::TaskSourceState,
    limits: &w::Limits,
) -> Result<w::TaskSourceState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskSourceState::Asking => w::TaskSourceState::Asking,
        c::TaskSourceState::Granted => w::TaskSourceState::Granted,
        c::TaskSourceState::Later { until } => w::TaskSourceState::Later(w::TaskSourceStateUntil::new(
            limits,
            w::TaskSourceStateUntilParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::TaskSourceState::Limited { until } => w::TaskSourceState::Limited(w::TaskSourceStateUntil::new(
            limits,
            w::TaskSourceStateUntilParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::TaskSourceState::Refused => w::TaskSourceState::Refused,
        c::TaskSourceState::NeedsSignIn => w::TaskSourceState::NeedsSignIn,
        c::TaskSourceState::Misconfigured => w::TaskSourceState::Misconfigured,
        c::TaskSourceState::Unloaded { at, why } => w::TaskSourceState::Unloaded(w::TaskSourceStateUnloaded::new(
            limits,
            w::TaskSourceStateUnloadedParts {
                at: wire_task_source_place(at, limits)?,
                why: wire_task_source_unread(why, limits)?,
            },
        )?),
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
        c::TaskWaitRefused::Hello { kind, at, why } => w::TaskWaitRefused::Hello(w::TaskWaitHello::new(
            limits,
            w::TaskWaitHelloParts {
                kind: wire_host_kind(kind, limits)?,
                at: (at).as_nanos(),
                why: wire_hello_refusal(why, limits)?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_me_model(value: &c::MeModel, limits: &w::Limits) -> Result<w::MeModel, w::Problem> {
    w::MeModel::new(
        limits,
        w::MeModelParts {
            name: bytes::copy_of(value.name.bytes()),
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            window: value.window,
            output: value.output,
            efforts: list(value.efforts.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
            prices: wire_me_prices(&value.prices, limits)?,
            description: value
                .description
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_me_prices(value: &c::ModelPrices, limits: &w::Limits) -> Result<w::MePrices, w::Problem> {
    w::MePrices::new(
        limits,
        w::MePricesParts {
            input: value.input,
            cached: value.cached,
            cache_write: value.cache_write.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            output: value.output,
            unit: value.unit,
        },
    )
}

pub(in crate::codec) fn wire_provider_offer(
    value: &c::ProviderOffer,
    limits: &w::Limits,
) -> Result<w::ProviderOffer, w::Problem> {
    w::ProviderOffer::new(
        limits,
        w::ProviderOfferParts {
            skein_bools: [value.sign_in, value.key, value.programs_cut, value.none, value.signed_in_plan],
            provider: value.provider,
            name: wire_document_text(&value.name, limits)?,
            programs: list(value.programs.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
            manage_usage: value
                .manage_usage
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of(item)))
                .transpose()?,
            key_bytes: value.key_bytes,
        },
    )
}

pub(in crate::codec) fn wire_payer_readiness(
    value: &c::PayerReadiness,
    limits: &w::Limits,
) -> Result<w::PayerReadiness, w::Problem> {
    w::PayerReadiness::new(
        limits,
        w::PayerReadinessParts {
            provider: value.provider,
            payer: wire_account_use(&value.payer, limits)?,
            readiness: wire_readiness(&value.readiness, limits)?,
            settled: value.settled.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_readiness(value: &c::Readiness, _limits: &w::Limits) -> Result<w::Readiness, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Readiness::Usable => w::Readiness::Usable,
        c::Readiness::Unavailable => w::Readiness::Unavailable,
        c::Readiness::Unknown => w::Readiness::Unknown,
    })
}
