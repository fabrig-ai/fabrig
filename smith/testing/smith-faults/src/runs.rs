//! Recorded observations and the retry/end table (`smith/docs/design/faults.md`, 4).

use std::collections::BTreeMap;

use skein_lib::Time;
use skein_llm_kit::faults::{
    CUT_BEFORE_OUTPUT, CUT_MID_CALL, CUT_MID_TEXT, EXPIRED_SIGN_IN, HANG_AFTER_HEAD, IDLE_MID_STREAM, RATE_WINDOW,
    SPENT_CREDITS, UNSERVED_MODEL,
};
use skein_world::faults::{Counter, FaultLedger, Injected, Kind, Struck};

use crate::{BILLING_REFUSAL, CHECK_WAITS, EDGE_REFUSAL, Ended, UNTOOLED_PROGRAM, WINDOW_ELSEWHERE};

/// Which layer made an attempt after the call's first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retried {
    /// The session retried a completion.
    Session,
    /// A program retried a provider request.
    Program,
}

impl Retried {
    fn counter(self) -> Counter {
        match self {
            Self::Session => Counter::SessionRetries,
            Self::Program => Counter::ProviderRetries,
        }
    }
}

#[derive(Debug)]
struct Run {
    from: Time,
    name: String,
    account: String,
    end: Option<(Time, Ended)>,
}

#[derive(Debug)]
struct Retry {
    at: Time,
    account: String,
    call: String,
    by: Retried,
}

/// The starts, retries and ends observed by one world, before its blame check.
#[derive(Default, Debug)]
pub struct Runs {
    runs: Vec<Run>,
    retries: Vec<Retry>,
}

impl Runs {
    /// Starts an empty set of observations.
    #[must_use]
    pub fn new() -> Runs {
        Self::default()
    }

    /// Records a run's start on its account.
    pub fn started(&mut self, at: Time, run: &str, account: &str) {
        self.runs.push(Run { from: at, name: run.into(), account: account.into(), end: None });
    }

    /// An attempt after a call's first, from its response record;
    /// `call` names the call across its attempts.
    pub fn retried(&mut self, at: Time, account: &str, call: &str, by: Retried) {
        self.retries.push(Retry { at, account: account.into(), call: call.into(), by });
    }

    /// Ends the named run on its account; the world must have recorded its start.
    pub fn ended(&mut self, at: Time, run: &str, account: &str, ended: Ended) {
        let started = self
            .runs
            .iter_mut()
            .find(|started| started.name == run && started.account == account && started.end.is_none())
            .expect("a run's end follows its start on the same account");
        assert!(at >= started.from, "a run's end does not precede its start");
        started.end = Some((at, ended));
    }

    /// Counts each retry against all faults striking it, then judges each fault's row.
    /// Call once for a world's ledger: counts are added even when findings are returned.
    pub fn judge(&self, ledger: &mut FaultLedger) -> Result<(), String> {
        let mut findings = Vec::new();
        let mut per_call = BTreeMap::new();
        for retry in &self.retries {
            let struck = Struck { account: Some(retry.account.clone()), ..Struck::default() };
            for injection in ledger.striking(retry.at, &struck) {
                // A window strike installs the account policy; only a refused
                // call's interval says that a retry is premature.
                let fault = ledger.get(injection);
                if fault.kind == RATE_WINDOW && fault.struck.request.is_none() {
                    continue;
                }
                ledger.count(injection, retry.by.counter());
                let count = per_call.entry((injection, &retry.call)).or_insert(0_u64);
                *count = count.checked_add(1).expect("a world's retries fit u64");
                let fault = ledger.get(injection);
                if let Some(row) = row(fault.kind) {
                    let excess = match row.retries {
                        Retries::None => true,
                        Retries::OnePerCall => *count > 1,
                        Retries::Any => false,
                    };
                    if excess {
                        findings.push(finding(fault, &format!("call {:?}: retry {count} is not allowed", retry.call)));
                    }
                }
            }
        }
        for run in &self.runs {
            if let Some((until, ended)) = run.end {
                // Latest start wins even if that fault has already stopped, or is
                // counted only. The ledger keeps closed intervals, including instants.
                let latest = ledger
                    .entries()
                    .iter()
                    .filter(|fault| {
                        fault.struck.account.as_deref() == Some(run.account.as_str())
                            && fault.from <= until
                            && fault.until.is_none_or(|stop| stop >= run.from)
                    })
                    .max_by_key(|fault| fault.from);
                if let Some(fault) = latest
                    && let Some(row) = row(fault.kind)
                    && (!row.ends.contains(&ended)
                        || (fault.kind == RATE_WINDOW
                            && ended == Ended::Limited
                            && !ledger.entries().iter().any(|refusal| {
                                refusal.kind == RATE_WINDOW
                                    && refusal.struck.account.as_deref() == Some(run.account.as_str())
                                    && refusal.struck.request.is_some()
                                    && refusal.from <= until
                                    && refusal.until.is_none_or(|reset| reset > until)
                            })))
                {
                    findings.push(finding(fault, &format!("run {:?}: end {ended:?} is not allowed", run.name)));
                }
            }
        }
        if findings.is_empty() { Ok(()) } else { Err(findings.join("\n")) }
    }
}

enum Retries {
    None,
    OnePerCall,
    Any,
}

struct Row {
    retries: Retries,
    ends: &'static [Ended],
}

#[expect(clippy::match_same_arms, reason = "the table names CHECK_WAITS explicitly as counted only beside open kinds")]
fn row(kind: Kind) -> Option<Row> {
    use Ended::{Completed, Limited, Outage, Refused, Unauthorized, Untooled};
    let row = match kind {
        RATE_WINDOW => Row { retries: Retries::None, ends: &[Completed, Limited] },
        SPENT_CREDITS => Row { retries: Retries::None, ends: &[Limited] },
        EXPIRED_SIGN_IN => Row { retries: Retries::OnePerCall, ends: &[Completed, Unauthorized] },
        UNSERVED_MODEL | BILLING_REFUSAL => Row { retries: Retries::None, ends: &[Refused] },
        EDGE_REFUSAL | HANG_AFTER_HEAD | IDLE_MID_STREAM => Row { retries: Retries::Any, ends: &[Completed, Outage] },
        CUT_BEFORE_OUTPUT | CUT_MID_TEXT | CUT_MID_CALL => Row { retries: Retries::Any, ends: &[Completed] },
        WINDOW_ELSEWHERE => Row { retries: Retries::None, ends: &[Completed] },
        UNTOOLED_PROGRAM => Row { retries: Retries::None, ends: &[Untooled] },
        // Kind is an open family/name pair. Today's classes and other trees'
        // kinds have no run-blame row; they still receive retry counts.
        CHECK_WAITS => return None,
        _ => return None,
    };
    Some(row)
}

fn finding(fault: &Injected, detail: &str) -> String {
    let until = fault.until.map_or_else(|| "ongoing".into(), |at| format!("{} ns", at.as_nanos()));
    format!("{}/{} [{} ns..{until}]: {detail}", fault.kind.family, fault.kind.name, fault.from.as_nanos())
}
