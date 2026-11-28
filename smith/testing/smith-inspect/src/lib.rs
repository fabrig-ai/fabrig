//! What smith holds at a quiet point: its waits, bounded tables, reserved
//! room and its side of the host-call relation (smith/docs/design/testing.md, 6).
//! These reads change no domain state and retain no observations of earlier points.

mod room;
mod session;

pub use room::{RoomSums, check_room_sum, room, room_sums};

use skein_lib::Id;
use skein_world::inspect::{Entry, Inspection, Related, Release, Table};
use smith_domain_session::InspectionState;

const PROVIDER: &str = "a session's call waiting on its provider's answer";
const RATE: &str = "a retry waiting out a rate window";
const BACKOFF: &str = "a retry waiting out a transient failure's backoff";
const TOOL: &str = "a run waiting on a tool's end";
const JOB: &str = "a run waiting on a job's end";

/// The kinds of waits smith reads, for an inspection of component `smith`.
pub const KINDS: &[&str] = &[PROVIDER, RATE, BACKOFF, TOOL, JOB];

/// Read the root and every child's waits and tables.
pub fn inspect(domain: &smith_domain::Domain, into: &mut Inspection) {
    let run = domain.inspection_run();
    let sessions = domain.inspection_session();
    session::records(sessions, Some(run), into);
    let runs = run.inspection_runs();
    into.table(table(
        "runs",
        runs.capacity(),
        runs.iter()
            .map(|(_, run)| {
                entry(named("run ", run.inspection_host_name().raw()), "the run's end", run.inspection_closed())
            })
            .collect(),
    ));

    let issued = domain.inspection_completions();
    into.table(table(
        "issued provider requests",
        issued.capacity(),
        issued
            .iter()
            .map(|(owner, _)| {
                let due = sessions.inspection_sessions().get(Id::from_token(*owner)).is_none_or(|session| {
                    !matches!(
                        session.inspection_state(),
                        InspectionState::Completing { .. } | InspectionState::Cancelling
                    )
                });
                entry(named("session ", owner.raw()), "the request's answer or deadline", due)
            })
            .collect(),
    ));
    let flights = domain.inspection_flights();
    into.table(table(
        "delegated requests in flight",
        flights.capacity(),
        flights
            .iter()
            .map(|(owner, flight)| {
                // An answer on the ready list still owns its handoff until delivered.
                let rule =
                    if flight.inspection_answered() { "delivery of the call's result" } else { "the call's result" };
                let due = sessions
                    .inspection_calls()
                    .get(Id::from_token(*owner))
                    .and_then(smith_domain_session::InspectionCall::inspection_call)
                    .is_none();
                entry(named("call ", owner.raw()), rule, due)
            })
            .collect(),
    ));
}

/// Read the session child alone, naming each run by its opener's number.
pub fn session(domain: &smith_domain_session::Domain, into: &mut Inspection) {
    session::records(domain, None, into);
}

/// Record each host relay still awaiting a terminal under the run's task key.
pub fn host_calls(domain: &smith_domain::Domain, task: u64, into: &mut Inspection) {
    for (_, call) in domain.inspection_run().inspection_calls() {
        if let Some(name) = call.inspection_host_call() {
            let (attempt, completion, position) = (name.activation, name.completion, name.position);
            into.related(Related {
                relation: "a run's host calls",
                key: format!("task {task} attempt {attempt} completion {completion} position {position}"),
                holds: "an open call",
            });
        }
    }
}

fn table(name: &'static str, capacity: u32, entries: Vec<Entry>) -> Table {
    Table { name, capacity: u64::from(capacity), entries, refused: Vec::new() }
}

fn entry(key: String, rule: &'static str, due: bool) -> Entry {
    Entry { key, rule, release: if due { Release::Due } else { Release::Pending } }
}

// Each identity fits once, including a slab token with a generation in its high bits.
fn named(prefix: &str, number: u64) -> String {
    use std::fmt::Write as _;
    let mut key = String::with_capacity(prefix.len() + 20);
    key.push_str(prefix);
    let _written = write!(key, "{number}");
    key
}
