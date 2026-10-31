//! Wire projections of owner accounts facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{
    wire_document_text, wire_owner_account_place, wire_owner_account_state, wire_owner_account_unread,
    wire_owner_program_host, wire_payer_readiness,
};
pub(in crate::codec) fn wire_owner_account_use(
    value: &c::OwnerAccountUse,
    limits: &w::Limits,
) -> Result<w::OwnerAccountUse, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountUse::Party(value) => w::OwnerAccountUse::Party(w::OwnerAccountUseParty::new(
            limits,
            w::OwnerAccountUsePartyParts { party: *(value) },
        )?),
        c::OwnerAccountUse::Project(value) => w::OwnerAccountUse::Project(w::OwnerAccountUseProject::new(
            limits,
            w::OwnerAccountUseProjectParts { project: *(value) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_account_holder(
    value: &c::OwnerAccountHolder,
    limits: &w::Limits,
) -> Result<w::OwnerAccountHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountHolder::Deployment => w::OwnerAccountHolder::Deployment,
        c::OwnerAccountHolder::Party(value) => w::OwnerAccountHolder::Party(w::OwnerAccountHolderParty::new(
            limits,
            w::OwnerAccountHolderPartyParts { party: *(value) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_account_source(
    value: &c::OwnerAccountSource,
    limits: &w::Limits,
) -> Result<w::OwnerAccountSource, w::Problem> {
    w::OwnerAccountSource::new(
        limits,
        w::OwnerAccountSourceParts {
            name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
            place: value.place,
            number: value.number,
            kind: wire_owner_account_kind(&value.kind, limits)?,
            state: wire_owner_account_state(&value.state, limits)?,
            usable: value.usable,
            expiring: value.expiring,
            attention: value.attention,
            spent_until: value.spent_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_account_kind(
    value: &c::OwnerAccountKind,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountKind::SignIn => w::OwnerAccountKind::SignIn,
        c::OwnerAccountKind::Key => w::OwnerAccountKind::Key,
        c::OwnerAccountKind::Program => w::OwnerAccountKind::Program,
        c::OwnerAccountKind::None => w::OwnerAccountKind::None,
    })
}

pub(in crate::codec) fn wire_owner_accounts(
    value: &c::OwnerAccounts,
    limits: &w::Limits,
) -> Result<w::OwnerAccounts, w::Problem> {
    w::OwnerAccounts::new(
        limits,
        w::OwnerAccountsParts {
            skein_bools: [value.usable, value.settled, value.cut, value.readiness_cut],
            program_hosts: pieces(
                value.program_hosts.iter().map(|item| wire_owner_program_host(item, limits)),
                w::OwnerProgramHost::measure,
                w::OwnerProgramHost::encode,
            )?,
            accounts: pieces(
                value.accounts.iter().map(|item| wire_owner_account_line(item, limits)),
                w::OwnerAccountLine::measure,
                w::OwnerAccountLine::encode,
            )?,
            signing: pieces(
                value.signing.iter().map(|item| wire_owner_account_sign_in(item, limits)),
                w::OwnerAccountSignIn::measure,
                w::OwnerAccountSignIn::encode,
            )?,
            last_end: value.last_end.as_ref().map(|item| wire_owner_account_last_end(item, limits)).transpose()?,
            readiness: pieces(
                value.readiness.iter().map(|item| wire_payer_readiness(item, limits)),
                w::PayerReadiness::measure,
                w::PayerReadiness::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_owner_account_line(
    value: &c::OwnerAccountLine,
    limits: &w::Limits,
) -> Result<w::OwnerAccountLine, w::Problem> {
    w::OwnerAccountLine::new(
        limits,
        w::OwnerAccountLineParts {
            number: value.number,
            place: value.place,
            uses: value
                .uses
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_owner_account_use(item, limits))))
                .transpose()?,
            plan: value.plan,
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            display: wire_document_text(&value.display, limits)?,
            holder: wire_owner_account_holder(&value.holder, limits)?,
            sources: pieces(
                value.sources.iter().map(|item| wire_owner_account_source(item, limits)),
                w::OwnerAccountSource::measure,
                w::OwnerAccountSource::encode,
            )?,
            cut: value.cut,
            provider_name: value.provider_name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            manage_usage: value
                .manage_usage
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of(item)))
                .transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_account_sign_in(
    value: &c::OwnerAccountSignIn,
    limits: &w::Limits,
) -> Result<w::OwnerAccountSignIn, w::Problem> {
    w::OwnerAccountSignIn::new(
        limits,
        w::OwnerAccountSignInParts {
            source: value.source,
            provider: value.provider,
            holder: wire_owner_account_holder(&value.holder, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_account_last_end(
    value: &c::OwnerAccountLastEnd,
    limits: &w::Limits,
) -> Result<w::OwnerAccountLastEnd, w::Problem> {
    w::OwnerAccountLastEnd::new(
        limits,
        w::OwnerAccountLastEndParts { source: value.source, end: wire_owner_account_end(&value.end, limits)? },
    )
}

pub(in crate::codec) fn wire_owner_account_end(
    value: &c::OwnerAccountEnd,
    limits: &w::Limits,
) -> Result<w::OwnerAccountEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountEnd::Made { account, declined } => w::OwnerAccountEnd::Made(w::OwnerAccountEndMade::new(
            limits,
            w::OwnerAccountEndMadeParts { account: *(account), declined: *(declined) },
        )?),
        c::OwnerAccountEnd::Renewed { account, declined } => {
            w::OwnerAccountEnd::Renewed(w::OwnerAccountEndRenewed::new(
                limits,
                w::OwnerAccountEndRenewedParts { account: *(account), declined: *(declined) },
            )?)
        }
        c::OwnerAccountEnd::Refused { refusal } => w::OwnerAccountEnd::Refused(w::OwnerAccountEndRefused::new(
            limits,
            w::OwnerAccountEndRefusedParts { refusal: wire_owner_account_refusal(refusal, limits)? },
        )?),
        c::OwnerAccountEnd::Failed { failure } => w::OwnerAccountEnd::Failed(w::OwnerAccountEndFailed::new(
            limits,
            w::OwnerAccountEndFailedParts { failure: wire_owner_account_failure(failure, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_account_refusal(
    value: &c::OwnerAccountRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerAccountRefusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountRefusal::NotLimited => w::OwnerAccountRefusal::NotLimited,
        c::OwnerAccountRefusal::Uses => w::OwnerAccountRefusal::Uses,
        c::OwnerAccountRefusal::NotKept { why } => w::OwnerAccountRefusal::NotKept(w::OwnerAccountRefusalNotKept::new(
            limits,
            w::OwnerAccountRefusalNotKeptParts { why: wire_owner_account_key_failure(why, limits)? },
        )?),
        c::OwnerAccountRefusal::Limit { bound } => w::OwnerAccountRefusal::Limit(w::OwnerAccountRefusalLimit::new(
            limits,
            w::OwnerAccountRefusalLimitParts { bound: wire_owner_account_bound(bound, limits)? },
        )?),
        c::OwnerAccountRefusal::Held { provider } => w::OwnerAccountRefusal::Held(w::OwnerAccountRefusalHeld::new(
            limits,
            w::OwnerAccountRefusalHeldParts { provider: *(provider) },
        )?),
        c::OwnerAccountRefusal::Provider { provider } => w::OwnerAccountRefusal::Provider(
            w::OwnerAccountRefusalProvider::new(limits, w::OwnerAccountRefusalProviderParts { provider: *(provider) })?,
        ),
        c::OwnerAccountRefusal::Subject => w::OwnerAccountRefusal::Subject,
        c::OwnerAccountRefusal::Unknown => w::OwnerAccountRefusal::Unknown,
        c::OwnerAccountRefusal::Numbers => w::OwnerAccountRefusal::Numbers,
        c::OwnerAccountRefusal::Kind { kind } => w::OwnerAccountRefusal::Kind(w::OwnerAccountRefusalKind::new(
            limits,
            w::OwnerAccountRefusalKindParts { kind: wire_owner_account_kind(kind, limits)? },
        )?),
        c::OwnerAccountRefusal::Closing => w::OwnerAccountRefusal::Closing,
        c::OwnerAccountRefusal::NotStarted => w::OwnerAccountRefusal::NotStarted,
        c::OwnerAccountRefusal::NotSignedIn => w::OwnerAccountRefusal::NotSignedIn,
        c::OwnerAccountRefusal::NotOurs => w::OwnerAccountRefusal::NotOurs,
        c::OwnerAccountRefusal::Cancelled => w::OwnerAccountRefusal::Cancelled,
        c::OwnerAccountRefusal::Moving => w::OwnerAccountRefusal::Moving,
    })
}

pub(in crate::codec) fn wire_owner_account_key_failure(
    value: &c::OwnerAccountKeyFailure,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountKeyFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountKeyFailure::TooLong => w::OwnerAccountKeyFailure::TooLong,
        c::OwnerAccountKeyFailure::NotStored => w::OwnerAccountKeyFailure::NotStored,
    })
}

pub(in crate::codec) fn wire_owner_account_bound(
    value: &c::OwnerAccountBound,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountBound, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountBound::Accounts => w::OwnerAccountBound::Accounts,
        c::OwnerAccountBound::HolderAccounts => w::OwnerAccountBound::HolderAccounts,
        c::OwnerAccountBound::Sources => w::OwnerAccountBound::Sources,
        c::OwnerAccountBound::SignIns => w::OwnerAccountBound::SignIns,
    })
}

pub(in crate::codec) fn wire_owner_account_failure(
    value: &c::OwnerAccountFailure,
    limits: &w::Limits,
) -> Result<w::OwnerAccountFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountFailure::Later { after } => w::OwnerAccountFailure::Later(w::OwnerAccountFailureLater::new(
            limits,
            w::OwnerAccountFailureLaterParts {
                after: (after).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::OwnerAccountFailure::SignInAgain => w::OwnerAccountFailure::SignInAgain,
        c::OwnerAccountFailure::MissingScope { scope } => {
            w::OwnerAccountFailure::MissingScope(w::OwnerAccountFailureMissingScope::new(
                limits,
                w::OwnerAccountFailureMissingScopeParts { scope: *(scope) },
            )?)
        }
        c::OwnerAccountFailure::Misconfigured => w::OwnerAccountFailure::Misconfigured,
        c::OwnerAccountFailure::Unloaded { at, why } => {
            w::OwnerAccountFailure::Unloaded(w::OwnerAccountFailureUnloaded::new(
                limits,
                w::OwnerAccountFailureUnloadedParts {
                    at: wire_owner_account_place(at, limits)?,
                    why: wire_owner_account_unread(why, limits)?,
                },
            )?)
        }
        c::OwnerAccountFailure::Cancelled => w::OwnerAccountFailure::Cancelled,
    })
}
