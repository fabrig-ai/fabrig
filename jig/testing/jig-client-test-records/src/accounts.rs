//! Independent literals for complete accounts watch records.
use super::live_values::{encoded, pieces, rows};
use super::{limits, w, wire_text};
use skein_lib::{Duration, List};

#[must_use]
pub fn account_sources_record() -> Box<[u8]> {
    let l = limits();
    let sources = [
        w::AccountSource::new(
            &l,
            w::AccountSourceParts {
                name: None,
                place: 2,
                number: 11,
                kind: w::AccountKind::SignIn,
                state: w::AccountState::Granted,
                usable: true,
                expiring: false,
                attention: true,
                spent_until: Some(101),
            },
        )
        .expect("literal"),
        w::AccountSource::new(
            &l,
            w::AccountSourceParts {
                name: None,
                place: 5,
                number: 12,
                kind: w::AccountKind::Key,
                state: w::AccountState::Later(
                    w::AccountStateLater::new(&l, w::AccountStateLaterParts { until: None }).expect("literal"),
                ),
                usable: false,
                expiring: true,
                attention: false,
                spent_until: None,
            },
        )
        .expect("literal"),
    ];
    pieces(sources.iter().map(|s| encoded(s.measure(), |w| s.encode(w))))
}
#[must_use]
pub fn account_line_record() -> w::AccountLine {
    let l = limits();
    w::AccountLine::new(
        &l,
        w::AccountLineParts {
            list: None,
            facts: account_fact_records(),
            provider_name: None,
            manage_usage: None,
            number: 7,
            place: 3,
            uses: Some(rows([
                w::AccountUse::Party(
                    w::AccountUseParty::new(&l, w::AccountUsePartyParts { party: 23 }).expect("literal"),
                ),
                w::AccountUse::Project(
                    w::AccountUseProject::new(&l, w::AccountUseProjectParts { project: 29 }).expect("literal"),
                ),
            ])),
            plan: true,
            provider: Some(3),
            display: w::DocumentText::new(
                &l,
                w::DocumentTextParts { bytes: Box::from(b"Personal".as_slice()), cut: true },
            )
            .expect("literal"),
            holder: w::AccountHolder::Party(
                w::AccountHolderParty::new(&l, w::AccountHolderPartyParts { party: 23 }).expect("literal"),
            ),
            sources: account_sources_record(),
            cut: true,
        },
    )
    .expect("literal")
}
#[must_use]
pub fn accounts_record(end: Option<w::AccountEnd>) -> w::Accounts {
    let l = limits();
    let line = account_line_record();
    let other = w::AccountLine::new(
        &l,
        w::AccountLineParts {
            list: None,
            facts: List::with_capacity(0),
            provider_name: None,
            manage_usage: None,
            number: 9,
            place: 4,
            uses: None,
            plan: false,
            provider: None,
            display: wire_text(b""),
            holder: w::AccountHolder::Deployment,
            sources: Box::from([]),
            cut: false,
        },
    )
    .expect("literal");
    let signing = w::AccountSignIn::new(
        &l,
        w::AccountSignInParts {
            source: 19,
            provider: 3,
            holder: w::AccountHolder::Party(
                w::AccountHolderParty::new(&l, w::AccountHolderPartyParts { party: 23 }).expect("literal"),
            ),
        },
    )
    .expect("literal");
    w::Accounts::new(
        &l,
        w::AccountsParts {
            skein_bools: [false, true, false, true, false],
            readiness: Box::new([]),
            program_hosts: Box::new([]),
            accounts: pieces([
                encoded(line.measure(), |w| line.encode(w)),
                encoded(other.measure(), |w| other.encode(w)),
            ]),
            signing: pieces([encoded(signing.measure(), |w| signing.encode(w))]),
            lending: None,
            lent_end: None,
            last_end: end
                .map(|end| w::AccountLastEnd::new(&l, w::AccountLastEndParts { source: 11, end }).expect("literal")),
        },
    )
    .expect("literal")
}
#[must_use]
pub fn account_end_records() -> Vec<w::AccountEnd> {
    let l = limits();
    let mut ends = vec![
        w::AccountEnd::Made(
            w::AccountEndMade::new(&l, w::AccountEndMadeParts { account: 7, declined: false }).expect("literal"),
        ),
        w::AccountEnd::Renewed(
            w::AccountEndRenewed::new(&l, w::AccountEndRenewedParts { account: 7, declined: true }).expect("literal"),
        ),
    ];
    let refusals = [
        w::AccountRefusal::NotLimited,
        w::AccountRefusal::Uses,
        w::AccountRefusal::NotKept(
            w::AccountRefusalNotKept::new(&l, w::AccountRefusalNotKeptParts { why: w::AccountKeyFailure::NotStored })
                .expect("literal"),
        ),
        w::AccountRefusal::Limit(
            w::AccountRefusalLimit::new(&l, w::AccountRefusalLimitParts { bound: w::AccountBound::HolderAccounts })
                .expect("literal"),
        ),
        w::AccountRefusal::Held(
            w::AccountRefusalHeld::new(&l, w::AccountRefusalHeldParts { provider: 3 }).expect("literal"),
        ),
        w::AccountRefusal::Provider(
            w::AccountRefusalProvider::new(&l, w::AccountRefusalProviderParts { provider: 3 }).expect("literal"),
        ),
        w::AccountRefusal::Subject,
        w::AccountRefusal::Unknown,
        w::AccountRefusal::Numbers,
        w::AccountRefusal::Kind(
            w::AccountRefusalKind::new(&l, w::AccountRefusalKindParts { kind: w::AccountKind::Program })
                .expect("literal"),
        ),
        w::AccountRefusal::Closing,
        w::AccountRefusal::NotStarted,
        w::AccountRefusal::NotSignedIn,
        w::AccountRefusal::NotOurs,
        w::AccountRefusal::Cancelled,
        w::AccountRefusal::Moving,
    ];
    for refusal in refusals {
        ends.push(w::AccountEnd::Refused(
            w::AccountEndRefused::new(&l, w::AccountEndRefusedParts { refusal }).expect("literal"),
        ));
    }
    let failures = [
        w::AccountFailure::Later(
            w::AccountFailureLater::new(&l, w::AccountFailureLaterParts { after: Some(Duration::from_nanos(103)) })
                .expect("literal"),
        ),
        w::AccountFailure::Later(
            w::AccountFailureLater::new(&l, w::AccountFailureLaterParts { after: None }).expect("literal"),
        ),
        w::AccountFailure::SignInAgain,
        w::AccountFailure::MissingScope(
            w::AccountFailureMissingScope::new(&l, w::AccountFailureMissingScopeParts { scope: 31 }).expect("literal"),
        ),
        w::AccountFailure::Misconfigured,
        w::AccountFailure::Unloaded(
            w::AccountFailureUnloaded::new(
                &l,
                w::AccountFailureUnloadedParts { at: w::AccountPlace::File, why: w::AccountUnread::Stalled },
            )
            .expect("literal"),
        ),
        w::AccountFailure::Cancelled,
    ];
    for failure in failures {
        ends.push(w::AccountEnd::Failed(
            w::AccountEndFailed::new(&l, w::AccountEndFailedParts { failure }).expect("literal"),
        ));
    }
    ends
}

/// A full list at the schema's bound, carrying each fact value shape.
#[must_use]
pub fn account_fact_records() -> List<w::AccountFact> {
    let l = limits();
    let mut facts = List::with_capacity(24);
    for _ in 0_u32..4 {
        for value in [
            w::AccountFactValue::Text(
                w::AccountFactText::new(&l, w::AccountFactTextParts { text: Box::from(b"you@example.com".as_slice()) })
                    .expect("literal"),
            ),
            w::AccountFactValue::Count(
                w::AccountFactCount::new(&l, w::AccountFactCountParts { count: 123 }).expect("literal"),
            ),
            w::AccountFactValue::Instant(
                w::AccountFactInstant::new(&l, w::AccountFactInstantParts { at: 127 }).expect("literal"),
            ),
            w::AccountFactValue::Span(
                w::AccountFactSpan::new(&l, w::AccountFactSpanParts { nanos: Duration::from_nanos(131) })
                    .expect("literal"),
            ),
            w::AccountFactValue::Share(
                w::AccountFactShare::new(&l, w::AccountFactShareParts { basis: 10_000 }).expect("literal"),
            ),
            w::AccountFactValue::Window(
                w::AccountWindow::new(
                    &l,
                    w::AccountWindowParts {
                        length: Some(Duration::from_nanos(137)),
                        used: Some(10_000),
                        resets: Some(139),
                        state: 2,
                    },
                )
                .expect("literal"),
            ),
        ] {
            facts
                .push(
                    w::AccountFact::new(
                        &l,
                        w::AccountFactParts {
                            kind: 1,
                            label: Box::from(b"label".as_slice()),
                            value,
                            seen: 149,
                            source: Some(11),
                            origin: 6,
                        },
                    )
                    .expect("literal"),
                )
                .expect("24 facts");
        }
    }
    facts
}
