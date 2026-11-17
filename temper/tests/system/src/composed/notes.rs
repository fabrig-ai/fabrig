//! Chat documents built in bounded calls and read through the engine's party face.
//! Contract: temper/docs/design/domain/agent/agents.md, 3;
//! jig/docs/design/domain/engine/briefs.md, 2.
use jig_domain_core::reads;
use smith_agent_world::Settings;

use super::{ReplyTo, Script, Token, World, authority, engine, open_config, people, worker};

const READ: Token = Token::new(950);
const ACT: Token = Token::new(951);
const PAYLOAD: u32 = 10_662;

/// The person has one read or keyed action outstanding at a time.
#[derive(Default)]
pub(super) struct Party {
    read: Option<reads::Answer>,
    answer: Option<people::Reply>,
}

impl Party {
    pub(super) fn read(&mut self, to: ReplyTo, answer: reads::Answer) {
        assert_eq!(to.into_token(), READ);
        assert!(self.read.replace(answer).is_none(), "one outstanding party read");
    }

    pub(super) fn answered(&mut self, to: Token, answer: &people::Reply) -> bool {
        if to != ACT {
            return false;
        }
        assert!(self.answer.replace(answer.clone()).is_none(), "one outstanding party act");
        true
    }
}

fn config(seed: u64) -> engine::Config {
    let mut config = open_config(seed);
    let charter = &mut config.core.defaults.charters[0];
    charter.policy.resume = false;
    charter.policy.model.output = 8192;
    let tools = authority::Tools(jig_domain_core::ToolFamilies::JIG.note.0);
    charter.policy.tools.0 = tools.0;
    config.core.settings.models = Box::new([charter.policy.model.clone()]);
    config.core.defaults.rules.ceiling.tools = tools;
    config.core.defaults.rules.ceiling.notes = authority::Scopes::GOAL;
    for project in &mut config.core.seed_projects {
        let policy = &mut project.policy.policy;
        policy.ceiling.tools = tools;
        policy.ceiling.notes = authority::Scopes::GOAL;
        for role in &mut policy.roles {
            role.authority.tools = tools;
            role.authority.notes = authority::Scopes::GOAL;
        }
    }
    config
}

fn limits(limits: &mut engine::Limits) {
    // The small profile's note bound stays; a stored whole body fits a load.
    limits.notes.body_bytes = 32_768;
    limits.loads.read_bytes = 65_536;
    limits.loads.bytes = 65_536;
    limits.loads.reply_bytes = 65_536;
    limits.people.amendment_bytes = 256;
    limits.people.requests = 8;
}

fn settings(settings: &mut Settings) {
    settings.output = 8192;
    settings.limits.decoded_call_bytes = u64::from(PAYLOAD);
    settings.limits.session.tools.tool_payload = PAYLOAD;
    settings.limits.session.answer_bytes = 65_536;
    settings.limits.session.completion_bytes = 65_536;
    settings.provider.script_bytes = 65_536;
    settings.provider.answer_bytes = 65_536;
    settings.provider.query_bytes = 131_072;
}

impl World {
    /// A requester-ended chat with the note-parts story's configuration and allowances.
    #[must_use]
    pub fn notes_chat(seed: u64, words: &[u8], script: Script) -> Self {
        let mut world = Self::configured_limits(seed, script, false, config, limits);
        let bounds = &mut world.worker_env.limits;
        bounds.bounds.endpoints[0].output = 8192;
        bounds.reply_bytes = 8192;
        bounds.host.event_bytes = 10_660;
        bounds.agent.message_bytes = u64::from(PAYLOAD);
        bounds.host.outcome_bytes = u64::from(PAYLOAD);
        bounds.agent.call_bytes = u64::from(PAYLOAD);
        bounds.checkout.message_bytes = PAYLOAD;
        assert!(worker::valid(bounds), "worker bounds: {:?}", worker::unfit(bounds));
        world.worker = worker::Domain::new(bounds, Box::new([]));
        world.party = super::signed_in_party(words);
        world.process.tool_payload = Some(PAYLOAD);
        world.process.configure_settings = Some(settings);
        world
    }

    /// The pane's authenticated party read, answered by the engine root and store.
    pub fn notes_read(&mut self, read: reads::Read) -> reads::Answer {
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(READ),
            sign_in: self.party.sign_in.expect("authenticated party"),
            read: Box::new(read),
        }));
        for _ in 0..10000 {
            self.tick();
            if let Some(answer) = self.notes.read.take() {
                return answer;
            }
        }
        panic!("party read did not answer");
    }

    /// A person's stop, amendment or release, using the same acts as ctl.
    pub fn notes_act(&mut self, ask: people::Ask, key: [u8; 16]) -> people::Reply {
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(ACT),
            sign_in: self.party.sign_in.expect("authenticated party"),
            key,
            ask,
        }));
        for _ in 0..10000 {
            self.tick();
            if let Some(answer) = self.notes.answer.take() {
                return answer;
            }
        }
        panic!("party act did not answer");
    }

    /// Prepare the fake model's next activation; the charter decides whether it resumes.
    pub fn notes_follow_up(&mut self, script: Script) {
        self.process.resume(script);
    }
}
