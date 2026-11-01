//! The stock agreed-goal walk through real tool admission and durable answers.
//! Contract: temper/docs/design/domain/agent.md, sections 4.6 and 8.2.
use super::{
    Duration, Finish, Key, Line, Process, ReplyTo, Script, Token, Turn, VecDeque, World, engine, fake, people, run,
    tasks, worker,
};
use std::fmt::Write;

fn config(seed: u64) -> engine::Config {
    let mut config = super::config(seed);
    let llm = jig_protocol_config::Llm {
        endpoints: Box::default(),
        providers: Box::default(),
        descriptions: Box::new([None]),
        models: config.core.settings.models.clone(),
    };
    config.core.defaults = temper_service::stock::defaults(&llm);
    config.core.connectors = Box::new([1]);
    config.forge_charter = 2;
    let mut policy = config.core.defaults.policies.get(&1).expect("stock policy").clone();
    let mut roles = policy.policy.roles.to_vec();
    for number in 1..4 {
        let mut role = roles[0].clone();
        role.number = number;
        roles.push(role);
    }
    policy.policy.roles = roles.into_boxed_slice();
    config.core.seed_projects[0].policy = policy;
    config
}

impl World {
    /// Delegate an agreed goal under stock policies, then run its recorded reads and question.
    #[must_use]
    pub fn agreed_goal(seed: u64, mut script: Script) -> Self {
        let chat = Script {
            cue: b"Help the person with their work.".as_slice().into(),
            turns: Box::new([
                call(b"goal", br#"{"words":"Add CSV export; ask the plan first","agree":true}"#),
                call(b"wait", b"{}"),
            ]),
        };
        let mut world = Self::configured_with(seed, chat.clone(), true, config);
        let head = world.forge.branch(b"org/repo", b"main").expect("main exists");
        let mut hex = String::new();
        for byte in skein_fake_forge_domain::object_bytes(head) {
            write!(&mut hex, "{byte:02x}").expect("writing to a string");
        }
        for turn in &mut script.turns {
            for line in &mut turn.lines {
                if let Line::Call { name, arguments } = line
                    && name.as_ref() == b"read_file"
                {
                    *arguments = String::from_utf8(arguments.to_vec())
                        .expect("fixture JSON")
                        .replace("0000000000000000000000000000000000000001", &hex)
                        .into_bytes()
                        .into_boxed_slice();
                }
            }
        }
        world.scripts = VecDeque::from([chat, script]);
        world
    }
}

/// One model completion that calls the named tool with the supplied JSON.
#[must_use]
pub fn call(name: &[u8], arguments: &[u8]) -> Turn {
    Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}

/// Worker capacities for stock inboxes and the recorded repository reads.
pub(super) fn worker_limits() -> worker::Limits {
    let mut limits = super::worker_limits();
    limits.bounds.message_bytes = 4096;
    limits.bounds.inbox = 8;
    limits.host.inbox = 8;
    limits.host.held = 8;
    limits.agent.messages = 8;
    limits.host.event_bytes = 4096;
    limits.agent.message_bytes = 8192;
    limits.reply_bytes = 8192;
    limits
}

// The world plays the person's inbox directly; the chat needs no notice activation.
/// Keep routine chat notices without requesting extra model activations.
pub(super) fn quiet(member: &mut people::Member) {
    member.wake.notices = people::WakeRule::Never;
    member.wake.news = people::WakeRule::Never;
    member.wake.results = people::ResultsWake::Never;
    member.wake.questions = false;
    member.wake.answers = false;
    member.wake.timers = false;
}

const ANSWER: Token = Token::new(902);

/// Acknowledge the direct fake person's durable inbox answer.
pub(super) fn answered(to: Token, reply: &people::Reply) -> bool {
    if to != ANSWER {
        return false;
    }
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::QuestionAnswered { .. })), "{reply:?}");
    true
}

impl World {
    /// Select the next task's script or restore its parked concrete history.
    pub(super) fn select_process(&mut self, task: u64) {
        if self.assignment.as_ref().is_some_and(|previous| previous.task == task) {
            if self.process.answered.is_some() {
                self.process.resume(Script { cue: Box::default(), turns: Box::default() });
            }
            return;
        }
        let next = if let Some(mut parked) = self.parked.remove(&task) {
            if !parked.prepared() {
                let turns = if task == 1 { Box::from([call(b"wait", b"{}")]) } else { Box::default() };
                parked.resume(Script { cue: Box::default(), turns });
            }
            parked
        } else if let Some(script) = self.scripts.pop_front() {
            Process::new(self.seed, script)
        } else {
            return;
        };
        let previous = core::mem::replace(&mut self.process, next);
        if let Some(assignment) = &self.assignment {
            assert!(
                previous.answered.is_some() || previous.prepared(),
                "one live process per slot: previous {}, next {task}",
                assignment.task
            );
            self.parked.insert(assignment.task, previous);
        }
    }

    /// Answer the pending plan through the authenticated person's inbox after parking.
    pub fn answer_goal(&mut self, words: &[u8], script: Script) {
        let task = self.assignment().task;
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            self.store.rows.get(&key)
        else {
            panic!("live goal");
        };
        let question = row.questions[0].number;
        let tasks::Party::Task(requester) = row.questions[0].asked else { panic!("question to the chat") };
        self.process.resume(script);
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::default(),
            reply_to: ReplyTo::new(ANSWER),
            sign_in: self.party.sign_in.expect("signed in"),
            key: [45; 16],
            ask: people::Ask::AnswerQuestion { project: 1, task: requester, question, words: words.into() },
        }));
    }

    /// Hold the person's answer past parking with no active process to drive.
    pub fn hold_goal(&mut self, duration: Duration) {
        assert!(matches!(self.agent().answer(), run::Answer::Parked { .. }));
        self.env.now = self.env.now.saturating_add(duration);
        self.tick();
    }

    /// Assert that the waiting plan owns its answer route and has made no change.
    pub fn pending_plan(&self) {
        let task = self.assignment().task;
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            self.store.rows.get(&key)
        else {
            panic!("the goal stays live");
        };
        assert!(row.asked_requester);
        assert_eq!(row.questions.len(), 1);
        assert_eq!(self.forge.branches(b"org/repo"), [b"main".to_vec()]);
        assert_eq!(self.observed.assignments, 2, "no change agent before the answer");
        assert!(
            !self.forge.observations().iter().any(|observation| matches!(
                observation,
                skein_fake_forge_domain::Observation::Opened { kind: fake::Kind::Pull, .. }
            )),
            "no pull request before the answer"
        );
    }

    /// Supply the next change agent's model script, without choosing its assignment.
    pub fn next_script(&mut self, script: Script) {
        self.scripts.push_back(script);
    }
}

impl World {
    /// Drive the change and the goal until the goal's successful result is durable.
    pub fn finish_goal(&mut self) {
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(2)));
        for _ in 0..10000 {
            if self.process.answered.is_some()
                && self.root_events.is_empty()
                && self.worker_events.is_empty()
                && !self.committing()
            {
                self.env.now = self.env.now.saturating_add(Duration::from_secs(1));
            }
            self.tick();
            if let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
                self.store.rows.get(&key)
            {
                assert!(matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_))), "{row:?}");
                return;
            }
        }
        panic!(
            "goal did not end: {:?}; task {}, attempt {}",
            self.observed,
            self.assignment().task,
            self.assignment().attempt
        );
    }
}

/// The fake CI completes one successful build for each pushed head.
pub(super) fn checks() -> fake::Checks {
    fake::Checks {
        cues: Box::default(),
        contexts: Box::new([b"build".as_slice().into()]),
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        silent: 0,
        passes: 1000,
        reruns: 0,
        cue: None,
    }
}

impl World {
    /// Verify the agreed file, pull request, successful goal and cumulative funding ledger.
    pub fn assert_goal_landed(&mut self) {
        let head = self.forge.branch(b"org/repo", b"main").expect("main exists");
        assert_eq!(
            self.forge.api(
                b"org/repo",
                1,
                fake::Op::Read(fake::Read::File { commit: head, path: b"export.csv".as_slice().into() })
            ),
            Ok(fake::Answer::File(b"fruit,count\n".as_slice().into()))
        );
        assert!(self.forge.observations().iter().any(|observation| matches!(
            observation,
            skein_fake_forge_domain::Observation::Opened { kind: fake::Kind::Pull, .. }
        )));
        assert!(
            self.forge
                .observations()
                .iter()
                .any(|observation| matches!(observation, skein_fake_forge_domain::Observation::Merged { .. }))
        );
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(2)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            self.store.rows.get(&key)
        else {
            panic!("ended goal");
        };
        let spent = self
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(
                    terminal,
                ))) if terminal.task == 2 => Some(terminal.cumulative),
                engine::Record::Core(_) | engine::Record::Forge { .. } => None,
            })
            .sum::<u64>();
        assert_eq!(row.numbers.spent, spent, "activations add to the same monetary ledger");
        assert!(spent > 2);
        assert!(spent + row.numbers.spent_below <= row.numbers.budget);
        let prompt = &self.agent().prompts()[0];
        assert_eq!(prompt.choice, skein_fake_llm_domain::api::ToolChoice::Auto);
        assert!(String::from_utf8_lossy(&prompt.system).contains("At open, you have 8 turns"));
        assert!(prompt.messages.iter().flat_map(|message| &message.parts).any(|part| {
            let bytes = match part {
                skein_fake_llm_domain::api::Part::Text { text } => text.as_ref(),
                skein_fake_llm_domain::api::Part::ToolOutput { output, .. } => output.as_ref(),
                skein_fake_llm_domain::api::Part::Opaque { .. } | skein_fake_llm_domain::api::Part::ToolCall { .. } => {
                    return false;
                }
            };
            String::from_utf8_lossy(bytes).contains("Earlier exhaustion and finish-now instructions are superseded")
        }));
    }
}
