//! Wire projections of owner accounts facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{
    wire_account_place, wire_account_state, wire_account_unread, wire_document_text, wire_payer_readiness,
    wire_program_host,
};
pub(in crate::codec) fn wire_terminal_login(value: &c::TerminalLogin, limits: &w::Limits) -> Result<w::TerminalLogin, w::Problem> {
    w::TerminalLogin::new(limits, w::TerminalLoginParts {
        number: value.number, provider: value.provider, name: bytes::copy_of(&value.name), cancelled: value.cancelled,
    })
}
pub(in crate::codec) fn wire_terminal(value: &c::Terminal, limits: &w::Limits) -> Result<w::Terminal, w::Problem> {
    w::Terminal::new(limits, w::TerminalParts { login: value.login.as_ref().map(|item| wire_terminal_login(item, limits)).transpose()? })
}
pub(in crate::codec) fn wire_login_exit(value: &c::LoginExit, limits: &w::Limits) -> Result<w::LoginExit, w::Problem> {
    Ok(match value {
        c::LoginExit::Exited { code } => w::LoginExit::Exited(w::LoginExitExited::new(limits, w::LoginExitExitedParts { code: *code })?),
        c::LoginExit::Killed { signal } => w::LoginExit::Killed(w::LoginExitKilled::new(limits, w::LoginExitKilledParts { signal: *signal })?),
        c::LoginExit::NotStarted => w::LoginExit::NotStarted,
    })
}
pub(in crate::codec) fn wire_login_end(value: &c::LoginEnd, limits: &w::Limits) -> Result<w::LoginEnd, w::Problem> {
    Ok(match value {
        c::LoginEnd::SignedIn { source } => w::LoginEnd::SignedIn(w::LoginEndSignedIn::new(limits, w::LoginEndSignedInParts { source: *source })?),
        c::LoginEnd::NotFinished { exit } => w::LoginEnd::NotFinished(w::LoginEndNotFinished::new(limits, w::LoginEndNotFinishedParts { exit: exit.as_ref().map(|item| wire_login_exit(item, limits)).transpose()? })?),
        c::LoginEnd::Cancelled => w::LoginEnd::Cancelled,
        c::LoginEnd::Refused { refusal } => w::LoginEnd::Refused(w::LoginEndRefused::new(limits, w::LoginEndRefusedParts { refusal: wire_account_refusal(refusal, limits)? })?),
    })
}
pub(in crate::codec) fn wire_account_use(
    value: &c::AccountUse,
    limits: &w::Limits,
) -> Result<w::AccountUse, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountUse::Party(value) => {
            w::AccountUse::Party(w::AccountUseParty::new(limits, w::AccountUsePartyParts { party: *(value) })?)
        }
        c::AccountUse::Project(value) => {
            w::AccountUse::Project(w::AccountUseProject::new(limits, w::AccountUseProjectParts { project: *(value) })?)
        }
    })
}

pub(in crate::codec) fn wire_account_holder(
    value: &c::AccountHolder,
    limits: &w::Limits,
) -> Result<w::AccountHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountHolder::Deployment => w::AccountHolder::Deployment,
        c::AccountHolder::Party(value) => {
            w::AccountHolder::Party(w::AccountHolderParty::new(limits, w::AccountHolderPartyParts { party: *(value) })?)
        }
    })
}

pub(in crate::codec) fn wire_account_source(
    value: &c::AccountSource,
    limits: &w::Limits,
) -> Result<w::AccountSource, w::Problem> {
    w::AccountSource::new(
        limits,
        w::AccountSourceParts {
            name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
            place: value.place,
            number: value.number,
            kind: wire_account_kind(&value.kind, limits)?,
            state: wire_account_state(&value.state, limits)?,
            usable: value.usable,
            expiring: value.expiring,
            attention: value.attention,
            spent_until: value.spent_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_account_kind(
    value: &c::AccountKind,
    _limits: &w::Limits,
) -> Result<w::AccountKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountKind::SignIn => w::AccountKind::SignIn,
        c::AccountKind::Key => w::AccountKind::Key,
        c::AccountKind::Program => w::AccountKind::Program,
        c::AccountKind::None => w::AccountKind::None,
    })
}

pub(in crate::codec) fn wire_accounts(value: &c::Accounts, limits: &w::Limits) -> Result<w::Accounts, w::Problem> {
    w::Accounts::new(
        limits,
        w::AccountsParts {
            skein_bools: [value.lendable, value.usable, value.settled, value.cut, value.readiness_cut],
            lending: value.lending.as_ref().map(|item| wire_terminal_login(item, limits)).transpose()?,
            lent_end: value.lent_end.as_ref().map(|item| {
                w::AccountLentEnd::new(limits, w::AccountLentEndParts { number: item.number, end: wire_login_end(&item.end, limits)? })
            }).transpose()?,
            program_hosts: pieces(
                value.program_hosts.iter().map(|item| wire_program_host(item, limits)),
                w::ProgramHost::measure,
                w::ProgramHost::encode,
            )?,
            accounts: pieces(
                value.accounts.iter().map(|item| wire_account_line(item, limits)),
                w::AccountLine::measure,
                w::AccountLine::encode,
            )?,
            signing: pieces(
                value.signing.iter().map(|item| wire_account_sign_in(item, limits)),
                w::AccountSignIn::measure,
                w::AccountSignIn::encode,
            )?,
            last_end: value.last_end.as_ref().map(|item| wire_account_last_end(item, limits)).transpose()?,
            readiness: pieces(
                value.readiness.iter().map(|item| wire_payer_readiness(item, limits)),
                w::PayerReadiness::measure,
                w::PayerReadiness::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_account_line(
    value: &c::AccountLine,
    limits: &w::Limits,
) -> Result<w::AccountLine, w::Problem> {
    w::AccountLine::new(
        limits,
        w::AccountLineParts {
            list: value.list.as_ref().map(|list| wire_account_list(list, limits)).transpose()?,
            facts: list(value.facts.iter().map(|fact| wire_account_fact(fact, limits)))?,
            number: value.number,
            place: value.place,
            uses: value
                .uses
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_account_use(item, limits))))
                .transpose()?,
            plan: value.plan,
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            display: wire_document_text(&value.display, limits)?,
            holder: wire_account_holder(&value.holder, limits)?,
            sources: pieces(
                value.sources.iter().map(|item| wire_account_source(item, limits)),
                w::AccountSource::measure,
                w::AccountSource::encode,
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

pub(in crate::codec) fn wire_account_sign_in(
    value: &c::AccountSignIn,
    limits: &w::Limits,
) -> Result<w::AccountSignIn, w::Problem> {
    w::AccountSignIn::new(
        limits,
        w::AccountSignInParts {
            source: value.source,
            provider: value.provider,
            holder: wire_account_holder(&value.holder, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_account_last_end(
    value: &c::AccountLastEnd,
    limits: &w::Limits,
) -> Result<w::AccountLastEnd, w::Problem> {
    w::AccountLastEnd::new(
        limits,
        w::AccountLastEndParts { source: value.source, end: wire_account_end(&value.end, limits)? },
    )
}

pub(in crate::codec) fn wire_account_end(
    value: &c::AccountEnd,
    limits: &w::Limits,
) -> Result<w::AccountEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountEnd::Made { account, declined } => w::AccountEnd::Made(w::AccountEndMade::new(
            limits,
            w::AccountEndMadeParts { account: *(account), declined: *(declined) },
        )?),
        c::AccountEnd::Renewed { account, declined } => w::AccountEnd::Renewed(w::AccountEndRenewed::new(
            limits,
            w::AccountEndRenewedParts { account: *(account), declined: *(declined) },
        )?),
        c::AccountEnd::Refused { refusal } => w::AccountEnd::Refused(w::AccountEndRefused::new(
            limits,
            w::AccountEndRefusedParts { refusal: wire_account_refusal(refusal, limits)? },
        )?),
        c::AccountEnd::Failed { failure } => w::AccountEnd::Failed(w::AccountEndFailed::new(
            limits,
            w::AccountEndFailedParts { failure: wire_account_failure(failure, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_account_refusal(
    value: &c::AccountRefusal,
    limits: &w::Limits,
) -> Result<w::AccountRefusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountRefusal::NotLimited => w::AccountRefusal::NotLimited,
        c::AccountRefusal::Uses => w::AccountRefusal::Uses,
        c::AccountRefusal::ProgramNameReserved => w::AccountRefusal::ProgramNameReserved,
        c::AccountRefusal::NotKept { why } => w::AccountRefusal::NotKept(w::AccountRefusalNotKept::new(
            limits,
            w::AccountRefusalNotKeptParts { why: wire_account_key_failure(why, limits)? },
        )?),
        c::AccountRefusal::Limit { bound } => w::AccountRefusal::Limit(w::AccountRefusalLimit::new(
            limits,
            w::AccountRefusalLimitParts { bound: wire_account_bound(bound, limits)? },
        )?),
        c::AccountRefusal::Held { provider } => w::AccountRefusal::Held(w::AccountRefusalHeld::new(
            limits,
            w::AccountRefusalHeldParts { provider: *(provider) },
        )?),
        c::AccountRefusal::Provider { provider } => w::AccountRefusal::Provider(w::AccountRefusalProvider::new(
            limits,
            w::AccountRefusalProviderParts { provider: *(provider) },
        )?),
        c::AccountRefusal::Subject => w::AccountRefusal::Subject,
        c::AccountRefusal::Unknown => w::AccountRefusal::Unknown,
        c::AccountRefusal::Numbers => w::AccountRefusal::Numbers,
        c::AccountRefusal::Kind { kind } => w::AccountRefusal::Kind(w::AccountRefusalKind::new(
            limits,
            w::AccountRefusalKindParts { kind: wire_account_kind(kind, limits)? },
        )?),
        c::AccountRefusal::Closing => w::AccountRefusal::Closing,
        c::AccountRefusal::NotStarted => w::AccountRefusal::NotStarted,
        c::AccountRefusal::NotSignedIn => w::AccountRefusal::NotSignedIn,
        c::AccountRefusal::NotOurs => w::AccountRefusal::NotOurs,
        c::AccountRefusal::Cancelled => w::AccountRefusal::Cancelled,
        c::AccountRefusal::Moving => w::AccountRefusal::Moving,
    })
}

pub(in crate::codec) fn wire_account_key_failure(
    value: &c::AccountKeyFailure,
    _limits: &w::Limits,
) -> Result<w::AccountKeyFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountKeyFailure::TooLong => w::AccountKeyFailure::TooLong,
        c::AccountKeyFailure::NotStored => w::AccountKeyFailure::NotStored,
    })
}

pub(in crate::codec) fn wire_account_bound(
    value: &c::AccountBound,
    _limits: &w::Limits,
) -> Result<w::AccountBound, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountBound::Accounts => w::AccountBound::Accounts,
        c::AccountBound::HolderAccounts => w::AccountBound::HolderAccounts,
        c::AccountBound::Sources => w::AccountBound::Sources,
        c::AccountBound::SignIns => w::AccountBound::SignIns,
    })
}

pub(in crate::codec) fn wire_account_failure(
    value: &c::AccountFailure,
    limits: &w::Limits,
) -> Result<w::AccountFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountFailure::Later { after } => w::AccountFailure::Later(w::AccountFailureLater::new(
            limits,
            w::AccountFailureLaterParts {
                after: (after).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::AccountFailure::SignInAgain => w::AccountFailure::SignInAgain,
        c::AccountFailure::MissingScope { scope } => w::AccountFailure::MissingScope(
            w::AccountFailureMissingScope::new(limits, w::AccountFailureMissingScopeParts { scope: *(scope) })?,
        ),
        c::AccountFailure::Misconfigured => w::AccountFailure::Misconfigured,
        c::AccountFailure::Unloaded { at, why } => w::AccountFailure::Unloaded(w::AccountFailureUnloaded::new(
            limits,
            w::AccountFailureUnloadedParts {
                at: wire_account_place(at, limits)?,
                why: wire_account_unread(why, limits)?,
            },
        )?),
        c::AccountFailure::Cancelled => w::AccountFailure::Cancelled,
    })
}

/// Public history only, without the child's model-name bits.
pub(in crate::codec) fn wire_account_list(
    value: &c::AccountList,
    limits: &w::Limits,
) -> Result<w::AccountList, w::Problem> {
    let failure = match value.failure {
        Some(c::AccountListFailure::Refused { status }) => Some(w::AccountListFailure::Refused(
            w::AccountListRefused::new(limits, w::AccountListRefusedParts { status })?,
        )),
        Some(c::AccountListFailure::Unreachable) => Some(w::AccountListFailure::Unreachable),
        Some(c::AccountListFailure::Unreadable) => Some(w::AccountListFailure::Unreadable),
        Some(c::AccountListFailure::Deadline) => Some(w::AccountListFailure::Deadline),
        Some(c::AccountListFailure::Agent) => Some(w::AccountListFailure::Agent),
        Some(c::AccountListFailure::Room) => Some(w::AccountListFailure::Room),
        None => None,
    };
    w::AccountList::new(
        limits,
        w::AccountListParts {
            read: value.read.map(skein_lib::Wall::as_nanos),
            tried: value.tried.map(skein_lib::Wall::as_nanos),
            failure,
            left_out: value.left_out,
        },
    )
}

pub(in crate::codec) fn wire_account_fact(
    fact: &c::AccountFact,
    limits: &w::Limits,
) -> Result<w::AccountFact, w::Problem> {
    let value = match &fact.value {
        c::AccountFactValue::Text(text) => {
            w::AccountFactValue::Text(w::AccountFactText::new(limits, w::AccountFactTextParts { text: text.clone() })?)
        }
        c::AccountFactValue::Count(count) => {
            w::AccountFactValue::Count(w::AccountFactCount::new(limits, w::AccountFactCountParts { count: *count })?)
        }
        c::AccountFactValue::Instant(at) => w::AccountFactValue::Instant(w::AccountFactInstant::new(
            limits,
            w::AccountFactInstantParts { at: at.as_nanos() },
        )?),
        c::AccountFactValue::Span(nanos) => {
            w::AccountFactValue::Span(w::AccountFactSpan::new(limits, w::AccountFactSpanParts { nanos: *nanos })?)
        }
        c::AccountFactValue::Share(basis) => {
            w::AccountFactValue::Share(w::AccountFactShare::new(limits, w::AccountFactShareParts { basis: *basis })?)
        }
        c::AccountFactValue::Window(window) => w::AccountFactValue::Window(w::AccountWindow::new(
            limits,
            w::AccountWindowParts {
                length: window.length,
                used: window.used,
                resets: window.resets.map(skein_lib::Wall::as_nanos),
                state: window.state,
            },
        )?),
    };
    let row = w::AccountFact::new(
        limits,
        w::AccountFactParts {
            kind: fact.kind,
            label: fact.label.clone(),
            value,
            seen: fact.seen.as_nanos(),
            source: fact.source,
            origin: fact.origin,
        },
    )?;
    Ok(row)
}
