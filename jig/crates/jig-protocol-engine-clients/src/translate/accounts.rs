//! Whole secret-free accounts documents (client/domain/contract/documents.md, 1; domain/accounts.md, 3, and domain/accounts-uses.md, 1, and domain/accounts.md, 5).
use super::{deployment, owner, pieces, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_accounts as a;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
pub fn facts(value: &List<d::AccountFactLine>, _limits: &w::Limits) -> Result<List<w::AccountFact>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for line in value {
        let fact = &line.fact;
        let value = match &fact.value {
            a::FactValue::Text(text) => w::AccountFactValue::Text(w::AccountFactText { text: text.clone() }),
            a::FactValue::Count(count) => w::AccountFactValue::Count(w::AccountFactCount { count: *count }),
            a::FactValue::Instant(at) => w::AccountFactValue::Instant(w::AccountFactInstant { at: at.as_nanos() }),
            a::FactValue::Span(nanos) => w::AccountFactValue::Span(w::AccountFactSpan { nanos: *nanos }),
            a::FactValue::Share(basis) => w::AccountFactValue::Share(w::AccountFactShare { basis: *basis }),
            a::FactValue::Window(window) => w::AccountFactValue::Window(w::AccountWindow {
                rate: line.rate,
                length: window.length,
                used: window.used,
                resets: match window.resets {
                    Some(at) => Some(at.as_nanos()),
                    None => None,
                },
                state: window.state,
            }),
        };
        let row = w::AccountFact {
            kind: fact.kind,
            label: fact.label.clone(),
            value,
            seen: fact.seen.as_nanos(),
            source: fact.source,
            origin: fact.origin,
        };
        rows.push(row).expect("supplied facts fit");
    }
    Ok(rows)
}
pub fn payer(value: a::Use, _limits: &w::Limits) -> Result<w::AccountUse, w::Problem> {
    Ok(match value {
        a::Use::Party(party) => w::AccountUse::Party(w::AccountUseParty { party }),
        a::Use::Project(project) => w::AccountUse::Project(w::AccountUseProject { project }),
    })
}
pub fn holder(value: a::Holder, _limits: &w::Limits) -> Result<w::AccountHolder, w::Problem> {
    Ok(match value {
        a::Holder::Deployment => w::AccountHolder::Deployment,
        a::Holder::Party(party) => w::AccountHolder::Party(w::AccountHolderParty { party }),
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
pub fn refusal(value: a::Refusal, _limits: &w::Limits) -> Result<w::AccountRefusal, w::Problem> {
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
        a::Refusal::NotKept { why } => w::AccountRefusal::NotKept(w::AccountRefusalNotKept { why: key_failure(why) }),
        a::Refusal::Limit { bound } => w::AccountRefusal::Limit(w::AccountRefusalLimit { bound: self::bound(bound) }),
        a::Refusal::Held { provider } => w::AccountRefusal::Held(w::AccountRefusalHeld { provider }),
        a::Refusal::Provider { provider } => w::AccountRefusal::Provider(w::AccountRefusalProvider { provider }),
        a::Refusal::Subject => w::AccountRefusal::Subject,
        a::Refusal::Numbers => w::AccountRefusal::Numbers,
        a::Refusal::Kind { kind } => w::AccountRefusal::Kind(w::AccountRefusalKind { kind: self::kind(kind) }),
        a::Refusal::Closing => w::AccountRefusal::Closing,
        a::Refusal::NotStarted => w::AccountRefusal::NotStarted,
    })
}
pub fn failure(value: a::Failure, _limits: &w::Limits) -> Result<w::AccountFailure, w::Problem> {
    Ok(match value {
        a::Failure::Later { after } => w::AccountFailure::Later(w::AccountFailureLater { after }),
        a::Failure::SignInAgain => w::AccountFailure::SignInAgain,
        a::Failure::MissingScope { scope } => w::AccountFailure::MissingScope(w::AccountFailureMissingScope { scope }),
        a::Failure::Misconfigured => w::AccountFailure::Misconfigured,
        a::Failure::Unloaded { at, why } => w::AccountFailure::Unloaded(w::AccountFailureUnloaded {
            at: deployment::account_place(at),
            why: deployment::account_unread(why),
        }),
        a::Failure::Cancelled => w::AccountFailure::Cancelled,
    })
}
pub fn end(value: a::End, limits: &w::Limits) -> Result<w::AccountEnd, w::Problem> {
    Ok(match value {
        a::End::Made { account, declined } => w::AccountEnd::Made(w::AccountEndMade { account, declined }),
        a::End::Renewed { account, declined } => w::AccountEnd::Renewed(w::AccountEndRenewed { account, declined }),
        a::End::Refused { refusal } => {
            w::AccountEnd::Refused(w::AccountEndRefused { refusal: self::refusal(refusal, limits)? })
        }
        a::End::Failed { failure } => {
            w::AccountEnd::Failed(w::AccountEndFailed { failure: self::failure(failure, limits)? })
        }
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
    Ok(w::AccountSource {
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
    })
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
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).expect("measured account source");
        encoded.push(out.finish()).expect("supplied source pieces fit");
    }
    pieces(&encoded, bound, path)
}
pub fn line(value: &d::AccountLine, limits: &w::Limits) -> Result<w::AccountLine, w::Problem> {
    Ok(w::AccountLine {
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
        sources: source_pieces(&value.sources, limits, limits.account_line_sources, w::paths::ACCOUNT_LINE_SOURCES)?,
        cut: value.cut,
    })
}
pub fn document(value: &d::Accounts, limits: &w::Limits) -> Result<w::Accounts, w::Problem> {
    let mut accounts = List::with_capacity(value.accounts.len());
    for row in &value.accounts {
        let row = line(row, limits)?;
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).expect("measured account line");
        accounts.push(out.finish()).expect("supplied account pieces fit");
    }
    let accounts = pieces(&accounts, limits.accounts_accounts, w::paths::ACCOUNTS_ACCOUNTS)?;
    let mut signing = List::with_capacity(value.signing.len());
    for row in &value.signing {
        let row = w::AccountSignIn { source: row.source, provider: row.provider, holder: holder(row.holder, limits)? };
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).expect("measured sign-in line");
        signing.push(out.finish()).expect("supplied sign-in pieces fit");
    }
    let signing = pieces(&signing, limits.accounts_signing, w::paths::ACCOUNTS_SIGNING)?;
    let last_end = match &value.last_end {
        Some(row) => Some(w::AccountLastEnd { source: row.source, end: end(row.end, limits)? }),
        None => None,
    };
    Ok(w::Accounts {
        lendable: value.lendable,
        usable: value.usable,
        settled: value.settled,
        cut: value.cut,
        readiness_cut: value.readiness_cut,
        lending: match &value.lending {
            Some(login) => Some(terminal_login(login, limits)?),
            None => None,
        },
        lent_end: match value.lent_end {
            Some(ended) => Some(w::AccountLentEnd { number: ended.number, end: login_end(ended.end, limits)? }),
            None => None,
        },
        readiness: super::catalogue::readiness(&value.readiness, limits.accounts_readiness, limits)?,
        program_hosts: super::programs::hosts(&value.program_hosts, limits)?,
        accounts,
        signing,
        last_end,
    })
}

/// Public history only, without the child's model-name bits.
pub fn list(value: Option<&d::AccountList>, _limits: &w::Limits) -> Result<Option<w::AccountList>, w::Problem> {
    let Some(value) = value else {
        return Ok(None);
    };
    let failure = match value.failure {
        Some(jig_domain_accounts::ReadFailure::Refused { status }) => {
            Some(w::AccountListFailure::Refused(w::AccountListRefused { status }))
        }
        Some(jig_domain_accounts::ReadFailure::Unreachable) => Some(w::AccountListFailure::Unreachable),
        Some(jig_domain_accounts::ReadFailure::Unreadable) => Some(w::AccountListFailure::Unreadable),
        Some(jig_domain_accounts::ReadFailure::Deadline) => Some(w::AccountListFailure::Deadline),
        Some(jig_domain_accounts::ReadFailure::Agent) => Some(w::AccountListFailure::Agent),
        Some(jig_domain_accounts::ReadFailure::Room) => Some(w::AccountListFailure::Room),
        None => None,
    };
    Ok(Some(w::AccountList {
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
    }))
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn terminal_login(value: &a::TerminalLogin, _limits: &w::Limits) -> Result<w::TerminalLogin, w::Problem> {
    Ok(w::TerminalLogin {
        number: value.number,
        provider: value.provider,
        name: value.name.clone(),
        cancelled: value.cancelled,
    })
}
pub(crate) fn terminal(value: Option<&a::TerminalLogin>, limits: &w::Limits) -> Result<w::Terminal, w::Problem> {
    Ok(w::Terminal {
        login: match value {
            Some(login) => Some(terminal_login(login, limits)?),
            None => None,
        },
    })
}
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn login_exit(value: a::LoginExit, _limits: &w::Limits) -> Result<w::LoginExit, w::Problem> {
    Ok(match value {
        a::LoginExit::Exited { code } => w::LoginExit::Exited(w::LoginExitExited { code }),
        a::LoginExit::Killed { signal } => w::LoginExit::Killed(w::LoginExitKilled { signal }),
        a::LoginExit::NotStarted => w::LoginExit::NotStarted,
    })
}
fn login_end(value: a::LoginEnd, limits: &w::Limits) -> Result<w::LoginEnd, w::Problem> {
    Ok(match value {
        a::LoginEnd::SignedIn { source } => w::LoginEnd::SignedIn(w::LoginEndSignedIn { source }),
        a::LoginEnd::NotFinished { exit } => w::LoginEnd::NotFinished(w::LoginEndNotFinished {
            exit: match exit {
                Some(exit) => Some(login_exit(exit, limits)?),
                None => None,
            },
        }),
        a::LoginEnd::Cancelled => w::LoginEnd::Cancelled,
        a::LoginEnd::Refused { refusal: why } => {
            w::LoginEnd::Refused(w::LoginEndRefused { refusal: refusal(why, limits)? })
        }
    })
}
