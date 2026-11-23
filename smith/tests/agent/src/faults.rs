//! Run observations for fault sweeps (smith/docs/design/faults.md, 3–5).
//! Response identities and retries come from the production event projection,
//! not from provider request counts. Only the host's run and account names are supplied.
use std::collections::BTreeMap;

use skein_lib::{Duration, Time, Token};
use smith_domain::{Fact, llm, run};
use smith_protocol_events::sinks::{Capture, Delivery, Destination, Sinks};

#[derive(Debug)]
pub(crate) struct Observations {
    sink: Sinks,
    run: String,
    account: String,
    runs: smith_faults::Runs,
    calls: BTreeMap<(u64, u64), String>,
    last_call: Option<String>,
}

const DESTINATION: Token = Token::new(9000);

impl Observations {
    pub(crate) fn new(now: Time, run: &str, account: &str) -> Self {
        let limits = smith_protocol_events_fixtures::sinks::small();
        let mut sink = Sinks::new(&limits, Capture::None, limits.output, Duration::from_secs(1));
        sink.add(Destination::Trace, DESTINATION, Delivery::Complete).expect("one observation destination");
        let mut runs = smith_faults::Runs::new();
        runs.started(now, run, account);
        Self { sink, run: run.into(), account: account.into(), runs, calls: BTreeMap::new(), last_call: None }
    }

    pub(crate) fn fact(&mut self, fact: &Fact) {
        match fact {
            Fact::Run { fact, opening } => match fact.kind {
                run::facts::FactKind::Opened { conversation, .. } => {
                    let opening = opening.as_ref().expect("opening accompanies its fact");
                    self.sink.conversation(
                        fact.at,
                        Time::ZERO,
                        conversation,
                        smith_events::ConversationOpened {
                            run: opening.activation,
                            conversation: u64::from(opening.ordinal),
                            kind: smith_events::ConversationKind::Main,
                            parent: None,
                            call: None,
                            thread: u64::from(opening.ordinal),
                            model: smith_protocol_events::opening_model(
                                &opening.llm,
                                b"fixture",
                                opening.llm.effort.as_deref(),
                                smith_events::Backend::Direct,
                            ),
                            requested: None,
                            share: None,
                            clamped: skein_lib::List::with_capacity(0),
                        },
                    );
                }
                run::facts::FactKind::Ended { conversation, end, .. } => {
                    self.sink.conversation_ended(fact.at, conversation, end);
                }
                _ => {}
            },
            Fact::Session { fact } => self.sink.session(Time::ZERO, fact),
        }
        self.drain();
    }

    pub(crate) fn request(&mut self, now: Time, owner: Token, prompt: &llm::Prompt) -> String {
        self.sink.request(now, Time::ZERO, owner, prompt);
        self.drain();
        self.last_call.clone().expect("a response start names its logical call")
    }

    pub(crate) fn terminal(&mut self, event: &smith_domain::Event) {
        match event {
            smith_domain::Event::Below(smith_domain::Below::Completed { owner, completion }) => {
                self.sink.received(*owner, completion);
            }
            smith_domain::Event::Below(smith_domain::Below::Failed { owner, failure, evidence, detail, report }) => {
                self.sink.failed(*owner, *failure, *evidence, detail, report);
            }
            _ => {}
        }
    }

    pub(crate) fn ended(&mut self, now: Time, answer: &run::Answer) {
        self.runs.ended(now, &self.run, &self.account, smith_faults::ended(answer));
    }

    pub(crate) fn judge(&mut self, ledger: &mut skein_world::faults::FaultLedger) -> Result<(), String> {
        self.drain();
        self.runs.judge(ledger)
    }

    fn drain(&mut self) {
        while self.sink.facts_drained(Time::ZERO) {}
        for record in smith_protocol_events_fixtures::sinks::drain_ready(&mut self.sink, DESTINATION) {
            if let smith_events::Event::ResponseStarted(start) = record.event {
                let next = self.calls.len() + 1;
                let call = self
                    .calls
                    .entry((start.conversation, start.turn))
                    .or_insert_with(|| format!("{} call {next}", self.run));
                if start.attempt > 0 {
                    self.runs.retried(
                        Time::ZERO.saturating_add(Duration::from_millis(record.t_ms)),
                        &self.account,
                        call,
                        smith_faults::Retried::Session,
                    );
                }
                self.last_call = Some(call.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Job, Settings, World};
    use skein_world::faults::{Counter, FaultLedger, Struck};

    #[test]
    fn the_window_exclusion_uses_the_shortest_first_backoff() {
        for (base, cap, shortest) in [(200, 5000, 100), (200, 50, 25)] {
            let mut settings = Settings::calm(1);
            settings.limits.session.backoff_base = Duration::from_millis(base);
            settings.limits.session.backoff_max = Duration::from_millis(cap);
            let world = World::new(settings);
            assert_eq!(world.shortest_backoff(), Duration::from_millis(shortest));
        }
    }

    #[test]
    fn response_records_count_retries_without_renaming_the_call() {
        let settings = Settings { job: Job::Reporting, ..Settings::calm(1) };
        let mut world = World::new(settings);
        world.fail_calls(1, llm::Failure::Unavailable, llm::Evidence::Unknown);
        world.record_fault_runs("run 1", "a");
        let mut ledger = FaultLedger::new();
        ledger.inject(
            smith_faults::EDGE_REFUSAL,
            Time::ZERO,
            Struck { account: Some("a".into()), ..Struck::default() },
        );
        world.run(10_000);
        world.judge_faults(&mut ledger).expect("one transient retry");
        assert_eq!(ledger.entries()[0].count(Counter::SessionRetries), 1);
        let observations = world.fault_runs.as_ref().expect("recording enabled");
        assert_eq!(
            observations.calls.values().map(String::as_str).collect::<Vec<_>>(),
            ["run 1 call 1", "run 1 call 2"]
        );
        assert!(matches!(world.answer(), run::Answer::Accepted { .. }));
    }
}
