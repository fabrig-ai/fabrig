//! The session child's own records. A composed world uses the host's run
//! names; a world holding only the child uses each opener's number.

use std::fmt::Write as _;

use skein_lib::{Id, Time, Token};
use skein_world::inspect::{Inspection, Release, Wake};
use smith_domain::run;
use smith_domain_session::{Domain, InspectionSession, InspectionState, ToolCall, ToolSource};

use crate::{BACKOFF, JOB, PROVIDER, RATE, TOOL, named};

pub(super) fn records(domain: &Domain, runs: Option<&run::Domain>, into: &mut Inspection) {
    let sessions = domain.inspection_sessions();
    let capacity = u64::from(sessions.capacity());
    into.fill_table("sessions", capacity, |rows| {
        for (id, session) in sessions {
            let release = release(matches!(session.inspection_state(), InspectionState::Closed));
            rows.entry("the run's end", release, |key| name(key, "session ", id.token().raw()));
        }
    });
    into.fill_table("requests in flight", capacity, |rows| {
        for (id, session) in sessions {
            let rule = match session.inspection_state() {
                InspectionState::Completing { .. } => "the request's answer or deadline",
                InspectionState::Cancelling => "the request's terminal after cancellation",
                InspectionState::Backoff { .. } | InspectionState::Other | InspectionState::Closed => continue,
            };
            rows.entry(rule, Release::Pending, |key| name(key, "session ", id.token().raw()));
        }
    });
    for (id, session) in sessions {
        let opener = session.inspection_opener();
        let run = runs.and_then(|runs| run::host_name(runs, opener)).unwrap_or(opener);
        match session.inspection_state() {
            InspectionState::Completing { deadline } => party_wait(into, run, PROVIDER, "provider", Some(deadline)),
            InspectionState::Cancelling => party_wait(into, run, PROVIDER, "provider cancellation terminal", None),
            InspectionState::Backoff { until, rate_limited } => {
                let kind = if rate_limited { RATE } else { BACKOFF };
                into.fill_wait(|wait| {
                    wait.waiter.clear();
                    name(&mut wait.waiter, "run ", run.raw());
                    wait.kind.clear();
                    wait.kind.push_str(kind);
                    wait.bound = Some(until);
                    wait.wakes.clear();
                    if let Some(wake) = domain.inspection_retry(id) {
                        wait.wakes.push(Wake::Timer(wake));
                    }
                });
            }
            InspectionState::Other | InspectionState::Closed => {}
        }
        open_calls(domain, run, id, session, into);
    }
    tools(domain.inspection_tools(), into);
}

fn open_calls(
    domain: &Domain,
    run: Token,
    owner: Id<InspectionSession>,
    session: &InspectionSession,
    into: &mut Inspection,
) {
    let session_number = owner.token().raw();
    let program = session.inspection_program();
    let live = || {
        domain.inspection_calls().iter().filter_map(|(_, call)| {
            if call.inspection_session() != owner {
                return None;
            }
            call.inspection_call()
        })
    };
    let claims = || {
        program.iter().filter_map(|(_, claim)| {
            claim.inspection_receiving()?;
            Some(claim.inspection_call())
        })
    };
    let capacity = u64::from(domain.inspection_parallel_tools()) + u64::from(program.capacity());
    into.fill_table("open calls", capacity, |rows| {
        for call in live().chain(claims()) {
            rows.entry("the call's result", Release::Pending, |key| {
                let _written = write!(key, "session {session_number} call {}", call.owner.raw());
            });
        }
    });
    for call in live() {
        let party = match call.source {
            ToolSource::Workspace => "tool IO",
            ToolSource::Run => "opener",
            ToolSource::Host => "host",
        };
        party_wait(into, run, tool_wait_kind(call), party, None);
    }
    for call in claims() {
        party_wait(into, run, tool_wait_kind(call), "provider", None);
    }
}

fn tool_wait_kind(call: &ToolCall) -> &'static str {
    if call.source == ToolSource::Workspace && call.name.as_ref() == b"job" { JOB } else { TOOL }
}

fn party_wait(into: &mut Inspection, run: Token, kind: &str, party: &str, bound: Option<Time>) {
    into.fill_wait(|wait| {
        wait.waiter.clear();
        name(&mut wait.waiter, "run ", run.raw());
        wait.kind.clear();
        wait.kind.push_str(kind);
        wait.bound = bound;
        wait.wakes.truncate(1);
        if let Some(Wake::Party(name)) = wait.wakes.first_mut() {
            name.clear();
            name.push_str(party);
        } else {
            wait.wakes.clear();
            wait.wakes.push(Wake::Party(party.into()));
        }
    });
}

fn tools(domain: &smith_domain_tools::Domain, into: &mut Inspection) {
    let kits = domain.inspection_kits();
    let flights = domain.inspection_flights();
    into.fill_table("kits", u64::from(kits.capacity()), |rows| {
        for (id, kit) in kits {
            rows.entry("the run's end", release(kit.inspection_closed()), |key| name(key, "kit ", id.token().raw()));
        }
    });
    for (id, kit) in kits {
        let kit_number = id.token().raw();
        let closed = kit.inspection_closed();
        let jobs = kit.inspection_jobs();
        let held = jobs.inspection_held();
        into.fill_table("job cells", u64::from(held.capacity()), |rows| {
            for owner in jobs.inspection_reserved() {
                let due = closed
                    || flights
                        .get(*owner)
                        .is_none_or(|flight| flight.inspection_kit() != id || !flight.inspection_yielding());
                rows.entry("the yielding call's terminal or transfer to its job", release(due), |key| {
                    let _written = write!(key, "kit {kit_number} yielding call {}", owner.token().raw());
                });
            }
            for (number, _) in held {
                rows.entry("the job's terminal read or its session's close", release(closed), |key| {
                    let _written = write!(key, "kit {kit_number} job {number}");
                });
            }
        });
    }
    // A command past its yield has answered its request; its job cell is read above.
    into.fill_table("tool requests in flight", u64::from(flights.capacity()), |rows| {
        for (id, flight) in flights {
            if flight.inspection_job().is_none() {
                rows.entry("the operation's terminal", release(flight.inspection_terminal()), |key| {
                    name(key, "operation ", id.token().raw());
                });
            }
        }
    });
}

pub(super) fn session_key(id: Id<InspectionSession>) -> String {
    named("session ", id.token().raw())
}

fn name(key: &mut String, prefix: &str, number: u64) {
    key.push_str(prefix);
    let _written = write!(key, "{number}");
}

fn release(due: bool) -> Release {
    if due { Release::Due } else { Release::Pending }
}
