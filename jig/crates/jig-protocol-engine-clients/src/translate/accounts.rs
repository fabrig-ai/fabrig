//! Whole secret-free accounts documents (client/domain/contract/documents.md, 1; domain/accounts.md, 3–5).
use super::{deployment, owner, pieces, w};
use alloc::boxed::Box;
use jig_domain_accounts as a;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
pub fn facts(value: &List<a::AccountFact>, limits: &w::Limits) -> Result<List<w::AccountFact>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for fact in value {
        let value = match &fact.value {
            a::FactValue::Text(text) => w::AccountFactValue::Text(w::AccountFactText::new(
                limits,
                w::AccountFactTextParts { text: text.clone() },
            )?),
            a::FactValue::Count(count) => w::AccountFactValue::Count(w::AccountFactCount::new(
                limits,
                w::AccountFactCountParts { count: *count },
            )?),
            a::FactValue::Instant(at) => w::AccountFactValue::Instant(w::AccountFactInstant::new(
                limits,
                w::AccountFactInstantParts { at: at.as_nanos() },
            )?),
            a::FactValue::Span(nanos) => {
                w::AccountFactValue::Span(w::AccountFactSpan::new(limits, w::AccountFactSpanParts { nanos: *nanos })?)
            }
            a::FactValue::Share(basis) => w::AccountFactValue::Share(w::AccountFactShare::new(
                limits,
                w::AccountFactShareParts { basis: *basis },
            )?),
            a::FactValue::Window(window) => w::AccountFactValue::Window(w::AccountWindow::new(
                limits,
                w::AccountWindowParts {
                    length: window.length,
                    used: window.used,
                    resets: match window.resets {
                        Some(at) => Some(at.as_nanos()),
                        None => None,
                    },
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
        rows.push(row).expect("supplied facts fit");
    }
    Ok(rows)
}
pub fn payer(value: a::Use, limits: &w::Limits) -> Result<w::AccountUse, w::Problem> {
    Ok(match value {
        a::Use::Party(party) => {
            w::AccountUse::Party(w::AccountUseParty::new(limits, w::AccountUsePartyParts { party })?)
        }
        a::Use::Project(project) => {
            w::AccountUse::Project(w::AccountUseProject::new(limits, w::AccountUseProjectParts { project })?)
        }
    })
}
pub fn holder(value: a::Holder, limits: &w::Limits) -> Result<w::AccountHolder, w::Problem> {
    Ok(match value {
        a::Holder::Deployment => w::AccountHolder::Deployment,
        a::Holder::Party(party) => {
            w::AccountHolder::Party(w::AccountHolderParty::new(limits, w::AccountHolderPartyParts { party })?)
        }
    })
}
fn kind(value: a::Kind) -> w::AccountKind {
    match value {
        a::Kind::SignIn => w::AccountKind::SignIn,
        a::Kind::Key => w::AccountKind::Key,
        a::Kind::Program => w::AccountKind::Program,
        a::Kind::None => w::AccountKind::None,
    }
}
fn bound(value: a::Bound) -> w::AccountBound {
    match value {
        a::Bound::Accounts => w::AccountBound::Accounts,
        a::Bound::HolderAccounts => w::AccountBound::HolderAccounts,
        a::Bound::Sources => w::AccountBound::Sources,
        a::Bound::SignIns => w::AccountBound::SignIns,
    }
}
fn key_failure(value: a::KeyFailure) -> w::AccountKeyFailure {
    match value {
        a::KeyFailure::TooLong => w::AccountKeyFailure::TooLong,
        a::KeyFailure::NotStored => w::AccountKeyFailure::NotStored,
    }
}
pub fn refusal(value: a::Refusal, limits: &w::Limits) -> Result<w::AccountRefusal, w::Problem> {
    Ok(match value {
        // A finish refusal leaves the sign-in waiting, so it never ends an accounts watch's sign-in.
        a::Refusal::SignInRedirect { .. } => unreachable!("a redirect refusal is only a finish answer"),
        a::Refusal::NameTaken | a::Refusal::ProgramNameReserved => w::AccountRefusal::ProgramNameReserved,
        a::Refusal::NoTerminal | a::Refusal::LoginUnderWay | a::Refusal::NotLending => {
            unreachable!("a login request refusal is never a login end")
        }
        a::Refusal::NotSignedIn => w::AccountRefusal::NotSignedIn,
        a::Refusal::NotOurs => w::AccountRefusal::NotOurs,
        a::Refusal::Cancelled => w::AccountRefusal::Cancelled,
        a::Refusal::Moving => w::AccountRefusal::Moving,
        a::Refusal::Unknown => w::AccountRefusal::Unknown,
        a::Refusal::NotLimited => w::AccountRefusal::NotLimited,
        a::Refusal::Uses => w::AccountRefusal::Uses,
        a::Refusal::NotKept { why } => w::AccountRefusal::NotKept(w::AccountRefusalNotKept::new(
            limits,
            w::AccountRefusalNotKeptParts { why: key_failure(why) },
        )?),
        a::Refusal::Limit { bound } => w::AccountRefusal::Limit(w::AccountRefusalLimit::new(
            limits,
            w::AccountRefusalLimitParts { bound: self::bound(bound) },
        )?),
        a::Refusal::Held { provider } => {
            w::AccountRefusal::Held(w::AccountRefusalHeld::new(limits, w::AccountRefusalHeldParts { provider })?)
        }
        a::Refusal::Provider { provider } => w::AccountRefusal::Provider(w::AccountRefusalProvider::new(
            limits,
            w::AccountRefusalProviderParts { provider },
        )?),
        a::Refusal::Subject => w::AccountRefusal::Subject,
        a::Refusal::Numbers => w::AccountRefusal::Numbers,
        a::Refusal::Kind { kind } => w::AccountRefusal::Kind(w::AccountRefusalKind::new(
            limits,
            w::AccountRefusalKindParts { kind: self::kind(kind) },
        )?),
        a::Refusal::Closing => w::AccountRefusal::Closing,
        a::Refusal::NotStarted => w::AccountRefusal::NotStarted,
    })
}
pub fn failure(value: a::Failure, limits: &w::Limits) -> Result<w::AccountFailure, w::Problem> {
    Ok(match value {
        a::Failure::Later { after } => {
            w::AccountFailure::Later(w::AccountFailureLater::new(limits, w::AccountFailureLaterParts { after })?)
        }
        a::Failure::SignInAgain => w::AccountFailure::SignInAgain,
        a::Failure::MissingScope { scope } => w::AccountFailure::MissingScope(w::AccountFailureMissingScope::new(
            limits,
            w::AccountFailureMissingScopeParts { scope },
        )?),
        a::Failure::Misconfigured => w::AccountFailure::Misconfigured,
        a::Failure::Unloaded { at, why } => w::AccountFailure::Unloaded(w::AccountFailureUnloaded::new(
            limits,
            w::AccountFailureUnloadedParts { at: deployment::account_place(at), why: deployment::account_unread(why) },
        )?),
        a::Failure::Cancelled => w::AccountFailure::Cancelled,
    })
}
pub fn end(value: a::End, limits: &w::Limits) -> Result<w::AccountEnd, w::Problem> {
    Ok(match value {
        a::End::Made { account, declined } => {
            w::AccountEnd::Made(w::AccountEndMade::new(limits, w::AccountEndMadeParts { account, declined })?)
        }
        a::End::Renewed { account, declined } => {
            w::AccountEnd::Renewed(w::AccountEndRenewed::new(limits, w::AccountEndRenewedParts { account, declined })?)
        }
        a::End::Refused { refusal } => w::AccountEnd::Refused(w::AccountEndRefused::new(
            limits,
            w::AccountEndRefusedParts { refusal: self::refusal(refusal, limits)? },
        )?),
        a::End::Failed { failure } => w::AccountEnd::Failed(w::AccountEndFailed::new(
            limits,
            w::AccountEndFailedParts { failure: self::failure(failure, limits)? },
        )?),
    })
}
pub fn uses(value: Option<&List<a::Use>>, limits: &w::Limits) -> Result<Option<List<w::AccountUse>>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let mut rows = List::with_capacity(value.len());
    for row in value {
        rows.push(payer(*row, limits)?).expect("supplied uses fit");
    }
    Ok(Some(rows))
}
pub fn source(value: &d::SourceLine, limits: &w::Limits) -> Result<w::AccountSource, w::Problem> {
    w::AccountSource::new(
        limits,
        w::AccountSourceParts {
            name: value.name.clone(),
            place: value.place,
            number: value.number,
            kind: kind(value.kind),
            state: deployment::account_state(value.state, limits)?,
            usable: value.usable,
            expiring: value.expiring,
            attention: value.attention,
            spent_until: match value.spent_until {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
        },
    )
}
pub fn source_pieces(
    value: &List<d::SourceLine>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(value.len());
    for row in value {
        let row = source(row, limits)?;
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        row.encode(&mut out).expect("measured account source");
        encoded.push(out.finish()).expect("supplied source pieces fit");
    }
    pieces(&encoded, bound, path)
}
pub fn line(value: &d::AccountLine, limits: &w::Limits) -> Result<w::AccountLine, w::Problem> {
    w::AccountLine::new(
        limits,
        w::AccountLineParts {
            list: list(value.list.as_ref(), limits)?,
            facts: facts(&value.facts, limits)?,
            number: value.number,
            place: value.place,
            uses: uses(value.uses.as_ref(), limits)?,
            plan: value.plan,
            provider: value.provider,
            provider_name: match &value.provider_name {
                Some(name) => Some(owner::text(name, limits)?),
                None => None,
            },
            manage_usage: value.manage_usage.clone(),
            display: owner::text(&value.display, limits)?,
            holder: holder(value.holder, limits)?,
            sources: source_pieces(&value.sources, limits, limits.account_line_sources, w::Path::AccountLineSources)?,
            cut: value.cut,
        },
    )
}
pub fn document(value: &d::Accounts, limits: &w::Limits) -> Result<w::Accounts, w::Problem> {
    let mut accounts = List::with_capacity(value.accounts.len());
    for row in &value.accounts {
        let row = line(row, limits)?;
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        row.encode(&mut out).expect("measured account line");
        accounts.push(out.finish()).expect("supplied account pieces fit");
    }
    let accounts = pieces(&accounts, limits.accounts_accounts, w::Path::AccountsAccounts)?;
    let mut signing = List::with_capacity(value.signing.len());
    for row in &value.signing {
        let row = w::AccountSignIn::new(
            limits,
            w::AccountSignInParts { source: row.source, provider: row.provider, holder: holder(row.holder, limits)? },
        )?;
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        row.encode(&mut out).expect("measured sign-in line");
        signing.push(out.finish()).expect("supplied sign-in pieces fit");
    }
    let signing = pieces(&signing, limits.accounts_signing, w::Path::AccountsSigning)?;
    let last_end = match &value.last_end {
        Some(row) => Some(w::AccountLastEnd::new(
            limits,
            w::AccountLastEndParts { source: row.source, end: end(row.end, limits)? },
        )?),
        None => None,
    };
    w::Accounts::new(
        limits,
        w::AccountsParts {
            skein_bools: [value.lendable, value.usable, value.settled, value.cut, value.readiness_cut],
            lending: match &value.lending {
                Some(login) => Some(terminal_login(login, limits)?),
                None => None,
            },
            lent_end: match value.lent_end {
                Some(ended) => Some(w::AccountLentEnd::new(
                    limits,
                    w::AccountLentEndParts { number: ended.number, end: login_end(ended.end, limits)? },
                )?),
                None => None,
            },
            readiness: super::catalogue::readiness(&value.readiness, limits.accounts_readiness, limits)?,
            program_hosts: super::programs::hosts(&value.program_hosts, limits)?,
            accounts,
            signing,
            last_end,
        },
    )
}

/// Public history only, without the child's model-name bits.
pub fn list(value: Option<&d::AccountList>, limits: &w::Limits) -> Result<Option<w::AccountList>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let failure = match value.failure {
        Some(jig_domain_accounts::ReadFailure::Refused { status }) => Some(w::AccountListFailure::Refused(
            w::AccountListRefused::new(limits, w::AccountListRefusedParts { status })?,
        )),
        Some(jig_domain_accounts::ReadFailure::Unreachable) => Some(w::AccountListFailure::Unreachable),
        Some(jig_domain_accounts::ReadFailure::Unreadable) => Some(w::AccountListFailure::Unreadable),
        Some(jig_domain_accounts::ReadFailure::Deadline) => Some(w::AccountListFailure::Deadline),
        Some(jig_domain_accounts::ReadFailure::Agent) => Some(w::AccountListFailure::Agent),
        Some(jig_domain_accounts::ReadFailure::Room) => Some(w::AccountListFailure::Room),
        None => None,
    };
    Ok(Some(w::AccountList::new(
        limits,
        w::AccountListParts {
            read: match value.read {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            tried: match value.tried {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            failure,
            left_out: value.left_out,
        },
    )?))
}

pub(crate) fn terminal_login(value: &a::TerminalLogin, limits: &w::Limits) -> Result<w::TerminalLogin, w::Problem> {
    w::TerminalLogin::new(
        limits,
        w::TerminalLoginParts {
            number: value.number,
            provider: value.provider,
            name: value.name.clone(),
            cancelled: value.cancelled,
        },
    )
}
pub(crate) fn terminal(value: Option<&a::TerminalLogin>, limits: &w::Limits) -> Result<w::Terminal, w::Problem> {
    w::Terminal::new(
        limits,
        w::TerminalParts {
            login: match value {
                Some(login) => Some(terminal_login(login, limits)?),
                None => None,
            },
        },
    )
}
fn login_exit(value: a::LoginExit, limits: &w::Limits) -> Result<w::LoginExit, w::Problem> {
    Ok(match value {
        a::LoginExit::Exited { code } => {
            w::LoginExit::Exited(w::LoginExitExited::new(limits, w::LoginExitExitedParts { code })?)
        }
        a::LoginExit::Killed { signal } => {
            w::LoginExit::Killed(w::LoginExitKilled::new(limits, w::LoginExitKilledParts { signal })?)
        }
        a::LoginExit::NotStarted => w::LoginExit::NotStarted,
    })
}
fn login_end(value: a::LoginEnd, limits: &w::Limits) -> Result<w::LoginEnd, w::Problem> {
    Ok(match value {
        a::LoginEnd::SignedIn { source } => {
            w::LoginEnd::SignedIn(w::LoginEndSignedIn::new(limits, w::LoginEndSignedInParts { source })?)
        }
        a::LoginEnd::NotFinished { exit } => w::LoginEnd::NotFinished(w::LoginEndNotFinished::new(
            limits,
            w::LoginEndNotFinishedParts {
                exit: match exit {
                    Some(exit) => Some(login_exit(exit, limits)?),
                    None => None,
                },
            },
        )?),
        a::LoginEnd::Cancelled => w::LoginEnd::Cancelled,
        a::LoginEnd::Refused { refusal: why } => w::LoginEnd::Refused(w::LoginEndRefused::new(
            limits,
            w::LoginEndRefusedParts { refusal: refusal(why, limits)? },
        )?),
    })
}
