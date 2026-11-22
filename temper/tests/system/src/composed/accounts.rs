//! Two parties' account isolation (temper/docs/design/testing/checks.md, 6.3).
//! The roots choose accounts and end tasks; this fixture only supplies parties,
//! provider scripts and observations of their durable decisions.
use super::{Process, Script, Time, World, call, engine, parties, people, tasks};
use jig_domain_accounts::{Holder, Kind, Stored};
use skein_fake_llm_domain::api::Strike;
use skein_fake_llm_domain::api::{Finish, Line, Query, Turn};
use skein_lib::{Duration, Token};
use skein_world::faults::{Counter, FaultLedger, Injection, Struck};
use smith_faults::{Retried, Runs};
use std::collections::{BTreeMap, BTreeSet};

pub const FIRST_ACCOUNT: u32 = 2;
pub const OTHER_ACCOUNT: u32 = 3;

/// Each key source's grant number is its fake-provider credential index.
#[must_use]
pub fn credential(account: u32) -> u32 {
    account - FIRST_ACCOUNT
}
const OTHER_REPLIES: u64 = 500;
// Two parties, each with one key account; same-holder scenarios use both.
const ACCOUNT_COUNT: u32 = 2;

pub(super) struct Story {
    pub other: parties::Party,
    pub assignments: Vec<engine::Assignment>,
    pub queries: Vec<(u64, Query)>,
    seen: usize,
    facts: usize,
    ended: bool,
    epoch: Option<Time>,
    steps: u32,
    pub ledger: FaultLedger,
    pub runs: Runs,
    fault: Option<Fault>,
    failures: BTreeSet<(u64, u64)>,
    pub ended_at: BTreeMap<u64, (Time, u32)>,
    windows: BTreeSet<(u64, u64, usize)>,
    marked: BTreeMap<u32, usize>,
    inspected_commit: u64,
}

struct Fault {
    account: u32,
    strike: Strike,
    at_call: usize,
    clear_after: Option<u32>,
    injection: Option<Injection>,
    struck_at: Option<Time>,
    hit: Option<u32>,
    cleared: bool,
    cleared_at: Option<(Time, u32)>,
}

fn config(seed: u64) -> engine::Config {
    // Account isolation needs the reporting charter, not the stock role catalogue.
    let mut config = super::config(seed);
    // Share the unchanged deployment allowance between two party pools.
    let share = config.core.defaults.rules.period_spend / 2;
    for project in &mut config.core.seed_projects {
        for role in &mut project.policy.policy.roles {
            role.period_spend = share;
        }
    }
    config
}

fn script() -> Script {
    Script {
        cue: b"@accounts".as_slice().into(),
        turns: Box::new([
            Turn {
                lines: Box::new([Line::Text { text: b"Preparing the report.".as_slice().into() }]),
                finish: Finish::Stop,
                tokens: 1,
            },
            call(b"finish", br#"{"report":"The repository needs no change."}"#),
        ]),
    }
}

impl World {
    /// Two independently funded parties, each with its own key account.
    #[must_use]
    pub fn two_accounts(seed: u64) -> Self {
        let mut world = Self::configured_limits(seed, script(), false, config, |limits| {
            limits.accounts.accounts = ACCOUNT_COUNT;
            limits.accounts.holder_accounts = ACCOUNT_COUNT;
            limits.accounts.sign_ins = ACCOUNT_COUNT;
            limits.accounts.credentials = ACCOUNT_COUNT;
            // Each account may contribute its binding and its credential.
            limits.fleet.grants = ACCOUNT_COUNT * 2;
        });
        let erases = world
            .store
            .rows
            .values()
            .filter(|row| matches!(row, engine::Record::Core(jig_domain_core::Record::Accounts(_))))
            .map(engine::Record::key)
            .collect();
        let mut rows = Vec::new();
        for (number, party) in [(FIRST_ACCOUNT, 1), (OTHER_ACCOUNT, 2)] {
            rows.push(Stored::Account {
                list: None,
                number,
                provider: 1,
                holder: Holder::Party(party),
                place: 0,
                uses: None,
                subject: Box::default(),
                display: format!("account {number}").into_bytes().into(),
            });
            rows.push(Stored::Source {
                account: number,
                number: credential(number),
                place: 0,
                kind: Kind::Key,
                name: Box::default(),
                held: None,
            });
        }
        rows.push(Stored::Next { number: OTHER_ACCOUNT + 1 });
        world.store.seed(
            rows.into_iter().map(|row| engine::Record::Core(jig_domain_core::Record::Accounts(row))).collect(),
            erases,
        );
        let mut holdings = match world
            .store
            .rows
            .get(&engine::Key::Core(jig_domain_core::Key::People(people::Key::Roles(1))))
            .expect("project roles")
        {
            engine::Record::Core(jig_domain_core::Record::People(people::Stored::Roles { holdings, .. })) => {
                holdings.to_vec()
            }
            _ => unreachable!("roles key"),
        };
        holdings.push(people::Holding { person: 2, role: people::Role::Owner });
        let rows = vec![
            people::Stored::Person {
                number: 2,
                identity: people::Identity {
                    key: temper_engine_domain_forge::user_key(1, 8),
                    login: b"second".as_slice().into(),
                    name: b"Second party".as_slice().into(),
                },
            },
            people::Stored::Roles { project: 1, holdings: holdings.into() },
        ];
        world.store.seed(
            rows.into_iter().map(|row| engine::Record::Core(jig_domain_core::Record::People(row))).collect(),
            Box::default(),
        );
        world.party = super::signed_in_party(b"@accounts Inspect the repository for party A.");
        world.account_case = Some(Story {
            other: parties::Party::new(
                1,
                parties::Role::Owner,
                OTHER_REPLIES,
                Box::new([
                    parties::Act::SignIn { provider: 0, subject: temper_engine_domain_forge::user_key(1, 8).subject },
                    parties::Act::Request {
                        key: [42; 16],
                        ask: parties::Ask::Chat {
                            words: b"@accounts Inspect the repository for party B.".as_slice().into(),
                        },
                    },
                ]),
            ),
            assignments: Vec::new(),
            queries: Vec::new(),
            seen: 0,
            facts: 0,
            ended: false,
            epoch: None,
            steps: 0,
            ledger: FaultLedger::new(),
            runs: Runs::new(),
            fault: None,
            failures: BTreeSet::new(),
            ended_at: BTreeMap::new(),
            windows: BTreeSet::new(),
            marked: BTreeMap::new(),
            inspected_commit: 0,
        });
        world
    }

    pub(super) fn account_goal(&self, ask: people::Ask) -> people::Ask {
        if self.account_case.is_none() {
            return ask;
        }
        walking_goal(ask)
    }

    pub(super) fn account_party(&mut self) {
        if self.opened && self.adoption != super::Adoption::Pending {
            let up = self.account_case.as_mut().and_then(|story| story.other.tick(self.env.now));
            if let Some(up) = up {
                self.party_request(up);
            }
        }
    }

    pub(super) fn account_reply(&mut self, to: Token, reply: &people::Reply) -> bool {
        if to.raw() == 905 {
            assert!(
                matches!(reply, people::Reply::Outcome(people::Outcome::AccountKeyKept { .. })),
                "holder replaces key: {reply:?}"
            );
            return true;
        }
        if !(OTHER_REPLIES..OTHER_REPLIES + 100).contains(&to.raw()) {
            return false;
        }
        let reply = match reply {
            people::Reply::SignedIn { person, sign_in, .. } => {
                parties::Reply::SignedIn { person: *person, sign_in: *sign_in }
            }
            people::Reply::Outcome(people::Outcome::Created { first, last, .. }) => {
                assert_eq!(first, last);
                parties::Reply::Started { task: *first }
            }
            other => panic!(
                "second party reply: {other:?}; funding {:?}; budget {}",
                self.store
                    .rows
                    .values()
                    .filter(|row| matches!(
                        row,
                        engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(_)))
                    ))
                    .collect::<Vec<_>>(),
                self.period_budget
            ),
        };
        self.account_case.as_mut().expect("second party").other.reply(to.raw(), reply);
        true
    }

    pub(super) fn account_assignment(&mut self, assignment: &engine::Assignment) -> bool {
        let Some(story) = self.account_case.as_mut() else {
            return false;
        };
        if self.process.prepared() {
            self.process = Process::new(self.seed, script());
        } else {
            assert!(self.process.answered.is_some(), "the previous process ended");
            self.parked.insert(
                self.assignment.as_ref().expect("previous assignment").task,
                core::mem::replace(&mut self.process, Process::new(self.seed, script())),
            );
        }
        self.process.configure_settings = Some(|settings| {
            settings.provider.accounts = ACCOUNT_COUNT;
            settings.provider.unserved = 1;
            // Both prompts fit inside the shortest drawn window; only the
            // session's backoff, not the fake's latency, should reset it.
            settings.provider.latency_min = Duration::ZERO;
            settings.provider.latency_max = Duration::ZERO;
            settings.network = skein_world::domain::Span::millis(1, 2);
        });
        if let Some(fault) = &story.fault
            && fault.struck_at.is_some()
            && !fault.cleared
        {
            self.process.strikes.push((credential(fault.account), fault.strike.clone()));
        }
        self.process.clocked = true;
        story.runs.started(self.env.now, &run_name(assignment), &account(assignment).to_string());
        story.assignments.push(assignment.clone());
        story.seen = 0;
        story.facts = 0;
        story.ended = false;
        story.epoch = None;
        true
    }

    pub(super) fn account_observe(&mut self) {
        if self.process.prepared() {
            return;
        }
        let Some(story) = &mut self.account_case else {
            return;
        };
        let assignment = self.assignment.as_ref().expect("assigned process");
        let task = assignment.task;
        let epoch = *story.epoch.get_or_insert(self.env.now);
        for fact in &self.process.agent().facts()[story.facts..] {
            if let smith_domain::Fact::Session { fact } = fact
                && let smith_domain_session::FactKind::CompletionStarted { opener, attempt, .. } = fact.kind
                && attempt > 0
            {
                let call = format!(
                    "{}-{}-{}",
                    run_name(assignment),
                    opener.raw(),
                    fact.response.expect("retry response").number
                );
                story.runs.retried(
                    epoch.saturating_add(Duration::from_nanos(fact.at.as_nanos())),
                    &account(assignment).to_string(),
                    &call,
                    Retried::Session,
                );
            }
        }
        story.facts = self.process.agent().facts().len();
        if let Some(Fault { strike: Strike::Window { length, .. }, .. }) = &story.fault
            && let Some(start) = self.process.struck_at
        {
            for (index, completion) in self.process.agent().completions().iter().enumerate() {
                if matches!(completion.terminal, Some((_, smith_agent_world::CompletionTerminal::Failed)))
                    && story.windows.insert((assignment.task, assignment.attempt, index))
                {
                    let offset = completion.started.saturating_since(start).as_nanos() % length.as_nanos();
                    let from = epoch.saturating_add(Duration::from_nanos(completion.started.as_nanos()));
                    let until = from.saturating_add(Duration::from_nanos(length.as_nanos() - offset));
                    let entry = story.ledger.inject(
                        skein_llm_kit::faults::RATE_WINDOW,
                        from,
                        Struck { account: Some(account(assignment).to_string()), ..Struck::default() },
                    );
                    story.ledger.stop(entry, until).expect("a fixed window ends at its reset");
                }
            }
        }
        if self.process.answered.is_some() && !story.ended {
            let end = smith_faults::ended(self.process.agent().answer());
            story.runs.ended(self.env.now, &run_name(assignment), &account(assignment).to_string(), end);
            story.ended = true;
        }
        for query in &self.process.agent().prompts()[story.seen..] {
            story.queries.push((task, query.clone()));
        }
        story.seen = self.process.agent().prompts().len();
        if let Some(fault) = &mut story.fault
            && fault.struck_at.is_some()
            && account(assignment) == fault.account
            && matches!(fault.strike, Strike::Window { .. })
            && !self.process.agent().completions().is_empty()
        {
            fault.hit.get_or_insert(story.steps);
        }
    }

    /// Two alternative key accounts of one holder, with only that holder's goal.
    #[must_use]
    pub fn same_holder_accounts(seed: u64) -> Self {
        let mut world = Self::two_accounts(seed);
        let rows = world
            .store
            .rows
            .values()
            .filter_map(|row| {
                if let engine::Record::Core(jig_domain_core::Record::Accounts(Stored::Account {
                    number: OTHER_ACCOUNT,
                    ..
                })) = row
                {
                    let mut row = row.clone();
                    if let engine::Record::Core(jig_domain_core::Record::Accounts(Stored::Account {
                        holder,
                        place,
                        ..
                    })) = &mut row
                    {
                        *holder = Holder::Party(1);
                        *place = 1;
                    }
                    Some(row)
                } else {
                    None
                }
            })
            .collect();
        world.store.seed(rows, Box::default());
        world.account_case.as_mut().expect("accounts").other =
            parties::Party::new(1, parties::Role::Owner, OTHER_REPLIES, Box::default());
        world
    }

    /// Run the one holder's finite goal within three minutes and 20,000 steps.
    pub fn finish_account_goal(&mut self) -> Result<(), String> {
        let began = self.env.now;
        for _ in 0..20_000 {
            self.account_step();
            if let Some(task) = self.party.last_task {
                if self.account_goal_done(task) {
                    return Ok(());
                }
                if let Some(row) = self.store.rows.values().find_map(|record| {
                    let engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = record else {
                        return None;
                    };
                    (row.number == task
                        && (matches!(row.phase, tasks::Phase::Held { .. })
                            || (row.phase == tasks::Phase::Active(tasks::Active::Due)
                                && row.kept_account.is_some_and(|account| self.source_held(account)))))
                    .then_some(row)
                }) {
                    return Err(format!(
                        "goal stayed on account {:?}, phase {:?}, tries {:?}",
                        row.kept_account, row.phase, row.tries
                    ));
                }
            }
            if self.env.now > began.saturating_add(Duration::from_secs(3 * 60)) {
                break;
            }
        }
        Err("the held account's goal did not choose another account and end within 3 minutes / 20,000 steps".into())
    }

    /// The spent source is durable, and every assignment after that mark used another account.
    pub fn spent_stopped_placement(&mut self, account_number: u32) -> bool {
        for _ in 0..1000 {
            self.account_step();
        }
        let story = self.account_case.as_ref().expect("two accounts");
        let Some(at) = story.marked.get(&account_number) else {
            return false;
        };
        let limited = self.store.rows.values().any(|row| {
            matches!(row,
            engine::Record::Core(jig_domain_core::Record::Accounts(Stored::Source {
                account, held: Some(jig_domain_accounts::Held::Limited { reset: None }), ..
            })) if *account == account_number)
        });
        limited && story.assignments[*at..].iter().all(|assignment| account(assignment) != account_number)
    }

    /// Last claimed account of the first party's task, retained after its end.
    #[must_use]
    pub fn first_goal_account(&self) -> Option<u32> {
        let task = self.party.last_task?;
        self.store.rows.values().find_map(|row| match row {
            engine::Record::Core(jig_domain_core::Record::Tasks(
                tasks::Stored::Live(row) | tasks::Stored::Ended(row),
            )) if row.number == task => row.kept_account,
            _ => None,
        })
    }

    /// Arrange one account fault at a query boundary of the struck goal's run.
    pub fn plan_account_fault(&mut self, account: u32, strike: Strike, at_call: usize, clear_after: Option<u32>) {
        assert!(at_call < 2, "one of this goal's two model requests");
        self.account_case.as_mut().expect("two accounts").fault = Some(Fault {
            account,
            strike,
            at_call,
            clear_after,
            injection: None,
            struck_at: None,
            hit: None,
            cleared: false,
            cleared_at: None,
        });
    }

    pub(super) fn account_clock(&mut self) {
        let Some(story) = &mut self.account_case else {
            return;
        };
        story.steps += 1;
        if story.inspected_commit != self.store.applied {
            story.inspected_commit = self.store.applied;
            for row in self.store.rows.values() {
                if let engine::Record::Core(jig_domain_core::Record::Accounts(Stored::Source {
                    account,
                    held: Some(_),
                    ..
                })) = row
                {
                    story.marked.entry(*account).or_insert(story.assignments.len());
                }
                if let engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) = row {
                    story.ended_at.entry(row.number).or_insert((self.env.now, story.steps));
                }
                let engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row))) = row else {
                    continue;
                };
                if matches!(row.end, Some(tasks::EndKind::Failed { .. }))
                    && story.failures.insert((row.task, row.attempt))
                {
                    let assignment = story
                        .assignments
                        .iter()
                        .find(|a| a.task == row.task && a.attempt == row.attempt)
                        .expect("failed run was assigned");
                    let struck = Struck { account: Some(account(assignment).to_string()), ..Struck::default() };
                    let at = Time::from_nanos(row.ended.expect("ended attempt").as_nanos());
                    for injection in story.ledger.striking(at, &struck) {
                        story.ledger.count(injection, Counter::TaskTries);
                    }
                }
            }
        }
        let Some(fault) = &mut story.fault else {
            return;
        };
        if fault.struck_at.is_some()
            && !matches!(fault.strike, Strike::Window { .. })
            && self.store.rows.values().any(|row| {
                matches!(row,
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                if row.kept_account == Some(fault.account) && row.phase == tasks::Phase::Active(tasks::Active::Due))
            })
        {
            fault.hit.get_or_insert(story.steps);
        }
        let goal = if fault.account == FIRST_ACCOUNT { self.party.last_task } else { story.other.last_task };
        let calls = story.queries.iter().filter(|(task, _)| Some(*task) == goal).count();
        if fault.struck_at.is_none() && self.assignment.is_some() && calls >= fault.at_call {
            let struck = Struck { account: Some(fault.account.to_string()), ..Struck::default() };
            fault.struck_at = Some(self.env.now);
            if !matches!(fault.strike, Strike::Window { .. }) {
                fault.injection =
                    Some(story.ledger.inject(skein_llm_kit::faults::struck(&fault.strike), self.env.now, struck));
            }
            self.process.strike(credential(fault.account), fault.strike.clone());
        }
        if let (Some(_), Some(hit), Some(after)) = (fault.struck_at, fault.hit, fault.clear_after)
            && !fault.cleared
            && story.steps >= hit + after
        {
            fault.cleared = true;
            fault.cleared_at = Some((self.env.now, story.steps));
            if let Some(injection) = fault.injection {
                story.ledger.stop(injection, self.env.now).expect("one clear after the strike");
            }
            let account = fault.account;
            self.process.clear(credential(account), &fault.strike);
            for process in self.parked.values_mut() {
                process.clear(credential(account), &fault.strike);
            }
            if !matches!(fault.strike, Strike::Window { .. }) {
                let sign_in = if account == FIRST_ACCOUNT { self.party.sign_in } else { story.other.sign_in };
                self.root_events.push_back(engine::Event::Party(engine::PartyInput::Key {
                    reply_to: skein_lib::ReplyTo::new(Token::new(905)),
                    sign_in: sign_in.expect("faulted account's holder signed in"),
                    key: [55; 16],
                    ask: Box::new(people::Ask::AccountChange {
                        holder: people::AccountHolder::Own,
                        change: people::AccountChange::ReplaceKey { source: credential(account), digest: [5; 32] },
                    }),
                    carried: jig_domain_core::Carried { number: 1_u64 << 63, digest: [5; 32] },
                }));
            }
        }
    }

    /// Bound both the unaffected goal and recovery, independently in time and steps.
    pub fn run_account_case(
        &mut self,
        struck: u32,
        clears: bool,
        other_bound: (Duration, u32),
        recovery_bound: (Duration, u32),
    ) -> Result<(), String> {
        let (other_time, other_steps) = other_bound;
        let (recovery_time, recovery_steps) = recovery_bound;
        assert!(other_time > Duration::ZERO && recovery_time > Duration::ZERO && other_steps > 0 && recovery_steps > 0);
        let began = self.env.now;
        for _ in 0..other_steps + recovery_steps {
            self.account_step();
            let story = self.account_case.as_ref().expect("two accounts");
            let (Some(a), Some(b)) = (self.party.last_task, story.other.last_task) else {
                continue;
            };
            let (faulted, other) = if struck == FIRST_ACCOUNT { (a, b) } else { (b, a) };
            if let Some((at, steps)) = story.ended_at.get(&other) {
                if *at > began.saturating_add(other_time) || *steps > other_steps {
                    return Err("unaffected goal exceeded OTHER_GOAL_MINUTES or OTHER_GOAL_STEPS".into());
                }
            } else if self.env.now > began.saturating_add(other_time) || story.steps > other_steps {
                return Err("unaffected goal did not end within OTHER_GOAL_MINUTES and OTHER_GOAL_STEPS".into());
            }
            if !self.account_goal_done(other) {
                continue;
            }
            let fault = story.fault.as_ref().expect("planned fault");
            let window = matches!(fault.strike, Strike::Window { .. });
            if clears || window {
                if (fault.cleared || window) && self.account_goal_done(faulted) {
                    let cleared = fault.injection.and_then(|entry| story.ledger.get(entry).until).unwrap_or(began);
                    let (at, step) = story.ended_at.get(&faulted).expect("ending observed");
                    let from_step = fault.cleared_at.map_or(0, |(_, step)| step);
                    if *at <= cleared.saturating_add(recovery_time) && step.saturating_sub(from_step) <= recovery_steps
                    {
                        return Ok(());
                    }
                    return Err("struck goal exceeded RECOVERY_MINUTES".into());
                }
            } else if self.account_goal_done(faulted) {
                return Err(format!("uncleared {:?} on account {struck} did not hold goal {faulted}", fault.strike));
            } else if self.source_held(struck)
                && self.store.rows.values().any(|row| {
                    matches!(row,
                engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                    if row.number == faulted && row.phase == tasks::Phase::Active(tasks::Active::Due))
                })
            {
                let answer = self.notes_read(jig_domain_core::reads::Read::Task { task: faulted });
                let jig_domain_core::reads::Answer::Document(doc) = answer else {
                    return Err(format!("held goal document: {answer:?}"));
                };
                let jig_domain_core::documents::Document::Task(doc) = *doc else {
                    return Err("held goal was not a task".into());
                };
                if matches!(doc.run_wait, Some(jig_domain_core::documents::TaskRunWait::Account { account: Some(account), .. }) if account == struck)
                {
                    return Ok(());
                }
                return Err(format!("goal {faulted} does not name account {struck}: {:?}", doc.run_wait));
            }
            let (from, step) = fault.cleared_at.unwrap_or((began, 0));
            if story.steps.saturating_sub(step) > recovery_steps || self.env.now > from.saturating_add(recovery_time) {
                break;
            }
        }
        Err(format!(
            "account {struck} recovery exceeded RECOVERY_STEPS; assignments {:?}; tasks {:?}",
            self.account_case
                .as_ref()
                .expect("accounts")
                .assignments
                .iter()
                .map(|a| (a.task, a.attempt))
                .collect::<Vec<_>>(),
            self.store
                .rows
                .values()
                .filter_map(|row| match row {
                    engine::Record::Core(jig_domain_core::Record::Tasks(
                        tasks::Stored::Live(row) | tasks::Stored::Ended(row),
                    )) => Some((row.number, row.phase.clone(), row.tries)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        ))
    }

    fn account_step(&mut self) {
        if !self.process.prepared()
            && self.store.idle()
            && self.root_events.is_empty()
            && self.worker_events.is_empty()
            && let Some(next) =
                [self.process.wake_at(), self.root.next_deadline(&self.env), self.worker.next_deadline()]
                    .into_iter()
                    .flatten()
                    .min()
        {
            let clearing = self
                .account_case
                .as_ref()
                .and_then(|story| story.fault.as_ref())
                .is_some_and(|fault| fault.hit.is_some() && fault.clear_after.is_some() && !fault.cleared);
            let next = next.min(self.env.now.saturating_add(if clearing {
                Duration::from_millis(1)
            } else {
                Duration::from_secs(1)
            }));
            if next > self.env.now {
                self.env.now = next;
            }
        }
        self.tick();
    }

    fn source_held(&self, account: u32) -> bool {
        self.store.rows.values().any(|row| matches!(row,
            engine::Record::Core(jig_domain_core::Record::Accounts(Stored::Source { account: number, held: Some(_), .. })) if *number == account))
    }

    /// Account failures attributed to their causal interval and smith's run table.
    pub fn judge_account_runs(&mut self) -> Result<(), String> {
        let story = self.account_case.as_mut().expect("two accounts");
        story.runs.judge(&mut story.ledger)?;
        for fault in story.ledger.entries() {
            if fault.count(Counter::TaskTries) != 0 {
                return Err(format!(
                    "{}/{} spent {} task tries",
                    fault.kind.family,
                    fault.kind.name,
                    fault.count(Counter::TaskTries)
                ));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn account_ledger(&self) -> &FaultLedger {
        &self.account_case.as_ref().expect("two accounts").ledger
    }

    /// Independently observed queries, named by their assigned task.
    #[must_use]
    pub fn account_queries(&self) -> &[(u64, Query)] {
        &self.account_case.as_ref().expect("two accounts").queries
    }

    /// The goal tasks created by the two real parties.
    #[must_use]
    pub fn account_goals(&self) -> [u64; 2] {
        [
            self.party.last_task.expect("first goal"),
            self.account_case.as_ref().expect("two accounts").other.last_task.expect("second goal"),
        ]
    }

    /// Whether the goal's successful ending is durable.
    #[must_use]
    pub fn account_goal_done(&self, task: u64) -> bool {
        self.store.rows.values().any(|row| {
            matches!(row,
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
                if row.number == task && matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_))))
        })
    }

    /// Let the real roots finish both finite, no-change goals.
    pub fn run_account_goals(&mut self) {
        for _ in 0..20_000 {
            self.account_step();
            if self.party.last_task.is_some()
                && self.account_case.as_ref().expect("two accounts").other.last_task.is_some()
                && self.account_goals().into_iter().all(|task| self.account_goal_done(task))
            {
                return;
            }
        }
        panic!("two account goals did not end: {:?}", self.observed);
    }
}

fn account(assignment: &engine::Assignment) -> u32 {
    assignment.run.accounts.first().expect("main account").account
}

fn run_name(assignment: &engine::Assignment) -> String {
    format!("{}:{}", assignment.task, assignment.attempt)
}

fn walking_goal(ask: people::Ask) -> people::Ask {
    super::walking::goal_from(
        ask,
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 40, priority: 1, deadline: None },
    )
}
