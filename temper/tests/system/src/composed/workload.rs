//! The composed workload and finite faults (temper/docs/design/testing/workloads.md, 5, and temper/docs/design/testing/workloads-sweeps.md, 1.2).
use super::{Key, Process, Script, Token, World, engine, host, link, parties, people, tasks};
use skein_lib::Duration;
use skein_world::streams::{Stream, Streams};
use std::collections::{BTreeMap, BTreeSet};
use temper_engine_domain_world::ends::{Bound, Expect};
use temper_fake_checkout::git::Remote;
use temper_workload::{
    draw::{COMPOSED, Goal, Party, Workload},
    names::Names,
};

mod comparison;
pub mod faults;
mod questions;
mod scripts;
mod setup;

// The pinned calm runs end within 18 simulated minutes and 5,301 steps,
// including the shared worker, preceding milestone goal and final tier run.
// Sixty minutes and 30,000 steps give at least threefold and fivefold headroom.
pub const GOAL_BOUND: Bound = Bound { minutes: 60, steps: 30_000 };
const REPLY: u64 = 3000;
const REMOTE: &[u8] = b"org/repo";

pub(super) struct Story {
    workload: Workload,
    producer_seeds: skein_fake_llm_domain::api::Seeds,
    names: Names,
    other: parties::Party,
    replies: BTreeMap<u64, people::Reply>,
    next_reply: u64,
    answered: BTreeSet<u64>,
    widened: BTreeSet<u64>,
    pending_asks: BTreeSet<u64>,
    assignments: Vec<engine::Assignment>,
    milestone: Option<u64>,
    chats: BTreeMap<Party, u64>,
    latch: Option<u64>,
    latched: BTreeSet<u64>,
    linked: BTreeSet<u64>,
    comparison: Option<comparison::Run>,
    named_at: Option<u64>,
}

impl World {
    /// Draw the kit's three goals independently of the world's scheduling.
    #[must_use]
    pub fn composed_workload(seed: u64) -> Self {
        Self::workload_with_faults(seed, &[])
    }

    fn workload_with_faults(seed: u64, kinds: &[skein_world::faults::Kind]) -> Self {
        Self::workload_with_restart(seed, kinds, None)
    }

    fn workload_with_restart(seed: u64, kinds: &[skein_world::faults::Kind], points: Option<(usize, usize)>) -> Self {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let scheduling = streams.seed_of(Stream::Scheduling, "system");
        let draw = (!kinds.is_empty()).then(|| faults::FaultDraw::for_workload(seed, &workload, kinds));
        let park_cell = draw.as_ref().is_some_and(faults::FaultDraw::has_park_cell);
        let mut config = setup::config(scheduling, &workload, park_cell);
        if draw.as_ref().is_some_and(|draw| draw.plans(temper_blame::SILENT)) {
            // Let the real timeout hold the landing within the workload's bound.
            temper_engine_domain_world::oracle::config::landing(&mut config, 1).gate_timeout_ms = 60_000;
        }
        let planned_bounds = draw.as_ref().map_or(&[][..], faults::FaultDraw::bounds);
        let close_refusal = draw.as_ref().is_some_and(|draw| draw.plans(temper_faults::forge::CLOSE_REFUSED));
        let world = Self::configured_values(scheduling, scripts::idle(), true, config, |limits| {
            setup::limits(limits, planned_bounds, &workload, close_refusal);
        });
        let mut world = Self::with_two_accounts(world);
        world.account_case.take().expect("two people");
        world.party = setup::party(1);
        let other = setup::party(2);
        let comparison = Some(comparison::Run::new(seed, &workload, points));
        world.workload = Some(Story {
            workload,
            producer_seeds: setup::producer_seeds(streams),
            comparison,
            named_at: None,
            names: Names::new(),
            other,
            replies: BTreeMap::new(),
            next_reply: REPLY,
            answered: BTreeSet::new(),
            widened: BTreeSet::new(),
            pending_asks: BTreeSet::new(),
            assignments: Vec::new(),
            milestone: None,
            chats: BTreeMap::new(),
            latch: None,
            latched: BTreeSet::new(),
            linked: BTreeSet::new(),
        });
        setup::forge(&mut world);
        // Both comparison sides use the same worker hello/admission path.
        world.absence.enabled = true;
        world.workload_faults = draw.map(|draw| faults::Faults::new(seed, draw));
        world.start_workload();
        world
    }

    /// The immutable draw whose logical names appear in every script's prompt.
    #[must_use]
    pub fn workload(&self) -> &Workload {
        &self.workload.as_ref().expect("workload world").workload
    }

    #[must_use]
    pub fn workload_names(&self) -> &Names {
        &self.workload.as_ref().expect("workload world").names
    }

    fn workload_ask(&mut self, party: Party, ask: people::Ask) -> people::Reply {
        let story = self.workload.as_mut().expect("workload world");
        let number = story.next_reply;
        story.next_reply += 1;
        let sign_in = match party {
            Party::First => self.party.sign_in,
            Party::Second => story.other.sign_in,
        }
        .expect("the party signed in");
        let mut key = [0; 16];
        key[..8].copy_from_slice(&number.to_le_bytes());
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::default(),
            reply_to: skein_lib::ReplyTo::new(Token::new(number)),
            sign_in,
            key,
            ask,
        }));
        for _ in 0..super::STEPS {
            self.tick();
            if let Some(reply) = self.workload.as_mut().expect("workload").replies.remove(&number) {
                return reply;
            }
        }
        panic!("workload request {number} was not answered");
    }

    fn start_workload(&mut self) {
        for _ in 0..super::STEPS {
            self.tick();
            if self.party.sign_in.is_some()
                && self.workload.as_ref().expect("workload").other.sign_in.is_some()
                && self.adoption == super::Adoption::Adopted
            {
                break;
            }
        }
        // A milestone's financial payer is its owner, even when the other
        // person delegates a goal. Each key owner permits this project's work;
        // explicit choices below still pin every goal to its party's account.
        for party in [Party::First, Party::Second] {
            let reply = self.workload_ask(
                party,
                people::Ask::AccountChange {
                    holder: people::AccountHolder::Own,
                    change: people::AccountChange::SetUses {
                        account: setup::account(party),
                        uses: Some(Box::new([people::Use::Project(1)])),
                    },
                },
            );
            assert!(!matches!(reply, people::Reply::Outcome(people::Outcome::Refused(_))), "key uses: {reply:?}");
        }
        let train = self.workload().goals.iter().find(|goal| goal.train).expect("train").clone();
        for party in [Party::First, Party::Second] {
            let words = if party == train.party {
                format!("@chat:{}@ Run the train goal.", train.name)
            } else {
                "@chat:questions@ Answer workload questions.".to_owned()
            };
            let ask = self.chat_request(1, words.as_bytes());
            let task = created(&self.workload_ask(party, ask));
            self.workload.as_mut().expect("workload").chats.insert(party, task);
        }
        let milestone = self.workload_ask(Party::First, setup::milestone(self));
        let milestone = created(&milestone);
        self.workload.as_mut().expect("workload").milestone = Some(milestone);
        let goals = self.workload().goals.clone();
        for goal in goals.iter().filter(|goal| !goal.train) {
            let ask = setup::goal(self, goal, milestone);
            let task = created(&self.workload_ask(goal.party, ask));
            self.name_goal(&goal.name, task);
        }
    }

    fn name_goal(&mut self, name: &str, task: u64) {
        let story = self.workload.as_mut().expect("workload");
        if story.names.task(name).is_none() {
            story.names.name(name, task).expect("unique goal task");
            self.ends.expect(task, Expect::Ends, GOAL_BOUND).expect("nonzero bound");
        }
    }

    pub(super) fn current_workload_request(&mut self, event: engine::Event) -> Option<engine::Event> {
        let engine::Event::Party(engine::PartyInput::Ask {
            reply_to,
            sign_in,
            key,
            models,
            ask: ask @ people::Ask::Release { task, revision, .. },
        }) = event
        else {
            return Some(event);
        };
        let token = reply_to.into_token();
        // Only queued forge recoveries are in pending_asks here. Another
        // recovery can release or change the hold before this one is consumed.
        if let Some(story) = &mut self.workload
            && story.pending_asks.contains(&token.raw())
            && !self
                .root
                .tasks()
                .task(task)
                .is_some_and(|row| row.revisions.hold == revision && matches!(row.phase, tasks::Phase::Held { .. }))
        {
            story.pending_asks.remove(&token.raw());
            return None;
        }
        Some(engine::Event::Party(engine::PartyInput::Ask {
            reply_to: skein_lib::ReplyTo::new(token),
            sign_in,
            key,
            models,
            ask,
        }))
    }

    pub(super) fn workload_reply(&mut self, to: Token, reply: &people::Reply) -> bool {
        let Some(story) = &mut self.workload else { return false };
        if to.raw() >= REPLY {
            if story.pending_asks.remove(&to.raw())
                && matches!(reply, people::Reply::Outcome(people::Outcome::Refused(_)) | people::Reply::Refused(_))
                && self.finding.is_none()
            {
                self.finding = Some(format!("workload action refused: {reply:?}"));
            }
            story.replies.insert(to.raw(), reply.clone());
            return true;
        }
        // The second sign-in uses a separate fake party; adoption uses the first.
        if to.raw() == setup::SECOND_REPLY {
            let people::Reply::SignedIn { person, sign_in, .. } = reply else { panic!("second sign-in: {reply:?}") };
            story.other.reply(to.raw(), parties::Reply::SignedIn { person: *person, sign_in: *sign_in });
            return true;
        }
        false
    }

    pub(super) fn workload_tick(&mut self) {
        if self.workload.is_none() {
            return;
        }
        if self.opened
            && self.adoption != super::Adoption::Pending
            && let Some(up) = self.workload.as_mut().expect("workload").other.tick(self.env.now)
        {
            self.party_request(up);
        }
        self.discover_workload_names();
        self.link_workload_changes();
        self.prepare_workload_questions();
        // These scripts enqueue requests without changing the live rows.
        self.answer_workload_questions();
        self.workload_answer_park();
        self.latch_workload();
        self.release_branch_load();
        self.workload_comparison_tick();
    }

    fn workload_tasks(&self) -> impl Iterator<Item = &tasks::TaskRecord> {
        let tasks = self.root.tasks();
        std::iter::successors(tasks.inspection_task_after(None), |row| tasks.inspection_task_after(Some(row.number)))
    }

    fn discover_workload_names(&mut self) {
        let story = self.workload.as_mut().expect("workload");
        if story.named_at == Some(self.store.applied) {
            return;
        }
        // Discovery reads only committed rows. A new commit may add an as-yet
        // unnamed task; unchanged rows cannot reveal another name.
        story.named_at = Some(self.store.applied);
        for index in 0..self.workload().goals.len() {
            let goal = &self.workload().goals[index];
            if self.workload_names().task(&goal.name).is_none() {
                let marker = scripts::goal_cue(goal);
                let task = super::task_rows(&self.store)
                    .find(|row| row.spec.words.starts_with(marker.as_bytes()))
                    .map(|row| row.number);
                if let Some(task) = task {
                    let name = goal.name.clone();
                    self.name_goal(&name, task);
                }
            }
            for change_index in 0..self.workload().goals[index].changes.len() {
                let change = &self.workload().goals[index].changes[change_index];
                if self.workload_names().task(&change.name).is_some() {
                    continue;
                }
                let cue = scripts::change_cue(change);
                let task = super::task_rows(&self.store)
                    .find(|row| {
                        matches!(row.executor, tasks::Executor::Procedure { connector: 1, code: 2 })
                            && String::from_utf8_lossy(&row.spec.words).contains(&cue)
                    })
                    .map(|row| row.number);
                if let Some(task) = task {
                    let name = change.name.clone();
                    self.workload.as_mut().expect("workload").names.name(&name, task).expect("unique change");
                }
            }
        }
    }

    pub(super) fn workload_assignment(&mut self, assignment: &engine::Assignment) -> bool {
        if self.workload.is_none() {
            return false;
        }
        self.discover_workload_names();
        if let Some(name) = self.workload_names().logical(assignment.task)
            && let Some(goal) = self.workload().goal(name)
            && let Some(after) = &goal.after
        {
            let earlier = self.workload_names().task(after).expect("earlier milestone goal");
            assert!(
                self.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(earlier)))),
                "{name} starts after {after} lands"
            );
        }
        if !self.workload_park_deferred() {
            self.workload_choose_park(assignment);
        }
        let mut script = scripts::assigned(self, assignment);
        let park_cell_job = self.workload_cell_script(assignment, &mut script);
        let mut process = if self.lanes[0].assignment.as_ref().is_some_and(|previous| previous.task == assignment.task)
        {
            if self.lanes[0].process.answered.is_some() {
                self.lanes[0].process.resume(script);
            }
            self.workload.as_mut().expect("workload").assignments.push(assignment.clone());
            self.workload_fault_assignment(assignment);
            self.workload_comparison_assignment(assignment);
            return true;
        } else if let Some(mut parked) = self.parked.remove(&assignment.task) {
            if !parked.prepared() {
                parked.resume(script);
            }
            parked
        } else {
            let draw = (!super::stretches::plan(&script).is_empty())
                .then(|| self.workload.as_ref().expect("workload").producer_seeds);
            let mut process = Process::new(self.seed, script);
            process.draw = draw;
            process.park_cell_job = park_cell_job;
            process
        };
        process.configure_settings = Some(|settings| {
            settings.provider.accounts = 2;
        });
        process.clocked = true;
        process.bounded_clock = true;
        let previous = core::mem::replace(&mut self.lanes[0].process, process);
        if let Some(previous_assignment) = &self.lanes[0].assignment {
            assert!(previous.answered.is_some() || previous.prepared(), "one live process");
            self.parked.insert(previous_assignment.task, previous);
        }
        self.workload.as_mut().expect("workload").assignments.push(assignment.clone());
        self.workload_fault_assignment(assignment);
        self.workload_comparison_assignment(assignment);
        true
    }

    pub(super) fn workload_job(
        &mut self,
        channel: Token,
        job: host::JobAssignment,
        workspace: Option<&engine::ForgeWorkspace>,
    ) {
        assert_eq!(channel, super::CHANNEL);
        assert_eq!(job.kind.0, 1, "the built-in train prepare");
        let limits = temper_worker_service::worker_settings::payload_limits(
            &temper_worker_service::worker_settings::Limits::shared(temper_service_core::limits::stock()),
        )
        .expect("worker payload limits");
        self.lanes[0].worker_events.push_back(link::job(job, workspace, &limits));
    }

    /// End only after all goals landed and the tier caught main's final head.
    pub fn run_workload(&mut self) -> Result<(), String> {
        for _ in 0..GOAL_BOUND.steps {
            self.try_move_to(self.env.now.saturating_add(Duration::from_millis(
                if self.workload_faults.as_ref().is_some_and(faults::Faults::needs_small_steps) { 1 } else { 100 },
            )))
            .and_then(|()| self.tick_at_now())
            .map_err(|finding| format!("{finding}\n{}", self.workload_state()))?;
            if self.workload_finished() {
                self.retire_workload_plans();
                if self.workload_faults_stopped() {
                    return self.ends.close(self.env.now, &self.store);
                }
            }
            if self.engine_quiet() && self.lanes[0].worker_events.is_empty() {
                let next = [
                    self.root.next_deadline(&self.env),
                    self.lanes[0].worker.next_deadline(),
                    self.lanes[0].process.wake_at(),
                    Some(self.env.now.saturating_add(
                        if self.workload_faults.as_ref().is_some_and(faults::Faults::needs_small_steps) {
                            Duration::from_millis(1)
                        } else {
                            Duration::from_secs(1)
                        },
                    )),
                ]
                .into_iter()
                .flatten()
                .filter(|at| *at > self.env.now)
                .min()
                .expect("clock cap");
                self.try_move_to(next)?;
            }
        }
        Err(format!("workload did not finish: {}", self.workload_state()))
    }

    fn workload_finished(&self) -> bool {
        self.workload_goals_ended() && self.workload_tier_green()
    }

    fn workload_goals_ended(&self) -> bool {
        self.workload()
            .goals
            .iter()
            .all(|goal| self.workload_names().task(&goal.name).is_some_and(|task| self.workload_task_ended(task)))
    }

    fn workload_task_ended(&self, task: u64) -> bool {
        self.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
    }

    fn workload_landings(&self) -> impl Iterator<Item = &temper_engine_domain_forge::LandingRow> {
        use temper_engine_domain_forge::{Key as ForgeKey, Stored};
        self.store.rows.range(Key::Forge(ForgeKey::Landing(0))..=Key::Forge(ForgeKey::Landing(u64::MAX))).filter_map(
            |(_, record)| match record {
                engine::Record::Forge { row } => match row.as_ref() {
                    Stored::Landing(row) => Some(row),
                    _ => None,
                },
                engine::Record::Core(_) => None,
            },
        )
    }

    #[must_use]
    pub fn workload_tier_green(&self) -> bool {
        setup::tier_green(self)
    }

    /// Judge the durable results, final tree, accounts and the train's membership.
    pub fn assert_workload(&self) -> Result<(), String> {
        if !self.workload_tier_green() {
            return Err("the after-landing tier did not follow main".into());
        }
        let main = self.forge.branch(REMOTE, b"main").ok_or("no main branch")?;
        let tree = self.forge.tree(main);
        let story = self.workload.as_ref().expect("workload");
        for goal in &story.workload.goals {
            let task = story.names.task(&goal.name).expect("goal named");
            let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
            let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
                self.store.rows.get(&key)
            else {
                return Err(format!("goal {} did not end", goal.name));
            };
            if !matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_))) {
                return Err(format!("{}: {:?}", goal.name, row.phase));
            }
            eprintln!(
                "{} task {task}: ended {:?}; run at {:?}, {} steps; bound {GOAL_BOUND:?}",
                goal.name, row.ended_at, self.env.now, self.iteration
            );
            for assignment in story.assignments.iter().filter(|assignment| assignment.task == task) {
                let account = assignment.run.accounts.first().map(|account| account.account);
                let expected = setup::account(goal.party);
                if account != Some(expected) {
                    return Err(format!("{}: expected account {expected}, found {account:?}", goal.name));
                }
            }
            let changes = goal
                .changes
                .iter()
                .map(|change| self.assert_workload_change(goal, change, &tree))
                .collect::<Result<Vec<_>, _>>()?;
            if goal.train {
                self.assert_workload_train(task, &changes)?;
            }
        }
        self.observed.stretches.check()
    }

    fn assert_workload_change(
        &self,
        goal: &Goal,
        change: &temper_workload::draw::Change,
        tree: &super::Tree,
    ) -> Result<u64, String> {
        let story = self.workload.as_ref().expect("workload");
        let task = story.names.task(&change.name).ok_or_else(|| format!("change {} was not named", change.name))?;
        let row = self.store.change(task).ok_or_else(|| format!("change {} was not kept", change.name))?;
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) =
            self.store.rows.get(&key)
        else {
            return Err(format!("change {} did not end", change.name));
        };
        for after in &change.after {
            if !story.names.task(after).is_some_and(|task| ended.dependencies.contains(&task)) {
                return Err(format!("{} lost its after {after}", change.name));
            }
        }
        if !matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. }) {
            return Err(format!("{} did not land: {:?}", change.name, row.change.state));
        }
        let landing_repairs = story
            .assignments
            .iter()
            .filter(|assignment| {
                self.store
                    .rows
                    .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(assignment.task))))
                    .is_some_and(|record| match record {
                        engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) => {
                            String::from_utf8_lossy(&row.spec.words).contains(&format!("Failed test: {} ", change.name))
                        }
                        _ => false,
                    })
            })
            .map(|assignment| assignment.task)
            .collect::<BTreeSet<_>>()
            .len();
        let repairs = u64::from(row.change.repairs) + u64::try_from(landing_repairs).expect("bounded repairs");
        if repairs != u64::from(change.red_once) {
            return Err(format!("{} repaired {repairs} times (red once: {})", change.name, change.red_once));
        }
        if change.asks {
            let questions: Vec<_> = self
                .store
                .rows
                .values()
                .filter_map(|record| match record {
                    engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
                        if row.spec.words.starts_with(format!("@ask:{}@", change.name).as_bytes()) =>
                    {
                        Some(row)
                    }
                    _ => None,
                })
                .collect();
            let [question] = &questions[..] else {
                return Err(format!("{} asked its party {} times", change.name, questions.len()));
            };
            let person = if goal.party == Party::First { 1 } else { 2 };
            if question.executor != tasks::Executor::Person(tasks::PersonAddress::Person(person))
                || !matches!(&question.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) if words.as_ref() == b"Yes, deliver the named files.")
            {
                return Err(format!("{}: wrong party or answer to producer's question", change.name));
            }
        }
        for file in &change.files {
            if tree.get(file.path.as_bytes()).map(Vec::as_slice) != Some(file.text.as_bytes()) {
                return Err(format!("{} has the wrong text on main", file.path));
            }
        }
        Ok(task)
    }

    fn assert_workload_train(&self, task: u64, changes: &[u64]) -> Result<(), String> {
        use temper_engine_domain_forge::LandingLeave;
        use temper_engine_domain_forge_change::State;
        let story = self.workload.as_ref().expect("workload");
        if story.latch.is_none() {
            return Err(format!("train coordinator {task} made no latch"));
        }
        let mut trains = self
            .workload_landings()
            .filter(|row| changes.iter().all(|task| row.landing.members.iter().any(|member| member.change == *task)));
        let Some(train) = trains.next() else { return Err(format!("no train gathered {changes:?}")) };
        if trains.next().is_some() {
            return Err(format!("more than one train gathered {changes:?}"));
        }
        for &change in changes {
            let mut landed = self.workload_landings().filter(|row| {
                row.landing.members.iter().any(|member| member.change == change && member.landed.is_some())
            });
            let Some(row) = landed.next() else { return Err(format!("train member {change} did not land")) };
            if landed.next().is_some() {
                return Err(format!("train member {change} landed more than once"));
            }
            let member = train.landing.members.iter().find(|member| member.change == change).expect("train member");
            if member.left == Some(LandingLeave::Held) {
                let release = self.workload_faults.as_ref().and_then(|f| f.released_member);
                let Some((released, at)) = release else {
                    return Err(format!("held train member {change} had no release"));
                };
                let state = self.store.change(change).expect("change kept").change.state;
                if released != change
                    || row.landing.task <= train.landing.task
                    || !matches!(state, State::Landed { since, .. } if since.as_nanos() >= at.as_nanos())
                {
                    return Err(format!("held train member {change} did not land later, after release"));
                }
            } else if row.landing.task != train.landing.task {
                return Err(format!("train member {change} split from its train without leaving on hold"));
            }
        }
        Ok(())
    }

    fn workload_state(&self) -> String {
        self.store
            .rows
            .values()
            .filter_map(|row| match row {
                engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) => Some(format!(
                    "{} {:?} questions {:?} {:?}",
                    row.number,
                    row.phase,
                    row.questions,
                    String::from_utf8_lossy(&row.spec.words)
                )),
                engine::Record::Forge { row } => match row.as_ref() {
                    temper_engine_domain_forge::Stored::ChangeHot(row) => {
                        Some(format!("change {} {:?} delegate {:?}", row.task, row.change.state, row.delegate))
                    }
                    temper_engine_domain_forge::Stored::Landing(row) => Some(format!("{row:?}")),
                    _ => None,
                },
                engine::Record::Core(_) => None,
            })
            .chain(
                (!self.lanes[0].process.prepared())
                    .then(|| self.lanes[0].process.agent())
                    .into_iter()
                    .filter_map(|agent| agent.prompts().last())
                    .flat_map(|prompt| &prompt.messages)
                    .flat_map(|message| &message.parts)
                    .filter_map(|part| match part {
                        skein_fake_llm_domain::api::Part::ToolOutput { output, .. } => {
                            Some(String::from_utf8_lossy(output).into_owned())
                        }
                        _ => None,
                    }),
            )
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn created(reply: &people::Reply) -> u64 {
    let people::Reply::Outcome(people::Outcome::Created { first, last, .. }) = reply else {
        panic!("created: {reply:?}")
    };
    assert_eq!(first, last);
    *first
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_commit_reveals_an_unnamed_change() {
        let mut world = World::composed_workload(0);
        let name = "g1.c0";
        // Complete the setup's queued commits before adding the store fixture.
        while !world.store.pending.is_empty() {
            world.store.tick(false).expect("setup commit");
        }
        world.discover_workload_names();
        assert_eq!(world.workload_names().task(name), None);
        let mut row = super::super::task_rows(&world.store).next().expect("opened task").clone();
        row.number = world.root.deployment().tasks + 1;
        row.executor = tasks::Executor::Procedure { connector: 1, code: 2 };
        row.spec.words = format!("@change:{name}@").into_bytes().into();
        let number = row.number;
        world.store.seed(
            Box::new([engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(row))))]),
            Box::default(),
        );
        world.discover_workload_names();
        assert_eq!(world.workload_names().task(name), Some(number));
    }
}
