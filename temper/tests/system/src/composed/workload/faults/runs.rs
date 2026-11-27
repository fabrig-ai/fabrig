//! Account strikes and the shared run-table referee (checks.md, 6.3).
use super::{Faults, World};
use crate::composed::{Key, accounts, engine, people, run, tasks};
use skein_fake_llm_domain::api::{Error, Strike};
use skein_lib::{Duration, ReplyTo, Time, Token};
use skein_llm_kit::faults::{RATE_WINDOW, SPENT_CREDITS};
use skein_world::faults::{Counter, Injection, Struck};
use temper_workload::draw::Party;

#[derive(Default)]
pub(super) struct ObservedRun {
    facts: usize,
    ended: bool,
    window: Option<Injection>,
    spent: Option<Injection>,
}

pub(super) struct SpentCredits {
    account: u32,
    party: Party,
    until: Time,
    injection: Injection,
    hit: Option<(u64, u64)>,
}

const WINDOW: Duration = Duration::from_millis(90);
const SPELL: Duration = Duration::from_secs(1);

impl World {
    pub(in crate::composed) fn workload_fault_assignment(&mut self, assignment: &engine::Assignment) {
        let restart_query =
            self.workload_faults.as_ref().and_then(|f| self.workload_restart_query(f, assignment.task, 0));
        if let Some(query) = restart_query {
            self.lanes[0].process.held_query = Some(query);
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
            self.lanes[0].process.strike(accounts::credential(account), Strike::Window { tokens: 1, length: WINDOW });
        }
        if let Some(spent) = &f.spent_credits {
            self.lanes[0].process.strike(accounts::credential(spent.account), Strike::Spent);
        }
        self.lanes[0].process.configure_settings = Some(|settings| {
            settings.provider.accounts = 2;
            settings.provider.latency_min = Duration::ZERO;
            settings.provider.latency_max = Duration::ZERO;
            settings.network = skein_world::domain::Span::millis(1, 2);
        });
    }

    pub(super) fn workload_provider_tick(&mut self, f: &mut Faults) {
        let target = f.plans.iter().find(|p| p.kind == RATE_WINDOW).and_then(|p| self.workload_names().task(&p.goal));
        if f.due(RATE_WINDOW)
            && self.lanes[0].assignment.as_ref().is_some_and(|a| Some(a.task) == target)
            && !self.lanes[0].process.prepared()
        {
            let goal = f.plans.iter().find(|p| p.kind == RATE_WINDOW).expect("provider plan").goal.clone();
            let goal = self.workload().goal(&goal).expect("drawn goal");
            let account = super::super::setup::account(goal.party);
            f.fire(RATE_WINDOW);
            f.provider = Some((account, self.env.now.saturating_add(SPELL)));
            self.lanes[0].process.strike(accounts::credential(account), Strike::Window { tokens: 1, length: WINDOW });
            for process in self.parked.values_mut() {
                process.strike(accounts::credential(account), Strike::Window { tokens: 1, length: WINDOW });
            }
        }
        if let Some((account, until)) = f.provider
            && self.env.now >= until
        {
            let strike = Strike::Window { tokens: 1, length: WINDOW };
            self.lanes[0].process.clear(accounts::credential(account), &strike);
            for process in self.parked.values_mut() {
                process.clear(accounts::credential(account), &strike);
            }
            f.provider = None;
            f.stopped(self.env.now);
        }
        self.workload_spent_credits_tick(f);
    }

    fn workload_spent_credits_tick(&mut self, f: &mut Faults) {
        if f.due(SPENT_CREDITS)
            && let Some(name) = f.target(SPENT_CREDITS)
            && let Some(task) = self.workload_names().task(name)
            && self.lanes[0].assignment.as_ref().is_some_and(|a| a.task == task)
            && !self.lanes[0].process.prepared()
            && self.lanes[0].process.answered.is_none()
        {
            let party = self.workload().goal(name).expect("drawn goal").party;
            let account = super::super::setup::account(party);
            let goal = f.fire(SPENT_CREDITS);
            let injection = f.ledger.inject(
                SPENT_CREDITS,
                self.env.now,
                Struck { account: Some(account.to_string()), request: Some(goal), ..Struck::default() },
            );
            f.spent_credits =
                Some(SpentCredits { account, party, until: self.env.now.saturating_add(SPELL), injection, hit: None });
            let credential = accounts::credential(account);
            self.lanes[0].process.strike(credential, Strike::Spent);
            for process in self.parked.values_mut() {
                process.strike(credential, Strike::Spent);
            }
        }
        // Once the provider has refused a request, wait for its limit to be
        // durable before replacing the key, as in the account sweep.
        if f.spent_credits.as_ref().is_some_and(|spent| {
            self.env.now >= spent.until
                && spent.hit.is_none_or(|(task, attempt)| {
                    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Attempt { task, attempt }));
                    self.source_held(spent.account)
                        && matches!(self.store.rows.get(&key),
                            Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row))))
                                if row.end.is_some())
                })
        }) {
            let spent = f.spent_credits.take().expect("active spent credits");
            let credential = accounts::credential(spent.account);
            self.lanes[0].process.clear(credential, &Strike::Spent);
            for process in self.parked.values_mut() {
                process.clear(credential, &Strike::Spent);
            }
            if spent.hit.is_some() {
                self.workload_replace_key(spent.party, credential);
            }
            f.ledger.stop(spent.injection, self.env.now).expect("one clear after the strike");
            f.stopped(self.env.now);
        }
    }

    fn workload_replace_key(&mut self, party: Party, source: u32) {
        let story = self.workload.as_mut().expect("workload");
        let sign_in = match party {
            Party::First => self.party.sign_in,
            Party::Second => story.other.sign_in,
        }
        .expect("struck account's holder signed in");
        let number = story.next_reply;
        story.next_reply += 1;
        story.pending_asks.insert(number);
        let mut key = [0; 16];
        key[..8].copy_from_slice(&number.to_le_bytes());
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Key {
            reply_to: ReplyTo::new(Token::new(number)),
            sign_in,
            key,
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Own,
                change: people::AccountChange::ReplaceKey { source, digest: [5; 32] },
            }),
            carried: jig_domain_core::Carried { number: 1_u64 << 63, digest: [5; 32] },
        }));
    }

    pub(super) fn observe_workload_runs(&mut self, f: &mut Faults) {
        if self.lanes[0].process.prepared() {
            return;
        }
        let Some(assignment) = &self.lanes[0].assignment else { return };
        let key = (assignment.task, assignment.attempt);
        let Some(seen) = f.observed_runs.get_mut(&key) else { return };
        let account = assignment.run.accounts.first().expect("account").account;
        let run = format!("{}:{}", key.0, key.1);
        for fact in &self.lanes[0].process.agent().facts()[seen.facts..] {
            if let smith_domain::Fact::Session { fact } = fact
                && let smith_domain_session::FactKind::CompletionStarted { opener, attempt, .. } = fact.kind
                && attempt > 0
            {
                f.runs.retried(
                    self.lanes[0].process.outer_time(fact.at),
                    &account.to_string(),
                    &format!("{run}-{}-{}", opener.raw(), fact.response.expect("retry").number),
                    smith_faults::Retried::Session,
                );
            }
        }
        seen.facts = self.lanes[0].process.agent().facts().len();
        // These are the fake's Reply errors, retained before their client translation,
        // not failures guessed from the engine's task or a planned strike.
        for (at, call, credential, error) in self.lanes[0].process.provider_faults() {
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
                f.last = f.last.max(until);
                seen.window = Some(injection);
            } else if error == Error::Spent
                && let Some(spent) = &mut f.spent_credits
                && accounts::credential(spent.account) == credential
            {
                spent.hit = Some(key);
                seen.spent = Some(spent.injection);
            }
        }
        if self.lanes[0].process.answered.is_some() && !seen.ended {
            let answer = self.lanes[0].process.agent().answer();
            let ended = smith_faults::ended(answer);
            let caused = seen
                .spent
                .or(seen.window)
                .filter(|_| matches!(answer, run::Answer::Failed { failure: run::RunFailure::Model(_), .. }));
            // An interrupted run on the same account is not a provider ending.
            if ended == smith_faults::Ended::Completed || caused.is_some() {
                f.runs.ended(self.env.now, &run, &account.to_string(), ended);
            }
            if let Some(injection) = caused {
                f.provider_failures.push((key, injection));
            }
            seen.ended = true;
        }
    }

    pub(super) fn count_provider_tries(&mut self) {
        let f = self.workload_faults.as_mut().expect("faulted workload");
        for &((task, attempt), injection) in &f.provider_failures {
            let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Attempt { task, attempt }));
            if matches!(self.store.rows.get(&key),
                Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row))))
                    if matches!(row.end, Some(tasks::EndKind::Failed { .. })))
            {
                f.ledger.count(injection, Counter::TaskTries);
            }
        }
    }
}
