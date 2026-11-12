//! Long saved conversations cross the engine, worker decoding and Smith admission.
//! Contract: temper/docs/design/domain/agent/README.md, 6; jig/docs/design/domain/engine/briefs.md, 1.
use super::{World, engine, open_config, parties, people, tasks, text_script, worker};

const HISTORY_BYTES: u32 = 2 * 1_048_576;
const EXCHANGES: u8 = 22;
const DESCRIPTION: &[u8] = b"Write the description of the configuration we agreed.";
const ANSWER: &[u8] = b"The configuration description is ready.";
const INSTRUCTIONS: &[u8] = b"You configure temper for the person.";

fn config(seed: u64) -> engine::Config {
    let mut config = open_config(seed);
    config.core.defaults.charters[0].policy.instructions = INSTRUCTIONS.into();
    let mut owner = config.core.owners[0].clone();
    owner.project = people::DEPLOYMENT;
    config.core.owners = Box::new([config.core.owners[0].clone(), owner]);
    let settings = jig_domain_core::ToolFamilies::JIG.settings;
    config.core.defaults.charters[0].policy.tools.0 |= settings;
    config.core.defaults.rules.ceiling.tools.0 |= settings;
    config.core.seed_projects[0].policy.policy.ceiling.tools.0 |= settings;
    for role in &mut config.core.seed_projects[0].policy.policy.roles {
        role.authority.tools.0 |= settings;
    }
    config.core.settings.resume_bytes = HISTORY_BYTES;
    config
}

fn window_config(seed: u64) -> engine::Config {
    let mut config = config(seed);
    config.core.settings.resume_bytes = 32768;
    config
}

fn limits(limits: &mut engine::Limits) {
    limits.journal.transcript_bytes = HISTORY_BYTES;
    limits.journal.run_bytes = 8192;
    limits.loads.transcript_bytes = 65536;
    limits.loads.bytes = 262_144;
    limits.loads.read_bytes = 262_144;
    limits.loads.reply_bytes = 262_144;
    limits.people.requests = 64;
    limits.people.initial_owners = 2;
}

impl World {
    /// Replay 22 configuring exchanges beyond the sighting's byte count.
    /// Mode 0 bounds engine resume bytes, 1 bounds worker decoding, and 2
    /// bounds Smith history admission after restarting both roots.
    pub fn recover_long_chat(mode: u8) {
        // Repeated real activations retain over a MiB of valid turns: the
        // system tier joins history decoding, worker classification and recovery.
        let reply = vec![b'x'; 48 * 1024];
        let mut world = Self::long_chat(mode, &reply);
        world.run();
        for number in 1..EXCHANGES {
            let proposal = world.pending_configuration();
            world.process.resume(exchange(&reply, u32::from(number) + 1));
            world.act(
                parties::Ask::Decision { proposer: parties::Proposer::Task(1), proposal, accept: true },
                [number; 16],
            );
            world.run();
        }
        let bytes: usize = world.store.rows.values().filter_map(transcript_bytes).sum();
        assert!(bytes > 1_053_156, "valid saved turns cross the observed correlation: {bytes}");
        assert_eq!(world.observed.assignments, usize::from(EXCHANGES));
        assert_eq!(world.long_task().tries, tasks::Tries::NONE, "these claims are not lost");
        world.recover_description(mode);
    }

    fn long_chat(mode: u8, reply: &[u8]) -> Self {
        let mut world = Self::configured_limits(
            933,
            exchange(reply, 1),
            false,
            if mode == 0 { window_config } else { config },
            limits,
        );
        let bounds = &mut world.worker_env.limits;
        bounds.host.transcript_bytes = u64::from(HISTORY_BYTES);
        bounds.agent.transcript_bytes = u64::from(HISTORY_BYTES);
        bounds.reply_bytes = 8192;
        bounds.agent.call_bytes = 8192;
        bounds.checkout.message_bytes = 8192;
        bounds.host.turn_bytes = 65536;
        bounds.agent.turn_bytes = 65536;
        bounds.host.turn_queue_bytes = 131_072;
        bounds.agent.unacknowledged_bytes = 131_072;
        world.worker = worker::Domain::new(bounds, Box::new([]));
        world.process.history_messages = Some(257);
        world
    }

    fn recover_description(&mut self, mode: u8) {
        match mode {
            0 => {}
            1 => self.worker_env.limits.agent.transcript_bytes = 32768,
            2 => {
                self.restart_long_chat();
                self.process.history_messages = Some(8);
            }
            _ => panic!("unknown history boundary"),
        }
        let mut script = text_script(ANSWER);
        script.cue = INSTRUCTIONS.into();
        self.process.resume(script);
        self.act(parties::Ask::Say { task: parties::Task::Last, words: DESCRIPTION.into(), release: None }, [80; 16]);
        for _ in 0..20000 {
            self.tick();
            if self.process.answered.is_some_and(|(turns, _)| turns > 0)
                && self.observed.answers > usize::from(EXCHANGES)
            {
                self.run();
                self.assert_description(mode);
                return;
            }
        }
        panic!(
            "unusable history did not recover: {:?}; tries={:?}; phase={:?}",
            self.observed,
            self.long_task().tries,
            self.long_task().phase
        );
    }

    fn restart_long_chat(&mut self) {
        assert!(self.store.idle(), "restart between committed activations");
        self.root = engine::Domain::new(config(self.seed), &self.env.limits).expect("restart configuration");
        self.root_events = super::VecDeque::from([engine::Event::Restart { pending: false }]);
        self.protocol = super::Protocol::default();
        self.worker = worker::Domain::new(&self.worker_env.limits, Box::new([]));
        self.worker_events = super::VecDeque::from([worker::Event::Ready]);
        self.opened = false;
    }

    fn assert_description(&self, mode: u8) {
        assert!(self.assignment().transcript.is_empty(), "unusable history starts fresh, mode {mode}");
        assert_eq!(self.assignment().task, 1, "the same chat answers");
        assert!(self.prompt_contains(DESCRIPTION), "the pending person's words reach Smith");
        assert!(
            self.prompt_contains(b"xxxxxxxxxxxxxxxx"),
            "the fresh activation carries the transcript tail, mode {mode}"
        );
        assert!(
            self.agent().turns().iter().flat_map(|turn| &turn.messages).flat_map(|message| &message.content).any(
                |block| {
                    matches!(block, smith_domain_session::llm::Block::Text { text, .. } if text.as_ref() == ANSWER)
                }
            ),
            "Smith answers the description request"
        );
        assert!(self.long_task().tries.lost == 0 && self.long_task().tries.invalid == 0);
    }

    fn prompt_contains(&self, words: &[u8]) -> bool {
        self.agent().prompts().iter().any(|prompt| {
            prompt.system.windows(words.len()).any(|bytes| bytes == words)
                || prompt.messages.iter().flat_map(|message| &message.parts).any(|part| {
                    matches!(part, skein_fake_llm_domain::api::Part::Text { text }
                        if text.windows(words.len()).any(|bytes| bytes == words))
                })
        })
    }

    fn long_task(&self) -> &tasks::TaskRecord {
        let row = self.store.rows.get(&super::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(1))));
        let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) = row else {
            panic!("the configuring chat is live")
        };
        task
    }

    fn pending_configuration(&self) -> u64 {
        self.long_task().proposal.as_ref().expect("the configuring agent proposed settings").number
    }
}

fn transcript_bytes(row: &engine::Record) -> Option<usize> {
    match row {
        engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))) => {
            Some(turn.transcript.len())
        }
        engine::Record::Core(_) | engine::Record::Forge { .. } => None,
    }
}

fn exchange(reply: &[u8], against: u32) -> super::Script {
    use jig_domain_core::{SettingsAnswer, reads::SettingDocument, settings as s};
    let config = config(933);
    let mut charter = config.core.defaults.charters[0].clone();
    charter.policy.instructions =
        [INSTRUCTIONS, format!("\nExplain configuration revision {against}.").as_bytes()].concat().into_boxed_slice();
    let value = SettingsAnswer::Setting(SettingDocument {
        overlay: None,
        setting: s::Setting::Charter(1),
        revision: against,
        value: Some(s::Value::Charter(Box::new(charter))),
        application: None,
        proposed: None,
    });
    let text = temper_engine_smith::settings_text(&value, None, &config.core.defaults.models).expect("setting text");
    let (_, json) = std::str::from_utf8(&text).expect("setting text").split_once('\n').expect("setting heading");
    let value: serde_json::Value = serde_json::from_str(json.trim()).expect("setting JSON");
    let input = serde_json::json!({"against":against,"reason":"Keep the agreed configuration explicit.","items":[{"setting":{"kind":"charter","number":1},"set":value}]});
    let mut script = text_script(reply);
    script.cue = INSTRUCTIONS.into();
    let mut turns =
        vec![super::call(b"settings", b"{}"), super::call(b"propose_settings", input.to_string().as_bytes())];
    turns.extend(script.turns);
    script.turns = turns.into_boxed_slice();
    script
}
