//! The host's run clock, played faults and response retries, kept independently
//! of program classification (`smith/docs/design/faults.md`, 3 and 4).

use std::collections::BTreeMap;

use skein_lib::{Duration, Reader, Rng, Time};
use skein_world::{
    faults::{FaultLedger, Kind, Struck},
    streams::{Stream, Streams},
};
use smith_channel::Record as _;
use smith_events::{Event, Outcome};
use smith_fake_claude::{End, Fault, Seen, Step, Turn};
use smith_faults::{Retried, Runs};

use super::{Case, Proc, Run};

/// Draw the single session's read turns from its workload generator.
#[must_use]
pub fn read_turns(workload: &mut Rng) -> Vec<Turn> {
    (0..workload.below(3))
        .map(|_| {
            Turn::new(vec![Step::Act { tool: "Read".into(), input: serde_json::json!({"file_path":"/work/repo/a"}) }])
        })
        .collect()
}

/// Draw one session's work independently of its provider fault.
#[must_use]
pub fn provider_draw(streams: Streams, faults: &mut Rng) -> (Vec<Turn>, Turn) {
    let mut workload = streams.rng(Stream::Workload, "runs");
    let turns = read_turns(&mut workload);
    let fault = match faults.below(3) {
        0 => Turn::new(vec![Step::RateLimit { resets_at: None }]),
        1 => Turn::new(vec![Step::LoseLogin, Step::Text("continue".into())]),
        2 => Turn { steps: vec![], end: End::edge_refusal() },
        _ => unreachable!("three faults"),
    };
    (turns, fault)
}

#[derive(Default)]
pub(super) struct Clock {
    origin: Option<Time>,
    started: Option<Time>,
    ended: Option<Time>,
    untooled: Option<Time>,
    account: Option<String>,
}

impl Clock {
    pub(super) fn observe(&mut self, now: Time, procs: &[Proc]) {
        if self.origin.is_none() && procs.iter().any(|proc| matches!(proc, Proc::Agent(_))) {
            // Inputs are observed before the processes' first iterate, which
            // supplies the event writer's process origin.
            self.origin = Some(now);
        }
        for proc in procs {
            if let Proc::Parent(parent) = proc {
                if self.started.is_none() && parent.peer.observed().iter().any(|frame| frame.kind == 0x0106) {
                    self.started = Some(now);
                }
                if self.ended.is_none() && parent.peer.observed().iter().any(|frame| frame.kind == 0x0110) {
                    self.ended = Some(now);
                }
            }
            if let Proc::Relay { process, .. } = proc
                && self.untooled.is_none()
                && skein_shell::Host::exit(process.as_ref()).is_some()
            {
                self.untooled = Some(now);
            }
        }
    }

    pub(super) fn account(&mut self, charter: &[u8]) {
        let charter =
            smith_charter::Charter::decode(&smith_charter::CEILINGS, &mut Reader::new(charter)).expect("world charter");
        self.account = charter.main.account.map(|account| account.to_string());
    }
}

pub(super) fn record(run: &mut Run, clock: &Clock, case: Case) {
    let Some(account) = &clock.account else { return };
    let struck = Struck { account: Some(account.clone()), ..Struck::default() };
    record_played(&mut run.ledger, account, &run.seen);
    if case == Case::MissingServer
        && let Some(at) = clock.untooled
    {
        run.ledger.inject(smith_faults::UNTOOLED_PROGRAM, at, struck);
    }
    let Some(started) = clock.started else { return };
    let name = "1"; // The host's Start activation.
    run.runs.started(started, name, account);
    record_retries(&mut run.runs, clock.origin.expect("agent iterated"), account, &run.events);
    if let Some(at) = clock.ended {
        let answer = run.answer().expect("observed answer");
        run.runs.ended(at, name, account, smith_faults::ended_channel(&answer));
    }
}

/// Copy a fake's played faults, preserving its moments and the host's account name.
/// Open intervals last through the end, including a program restart; rejected
/// rate limits stop at their reset on the world's calm simulated wall.
pub fn record_played(ledger: &mut FaultLedger, account: &str, seen: &[Seen]) {
    for seen in seen {
        if let Seen::Fault { fault, at_nanos } = seen
            && let Some(kind) = kind(fault)
        {
            let at = Time::from_nanos(*at_nanos);
            let entry = ledger.inject(
                kind,
                at,
                Struck {
                    account: Some(account.into()),
                    // Played observations have no response identity; name the
                    // host's Start activation as record does.
                    request: (kind == skein_llm_kit::faults::RATE_WINDOW).then(|| "1".into()),
                    ..Struck::default()
                },
            );
            if let Fault::Step(
                Step::RateLimit { resets_at: Some(reset) } | Step::RateLimitWindow { resets_at: Some(reset), .. },
            ) = fault
            {
                // The world uses Config::calm(): wall = initial wall + now.
                let reset = Time::from_nanos(
                    Duration::from_secs(*reset).as_nanos().saturating_sub(skein_sim::Config::calm().wall.as_nanos()),
                );
                ledger.stop(entry, reset.max(at)).expect("a played refusal stops once, no earlier than it played");
            }
        }
    }
}

fn kind(fault: &Fault) -> Option<Kind> {
    use skein_llm_kit::faults::{EXPIRED_SIGN_IN, HANG_AFTER_HEAD, RATE_WINDOW, UNSERVED_MODEL};
    match fault {
        Fault::Step(Step::RateLimit { .. }) => Some(RATE_WINDOW),
        Fault::Step(Step::RateLimitWindow { status, .. }) if status == "rejected" => Some(RATE_WINDOW),
        Fault::Step(Step::LoseLogin) => Some(EXPIRED_SIGN_IN),
        Fault::Step(Step::Hang) => Some(HANG_AFTER_HEAD),
        Fault::End(end) if *end == End::edge_refusal() => Some(smith_faults::EDGE_REFUSAL),
        Fault::End(end) if *end == End::billing_refusal() => Some(smith_faults::BILLING_REFUSAL),
        Fault::End(End::Error { api_error_status: Some(404), result: Some(result), .. }) => {
            let prefix = "API Error: 404 ";
            let value: serde_json::Value = serde_json::from_str(result.strip_prefix(prefix)?).ok()?;
            let model = value.get("error")?.get("message")?.as_str()?.strip_prefix("model: ")?;
            (*fault == Fault::End(End::unserved_model(model))).then_some(UNSERVED_MODEL)
        }
        Fault::Step(_) | Fault::End(_) => None,
    }
}

/// Count program and session retries from response records in emission order.
/// Each call keeps its run/conversation/window/turn identity across attempts.
pub fn record_retries(runs: &mut Runs, origin: Time, account: &str, records: &[smith_events::Record]) {
    let mut calls = BTreeMap::new();
    let mut pending_retry = BTreeMap::new();
    for record in records {
        let at = origin.saturating_add(Duration::from_millis(record.t_ms));
        match &record.event {
            Event::ResponseStarted(start) => {
                let call = format!("{}/{}/{}/{}", start.run, start.conversation, start.window, start.turn);
                let conversation = (start.run, start.conversation);
                let failed_at = pending_retry.remove(&conversation);
                if start.attempt > 0 {
                    // A program's api_retry renews its response without a new
                    // prompt. The session's retry also has retry_ms, but sends
                    // the program its input again after restarting it.
                    if start.messages == 0 {
                        if let Some(failed_at) = failed_at {
                            runs.retried(failed_at, account, &call, Retried::Program);
                        }
                    } else {
                        runs.retried(at, account, &call, Retried::Session);
                    }
                }
                calls.insert(start.response, call);
            }
            Event::ResponseCompleted(response)
                if response.outcome == Outcome::Failed && response.retry_ms.is_some() =>
            {
                assert!(calls.contains_key(&response.response), "response start before its retry");
                pending_retry.insert((response.run, response.conversation), at);
            }
            _ => {}
        }
    }
}
