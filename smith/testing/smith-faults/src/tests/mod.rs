//! Hand-made observations exercise the blame table without a world's scheduling.

mod answers;
mod runs;

use skein_lib::{Duration, Time};
use skein_world::faults::{FaultLedger, Injection, Kind, Struck};

use crate::{BILLING_REFUSAL, CHECK_WAITS, EDGE_REFUSAL, UNTOOLED_PROGRAM, WINDOW_ELSEWHERE};

fn time(seconds: u64) -> Time {
    Time::ZERO.checked_add(Duration::from_secs(seconds)).expect("test time fits")
}

fn inject(ledger: &mut FaultLedger, kind: Kind, from: u64, until: Option<u64>, account: &str) -> Injection {
    let injection = ledger.inject(kind, time(from), Struck { account: Some(account.into()), ..Struck::default() });
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
