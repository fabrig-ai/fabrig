//! Hand-made observations exercise the blame table without a world's scheduling.

mod answers;
mod runs;

use skein_lib::{Duration, Time};
use skein_world::faults::{FaultLedger, Injection, Kind, Struck};

use crate::{BILLING_REFUSAL, CHECK_WAITS, EDGE_REFUSAL, UNTOOLED_PROGRAM, WINDOW_ELSEWHERE, parks};

fn time(seconds: u64) -> Time {
    Time::ZERO.checked_add(Duration::from_secs(seconds)).expect("test time fits")
}

fn inject(ledger: &mut FaultLedger, kind: Kind, from: u64, until: Option<u64>, account: &str) -> Injection {
    let injection = ledger.inject(
        kind,
        time(from),
        Struck {
            account: Some(account.into()),
            request: (kind == skein_llm_kit::faults::RATE_WINDOW).then(|| "run 1 call 2".into()),
            ..Struck::default()
        },
    );
    if let Some(until) = until {
        ledger.stop(injection, time(until)).expect("test interval is ordered and stopped once");
    }
    injection
}

#[test]
fn smith_kinds_name_their_family_and_fault() {
    for (kind, family, name) in [
        (BILLING_REFUSAL, "provider", "billing refusal"),
        (EDGE_REFUSAL, "provider", "edge refusal"),
        (WINDOW_ELSEWHERE, "provider", "window elsewhere"),
        (UNTOOLED_PROGRAM, "worker", "untooled program"),
        (CHECK_WAITS, "worker", "check waits for a place"),
    ] {
        assert_eq!(kind, Kind { family, name });
    }
}

#[test]
fn park_kinds_have_their_contract_names_family_and_order() {
    let family = "a session's state at a park";
    assert_eq!(parks::FAMILY, family);
    let expected = [
        (parks::WITHHELD_WAIT, "a withheld wait"),
        (parks::JOB_RUNNING, "a job running"),
        (parks::CUT_TURN, "a cut turn with kept parts"),
        (parks::HOST_CALL_ANSWERED, "a host call answered"),
        (parks::HOST_CALL_UNANSWERED, "a host call unanswered"),
        (parks::SUB_AGENT_KEPT, "a sub-agent's conversation kept"),
    ];
    assert_eq!(parks::KINDS, expected.map(|(kind, _)| kind));
    for (kind, name) in expected {
        assert_eq!(kind, Kind { family, name });
    }
}
