//! Account strikes and the shared run-table referee (checks.md, 6.3).
use super::{Faults, World};
use crate::composed::{accounts, engine, tasks};
use skein_fake_llm_domain::api::{Error, Strike};
use skein_lib::{Duration, Time};
use skein_llm_kit::faults::RATE_WINDOW;
use skein_world::faults::{Counter, Struck};

#[derive(Default)]
pub(super) struct ObservedRun {
    facts: usize,
    ended: bool,
}

const WINDOW: Duration = Duration::from_millis(90);
const SPELL: Duration = Duration::from_secs(1);

impl World {
    pub(in crate::composed) fn workload_fault_assignment(&mut self, assignment: &engine::Assignment) {
        let restart_point = self
            .workload_faults
            .as_ref()
            .and_then(|f| f.plans.iter().find(|p| p.kind == temper_faults::restarts::ENGINE_CRASH_MID_TURN && !p.fired))
            .and_then(|p| self.workload_names().task(&p.goal).map(|task| (task, p.query)));
        if let Some((task, query)) = restart_point
            && assignment.task == task
        {
            self.process.held_query = Some(query);
        }
        let Some(f) = &mut self.workload_faults else { return };
        let key = (assignment.task, assignment.attempt);
        if f.observed_runs.contains_key(&key) {
            return;
        }
        f.observed_runs.insert(key, ObservedRun::default());
        let account = assignment.run.accounts.first().expect("run account").account;
        f.runs.started(self.env.now, &format!("{}:{}", key.0, key.1), &account.to_string());
        if let Some((account, _)) = f.provider {
            self.process.strike(accounts::credential(account), Strike::Window { tokens: 1, length: WINDOW });
        }
        self.process.configure_settings = Some(|settings| {
            settings.provider.accounts = 2;
            settings.provider.latency_min = Duration::ZERO;
            settings.provider.latency_max = Duration::ZERO;
            settings.network = skein_world::domain::Span::millis(1, 2);
        });
    }

    pub(super) fn workload_provider_tick(&mut self, f: &mut Faults) {
        let target = f.plans.iter().find(|p| p.kind == RATE_WINDOW).and_then(|p| self.workload_names().task(&p.goal));
        if f.due(RATE_WINDOW)
            && self.assignment.as_ref().is_some_and(|a| Some(a.task) == target)
            && !self.process.prepared()
        {
            let goal = f.plans.iter().find(|p| p.kind == RATE_WINDOW).expect("provider plan").goal.clone();
            let goal = self.workload().goal(&goal).expect("drawn goal");
            let account = super::super::setup::account(goal.party);
            f.fire(RATE_WINDOW);
            f.provider = Some((account, self.env.now.saturating_add(SPELL)));
            self.process.strike(accounts::credential(account), Strike::Window { tokens: 1, length: WINDOW });
            for process in self.parked.values_mut() {
                process.strike(accounts::credential(account), Strike::Window { tokens: 1, length: WINDOW });
            }
        }
        if let Some((account, until)) = f.provider
            && self.env.now >= until
        {
            let strike = Strike::Window { tokens: 1, length: WINDOW };
            self.process.clear(accounts::credential(account), &strike);
            for process in self.parked.values_mut() {
                process.clear(accounts::credential(account), &strike);
            }
            f.provider = None;
            f.stopped(self.env.now);
        }
    }

    pub(super) fn observe_workload_runs(&mut self, f: &mut Faults) {
        if self.process.prepared() {
            return;
        }
        let Some(assignment) = &self.assignment else { return };
        let key = (assignment.task, assignment.attempt);
        let Some(seen) = f.observed_runs.get_mut(&key) else { return };
        let account = assignment.run.accounts.first().expect("account").account;
        let run = format!("{}:{}", key.0, key.1);
        for fact in &self.process.agent().facts()[seen.facts..] {
            if let smith_domain::Fact::Session { fact } = fact
                && let smith_domain_session::FactKind::CompletionStarted { opener, attempt, .. } = fact.kind
                && attempt > 0
            {
                f.runs.retried(
                    self.process.outer_time(fact.at),
                    &account.to_string(),
                    &format!("{run}-{}-{}", opener.raw(), fact.response.expect("retry").number),
                    smith_faults::Retried::Session,
                );
            }
        }
        seen.facts = self.process.agent().facts().len();
        if self.process.answered.is_some() && !seen.ended {
            f.runs.ended(self.env.now, &run, &account.to_string(), smith_faults::ended(self.process.agent().answer()));
            seen.ended = true;
        }
        // These are the fake's Reply errors, retained before their client translation,
        // not failures guessed from the engine's task or a planned strike.
        for (at, call, credential, error) in self.process.provider_faults() {
            if let Error::Window { reset, .. } = error {
                let injection = f.ledger.inject(
                    RATE_WINDOW,
                    at,
                    Struck {
                        account: Some((credential + accounts::FIRST_ACCOUNT).to_string()),
                        request: Some(format!("{run}:{}", call.raw())),
                        ..Struck::default()
                    },
                );
                let until = at.saturating_add(reset);
                f.ledger.stop(injection, until).expect("fixed rate window");
                f.stopped(until);
            }
        }
        for row in self.store.rows.values() {
            if let engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row))) = row
                && matches!(row.end, Some(tasks::EndKind::Failed { .. }))
                && f.failed_attempts.insert((row.task, row.attempt))
            {
                let Some(assignment) = self
                    .workload
                    .as_ref()
                    .expect("workload")
                    .assignments
                    .iter()
                    .find(|a| a.task == row.task && a.attempt == row.attempt)
                else {
                    continue;
                };
                let at = Time::from_nanos(row.ended.expect("failed end").as_nanos());
                let struck = Struck {
                    account: Some(assignment.run.accounts.first().expect("account").account.to_string()),
                    ..Struck::default()
                };
                for injection in f.ledger.striking(at, &struck) {
                    f.ledger.count(injection, Counter::TaskTries);
                }
            }
        }
    }
}
