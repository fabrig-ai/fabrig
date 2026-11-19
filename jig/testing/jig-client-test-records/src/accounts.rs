//! Independent literals for complete accounts watch records.
use jig_client_protocol_wire::Record as _;
use super::live_values::{pieces, rows};
use super::{w, wire_text};
use skein_lib::{Duration, List};

#[must_use]
pub fn account_sources_record() -> Box<[u8]> {
    let sources = [
        w::AccountSource {
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
        w::AccountSource {
                name: None,
                place: 5,
                number: 12,
                kind: w::AccountKind::Key,
                state: w::AccountState::Later(
                    w::AccountStateLater { until: None },
                ),
                usable: false,
                expiring: true,
                attention: false,
                spent_until: None,
            },
    ];
    pieces(sources.iter().map(|s| s.encode(&crate::limits()).expect("literal record encodes")))
}
#[must_use]
pub fn account_line_record() -> w::AccountLine {
    w::AccountLine {
            list: None,
            facts: account_fact_records(),
            provider_name: None,
            manage_usage: None,
            number: 7,
            place: 3,
            uses: Some(rows([
                w::AccountUse::Party(
                    w::AccountUseParty { party: 23 },
                ),
                w::AccountUse::Project(
                    w::AccountUseProject { project: 29 },
                ),
            ])),
            plan: true,
            provider: Some(3),
            display: w::DocumentText { bytes: Box::from(b"Personal".as_slice()), cut: true },
            holder: w::AccountHolder::Party(
                w::AccountHolderParty { party: 23 },
            ),
            sources: account_sources_record(),
            cut: true,
        }
}
#[must_use]
pub fn accounts_record(end: Option<w::AccountEnd>) -> w::Accounts {
    let line = account_line_record();
    let other = w::AccountLine {
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
        };
    let signing = w::AccountSignIn {
            source: 19,
            provider: 3,
            holder: w::AccountHolder::Party(
                w::AccountHolderParty { party: 23 },
            ),
        };
    w::Accounts {
            lendable: false,
            usable: true,
            settled: false,
            cut: true,
            readiness_cut: false,
            readiness: Box::new([]),
            program_hosts: Box::new([]),
            accounts: pieces([
                line.encode(&crate::limits()).expect("literal record encodes"),
                other.encode(&crate::limits()).expect("literal record encodes"),
            ]),
            signing: pieces([signing.encode(&crate::limits()).expect("literal record encodes")]),
            lending: None,
            lent_end: None,
            last_end: end
                .map(|end| w::AccountLastEnd { source: 11, end }),
        }
}
#[must_use]
pub fn account_end_records() -> Vec<w::AccountEnd> {
    let mut ends = vec![
        w::AccountEnd::Made(
            w::AccountEndMade { account: 7, declined: false },
        ),
        w::AccountEnd::Renewed(
            w::AccountEndRenewed { account: 7, declined: true },
        ),
    ];
    let refusals = [
        w::AccountRefusal::NotLimited,
        w::AccountRefusal::Uses,
        w::AccountRefusal::NotKept(
            w::AccountRefusalNotKept { why: w::AccountKeyFailure::NotStored },
        ),
        w::AccountRefusal::Limit(
            w::AccountRefusalLimit { bound: w::AccountBound::HolderAccounts },
        ),
        w::AccountRefusal::Held(
            w::AccountRefusalHeld { provider: 3 },
        ),
        w::AccountRefusal::Provider(
            w::AccountRefusalProvider { provider: 3 },
        ),
        w::AccountRefusal::Subject,
        w::AccountRefusal::Unknown,
        w::AccountRefusal::Numbers,
        w::AccountRefusal::Kind(
            w::AccountRefusalKind { kind: w::AccountKind::Program },
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
            w::AccountEndRefused { refusal },
        ));
    }
    let failures = [
        w::AccountFailure::Later(
            w::AccountFailureLater { after: Some(Duration::from_nanos(103)) },
        ),
        w::AccountFailure::Later(
            w::AccountFailureLater { after: None },
        ),
        w::AccountFailure::SignInAgain,
        w::AccountFailure::MissingScope(
            w::AccountFailureMissingScope { scope: 31 },
        ),
        w::AccountFailure::Misconfigured,
        w::AccountFailure::Unloaded(
            w::AccountFailureUnloaded { at: w::AccountPlace::File, why: w::AccountUnread::Stalled },
        ),
        w::AccountFailure::Cancelled,
    ];
    for failure in failures {
        ends.push(w::AccountEnd::Failed(
            w::AccountEndFailed { failure },
        ));
    }
    ends
}

/// A full list at the schema's bound, carrying each fact value shape.
#[must_use]
pub fn account_fact_records() -> List<w::AccountFact> {
    let mut facts = List::with_capacity(24);
    for _ in 0_u32..4 {
        for value in [
            w::AccountFactValue::Text(
                w::AccountFactText { text: Box::from(b"you@example.com".as_slice()) },
            ),
            w::AccountFactValue::Count(
                w::AccountFactCount { count: 123 },
            ),
            w::AccountFactValue::Instant(
                w::AccountFactInstant { at: 127 },
            ),
            w::AccountFactValue::Span(
                w::AccountFactSpan { nanos: Duration::from_nanos(131) },
            ),
            w::AccountFactValue::Share(
                w::AccountFactShare { basis: 10_000 },
            ),
            w::AccountFactValue::Window(
                w::AccountWindow {
                        length: Some(Duration::from_nanos(137)),
                        used: Some(10_000),
                        resets: Some(139),
                        state: 2,
                    },
            ),
        ] {
            facts
                .push(
                    w::AccountFact {
                            kind: 1,
                            label: Box::from(b"label".as_slice()),
                            value,
                            seen: 149,
                            source: Some(11),
                            origin: 6,
                        },
                )
                .expect("24 facts");
        }
    }
    facts
}
