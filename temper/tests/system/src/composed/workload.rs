//! The composed workload and finite faults (temper/docs/design/testing/workloads.md, 5 and 8.2).
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
}

impl World {
    /// Draw the kit's three goals independently of the world's scheduling.
    #[must_use]
    pub fn composed_workload(seed: u64) -> Self {
        Self::workload_with_faults(seed, &[])
    }

    fn workload_with_faults(seed: u64, kinds: &[skein_world::faults::Kind]) -> Self {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let scheduling = streams.seed_of(Stream::Scheduling, "system");
        let config = setup::config(scheduling, &workload);
        let world = Self::configured_values(scheduling, scripts::idle(), true, config, setup::limits);
        let mut world = Self::with_two_accounts(world);
        world.account_case.take().expect("two people");
        world.party = setup::party(1);
        let other = setup::party(2);
        world.workload = Some(Story {
            workload,
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
        if !kinds.is_empty() {
            world.absence.enabled = true;
            world.workload_faults = Some(faults::Faults::new(seed, world.workload(), kinds));
        }
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
            self.name_goal(goal, task);
        }
    }

    fn name_goal(&mut self, goal: &Goal, task: u64) {
        let story = self.workload.as_mut().expect("workload");
        if story.names.task(&goal.name).is_none() {
            story.names.name(&goal.name, task).expect("unique goal task");
            self.ends.expect(task, Expect::Ends, GOAL_BOUND).expect("nonzero bound");
        }
    }

    pub(super) fn workload_reply(&mut self, to: Token, reply: &people::Reply) -> bool {
        let Some(story) = &mut self.workload else { return false };
        if to.raw() >= REPLY {
            if story.pending_asks.remove(&to.raw()) {
                assert!(
                    !matches!(reply, people::Reply::Outcome(people::Outcome::Refused(_)) | people::Reply::Refused(_)),
                    "workload action refused: {reply:?}"
                );
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
        self.answer_workload_questions();
        self.latch_workload();
    }

    fn discover_workload_names(&mut self) {
        let goals = self.workload().goals.clone();
        for goal in &goals {
            if self.workload_names().task(&goal.name).is_none() {
                let marker = scripts::goal_cue(goal);
                let task = self.store.rows.values().find_map(|row| match row {
                    engine::Record::Core(jig_domain_core::Record::Tasks(
                        tasks::Stored::Live(row) | tasks::Stored::Ended(row),
                    )) if row.spec.words.starts_with(marker.as_bytes()) => Some(row.number),
                    _ => None,
                });
                if let Some(task) = task {
                    self.name_goal(goal, task);
                }
            }
            for change in &goal.changes {
                if self.workload_names().task(&change.name).is_some() {
                    continue;
                }
                let cue = scripts::change_cue(change);
                let task = self.store.rows.values().find_map(|row| match row {
                    engine::Record::Core(jig_domain_core::Record::Tasks(
                        tasks::Stored::Live(row) | tasks::Stored::Ended(row),
                    )) if matches!(row.executor, tasks::Executor::Procedure { connector: 1, code: 2 })
                        && String::from_utf8_lossy(&row.spec.words).contains(&cue) =>
                    {
                        Some(row.number)
                    }
                    _ => None,
                });
                if let Some(task) = task {
                    self.workload.as_mut().expect("workload").names.name(&change.name, task).expect("unique change");
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
        let script = scripts::assigned(self, assignment);
        let mut process = if self.assignment.as_ref().is_some_and(|previous| previous.task == assignment.task) {
            if self.process.answered.is_some() {
                self.process.resume(script);
            }
            self.workload.as_mut().expect("workload").assignments.push(assignment.clone());
            self.workload_fault_assignment(assignment);
            return true;
        } else if let Some(mut parked) = self.parked.remove(&assignment.task) {
            if !parked.prepared() {
                parked.resume(script);
            }
            parked
        } else {
            Process::new(self.seed, script)
        };
        process.configure_settings = Some(|settings| {
            settings.provider.accounts = 2;
        });
        process.clocked = true;
        process.bounded_clock = true;
        let previous = core::mem::replace(&mut self.process, process);
        if let Some(previous_assignment) = &self.assignment {
            assert!(previous.answered.is_some() || previous.prepared(), "one live process");
            self.parked.insert(previous_assignment.task, previous);
        }
        self.workload.as_mut().expect("workload").assignments.push(assignment.clone());
        self.workload_fault_assignment(assignment);
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
        self.worker_events.push_back(link::job(job, workspace, &limits));
    }

    /// End only after all goals landed and the tier caught main's final head.
    pub fn run_workload(&mut self) {
        for _ in 0..GOAL_BOUND.steps {
            self.try_move_to(self.env.now.saturating_add(Duration::from_millis(
                if self.workload_faults.as_ref().is_some_and(faults::Faults::needs_small_steps) { 1 } else { 100 },
            )))
            .and_then(|()| self.tick_at_now())
            .unwrap_or_else(|finding| panic!("{finding}\n{}", self.workload_state()));
            if self.workload_finished() && self.workload_faults_stopped() {
                self.ends.close(self.env.now, &self.store).expect("all goal bounds met");
                return;
            }
            if self.engine_quiet() && self.worker_events.is_empty() {
                let next = [
                    self.root.next_deadline(&self.env),
                    self.worker.next_deadline(),
                    self.process.wake_at(),
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
                self.try_move_to(next).expect("visit the engine's retained bounds");
            }
        }
        panic!("workload did not finish: {}", self.workload_state());
    }

    fn workload_finished(&self) -> bool {
        self.workload().goals.iter().all(|goal| {
            self.workload_names().task(&goal.name).is_some_and(|task| {
                self.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
            })
        }) && self.workload_tier_green()
    }

    #[must_use]
    pub fn workload_tier_green(&self) -> bool {
        setup::tier_green(self)
    }

    /// Judge the durable results, final tree, accounts and the train's membership.
    pub fn assert_workload(&self) {
        assert!(self.workload_tier_green(), "the after-landing tier follows main");
        let main = self.forge.branch(REMOTE, b"main").expect("main");
        let tree = self.forge.tree(main);
        let story = self.workload.as_ref().expect("workload");
        for goal in &story.workload.goals {
            let task = story.names.task(&goal.name).expect("goal named");
            let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
            let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
                self.store.rows.get(&key)
            else {
                panic!("goal {} did not end", goal.name)
            };
            assert!(matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_))), "{}: {:?}", goal.name, row.phase);
            eprintln!(
                "{} task {task}: ended {:?}; run at {:?}, {} steps; bound {GOAL_BOUND:?}",
                goal.name, row.ended_at, self.env.now, self.iteration
            );
            for assignment in story.assignments.iter().filter(|assignment| assignment.task == task) {
                assert_eq!(
                    assignment.run.accounts.first().expect("account").account,
                    setup::account(goal.party),
                    "{} account",
                    goal.name
                );
            }
            let changes: Vec<_> =
                goal.changes.iter().map(|change| self.assert_workload_change(goal, change, &tree)).collect();
            if goal.train {
                self.assert_workload_train(task, &changes);
            }
        }
    }

    fn assert_workload_change(&self, goal: &Goal, change: &temper_workload::draw::Change, tree: &super::Tree) -> u64 {
        let story = self.workload.as_ref().expect("workload");
        let task = story.names.task(&change.name).expect("change named");
        let row = self.store.change(task).expect("change kept");
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(ended)))) =
            self.store.rows.get(&key)
        else {
            panic!("ended change")
        };
        for after in &change.after {
            assert!(
                ended.dependencies.contains(&story.names.task(after).expect("named dependency")),
                "{} keeps its after {after}",
                change.name
            );
        }
        assert!(
            matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. }),
            "{}: {:?}",
            change.name,
            row.change.state
        );
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
        assert_eq!(
            u64::from(row.change.repairs) + u64::try_from(landing_repairs).expect("bounded repairs"),
            u64::from(change.red_once),
            "{} repaired once",
            change.name
        );
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
            assert_eq!(questions.len(), 1, "{} asks its party once", change.name);
            let person = if goal.party == Party::First { 1 } else { 2 };
            assert_eq!(questions[0].executor, tasks::Executor::Person(tasks::PersonAddress::Person(person)));
            assert!(
                matches!(&questions[0].phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) if words.as_ref() == b"Yes, deliver the named files.")
            );
        }
        for file in &change.files {
            assert_eq!(tree.get(file.path.as_bytes()), Some(&file.text.as_bytes().to_vec()), "{} on main", file.path);
        }
        task
    }

    fn assert_workload_train(&self, task: u64, changes: &[u64]) {
        let story = self.workload.as_ref().expect("workload");
        let process = self.parked.get(&task).unwrap_or(&self.process);
        let outputs: Vec<_> = process
            .agent()
            .prompts()
            .last()
            .into_iter()
            .flat_map(|p| &p.messages)
            .flat_map(|m| &m.parts)
            .filter_map(|part| match part {
                skein_fake_llm_domain::api::Part::ToolOutput { output, .. } => Some(String::from_utf8_lossy(output)),
                _ => None,
            })
            .collect();
        assert!(story.latch.is_some(), "the coordinator makes a latch: {outputs:?}");
        let trains: Vec<_> = self
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                engine::Record::Forge { row } => match row.as_ref() {
                    temper_engine_domain_forge::Stored::Landing(row)
                        if changes
                            .iter()
                            .all(|task| row.landing.members.iter().any(|member| member.change == *task)) =>
                    {
                        Some(row)
                    }
                    _ => None,
                },
                engine::Record::Core(_) => None,
            })
            .collect();
        assert_eq!(
            trains.len(),
            1,
            "all train members land together: {:?}",
            self.store
                .rows
                .values()
                .filter_map(|record| match record {
                    engine::Record::Forge { row } => match row.as_ref() {
                        temper_engine_domain_forge::Stored::Landing(row) => Some((
                            row.landing.task,
                            row.last_event,
                            row.landing.members.iter().map(|member| member.change).collect::<Vec<_>>()
                        )),
                        _ => None,
                    },
                    engine::Record::Core(_) => None,
                })
                .collect::<Vec<_>>()
        );
        assert!(trains[0].landing.members.iter().all(|member| member.landed.is_some()), "the complete train landed");
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
                (!self.process.prepared())
                    .then(|| self.process.agent())
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
