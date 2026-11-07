//! Picker amendments cross both roots to the next Open activation.
//! Contract: jig/docs/design/domain/tasks.md, 7; smith/docs/design/domain/session.md, 8.2.
use super::{Duration, Key, ReplyTo, Script, Token, World, engine, parties, people, tasks};

const PICK: Token = Token::new(950);
// The composed world's bounded settling loop, with one millisecond per tick.
const STEPS: u32 = 10000;

fn config(seed: u64) -> engine::Config {
    let mut config = super::open_config(seed);
    let charter = &mut config.core.defaults.charters[0];
    charter.policy.model.efforts = Box::new([b"low".as_slice().into(), b"high".as_slice().into()]);
    charter.policy.model.effort = Some(b"low".as_slice().into());
    let mut second = charter.policy.model.clone();
    second.name = b"fake-2".as_slice().into();
    charter.policy.alternatives = Box::new([second.clone()]);
    config.core.settings.models = Box::new([charter.policy.model.clone(), second]);
    config
}

pub(super) fn picked(to: Token, reply: &people::Reply) -> bool {
    if to != PICK {
        return false;
    }
    assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Amended { .. })), "{reply:?}");
    true
}

impl World {
    /// An Open chat whose charter allows `fake-1` and `fake-2`.
    #[must_use]
    pub fn chat_on_two_models(seed: u64, script: Script) -> Self {
        let mut world = Self::configured_limits(seed, script, false, config, |_| {});
        world.process.clocked = true;
        world
    }

    /// Tick until the reply's turn is committed and the run still listens.
    pub fn until_listening(&mut self) {
        let previous = self.observed.turn_acks;
        for _ in 0..STEPS {
            self.tick();
            if self.observed.turn_acks > previous
                && self.observed.turns == self.observed.turn_acks
                && self.process.answered.is_none()
            {
                return;
            }
        }
        panic!("chat did not listen: {:?}", self.observed);
    }

    /// The person's new LLM choice, as the picker sends it: `Ask::Amend` with `llm` alone.
    pub fn pick(&mut self, model: u8, effort: Option<u8>, key: [u8; 16]) {
        let task = self.assignment().task;
        let revision = self.next_turn_task().revisions.spec;
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::default(),
            reply_to: ReplyTo::new(PICK),
            sign_in: self.party.sign_in.expect("authenticated picker"),
            key,
            ask: people::Ask::Amend {
                project: 1,
                task,
                revision,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: None,
                    llm: Some(people::LlmChoice {
                        account: None,
                        main: people::ModelEffort { model, effort },
                        subagents: Box::default(),
                    }),
                    reason: b"Change next run".as_slice().into(),
                },
            },
        }));
        for _ in 0..STEPS {
            self.tick();
            if self.next_turn_task().revisions.spec > revision && !self.committing() {
                return;
            }
        }
        panic!("picker amendment did not commit");
    }

    /// The next activation's script, then the person's words.
    pub fn say_again(&mut self, words: &[u8], script: Script) {
        let changing = self.process.answered.is_none() && self.next_turn_task().llm_changed;
        let assignments = self.observed.assignments;
        let requests = self.agent().prompts().len();
        if self.process.answered.is_some() {
            self.process.resume(script);
        } else {
            // The live fake keeps its script; a later process receives this extension.
            let mut turns = self.process.script.turns.to_vec();
            turns.extend(script.turns.iter().cloned());
            self.process.script.turns = turns.into_boxed_slice();
            self.process.fresh = Some(script);
        }
        self.act(parties::Ask::Say { task: parties::Task::Last, words: words.into(), release: None }, [44; 16]);
        if changing {
            for _ in 0..STEPS {
                self.tick();
                if self.observed.assignments > assignments {
                    return;
                }
                assert_eq!(self.agent().prompts().len(), requests, "the old run asks nothing more");
            }
            panic!("the next words did not start a new assignment");
        }
    }

    /// The durable chat record, including its try counters and choice revision.
    #[must_use]
    pub fn next_turn_task(&self) -> &tasks::TaskRecord {
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(self.assignment().task)));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) =
            self.store.rows.get(&key)
        else {
            panic!("the Open chat remains live")
        };
        task
    }

    /// Advance beyond this activation's shipped waiting time.
    pub fn past_listening(&mut self) {
        self.advance(self.assignment().run.policy.waiting.saturating_add(Duration::from_secs(1)));
    }
}
